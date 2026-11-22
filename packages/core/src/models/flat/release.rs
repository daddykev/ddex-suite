// core/src/models/flat/release.rs
//! Parsed release types

use super::{ParsedImage, ParsedTrack, ParsedVideo, TerritoryInfo};
use crate::models::{
    common::{Copyright, LocalizedString, TerritoryScope},
    Extensions,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRelease {
    pub release_id: String,
    pub identifiers: ReleaseIdentifiers,
    pub title: Vec<LocalizedString>,
    /// First release title (`ReferenceTitle` in 3.8.x, `DisplayTitleText` in 4.x); `None` when absent.
    pub default_title: Option<String>,
    pub subtitle: Option<Vec<LocalizedString>>,
    pub default_subtitle: Option<String>,
    /// Display artist; `None` when the source has none.
    pub display_artist: Option<String>,
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
    /// Extensions for parsed release
    pub extensions: Option<Extensions>,
    pub p_line: Option<Copyright>,
    pub c_line: Option<Copyright>,
    pub parent_release: Option<String>,
    pub child_releases: Vec<String>,
    /// Every genre in document order (`genre` / `sub_genre` hold the default one).
    #[serde(default)]
    pub genres: Vec<GenreInfo>,
    /// The default `ParentalWarningType` value.
    #[serde(default)]
    pub parental_warning: Option<String>,
    /// Every `ParentalWarningType` value in document order.
    #[serde(default)]
    pub parental_warnings: Vec<String>,
    /// The default label name: 3.8.x `LabelName`; 4.x the `PartyList` name of the first
    /// `ReleaseLabelReference`.
    #[serde(default)]
    pub label_name: Option<String>,
    /// Every release date as written (`release_date` holds the default one when it is a
    /// full date).
    #[serde(default)]
    pub release_dates: Vec<DateInfo>,
    /// Every original release date as written.
    #[serde(default)]
    pub original_release_dates: Vec<DateInfo>,
}

/// One genre with its territory scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenreInfo {
    pub genre: String,
    pub sub_genre: Option<String>,
    #[serde(default, flatten)]
    pub scope: TerritoryScope,
}

/// One date as written (`YYYY`, `YYYY-MM` or `YYYY-MM-DD`) with its territory scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateInfo {
    pub date: String,
    #[serde(default)]
    pub is_approximate: Option<bool>,
    #[serde(default, flatten)]
    pub scope: TerritoryScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseIdentifiers {
    pub upc: Option<String>,
    pub ean: Option<String>,
    pub catalog_number: Option<String>,
    pub grid: Option<String>,
    /// `ReleaseId/ISRC` (ERN 3.8.x; 4.x has no release ISRC).
    #[serde(default)]
    pub isrc: Option<String>,
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
