// core/src/models/flat/message.rs
//! Flattened message types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use super::{ParsedRelease, ParsedResource, ParsedDeal};
use crate::models::graph::{Party, ERNMessage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedERNMessage {
    pub graph: ERNMessage,
    pub flat: FlattenedMessage,
}

impl ParsedERNMessage {
    pub fn releases(&self) -> &[ParsedRelease] {
        &self.flat.releases
    }
    
    pub fn resources(&self) -> &HashMap<String, ParsedResource> {
        &self.flat.resources
    }
    
    pub fn deals(&self) -> &[ParsedDeal] {
        &self.flat.deals
    }
    
    pub fn parties(&self) -> &HashMap<String, Party> {
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
    pub resources: HashMap<String, ParsedResource>,
    pub deals: Vec<ParsedDeal>,
    pub parties: HashMap<String, Party>,
    pub version: String,
    pub profile: Option<String>,
    pub stats: MessageStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    pub release_count: usize,
    pub track_count: usize,
    pub deal_count: usize,
    pub total_duration: u64,
}