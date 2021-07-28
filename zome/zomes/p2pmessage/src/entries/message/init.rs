use hdk::prelude::*;

/*
 * ZOME INIT FUNCTION TO SET UNRESTRICTED ACCESS
 */

pub fn init_handler() -> ExternResult<InitCallbackResult> {
    let zome_name: ZomeName = zome_info()?.zome_name;

    let mut receive_message_function: GrantedFunctions = HashSet::new();
    receive_message_function.insert((zome_name.clone(), "receive_message".into()));

    let mut receive_receipt_function = HashSet::new();
    receive_receipt_function.insert((zome_name.clone(), "receive_read_receipt".into()));

    // let mut typing_function: GrantedFunctions = HashSet::new();
    // typing_function.insert((zome_name.clone(), "typing".into()));

    let mut recv_remote_signal_function: GrantedFunctions = HashSet::new();
    recv_remote_signal_function.insert((zome_name.clone(), "recv_remote_signal".into()));

    let mut sync_pins_function: GrantedFunctions = HashSet::new();
    sync_pins_function.insert((zome_name.clone(), "sync_pins".into()));

    create_cap_grant(CapGrantEntry {
        tag: "receive_message".into(),
        access: CapAccess::Unrestricted,
        functions: receive_message_function,
    })?;

    create_cap_grant(CapGrantEntry {
        tag: "receive_read_receipt".into(),
        access: CapAccess::Unrestricted,
        functions: receive_receipt_function,
    })?;

    // create_cap_grant(CapGrantEntry {
    //     tag: "typing".into(),
    //     access: CapAccess::Unrestricted,
    //     functions: typing_function,
    // })?;

    create_cap_grant(CapGrantEntry {
        tag: "recv_remote_signal".into(),
        access: CapAccess::Unrestricted,
        functions: recv_remote_signal_function,
    })?;

    create_cap_grant(CapGrantEntry {
        tag: "sync_pins".into(),
        access: CapAccess::Unrestricted,
        functions: sync_pins_function,
    })?;

    Ok(InitCallbackResult::Pass)
}
