// core/src/models/graph/header.rs
//! Message header types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::{Extensions, Comment, AttributeMap, common::{Identifier, LocalizedString}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub message_id: String,
    pub message_type: MessageType,
    pub message_created_date_time: DateTime<Utc>,
    pub message_sender: MessageSender,
    pub message_recipient: MessageRecipient,
    pub message_control_type: Option<MessageControlType>,
    pub message_thread_id: Option<String>,
    /// All XML attributes (standard and custom)
    pub attributes: Option<AttributeMap>,
    /// Extensions for message header
    pub extensions: Option<Extensions>,
    /// Comments associated with message header
    pub comments: Option<Vec<Comment>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    NewReleaseMessage,
    UpdateReleaseMessage,
    TakedownMessage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageControlType {
    LiveMessage,
    TestMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSender {
    pub party_id: Vec<Identifier>,
    pub party_name: Vec<LocalizedString>,
    pub trading_name: Option<String>,
    /// All XML attributes (standard and custom)
    pub attributes: Option<AttributeMap>,
    /// Extensions for message sender
    pub extensions: Option<Extensions>,
    /// Comments associated with message sender
    pub comments: Option<Vec<Comment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecipient {
    pub party_id: Vec<Identifier>,
    pub party_name: Vec<LocalizedString>,
    pub trading_name: Option<String>,
    /// All XML attributes (standard and custom)
    pub attributes: Option<AttributeMap>,
    /// Extensions for message recipient
    pub extensions: Option<Extensions>,
    /// Comments associated with message recipient
    pub comments: Option<Vec<Comment>>,
}