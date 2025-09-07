// core/src/models/flat/release.rs
//! Parsed release types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::common::{LocalizedString, Copyright};
use super::{ParsedTrack, ParsedImage, ParsedVideo, TerritoryInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRelease {
    pub release_id: String,
    pub identifiers: ReleaseIdentifiers,
    pub title: Vec<LocalizedString>,
    pub default_title: String,
    pub subtitle: Option<Vec<LocalizedString>>,
    pub default_subtitle: Option<String>,
    pub display_artist: String,
    pub artists: Vec<ArtistInfo>,
    pub release_type: String,
    pub genre: Option<String>,
    pub sub_genre: Option<String>,
    pub tracks: Vec<ParsedTrack>,
    pub track_count: usize,
    pub disc_count: Option<usize>,
    pub videos: Vec<ParsedVideo>,
    pub images: Vec<ParsedImage>,
    pub cover_art: Option<ParsedImage>,
    pub release_date: Option<DateTime<Utc>>,
    pub original_release_date: Option<DateTime<Utc>>,
    pub territories: Vec<TerritoryInfo>,
    pub p_line: Option<Copyright>,
    pub c_line: Option<Copyright>,
    pub parent_release: Option<String>,
    pub child_releases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseIdentifiers {
    pub upc: Option<String>,
    pub ean: Option<String>,
    pub catalog_number: Option<String>,
    pub grid: Option<String>,
    pub proprietary: Vec<ProprietaryId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProprietaryId {
    pub namespace: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistInfo {
    pub name: String,
    pub role: String,
    pub party_id: Option<String>,
}