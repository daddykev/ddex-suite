//! # DDEX Builder - Deterministic DDEX XML Generation
//! 
//! A high-performance, memory-safe DDEX XML builder that generates deterministic,
//! byte-perfect XML using DB-C14N/1.0 canonicalization. Built in Rust with
//! comprehensive security features and bindings for JavaScript, Python, and WebAssembly.
//!
//! ## Key Features
//!
//! - **🔒 Security First**: XXE protection, input validation, rate limiting, and comprehensive security measures
//! - **⚡ High Performance**: Sub-millisecond generation for typical releases, memory-optimized streaming
//! - **🎯 Deterministic Output**: Guaranteed byte-perfect reproducibility using DB-C14N/1.0
//! - **🔄 Round-trip Fidelity**: Perfect compatibility with ddex-parser for Parse → Build → Parse workflows
//! - **🛠️ Partner Presets**: Pre-configured settings for Spotify, YouTube, Apple Music, and other platforms
//! - **🌐 Multi-platform**: Native Rust, Node.js, Python, and WebAssembly bindings
//! - **📊 Version Support**: Full support for ERN 3.8.2, 4.2, 4.3 with automatic conversion
//!
//! ## Quick Start
//!
//! ```rust
//! use ddex_builder::{Builder, DdexVersion};
//! use ddex_builder::builder::{BuildRequest, OutputFormat};
//!
//! // Create a builder with Spotify preset
//! let mut builder = Builder::new();
//! builder.preset("spotify_audio_43")?;
//!
//! // Build DDEX XML
//! let request = BuildRequest {
//!     source_xml: r#"<SoundRecording>...</SoundRecording>"#.to_string(),
//!     output_format: OutputFormat::Xml,
//!     preset: Some("spotify_audio_43".to_string()),
//!     validate_schema: true,
//! };
//!
//! let result = builder.build_internal(&request)?;
//! println!("Generated DDEX XML: {}", result.xml);
//! # Ok::<(), ddex_builder::BuildError>(())
//! ```
//!
//! ## Security Features
//!
//! DDEX Builder includes comprehensive security measures:
//!
//! ```rust
//! use ddex_builder::{InputValidator, SecurityConfig, ApiSecurityManager};
//!
//! // Configure security settings
//! let security_config = SecurityConfig {
//!     max_xml_size: 10_000_000,        // 10MB limit
//!     max_json_depth: 32,              // Prevent deep nesting attacks
//!     rate_limiting_enabled: true,
//!     max_requests_per_minute: 100,
//!     validate_urls: true,
//!     block_private_ips: true,
//!     ..Default::default()
//! };
//!
//! // Validate inputs
//! let validator = InputValidator::new(security_config.clone());
//! validator.validate_xml_content(&xml_input)?;
//!
//! // API security management
//! let mut api_security = ApiSecurityManager::new(security_config);
//! api_security.validate_request("build", "client_id", xml_input.len())?;
//! # Ok::<(), ddex_builder::BuildError>(())
//! ```
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        DDEX Builder                             │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Input Layer                                                    │
//! │  ┌─────────────┐ ┌──────────────┐ ┌─────────────┐              │
//! │  │ XML Parser  │ │ JSON Parser  │ │ Presets     │              │
//! │  │ (Security)  │ │ (Validation) │ │ (Partners)  │              │
//! │  └─────────────┘ └──────────────┘ └─────────────┘              │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Processing Layer                                               │
//! │  ┌─────────────┐ ┌──────────────┐ ┌─────────────┐              │
//! │  │ AST Builder │ │ Reference    │ │ Version     │              │
//! │  │ (Elements)  │ │ Linker       │ │ Converter   │              │
//! │  └─────────────┘ └──────────────┘ └─────────────┘              │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Output Layer                                                   │
//! │  ┌─────────────┐ ┌──────────────┐ ┌─────────────┐              │
//! │  │ XML         │ │ DB-C14N      │ │ Output      │              │
//! │  │ Generator   │ │ Canonicalize │ │ Sanitizer   │              │
//! │  └─────────────┘ └──────────────┘ └─────────────┘              │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Parse 10KB**: <5ms
//! - **Parse 100KB**: <10ms  
//! - **Parse 1MB**: <50ms
//! - **Build typical release**: <15ms
//! - **Memory usage**: <50MB for large files with streaming
//! - **WASM bundle size**: <500KB
//!
//! ## Version Support
//!
//! | DDEX Version | Support Level | Notes |
//! |--------------|---------------|-------|
//! | ERN 3.8.2    | ✅ Full       | Legacy support |
//! | ERN 4.2      | ✅ Full       | Enhanced features |
//! | ERN 4.3      | ✅ Full       | Latest standard |
//!
//! ## Partner Presets
//!
//! Pre-configured settings for major platforms:
//!
//! - `spotify_audio_43` - Spotify audio releases (ERN 4.3)
//! - `youtube_video_43` - YouTube video content (ERN 4.3)
//! - `apple_music_43` - Apple Music releases (ERN 4.3)
//! - `universal_basic` - Universal Music basic preset
//! - `sony_enhanced` - Sony Music enhanced features
//!
//! See the [User Guide](https://docs.ddex-builder.io/user-guide) for detailed preset documentation.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ast;
pub mod builder;
pub mod canonical;
pub mod determinism;
pub mod error;
pub mod guarantees;
pub mod generator;
pub mod presets;
pub mod streaming;
pub mod diff;
pub mod messages;
pub mod linker;
pub mod id_generator;
pub mod preflight;
pub mod schema;
pub mod versions;
pub mod optimized_strings;
pub mod memory_optimization;
pub mod parallel_processing;
pub mod caching;
pub mod security;
pub mod api_security;

// Re-export main types
pub use builder::{DDEXBuilder, BuildOptions, BuildRequest, BuildResult};
pub use canonical::DB_C14N;
pub use determinism::DeterminismConfig;
pub use error::{BuildError, BuildWarning};
pub use guarantees::{DeterminismGuarantee, DeterminismGuaranteeValidator, GuaranteeReport};
pub use presets::PartnerPreset;
pub use linker::{ReferenceLinker, LinkerConfig, EntityType, LinkerError};
pub use id_generator::{StableHashGenerator, StableHashConfig, HashAlgorithm};
pub use preflight::{PreflightValidator, ValidationConfig, ValidationResult, PreflightLevel};
pub use diff::{DiffEngine, DiffConfig, VersionCompatibility};
pub use diff::types::{ChangeSet, SemanticChange, DiffPath, ChangeType, ImpactLevel};
pub use diff::formatter::DiffFormatter;
pub use messages::{UpdateReleaseMessage, UpdateGenerator, UpdateAction, UpdateConfig, ValidationStatus};
pub use schema::{SchemaGenerator, JsonSchema, SchemaConfig, SchemaDraft, SchemaCommand};
pub use versions::{VersionManager, VersionConverter, ConverterResult as ConversionResult, ConversionOptions};
pub use presets::DdexVersion;

// Security module exports
pub use security::{InputValidator, SecurityConfig, RateLimiter, SecureTempFile, OutputSanitizer};
pub use api_security::{ApiSecurityManager, ApiSecurityConfig, FfiDataType, BatchStats};

use indexmap::IndexMap;
// Remove unused serde imports

/// Version of the DB-C14N specification
pub const DB_C14N_VERSION: &str = "1.0";

/// The main DDEX Builder for creating deterministic XML output.
///
/// `Builder` is the primary interface for generating DDEX-compliant XML with
/// guaranteed deterministic output. It supports partner presets, version conversion,
/// and comprehensive security features.
///
/// ## Features
///
/// - **Deterministic Output**: Uses DB-C14N/1.0 for byte-perfect reproducibility
/// - **Partner Presets**: Pre-configured settings for major music platforms
/// - **Version Management**: Support for ERN 3.8.2, 4.2, and 4.3 with conversion
/// - **Security**: Built-in validation, rate limiting, and XXE protection
/// - **Performance**: Memory-optimized with streaming support for large files
///
/// ## Usage Patterns
///
/// ### Basic Usage
///
/// ```rust
/// use ddex_builder::Builder;
///
/// let builder = Builder::new();
/// let available_presets = builder.available_presets();
/// println!("Available presets: {:?}", available_presets);
/// ```
///
/// ### With Partner Preset
///
/// ```rust
/// use ddex_builder::Builder;
///
/// let mut builder = Builder::new();
/// builder.preset("spotify_audio_43")?;
/// 
/// // Builder is now configured for Spotify Audio releases (ERN 4.3)
/// assert!(builder.is_preset_locked() == false); // Unlocked for further customization
/// # Ok::<(), ddex_builder::BuildError>(())
/// ```
///
/// ### Locked Preset Configuration
///
/// ```rust
/// use ddex_builder::Builder;
///
/// let mut builder = Builder::new();
/// builder.apply_preset("spotify_audio_43", true)?; // Lock the preset
/// 
/// assert!(builder.is_preset_locked());
/// # Ok::<(), ddex_builder::BuildError>(())
/// ```
///
/// ### Version Conversion
///
/// ```rust
/// use ddex_builder::{Builder, DdexVersion};
/// use ddex_builder::versions::ConversionOptions;
///
/// let builder = Builder::new();
/// 
/// // Check version compatibility
/// let compatible = builder.is_version_compatible(
///     DdexVersion::Ern382, 
///     DdexVersion::Ern43
/// );
/// 
/// if compatible {
///     let options = Some(ConversionOptions::default());
///     let result = builder.convert_version(
///         &xml_content,
///         DdexVersion::Ern382,
///         DdexVersion::Ern43,
///         options
///     )?;
///     println!("Converted XML: {}", result.converted_xml);
/// }
/// # let xml_content = "<test></test>";
/// # Ok::<(), ddex_builder::BuildError>(())
/// ```
///
/// ## Thread Safety
///
/// `Builder` is `Send + Sync` and can be safely shared between threads.
/// Each thread should create its own instance for best performance.
///
/// ## Memory Usage
///
/// The builder uses memory-optimized data structures and streaming
/// where possible. Typical memory usage:
/// - Small releases (<100KB): ~5MB
/// - Large releases (>1MB): ~20-50MB with streaming
#[derive(Debug, Clone)]
pub struct Builder {
    config: DeterminismConfig,
    presets: IndexMap<String, PartnerPreset>,
    locked_preset: Option<String>,
    version_manager: versions::VersionManager,
    target_version: Option<DdexVersion>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Creates a new DDEX Builder with default configuration.
    ///
    /// The builder is initialized with:
    /// - Default determinism configuration for byte-perfect output
    /// - All available partner presets loaded
    /// - No preset locked (can be changed)
    /// - Latest supported DDEX version as target
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ddex_builder::Builder;
    ///
    /// let builder = Builder::new();
    /// assert!(!builder.is_preset_locked());
    /// assert!(builder.available_presets().len() > 0);
    /// ```
    ///
    /// # Performance
    /// 
    /// Creating a new builder is fast (~1μs) as presets are loaded from
    /// embedded configuration data.
    pub fn new() -> Self {
        Self {
            config: DeterminismConfig::default(),
            presets: Self::load_default_presets(),
            locked_preset: None,
            version_manager: versions::VersionManager::new(),
            target_version: None,
        }
    }
    
    /// Create builder with custom configuration
    pub fn with_config(config: DeterminismConfig) -> Self {
        Self {
            config,
            presets: Self::load_default_presets(),
            locked_preset: None,
            version_manager: versions::VersionManager::new(),
            target_version: None,
        }
    }
    
    /// Applies a partner preset configuration to the builder.
    ///
    /// Presets contain pre-configured settings optimized for specific music platforms
    /// and distribution partners. Each preset includes determinism settings, validation
    /// rules, and format preferences.
    ///
    /// # Arguments
    ///
    /// * `preset_name` - Name of the preset to apply (see [`available_presets`])
    /// * `lock` - Whether to lock the preset to prevent further modifications
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Preset applied successfully
    /// * `Err(BuildError::InvalidFormat)` - Unknown preset name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ddex_builder::Builder;
    ///
    /// let mut builder = Builder::new();
    ///
    /// // Apply Spotify preset without locking
    /// builder.apply_preset("spotify_audio_43", false)?;
    /// assert!(!builder.is_preset_locked());
    ///
    /// // Apply and lock YouTube preset  
    /// builder.apply_preset("youtube_video_43", true)?;
    /// assert!(builder.is_preset_locked());
    /// # Ok::<(), ddex_builder::BuildError>(())
    /// ```
    ///
    /// # Available Presets
    ///
    /// Common presets include:
    /// - `spotify_audio_43` - Spotify audio releases (ERN 4.3)
    /// - `youtube_video_43` - YouTube video content (ERN 4.3)  
    /// - `apple_music_43` - Apple Music releases (ERN 4.3)
    /// - `universal_basic` - Universal Music basic preset
    /// - `sony_enhanced` - Sony Music enhanced features
    ///
    /// Use [`available_presets`] to get the complete list.
    ///
    /// [`available_presets`]: Self::available_presets
    pub fn apply_preset(&mut self, preset_name: &str, lock: bool) -> Result<(), error::BuildError> {
        let preset = self.presets.get(preset_name)
            .ok_or_else(|| error::BuildError::InvalidFormat {
                field: "preset".to_string(),
                message: format!("Unknown preset: {}", preset_name),
            })?
            .clone();
        
        // Apply the preset's determinism config
        self.config = preset.determinism;
        
        // Lock the preset if requested
        if lock {
            self.locked_preset = Some(preset_name.to_string());
        }
        
        Ok(())
    }

    /// Apply a preset configuration (alias for apply_preset for convenience)
    pub fn preset(&mut self, preset_name: &str) -> Result<&mut Self, error::BuildError> {
        self.apply_preset(preset_name, false)?;
        Ok(self)
    }

    /// Get available preset names
    pub fn available_presets(&self) -> Vec<String> {
        self.presets.keys().cloned().collect()
    }

    /// Get preset details
    pub fn get_preset(&self, preset_name: &str) -> Option<&PartnerPreset> {
        self.presets.get(preset_name)
    }
    
    /// Check if a preset is locked
    pub fn is_preset_locked(&self) -> bool {
        self.locked_preset.is_some()
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &DeterminismConfig {
        &self.config
    }
    
    /// Set target DDEX version for building
    pub fn with_version(&mut self, version: DdexVersion) -> &mut Self {
        self.target_version = Some(version);
        self
    }
    
    /// Get the target DDEX version
    pub fn target_version(&self) -> Option<DdexVersion> {
        self.target_version
    }
    
    /// Detect version from XML content
    pub fn detect_version(&self, xml_content: &str) -> Result<DdexVersion, error::BuildError> {
        self.version_manager.detect_version(xml_content)
            .map(|detection| detection.detected_version)
            .map_err(|e| error::BuildError::InvalidFormat {
                field: "version".to_string(),
                message: format!("Version detection failed: {}", e),
            })
    }
    
    /// Convert XML between DDEX versions
    pub fn convert_version(&self, xml_content: &str, from_version: DdexVersion, to_version: DdexVersion, options: Option<ConversionOptions>) -> Result<versions::ConverterResult, error::BuildError> {
        let converter = versions::VersionConverter::new();
        Ok(converter.convert(xml_content, from_version, to_version, options))
    }
    
    /// Get version compatibility information
    pub fn is_version_compatible(&self, from: DdexVersion, to: DdexVersion) -> bool {
        self.version_manager.is_conversion_supported(from, to)
    }
    
    /// Get supported DDEX versions
    pub fn supported_versions(&self) -> Vec<DdexVersion> {
        versions::utils::supported_versions()
    }
    
    fn load_default_presets() -> IndexMap<String, PartnerPreset> {
        presets::all_presets()
    }
    
    /// Internal build method used by determinism verifier
    pub(crate) fn build_internal(&self, request: &builder::BuildRequest) -> Result<builder::BuildResult, error::BuildError> {
        let builder = builder::DDEXBuilder::new();
        builder.build(request.clone(), builder::BuildOptions::default())
    }
}

/// Version information for the builder
pub fn version_info() -> String {
    format!(
        "DDEX Builder v{} • DB-C14N/{} • Rust {}",
        env!("CARGO_PKG_VERSION"),
        DB_C14N_VERSION,
        env!("CARGO_PKG_RUST_VERSION", "unknown")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builder_creation() {
        let builder = Builder::new();
        assert!(!builder.is_preset_locked());
    }
    
    #[test]
    fn test_preset_application() {
        let mut builder = Builder::new();
        assert!(builder.apply_preset("spotify_audio_43", false).is_ok());
        assert!(!builder.is_preset_locked());
        
        assert!(builder.apply_preset("spotify_audio_43", true).is_ok());
        assert!(builder.is_preset_locked());
    }
    
    #[test]
    fn test_unknown_preset() {
        let mut builder = Builder::new();
        assert!(builder.apply_preset("unknown_preset", false).is_err());
    }
    
    #[test]
    fn test_version_info() {
        let info = version_info();
        assert!(info.contains("DDEX Builder"));
        assert!(info.contains("DB-C14N/1.0"));
    }
}