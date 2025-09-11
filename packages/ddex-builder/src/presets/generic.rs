//! Generic industry-standard DDEX presets
//!
//! These presets provide baseline DDEX-compliant configurations that work
//! across most platforms and distribution scenarios. They follow DDEX
//! specification requirements without platform-specific customizations.

use super::{DdexVersion, MessageProfile, PresetConfig, PartnerPreset, PresetDefaults, PresetSource, ValidationRule};
use indexmap::IndexMap;

/// Generic Audio Album preset (ERN 4.3)
/// 
/// A baseline configuration for audio album releases that follows DDEX ERN 4.3
/// specification requirements. This preset ensures compliance with core DDEX
/// standards and can be used as a starting point for platform-specific customizations.
pub fn audio_album() -> PartnerPreset {
    let mut validation_rules = IndexMap::new();
    validation_rules.insert("ISRC".to_string(), ValidationRule::Required);
    validation_rules.insert("ReleaseDate".to_string(), ValidationRule::Required);
    validation_rules.insert("Genre".to_string(), ValidationRule::Required);
    validation_rules.insert("AlbumTitle".to_string(), ValidationRule::Required);
    validation_rules.insert("ArtistName".to_string(), ValidationRule::Required);
    validation_rules.insert("TrackTitle".to_string(), ValidationRule::Required);
    validation_rules.insert("ISRC".to_string(), ValidationRule::Pattern(
        r"^[A-Z]{2}[A-Z0-9]{3}\d{7}$".to_string()
    ));
    validation_rules.insert("Duration".to_string(), ValidationRule::Pattern(
        r"^PT(\d+H)?(\d+M)?(\d+(\.\d+)?S)?$".to_string()
    ));
    
    let mut default_values = IndexMap::new();
    default_values.insert("MessageControlType".to_string(), "LiveMessage".to_string());
    default_values.insert("ReleaseType".to_string(), "Album".to_string());
    
    let config = PresetConfig {
        version: DdexVersion::Ern43,
        profile: MessageProfile::AudioAlbum,
        required_fields: vec![
            "ISRC".to_string(),
            "ReleaseDate".to_string(),
            "Genre".to_string(),
            "AlbumTitle".to_string(),
            "ArtistName".to_string(),
            "TrackTitle".to_string(),
        ],
        validation_rules: validation_rules.clone(),
        default_values,
        custom_mappings: IndexMap::new(),
        territory_codes: vec!["Worldwide".to_string()],
        distribution_channels: vec!["01".to_string()], // Download/Stream
        release_types: vec!["Album".to_string(), "CompilationAlbum".to_string(), "LiveAlbum".to_string()],
    };

    PartnerPreset {
        name: "audio_album".to_string(),
        description: "Generic Audio Album ERN 4.3 - DDEX-compliant baseline configuration".to_string(),
        source: PresetSource::Community,
        provenance_url: Some("https://ddex.net/standards/".to_string()),
        version: "1.0.0".to_string(),
        locked: false,
        disclaimer: "Generic industry-standard preset based on DDEX ERN 4.3 specification. Customize for specific platform requirements.".to_string(),
        determinism: super::super::determinism::DeterminismConfig::default(),
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

/// Generic Audio Single preset (ERN 4.3)
/// 
/// A baseline configuration for audio single releases following DDEX ERN 4.3
/// specification. Optimized for single-track releases with simplified requirements.
pub fn audio_single() -> PartnerPreset {
    let mut validation_rules = IndexMap::new();
    validation_rules.insert("ISRC".to_string(), ValidationRule::Required);
    validation_rules.insert("ReleaseDate".to_string(), ValidationRule::Required);
    validation_rules.insert("Genre".to_string(), ValidationRule::Required);
    validation_rules.insert("TrackTitle".to_string(), ValidationRule::Required);
    validation_rules.insert("ArtistName".to_string(), ValidationRule::Required);
    validation_rules.insert("ISRC".to_string(), ValidationRule::Pattern(
        r"^[A-Z]{2}[A-Z0-9]{3}\d{7}$".to_string()
    ));
    validation_rules.insert("Duration".to_string(), ValidationRule::Pattern(
        r"^PT(\d+H)?(\d+M)?(\d+(\.\d+)?S)?$".to_string()
    ));
    
    let mut default_values = IndexMap::new();
    default_values.insert("MessageControlType".to_string(), "LiveMessage".to_string());
    default_values.insert("ReleaseType".to_string(), "Single".to_string());
    
    let config = PresetConfig {
        version: DdexVersion::Ern43,
        profile: MessageProfile::AudioSingle,
        required_fields: vec![
            "ISRC".to_string(),
            "ReleaseDate".to_string(),
            "Genre".to_string(),
            "TrackTitle".to_string(),
            "ArtistName".to_string(),
        ],
        validation_rules: validation_rules.clone(),
        default_values,
        custom_mappings: IndexMap::new(),
        territory_codes: vec!["Worldwide".to_string()],
        distribution_channels: vec!["01".to_string()],
        release_types: vec!["Single".to_string()],
    };

    PartnerPreset {
        name: "audio_single".to_string(),
        description: "Generic Audio Single ERN 4.3 - DDEX-compliant single track configuration".to_string(),
        source: PresetSource::Community,
        provenance_url: Some("https://ddex.net/standards/".to_string()),
        version: "1.0.0".to_string(),
        locked: false,
        disclaimer: "Generic industry-standard preset based on DDEX ERN 4.3 specification. Customize for specific platform requirements.".to_string(),
        determinism: super::super::determinism::DeterminismConfig::default(),
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

/// Generic Video Single preset (ERN 4.3)
/// 
/// A baseline configuration for video releases with synchronized audio.
/// Includes requirements for both audio and video resources.
pub fn video_single() -> PartnerPreset {
    let mut validation_rules = IndexMap::new();
    validation_rules.insert("ISRC".to_string(), ValidationRule::Required);
    validation_rules.insert("ReleaseDate".to_string(), ValidationRule::Required);
    validation_rules.insert("Genre".to_string(), ValidationRule::Required);
    validation_rules.insert("VideoTitle".to_string(), ValidationRule::Required);
    validation_rules.insert("ArtistName".to_string(), ValidationRule::Required);
    validation_rules.insert("VideoResource".to_string(), ValidationRule::Required);
    validation_rules.insert("AudioResource".to_string(), ValidationRule::Required);
    validation_rules.insert("ISRC".to_string(), ValidationRule::Pattern(
        r"^[A-Z]{2}[A-Z0-9]{3}\d{7}$".to_string()
    ));
    
    let mut default_values = IndexMap::new();
    default_values.insert("MessageControlType".to_string(), "LiveMessage".to_string());
    default_values.insert("ReleaseType".to_string(), "VideoSingle".to_string());
    
    let mut custom_mappings = IndexMap::new();
    custom_mappings.insert("VideoResource".to_string(), "VideoTechnicalResourceDetails".to_string());
    custom_mappings.insert("AudioResource".to_string(), "SoundRecordingTechnicalResourceDetails".to_string());
    
    let config = PresetConfig {
        version: DdexVersion::Ern43,
        profile: MessageProfile::VideoSingle,
        required_fields: vec![
            "ISRC".to_string(),
            "ReleaseDate".to_string(),
            "Genre".to_string(),
            "VideoTitle".to_string(),
            "ArtistName".to_string(),
            "VideoResource".to_string(),
            "AudioResource".to_string(),
        ],
        validation_rules: validation_rules.clone(),
        default_values,
        custom_mappings: custom_mappings.clone(),
        territory_codes: vec!["Worldwide".to_string()],
        distribution_channels: vec!["01".to_string(), "02".to_string()], // Download + Streaming
        release_types: vec!["VideoSingle".to_string(), "MusicVideo".to_string()],
    };

    PartnerPreset {
        name: "video_single".to_string(),
        description: "Generic Video Single ERN 4.3 - DDEX-compliant video release configuration".to_string(),
        source: PresetSource::Community,
        provenance_url: Some("https://ddex.net/standards/".to_string()),
        version: "1.0.0".to_string(),
        locked: false,
        disclaimer: "Generic industry-standard preset based on DDEX ERN 4.3 specification. Customize for specific platform requirements.".to_string(),
        determinism: super::super::determinism::DeterminismConfig::default(),
        defaults: PresetDefaults {
            message_control_type: Some("LiveMessage".to_string()),
            territory_code: vec!["Worldwide".to_string()],
            distribution_channel: vec!["01".to_string(), "02".to_string()],
        },
        required_fields: config.required_fields.clone(),
        format_overrides: IndexMap::new(),
        config,
        validation_rules,
        custom_mappings,
    }
}

/// Generic Compilation preset (ERN 4.3)
/// 
/// A baseline configuration for compilation releases with multiple artists.
/// Includes validation for various artist scenarios and copyright handling.
pub fn compilation() -> PartnerPreset {
    let mut preset = audio_album();
    
    // Modify for compilation-specific settings
    preset.name = "compilation".to_string();
    preset.description = "Generic Compilation ERN 4.3 - DDEX-compliant multi-artist compilation configuration".to_string();
    preset.config.release_types = vec!["CompilationAlbum".to_string()];
    preset.config.default_values.insert("ReleaseType".to_string(), "CompilationAlbum".to_string());
    
    // Add compilation-specific validation
    preset.validation_rules.insert("CompilationIndicator".to_string(), ValidationRule::Required);
    preset.validation_rules.insert("VariousArtists".to_string(), ValidationRule::Custom("Multiple contributing artists".to_string()));
    preset.config.required_fields.push("CompilationIndicator".to_string());
    preset.required_fields.push("CompilationIndicator".to_string());
    
    preset
}

/// Get all generic presets
pub fn all_generic_presets() -> IndexMap<String, PartnerPreset> {
    let mut presets = IndexMap::new();
    presets.insert("audio_album".to_string(), audio_album());
    presets.insert("audio_single".to_string(), audio_single());
    presets.insert("video_single".to_string(), video_single());
    presets.insert("compilation".to_string(), compilation());
    presets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_album_preset() {
        let preset = audio_album();
        assert_eq!(preset.name, "audio_album");
        assert_eq!(preset.config.version, DdexVersion::Ern43);
        assert_eq!(preset.config.profile, MessageProfile::AudioAlbum);
        assert!(preset.required_fields.contains(&"ISRC".to_string()));
        assert!(preset.required_fields.contains(&"AlbumTitle".to_string()));
        assert_eq!(preset.source, PresetSource::Community);
    }

    #[test]
    fn test_audio_single_preset() {
        let preset = audio_single();
        assert_eq!(preset.name, "audio_single");
        assert_eq!(preset.config.profile, MessageProfile::AudioSingle);
        assert!(preset.required_fields.contains(&"TrackTitle".to_string()));
        assert!(!preset.required_fields.contains(&"AlbumTitle".to_string()));
    }

    #[test]
    fn test_video_single_preset() {
        let preset = video_single();
        assert_eq!(preset.name, "video_single");
        assert_eq!(preset.config.profile, MessageProfile::VideoSingle);
        assert!(preset.required_fields.contains(&"VideoResource".to_string()));
        assert!(preset.required_fields.contains(&"AudioResource".to_string()));
    }

    #[test]
    fn test_compilation_preset() {
        let preset = compilation();
        assert_eq!(preset.name, "compilation");
        assert!(preset.config.release_types.contains(&"CompilationAlbum".to_string()));
        assert!(preset.required_fields.contains(&"CompilationIndicator".to_string()));
    }

    #[test]
    fn test_all_generic_presets() {
        let presets = all_generic_presets();
        assert_eq!(presets.len(), 4);
        assert!(presets.contains_key("audio_album"));
        assert!(presets.contains_key("audio_single"));
        assert!(presets.contains_key("video_single"));
        assert!(presets.contains_key("compilation"));
    }

    #[test]
    fn test_generic_presets_source() {
        let presets = all_generic_presets();
        for (_, preset) in presets {
            assert_eq!(preset.source, PresetSource::Community);
            assert!(preset.disclaimer.contains("Generic industry-standard"));
        }
    }
}