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
//! use ddex_builder::presets::all_presets;
//! use ddex_builder::Builder;
//!
//! // Use the generic audio album preset
//! let mut builder = Builder::new();
//! builder.apply_preset("audio_album", false)?;
//!
//! // Switch to the YouTube preset for video content
//! builder.apply_preset("youtube_video", false)?;
//!
//! // List available presets
//! for (name, preset) in all_presets() {
//!     println!("{}: {}", name, preset.description);
//! }
//! # Ok::<(), ddex_builder::BuildError>(())
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

/// DDEX version for presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DdexVersion {
    /// ERN 3.8.2
    #[serde(rename = "ERN/3.8.2")]
    Ern382,
    /// ERN 4.2
    #[serde(rename = "ERN/4.2")]
    Ern42,
    /// ERN 4.3
    #[serde(rename = "ERN/4.3")]
    Ern43,
    /// ERN 4.1
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

/// Message profile type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageProfile {
    /// Audio album release
    AudioAlbum,
    /// Audio single release
    AudioSingle,
    /// Video album release
    VideoAlbum,
    /// Video single release
    VideoSingle,
    /// Mixed content release
    Mixed,
}

/// Validation rule for preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    /// Field is required
    Required,
    /// Minimum length requirement
    MinLength(usize),
    /// Maximum length requirement
    MaxLength(usize),
    /// Must match regex pattern
    Pattern(String),
    /// Must be one of specified values
    OneOf(Vec<String>),
    /// Audio quality requirements
    AudioQuality {
        /// Minimum bit depth in bits
        min_bit_depth: u8,
        /// Minimum sample rate in Hz
        min_sample_rate: u32,
    },
    /// Territory code restrictions
    TerritoryCode {
        /// List of allowed territory codes
        allowed: Vec<String>,
    },
    /// Custom validation rule
    Custom(String),
}

/// Preset defaults configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfig {
    /// DDEX version to use
    pub version: DdexVersion,
    /// Message profile type
    pub profile: MessageProfile,
    /// Required fields list
    pub required_fields: Vec<String>,
    /// Validation rules by field name
    pub validation_rules: IndexMap<String, ValidationRule>,
    /// Default values by field name
    pub default_values: IndexMap<String, String>,
    /// Custom field mappings
    pub custom_mappings: IndexMap<String, String>,
    /// Supported territory codes
    pub territory_codes: Vec<String>,
    /// Supported distribution channels
    pub distribution_channels: Vec<String>,
    /// Supported release types
    pub release_types: Vec<String>,
}

/// Partner preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnerPreset {
    /// Preset name
    pub name: String,
    /// Preset description
    pub description: String,
    /// Source of preset definition
    pub source: PresetSource,
    /// URL to documentation
    pub provenance_url: Option<String>,
    /// Preset version
    pub version: String,
    /// Whether preset is locked from editing
    pub locked: bool,
    /// Legal disclaimer
    pub disclaimer: String,
    /// Determinism configuration
    pub determinism: super::determinism::DeterminismConfig,
    /// Default values for preset
    pub defaults: PresetDefaults,
    /// Required fields that must be present for this partner
    pub required_fields: Vec<String>,
    /// Format overrides for specific fields (field_name -> format_string)
    pub format_overrides: IndexMap<String, String>,
    // Enhanced fields
    /// Preset configuration settings
    pub config: PresetConfig,
    /// Validation rules specific to this partner
    pub validation_rules: IndexMap<String, ValidationRule>,
    /// Custom field mappings for partner-specific requirements
    pub custom_mappings: IndexMap<String, String>,
}

/// Source of preset definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetSource {
    /// Official public documentation from partner
    PublicDocs,
    /// Based on customer feedback and testing
    CustomerFeedback,
    /// Community-contributed preset
    Community,
}

/// Preset configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetDefaults {
    /// Type of message control (e.g., "NewReleaseMessage")
    pub message_control_type: Option<String>,
    /// Territory codes this preset applies to
    pub territory_code: Vec<String>,
    /// Distribution channels (e.g., "Streaming", "Download")
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
