use crate::utils::{error, try_from_element};
use hdk::prelude::*;
use std::collections::HashMap;
// use super::helpers::commit_receipts;

use super::{
    MessageAndReceiptTuple, P2PMessage, P2PMessageReceipt, ReceiptContents, ReceiptSignal, Signal,
    SignalDetails,
};

pub fn receive_receipt_handler(input: MessageAndReceiptTuple) -> ExternResult<ReceiptContents> {
    let receipt_message_hash = input.receipt.id[0].clone();
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
        let message_entry: P2PMessage = try_from_element(queried_message)?;
        let message_hash = hash_entry(&message_entry)?;

        if message_hash == receipt_message_hash {
            ();
        }
    }

    // debug!("queried message succeeds");
    let queried_receipts: Vec<Element> = query(
        QueryFilter::new()
            .entry_type(EntryType::App(AppEntryType::new(
                EntryDefIndex::from(1),
                zome_info()?.zome_id,
                EntryVisibility::Private,
            )))
            .include_entries(true),
    )?;
    let receipt_input_hash = hash_entry(&input.receipt)?;

    for queried_receipt in queried_receipts.clone().into_iter() {
        let receipt_entry: P2PMessageReceipt = try_from_element(queried_receipt)?;
        let receipt_hash = hash_entry(&receipt_entry)?;

        if receipt_hash == receipt_input_hash {
            return error("Duplicate receipt");
        }
    }

    let receipt_hash = create_entry(&input.receipt)?;

    let mut receipt_contents: HashMap<String, P2PMessageReceipt> = HashMap::new();
    receipt_contents.insert(receipt_hash.to_string(), input.receipt.clone());

    let signal = Signal::P2PMessageReceipt(ReceiptSignal {
        receipt: ReceiptContents(receipt_contents.clone()),
    });
    let signal_details = SignalDetails {
        name: "RECEIVE_P2P_RECEIPT".to_string(),
        payload: signal,
    };

    emit_signal(&signal_details)?;

    Ok(ReceiptContents(receipt_contents))
}
