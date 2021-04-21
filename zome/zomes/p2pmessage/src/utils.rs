use hdk::prelude::{GetOptions, *};

pub fn _try_get_and_convert<T: TryFrom<SerializedBytes>>(
    entry_hash: EntryHash,
) -> ExternResult<(EntryHash, T)> {
    let option = GetOptions::latest();
    match get(entry_hash.clone(), option)? {
        Some(element) => Ok((entry_hash, try_from_element(element)?)),
        None => crate::error("Entry not found"),
    }
}

pub fn try_from_element<T: TryFrom<SerializedBytes>>(element: Element) -> ExternResult<T> {
    match element.entry() {
        element::ElementEntry::Present(entry) => try_from_entry::<T>(entry.clone()),
        _ => crate::error("Could not convert element"),
    }
}

pub fn try_from_entry<T: TryFrom<SerializedBytes>>(entry: Entry) -> ExternResult<T> {
    match entry {
        Entry::App(content) => match T::try_from(content.into_sb()) {
            Ok(e) => Ok(e),
            Err(_) => crate::error("Could not convert entry"),
        },
        _ => crate::error("Could not convert entry"),
    }
}

pub fn _address_deduper(agent_vec: Vec<AgentPubKey>) -> Vec<AgentPubKey> {
    let mut ids = agent_vec;
    ids.sort();
    ids.dedup_by(|a, b| a == b);
    return ids;
}
