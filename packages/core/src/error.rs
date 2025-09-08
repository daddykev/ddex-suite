//! Shared error types for DDEX Suite

use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Common error types used across parser and builder
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum DDEXError {
    #[error("XML parsing error: {message}")]
    XmlError {
        message: String,
        location: ErrorLocation,
    },

    #[error("Validation error: {message}")]
    ValidationError {
        message: String,
        field: Option<String>,
    },

    #[error("Reference error: {message}")]
    ReferenceError {
        message: String,
        reference: String,
    },

    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch {
        expected: String,
        found: String,
    },

    #[error("IO error: {message}")]
    IoError {
        message: String,
    },
}

/// Location information for errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    pub line: usize,
    pub column: usize,
    pub byte_offset: Option<usize>,
    pub path: String,
}

/// FFI-friendly error representation
pub mod ffi {
    use super::*;
    use crate::ffi::{FFIError, FFIErrorSeverity, FFIErrorCategory, FFIErrorLocation};
    
    /// Convert from DDEXError to FFIError
    impl From<DDEXError> for FFIError {
        fn from(err: DDEXError) -> Self {
            match err {
                DDEXError::XmlError { message, location } => FFIError {
                    code: "XML_PARSE_ERROR".to_string(),
                    message,
                    location: Some(FFIErrorLocation {
                        line: location.line,
                        column: location.column,
                        path: location.path,
                    }),
                    severity: FFIErrorSeverity::Error,
                    hint: Some("Check XML syntax".to_string()),
                    category: FFIErrorCategory::XmlParsing,
                },
                DDEXError::ValidationError { message, field } => FFIError {
                    code: "VALIDATION_ERROR".to_string(),
                    message: message.clone(),
                    location: field.map(|f| FFIErrorLocation {
                        line: 0,
                        column: 0,
                        path: f,
                    }),
                    severity: FFIErrorSeverity::Error,
                    hint: Some("Check field requirements".to_string()),
                    category: FFIErrorCategory::Validation,
                },
                DDEXError::ReferenceError { message, reference } => FFIError {
                    code: "REFERENCE_ERROR".to_string(),
                    message,
                    location: Some(FFIErrorLocation {
                        line: 0,
                        column: 0,
                        path: reference,
                    }),
                    severity: FFIErrorSeverity::Error,
                    hint: Some("Verify reference exists".to_string()),
                    category: FFIErrorCategory::Reference,
                },
                DDEXError::VersionMismatch { expected, found } => FFIError {
                    code: "VERSION_MISMATCH".to_string(),
                    message: format!("Expected version {}, found {}", expected, found),
                    location: None,
                    severity: FFIErrorSeverity::Error,
                    hint: Some("Use correct DDEX version".to_string()),
                    category: FFIErrorCategory::Version,
                },
                DDEXError::IoError { message } => FFIError {
                    code: "IO_ERROR".to_string(),
                    message,
                    location: None,
                    severity: FFIErrorSeverity::Error,
                    hint: None,
                    category: FFIErrorCategory::Io,
                },
            }
        }
    }
}
