//! FFI type definitions for cross-language bindings

use serde::{Serialize, Deserialize};

/// Location information for FFI errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFIErrorLocation {
    pub line: usize,
    pub column: usize,
    pub path: String,
}

/// FFI-safe error type for cross-language bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFIError {
    pub code: String,
    pub message: String,
    pub location: Option<FFIErrorLocation>,
    pub severity: FFIErrorSeverity,
    pub hint: Option<String>,
    pub category: FFIErrorCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FFIErrorSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FFIErrorCategory {
    XmlParsing,
    Validation,
    Reference,
    Version,
    Io,
    Internal,
}

/// Result type for FFI boundaries
#[derive(Debug, Serialize, Deserialize)]
pub struct FFIResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<FFIError>,
}

impl<T> FFIResult<T> {
    pub fn ok(data: T) -> Self {
        FFIResult {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: FFIError) -> Self {
        FFIResult {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// Parse options for FFI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFIParseOptions {
    pub include_raw_extensions: bool,
    pub include_comments: bool,
    pub strict_mode: bool,
    pub max_depth: Option<usize>,
    pub timeout_seconds: Option<u64>,
}

impl Default for FFIParseOptions {
    fn default() -> Self {
        FFIParseOptions {
            include_raw_extensions: false,
            include_comments: false,
            strict_mode: false,
            max_depth: Some(100),
            timeout_seconds: Some(30),
        }
    }
}
