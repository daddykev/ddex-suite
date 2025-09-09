//! DDEX Builder - Deterministic DDEX XML generation with DB-C14N/1.0
//! 
//! This crate provides deterministic, byte-perfect DDEX XML generation
//! with full round-trip fidelity when used with ddex-parser.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod builder;
pub mod canonical;
pub mod determinism;
pub mod error;
pub mod generator;
pub mod linker;
pub mod preflight;
pub mod presets;

// Re-export main types
pub use builder::{DDEXBuilder, BuildOptions, BuildRequest, BuildResult};
pub use canonical::DB_C14N;
pub use determinism::DeterminismConfig;
pub use error::{BuildError, BuildWarning};
pub use presets::PartnerPreset;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

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
    
    fn load_default_presets() -> IndexMap<String, PartnerPreset> {
        let mut presets = IndexMap::new();
        // Load built-in presets
        presets.insert("spotify_audio_43".to_string(), presets::spotify_audio_43());
        presets.insert("apple_music_43".to_string(), presets::apple_music_43());
        presets
    }
}