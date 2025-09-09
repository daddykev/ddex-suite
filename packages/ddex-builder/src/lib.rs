//! DDEX Builder - Deterministic DDEX XML generation with DB-C14N/1.0
//! 
//! This crate provides deterministic, byte-perfect DDEX XML generation
//! with full round-trip fidelity when used with ddex-parser.

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

use indexmap::IndexMap;
// Remove unused serde imports

/// Version of the DB-C14N specification
pub const DB_C14N_VERSION: &str = "1.0";

/// Main builder struct
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
    /// Create a new builder with default configuration
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
    
    /// Apply a preset configuration
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