# DDEX Builder Preset Architecture

## Overview

The DDEX Builder preset system provides community-maintained configuration templates that ensure DDEX compliance while allowing customization for specific platforms or organizational needs. This document explains the architecture, philosophy, and best practices behind the preset system.

## Design Philosophy

### 🎯 Honesty First
- **Only include presets with verifiable public specifications**
- **Clear source attribution and provenance tracking**
- **Honest disclaimers about community-maintained status**

### 🏗️ Solid Foundation
- **Generic presets provide DDEX-compliant baselines**
- **Platform presets only for public documentation**
- **Custom framework for organization-specific needs**

### 🔒 Legal & Ethical
- **No reverse engineering of confidential specifications**
- **Respect platform intellectual property**
- **Community-driven approach with proper disclaimers**

## Architecture Overview

```
DDEX Builder Preset System
├── Generic Presets (Community-maintained)
│   ├── audio_album      - DDEX ERN 4.3 audio album baseline
│   ├── audio_single     - DDEX ERN 4.3 single track baseline
│   ├── video_single     - DDEX ERN 4.3 video release baseline
│   └── compilation      - DDEX ERN 4.3 compilation baseline
│
├── Platform Presets (Public documentation only)
│   ├── youtube_album    - Based on YouTube Partner docs
│   ├── youtube_video    - Based on YouTube Partner docs
│   └── youtube_single   - Based on YouTube Partner docs
│
└── Custom Framework
    ├── CustomPresetBuilder  - Template builder
    ├── Validation system    - Rule enforcement
    └── Examples             - Real-world templates
```

## Preset Components

### Core Structure

```rust
pub struct PartnerPreset {
    // Identification
    pub name: String,
    pub description: String,
    pub version: String,
    
    // Provenance & Trust
    pub source: PresetSource,           // PublicDocs | Community | CustomerFeedback
    pub provenance_url: Option<String>, // Source documentation URL
    pub disclaimer: String,             // Clear usage guidance
    pub locked: bool,                   // Prevent modifications
    
    // Configuration
    pub config: PresetConfig,           // DDEX configuration
    pub determinism: DeterminismConfig, // Output consistency
    pub defaults: PresetDefaults,       // Field defaults
    
    // Validation & Mapping
    pub required_fields: Vec<String>,
    pub validation_rules: IndexMap<String, ValidationRule>,
    pub custom_mappings: IndexMap<String, String>,
    pub format_overrides: IndexMap<String, String>,
}
```

### Validation Rules

The preset system supports comprehensive validation:

```rust
pub enum ValidationRule {
    Required,                                    // Field must be present
    MinLength(usize),                           // Minimum string length
    MaxLength(usize),                           // Maximum string length
    Pattern(String),                            // Regex pattern match
    OneOf(Vec<String>),                         // Value from allowed list
    AudioQuality { min_bit_depth: u8, min_sample_rate: u32 }, // Audio specs
    TerritoryCode { allowed: Vec<String> },     // Territory restrictions
    Custom(String),                             // Custom validation description
}
```

### Configuration Structure

```rust
pub struct PresetConfig {
    pub version: DdexVersion,              // ERN 3.8.2, 4.2, 4.3, etc.
    pub profile: MessageProfile,          // AudioAlbum, VideoSingle, etc.
    pub required_fields: Vec<String>,     // Must-have fields
    pub validation_rules: IndexMap<String, ValidationRule>,
    pub default_values: IndexMap<String, String>,
    pub custom_mappings: IndexMap<String, String>,
    pub territory_codes: Vec<String>,     // Allowed territories
    pub distribution_channels: Vec<String>, // Distribution channels
    pub release_types: Vec<String>,       // Supported release types
}
```

## Generic Presets

### Purpose
Generic presets provide DDEX-compliant baseline configurations that work across most platforms. They ensure structural correctness while allowing customization for specific needs.

### Available Generic Presets

#### `audio_album`
- **Purpose**: Complete audio album releases
- **Profile**: `MessageProfile::AudioAlbum`
- **Required**: ISRC, Album Title, Artist Name, Track Title, Release Date, Genre
- **Validation**: ISRC pattern, ISO8601 dates, PT duration format
- **Use Case**: Multi-track audio releases, compilations, EPs

#### `audio_single` 
- **Purpose**: Single track audio releases
- **Profile**: `MessageProfile::AudioSingle`
- **Required**: ISRC, Track Title, Artist Name, Release Date, Genre
- **Simplified**: No album-specific requirements
- **Use Case**: Single song releases, promotional tracks

#### `video_single`
- **Purpose**: Video releases with synchronized audio
- **Profile**: `MessageProfile::VideoSingle`
- **Required**: ISRC, Video Title, Artist Name, Video Resource, Audio Resource
- **Mapping**: VideoTechnicalResourceDetails, SoundRecordingTechnicalResourceDetails
- **Use Case**: Music videos, video singles, promotional videos

#### `compilation`
- **Purpose**: Multi-artist compilation albums
- **Profile**: `MessageProfile::AudioAlbum`
- **Special**: CompilationIndicator required, VariousArtists support
- **Use Case**: Various artist compilations, label samplers

### Generic Preset Characteristics

```rust
// All generic presets share these characteristics:
pub source: PresetSource::Community,
pub provenance_url: Some("https://ddex.net/standards/"),
pub disclaimer: "Generic industry-standard preset based on DDEX ERN 4.3 specification. Customize for specific platform requirements.",

// Common validation rules:
validation_rules.insert("ISRC", ValidationRule::Pattern(r"^[A-Z]{2}[A-Z0-9]{3}\d{7}$"));
validation_rules.insert("Duration", ValidationRule::Pattern(r"^PT(\d+H)?(\d+M)?(\d+(\.\d+)?S)?$"));

// Common defaults:
default_values.insert("MessageControlType", "LiveMessage");
default_values.insert("TerritoryCode", "Worldwide");
```

## Platform Presets

### Inclusion Criteria
Platform presets are only included when:
1. **Public documentation is available** from the platform
2. **DDEX specifications are officially published** 
3. **Requirements are verifiable** and not based on speculation
4. **Documentation is maintained** and reasonably current

### Current Platform Presets

#### YouTube Music Presets
Based on publicly available YouTube Partner documentation:

**`youtube_album`**
- **Source**: YouTube Partner Program documentation
- **URL**: https://support.google.com/youtube/answer/1311402
- **Requirements**: Content ID, ISRC, UPC, Territory codes
- **Channel**: Streaming (02)
- **Disclaimer**: "Based on publicly available YouTube Partner documentation. This preset is community-maintained and not an official YouTube specification."

**`youtube_video`**
- **Additional**: ISVN (International Standard Video Number), Video quality specs
- **Video Quality**: HD720, HD1080, 4K options
- **Resources**: VideoTechnicalResourceDetails, SoundRecordingTechnicalResourceDetails

**`youtube_single`**
- **Simplified**: Audio-only requirements, no video resources
- **Based on**: YouTube album preset with video elements removed

### Platform Preset Characteristics

```rust
// All YouTube presets share:
pub source: PresetSource::PublicDocs,
pub provenance_url: Some("https://support.google.com/youtube/answer/1311402"),
pub disclaimer: "Based on publicly available YouTube Partner documentation...",

// YouTube-specific requirements:
required_fields.push("ContentID");
custom_mappings.insert("ContentID", "YouTubeContentID");
default_values.insert("DistributionChannel", "02"); // Streaming
```

## Custom Preset Framework

### CustomPresetBuilder

The `CustomPresetBuilder` provides a fluent API for creating organization-specific presets:

```rust
use ddex_builder::presets::custom_template::CustomPresetBuilder;

let preset = CustomPresetBuilder::new(
    "my_platform_album".to_string(),
    "My Platform Album Requirements".to_string(),
    MessageProfile::AudioAlbum
)
.add_required_field("PlatformID".to_string())
.add_validation_rule(
    "AudioQuality".to_string(),
    ValidationRule::AudioQuality { min_bit_depth: 24, min_sample_rate: 96000 }
)
.set_territories(vec!["US".to_string(), "CA".to_string()])
.set_source(PresetSource::CustomerFeedback, Some("https://my-company.com/docs"))
.set_disclaimer("Based on integration testing with Platform X. Not official.")
.build();
```

### Builder Methods

#### Core Configuration
- `add_required_field(field)` - Make field mandatory
- `add_validation_rule(field, rule)` - Add validation constraint
- `set_default(field, value)` - Set default value
- `add_custom_mapping(source, target)` - Map field names

#### Geographic & Distribution
- `set_territories(territories)` - Restrict to specific territories  
- `set_distribution_channels(channels)` - Set distribution methods
- `set_release_types(types)` - Supported release types

#### Metadata & Provenance  
- `set_source(source, url)` - Document preset source
- `set_disclaimer(text)` - Add usage guidance
- `set_version(version)` - Version the preset
- `lock()` - Prevent modifications

### Templates & Examples

#### Platform Integration Template
```rust
pub fn platform_x_album() -> PartnerPreset {
    create_audio_album_preset("platform_x_album", "Platform X Album Requirements")
        .add_required_field("PlatformCustomID".to_string())
        .add_validation_rule("Genre", ValidationRule::OneOf(platform_x_genres()))
        .set_territories(platform_x_territories())
        .set_source(
            PresetSource::CustomerFeedback,
            Some("https://internal-wiki.com/platform-x-integration")
        )
        .set_disclaimer("Based on Platform X integration testing and support feedback.")
        .build()
}
```

#### Label Standards Template
```rust
pub fn label_standards() -> PartnerPreset {
    create_audio_album_preset("our_label_standard", "Our Record Label Standards")
        .add_required_field("ISWC".to_string())          // Publishing requirement
        .add_required_field("LabelCatalogNumber".to_string())
        .add_validation_rule("Duration", duration_limits())
        .add_validation_rule("Quality", our_quality_standards())
        .set_source(PresetSource::CustomerFeedback, Some("https://our-label.com/standards"))
        .lock() // Prevent accidental modification
        .build()
}
```

## Preset Lifecycle

### Development Process

1. **Requirements Gathering**
   - Document platform requirements from official sources
   - Test with real submissions when possible
   - Gather community feedback

2. **Implementation**
   - Start with appropriate generic baseline
   - Add platform-specific requirements
   - Document sources and assumptions
   - Write comprehensive tests

3. **Validation**
   - Test with real-world data
   - Validate against platform submissions
   - Community review and feedback
   - Documentation review

4. **Maintenance**
   - Monitor for platform changes
   - Update based on community feedback
   - Version appropriately
   - Deprecate when necessary

### Versioning Strategy

Presets use semantic versioning:
- **Major (2.0.0)**: Breaking changes to structure or requirements
- **Minor (1.1.0)**: New optional fields or relaxed constraints
- **Patch (1.0.1)**: Bug fixes, documentation updates, clarifications

### Deprecation Process

When presets need to be removed or changed:
1. **Deprecation warning** in documentation and runtime
2. **Migration guide** with alternatives
3. **Grace period** for transition (typically 2 minor versions)
4. **Removal** with clear communication

## Best Practices

### For Generic Preset Contributors

1. **Stick to DDEX standards** - Only include fields required by DDEX spec
2. **Document thoroughly** - Explain the purpose of each validation rule
3. **Test extensively** - Validate with multiple real-world examples
4. **Be conservative** - Better to require too little than too much
5. **Consider compatibility** - Ensure presets work across DDEX versions where possible

### For Platform Preset Contributors

1. **Verify public availability** - Only use publicly documented requirements
2. **Link to sources** - Always provide provenance URLs
3. **Add clear disclaimers** - Explain community-maintained status
4. **Regular updates** - Monitor platform documentation for changes
5. **Community input** - Gather feedback from actual platform users

### For Custom Preset Creators

1. **Start with generic baseline** - Build on solid DDEX foundation
2. **Document your sources** - Track where requirements come from
3. **Test thoroughly** - Validate with your actual platform submissions
4. **Version appropriately** - Track changes to requirements
5. **Share improvements** - Contribute generic enhancements back

### For Organizations

1. **Internal documentation** - Maintain clear preset documentation
2. **Testing strategy** - Validate presets with real submissions
3. **Change management** - Process for updating internal presets
4. **Training** - Ensure team understands preset system
5. **Backup strategy** - Keep copies of working preset configurations

## Security Considerations

### Confidential Information
- **Never include** confidential platform requirements in public presets
- **Respect NDAs** and partnership agreements
- **Internal presets only** for proprietary requirements

### Validation Security
- **Input sanitization** - Validate all preset inputs
- **Pattern safety** - Ensure regex patterns can't cause DoS
- **Resource limits** - Prevent excessive validation computation

### Supply Chain Security
- **Source verification** - Verify preset sources and contributors
- **Code review** - All preset changes should be reviewed
- **Dependency tracking** - Monitor third-party preset dependencies

## Future Roadmap

### Short Term (v0.3.x)
- Enhanced custom preset tooling
- Preset validation and linting tools
- More comprehensive generic preset coverage
- Improved documentation and examples

### Medium Term (v0.4.x - v1.0.0)
- Preset marketplace and discovery
- Automated testing against platform submissions
- Visual preset builder
- Integration with CI/CD pipelines

### Long Term (v1.x+)
- Machine learning-assisted preset optimization
- Dynamic preset updates based on platform changes
- Preset analytics and usage tracking
- Enterprise preset management features

## Contributing

### How to Contribute

1. **Generic improvements** - Always welcome via pull requests
2. **Platform presets** - Only for platforms with public documentation
3. **Documentation** - Help improve guides and examples
4. **Testing** - Contribute test cases and validation scenarios
5. **Tooling** - Build better preset development tools

### Contribution Guidelines

1. **Follow templates** - Use provided preset templates
2. **Document sources** - Always include provenance information
3. **Add tests** - Include comprehensive test coverage
4. **Update docs** - Keep documentation current
5. **Community review** - Participate in preset reviews

### Community Resources

- **GitHub Issues**: Report bugs and request features
- **Discussions**: Ask questions and share experiences
- **Discord**: Real-time community chat
- **Monthly calls**: Community preset review sessions

---

This architecture ensures the DDEX Builder preset system remains honest, maintainable, and valuable to the community while respecting platform intellectual property and legal constraints.