use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    pub line: usize,
    pub column: usize,
    pub byte_offset: Option<usize>,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("XML parsing error: {message} at {location:?}")]
    XmlError {
        message: String,
        location: ErrorLocation,
    },
    
    #[error("Invalid DDEX version: {version}")]
    InvalidVersion { version: String },
    
    #[error("Unresolved reference: {reference} at {location:?}")]
    UnresolvedReference {
        reference: String,
        location: ErrorLocation,
    },
    
    #[error("Security violation: exceeded {limit}")]
    SecurityViolation { limit: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Timeout after {seconds} seconds")]
    Timeout { seconds: u64 },
    
    #[error("Structure error: {0}")]
    StructureError(String),
    
    #[error("Version error: {0}")]
    VersionError(String),
    
    #[error("Reference error: {0}")]
    ReferenceError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, ParseError>;

// Re-export for compatibility
pub use ParseError as Error;

impl From<ddex_core::error::ParseError> for ParseError {
    fn from(err: ddex_core::error::ParseError) -> Self {
        ParseError::Unknown(err.to_string())
    }
}

pub mod ffi;
