// core/src/models/graph/release.rs
//! Release types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::common::{Identifier, LocalizedString};
use super::Artist;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub release_reference: String,
    pub release_id: Vec<Identifier>,
    pub release_title: Vec<LocalizedString>,
    pub release_subtitle: Option<Vec<LocalizedString>>,
    pub release_type: Option<ReleaseType>,
    pub genre: Vec<Genre>,
    pub release_resource_reference_list: Vec<ReleaseResourceReference>,
    pub display_artist: Vec<Artist>,
    pub party_list: Vec<ReleaseParty>,
    pub release_date: Vec<ReleaseEvent>,
    pub territory_code: Vec<String>,
    pub excluded_territory_code: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseType {
    Album,
    Single,
    EP,
    Compilation,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub genre_text: String,
    pub sub_genre: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseResourceReference {
    pub resource_reference: String,
    pub sequence_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub track_number: Option<i32>,
    pub side: Option<String>,
    pub is_hidden: bool,
    pub is_bonus: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseParty {
    pub party_reference: String,
    pub role: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseEvent {
    pub release_event_type: String,
    pub event_date: Option<DateTime<Utc>>,
    pub territory: Option<String>,
}