// core/src/models/graph/party.rs
//! Party types

use serde::{Deserialize, Serialize};
use crate::models::common::{Identifier, LocalizedString};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    pub party_id: Vec<Identifier>,
    pub isni: Option<String>,
    pub ipi: Option<String>,
    pub party_name: Vec<LocalizedString>,
    pub party_role: Vec<PartyRole>,
    pub contact_details: Option<ContactDetails>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartyRole {
    Artist,
    Producer,
    Composer,
    Lyricist,
    Publisher,
    Performer,
    Engineer,
    Label,
    Distributor,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactDetails {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub party_reference: Option<String>,
    pub artist_role: Vec<String>,
    pub display_artist_name: Vec<LocalizedString>,
    pub sequence_number: Option<i32>,
}