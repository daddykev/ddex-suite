//! Parser-specific error handling

use ddex_core::error::DDEXError;
use ddex_core::ffi::{FFIError, FFIErrorSeverity, FFIErrorCategory};
use thiserror::Error;

// Re-export ErrorLocation for use in this crate
pub use ddex_core::error::ErrorLocation;

// Define Result type alias
pub type Result<T> = std::result::Result<T, ParseError>;

/// Parser-specific errors
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("XML parsing error: {message}")]
    XmlError {
        message: String,
        location: ErrorLocation,
    },
    
    #[error("Unsupported DDEX version: {version}")]
    UnsupportedVersion {
        version: String,
    },
    
    #[error("Security violation: {message}")]
    SecurityViolation {
        message: String,
    },
    
    #[error("Parse timeout after {seconds} seconds")]
    Timeout {
        seconds: u64,
    },
    
    #[error("Core error: {0}")]
    Core(#[from] DDEXError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<ParseError> for FFIError {
    fn from(err: ParseError) -> Self {
        match err {
            ParseError::Core(core_err) => core_err.into(),
            ParseError::XmlError { message, location } => FFIError {
                code: "PARSE_XML_ERROR".to_string(),
                message,
                location: Some(ddex_core::ffi::FFIErrorLocation {
                    line: location.line,
                    column: location.column,
                    path: location.path,
                }),
                severity: FFIErrorSeverity::Error,
                hint: Some("Check XML syntax".to_string()),
                category: FFIErrorCategory::XmlParsing,
            },
            ParseError::UnsupportedVersion { version } => FFIError {
                code: "UNSUPPORTED_VERSION".to_string(),
                message: format!("Unsupported DDEX version: {}", version),
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: Some("Use ERN 3.8.2, 4.2, or 4.3".to_string()),
                category: FFIErrorCategory::Version,
            },
            ParseError::SecurityViolation { message } => FFIError {
                code: "SECURITY_VIOLATION".to_string(),
                message,
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: Some("Check for XXE or entity expansion attacks".to_string()),
                category: FFIErrorCategory::Validation,
            },
            ParseError::Timeout { seconds } => FFIError {
                code: "PARSE_TIMEOUT".to_string(),
                message: format!("Parse timeout after {} seconds", seconds),
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: Some("File may be too large or complex".to_string()),
                category: FFIErrorCategory::Io,
            },
            ParseError::Io(io_err) => FFIError {
                code: "IO_ERROR".to_string(),
                message: io_err.to_string(),
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: None,
                category: FFIErrorCategory::Io,
            },
        }
    }
}
