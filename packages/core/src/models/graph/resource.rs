// core/src/models/graph/resource.rs
//! Resource types

use super::{Artist, DisplayText, Genre, LabelName, ParentalWarning};
use crate::models::{
    common::{Copyright, Identifier, LocalizedString},
    Extensions,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_reference: String,
    pub resource_type: ResourceType,
    pub resource_id: Vec<Identifier>,
    pub reference_title: Vec<LocalizedString>,
    pub duration: Option<std::time::Duration>,
    pub technical_details: Vec<TechnicalDetails>,
    pub rights_controller: Vec<String>,
    pub p_line: Vec<Copyright>,
    pub c_line: Vec<Copyright>,
    /// Extensions for resource
    pub extensions: Option<Extensions>,
    /// ERN 4.x `DisplayTitleText` entries (empty for 3.8.x).
    #[serde(default)]
    pub display_title_text: Vec<DisplayText>,
    /// ERN 4.x `DisplayTitle/TitleText` entries (empty for 3.8.x).
    #[serde(default)]
    pub display_title: Vec<DisplayText>,
    /// ERN 4.x `DisplayArtistName` entries (empty for 3.8.x).
    #[serde(default)]
    pub display_artist_name: Vec<DisplayText>,
    /// ERN 4.x `DisplayArtist` entries, referencing parties via `party_reference`
    /// (empty for 3.8.x).
    #[serde(default)]
    pub display_artist: Vec<Artist>,
    /// ERN 4.x `Type` (e.g. `MusicalWorkSoundRecording`); `None` for 3.8.x.
    #[serde(default)]
    pub type_value: Option<String>,
    /// ERN 4.x `Type/@UserDefinedValue` (with `Type` `UserDefined`).
    #[serde(default)]
    pub type_user_defined_value: Option<String>,
    /// ERN 4.x `ParentalWarningType` values, in document order (empty for 3.8.x).
    #[serde(default)]
    pub parental_warning_type: Vec<String>,
    /// `ParentalWarningType` entries with their attributes / territory scope.
    #[serde(default)]
    pub parental_warnings: Vec<ParentalWarning>,
    /// 3.8.x `...DetailsByTerritory/Genre` entries (4.x resources carry no genre).
    #[serde(default)]
    pub genre: Vec<Genre>,
    /// 3.8.x `...DetailsByTerritory/LabelName` entries.
    #[serde(default)]
    pub label_name: Vec<LabelName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    SoundRecording,
    Video,
    Image,
    Text,
    SheetMusic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDetails {
    pub technical_resource_details_reference: String,
    pub audio_codec: Option<String>,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub file_format: Option<String>,
    pub file_size: Option<u64>,
    /// Extensions for technical details
    pub extensions: Option<Extensions>,
}
