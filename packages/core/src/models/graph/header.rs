// core/src/models/graph/header.rs
//! Message header types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::common::{Identifier, LocalizedString};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub message_id: String,
    pub message_type: MessageType,
    pub message_created_date_time: DateTime<Utc>,
    pub message_sender: MessageSender,
    pub message_recipient: MessageRecipient,
    pub message_control_type: Option<MessageControlType>,
    pub message_thread_id: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecipient {
    pub party_id: Vec<Identifier>,
    pub party_name: Vec<LocalizedString>,
    pub trading_name: Option<String>,
}