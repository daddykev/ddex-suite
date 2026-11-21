// core/src/models/graph/release.rs
//! Release types

use super::{Artist, DisplayText, ResourceGroup};
use crate::models::{
    common::{Identifier, LocalizedString},
    AttributeMap, Comment, Extensions,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    /// ERN 4.x `DisplayTitleText` entries (empty for 3.8.x).
    #[serde(default)]
    pub display_title_text: Vec<DisplayText>,
    /// ERN 4.x `DisplayTitle/TitleText` entries (empty for 3.8.x).
    #[serde(default)]
    pub display_title: Vec<DisplayText>,
    /// ERN 4.x `DisplayArtistName` entries (empty for 3.8.x).
    #[serde(default)]
    pub display_artist_name: Vec<DisplayText>,
    /// ERN 4.x `ResourceGroup` tree (`None` for 3.8.x).
    #[serde(default)]
    pub resource_group: Option<ResourceGroup>,
    /// All XML attributes (standard and custom)
    pub attributes: Option<AttributeMap>,
    /// Extensions for release
    pub extensions: Option<Extensions>,
    /// Comments associated with release
    pub comments: Option<Vec<Comment>>,
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
    /// All XML attributes (standard and custom)
    pub attributes: Option<AttributeMap>,
    /// Extensions for genre
    pub extensions: Option<Extensions>,
    /// Comments associated with genre
    pub comments: Option<Vec<Comment>>,
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
    /// Extensions for resource reference
    pub extensions: Option<Extensions>,
    /// Comments associated with resource reference
    pub comments: Option<Vec<Comment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseParty {
    pub party_reference: String,
    pub role: Vec<String>,
    /// Extensions for release party
    pub extensions: Option<Extensions>,
    /// Comments associated with release party
    pub comments: Option<Vec<Comment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseEvent {
    pub release_event_type: String,
    pub event_date: Option<DateTime<Utc>>,
    pub territory: Option<String>,
    /// Extensions for release event
    pub extensions: Option<Extensions>,
    /// Comments associated with release event
    pub comments: Option<Vec<Comment>>,
}
