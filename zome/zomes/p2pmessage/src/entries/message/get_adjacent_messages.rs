use hdk::prelude::*;
use std::collections::HashMap;

use super::helpers::{get_receipts, get_replies, insert_message, insert_reply};
use crate::utils::try_from_element;
// use num_traits::pow::Pow;

use super::{
    AgentMessages, FileType, MessageBundle, MessageContents, P2PMessage, P2PMessageFilterBatch,
    P2PMessageHashTables, P2PMessageReceipt, Payload, ReceiptContents,
};

pub fn get_adjacent_messages_handler(
    filter: P2PMessageFilterBatch,
) -> ExternResult<P2PMessageHashTables> {
    debug!("entered get_next_messages");
    let queried_messages: Vec<Element> = query(
        QueryFilter::new()
            .entry_type(EntryType::App(AppEntryType::new(
                EntryDefIndex::from(0),
                zome_info()?.zome_id,
                EntryVisibility::Private,
            )))
            .include_entries(true),
    )?;
    debug!("query success");

    let mut agent_messages: HashMap<String, Vec<String>> = HashMap::new();
    // agent_messages.insert(format!("{:?}", filter.conversant.clone()), Vec::new());
    agent_messages.insert(filter.conversant.clone().to_string(), Vec::new());
    let mut message_contents: HashMap<String, MessageBundle> = HashMap::new();
    let mut receipt_contents: HashMap<String, P2PMessageReceipt> = HashMap::new();
    let mut reply_pairs: HashMap<String, Vec<String>> = HashMap::new();
    let mut later_message_hashes: Vec<EntryHash> = Vec::new();
    let mut later_messages: Vec<P2PMessage> = Vec::new();

    debug!("filter {:?}", filter.clone());

    let filter_timestamp = match filter.last_fetched_timestamp {
        Some(timestamp) => timestamp,
        None => {
            let now = sys_time()?;
            Timestamp(now.as_secs() as i64, 0)
        }
    };
    debug!("filter timestamp {:?}", filter_timestamp.0);

    for message in queried_messages.into_iter() {
        let message_entry: P2PMessage = try_from_element(message)?;
        let message_hash = hash_entry(&message_entry)?;

        debug!("iterated timestamp {:?}", message_entry.time_sent.0.clone());
        debug!(
            "message > filter {:?}",
            message_entry.time_sent.0.clone() >= filter_timestamp.0
        );

        if (match filter.last_fetched_message_id {
            Some(ref id) if *id == message_hash => false,
            Some(ref id) if *id != message_hash => true,
            _ => false,
        } || filter.last_fetched_message_id == None)
            && (message_entry.author == filter.conversant
                || message_entry.receiver == filter.conversant)
        {
            if message_entry.time_sent.0 >= filter_timestamp.0 {
                debug!("timestmap is greater");
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
            } else {
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
                    },
                }
            }
        }
    }

    debug!("iterated through messages");
    let mut start_index: usize = 0;
    if later_messages.len() > filter.batch_size.into() {
        start_index = later_messages.len() as usize - filter.batch_size as usize;
    }
    debug!("start index is {:?}", start_index);
    debug!("later messages {:?}", later_messages);
    debug!("later message hashes {:?}", later_message_hashes);

    debug!("sliced");
    for index in start_index..later_messages.len() {
        debug!("{:?}", index);
        insert_message(
            &mut agent_messages,
            &mut message_contents,
            later_messages[index].clone(),
            later_message_hashes[index].clone(),
            filter.conversant.clone(),
        )?;
    }
    debug!("iterated through splice");

    get_receipts(&mut message_contents, &mut receipt_contents)?;

    debug!("got receipts");
    get_replies(&mut reply_pairs, &mut message_contents)?;
    debug!("got replies");
    Ok(P2PMessageHashTables(
        AgentMessages(agent_messages),
        MessageContents(message_contents),
        ReceiptContents(receipt_contents),
    ))
}
