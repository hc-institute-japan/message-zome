use hdk::prelude::*;
use std::collections::HashMap;

use super::{
    P2PMessageReceipt, ReadMessageInput, ReceiptContents, RemoteReceiptSignal, Signal,
    SignalDetails, Status,
};

pub fn read_message_remote_signal_handler(
    read_message_input: ReadMessageInput,
) -> ExternResult<ReceiptContents> {
    let receipt = P2PMessageReceipt {
        id: read_message_input.message_hashes,
        status: Status::Read {
            timestamp: read_message_input.timestamp,
        },
    };

    debug!("nicko read {:?}", receipt.clone());

    let receipt_hash = create_entry(&receipt)?;

    let mut receipt_contents: HashMap<String, P2PMessageReceipt> = HashMap::new();
    receipt_contents.insert(receipt_hash.to_string(), receipt.clone());

    debug!("nicko read hashmap {:?}", receipt_contents.clone());

    let signal_payload = Signal::P2PReceiveReceipt(RemoteReceiptSignal { receipt: receipt });

    let signal = SignalDetails {
        name: "P2P_REMOTE_READ_RECEIPT".to_string(),
        payload: signal_payload,
    };

    remote_signal(
        ExternIO::encode(signal)?,
        vec![read_message_input.sender.clone()],
    )?;

    debug!("nicko read remote_signal done");

    Ok(ReceiptContents(receipt_contents))
}
