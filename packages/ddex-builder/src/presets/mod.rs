//! Partner presets for common configurations

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Partner preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnerPreset {
    pub name: String,
    pub description: String,
    pub source: PresetSource,
    pub provenance_url: Option<String>,
    pub version: String,
    pub locked: bool,
    pub disclaimer: String,
    pub determinism: super::determinism::DeterminismConfig,
    pub defaults: PresetDefaults,
    pub required_fields: Vec<String>,
    pub format_overrides: IndexMap<String, String>,
}

/// Preset source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetSource {
    PublicDocs,
    CustomerFeedback,
    Community,
}

/// Preset defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetDefaults {
    pub message_control_type: Option<String>,
    pub territory_code: Vec<String>,
    pub distribution_channel: Vec<String>,
}

/// Spotify Audio Album ERN 4.3 preset
pub fn spotify_audio_43() -> PartnerPreset {
    PartnerPreset {
        name: "spotify_audio_43".to_string(),
        description: "Spotify Audio Album ERN 4.3 requirements".to_string(),
        source: PresetSource::PublicDocs,
        provenance_url: Some("https://support.spotify.com/artists/article/ddex-delivery-spec".to_string()),
        version: "1.0.0".to_string(),
        locked: false,
        disclaimer: "Community-maintained config template. Not an official spec.".to_string(),
        determinism: super::determinism::DeterminismConfig::default(),
        defaults: PresetDefaults {
            message_control_type: Some("LiveMessage".to_string()),
            territory_code: vec!["Worldwide".to_string()],
            distribution_channel: vec!["01".to_string()], // Download
        },
        required_fields: vec![
            "ISRC".to_string(),
            "UPC".to_string(),
            "ReleaseDate".to_string(),
            "Genre".to_string(),
        ],
        format_overrides: IndexMap::new(),
    }
}

/// Apple Music ERN 4.3 preset
pub fn apple_music_43() -> PartnerPreset {
    PartnerPreset {
        name: "apple_music_43".to_string(),
        description: "Apple Music ERN 4.3 requirements".to_string(),
        source: PresetSource::PublicDocs,
        provenance_url: Some("https://help.apple.com/itc/musicspec/".to_string()),
        version: "1.0.0".to_string(),
        locked: false,
        disclaimer: "Community-maintained config template. Not an official spec.".to_string(),
        determinism: super::determinism::DeterminismConfig::default(),
        defaults: PresetDefaults {
            message_control_type: Some("LiveMessage".to_string()),
            territory_code: vec!["Worldwide".to_string()],
            distribution_channel: vec!["01".to_string()],
        },
        required_fields: vec![
            "ISRC".to_string(),
            "UPC".to_string(),
            "ReleaseDate".to_string(),
        ],
        format_overrides: IndexMap::new(),
    }
}