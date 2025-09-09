//! Spotify-specific DDEX presets and configurations

use super::{DdexVersion, MessageProfile, PresetConfig, PartnerPreset, PresetDefaults, PresetSource, ValidationRule};
use indexmap::IndexMap;

/// Spotify Album preset (ERN 4.3)
pub fn spotify_album() -> PartnerPreset {
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
    validation_rules.insert("TerritoryCode".to_string(), ValidationRule::TerritoryCode { 
        allowed: vec!["Worldwide".to_string(), "WW".to_string()] 
    });
    validation_rules.insert("ReleaseType".to_string(), ValidationRule::OneOf(vec![
        "Album".to_string(),
        "CompilationAlbum".to_string(),
        "LiveAlbum".to_string(),
    ]));
    
    let mut default_values = IndexMap::new();
    default_values.insert("MessageControlType".to_string(), "LiveMessage".to_string());
    default_values.insert("TerritoryCode".to_string(), "Worldwide".to_string());
    default_values.insert("DistributionChannel".to_string(), "01".to_string());
    default_values.insert("ReleaseType".to_string(), "Album".to_string());
    
    let mut custom_mappings = IndexMap::new();
    custom_mappings.insert("ExplicitContent".to_string(), "ParentalWarningType".to_string());
    custom_mappings.insert("AudioQuality".to_string(), "SoundRecordingTechnicalResourceDetails".to_string());
    
    let config = PresetConfig {
        version: DdexVersion::Ern43,
        profile: MessageProfile::AudioAlbum,
        required_fields: vec![
            "ISRC".to_string(),
            "UPC".to_string(),
            "ReleaseDate".to_string(),
            "Genre".to_string(),
            "ExplicitContent".to_string(),
            "AlbumTitle".to_string(),
            "ArtistName".to_string(),
            "TrackTitle".to_string(),
        ],
        validation_rules: validation_rules.clone(),
        default_values,
        custom_mappings: custom_mappings.clone(),
        territory_codes: vec!["Worldwide".to_string()],
        distribution_channels: vec!["01".to_string()], // Download/Stream
        release_types: vec!["Album".to_string(), "CompilationAlbum".to_string(), "LiveAlbum".to_string()],
    };

    PartnerPreset {
        name: "spotify_album".to_string(),
        description: "Spotify Album ERN 4.3 requirements with audio quality validation".to_string(),
        source: PresetSource::PublicDocs,
        provenance_url: Some("https://support.spotify.com/artists/article/ddex-delivery-spec".to_string()),
        version: "1.0.0".to_string(),
        locked: false,
        disclaimer: "Based on Spotify public documentation. Verify current requirements.".to_string(),
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
        custom_mappings,
    }
}

/// Spotify Single preset (ERN 4.3)
pub fn spotify_single() -> PartnerPreset {
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
    validation_rules.insert("TerritoryCode".to_string(), ValidationRule::TerritoryCode { 
        allowed: vec!["Worldwide".to_string(), "WW".to_string()] 
    });
    validation_rules.insert("ReleaseType".to_string(), ValidationRule::OneOf(vec![
        "Single".to_string(),
        "VideoSingle".to_string(),
    ]));
    
    let mut default_values = IndexMap::new();
    default_values.insert("MessageControlType".to_string(), "LiveMessage".to_string());
    default_values.insert("TerritoryCode".to_string(), "Worldwide".to_string());
    default_values.insert("DistributionChannel".to_string(), "01".to_string());
    default_values.insert("ReleaseType".to_string(), "Single".to_string());
    
    let mut custom_mappings = IndexMap::new();
    custom_mappings.insert("ExplicitContent".to_string(), "ParentalWarningType".to_string());
    custom_mappings.insert("AudioQuality".to_string(), "SoundRecordingTechnicalResourceDetails".to_string());
    
    let config = PresetConfig {
        version: DdexVersion::Ern43,
        profile: MessageProfile::AudioSingle,
        required_fields: vec![
            "ISRC".to_string(),
            "UPC".to_string(),
            "ReleaseDate".to_string(),
            "Genre".to_string(),
            "ExplicitContent".to_string(),
            "TrackTitle".to_string(),
            "ArtistName".to_string(),
        ],
        validation_rules: validation_rules.clone(),
        default_values,
        custom_mappings: custom_mappings.clone(),
        territory_codes: vec!["Worldwide".to_string()],
        distribution_channels: vec!["01".to_string()],
        release_types: vec!["Single".to_string(), "VideoSingle".to_string()],
    };

    PartnerPreset {
        name: "spotify_single".to_string(),
        description: "Spotify Single ERN 4.3 requirements with simplified track structure".to_string(),
        source: PresetSource::PublicDocs,
        provenance_url: Some("https://support.spotify.com/artists/article/ddex-delivery-spec".to_string()),
        version: "1.0.0".to_string(),
        locked: false,
        disclaimer: "Based on Spotify public documentation. Verify current requirements.".to_string(),
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
        custom_mappings,
    }
}

/// Spotify EP preset (ERN 4.3) 
pub fn spotify_ep() -> PartnerPreset {
    let mut preset = spotify_album();
    
    // Modify for EP-specific settings
    preset.name = "spotify_ep".to_string();
    preset.description = "Spotify EP ERN 4.3 requirements (2-6 tracks)".to_string();
    preset.config.profile = MessageProfile::AudioAlbum;
    preset.config.release_types = vec!["EP".to_string()];
    preset.config.default_values.insert("ReleaseType".to_string(), "EP".to_string());
    preset.validation_rules.insert("TrackCount".to_string(), ValidationRule::Custom("2-6 tracks".to_string()));
    
    preset
}

/// Get all Spotify presets
pub fn all_spotify_presets() -> IndexMap<String, PartnerPreset> {
    let mut presets = IndexMap::new();
    presets.insert("spotify_album".to_string(), spotify_album());
    presets.insert("spotify_single".to_string(), spotify_single());
    presets.insert("spotify_ep".to_string(), spotify_ep());
    presets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spotify_album_preset() {
        let preset = spotify_album();
        assert_eq!(preset.name, "spotify_album");
        assert_eq!(preset.config.version, DdexVersion::Ern43);
        assert_eq!(preset.config.profile, MessageProfile::AudioAlbum);
        assert!(preset.required_fields.contains(&"ISRC".to_string()));
        assert!(preset.required_fields.contains(&"UPC".to_string()));
    }

    #[test]
    fn test_spotify_single_preset() {
        let preset = spotify_single();
        assert_eq!(preset.name, "spotify_single");
        assert_eq!(preset.config.profile, MessageProfile::AudioSingle);
        assert!(preset.required_fields.contains(&"TrackTitle".to_string()));
    }

    #[test]
    fn test_spotify_ep_preset() {
        let preset = spotify_ep();
        assert_eq!(preset.name, "spotify_ep");
        assert!(preset.config.release_types.contains(&"EP".to_string()));
    }

    #[test]
    fn test_all_spotify_presets() {
        let presets = all_spotify_presets();
        assert_eq!(presets.len(), 3);
        assert!(presets.contains_key("spotify_album"));
        assert!(presets.contains_key("spotify_single"));
        assert!(presets.contains_key("spotify_ep"));
    }
}