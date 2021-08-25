use hdk::prelude::*;

mod entries;
mod utils;
use entries::message::{self};

use message::*;

use message::get_adjacent_messages::get_adjacent_messages_handler;
use message::get_file_bytes::get_file_bytes_handler;
use message::get_latest_messages::get_latest_messages_handler;
use message::get_messages_by_agent_by_timestamp::get_messages_by_agent_by_timestamp_handler;
use message::get_next_batch_messages::get_next_batch_messages_handler;
use message::get_next_messages::get_next_messages_handler;
use message::get_pinned_messages::get_pinned_messages_handler;
use message::init::init_handler;
use message::pin_message::pin_message_handler;
use message::read_message_call_remote::read_message_call_remote_handler;
use message::read_message_remote_signal::read_message_remote_signal_handler;
use message::receive_message::receive_message_handler;
use message::receive_read_receipt::receive_read_receipt_handler;
use message::receive_receipt::receive_receipt_handler;
use message::send_message_call_remote::send_message_call_remote_handler;
use message::send_message_remote_signal::send_message_remote_signal_handler;
use message::send_message_with_timestamp::send_message_with_timestamp_handler;
use message::sync_pins::sync_pins_handler;
use message::typing::typing_handler;

entry_defs![
    P2PMessage::entry_def(),
    P2PMessageReceipt::entry_def(),
    P2PFileBytes::entry_def(),
    P2PMessagePin::entry_def()
];

pub fn error<T>(reason: &str) -> ExternResult<T> {
    Err(WasmError::Guest(String::from(reason)))
}

pub fn err<T>(code: &str, message: &str) -> ExternResult<T> {
    Err(WasmError::Guest(format!(
        "{{\"code\": \"{}\", \"message\": \"{}\"}}",
        code, message
    )))
}

#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    return init_handler();
}

#[hdk_extern]
fn recv_remote_signal(signal: ExternIO) -> ExternResult<()> {
    let signal_detail: SignalDetails = signal.decode()?;

    let signal_name: &str = &signal_detail.name;

    match signal_name {
        "P2P_REMOTE_MESSAGE" => {
            if let Signal::P2PReceiveMessage(RemoteMessageSignal { input }) = signal_detail.payload
            {
                match receive_message_handler(input.clone()) {
                    Ok(MessageDataAndReceipt(message, receipt)) => {
                        debug!("receive message succeeds");
                        let signal = Signal::Message(MessageSignal {
                            message: MessageDataAndReceipt(
                                (message.0, message.1),
                                (receipt.0, receipt.1),
                            ),
                        });

                        let signal_details = SignalDetails {
                            name: "RECEIVE_P2P_MESSAGE".to_string(),
                            payload: signal,
                        };
                        emit_signal(&signal_details)?;
                    }
                    _ => {
                        debug!("nicko p2p receive message failed");
                        let signal = Signal::P2PRetryMessage(RetryMessageSignal { input: input });
                        let signal_details = SignalDetails {
                            name: "P2P_RETRY_RECEIVE_MESSAGE".to_string(),
                            payload: signal,
                        };
                        emit_signal(&signal_details)?;
                        ()
                    }
                };
            }
        }
        "P2P_REMOTE_DELIVERED_RECEIPT" => {
            if let Signal::P2PReceiveReceipt(RemoteReceiptSignal { receipt, message }) =
                signal_detail.payload
            {
                match receive_receipt_handler(MessageAndReceiptTuple {
                    message: message.clone(),
                    receipt: receipt.clone(),
                }) {
                    Ok(_) => (),
                    _ => {
                        debug!("nicko p2p receive receipt failed");
                        let signal =
                            Signal::P2PRetryReceipt(RetryReceiptSignal { receipt: receipt });
                        let signal_details = SignalDetails {
                            name: "P2P_RETRY_DELIVERED_RECEIPT".to_string(),
                            payload: signal,
                        };
                        emit_signal(&signal_details)?;
                        ()
                    }
                };
            }
        }
        "P2P_REMOTE_READ_RECEIPT" => {
            if let Signal::P2PReceiveReadReceipt(RemoteReadReceiptSignal { receipt }) =
                signal_detail.payload
            {
                match receive_read_receipt_handler(receipt.clone()) {
                    Ok(_) => (),
                    _ => {
                        debug!("nicko p2p receive receipt failed");
                        let signal =
                            Signal::P2PRetryReceipt(RetryReceiptSignal { receipt: receipt });
                        let signal_details = SignalDetails {
                            name: "P2P_RETRY_READ_RECEIPT".to_string(),
                            payload: signal,
                        };
                        emit_signal(&signal_details)?;
                        ()
                    }
                };
            }
        }
        "TYPING_P2P" => emit_signal(&signal_detail)?,
        _ => debug!("unknown signal"),
    };

    Ok(())
}

// call_remote set
#[hdk_extern]
fn send_message_call_remote(message_input: MessageInput) -> ExternResult<MessageDataAndReceipt> {
    return send_message_call_remote_handler(message_input);
}
#[hdk_extern]
fn read_message_call_remote(read_message_input: ReadMessageInput) -> ExternResult<ReceiptContents> {
    return read_message_call_remote_handler(read_message_input);
}
#[hdk_extern]
fn send_message_with_timestamp(
    message_input: MessageInputWithTimestamp,
) -> ExternResult<MessageDataAndReceipt> {
    return send_message_with_timestamp_handler(message_input);
}
// end of call_remote set

// remote_signal set
#[hdk_extern]
fn send_message(message_input: MessageInput) -> ExternResult<MessageDataAndReceipt> {
    let timestamp_string: String = match message_input.timestamp {
        Some(timestamp) => format!("[{:?}, {:?}]", timestamp.0, timestamp.1).to_string(),
        None => "".to_string(),
    };

    match send_message_remote_signal_handler(message_input) {
        Ok(MessageDataAndReceipt(message_tuple, receipt_tuple)) => {
            debug!("send message succeeded {:?}", message_tuple.1.clone());
            return Ok(MessageDataAndReceipt(message_tuple, receipt_tuple));
        }
        _ => {
            debug!("sending timestamp back");
            return err("TODO: 000", &timestamp_string);
        }
    }
    // validation runs after this extern call
}

#[hdk_extern]
fn read_message(read_message_input: ReadMessageInput) -> ExternResult<ReceiptContents> {
    return read_message_remote_signal_handler(read_message_input);
}
// end of remote_signal set

#[hdk_extern]
fn receive_read_receipt(receipt: P2PMessageReceipt) -> ExternResult<ReceiptContents> {
    return receive_read_receipt_handler(receipt);
}

#[hdk_extern]
fn pin_message(pin_message_input: PinMessageInput) -> ExternResult<PinContents> {
    return pin_message_handler(pin_message_input);
}

#[hdk_extern]
fn sync_pins(pin: P2PMessagePin) -> ExternResult<PinContents> {
    return sync_pins_handler(pin);
}

#[hdk_extern]
fn receive_message(input: ReceiveMessageInput) -> ExternResult<MessageDataAndReceipt> {
    return receive_message_handler(input);
}

#[hdk_extern]
fn get_latest_messages(batch_size: BatchSize) -> ExternResult<P2PMessageHashTables> {
    return get_latest_messages_handler(batch_size);
}

#[hdk_extern]
fn get_next_batch_messages(filter: P2PMessageFilterBatch) -> ExternResult<P2PMessageHashTables> {
    return get_next_batch_messages_handler(filter);
}

#[hdk_extern]
fn get_messages_by_agent_by_timestamp(
    filter: P2PMessageFilterAgentTimestamp,
) -> ExternResult<P2PMessageHashTables> {
    return get_messages_by_agent_by_timestamp_handler(filter);
}

#[hdk_extern]
fn typing(typing_info: P2PTypingDetailIO) -> ExternResult<()> {
    return typing_handler(typing_info);
}

#[hdk_extern]
fn get_file_bytes(file_hashes: Vec<EntryHash>) -> ExternResult<FileContents> {
    return get_file_bytes_handler(file_hashes);
}

#[hdk_extern]
fn get_pinned_messages(conversant: AgentPubKey) -> ExternResult<P2PMessageHashTables> {
    return get_pinned_messages_handler(conversant);
}

#[hdk_extern]
fn get_next_messages(filter: P2PMessageFilterBatch) -> ExternResult<P2PMessageHashTables> {
    return get_next_messages_handler(filter);
}

#[hdk_extern]
fn get_adjacent_messages(filter: P2PMessageFilterBatch) -> ExternResult<P2PMessageHashTables> {
    return get_adjacent_messages_handler(filter);
}
