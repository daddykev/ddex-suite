//! Custom preset template for creating organization-specific DDEX configurations
//! 
//! This template provides a well-documented starting point for building custom
//! presets tailored to your specific platform requirements or organizational standards.
//! 
//! ## Usage
//! 
//! 1. Copy this template to your own preset file
//! 2. Modify the configuration to match your requirements
//! 3. Test thoroughly with your platform submissions
//! 4. Document your sources and validation process
//! 
//! ## Example
//! 
//! ```rust
//! use ddex_builder::presets::custom_template;
//! 
//! // Create a custom preset based on the template
//! let mut my_preset = custom_template::create_custom_preset(
//!     "my_platform_album",
//!     "My Platform Album Requirements",
//!     MessageProfile::AudioAlbum
//! );
//! 
//! // Customize for your needs
//! my_preset.add_required_field("CustomPlatformID");
//! my_preset.add_validation_rule("Genre", ValidationRule::OneOf(vec!["Pop", "Rock"]));
//! ```

use super::{DdexVersion, MessageProfile, PresetConfig, PartnerPreset, PresetDefaults, PresetSource, ValidationRule};
use indexmap::IndexMap;

/// Template builder for creating custom presets
pub struct CustomPresetBuilder {
    preset: PartnerPreset,
}

impl CustomPresetBuilder {
    /// Create a new custom preset builder starting from a generic baseline
    pub fn new(name: String, description: String, profile: MessageProfile) -> Self {
        let mut validation_rules = IndexMap::new();
        
        // Start with essential DDEX requirements
        validation_rules.insert("ISRC".to_string(), ValidationRule::Required);
        validation_rules.insert("ReleaseDate".to_string(), ValidationRule::Required);
        validation_rules.insert("Genre".to_string(), ValidationRule::Required);
        validation_rules.insert("ArtistName".to_string(), ValidationRule::Required);
        
        // Add ISRC pattern validation
        validation_rules.insert("ISRC".to_string(), ValidationRule::Pattern(
            r"^[A-Z]{2}[A-Z0-9]{3}\d{7}$".to_string()
        ));
        
        // Add duration pattern validation
        validation_rules.insert("Duration".to_string(), ValidationRule::Pattern(
            r"^PT(\d+H)?(\d+M)?(\d+(\.\d+)?S)?$".to_string()
        ));
        
        let mut default_values = IndexMap::new();
        default_values.insert("MessageControlType".to_string(), "LiveMessage".to_string());
        
        // Add profile-specific defaults
        match profile {
            MessageProfile::AudioAlbum => {
                default_values.insert("ReleaseType".to_string(), "Album".to_string());
                validation_rules.insert("AlbumTitle".to_string(), ValidationRule::Required);
                validation_rules.insert("TrackTitle".to_string(), ValidationRule::Required);
            }
            MessageProfile::AudioSingle => {
                default_values.insert("ReleaseType".to_string(), "Single".to_string());
                validation_rules.insert("TrackTitle".to_string(), ValidationRule::Required);
            }
            MessageProfile::VideoSingle => {
                default_values.insert("ReleaseType".to_string(), "VideoSingle".to_string());
                validation_rules.insert("VideoTitle".to_string(), ValidationRule::Required);
                validation_rules.insert("VideoResource".to_string(), ValidationRule::Required);
                validation_rules.insert("AudioResource".to_string(), ValidationRule::Required);
            }
            _ => {}
        }
        
        let config = PresetConfig {
            version: DdexVersion::Ern43,
            profile,
            required_fields: vec![
                "ISRC".to_string(),
                "ReleaseDate".to_string(),
                "Genre".to_string(),
                "ArtistName".to_string(),
            ],
            validation_rules: validation_rules.clone(),
            default_values: default_values.clone(),
            custom_mappings: IndexMap::new(),
            territory_codes: vec!["Worldwide".to_string()],
            distribution_channels: vec!["01".to_string()], // Download/Stream
            release_types: vec!["Album".to_string(), "Single".to_string()],
        };

        let preset = PartnerPreset {
            name,
            description,
            source: PresetSource::CustomerFeedback, // Customize this
            provenance_url: None, // Add your documentation URL
            version: "1.0.0".to_string(),
            locked: false,
            disclaimer: "Custom preset template - customize for your specific requirements. Test thoroughly before production use.".to_string(),
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
        };

        Self { preset }
    }
    
    /// Add a required field to the preset
    pub fn add_required_field(&mut self, field: String) -> &mut Self {
        if !self.preset.required_fields.contains(&field) {
            self.preset.required_fields.push(field.clone());
            self.preset.config.required_fields.push(field.clone());
            self.preset.validation_rules.insert(field, ValidationRule::Required);
        }
        self
    }
    
    /// Add a validation rule
    pub fn add_validation_rule(&mut self, field: String, rule: ValidationRule) -> &mut Self {
        self.preset.validation_rules.insert(field.clone(), rule.clone());
        self.preset.config.validation_rules.insert(field, rule);
        self
    }
    
    /// Set a default value
    pub fn set_default(&mut self, field: String, value: String) -> &mut Self {
        self.preset.config.default_values.insert(field, value);
        self
    }
    
    /// Add a custom field mapping
    pub fn add_custom_mapping(&mut self, source: String, target: String) -> &mut Self {
        self.preset.custom_mappings.insert(source.clone(), target.clone());
        self.preset.config.custom_mappings.insert(source, target);
        self
    }
    
    /// Set allowed territories
    pub fn set_territories(&mut self, territories: Vec<String>) -> &mut Self {
        self.preset.config.territory_codes = territories.clone();
        self.preset.defaults.territory_code = territories;
        self
    }
    
    /// Set distribution channels
    pub fn set_distribution_channels(&mut self, channels: Vec<String>) -> &mut Self {
        self.preset.config.distribution_channels = channels.clone();
        self.preset.defaults.distribution_channel = channels;
        self
    }
    
    /// Set release types
    pub fn set_release_types(&mut self, types: Vec<String>) -> &mut Self {
        self.preset.config.release_types = types;
        self
    }
    
    /// Set source and provenance information
    pub fn set_source(&mut self, source: PresetSource, url: Option<String>) -> &mut Self {
        self.preset.source = source;
        self.preset.provenance_url = url;
        self
    }
    
    /// Set disclaimer
    pub fn set_disclaimer(&mut self, disclaimer: String) -> &mut Self {
        self.preset.disclaimer = disclaimer;
        self
    }
    
    /// Set version
    pub fn set_version(&mut self, version: String) -> &mut Self {
        self.preset.version = version;
        self
    }
    
    /// Lock the preset to prevent modifications
    pub fn lock(&mut self) -> &mut Self {
        self.preset.locked = true;
        self
    }
    
    /// Build the final preset
    pub fn build(self) -> PartnerPreset {
        self.preset
    }
}

/// Create a custom audio album preset
pub fn create_audio_album_preset(
    name: String, 
    description: String
) -> CustomPresetBuilder {
    let mut builder = CustomPresetBuilder::new(name, description, MessageProfile::AudioAlbum);
    
    // Audio album specific defaults
    builder.add_required_field("AlbumTitle".to_string());
    builder.add_required_field("TrackTitle".to_string());
    builder.add_validation_rule(
        "ReleaseType".to_string(),
        ValidationRule::OneOf(vec![
            "Album".to_string(),
            "CompilationAlbum".to_string(),
            "LiveAlbum".to_string()
        ])
    );
    
    builder
}

/// Create a custom audio single preset  
pub fn create_audio_single_preset(
    name: String,
    description: String
) -> CustomPresetBuilder {
    let mut builder = CustomPresetBuilder::new(name, description, MessageProfile::AudioSingle);
    
    // Audio single specific defaults
    builder.add_required_field("TrackTitle".to_string());
    builder.set_default("ReleaseType".to_string(), "Single".to_string());
    builder.set_release_types(vec!["Single".to_string()]);
    
    builder
}

/// Create a custom video single preset
pub fn create_video_single_preset(
    name: String,
    description: String  
) -> CustomPresetBuilder {
    let mut builder = CustomPresetBuilder::new(name, description, MessageProfile::VideoSingle);
    
    // Video single specific defaults
    builder.add_required_field("VideoTitle".to_string());
    builder.add_required_field("VideoResource".to_string());
    builder.add_required_field("AudioResource".to_string());
    builder.add_custom_mapping("VideoResource".to_string(), "VideoTechnicalResourceDetails".to_string());
    builder.add_custom_mapping("AudioResource".to_string(), "SoundRecordingTechnicalResourceDetails".to_string());
    builder.set_default("ReleaseType".to_string(), "VideoSingle".to_string());
    builder.set_distribution_channels(vec!["01".to_string(), "02".to_string()]); // Download + Streaming
    
    builder
}

/// Example: Platform-specific preset
/// 
/// This demonstrates how to create a preset for a fictional "MusicPlatform X"
/// based on integration testing and support documentation.
pub fn example_platform_x_album() -> PartnerPreset {
    create_audio_album_preset(
        "platform_x_album".to_string(),
        "Platform X Album Requirements - Based on Integration Testing".to_string()
    )
    // Add platform-specific requirements
    .add_required_field("UPC".to_string())
    .add_required_field("ExplicitContent".to_string())
    .add_validation_rule(
        "AudioQuality".to_string(),
        ValidationRule::AudioQuality { min_bit_depth: 16, min_sample_rate: 44100 }
    )
    .add_validation_rule(
        "Genre".to_string(),
        ValidationRule::OneOf(vec![
            "Pop".to_string(),
            "Rock".to_string(), 
            "Hip-Hop".to_string(),
            "Electronic".to_string(),
            "Classical".to_string()
        ])
    )
    .add_custom_mapping("ExplicitContent".to_string(), "ParentalWarningType".to_string())
    .set_territories(vec!["US".to_string(), "CA".to_string(), "GB".to_string()])
    .set_distribution_channels(vec!["01".to_string()]) // Download only
    .set_source(
        PresetSource::CustomerFeedback,
        Some("https://your-company.com/platform-x-integration".to_string())
    )
    .set_disclaimer(
        "Based on Platform X integration testing and support feedback. \
         Not an official specification. Verify current requirements with Platform X support.".to_string()
    )
    .set_version("1.2.0".to_string())
    .build()
}

/// Example: Label-specific preset
/// 
/// This demonstrates how to create a preset for internal label standards
pub fn example_record_label_standard() -> PartnerPreset {
    create_audio_album_preset(
        "our_label_standard".to_string(), 
        "Our Record Label Internal Standards".to_string()
    )
    // Add label-specific requirements
    .add_required_field("ISWC".to_string()) // For publishing
    .add_required_field("LabelName".to_string())
    .add_required_field("CopyrightYear".to_string())
    .add_validation_rule(
        "ISWC".to_string(),
        ValidationRule::Pattern(r"^T-\d{3}\.\d{3}\.\d{3}-\d$".to_string())
    )
    .add_validation_rule(
        "LabelName".to_string(),
        ValidationRule::OneOf(vec!["Our Main Label".to_string(), "Our Sub Label".to_string()])
    )
    .add_validation_rule(
        "Duration".to_string(),
        ValidationRule::Custom("Must be between 30 seconds and 10 minutes".to_string())
    )
    .set_territories(vec!["Worldwide".to_string()])
    .set_distribution_channels(vec!["01".to_string(), "02".to_string()]) // Download + Streaming
    .set_source(
        PresetSource::CustomerFeedback,
        Some("https://our-label.com/internal-standards".to_string())
    )
    .set_disclaimer(
        "Internal record label standards. Based on our quality requirements and distribution agreements.".to_string()
    )
    .set_version("2.0.1".to_string())
    .lock() // Prevent accidental modifications
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_preset_builder() {
        let preset = create_audio_album_preset(
            "test_preset".to_string(),
            "Test Preset".to_string()
        )
        .add_required_field("CustomField".to_string())
        .add_validation_rule("CustomField".to_string(), ValidationRule::Required)
        .set_default("CustomDefault".to_string(), "TestValue".to_string())
        .build();
        
        assert_eq!(preset.name, "test_preset");
        assert_eq!(preset.config.profile, MessageProfile::AudioAlbum);
        assert!(preset.required_fields.contains(&"CustomField".to_string()));
        assert!(preset.validation_rules.contains_key("CustomField"));
        assert_eq!(preset.config.default_values.get("CustomDefault"), Some(&"TestValue".to_string()));
    }
    
    #[test]
    fn test_example_platform_preset() {
        let preset = example_platform_x_album();
        
        assert_eq!(preset.name, "platform_x_album");
        assert_eq!(preset.source, PresetSource::CustomerFeedback);
        assert!(preset.required_fields.contains(&"UPC".to_string()));
        assert!(preset.required_fields.contains(&"ExplicitContent".to_string()));
        assert!(preset.validation_rules.contains_key("AudioQuality"));
        assert!(preset.custom_mappings.contains_key("ExplicitContent"));
    }
    
    #[test]
    fn test_label_standard_preset() {
        let preset = example_record_label_standard();
        
        assert_eq!(preset.name, "our_label_standard");
        assert!(preset.locked);
        assert!(preset.required_fields.contains(&"ISWC".to_string()));
        assert!(preset.required_fields.contains(&"LabelName".to_string()));
        assert!(preset.validation_rules.contains_key("ISWC"));
    }
    
    #[test]
    fn test_all_message_profiles() {
        let audio_album = create_audio_album_preset("test1".to_string(), "Test 1".to_string()).build();
        let audio_single = create_audio_single_preset("test2".to_string(), "Test 2".to_string()).build();
        let video_single = create_video_single_preset("test3".to_string(), "Test 3".to_string()).build();
        
        assert_eq!(audio_album.config.profile, MessageProfile::AudioAlbum);
        assert_eq!(audio_single.config.profile, MessageProfile::AudioSingle);
        assert_eq!(video_single.config.profile, MessageProfile::VideoSingle);
        
        // Check profile-specific fields
        assert!(audio_album.required_fields.contains(&"AlbumTitle".to_string()));
        assert!(!audio_single.required_fields.contains(&"AlbumTitle".to_string()));
        assert!(video_single.required_fields.contains(&"VideoResource".to_string()));
    }
}