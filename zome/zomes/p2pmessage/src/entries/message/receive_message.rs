use super::{
    MessageDataAndReceipt, P2PMessage, P2PMessageData, P2PMessageReceipt, P2PMessageReplyTo,
    ReceiveMessageInput, RemoteReceiptSignal, Signal, SignalDetails,
};
use crate::utils::error;
use crate::utils::try_from_element;
use hdk::prelude::*;

pub fn receive_message_handler(input: ReceiveMessageInput) -> ExternResult<MessageDataAndReceipt> {
    let queried_messages: Vec<Element> = query(
        QueryFilter::new()
            .entry_type(EntryType::App(AppEntryType::new(
                EntryDefIndex::from(0),
                zome_info()?.zome_id,
                EntryVisibility::Private,
            )))
            .include_entries(true),
    )?;
    let message_input_hash = hash_entry(&input.0)?;

    for queried_message in queried_messages.clone().into_iter() {
        let message_entry: P2PMessage = try_from_element(queried_message)?;
        let message_hash = hash_entry(&message_entry)?;

        if message_hash == message_input_hash {
            return error("Duplicate message");
        }
    }

    let receipt = P2PMessageReceipt::from_message(input.0.clone(), "Delivered")?;
    create_entry(&input.0)?;
    create_entry(&receipt)?;
    if let Some(file) = input.1 {
        create_entry(&file)?;
    };

    let queried_messages: Vec<Element> = query(
        QueryFilter::new()
            .entry_type(EntryType::App(AppEntryType::new(
                EntryDefIndex::from(0),
                zome_info()?.zome_id,
                EntryVisibility::Private,
            )))
            .include_entries(true),
    )?;

    let mut message_return = P2PMessageData {
        author: input.0.author.clone(),
        receiver: input.0.receiver.clone(),
        payload: input.0.payload.clone(),
        time_sent: input.0.time_sent.clone(),
        reply_to: None,
    };

    if input.0.reply_to != None {
        for queried_message in queried_messages.clone().into_iter() {
            let message_entry: P2PMessage = try_from_element(queried_message)?;
            let message_hash = hash_entry(&message_entry)?;

            if let Some(ref reply_to_hash) = input.0.reply_to {
                if *reply_to_hash == message_hash {
                    let replied_to_message = P2PMessageReplyTo {
                        hash: message_hash.clone(),
                        author: message_entry.author,
                        receiver: message_entry.receiver,
                        payload: message_entry.payload,
                        time_sent: message_entry.time_sent,
                        reply_to: None,
                    };

                    message_return = P2PMessageData {
                        author: input.0.author.clone(),
                        receiver: input.0.receiver.clone(),
                        payload: input.0.payload.clone(),
                        time_sent: input.0.time_sent.clone(),
                        reply_to: Some(replied_to_message),
                    };
                }
            }
        }
    }

    let signal_payload = Signal::P2PReceiveReceipt(RemoteReceiptSignal {
        receipt: receipt.clone(),
        message: input.0.clone(),
    });

    let signal = SignalDetails {
        name: "P2P_REMOTE_DELIVERED_RECEIPT".to_string(),
        payload: signal_payload,
    };

    remote_signal(ExternIO::encode(signal)?, vec![input.0.author.clone()])?;

    Ok(MessageDataAndReceipt(
        (hash_entry(&input.0)?, message_return),
        (hash_entry(&receipt)?, receipt),
    ))
}
