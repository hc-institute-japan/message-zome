use hdk::prelude::*;
// use std::collections::HashMap;

use super::helpers::commit_receipts;

use super::{P2PMessageReceipt, ReceiptContents, ReceiptSignal, Signal, SignalDetails};

pub fn receive_read_receipt_handler(receipt: P2PMessageReceipt) -> ExternResult<ReceiptContents> {
    let receipts = commit_receipts(vec![receipt.clone()])?;

    // let mut receipt_contents: HashMap<String, P2PMessageReceipt> = HashMap::new();
    // receipt_contents.insert(hash_entry(&receipt.clone())?.to_string(), receipt.clone());

    let signal = Signal::P2PMessageReceipt(ReceiptSignal {
        receipt: receipts.clone(),
    });

    let signal_details = SignalDetails {
        name: "RECEIVE_P2P_RECEIPT".to_string(),
        payload: signal,
    };

    emit_signal(&signal_details)?;

    Ok(receipts)
}
