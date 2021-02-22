use derive_more::{From, Into};
use hdk3::prelude::{timestamp::Timestamp, *};
pub mod handlers;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct P2PMessage {
    author: AgentPubKey,
    receiver: AgentPubKey,
    payload: Payload,
    time_sent: Timestamp,
    reply_to: Option<EntryHash>,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct P2PMessageReceipt {
    id: EntryHash,
    status: Status,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct P2PFileBytes(SerializedBytes);

entry_def!(P2PMessage
    EntryDef {
        id: "p2pmessage".into(),
        visibility: EntryVisibility::Private,
        crdt_type: CrdtType,
        required_validations: RequiredValidations::default(),
        required_validation_type: RequiredValidationType::Element
    }
);

entry_def!(P2PMessageReceipt
    EntryDef {
        id: "p2pmessagereceipt".into(),
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

impl P2PMessage {
    pub fn from_input(input: MessageInput) -> ExternResult<Self> {
        let now = sys_time()?;

        let message = P2PMessage {
            author: agent_info()?.agent_latest_pubkey,
            receiver: input.receiver,
            payload: match input.payload {
                PayloadInput::Text { payload } => Payload::Text { payload },
                PayloadInput::File {
                    file_name,
                    file_size,
                    file_type,
                    file_hash,
                    ..
                } => Payload::File {
                    metadata: FileMetadata {
                        file_name: file_name,
                        file_size: file_size,
                        file_type: file_type.clone(),
                        file_hash: file_hash,
                    },
                    file_type: file_type,
                },
            },
            time_sent: Timestamp(now.as_secs() as i64, now.subsec_nanos()),
            reply_to: input.reply_to,
        };
        Ok(message)
    }
}

impl P2PMessageReceipt {
    pub fn from_message(message: P2PMessage) -> ExternResult<Self> {
        let now = sys_time()?;
        let receipt = P2PMessageReceipt {
            id: hash_entry(&message)?,
            status: Status::Delivered {
                timestamp: Timestamp(now.as_secs() as i64, now.subsec_nanos()),
            },
        };
        Ok(receipt)
    }
}

impl P2PFileBytes {
    pub fn from_input(input: MessageInput) -> ExternResult<Self> {
        match input.payload {
            PayloadInput::Text { .. } => crate::err("TODO: 000", "no file bytes in input"),
            PayloadInput::File { bytes, .. } => Ok(P2PFileBytes(bytes)),
        }
    }
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct MessageInput {
    receiver: AgentPubKey,
    payload: PayloadInput,
    reply_to: Option<EntryHash>,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub enum PayloadInput {
    Text {
        payload: String,
    },
    File {
        file_name: String,
        file_size: u8,
        file_type: FileType,
        file_hash: String,
        bytes: SerializedBytes,
    },
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct ReceiveMessageInput(P2PMessage, Option<P2PFileBytes>);

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct FileMetadata {
    file_name: String,
    file_size: u8,
    file_type: FileType,
    file_hash: String,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub enum FileType {
    Image { thumbnail: SerializedBytes },
    Video { thumbnail: SerializedBytes },
    Others,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub enum Payload {
    Text {
        payload: String,
    },
    File {
        metadata: FileMetadata,
        file_type: FileType,
    },
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub enum Status {
    Sent,
    Delivered { timestamp: Timestamp },
    Read { timestamp: Timestamp },
}

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes)]
pub struct BooleanWrapper(bool);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes)]
pub struct ReceiptHash(EntryHash);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes)]
pub struct MessageHash(EntryHash);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes)]
pub struct MessageBundle(P2PMessage, Vec<String>);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes)]
pub struct MessageAndReceipt(P2PMessage, P2PMessageReceipt);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes)]
pub struct AgentMessages(HashMap<String, Vec<String>>);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes)]
pub struct MessageContents(HashMap<String, MessageBundle>);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes)]
pub struct ReceiptContents(HashMap<String, P2PMessageReceipt>);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes)]
pub struct P2PMessageHashTables(AgentMessages, MessageContents, ReceiptContents);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes)]
pub struct P2PMessageFilterAgentTimestamp {
    conversant: AgentPubKey,
    date: Timestamp,
    payload_type: String,
}

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes)]
pub struct BatchSize(u8);

#[derive(From, Into, Serialize, Deserialize, Clone, SerializedBytes)]
pub struct P2PMessageFilterBatch {
    conversant: AgentPubKey,
    batch_size: u8,
    payload_type: String,
    last_fetched_timestamp: Option<Timestamp>, // header timestamp; oldest message in the last fetched message
    last_fetched_message_id: Option<EntryHash>,
}

// TYPING
#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct P2PTypingDetailIO {
    agent: AgentPubKey,
    is_typing: bool,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct TypingSignal {
    kind: String,
    agent: AgentPubKey,
    is_typing: bool,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub struct MessageSignal {
    kind: String,
    message: P2PMessage,
}

#[derive(Serialize, Deserialize, SerializedBytes, Clone, Debug)]
pub enum Signal {
    Message(MessageSignal),
    P2PTypingDetailSignal(TypingSignal),
}

pub struct SignalTypes;
impl SignalTypes {
    pub const P2P_TYPING_SIGNAL: &'static str = "P2P_TYPING_SIGNAL";
}
