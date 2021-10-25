use hdk::prelude::*;
use std::collections::HashMap;

use super::helpers::{get_receipts, get_replies, insert_message, insert_reply};

use super::{
    FileType, P2PMessage, P2PMessageData, P2PMessageFilterBatch, P2PMessageHashTables,
    P2PMessageReceipt, Payload,
};

pub fn get_next_messages_handler(
    filter: P2PMessageFilterBatch,
) -> ExternResult<P2PMessageHashTables> {
    let mut queried_messages: Vec<Element> = query(
        QueryFilter::new()
            .entry_type(EntryType::App(AppEntryType::new(
                EntryDefIndex::from(0),
                zome_info()?.id,
                EntryVisibility::Private,
            )))
            .include_entries(true),
    )?;
    queried_messages.reverse();

    let mut agent_messages: HashMap<String, Vec<String>> = HashMap::new();
    agent_messages.insert(filter.conversant.clone().to_string(), Vec::new());
    let mut message_contents: HashMap<String, (P2PMessageData, Vec<String>)> = HashMap::new();
    let mut receipt_contents: HashMap<String, P2PMessageReceipt> = HashMap::new();
    let mut reply_pairs: HashMap<String, Vec<String>> = HashMap::new();
    let mut later_message_hashes: Vec<EntryHash> = Vec::new();
    let mut later_messages: Vec<P2PMessage> = Vec::new();

    let filter_timestamp = match filter.last_fetched_timestamp {
        Some(timestamp) => timestamp,
        None => sys_time()?,
    };

    for message in queried_messages.into_iter() {
        if let Ok(message_entry) = TryInto::<P2PMessage>::try_into(message.clone()) {
            let message_hash = hash_entry(&message_entry)?;

            if (message_entry.time_sent.as_micros() >= filter_timestamp.as_micros())
                && (match filter.last_fetched_message_id {
                    Some(ref id) if *id == message_hash => false,
                    Some(ref id) if *id != message_hash => true,
                    _ => false,
                } || filter.last_fetched_message_id == None)
                && (message_entry.author == filter.conversant
                    || message_entry.receiver == filter.conversant)
            {
                match message_entry.payload {
                    Payload::Text { .. } => {
                        if filter.payload_type == "Text" || filter.payload_type == "All" {
                            if message_entry.reply_to != None {
                                insert_reply(
                                    &mut reply_pairs,
                                    message_entry.clone(),
                                    message_hash.clone(),
                                );
                            }

                            later_message_hashes.push(message_hash);
                            later_messages.push(message_entry);
                        }
                    }
                    Payload::File { ref file_type, .. } => match file_type {
                        FileType::Image { .. } => {
                            if filter.payload_type == "Media"
                                || filter.payload_type == "File"
                                || filter.payload_type == "All"
                            {
                                if message_entry.reply_to != None {
                                    insert_reply(
                                        &mut reply_pairs,
                                        message_entry.clone(),
                                        message_hash.clone(),
                                    );
                                }

                                later_message_hashes.push(message_hash);
                                later_messages.push(message_entry);
                            }
                        }
                        FileType::Video { .. } => {
                            if filter.payload_type == "Media"
                                || filter.payload_type == "File"
                                || filter.payload_type == "All"
                            {
                                if message_entry.reply_to != None {
                                    insert_reply(
                                        &mut reply_pairs,
                                        message_entry.clone(),
                                        message_hash.clone(),
                                    );
                                }

                                later_message_hashes.push(message_hash);
                                later_messages.push(message_entry);
                            }
                        }
                        FileType::Other { .. } => {
                            if filter.payload_type == "Other"
                                || filter.payload_type == "File"
                                || filter.payload_type == "All"
                            {
                                if message_entry.reply_to != None {
                                    insert_reply(
                                        &mut reply_pairs,
                                        message_entry.clone(),
                                        message_hash.clone(),
                                    );
                                }

                                later_message_hashes.push(message_hash);
                                later_messages.push(message_entry);
                            }
                        }
                    },
                }
            }
        } else {
            continue;
        }
    }

    let mut start_index: usize = 0;
    if later_messages.len() > filter.batch_size.into() {
        start_index = later_messages.len() as usize - filter.batch_size as usize;
    }

    for index in start_index..later_messages.len() {
        insert_message(
            &mut agent_messages,
            &mut message_contents,
            later_messages[index].clone(),
            later_message_hashes[index].clone(),
            filter.conversant.clone(),
        )?;
    }

    get_receipts(&mut message_contents, &mut receipt_contents)?;

    get_replies(&mut reply_pairs, &mut message_contents)?;

    // Ok(P2PMessageHashTables(
    //     AgentMessages(agent_messages),
    //     MessageContents(message_contents),
    //     ReceiptContents(receipt_contents),
    // ))

    Ok(P2PMessageHashTables(
        agent_messages,
        message_contents,
        receipt_contents,
    ))
}
