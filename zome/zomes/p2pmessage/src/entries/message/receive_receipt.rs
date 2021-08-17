use hdk::prelude::*;
use std::collections::HashMap;

// use super::helpers::commit_receipts;

use super::{P2PMessageReceipt, ReceiptContents, ReceiptSignal, Signal, SignalDetails};

pub fn receive_receipt_handler(receipt: P2PMessageReceipt) -> ExternResult<ReceiptContents> {
    let receipt_hash = create_entry(&receipt)?;

    let mut receipt_contents: HashMap<String, P2PMessageReceipt> = HashMap::new();
    receipt_contents.insert(receipt_hash.to_string(), receipt.clone());

    let signal = Signal::P2PMessageReceipt(ReceiptSignal {
        receipt: ReceiptContents(receipt_contents.clone()),
    });

    let signal_details = SignalDetails {
        name: "RECEIVE_DELIVERED_RECEIPT".to_string(),
        payload: signal,
    };

    emit_signal(&signal_details)?;

    Ok(ReceiptContents(receipt_contents))
}
