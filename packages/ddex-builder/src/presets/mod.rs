//! Partner presets for common configurations

pub mod spotify;
pub mod youtube;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// DDEX version enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DdexVersion {
    #[serde(rename = "ERN/3.8.2")]
    Ern382,
    #[serde(rename = "ERN/4.2")]
    Ern42,
    #[serde(rename = "ERN/4.3")]
    Ern43,
    #[serde(rename = "ERN/4.1")]
    Ern41,
}

impl std::fmt::Display for DdexVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DdexVersion::Ern382 => write!(f, "ERN/3.8.2"),
            DdexVersion::Ern42 => write!(f, "ERN/4.2"),
            DdexVersion::Ern43 => write!(f, "ERN/4.3"),
            DdexVersion::Ern41 => write!(f, "ERN/4.1"),
        }
    }
}

/// Message profile enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageProfile {
    AudioAlbum,
    AudioSingle,
    VideoAlbum,
    VideoSingle,
    Mixed,
}

/// Validation rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    OneOf(Vec<String>),
    AudioQuality { min_bit_depth: u8, min_sample_rate: u32 },
    TerritoryCode { allowed: Vec<String> },
    Custom(String),
}

/// Enhanced preset configuration with validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfig {
    pub version: DdexVersion,
    pub profile: MessageProfile,
    pub required_fields: Vec<String>,
    pub validation_rules: IndexMap<String, ValidationRule>,
    pub default_values: IndexMap<String, String>,
    pub custom_mappings: IndexMap<String, String>,
    pub territory_codes: Vec<String>,
    pub distribution_channels: Vec<String>,
    pub release_types: Vec<String>,
}

/// Partner preset configuration (legacy structure, enhanced)
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
    // Enhanced fields
    pub config: PresetConfig,
    pub validation_rules: IndexMap<String, ValidationRule>,
    pub custom_mappings: IndexMap<String, String>,
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
    let mut validation_rules = IndexMap::new();
    validation_rules.insert("ISRC".to_string(), ValidationRule::Required);
    validation_rules.insert("UPC".to_string(), ValidationRule::Required);
    validation_rules.insert("ReleaseDate".to_string(), ValidationRule::Required);
    validation_rules.insert("Genre".to_string(), ValidationRule::Required);
    validation_rules.insert("ExplicitContent".to_string(), ValidationRule::Required);
    validation_rules.insert("AudioQuality".to_string(), ValidationRule::AudioQuality { 
        min_bit_depth: 16, 
        min_sample_rate: 44100 
    });
    
    let mut default_values = IndexMap::new();
    default_values.insert("MessageControlType".to_string(), "LiveMessage".to_string());
    default_values.insert("TerritoryCode".to_string(), "Worldwide".to_string());
    default_values.insert("DistributionChannel".to_string(), "01".to_string());
    
    let config = PresetConfig {
        version: DdexVersion::Ern43,
        profile: MessageProfile::AudioAlbum,
        required_fields: vec![
            "ISRC".to_string(),
            "UPC".to_string(),
            "ReleaseDate".to_string(),
            "Genre".to_string(),
            "ExplicitContent".to_string(),
        ],
        validation_rules: validation_rules.clone(),
        default_values,
        custom_mappings: IndexMap::new(),
        territory_codes: vec!["Worldwide".to_string()],
        distribution_channels: vec!["01".to_string()],
        release_types: vec!["Album".to_string(), "Single".to_string(), "EP".to_string()],
    };

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
            distribution_channel: vec!["01".to_string()],
        },
        required_fields: config.required_fields.clone(),
        format_overrides: IndexMap::new(),
        config,
        validation_rules,
        custom_mappings: IndexMap::new(),
    }
}

/// Apple Music ERN 4.3 preset (updated with new structure)
pub fn apple_music_43() -> PartnerPreset {
    let mut validation_rules = IndexMap::new();
    validation_rules.insert("ISRC".to_string(), ValidationRule::Required);
    validation_rules.insert("UPC".to_string(), ValidationRule::Required);
    validation_rules.insert("ReleaseDate".to_string(), ValidationRule::Required);
    
    let mut default_values = IndexMap::new();
    default_values.insert("MessageControlType".to_string(), "LiveMessage".to_string());
    default_values.insert("TerritoryCode".to_string(), "Worldwide".to_string());
    default_values.insert("DistributionChannel".to_string(), "01".to_string());
    
    let config = PresetConfig {
        version: DdexVersion::Ern43,
        profile: MessageProfile::AudioAlbum,
        required_fields: vec![
            "ISRC".to_string(),
            "UPC".to_string(),
            "ReleaseDate".to_string(),
        ],
        validation_rules: validation_rules.clone(),
        default_values,
        custom_mappings: IndexMap::new(),
        territory_codes: vec!["Worldwide".to_string()],
        distribution_channels: vec!["01".to_string()],
        release_types: vec!["Album".to_string(), "Single".to_string()],
    };

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
        required_fields: config.required_fields.clone(),
        format_overrides: IndexMap::new(),
        config,
        validation_rules,
        custom_mappings: IndexMap::new(),
    }
}

/// Get all built-in presets
pub fn all_presets() -> IndexMap<String, PartnerPreset> {
    let mut presets = IndexMap::new();
    
    // Legacy presets
    presets.insert("spotify_audio_43".to_string(), spotify_audio_43());
    presets.insert("apple_music_43".to_string(), apple_music_43());
    
    // Spotify presets
    presets.extend(spotify::all_spotify_presets());
    
    // YouTube presets  
    presets.extend(youtube::all_youtube_presets());
    
    presets
}