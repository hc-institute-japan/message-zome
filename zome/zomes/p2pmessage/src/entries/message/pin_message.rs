use hdk::prelude::*;

use super::{P2PMessagePin, PinContents, PinMessageInput, PinStatus};

pub fn pin_message_handler(pin_message_input: PinMessageInput) -> ExternResult<PinContents> {
    let mut pinned_messages: HashMap<String, P2PMessagePin> = HashMap::new();

    let pin = P2PMessagePin {
        id: pin_message_input.message_hashes,
        status: if pin_message_input.status == "Pinned" {
            PinStatus::Pinned {
                timestamp: pin_message_input.timestamp,
            }
        } else {
            PinStatus::Unpinned {
                timestamp: pin_message_input.timestamp,
            }
        },
    };

    let pin_hash = create_entry(&pin)?;

    pinned_messages.insert(pin_hash.to_string(), pin.clone());

    let zome_call_response: ZomeCallResponse = call_remote(
        pin_message_input.conversant,
        zome_info()?.zome_name,
        FunctionName("sync_pins".into()),
        None,
        &pin,
    )?;

    match zome_call_response {
        ZomeCallResponse::Ok(extern_io) => {
            return Ok(extern_io.decode()?);
        }
        _ => return crate::error("we have an error trying to get the receive_read_receipt"),
    }
}
