use hdk::prelude::*;

use crate::utils::try_from_element;
use file_types::{FileMetadata, Payload, PayloadInput};

use super::{
    MessageDataAndReceipt, MessageInputWithTimestamp, P2PFileBytes, P2PMessage, P2PMessageData,
    P2PMessageReceipt, P2PMessageReplyTo, ReceiveMessageInput,
};
use crate::utils::error;

// test_stub: this zome function is a test function to test get by timestamp
pub fn send_message_with_timestamp_handler(
    message_input: MessageInputWithTimestamp,
) -> ExternResult<MessageDataAndReceipt> {
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
        time_sent: message_input.timestamp,
        reply_to: message_input.reply_to,
    };

    let receipt = P2PMessageReceipt::from_message(message.clone(), "Sent")?;
    create_entry(&message)?;
    create_entry(&receipt)?;

    let file = match message_input.payload {
        PayloadInput::Text { .. } => None,
        PayloadInput::File { file_bytes, .. } => Some(P2PFileBytes(file_bytes)),
    };

    // create message input to receive function of recipient
    let receive_input = ReceiveMessageInput(message.clone(), file.clone());

    let receive_call_result: ZomeCallResponse = call_remote(
        message.receiver.clone(),
        zome_info()?.zome_name,
        "receive_message".into(),
        None,
        &receive_input,
    )?;

    match receive_call_result {
        ZomeCallResponse::Ok(extern_io) => {
            let receipt: P2PMessageReceipt = extern_io.decode()?;
            create_entry(&receipt)?;

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
                            (hash_entry(&receipt)?, receipt),
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
                (hash_entry(&receipt)?, receipt),
            ))
        }
        // This case shouldn't happen because of unrestricted access to receive message
        // keeping it here for exhaustive matching
        ZomeCallResponse::Unauthorized(_, _, _, _) => {
            return error("Sorry, something went wrong. [Authorization error]");
        }
        // Error that might happen when
        ZomeCallResponse::NetworkError(e) => {
            return error(&e);
        }
    }
}
