use hdk::prelude::*;

use file_types::{FileMetadata, Payload, PayloadInput};

use super::{
    MessageInputWithTimestamp, MessageReceipt, P2PFileBytes, P2PMessage, P2PMessageReceipt,
    ReceiveMessageInput,
};
use crate::utils::error;

pub fn send_message_2_handler(
    message_input: MessageInputWithTimestamp,
) -> ExternResult<MessageReceipt> {
    let message = P2PMessage {
        // consider querying from source chain instead of accepting input
        // to avoid making UI as a source of data integrity
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
                create_entry(&p2pfile)?; // TODO: remove this
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

    let file = match message_input.payload {
        PayloadInput::Text { .. } => None,
        PayloadInput::File { file_bytes, .. } => Some(P2PFileBytes(file_bytes)),
    };

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
            let received_receipt: P2PMessageReceipt = extern_io.decode()?;

            Ok(MessageReceipt(
                hash_entry(&received_receipt)?,
                received_receipt,
            ))
        }
        ZomeCallResponse::Unauthorized(_, _, _, _) => {
            return error("Sorry, something went wrong. [Authorization error]");
        }
        ZomeCallResponse::NetworkError(_e) => {
            return error("Sorry, something went wrong. [Network error]");
        }
    }
}
