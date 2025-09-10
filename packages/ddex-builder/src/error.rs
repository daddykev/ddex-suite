//! Error types for DDEX Builder

use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Build error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BuildError {
    /// Invalid format
    #[error("Invalid format in {field}: {message}")]
    InvalidFormat {
        field: String,
        message: String,
    },
    
    /// Missing required field
    #[error("Missing required field: {field}")]
    MissingRequired {
        field: String,
    },
    
    /// Invalid reference
    #[error("Invalid reference: {reference}")]
    InvalidReference {
        reference: String,
    },
    
    /// Validation failed
    #[error("Validation failed: {}", errors.join(", "))]
    ValidationFailed {
        errors: Vec<String>,
    },
    
    /// IO error
    #[error("IO error: {0}")]
    Io(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// XML generation error
    #[error("XML generation error: {0}")]
    XmlGeneration(String),
    
    /// Determinism verification failed
    #[error("Determinism verification failed: {message}")]
    DeterminismFailed {
        message: String,
    },
    
    /// Determinism guarantee violation
    #[error("Determinism guarantee violated: {guarantee} - {details}")]
    DeterminismGuaranteeViolated {
        guarantee: String,
        details: String,
    },
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Parallel processing error
    #[error("Parallel processing error: {0}")]
    Parallel(String),
    
    /// Security violation
    #[error("Security violation: {0}")]
    Security(String),
    
    /// Input sanitization failed
    #[error("Input sanitization failed: {0}")]
    InputSanitization(String),
    
    /// Other error
    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for BuildError {
    fn from(err: std::io::Error) -> Self {
        BuildError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for BuildError {
    fn from(err: serde_json::Error) -> Self {
        BuildError::Serialization(err.to_string())
    }
}

impl From<quick_xml::Error> for BuildError {
    fn from(err: quick_xml::Error) -> Self {
        BuildError::XmlGeneration(err.to_string())
    }
}

/// Build warning (non-fatal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildWarning {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}