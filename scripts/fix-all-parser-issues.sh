#!/bin/bash

echo "Fixing all parser compilation issues..."

# 1. Remove the models module declaration from parser (it's in core now)
echo "Removing models module from parser lib.rs..."
sed -i '' '/^pub mod models;/d' packages/ddex-parser/src/lib.rs

# 2. Fix ERNVersion imports in parser lib.rs
echo "Fixing ERNVersion imports in parser lib.rs..."
sed -i '' 's/parser::detector::ERNVersion/ddex_core::models::versions::ERNVersion/g' packages/ddex-parser/src/lib.rs
sed -i '' 's/pub use parser::detector::ERNVersion;/pub use ddex_core::models::versions::ERNVersion;/g' packages/ddex-parser/src/lib.rs

# 3. Remove the duplicate ERNVersion impl from detector.rs
echo "Removing duplicate ERNVersion implementation..."
sed -i '' '/^impl ERNVersion {/,/^}/d' packages/ddex-parser/src/parser/detector.rs
sed -i '' '/^#\[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)\]/,/^\/\/ impl ERNVersion/d' packages/ddex-parser/src/parser/detector.rs

# 4. Create the ErrorLocation and other missing types in parser's error module
echo "Creating complete error types in parser..."
cat > packages/ddex-parser/src/error.rs << 'RUST'
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
RUST

# 5. Fix the parser's use of core error vs local error
echo "Updating parser to use local error types..."
find packages/ddex-parser/src -name "*.rs" -type f -exec sed -i '' \
  -e 's/ddex_core::error::ParseError/crate::error::ParseError/g' \
  -e 's/ddex_core::error::ErrorLocation/crate::error::ErrorLocation/g' \
  -e 's/ddex_core::error::ErrorSeverity/crate::error::ErrorSeverity/g' \
  -e 's/use ddex_core::error::{ParseError/use crate::error::{ParseError/g' \
  {} \;

# 6. Add conversion from core errors to parser errors
echo "Adding error conversion..."
cat >> packages/ddex-parser/src/error.rs << 'RUST'

impl From<ddex_core::error::ParseError> for ParseError {
    fn from(err: ddex_core::error::ParseError) -> Self {
        ParseError::Unknown(err.to_string())
    }
}
RUST

# 7. Create extension methods for ERNVersion in parser
echo "Creating ERNVersion extension trait..."
cat > packages/ddex-parser/src/parser/version_ext.rs << 'RUST'
use ddex_core::models::versions::ERNVersion;

pub trait ERNVersionExt {
    fn namespace_uri(&self) -> &str;
}

impl ERNVersionExt for ERNVersion {
    fn namespace_uri(&self) -> &str {
        match self {
            ERNVersion::V3_8_2 => "http://ddex.net/xml/ern/382",
            ERNVersion::V4_2 => "http://ddex.net/xml/ern/42",
            ERNVersion::V4_3 => "http://ddex.net/xml/ern/43",
        }
    }
}
RUST

# 8. Update detector.rs to use the extension trait
echo "Updating detector.rs to use extension trait..."
sed -i '' '1i\
mod version_ext;\
use version_ext::ERNVersionExt;\
' packages/ddex-parser/src/parser/detector.rs

# 9. Update parser mod.rs to include version_ext
echo "Adding version_ext to parser mod.rs..."
if [ -f "packages/ddex-parser/src/parser/mod.rs" ]; then
    echo "pub mod version_ext;" >> packages/ddex-parser/src/parser/mod.rs
fi

# 10. Fix the FFI error conversion
echo "Fixing FFI error conversion..."
if [ -f "packages/ddex-parser/src/error/ffi.rs" ]; then
    sed -i '' 's/ParseError::XmlError { message, location }/ParseError::XmlError { message, location }/g' packages/ddex-parser/src/error/ffi.rs
fi

echo "All fixes applied!"
