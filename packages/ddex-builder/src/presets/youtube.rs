//! YouTube Music-specific DDEX presets and configurations

use super::{DdexVersion, MessageProfile, PresetConfig, PartnerPreset, PresetDefaults, PresetSource, ValidationRule};
use indexmap::IndexMap;

/// YouTube Album preset (ERN 4.2/4.3)
pub fn youtube_album() -> PartnerPreset {
    let mut validation_rules = IndexMap::new();
    validation_rules.insert("ISRC".to_string(), ValidationRule::Required);
    validation_rules.insert("UPC".to_string(), ValidationRule::Required);
    validation_rules.insert("ReleaseDate".to_string(), ValidationRule::Required);
    validation_rules.insert("Genre".to_string(), ValidationRule::Required);
    validation_rules.insert("ContentID".to_string(), ValidationRule::Required);
    validation_rules.insert("AssetType".to_string(), ValidationRule::OneOf(vec![
        "SoundRecording".to_string(),
        "Video".to_string(),
        "Image".to_string(),
    ]));
    validation_rules.insert("TerritoryCode".to_string(), ValidationRule::TerritoryCode { 
        allowed: vec!["Worldwide".to_string(), "WW".to_string()] 
    });
    validation_rules.insert("ReleaseType".to_string(), ValidationRule::OneOf(vec![
        "Album".to_string(),
        "CompilationAlbum".to_string(),
        "LiveAlbum".to_string(),
        "Soundtrack".to_string(),
    ]));
    
    let mut default_values = IndexMap::new();
    default_values.insert("MessageControlType".to_string(), "LiveMessage".to_string());
    default_values.insert("TerritoryCode".to_string(), "Worldwide".to_string());
    default_values.insert("DistributionChannel".to_string(), "02".to_string()); // Streaming
    default_values.insert("ReleaseType".to_string(), "Album".to_string());
    default_values.insert("ContentIDEnabled".to_string(), "true".to_string());
    
    let mut custom_mappings = IndexMap::new();
    custom_mappings.insert("ContentID".to_string(), "YouTubeContentID".to_string());
    custom_mappings.insert("AssetType".to_string(), "ResourceType".to_string());
    custom_mappings.insert("VideoMetadata".to_string(), "VideoDetails".to_string());
    custom_mappings.insert("ThumbnailImage".to_string(), "Image".to_string());
    
    let config = PresetConfig {
        version: DdexVersion::Ern43, // Supports both 4.2 and 4.3
        profile: MessageProfile::AudioAlbum,
        required_fields: vec![
            "ISRC".to_string(),
            "UPC".to_string(),
            "ReleaseDate".to_string(),
            "Genre".to_string(),
            "ContentID".to_string(),
            "AlbumTitle".to_string(),
            "ArtistName".to_string(),
            "TrackTitle".to_string(),
            "AssetType".to_string(),
        ],
        validation_rules: validation_rules.clone(),
        default_values,
        custom_mappings: custom_mappings.clone(),
        territory_codes: vec!["Worldwide".to_string()],
        distribution_channels: vec!["02".to_string()], // Streaming
        release_types: vec![
            "Album".to_string(), 
            "CompilationAlbum".to_string(), 
            "LiveAlbum".to_string(),
            "Soundtrack".to_string(),
        ],
    };

    PartnerPreset {
        name: "youtube_album".to_string(),
        description: "YouTube Music Album ERN 4.2/4.3 with Content ID requirements".to_string(),
        source: PresetSource::PublicDocs,
        provenance_url: Some("https://support.google.com/youtube/answer/1311402".to_string()),
        version: "1.0.0".to_string(),
        locked: false,
        disclaimer: "Based on YouTube Partner documentation. Content ID requirements may vary.".to_string(),
        determinism: super::super::determinism::DeterminismConfig::default(),
        defaults: PresetDefaults {
            message_control_type: Some("LiveMessage".to_string()),
            territory_code: vec!["Worldwide".to_string()],
            distribution_channel: vec!["02".to_string()],
        },
        required_fields: config.required_fields.clone(),
        format_overrides: IndexMap::new(),
        config,
        validation_rules,
        custom_mappings,
    }
}

/// YouTube Video preset (ERN 4.2/4.3) for video content with audio
pub fn youtube_video() -> PartnerPreset {
    let mut validation_rules = IndexMap::new();
    validation_rules.insert("ISRC".to_string(), ValidationRule::Required);
    validation_rules.insert("ISVN".to_string(), ValidationRule::Required); // International Standard Video Number
    validation_rules.insert("ReleaseDate".to_string(), ValidationRule::Required);
    validation_rules.insert("Genre".to_string(), ValidationRule::Required);
    validation_rules.insert("ContentID".to_string(), ValidationRule::Required);
    validation_rules.insert("VideoResource".to_string(), ValidationRule::Required);
    validation_rules.insert("AudioResource".to_string(), ValidationRule::Required);
    validation_rules.insert("AssetType".to_string(), ValidationRule::OneOf(vec![
        "Video".to_string(),
        "MusicVideo".to_string(),
    ]));
    validation_rules.insert("VideoQuality".to_string(), ValidationRule::OneOf(vec![
        "HD720".to_string(),
        "HD1080".to_string(),
        "4K".to_string(),
    ]));
    validation_rules.insert("TerritoryCode".to_string(), ValidationRule::TerritoryCode { 
        allowed: vec!["Worldwide".to_string(), "WW".to_string()] 
    });
    
    let mut default_values = IndexMap::new();
    default_values.insert("MessageControlType".to_string(), "LiveMessage".to_string());
    default_values.insert("TerritoryCode".to_string(), "Worldwide".to_string());
    default_values.insert("DistributionChannel".to_string(), "02".to_string());
    default_values.insert("AssetType".to_string(), "MusicVideo".to_string());
    default_values.insert("ContentIDEnabled".to_string(), "true".to_string());
    default_values.insert("VideoQuality".to_string(), "HD1080".to_string());
    
    let mut custom_mappings = IndexMap::new();
    custom_mappings.insert("ContentID".to_string(), "YouTubeContentID".to_string());
    custom_mappings.insert("VideoResource".to_string(), "VideoTechnicalResourceDetails".to_string());
    custom_mappings.insert("AudioResource".to_string(), "SoundRecordingTechnicalResourceDetails".to_string());
    custom_mappings.insert("VideoMetadata".to_string(), "VideoDetails".to_string());
    custom_mappings.insert("ISVN".to_string(), "VideoResourceReference".to_string());
    
    let config = PresetConfig {
        version: DdexVersion::Ern43,
        profile: MessageProfile::VideoSingle,
        required_fields: vec![
            "ISRC".to_string(),
            "ISVN".to_string(),
            "ReleaseDate".to_string(),
            "Genre".to_string(),
            "ContentID".to_string(),
            "VideoResource".to_string(),
            "AudioResource".to_string(),
            "VideoTitle".to_string(),
            "ArtistName".to_string(),
            "AssetType".to_string(),
            "VideoQuality".to_string(),
        ],
        validation_rules: validation_rules.clone(),
        default_values,
        custom_mappings: custom_mappings.clone(),
        territory_codes: vec!["Worldwide".to_string()],
        distribution_channels: vec!["02".to_string()],
        release_types: vec!["VideoSingle".to_string(), "MusicVideo".to_string()],
    };

    PartnerPreset {
        name: "youtube_video".to_string(),
        description: "YouTube Music Video ERN 4.2/4.3 with video resource handling".to_string(),
        source: PresetSource::PublicDocs,
        provenance_url: Some("https://support.google.com/youtube/answer/1311402".to_string()),
        version: "1.0.0".to_string(),
        locked: false,
        disclaimer: "Based on YouTube Partner documentation. Video encoding requirements may vary.".to_string(),
        determinism: super::super::determinism::DeterminismConfig::default(),
        defaults: PresetDefaults {
            message_control_type: Some("LiveMessage".to_string()),
            territory_code: vec!["Worldwide".to_string()],
            distribution_channel: vec!["02".to_string()],
        },
        required_fields: config.required_fields.clone(),
        format_overrides: IndexMap::new(),
        config,
        validation_rules,
        custom_mappings,
    }
}

/// YouTube Music Single preset (audio-only)
pub fn youtube_single() -> PartnerPreset {
    let mut preset = youtube_album();
    
    // Modify for single-specific settings
    preset.name = "youtube_single".to_string();
    preset.description = "YouTube Music Single ERN 4.2/4.3 audio-only release".to_string();
    preset.config.profile = MessageProfile::AudioSingle;
    preset.config.release_types = vec!["Single".to_string()];
    preset.config.default_values.insert("ReleaseType".to_string(), "Single".to_string());
    
    // Remove video-specific requirements
    preset.config.required_fields.retain(|field| field != "VideoResource");
    preset.validation_rules.shift_remove("VideoQuality");
    preset.custom_mappings.shift_remove("VideoResource");
    preset.custom_mappings.shift_remove("VideoMetadata");
    
    preset
}

/// Get all YouTube presets
pub fn all_youtube_presets() -> IndexMap<String, PartnerPreset> {
    let mut presets = IndexMap::new();
    presets.insert("youtube_album".to_string(), youtube_album());
    presets.insert("youtube_video".to_string(), youtube_video());
    presets.insert("youtube_single".to_string(), youtube_single());
    presets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_youtube_album_preset() {
        let preset = youtube_album();
        assert_eq!(preset.name, "youtube_album");
        assert_eq!(preset.config.version, DdexVersion::Ern43);
        assert_eq!(preset.config.profile, MessageProfile::AudioAlbum);
        assert!(preset.required_fields.contains(&"ContentID".to_string()));
        assert!(preset.required_fields.contains(&"ISRC".to_string()));
    }

    #[test]
    fn test_youtube_video_preset() {
        let preset = youtube_video();
        assert_eq!(preset.name, "youtube_video");
        assert_eq!(preset.config.profile, MessageProfile::VideoSingle);
        assert!(preset.required_fields.contains(&"ISVN".to_string()));
        assert!(preset.required_fields.contains(&"VideoResource".to_string()));
    }

    #[test]
    fn test_youtube_single_preset() {
        let preset = youtube_single();
        assert_eq!(preset.name, "youtube_single");
        assert_eq!(preset.config.profile, MessageProfile::AudioSingle);
        assert!(!preset.required_fields.contains(&"VideoResource".to_string()));
    }

    #[test]
    fn test_all_youtube_presets() {
        let presets = all_youtube_presets();
        assert_eq!(presets.len(), 3);
        assert!(presets.contains_key("youtube_album"));
        assert!(presets.contains_key("youtube_video"));
        assert!(presets.contains_key("youtube_single"));
    }

    #[test]
    fn test_content_id_requirements() {
        let album_preset = youtube_album();
        let video_preset = youtube_video();
        
        assert!(album_preset.required_fields.contains(&"ContentID".to_string()));
        assert!(video_preset.required_fields.contains(&"ContentID".to_string()));
        assert!(album_preset.custom_mappings.contains_key("ContentID"));
        assert!(video_preset.custom_mappings.contains_key("ContentID"));
    }
}