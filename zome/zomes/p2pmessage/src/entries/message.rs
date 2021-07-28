use derive_more::{From, Into};
use hdk::prelude::{timestamp::Timestamp, *};
use std::collections::HashMap;

//pub mod handlers; MANUEL:this file contains a method never used maybe we want to erase the file

//this are the files for each method definition
pub mod get_adjacent_messages;
pub mod get_file_bytes;
pub mod get_latest_messages;
pub mod get_messages_by_agent_by_timestamp;
pub mod get_next_batch_messages;
pub mod get_next_messages;
pub mod get_pinned_messages;
pub mod helpers;
pub mod init;
pub mod pin_message;
pub mod read_message;
pub mod receive_message;
pub mod receive_read_receipt;
pub mod send_message;
pub mod send_message_with_timestamp;
pub mod sync_pins;
pub mod typing;

use file_types::{FileType, Payload, PayloadInput};

// ENTRY STRUCTURES
#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct P2PMessage {
    author: AgentPubKey,
    receiver: AgentPubKey,
    payload: Payload,
    time_sent: Timestamp,
    reply_to: Option<EntryHash>,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct P2PMessageReceipt {
    id: Vec<EntryHash>,
    status: Status,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct P2PMessagePin {
    id: Vec<EntryHash>,
    conversants: Vec<AgentPubKey>,
    status: PinStatus,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct P2PFileBytes(SerializedBytes);

// ENTRY DEFINITIONS
entry_def!(P2PMessage
    EntryDef {
        id: "p2pmessage".into(),
        visibility: EntryVisibility::Private,
        crdt_type: CrdtType,
        required_validations: RequiredValidations::default(),
        required_validation_type: RequiredValidationType::Element
    }
);

entry_def!(P2PFileBytes
    EntryDef {
        id: "p2pfilebytes".into(),
        visibility: EntryVisibility::Private,
        crdt_type: CrdtType,
        required_validations: RequiredValidations::default(),
        required_validation_type: RequiredValidationType::Element
    }
);

entry_def!(P2PMessageReceipt EntryDef {
    id: "p2pmessagereceipt".into(),
    visibility: EntryVisibility::Private,
    crdt_type: CrdtType,
    required_validations: RequiredValidations::default(),
    required_validation_type: RequiredValidationType::Element
});

entry_def!(P2PMessagePin EntryDef {
    id: "p2pmessagepin".into(),
    visibility: EntryVisibility::Private,
    crdt_type: CrdtType,
    required_validations: RequiredValidations::default(),
    required_validation_type: RequiredValidationType::Element
});

// ENTRY IMPLEMENTATIONS
impl P2PMessageReceipt {
    pub fn from_message(message: P2PMessage) -> ExternResult<Self> {
        let now = sys_time()?;
        let receipt = P2PMessageReceipt {
            id: vec![hash_entry(&message)?],
            status: Status::Delivered {
                timestamp: Timestamp(now.as_secs() as i64, now.subsec_nanos()),
            },
        };
        Ok(receipt)
    }
}

// INPUT STRUCTURES
#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct MessageInput {
    receiver: AgentPubKey,
    payload: PayloadInput,
    reply_to: Option<EntryHash>,
}

// test_stub: test structure for accepting timestamp as input in send_message
#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct MessageInputWithTimestamp {
    receiver: AgentPubKey,
    payload: PayloadInput,
    timestamp: Timestamp,
    reply_to: Option<EntryHash>,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct ReadMessageInput {
    message_hashes: Vec<EntryHash>,
    sender: AgentPubKey,
    timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct PinMessageInput {
    message_hashes: Vec<EntryHash>,
    conversants: Vec<AgentPubKey>,
    status: String,
    timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct ReceiveMessageInput(P2PMessage, Option<P2PFileBytes>);

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Status {
    Sent,
    Delivered { timestamp: Timestamp },
    Read { timestamp: Timestamp },
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
#[serde(tag = "pinstatus", rename_all = "camelCase")]
pub enum PinStatus {
    Pinned { timestamp: Timestamp },
    Unpinned { timestamp: Timestamp },
}

// GET FILTERS
#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct P2PMessageFilterAgentTimestamp {
    conversant: AgentPubKey,
    date: Timestamp,
    payload_type: String,
}

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct BatchSize(u8);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct P2PMessageFilterBatch {
    conversant: AgentPubKey,
    batch_size: u8,
    payload_type: String,
    last_fetched_timestamp: Option<Timestamp>, // header timestamp; oldest message in the last fetched message
    last_fetched_message_id: Option<EntryHash>,
}

// WRAPPER STRUCTURES
#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct BooleanWrapper(bool);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct MessageHash(EntryHash);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct ReceiptHash(EntryHash);

// GROUPING STRUCTURES
#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct MessageBundle(P2PMessageData, Vec<String>);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct MessageAndReceipt((EntryHash, P2PMessage), (EntryHash, P2PMessageReceipt));

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct MessageDataAndReceipt((EntryHash, P2PMessageData), (EntryHash, P2PMessageReceipt));

// OUTPUT STRUCTURES
#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
#[serde(rename_all = "camelCase")]
pub struct P2PMessageReplyTo {
    hash: EntryHash,
    author: AgentPubKey,
    receiver: AgentPubKey,
    payload: Payload,
    time_sent: Timestamp,
    reply_to: Option<EntryHash>,
}

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
#[serde(rename_all = "camelCase")]
pub struct P2PMessageData {
    author: AgentPubKey,
    receiver: AgentPubKey,
    payload: Payload,
    time_sent: Timestamp,
    reply_to: Option<P2PMessageReplyTo>,
}

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct AgentMessages(HashMap<String, Vec<String>>);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct MessageContents(HashMap<String, MessageBundle>);

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct ReceiptContents(HashMap<String, P2PMessageReceipt>);

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct FileContents(HashMap<String, P2PFileBytes>);

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct PinContents(HashMap<String, P2PMessagePin>);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes, Debug)]
pub struct P2PMessageHashTables(AgentMessages, MessageContents, ReceiptContents);

// SIGNAL STRUCTURES
#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
#[serde(tag = "type")]
pub enum Signal {
    Message(MessageSignal),
    P2PTypingDetailSignal(TypingSignal),
    P2PMessageReceipt(ReceiptSignal),
    P2PPinSignal(PinSignal),
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct SignalDetails {
    pub name: String,
    pub payload: Signal,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct MessageSignal {
    message: MessageDataAndReceipt,
}
#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct ReceiptSignal {
    receipt: ReceiptContents,
}
#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct PinSignal {
    pin: PinContents,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct TypingSignal {
    agent: AgentPubKey,
    is_typing: bool,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct P2PTypingDetailIO {
    agent: AgentPubKey,
    is_typing: bool,
}
