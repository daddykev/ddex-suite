// core/src/error/ffi.rs
//! FFI-friendly error types for cross-language bindings

use serde::{Deserialize, Serialize};
use crate::error::{ParseError, ErrorLocation, ErrorSeverity};

/// FFI-friendly error representation that can be safely passed across language boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct FFIError {
    /// Machine-readable error code for programmatic handling
    pub code: String,
    
    /// Human-readable error message
    pub message: String,
    
    /// Location information if available
    pub location: Option<FFIErrorLocation>,
    
    /// Error severity level
    pub severity: FFIErrorSeverity,
    
    /// Optional hint for fixing the error
    pub hint: Option<String>,
    
    /// Category of error for grouping
    pub category: FFIErrorCategory,
}

/// Simplified location information for FFI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct FFIErrorLocation {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub byte_offset: Option<usize>,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub enum FFIErrorSeverity {
    Error,
    Warning,
    Info,
}

/// Error categories for better error handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub enum FFIErrorCategory {
    XmlParsing,
    InvalidVersion,
    UnresolvedReference,
    SecurityViolation,
    IoError,
    Timeout,
}

impl From<ErrorSeverity> for FFIErrorSeverity {
    fn from(severity: ErrorSeverity) -> Self {
        match severity {
            ErrorSeverity::Error => FFIErrorSeverity::Error,
            ErrorSeverity::Warning => FFIErrorSeverity::Warning,
            ErrorSeverity::Info => FFIErrorSeverity::Info,
        }
    }
}

impl From<ErrorLocation> for FFIErrorLocation {
    fn from(loc: ErrorLocation) -> Self {
        FFIErrorLocation {
            path: loc.path,
            line: loc.line,
            column: loc.column,
            byte_offset: loc.byte_offset,
        }
    }
}

impl From<ParseError> for FFIError {
    fn from(error: ParseError) -> Self {
        match error {
            ParseError::XmlError { message, location } => FFIError {
                code: "XML_PARSE_ERROR".to_string(),
                message,
                location: Some(location.into()),
                severity: FFIErrorSeverity::Error,
                hint: Some("Check XML syntax and ensure it's well-formed".to_string()),
                category: FFIErrorCategory::XmlParsing,
            },
            ParseError::InvalidVersion { version } => FFIError {
                code: "INVALID_VERSION".to_string(),
                message: format!("Invalid DDEX version: {}", version),
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: Some("Supported versions are: 3.8.2, 4.2, 4.3".to_string()),
                category: FFIErrorCategory::InvalidVersion,
            },
            ParseError::UnresolvedReference { reference, location } => FFIError {
                code: "UNRESOLVED_REFERENCE".to_string(),
                message: format!("Cannot resolve reference: {}", reference),
                location: Some(location.into()),
                severity: FFIErrorSeverity::Warning,
                hint: Some(format!("Check that '{}' is defined in the document", reference)),
                category: FFIErrorCategory::UnresolvedReference,
            },
            ParseError::SecurityViolation { limit } => FFIError {
                code: "SECURITY_VIOLATION".to_string(),
                message: format!("Security limit exceeded: {}", limit),
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: Some("File may be malicious or too complex. Check security settings.".to_string()),
                category: FFIErrorCategory::SecurityViolation,
            },
            ParseError::Io(io_error) => FFIError {
                code: "IO_ERROR".to_string(),
                message: io_error.to_string(),
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: Some("Check file permissions and disk space".to_string()),
                category: FFIErrorCategory::IoError,
            },
            ParseError::Timeout { seconds } => FFIError {
                code: "PARSE_TIMEOUT".to_string(),
                message: format!("Parse timeout after {} seconds", seconds),
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: Some("File may be too large. Try streaming mode or increase timeout.".to_string()),
                category: FFIErrorCategory::Timeout,
            },
            ParseError::StructureError(msg) => FFIError {
                code: "STRUCTURE_ERROR".to_string(),
                message: msg,
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: None,
                category: FFIErrorCategory::XmlParsing,
            },
            ParseError::VersionError(msg) => FFIError {
                code: "VERSION_ERROR".to_string(),
                message: msg,
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: None,
                category: FFIErrorCategory::XmlParsing,
            },
            ParseError::ReferenceError(msg) => FFIError {
                code: "REFERENCE_ERROR".to_string(),
                message: msg,
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: None,
                category: FFIErrorCategory::XmlParsing,
            },
            ParseError::SerializationError(err) => FFIError {
                code: "SERIALIZATION_ERROR".to_string(),
                message: err.to_string(),
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: None,
                category: FFIErrorCategory::XmlParsing,
            },
            ParseError::Unknown(msg) => FFIError {
                code: "UNKNOWN_ERROR".to_string(),
                message: msg,
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: None,
                category: FFIErrorCategory::XmlParsing,
            },
        }
    }
}

/// Result type for FFI boundaries
pub type FFIResult<T> = Result<T, FFIError>;

/// Convert internal results to FFI results
pub trait IntoFFIResult<T> {
    fn into_ffi_result(self) -> FFIResult<T>;
}

impl<T> IntoFFIResult<T> for Result<T, ParseError> {
    fn into_ffi_result(self) -> FFIResult<T> {
        self.map_err(FFIError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_error_to_ffi_error() {
        let location = ErrorLocation {
            line: 42,
            column: 10,
            byte_offset: Some(1000),
            path: "/ReleaseList/Release[1]".to_string(),
        };
        
        let parse_error = ParseError::XmlError {
            message: "Unexpected end tag".to_string(),
            location: location.clone(),
        };
        
        let ffi_error: FFIError = parse_error.into();
        
        assert_eq!(ffi_error.code, "XML_PARSE_ERROR");
        assert_eq!(ffi_error.message, "Unexpected end tag");
        assert_eq!(ffi_error.category, FFIErrorCategory::XmlParsing);
        
        let ffi_location = ffi_error.location.unwrap();
        assert_eq!(ffi_location.line, 42);
        assert_eq!(ffi_location.column, 10);
        assert_eq!(ffi_location.path, "/ReleaseList/Release[1]");
    }
}