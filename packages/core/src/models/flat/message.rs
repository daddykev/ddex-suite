// core/src/models/flat/message.rs
//! Flattened message types

use super::{ParsedDeal, ParsedRelease, ParsedResource};
use crate::models::{
    graph::{ERNMessage, Party},
    Extensions,
};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedERNMessage {
    pub graph: ERNMessage,
    pub flat: FlattenedMessage,
    /// Extensions from the original XML that need preservation
    pub extensions: Option<Extensions>,
}

impl ParsedERNMessage {
    pub fn releases(&self) -> &[ParsedRelease] {
        &self.flat.releases
    }

    pub fn resources(&self) -> &IndexMap<String, ParsedResource> {
        &self.flat.resources
    }

    pub fn deals(&self) -> &[ParsedDeal] {
        &self.flat.deals
    }

    pub fn parties(&self) -> &IndexMap<String, Party> {
        &self.flat.parties
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlattenedMessage {
    pub message_id: String,
    pub message_type: String,
    pub message_date: DateTime<Utc>,
    pub sender: Organization,
    pub recipient: Organization,
    pub releases: Vec<ParsedRelease>,
    pub resources: IndexMap<String, ParsedResource>,
    pub deals: Vec<ParsedDeal>,
    pub parties: IndexMap<String, Party>,
    pub version: String,
    pub profile: Option<String>,
    pub stats: MessageStats,
    /// Extensions for flattened message
    pub extensions: Option<Extensions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// `PartyName/FullName`; optional per the XSD (`None` when absent).
    pub name: Option<String>,
    pub id: String,
    /// Extensions for organization
    pub extensions: Option<Extensions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    pub release_count: usize,
    pub track_count: usize,
    pub deal_count: usize,
    pub total_duration: u64,
}
