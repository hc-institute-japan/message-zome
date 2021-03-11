use super::*;

use crate::timestamp::Timestamp;
use crate::utils::try_from_element;

// use hdk3::prelude::*;

use std::collections::HashMap;

/* TODO:
 * - proper error codes
 * - sending messages to self
 */

/*
 * ZOME FUNCTIONS ARE UNRESTRICTED BY DEFAULT
 * USERS OF THIS ZOME COULD IMPLEMENT
 * A WAY TO SET AND GET CAPABILITY GRANTS AND CLAIMS FOR CALL_REMOTE
 * TO SET SELECTED ACCESS TO ZOME FUNCTIONS
 */

/*
 * ZOME INIT FUNCTION TO SET UNRESTRICTED ACCESS
 */
#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    let mut receive_functions: GrantedFunctions = HashSet::new();
    receive_functions.insert((zome_info()?.zome_name, "receive_message".into()));
    let mut notify_functions: GrantedFunctions = HashSet::new();
    notify_functions.insert((zome_info()?.zome_name, "notify_delivery".into()));

    create_cap_grant(CapGrantEntry {
        tag: "receive".into(),
        access: ().into(),
        functions: receive_functions,
    })?;

    let mut emit_functions: GrantedFunctions = HashSet::new();
    emit_functions.insert((zome_info()?.zome_name, "is_typing".into()));

    create_cap_grant(CapGrantEntry {
        tag: "notify".into(),
        access: ().into(),
        functions: notify_functions,
    })?;

    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        access: ().into(),
        functions: emit_functions,
    })?;

    //
    let mut fuctions = HashSet::new();

    // TODO: name may be changed to better suit the context of cap grant.s
    let tag: String = "create_group_cap_grant".into();
    let access: CapAccess = CapAccess::Unrestricted;

    let zome_name: ZomeName = zome_info()?.zome_name;
    let function_name: FunctionName = FunctionName("recv_remote_signal".into());

    fuctions.insert((zome_name, function_name));

    let cap_grant_entry: CapGrantEntry = CapGrantEntry::new(
        tag,    // A string by which to later query for saved grants.
        access, // Unrestricted access means any external agent can call the extern
        fuctions,
    );

    create_cap_grant(cap_grant_entry)?;

    let mut receive_receipt_function: GrantedFunctions = HashSet::new();
    receive_receipt_function.insert((zome_info()?.zome_name, "receive_read_receipt".into()));

    create_cap_grant(CapGrantEntry {
        tag: "receipt".into(),
        access: ().into(),
        functions: receive_receipt_function,
    })?;

    Ok(InitCallbackResult::Pass)
}

pub(crate) fn send_message(message_input: MessageInput) -> ExternResult<MessageAndReceipt> {
    // TODO: check if receiver is blocked

    let message = P2PMessage::from_input(message_input.clone())?;

    let file = match message_input.payload {
        PayloadInput::File { .. } => Some(P2PFileBytes::from_input(message_input)?),
        _ => None,
    };

    let receive_input = ReceiveMessageInput(message.clone(), file.clone());

    let receive_call_result: Result<P2PMessageReceipt, HdkError> = call_remote(
        message.receiver.clone(),
        zome_info()?.zome_name,
        "receive_message".into(),
        None,
        &receive_input,
    );

    match receive_call_result {
        Ok(receive_output) => {
            let receipt = receive_output;
            create_entry(&message)?;
            create_entry(&receipt)?;
            if let Some(file) = file {
                create_entry(&file)?;
            };
            // TODO: CREATE AND RETURN ELEMENT HERE
            Ok(MessageAndReceipt(message, receipt))
        }
        Err(kind) => {
            match kind {
                // TIMEOUT; RECIPIENT IS OFFLINE; MESSAGE NEEDS TO BE SENT ASYNC
                // WILL BE IMPLEMENTED ONCE EPHEMERAL STORAGE IS IN PLACE
                HdkError::ZomeCallNetworkError(_err) => {
                    crate::err("TODO: 000", "Unknown other error")
                }
                HdkError::UnauthorizedZomeCall(_c, _z, _f, _p) => crate::err(
                    "TODO: 000:",
                    "This case shouldn't happen because of unrestricted access to receive message",
                ),
                _ => crate::err("TODO: 000", "Unknown other error"),
            }
        }
    }
}

pub(crate) fn receive_message(input: ReceiveMessageInput) -> ExternResult<P2PMessageReceipt> {
    let receipt = P2PMessageReceipt::from_message(input.0.clone())?;
    create_entry(&input.0)?;
    create_entry(&receipt)?;
    if let Some(file) = input.1 {
        create_entry(&file)?;
    };
    Ok(receipt)
}

pub(crate) fn get_latest_messages(batch_size: BatchSize) -> ExternResult<P2PMessageHashTables> {
    let queried_messages = query(
        QueryFilter::new()
            .entry_type(EntryType::App(AppEntryType::new(
                EntryDefIndex::from(0),
                zome_info()?.zome_id,
                EntryVisibility::Private,
            )))
            .include_entries(true),
    )?;

    let mut agent_messages: HashMap<String, Vec<String>> = HashMap::new();
    let mut message_contents: HashMap<String, MessageBundle> = HashMap::new();
    let mut receipt_contents: HashMap<String, P2PMessageReceipt> = HashMap::new();

    for message in queried_messages.0.into_iter() {
        let message_entry: P2PMessage = try_from_element(message)?;
        let message_hash = hash_entry(&message_entry)?;
        if message_entry.author.clone() == agent_info()?.agent_latest_pubkey {
            // match agent_messages.get(&format!("{:?}", message_entry.receiver.clone())) {
            match agent_messages.get(&message_entry.receiver.clone().to_string()) {
                Some(messages) if messages.len() >= batch_size.0.into() => continue,
                Some(messages) if messages.len() < batch_size.0.into() => {
                    insert_message(
                        &mut agent_messages,
                        &mut message_contents,
                        message_entry.clone(),
                        message_hash,
                        message_entry.receiver.clone(),
                    )?;
                }
                _ => {
                    insert_message(
                        &mut agent_messages,
                        &mut message_contents,
                        message_entry.clone(),
                        message_hash,
                        message_entry.receiver.clone(),
                    )?;
                }
            }
        } else {
            // add this message to author's array in hashmap
            // match agent_messages.get(&format!("{:?}", message_entry.author.clone())) {
            match agent_messages.get(&message_entry.author.clone().to_string()) {
                Some(messages) if messages.len() >= batch_size.0.into() => continue,
                Some(messages) if messages.len() < batch_size.0.into() => {
                    insert_message(
                        &mut agent_messages,
                        &mut message_contents,
                        message_entry.clone(),
                        message_hash,
                        message_entry.author.clone(),
                    )?;
                }
                _ => {
                    insert_message(
                        &mut agent_messages,
                        &mut message_contents,
                        message_entry.clone(),
                        message_hash,
                        message_entry.author.clone(),
                    )?;
                }
            }
        }
    }

    get_receipts(&mut message_contents, &mut receipt_contents)?;

    Ok(P2PMessageHashTables(
        AgentMessages(agent_messages),
        MessageContents(message_contents),
        ReceiptContents(receipt_contents),
    ))
}

pub(crate) fn get_next_batch_messages(
    filter: P2PMessageFilterBatch,
) -> ExternResult<P2PMessageHashTables> {
    let queried_messages = query(
        QueryFilter::new()
            .entry_type(EntryType::App(AppEntryType::new(
                EntryDefIndex::from(0),
                zome_info()?.zome_id,
                EntryVisibility::Private,
            )))
            .include_entries(true),
    )?;

    let mut agent_messages: HashMap<String, Vec<String>> = HashMap::new();
    // agent_messages.insert(format!("{:?}", filter.conversant.clone()), Vec::new());
    agent_messages.insert(filter.conversant.clone().to_string(), Vec::new());
    let mut message_contents: HashMap<String, MessageBundle> = HashMap::new();
    let mut receipt_contents: HashMap<String, P2PMessageReceipt> = HashMap::new();

    let filter_timestamp = match filter.last_fetched_timestamp {
        Some(timestamp) => timestamp,
        None => {
            let now = sys_time()?;
            Timestamp(now.as_secs() as i64 / 84600, 0)
        }
    };

    for message in queried_messages.0.into_iter() {
        let message_entry: P2PMessage = try_from_element(message)?;
        let message_hash = hash_entry(&message_entry)?;

        if message_entry.time_sent.0 <= filter_timestamp.0
            && (match filter.last_fetched_message_id {
                Some(ref id) if *id == message_hash => false,
                Some(ref id) if *id != message_hash => true,
                _ => false,
            })
            || filter.last_fetched_message_id == None
                && (message_entry.author == filter.conversant
                    || message_entry.receiver == filter.conversant)
        {
            match message_entry.payload {
                Payload::Text { .. } => {
                    if filter.payload_type == "Text" || filter.payload_type == "All" {
                        let current_batch_size = insert_message(
                            &mut agent_messages,
                            &mut message_contents,
                            message_entry,
                            message_hash,
                            filter.conversant.clone(),
                        )?;

                        if current_batch_size >= filter.batch_size.into() {
                            break;
                        }
                    }
                }
                Payload::File { .. } => {
                    if filter.payload_type == "File" || filter.payload_type == "All" {
                        let current_batch_size = insert_message(
                            &mut agent_messages,
                            &mut message_contents,
                            message_entry,
                            message_hash,
                            filter.conversant.clone(),
                        )?;

                        if current_batch_size >= filter.batch_size.into() {
                            break;
                        }
                    }
                }
            }
        }
    }

    get_receipts(&mut message_contents, &mut receipt_contents)?;

    Ok(P2PMessageHashTables(
        AgentMessages(agent_messages),
        MessageContents(message_contents),
        ReceiptContents(receipt_contents),
    ))
}

pub(crate) fn get_messages_by_agent_by_timestamp(
    filter: P2PMessageFilterAgentTimestamp,
) -> ExternResult<P2PMessageHashTables> {
    let queried_messages = query(
        QueryFilter::new()
            .entry_type(EntryType::App(AppEntryType::new(
                EntryDefIndex::from(0),
                zome_info()?.zome_id,
                EntryVisibility::Private,
            )))
            .include_entries(true),
    )?;

    let mut agent_messages: HashMap<String, Vec<String>> = HashMap::new();
    agent_messages.insert(filter.conversant.clone().to_string(), Vec::new());
    let mut message_contents: HashMap<String, MessageBundle> = HashMap::new();
    let mut receipt_contents: HashMap<String, P2PMessageReceipt> = HashMap::new();

    let day_start = (filter.date.0 / 86400) * 86400;
    let day_end = day_start + 86399;

    for message in queried_messages.0.into_iter() {
        let message_entry: P2PMessage = try_from_element(message)?;
        let message_hash = hash_entry(&message_entry)?;

        // TODO: use header timestamp for message_time
        if message_entry.time_sent.0 >= day_start
            && message_entry.time_sent.0 <= day_end
            && (message_entry.author == filter.conversant
                || message_entry.receiver == filter.conversant)
        {
            match message_entry.payload {
                Payload::Text { .. } => {
                    if filter.payload_type == "Text" || filter.payload_type == "All" {
                        insert_message(
                            &mut agent_messages,
                            &mut message_contents,
                            message_entry,
                            message_hash,
                            filter.conversant.clone(),
                        )?;
                    }
                }
                Payload::File { .. } => {
                    if filter.payload_type == "File" || filter.payload_type == "All" {
                        insert_message(
                            &mut agent_messages,
                            &mut message_contents,
                            message_entry,
                            message_hash,
                            filter.conversant.clone(),
                        )?;
                    }
                }
            }
        }
    }

    get_receipts(&mut message_contents, &mut receipt_contents)?;

    Ok(P2PMessageHashTables(
        AgentMessages(agent_messages),
        MessageContents(message_contents),
        ReceiptContents(receipt_contents),
    ))
}

fn get_receipts(
    message_contents: &mut HashMap<String, MessageBundle>,
    receipt_contents: &mut HashMap<String, P2PMessageReceipt>,
) -> ExternResult<()> {
    let queried_receipts = query(
        QueryFilter::new()
            .entry_type(EntryType::App(AppEntryType::new(
                EntryDefIndex::from(1),
                zome_info()?.zome_id,
                EntryVisibility::Private,
            )))
            .include_entries(true),
    )?;

    for receipt in queried_receipts.clone().0.into_iter() {
        let receipt_entry: P2PMessageReceipt = try_from_element(receipt)?;
        let receipt_hash = hash_entry(&receipt_entry)?;
        // if message_contents.contains_key(&format!("{:?}", &receipt_entry.id)) {
        if message_contents.contains_key(&receipt_entry.id.to_string()) {
            receipt_contents.insert(receipt_hash.clone().to_string(), receipt_entry.clone());
            if let Some(message_bundle) =
                // message_contents.get_mut(&format!("{:?}", &receipt_entry.id))
                message_contents.get_mut(&receipt_entry.id.to_string())
            {
                // message_bundle.1.push(format!("{:?}", receipt_hash))
                message_bundle.1.push(receipt_hash.to_string())
            };
        }
    }

    Ok(())
}

fn insert_message(
    agent_messages: &mut HashMap<String, Vec<String>>,
    message_contents: &mut HashMap<String, MessageBundle>,
    message_entry: P2PMessage,
    message_hash: EntryHash,
    key: AgentPubKey,
) -> ExternResult<usize> {
    let mut message_array_length = 0;
    // match agent_messages.get_mut(&format!("{:?}", key)) {
    match agent_messages.get_mut(&key.to_string()) {
        Some(messages) => {
            // messages.push(format!("{:?}", message_hash.clone()));
            messages.push(message_hash.clone().to_string());
            message_array_length = messages.len();
        }
        None => {
            agent_messages.insert(
                // format!("{:?}", key),
                key.to_string(),
                vec![message_hash.clone().to_string()],
            );
        }
    };
    message_contents.insert(
        // format!("{:?}", message_hash),
        message_hash.to_string(),
        MessageBundle(message_entry, Vec::new()),
    );

    Ok(message_array_length)
}

// fn is_user_blocked(agent_pubkey: AgentPubKey) -> ExternResult<bool> {
//     match call::<AgentPubKey, BooleanWrapper>(
//         None,
//         "contacts".into(),
//         "in_blocked".into(),
//         None,
//         &agent_pubkey.clone()
//     ) {
//         Ok(output) => Ok(output.0),
//         _ => return crate::error("{\"code\": \"401\", \"message\": \"This agent has no proper authorization\"}")
//     }

//     let block_result: Result<BooleanWrapper, HdkError> = call_remote(
//         message_input.clone().receiver,
//         "contacts".into(),
//         "in_blocked".into(),
//         None,
//         &agent_pubkey
//     );

//     match block_result {
//         Ok(receive_output) => {
//             let message_entry = P2PMessage::from_parameter(receive_output.clone());
//             create_entry(&message_entry)?;
//             Ok(receive_output)
//         },
//         Err(kind) => {
//             match kind {
//                 // TIMEOUT; RECIPIENT IS OFFLINE; MESSAGE NEEDS TO BE SENT ASYNC
//                 HdkError::ZomeCallNetworkError(_err) => {
//                     match send_message_async(message_input) {
//                         Ok(async_result) => {
//                             let message_entry = P2PMessage::from_parameter(async_result.clone());
//                             create_entry(&message_entry)?;
//                             Ok(async_result)
//                         },
//                         _ => crate::err("TODO: 000", "This agent has no proper authorization")
//                     }
//                 },
//                 HdkError::UnauthorizedZomeCall(_c,_z,_f,_p) => crate::err("TODO: 000:", "This case shouldn't happen because of unrestricted access to receive message"),
//                 _ => crate::err("TODO: 000", "Unknown other error")
//             }
//         }
//     }
// }

pub(crate) fn typing(typing_info: P2PTypingDetailIO) -> ExternResult<()> {
    let payload = Signal::P2PTypingDetailSignal(P2PTypingDetailIO {
        agent: agent_info()?.agent_latest_pubkey,
        is_typing: typing_info.is_typing,
    });

    let mut agents = Vec::new();

    agents.push(typing_info.agent);
    agents.push(agent_info()?.agent_latest_pubkey);

    remote_signal(&payload, agents)?;
    Ok(())
}

pub(crate) fn read_message(read_receipt_input: ReadReceiptInput) -> ExternResult<ReceiptContents> {
    create_entry(&read_receipt_input.receipt)?;
    call_remote(
        read_receipt_input.sender,
        zome_info()?.zome_name,
        FunctionName("receive_read_receipt".into()),
        None,
        &read_receipt_input.receipt,
    )
}

pub(crate) fn receive_read_receipt(receipt: P2PMessageReceipt) -> ExternResult<ReceiptContents> {
    let receipts = commit_receipts(vec![receipt])?;
    emit_signal(Signal::P2PMessageReceipt(receipts.clone()))?;
    Ok(receipts)
}

fn commit_receipts(receipts: Vec<P2PMessageReceipt>) -> ExternResult<ReceiptContents> {
    // Query all the receipts
    let query_result = query(
        QueryFilter::new()
            .entry_type(EntryType::App(AppEntryType::new(
                EntryDefIndex::from(1),
                zome_info()?.zome_id,
                EntryVisibility::Private,
            )))
            .include_entries(true),
    )?;

    // Get all receipts from query result
    let all_receipts = query_result
        .0
        .into_iter()
        .filter_map(|el| {
            if let Ok(Some(receipt)) = el.into_inner().1.to_app_option::<P2PMessageReceipt>() {
                return Some(receipt);
            } else {
                None
            }
        })
        .collect::<Vec<P2PMessageReceipt>>();

    // initialize hash map that will be returned
    let mut receipts_hash_map: HashMap<String, P2PMessageReceipt> = HashMap::new();

    // Iterate through the receipts in the argument and push them into the hash map
    receipts.clone().into_iter().for_each(|receipt| {
        // receipts_hash_map.insert(format!("{:?}", receipt.id), receipt);
        receipts_hash_map.insert(receipt.id.to_string(), receipt);
    });

    // Iterate through the receipts to check if the receipt has been committed, remove them from the hash map if it is
    // used for loops instead of for_each because you cant break iterators
    for i in 0..all_receipts.len() {
        let receipt = all_receipts[i].clone();
        // let hash = format!("{:?}", receipt.id);
        let hash = receipt.id;

        if receipts_hash_map.contains_key(&hash.to_string()) {
            if let Status::Read { timestamp: _ } = receipt.status {
                receipts_hash_map.remove(&hash.to_string());
            }
        }

        if receipts_hash_map.is_empty() {
            break;
        }
    }

    // iterate the remaining contents of the hashmap
    receipts_hash_map
        .clone()
        .into_iter()
        .for_each(|(_entry_hash, receipt)| {
            create_entry(&receipt).expect("Expected P2P message receipt entry");
        });

    Ok(ReceiptContents(receipts_hash_map))
}
