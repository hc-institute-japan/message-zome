use hdk::prelude::*;

use crate::utils::try_from_element;
use file_types::{FileMetadata, Payload, PayloadInput};

use super::{
    MessageDataAndReceipt, MessageInput, P2PFileBytes, P2PMessage, P2PMessageData,
    P2PMessageReceipt, P2PMessageReplyTo, ReceiveMessageInput, RemoteMessageSignal, Signal,
    SignalDetails,
};

pub fn send_message_remote_signal_handler(
    message_input: MessageInput,
) -> ExternResult<MessageDataAndReceipt> {
    debug!("nicko send_message_signal is called");
    let now = sys_time()?;

    let message = P2PMessage {
        author: agent_info()?.agent_latest_pubkey,
        receiver: message_input.receiver,
        payload: match message_input.payload {
            PayloadInput::Text { ref payload } => Payload::Text {
                payload: payload.to_owned(),
            },
            PayloadInput::File {
                ref metadata,
                ref file_type,
                ref file_bytes,
            } => {
                let p2pfile = P2PFileBytes(file_bytes.clone());
                create_entry(&p2pfile)?;
                let file_hash = hash_entry(&p2pfile)?;
                Payload::File {
                    metadata: FileMetadata {
                        file_name: metadata.file_name.clone(),
                        file_size: metadata.file_size.clone(),
                        file_type: metadata.file_type.clone(),
                        file_hash: file_hash,
                    },
                    file_type: file_type.clone(),
                }
            }
        },
        time_sent: Timestamp(now.as_secs() as i64, now.subsec_nanos()),
        reply_to: message_input.reply_to,
    };

    let file = match message_input.payload {
        PayloadInput::Text { .. } => None,
        PayloadInput::File { file_bytes, .. } => Some(P2PFileBytes(file_bytes)),
    };

    // TODO: remove sent/saved receipt
    // TODO: this is delivered receipt
    let sent_receipt = P2PMessageReceipt::from_message(message.clone())?;

    create_entry(&message)?;
    create_entry(&sent_receipt)?;

    let receive_input = ReceiveMessageInput(message.clone(), file.clone());

    let signal_payload = Signal::P2PReceiveMessage(RemoteMessageSignal {
        input: receive_input,
    });

    let signal = SignalDetails {
        name: "P2P_REMOTE_MESSAGE".to_string(),
        payload: signal_payload,
    };

    remote_signal(ExternIO::encode(signal)?, vec![message.receiver.clone()])?;

    debug!("nicko send_message_signal remote_signal done");

    let queried_messages: Vec<Element> = query(
        QueryFilter::new()
            .entry_type(EntryType::App(AppEntryType::new(
                EntryDefIndex::from(0),
                zome_info()?.zome_id,
                EntryVisibility::Private,
            )))
            .include_entries(true),
    )?;

    let message_return;
    for queried_message in queried_messages.clone().into_iter() {
        let message_entry: P2PMessage = try_from_element(queried_message)?;
        let message_hash = hash_entry(&message_entry)?;

        if let Some(ref reply_to_hash) = message.reply_to {
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
                    author: message.author.clone(),
                    receiver: message.receiver.clone(),
                    payload: message.payload.clone(),
                    time_sent: message.time_sent.clone(),
                    reply_to: Some(replied_to_message),
                };

                return Ok(MessageDataAndReceipt(
                    (hash_entry(&message)?, message_return),
                    (hash_entry(&sent_receipt)?, sent_receipt),
                ));
            }
        }
    }

    message_return = P2PMessageData {
        author: message.author.clone(),
        receiver: message.receiver.clone(),
        payload: message.payload.clone(),
        time_sent: message.time_sent.clone(),
        reply_to: None,
    };

    Ok(MessageDataAndReceipt(
        (hash_entry(&message)?, message_return),
        (hash_entry(&sent_receipt)?, sent_receipt),
    ))
}
