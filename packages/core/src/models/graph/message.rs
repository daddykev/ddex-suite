// core/src/models/graph/message.rs
//! ERN Message types

use serde::{Deserialize, Serialize};
use crate::models::versions::ERNVersion;
use super::{MessageHeader, Party, Resource, Release, Deal};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ERNMessage {
    pub message_header: MessageHeader,
    pub parties: Vec<Party>,
    pub resources: Vec<Resource>,
    pub releases: Vec<Release>,
    pub deals: Vec<Deal>,
    pub version: ERNVersion,
    pub profile: Option<ERNProfile>,
    pub message_audit_trail: Option<MessageAuditTrail>,
    pub extensions: Option<std::collections::HashMap<String, String>>,
    pub comments: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ERNProfile {
    AudioAlbum,
    AudioSingle,
    Video,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAuditTrail {
    pub audit_trail_events: Vec<AuditTrailEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEvent {
    pub message_audit_trail_event_reference: String,
    pub message_audit_trail_event_type: String,
    pub date_time: chrono::DateTime<chrono::Utc>,
    pub responsible_party_reference: Option<String>,
}

impl ERNMessage {
    pub fn to_build_request(&self) -> Self {
        self.clone()
    }
}