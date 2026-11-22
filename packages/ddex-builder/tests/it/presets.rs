//! Comprehensive tests for DDEX preset functionality

use ddex_builder::presets::{
    all_presets, generic, youtube, DdexVersion, MessageProfile, PresetSource, ValidationRule,
};
use ddex_builder::Builder;

#[test]
fn test_all_presets_loaded() {
    let presets = all_presets();

    // Should have generic presets + YouTube presets
    assert!(presets.len() >= 7); // 4 generic + 3 YouTube

    // Check that generic presets are present
    assert!(presets.contains_key("audio_album"));
    assert!(presets.contains_key("audio_single"));
    assert!(presets.contains_key("video_single"));
    assert!(presets.contains_key("compilation"));

    // Check that YouTube presets are present
    assert!(presets.contains_key("youtube_album"));
    assert!(presets.contains_key("youtube_video"));
    assert!(presets.contains_key("youtube_single"));
}

#[test]
fn test_generic_presets() {
    let generic_presets = generic::all_generic_presets();

    assert_eq!(generic_presets.len(), 4);
    assert!(generic_presets.contains_key("audio_album"));
    assert!(generic_presets.contains_key("audio_single"));
    assert!(generic_presets.contains_key("video_single"));
    assert!(generic_presets.contains_key("compilation"));

    // Test audio album preset specifics
    let audio_album = generic_presets.get("audio_album").unwrap();
    assert_eq!(audio_album.config.version, DdexVersion::Ern43);
    assert_eq!(audio_album.config.profile, MessageProfile::AudioAlbum);
    assert_eq!(audio_album.source, PresetSource::Community);
    assert!(audio_album.required_fields.contains(&"ISRC".to_string()));
    assert!(audio_album
        .required_fields
        .contains(&"AlbumTitle".to_string()));
    assert!(audio_album
        .required_fields
        .contains(&"ArtistName".to_string()));

    // Test audio single preset specifics
    let audio_single = generic_presets.get("audio_single").unwrap();
    assert_eq!(audio_single.config.profile, MessageProfile::AudioSingle);
    assert!(audio_single
        .required_fields
        .contains(&"TrackTitle".to_string()));
    assert!(!audio_single
        .required_fields
        .contains(&"AlbumTitle".to_string()));

    // Test video single preset specifics
    let video_single = generic_presets.get("video_single").unwrap();
    assert_eq!(video_single.config.profile, MessageProfile::VideoSingle);
    assert!(video_single
        .required_fields
        .contains(&"VideoResource".to_string()));
    assert!(video_single
        .required_fields
        .contains(&"AudioResource".to_string()));

    // Test compilation preset specifics
    let compilation = generic_presets.get("compilation").unwrap();
    assert!(compilation
        .config
        .release_types
        .contains(&"CompilationAlbum".to_string()));
    assert!(compilation
        .required_fields
        .contains(&"CompilationIndicator".to_string()));
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
    assert_eq!(video.source, PresetSource::PublicDocs);
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

    // Test YouTube album preset
    let album = youtube_presets.get("youtube_album").unwrap();
    assert_eq!(album.config.profile, MessageProfile::AudioAlbum);
    assert!(album.required_fields.contains(&"ContentID".to_string()));
}

#[test]
fn test_preset_validation_rules() {
    let presets = all_presets();

    for (name, preset) in presets.iter() {
        // Every preset should have some validation rules
        assert!(
            !preset.validation_rules.is_empty(),
            "Preset {} should have validation rules",
            name
        );

        // Required fields should have corresponding validation where appropriate
        for field in &preset.required_fields {
            if field == "ISRC" {
                // ISRC should have pattern validation
                assert!(
                    preset.validation_rules.contains_key(field),
                    "Preset {} should have validation for required field {}",
                    name,
                    field
                );
                if let Some(ValidationRule::Pattern(pattern)) = preset.validation_rules.get(field) {
                    assert!(pattern.contains("[A-Z]{2}[A-Z0-9]{3}"));
                }
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
            MessageProfile::AudioAlbum
            | MessageProfile::AudioSingle
            | MessageProfile::VideoAlbum
            | MessageProfile::VideoSingle
            | MessageProfile::Mixed => {
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

    // Test applying generic audio album preset
    assert!(builder.preset("audio_album").is_ok());
    let audio_preset = builder.get_preset("audio_album").unwrap();
    assert_eq!(audio_preset.name, "audio_album");

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
    assert!(builder.apply_preset("audio_album", false).is_ok());
    assert!(!builder.is_preset_locked());

    // Apply preset with locking
    assert!(builder.apply_preset("audio_album", true).is_ok());
    assert!(builder.is_preset_locked());
}

#[test]
fn test_custom_mappings() {
    let generic_video = generic::video_single();
    let youtube_video = youtube::youtube_video();

    // Generic video should have basic resource mappings
    assert!(generic_video.custom_mappings.contains_key("VideoResource"));
    assert_eq!(
        generic_video.custom_mappings.get("VideoResource").unwrap(),
        "VideoTechnicalResourceDetails"
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
    let generic_single = generic::audio_single();
    let youtube_album = youtube::youtube_album();

    // Generic single should default to Single release type
    assert_eq!(
        generic_single
            .config
            .default_values
            .get("ReleaseType")
            .unwrap(),
        "Single"
    );

    // YouTube should default to streaming channel
    assert_eq!(
        youtube_album
            .config
            .default_values
            .get("DistributionChannel")
            .unwrap(),
        "02" // Streaming
    );

    // Both should default to LiveMessage
    assert_eq!(
        generic_single
            .config
            .default_values
            .get("MessageControlType")
            .unwrap(),
        "LiveMessage"
    );
    assert_eq!(
        youtube_album
            .config
            .default_values
            .get("MessageControlType")
            .unwrap(),
        "LiveMessage"
    );
}

#[test]
fn test_release_type_configurations() {
    let generic_album = generic::audio_album();
    let generic_single = generic::audio_single();
    let generic_compilation = generic::compilation();

    // Album should support album types
    assert!(generic_album
        .config
        .release_types
        .contains(&"Album".to_string()));
    assert!(generic_album
        .config
        .release_types
        .contains(&"CompilationAlbum".to_string()));

    // Single should support single types
    assert!(generic_single
        .config
        .release_types
        .contains(&"Single".to_string()));

    // Compilation should support compilation type
    assert!(generic_compilation
        .config
        .release_types
        .contains(&"CompilationAlbum".to_string()));
}

#[test]
fn test_territory_codes() {
    let generic_album = generic::audio_album();
    let youtube_video = youtube::youtube_video();

    // Both should support worldwide distribution
    assert!(generic_album
        .config
        .territory_codes
        .contains(&"Worldwide".to_string()));
    assert!(youtube_video
        .config
        .territory_codes
        .contains(&"Worldwide".to_string()));

    // YouTube should have territory validation
    if let Some(ValidationRule::TerritoryCode { allowed }) =
        youtube_video.validation_rules.get("TerritoryCode")
    {
        assert!(allowed.contains(&"Worldwide".to_string()));
        assert!(allowed.contains(&"WW".to_string()));
    }
}

#[test]
fn test_distribution_channels() {
    let generic_album = generic::audio_album();
    let youtube_album = youtube::youtube_album();

    // Generic should default to download channel
    assert!(generic_album
        .config
        .distribution_channels
        .contains(&"01".to_string()));

    // YouTube should default to streaming channel
    assert!(youtube_album
        .config
        .distribution_channels
        .contains(&"02".to_string()));
}

#[test]
fn test_preset_provenance() {
    let presets = all_presets();

    for (name, preset) in presets.iter() {
        // Each preset should have a clear source
        match preset.source {
            PresetSource::PublicDocs | PresetSource::CustomerFeedback | PresetSource::Community => {
                // Valid source
            }
        }

        // Generic presets should be community-maintained
        if name.starts_with("audio_") || name == "video_single" || name == "compilation" {
            assert_eq!(preset.source, PresetSource::Community);
            assert!(preset.provenance_url.as_ref().unwrap().contains("ddex.net"));
        }

        // YouTube presets should have public docs provenance
        if name.contains("youtube") {
            assert_eq!(preset.source, PresetSource::PublicDocs);
            assert!(
                preset.provenance_url.is_some(),
                "YouTube preset {} should have provenance URL",
                name
            );
            assert!(
                preset
                    .provenance_url
                    .as_ref()
                    .unwrap()
                    .contains("google.com")
                    || preset.provenance_url.as_ref().unwrap().contains("youtube")
            );
        }

        // All presets should have disclaimers
        assert!(
            !preset.disclaimer.is_empty(),
            "Preset {} should have a disclaimer",
            name
        );

        // All presets should have proper disclaimers indicating their nature
        if name.contains("youtube") {
            assert!(preset.disclaimer.contains("community-maintained"));
            assert!(preset.disclaimer.contains("not an official"));
        }

        if preset.source == PresetSource::Community {
            assert!(preset.disclaimer.contains("Generic industry-standard"));
        }
    }
}

#[test]
fn test_validation_rule_types() {
    let generic_album = generic::audio_album();
    let youtube_video = youtube::youtube_video();

    // Test different validation rule types in generic presets
    for (field, rule) in &generic_album.validation_rules {
        match rule {
            ValidationRule::Required => {
                assert!([
                    "ISRC",
                    "ReleaseDate",
                    "Genre",
                    "AlbumTitle",
                    "ArtistName",
                    "TrackTitle"
                ]
                .contains(&field.as_str()));
            }
            ValidationRule::Pattern(pattern) => {
                if field == "ISRC" {
                    assert!(pattern.contains("[A-Z]{2}[A-Z0-9]{3}"));
                }
            }
            _ => {} // Other rule types are valid
        }
    }

    // Test YouTube-specific validation rules
    assert!(youtube_video.validation_rules.contains_key("VideoQuality"));
    if let Some(ValidationRule::OneOf(qualities)) =
        youtube_video.validation_rules.get("VideoQuality")
    {
        assert!(qualities.contains(&"HD1080".to_string()));
    }
}

#[test]
fn test_generic_preset_baseline_compliance() {
    let generic_presets = generic::all_generic_presets();

    for (name, preset) in generic_presets {
        // All generic presets should be ERN 4.3
        assert_eq!(preset.config.version, DdexVersion::Ern43);

        // All should have ISRC validation
        assert!(preset.validation_rules.contains_key("ISRC"));
        assert!(preset.required_fields.contains(&"ISRC".to_string()));

        // All should have basic required fields
        assert!(preset.required_fields.contains(&"ReleaseDate".to_string()));
        assert!(preset.required_fields.contains(&"Genre".to_string()));
        assert!(preset.required_fields.contains(&"ArtistName".to_string()));

        // All should have proper DDEX baseline defaults
        assert_eq!(
            preset
                .config
                .default_values
                .get("MessageControlType")
                .unwrap(),
            "LiveMessage"
        );

        // All should support worldwide territory
        assert!(preset
            .config
            .territory_codes
            .contains(&"Worldwide".to_string()));

        println!(
            "✅ Generic preset '{}' passes baseline DDEX compliance",
            name
        );
    }
}

#[test]
fn test_youtube_preset_public_doc_compliance() {
    let youtube_presets = youtube::all_youtube_presets();

    for (name, preset) in youtube_presets {
        // All YouTube presets should have PublicDocs source
        assert_eq!(preset.source, PresetSource::PublicDocs);

        // All should have Content ID requirements
        assert!(preset.required_fields.contains(&"ContentID".to_string()));

        // All should have proper disclaimer
        assert!(preset
            .disclaimer
            .contains("publicly available YouTube Partner documentation"));
        assert!(preset.disclaimer.contains("community-maintained"));
        assert!(preset
            .disclaimer
            .contains("not an official YouTube specification"));

        // All should have YouTube-specific mappings
        assert!(preset.custom_mappings.contains_key("ContentID"));

        // All should default to streaming channel
        assert!(preset
            .config
            .distribution_channels
            .contains(&"02".to_string()));

        println!(
            "✅ YouTube preset '{}' passes public documentation compliance",
            name
        );
    }
}

#[test]
fn test_no_speculative_platform_presets() {
    let presets = all_presets();

    // Ensure no speculative platform presets remain
    let speculative_names = [
        "spotify",
        "apple",
        "amazon",
        "deezer",
        "tidal",
        "pandora",
        "universal",
        "sony",
        "warner",
        "distrokid",
        "tunecore",
    ];

    for preset_name in presets.keys() {
        for speculative in &speculative_names {
            assert!(!preset_name.to_lowercase().contains(speculative),
                    "Found speculative preset '{}' - only YouTube (public docs) and generic presets should exist", 
                    preset_name);
        }
    }

    println!("✅ No speculative platform presets found - only YouTube + Generic");
}
