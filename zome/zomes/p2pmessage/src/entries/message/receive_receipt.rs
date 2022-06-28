use hdk::prelude::*;
use std::collections::HashMap;

use super::{P2PMessageReceipt, ReceiptSignal, Signal, SignalDetails};

pub fn receive_receipt_handler(
    receipt: P2PMessageReceipt,
) -> ExternResult<HashMap<String, P2PMessageReceipt>> {
    let receipt_entry = Entry::App(receipt.clone().try_into()?);
    
    let receipt_hash = host_call::<CreateInput, ActionHash>(
        __create,
        CreateInput::new(
            EntryDefLocation::app(0),
            EntryVisibility::Private,
            receipt_entry,
            ChainTopOrdering::Relaxed,
        ),
    )?;

    let mut receipt_contents: HashMap<String, P2PMessageReceipt> = HashMap::new();
    receipt_contents.insert(receipt_hash.to_string(), receipt.clone());

    let signal = Signal::P2PMessageReceipt(ReceiptSignal {
        receipt: receipt_contents.clone(),
    });

    let signal_details = SignalDetails {
        name: "RECEIVE_P2P_RECEIPT".to_string(),
        payload: signal,
    };

    debug!("receive receipt emitting signal {:?} for agent {:}", signal_details.clone(), agent_info()?.agent_latest_pubkey);

    emit_signal(&signal_details)?;
    
    debug!("signal emitted");

    Ok(receipt_contents)
}
