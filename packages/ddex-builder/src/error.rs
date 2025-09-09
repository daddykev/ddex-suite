//! Error types for the builder

use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Build error
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum BuildError {
    /// Missing required field
    #[error("Missing required field: {field}")]
    MissingRequired { field: String },
    
    /// Invalid format
    #[error("Invalid format for {field}: {message}")]
    InvalidFormat { field: String, message: String },
    
    /// Unknown field
    #[error("Unknown field: {field}")]
    UnknownField { field: String },
    
    /// Bad reference
    #[error("Invalid reference: {reference}")]
    BadReference { reference: String },
    
    /// Cycle detected
    #[error("Circular reference detected: {reference}")]
    CycleDetected { reference: String },
    
    /// Namespace violation
    #[error("Namespace lock violation: {message}")]
    NamespaceLockViolation { message: String },
    
    /// Determinism failure
    #[error("Determinism check failed: {message}")]
    DeterminismFailure { message: String },
    
    /// IO error (changed to store string instead of std::io::Error)
    #[error("IO error: {0}")]
    Io(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

// Implement From<std::io::Error> manually
impl From<std::io::Error> for BuildError {
    fn from(err: std::io::Error) -> Self {
        BuildError::Io(err.to_string())
    }
}

/// Build warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildWarning {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
    pub hint: Option<String>,
}