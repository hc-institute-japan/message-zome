use hdk::prelude::*;

use super::{
    MessageDataAndReceipt, MessageSignal, P2PFileBytes, P2PMessage, P2PMessageData,
    P2PMessageReceipt, P2PMessageReplyTo, ReceiveMessageInput, Signal, SignalDetails, Status,
};

pub fn receive_message_handler(input: ReceiveMessageInput) -> ExternResult<P2PMessageReceipt> {
    // let receipt = P2PMessageReceipt::from_message(input.0.clone())?;
    let receipt = P2PMessageReceipt {
        id: vec![hash_entry(&input.0)?],
        status: Status::Delivered {
            timestamp: sys_time()?,
        },
    };
    let receipt_entry = Entry::App(receipt.clone().try_into()?);
    let message_entry = Entry::App(input.0.clone().try_into()?);
    host_call::<CreateInput, HeaderHash>(
        __create,
        CreateInput::new(
            P2PMessage::entry_def().id,
            message_entry,
            ChainTopOrdering::Relaxed,
        ),
    )?;
    host_call::<CreateInput, HeaderHash>(
        __create,
        CreateInput::new(
            P2PMessageReceipt::entry_def().id,
            receipt_entry,
            ChainTopOrdering::Relaxed,
        ),
    )?;

    if let Some(file) = input.1.clone() {
        let file_entry = Entry::App(file.clone().try_into()?);
        host_call::<CreateInput, HeaderHash>(
            __create,
            CreateInput::new(
                P2PFileBytes::entry_def().id,
                file_entry,
                ChainTopOrdering::Relaxed,
            ),
        )?;
    };

    let mut message_return;
    message_return = P2PMessageData {
        author: input.0.author.clone(),
        receiver: input.0.receiver.clone(),
        payload: input.0.payload.clone(),
        time_sent: input.0.time_sent.clone(),
        reply_to: None,
    };
    if let Some(ref reply_to_hash) = input.0.reply_to {
        let queried_messages: Vec<Element> = query(
            QueryFilter::new()
                .entry_type(EntryType::App(AppEntryType::new(
                    EntryDefIndex::from(0),
                    zome_info()?.zome_id,
                    EntryVisibility::Private,
                )))
                .include_entries(true),
        )?;
        for queried_message in queried_messages.clone().into_iter() {
            if let Ok(message_entry) = TryInto::<P2PMessage>::try_into(queried_message.clone()) {
                let message_hash = hash_entry(&message_entry)?;

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

    let signal = Signal::Message(MessageSignal {
        message: MessageDataAndReceipt(
            (hash_entry(&input.0.clone())?, message_return),
            (hash_entry(&receipt.clone())?, receipt.clone()),
        ),
    });

    let signal_details = SignalDetails {
        name: "RECEIVE_P2P_MESSAGE".to_string(),
        payload: signal,
    };
    emit_signal(&signal_details)?;

    Ok(receipt)
}
