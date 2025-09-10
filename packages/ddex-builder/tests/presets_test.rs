//! Comprehensive tests for DDEX preset functionality

use ddex_builder::presets::{
    all_presets, spotify, youtube, DdexVersion, MessageProfile, ValidationRule, PartnerPreset, PresetConfig
};
use ddex_builder::{Builder, error::BuildError};
use indexmap::IndexMap;

#[test]
fn test_all_presets_loaded() {
    let presets = all_presets();
    
    // Should have at least the core presets
    assert!(presets.len() >= 7); // 2 legacy + 3 Spotify + 3 YouTube - 1 overlap
    
    // Check that key presets are present
    assert!(presets.contains_key("spotify_album"));
    assert!(presets.contains_key("spotify_single"));
    assert!(presets.contains_key("youtube_album"));
    assert!(presets.contains_key("youtube_video"));
    
    // Legacy presets should still be available
    assert!(presets.contains_key("spotify_audio_43"));
    assert!(presets.contains_key("apple_music_43"));
}

#[test]
fn test_spotify_presets() {
    let spotify_presets = spotify::all_spotify_presets();
    
    assert_eq!(spotify_presets.len(), 3);
    assert!(spotify_presets.contains_key("spotify_album"));
    assert!(spotify_presets.contains_key("spotify_single"));
    assert!(spotify_presets.contains_key("spotify_ep"));
    
    // Test album preset specifics
    let album = spotify_presets.get("spotify_album").unwrap();
    assert_eq!(album.config.version, DdexVersion::Ern43);
    assert_eq!(album.config.profile, MessageProfile::AudioAlbum);
    assert!(album.required_fields.contains(&"ISRC".to_string()));
    assert!(album.required_fields.contains(&"UPC".to_string()));
    assert!(album.required_fields.contains(&"ExplicitContent".to_string()));
    
    // Test audio quality validation
    assert!(album.validation_rules.contains_key("AudioQuality"));
    if let Some(ValidationRule::AudioQuality { min_bit_depth, min_sample_rate }) = 
        album.validation_rules.get("AudioQuality") {
        assert_eq!(*min_bit_depth, 16);
        assert_eq!(*min_sample_rate, 44100);
    } else {
        panic!("AudioQuality validation rule not found or incorrect type");
    }
    
    // Test territory code validation
    assert!(album.validation_rules.contains_key("TerritoryCode"));
    if let Some(ValidationRule::TerritoryCode { allowed }) = 
        album.validation_rules.get("TerritoryCode") {
        assert!(allowed.contains(&"Worldwide".to_string()));
        assert!(allowed.contains(&"WW".to_string()));
    }
}

#[test]
fn test_youtube_presets() {
    let youtube_presets = youtube::all_youtube_presets();
    
    assert_eq!(youtube_presets.len(), 3);
    assert!(youtube_presets.contains_key("youtube_album"));
    assert!(youtube_presets.contains_key("youtube_video"));
    assert!(youtube_presets.contains_key("youtube_single"));
    
    // Test video preset specifics
    let video = youtube_presets.get("youtube_video").unwrap();
    assert_eq!(video.config.version, DdexVersion::Ern43);
    assert_eq!(video.config.profile, MessageProfile::VideoSingle);
    assert!(video.required_fields.contains(&"ContentID".to_string()));
    assert!(video.required_fields.contains(&"ISVN".to_string()));
    assert!(video.required_fields.contains(&"VideoResource".to_string()));
    
    // Test video quality validation
    assert!(video.validation_rules.contains_key("VideoQuality"));
    if let Some(ValidationRule::OneOf(options)) = video.validation_rules.get("VideoQuality") {
        assert!(options.contains(&"HD720".to_string()));
        assert!(options.contains(&"HD1080".to_string()));
        assert!(options.contains(&"4K".to_string()));
    }
    
    // Test Content ID requirement
    assert!(video.required_fields.contains(&"ContentID".to_string()));
    assert!(video.custom_mappings.contains_key("ContentID"));
}

#[test]
fn test_preset_validation_rules() {
    let presets = all_presets();
    
    for (name, preset) in presets.iter() {
        // Every preset should have some validation rules
        assert!(!preset.validation_rules.is_empty(), 
                "Preset {} should have validation rules", name);
        
        // Required fields should have Required validation
        for field in &preset.required_fields {
            if field == "ISRC" || field == "UPC" {
                assert!(preset.validation_rules.contains_key(field), 
                        "Preset {} should have validation for required field {}", name, field);
            }
        }
        
        // Check version is set correctly
        match preset.config.version {
            DdexVersion::Ern382 | DdexVersion::Ern41 | DdexVersion::Ern42 | DdexVersion::Ern43 => {
                // Valid version
            }
        }
        
        // Check profile is set
        match preset.config.profile {
            MessageProfile::AudioAlbum | MessageProfile::AudioSingle | 
            MessageProfile::VideoAlbum | MessageProfile::VideoSingle | 
            MessageProfile::Mixed => {
                // Valid profile
            }
        }
    }
}

#[test]
fn test_builder_preset_integration() {
    let mut builder = Builder::new();
    
    // Test that all presets can be loaded
    let available = builder.available_presets();
    assert!(available.len() >= 7);
    
    // Test applying Spotify album preset
    assert!(builder.preset("spotify_album").is_ok());
    let spotify_preset = builder.get_preset("spotify_album").unwrap();
    assert_eq!(spotify_preset.name, "spotify_album");
    
    // Test applying YouTube video preset
    assert!(builder.preset("youtube_video").is_ok());
    let youtube_preset = builder.get_preset("youtube_video").unwrap();
    assert_eq!(youtube_preset.name, "youtube_video");
    
    // Test unknown preset returns error
    assert!(builder.preset("unknown_preset").is_err());
}

#[test]
fn test_preset_locking() {
    let mut builder = Builder::new();
    
    // Initially not locked
    assert!(!builder.is_preset_locked());
    
    // Apply preset without locking
    assert!(builder.apply_preset("spotify_album", false).is_ok());
    assert!(!builder.is_preset_locked());
    
    // Apply preset with locking
    assert!(builder.apply_preset("spotify_album", true).is_ok());
    assert!(builder.is_preset_locked());
}

#[test]
fn test_custom_mappings() {
    let spotify_album = spotify::spotify_album();
    let youtube_video = youtube::youtube_video();
    
    // Spotify should have explicit content mapping
    assert!(spotify_album.custom_mappings.contains_key("ExplicitContent"));
    assert_eq!(
        spotify_album.custom_mappings.get("ExplicitContent").unwrap(),
        "ParentalWarningType"
    );
    
    // YouTube should have Content ID mapping
    assert!(youtube_video.custom_mappings.contains_key("ContentID"));
    assert_eq!(
        youtube_video.custom_mappings.get("ContentID").unwrap(),
        "YouTubeContentID"
    );
    
    // YouTube should have video resource mappings
    assert!(youtube_video.custom_mappings.contains_key("VideoResource"));
    assert!(youtube_video.custom_mappings.contains_key("ISVN"));
}

#[test]
fn test_default_values() {
    let spotify_single = spotify::spotify_single();
    let youtube_album = youtube::youtube_album();
    
    // Spotify single should default to Single release type
    assert_eq!(
        spotify_single.config.default_values.get("ReleaseType").unwrap(),
        "Single"
    );
    
    // YouTube should default to streaming channel
    assert_eq!(
        youtube_album.config.default_values.get("DistributionChannel").unwrap(),
        "02" // Streaming
    );
    
    // Both should default to LiveMessage
    assert_eq!(
        spotify_single.config.default_values.get("MessageControlType").unwrap(),
        "LiveMessage"
    );
    assert_eq!(
        youtube_album.config.default_values.get("MessageControlType").unwrap(),
        "LiveMessage"
    );
}

#[test]
fn test_release_type_configurations() {
    let spotify_album = spotify::spotify_album();
    let spotify_single = spotify::spotify_single();
    let spotify_ep = spotify::spotify_ep();
    
    // Album should support album types
    assert!(spotify_album.config.release_types.contains(&"Album".to_string()));
    assert!(spotify_album.config.release_types.contains(&"CompilationAlbum".to_string()));
    
    // Single should support single types
    assert!(spotify_single.config.release_types.contains(&"Single".to_string()));
    assert!(spotify_single.config.release_types.contains(&"VideoSingle".to_string()));
    
    // EP should support EP type
    assert!(spotify_ep.config.release_types.contains(&"EP".to_string()));
}

#[test]
fn test_territory_codes() {
    let spotify_album = spotify::spotify_album();
    let youtube_video = youtube::youtube_video();
    
    // Both should support worldwide distribution
    assert!(spotify_album.config.territory_codes.contains(&"Worldwide".to_string()));
    assert!(youtube_video.config.territory_codes.contains(&"Worldwide".to_string()));
    
    // Territory validation should allow worldwide
    if let Some(ValidationRule::TerritoryCode { allowed }) = 
        spotify_album.validation_rules.get("TerritoryCode") {
        assert!(allowed.contains(&"Worldwide".to_string()));
        assert!(allowed.contains(&"WW".to_string()));
    }
}

#[test]
fn test_distribution_channels() {
    let spotify_album = spotify::spotify_album();
    let youtube_album = youtube::youtube_album();
    
    // Spotify should default to download/purchase channel
    assert!(spotify_album.config.distribution_channels.contains(&"01".to_string()));
    
    // YouTube should default to streaming channel
    assert!(youtube_album.config.distribution_channels.contains(&"02".to_string()));
}

#[test]
fn test_preset_provenance() {
    let presets = all_presets();
    
    for (name, preset) in presets.iter() {
        // Each preset should have a clear source
        match preset.source {
            ddex_builder::presets::PresetSource::PublicDocs |
            ddex_builder::presets::PresetSource::CustomerFeedback |
            ddex_builder::presets::PresetSource::Community => {
                // Valid source
            }
        }
        
        // Most presets should have provenance URLs
        if name.contains("spotify") {
            assert!(preset.provenance_url.is_some(), 
                    "Spotify preset {} should have provenance URL", name);
            assert!(preset.provenance_url.as_ref().unwrap().contains("spotify.com"));
        }
        
        if name.contains("youtube") {
            assert!(preset.provenance_url.is_some(), 
                    "YouTube preset {} should have provenance URL", name);
            assert!(preset.provenance_url.as_ref().unwrap().contains("google.com") ||
                    preset.provenance_url.as_ref().unwrap().contains("youtube"));
        }
        
        // All presets should have disclaimers
        assert!(!preset.disclaimer.is_empty(), 
                "Preset {} should have a disclaimer", name);
    }
}

#[test]  
fn test_validation_rule_types() {
    let spotify_album = spotify::spotify_album();
    let youtube_video = youtube::youtube_video();
    
    // Test different validation rule types
    for (field, rule) in &spotify_album.validation_rules {
        match rule {
            ValidationRule::Required => {
                assert!(["ISRC", "UPC", "ReleaseDate", "Genre", "ExplicitContent"]
                        .contains(&field.as_str()));
            }
            ValidationRule::AudioQuality { min_bit_depth, min_sample_rate } => {
                assert_eq!(*min_bit_depth, 16);
                assert_eq!(*min_sample_rate, 44100);
            }
            ValidationRule::TerritoryCode { allowed } => {
                assert!(allowed.contains(&"Worldwide".to_string()));
            }
            ValidationRule::OneOf(options) => {
                assert!(!options.is_empty());
            }
            _ => {} // Other rule types are valid
        }
    }
    
    // Test YouTube-specific validation rules
    assert!(youtube_video.validation_rules.contains_key("VideoQuality"));
    if let Some(ValidationRule::OneOf(qualities)) = 
        youtube_video.validation_rules.get("VideoQuality") {
        assert!(qualities.contains(&"HD1080".to_string()));
    }
}