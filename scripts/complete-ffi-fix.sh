#!/bin/bash

echo "Applying complete FFI fix..."

# Find the line number where we need to insert (after SecurityViolation block)
LINE_NUM=$(grep -n "SecurityViolation { limit }" packages/ddex-parser/src/error/ffi.rs | cut -d: -f1)

# We need to add the missing cases after the SecurityViolation case
# Let's find where that block ends (looking for the closing brace and comma)
INSERTION_LINE=$((LINE_NUM + 5))

# Create the missing match arms
cat > /tmp/missing_arms.txt << 'RUST'
            },
            ParseError::Io(io_err) => FFIError {
                code: "IO_ERROR".to_string(),
                message: format!("IO error: {}", io_err),
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: Some("Check file permissions and disk space".to_string()),
                category: FFIErrorCategory::Parsing,
            },
            ParseError::Timeout { seconds } => FFIError {
                code: "TIMEOUT".to_string(),
                message: format!("Operation timed out after {} seconds", seconds),
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: Some("Consider increasing timeout or processing smaller files".to_string()),
                category: FFIErrorCategory::Parsing,
            },
            ParseError::StructureError(msg) => FFIError {
                code: "STRUCTURE_ERROR".to_string(),
                message: msg,
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: None,
                category: FFIErrorCategory::Parsing,
            },
            ParseError::VersionError(msg) => FFIError {
                code: "VERSION_ERROR".to_string(),
                message: msg,
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: None,
                category: FFIErrorCategory::Parsing,
            },
            ParseError::ReferenceError(msg) => FFIError {
                code: "REFERENCE_ERROR".to_string(),
                message: msg,
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: None,
                category: FFIErrorCategory::Parsing,
            },
            ParseError::SerializationError(err) => FFIError {
                code: "SERIALIZATION_ERROR".to_string(),
                message: err.to_string(),
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: None,
                category: FFIErrorCategory::Parsing,
            },
            ParseError::Unknown(msg) => FFIError {
                code: "UNKNOWN_ERROR".to_string(),
                message: msg,
                location: None,
                severity: FFIErrorSeverity::Error,
                hint: None,
                category: FFIErrorCategory::Parsing,
RUST

# Now we need to find the exact location and insert these
# Let's use a different approach - replace the entire match statement

# First backup
cp packages/ddex-parser/src/error/ffi.rs packages/ddex-parser/src/error/ffi.rs.bak

# Find where the match starts and ends, then replace it
awk '
/match error \{/ { 
    print
    in_match = 1
    brace_count = 1
    next
}
in_match {
    if (/\{/) brace_count++
    if (/\}/) {
        brace_count--
        if (brace_count == 0) {
            # Insert missing arms before closing
            print "            ParseError::Io(io_err) => FFIError {"
            print "                code: \"IO_ERROR\".to_string(),"
            print "                message: format!(\"IO error: {}\", io_err),"
            print "                location: None,"
            print "                severity: FFIErrorSeverity::Error,"
            print "                hint: Some(\"Check file permissions and disk space\".to_string()),"
            print "                category: FFIErrorCategory::Parsing,"
            print "            },"
            print "            ParseError::Timeout { seconds } => FFIError {"
            print "                code: \"TIMEOUT\".to_string(),"
            print "                message: format!(\"Operation timed out after {} seconds\", seconds),"
            print "                location: None,"
            print "                severity: FFIErrorSeverity::Error,"
            print "                hint: Some(\"Consider increasing timeout or processing smaller files\".to_string()),"
            print "                category: FFIErrorCategory::Parsing,"
            print "            },"
            print "            ParseError::StructureError(msg) => FFIError {"
            print "                code: \"STRUCTURE_ERROR\".to_string(),"
            print "                message: msg,"
            print "                location: None,"
            print "                severity: FFIErrorSeverity::Error,"
            print "                hint: None,"
            print "                category: FFIErrorCategory::Parsing,"
            print "            },"
            print "            ParseError::VersionError(msg) => FFIError {"
            print "                code: \"VERSION_ERROR\".to_string(),"
            print "                message: msg,"
            print "                location: None,"
            print "                severity: FFIErrorSeverity::Error,"
            print "                hint: None,"
            print "                category: FFIErrorCategory::Parsing,"
            print "            },"
            print "            ParseError::ReferenceError(msg) => FFIError {"
            print "                code: \"REFERENCE_ERROR\".to_string(),"
            print "                message: msg,"
            print "                location: None,"
            print "                severity: FFIErrorSeverity::Error,"
            print "                hint: None,"
            print "                category: FFIErrorCategory::Parsing,"
            print "            },"
            print "            ParseError::SerializationError(err) => FFIError {"
            print "                code: \"SERIALIZATION_ERROR\".to_string(),"
            print "                message: err.to_string(),"
            print "                location: None,"
            print "                severity: FFIErrorSeverity::Error,"
            print "                hint: None,"
            print "                category: FFIErrorCategory::Parsing,"
            print "            },"
            print "            ParseError::Unknown(msg) => FFIError {"
            print "                code: \"UNKNOWN_ERROR\".to_string(),"
            print "                message: msg,"
            print "                location: None,"
            print "                severity: FFIErrorSeverity::Error,"
            print "                hint: None,"
            print "                category: FFIErrorCategory::Parsing,"
            print "            },"
            in_match = 0
        }
    }
    print
}
!in_match { print }
' packages/ddex-parser/src/error/ffi.rs.bak > packages/ddex-parser/src/error/ffi.rs

echo "FFI fix complete!"
