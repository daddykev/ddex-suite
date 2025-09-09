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
pub mod generator;
pub mod presets;

// Re-export main types
pub use builder::{DDEXBuilder, BuildOptions, BuildRequest, BuildResult};
pub use canonical::DB_C14N;
pub use determinism::DeterminismConfig;
pub use error::{BuildError, BuildWarning};
pub use presets::PartnerPreset;

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
        }
    }
    
    /// Create builder with custom configuration
    pub fn with_config(config: DeterminismConfig) -> Self {
        Self {
            config,
            presets: Self::load_default_presets(),
            locked_preset: None,
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
    
    /// Check if a preset is locked
    pub fn is_preset_locked(&self) -> bool {
        self.locked_preset.is_some()
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &DeterminismConfig {
        &self.config
    }
    
    fn load_default_presets() -> IndexMap<String, PartnerPreset> {
        let mut presets = IndexMap::new();
        // Load built-in presets
        presets.insert("spotify_audio_43".to_string(), presets::spotify_audio_43());
        presets.insert("apple_music_43".to_string(), presets::apple_music_43());
        presets
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