//! Type definitions for the linker module

use std::fmt;
use thiserror::Error;

/// Entity types in DDEX
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EntityType {
    Release,
    Resource,
    Party,
    Deal,
    TechnicalDetails,
    RightsController,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Release => write!(f, "Release"),
            Self::Resource => write!(f, "Resource"),
            Self::Party => write!(f, "Party"),
            Self::Deal => write!(f, "Deal"),
            Self::TechnicalDetails => write!(f, "TechnicalDetails"),
            Self::RightsController => write!(f, "RightsController"),
        }
    }
}

/// Reference generation style
#[derive(Debug, Clone)]
pub enum ReferenceStyle {
    /// Sequential numbering (A1, A2, R1, R2)
    Sequential,
    
    /// Prefixed with custom separator
    Prefixed { separator: String },
    
    /// Custom formatter function
    Custom(fn(EntityType, u32) -> String),
}

impl Default for ReferenceStyle {
    fn default() -> Self {
        Self::Sequential
    }
}

/// Configuration for the reference linker
#[derive(Debug, Clone)]
pub struct LinkerConfig {
    /// Reference generation style
    pub reference_style: ReferenceStyle,
    
    /// Enable auto-linking
    pub auto_link: bool,
    
    /// Validate references on build
    pub validate_on_build: bool,
    
    /// Strict mode (fail on warnings)
    pub strict: bool,
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            reference_style: ReferenceStyle::default(),
            auto_link: true,
            validate_on_build: true,
            strict: false,
        }
    }
}

/// Release-Resource reference mapping
#[derive(Debug, Clone)]
pub struct ReleaseResourceReference {
    pub release_reference: String,
    pub resource_reference: String,
    pub sequence_number: u32,
}

/// Report from auto-linking process
#[derive(Debug, Clone, Default)]
pub struct LinkingReport {
    pub generated_refs: usize,
    pub linked_resources: usize,
    pub linked_deals: usize,
    pub linked_parties: usize,
    pub validation_passed: bool,
    pub warnings: Vec<String>,
}

/// Linker errors
#[derive(Debug, Error)]
pub enum LinkerError {
    #[error("Unknown resource: {0}")]
    UnknownResource(String),
    
    #[error("Unknown release: {0}")]
    UnknownRelease(String),
    
    #[error("Orphaned reference: {0}")]
    OrphanedReference(String),
    
    #[error("Broken reference from {from} to {to}")]
    BrokenReference { from: String, to: String },
    
    #[error("Duplicate reference: {0}")]
    DuplicateReference(String),
    
    #[error("Circular reference detected: {0}")]
    CircularReference(String),
    
    #[error("Invalid entity type: {0}")]
    InvalidEntityType(String),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}