//! # DDEX Configuration Presets
//! 
//! This module provides pre-configured settings for DDEX message generation.
//! Presets are community-maintained configuration templates that help ensure
//! DDEX compliance and reduce configuration complexity.
//! 
//! ## Available Presets
//! 
//! ### Generic Industry-Standard Presets
//! - **audio_album**: DDEX-compliant audio album configuration
//! - **audio_single**: DDEX-compliant single track configuration  
//! - **video_single**: DDEX-compliant video release configuration
//! - **compilation**: DDEX-compliant compilation album configuration
//! 
//! ### Platform Presets (Based on Public Documentation)
//! - **YouTube Music**: Audio and video releases (based on public Partner docs)
//! 
//! ## Architecture
//! 
//! ```text
//! Preset System
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │  Base Config    │───▶│  Partner Rules   │───▶│ Final Settings  │
//! │ (DDEX defaults) │    │ (customizations) │    │ (ready to use)  │
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//!           │                       │                       │
//!           ▼                       ▼                       ▼
//!    ┌─────────────┐      ┌─────────────────┐    ┌─────────────────┐
//!    │ • Version   │      │ • Required      │    │ • Validation    │
//!    │ • Profile   │      │ • Validation    │    │ • Defaults      │
//!    │ • Schema    │      │ • Territories   │    │ • Mappings      │
//!    │ • Defaults  │      │ • Quality       │    │ • Overrides     │
//!    └─────────────┘      └─────────────────┘    └─────────────────┘
//! ```
//! 
//! ## Usage Example
//! 
//! ```rust
//! use ddex_builder::presets::*;
//! use ddex_builder::Builder;
//! 
//! // Use generic audio album preset
//! let mut builder = Builder::new();
//! builder.apply_preset(&generic::audio_album())?;
//! 
//! // Use YouTube preset for video content
//! builder.apply_preset(&youtube::youtube_video())?;
//! 
//! // Load by name
//! let presets = all_presets();
//! let audio_album = &presets["audio_album"];
//! builder.apply_partner_preset(audio_album)?;
//! 
//! // List available presets
//! for (name, preset) in all_presets() {
//!     println!("{}: {}", name, preset.description);
//! }
//! ```
//! 
//! ## Preset Features
//! 
//! Each preset includes:
//! 
//! - **Schema Version**: DDEX ERN version (3.8.2, 4.2, 4.3)
//! - **Message Profile**: Audio, Video, or Mixed content
//! - **Required Fields**: Mandatory metadata fields
//! - **Validation Rules**: Data format and quality requirements
//! - **Default Values**: Common field defaults
//! - **Territory Codes**: Allowed distribution territories
//! - **Quality Standards**: Audio/video quality minimums
//! 
//! ## Custom Presets
//! 
//! Create your own preset for internal standards:
//! 
//! ```rust
//! use ddex_builder::presets::*;
//! use indexmap::IndexMap;
//! 
//! // Start with a generic preset as base
//! let mut custom_preset = generic::audio_album();
//! custom_preset.name = "my_label_preset".to_string();
//! custom_preset.description = "My Record Label Requirements".to_string();
//! 
//! // Add custom validation rules
//! custom_preset.validation_rules.insert(
//!     "Genre".to_string(), 
//!     ValidationRule::OneOf(vec!["Rock".to_string(), "Pop".to_string()])
//! );
//! 
//! // Add custom territory restrictions
//! custom_preset.config.territory_codes = vec!["US".to_string(), "CA".to_string()];
//! ```
//! 
//! ## Validation Rules
//! 
//! Presets support comprehensive validation:
//! 
//! - **Required**: Field must be present
//! - **MinLength/MaxLength**: String length constraints
//! - **Pattern**: Regex pattern matching
//! - **OneOf**: Value must be from allowed list
//! - **AudioQuality**: Minimum bit depth and sample rate
//! - **TerritoryCode**: Allowed distribution territories
//! - **Custom**: Partner-specific validation logic

pub mod generic;
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



/// Get all built-in presets
/// 
/// Returns a collection of community-maintained DDEX configuration presets.
/// These presets provide baseline DDEX-compliant configurations and platform-specific
/// templates based on publicly available documentation.
pub fn all_presets() -> IndexMap<String, PartnerPreset> {
    let mut presets = IndexMap::new();
    
    // Generic industry-standard presets
    presets.extend(generic::all_generic_presets());
    
    // Platform presets (based on public documentation)
    presets.extend(youtube::all_youtube_presets());
    
    presets
}