#!/bin/bash

echo "Fixing FFI error pattern matching..."

# Add the missing match arms to the FFI error conversion
cat >> packages/ddex-parser/src/error/ffi.rs << 'RUST'

            // Add these cases to the existing match statement
            ParseError::StructureError(msg) => FFIError {
                code: "STRUCTURE_ERROR",
                message: msg.clone(),
                details: None,
                category: FFIErrorCategory::Parsing,
                severity: FFIErrorSeverity::Error,
                location: None,
            },
            ParseError::VersionError(msg) => FFIError {
                code: "VERSION_ERROR",
                message: msg.clone(),
                details: None,
                category: FFIErrorCategory::Parsing,
                severity: FFIErrorSeverity::Error,
                location: None,
            },
            ParseError::ReferenceError(msg) => FFIError {
                code: "REFERENCE_ERROR",
                message: msg.clone(),
                details: None,
                category: FFIErrorCategory::Parsing,
                severity: FFIErrorSeverity::Error,
                location: None,
            },
            ParseError::SerializationError(err) => FFIError {
                code: "SERIALIZATION_ERROR",
                message: err.to_string(),
                details: None,
                category: FFIErrorCategory::Parsing,
                severity: FFIErrorSeverity::Error,
                location: None,
            },
            ParseError::Unknown(msg) => FFIError {
                code: "UNKNOWN_ERROR",
                message: msg.clone(),
                details: None,
                category: FFIErrorCategory::Parsing,
                severity: FFIErrorSeverity::Error,
                location: None,
            },
        }
    }
}
RUST

echo "FFI errors fixed!"
