#!/bin/bash

echo "Adding missing match arms to ffi.rs..."

# Insert the missing cases at line 137 (before the closing brace of the match)
sed -i '' '137i\
            ParseError::StructureError(msg) => FFIError {\
                code: "STRUCTURE_ERROR".to_string(),\
                message: msg,\
                location: None,\
                severity: FFIErrorSeverity::Error,\
                hint: None,\
                category: FFIErrorCategory::Parsing,\
            },\
            ParseError::VersionError(msg) => FFIError {\
                code: "VERSION_ERROR".to_string(),\
                message: msg,\
                location: None,\
                severity: FFIErrorSeverity::Error,\
                hint: None,\
                category: FFIErrorCategory::Parsing,\
            },\
            ParseError::ReferenceError(msg) => FFIError {\
                code: "REFERENCE_ERROR".to_string(),\
                message: msg,\
                location: None,\
                severity: FFIErrorSeverity::Error,\
                hint: None,\
                category: FFIErrorCategory::Parsing,\
            },\
            ParseError::SerializationError(err) => FFIError {\
                code: "SERIALIZATION_ERROR".to_string(),\
                message: err.to_string(),\
                location: None,\
                severity: FFIErrorSeverity::Error,\
                hint: None,\
                category: FFIErrorCategory::Parsing,\
            },\
            ParseError::Unknown(msg) => FFIError {\
                code: "UNKNOWN_ERROR".to_string(),\
                message: msg,\
                location: None,\
                severity: FFIErrorSeverity::Error,\
                hint: None,\
                category: FFIErrorCategory::Parsing,\
            },
' packages/ddex-parser/src/error/ffi.rs

echo "Missing arms added!"
