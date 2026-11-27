//! Error types for DDEX Builder

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Build error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BuildError {
    /// Invalid format
    #[error("Invalid format in {field}: {message}")]
    InvalidFormat {
        /// Field that failed validation
        field: String,
        /// Error message describing the issue
        message: String,
    },

    /// Missing required field
    #[error("Missing required field: {field}")]
    MissingRequired {
        /// Field that failed validation
        field: String,
    },

    /// Invalid reference
    #[error("Invalid reference: {reference}")]
    InvalidReference {
        /// Reference that could not be resolved
        reference: String,
    },

    /// Validation failed
    #[error("Validation failed: {}", errors.join(", "))]
    ValidationFailed {
        /// List of validation errors
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

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

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
    /// Error code for categorization
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional location information
    pub location: Option<String>,
}
