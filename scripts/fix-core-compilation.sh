#!/bin/bash

echo "Fixing core compilation errors..."

# 1. First, let's check what Error type exists in error.rs
echo "Checking error.rs structure..."

# Look for the Error enum/struct
if grep -q "pub enum Error" packages/core/src/error.rs; then
    echo "Found Error enum"
    ERROR_TYPE="enum"
elif grep -q "pub struct Error" packages/core/src/error.rs; then
    echo "Found Error struct"
    ERROR_TYPE="struct"
else
    echo "Error type not found, looking for ParseError..."
    if grep -q "pub enum ParseError" packages/core/src/error.rs; then
        echo "Found ParseError, will use that"
        ERROR_TYPE="ParseError"
    fi
fi

# 2. Fix the ERNVersion enum variants
echo "Fixing ERNVersion enum variants..."
cat > packages/core/src/models/versions/version.rs << 'RUST'
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ERNVersion {
    V3_8_2,  // Changed from ERN382
    V4_2,    // Changed from ERN42
    V4_3,    // Changed from ERN43
}

impl ERNVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            ERNVersion::V3_8_2 => "3.8.2",
            ERNVersion::V4_2 => "4.2",
            ERNVersion::V4_3 => "4.3",
        }
    }
    
    pub fn namespace(&self) -> &'static str {
        match self {
            ERNVersion::V3_8_2 => "http://ddex.net/xml/ern/382",
            ERNVersion::V4_2 => "http://ddex.net/xml/ern/42",
            ERNVersion::V4_3 => "http://ddex.net/xml/ern/43",
        }
    }
}

impl std::fmt::Display for ERNVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERN {}", self.as_str())
    }
}
RUST

# 3. Now let's properly fix the error.rs exports
echo "Fixing error.rs exports..."

# Check if ParseError exists and is the main error type
if grep -q "pub enum ParseError" packages/core/src/error.rs; then
    # Update lib.rs to use ParseError
    cat > packages/core/src/lib.rs << 'RUST'
pub mod models;
pub mod error;

// Re-export commonly used types
pub use error::ParseError as Error;  // Alias ParseError as Error for compatibility
pub use error::{ParseError, Result};
pub use models::{graph, flat, common, versions};
RUST

    # Make sure Result type uses ParseError
    if ! grep -q "pub type Result<T> = std::result::Result<T, ParseError>" packages/core/src/error.rs; then
        # Add or update the Result type
        sed -i '' '/pub type Result/d' packages/core/src/error.rs
        sed -i '' '/^use thiserror/a\
\
pub type Result<T> = std::result::Result<T, ParseError>;\
' packages/core/src/error.rs
    fi
else
    # If there's a different Error type, we need to handle it differently
    echo "Looking for other error types..."
fi

echo "Fixes applied!"
