//! UTF-8 handling utilities for safe text processing

use crate::error::ParseError;
use quick_xml::events::BytesText;

/// Process text content from raw bytes, ensuring valid UTF-8
#[allow(dead_code)]
pub fn process_text_content(raw_bytes: &[u8]) -> Result<String, ParseError> {
    String::from_utf8(raw_bytes.to_vec()).map_err(|e| ParseError::InvalidUtf8 {
        message: format!("UTF-8 decoding error at position 0: {}", e),
    })
}

/// Process text content with lossy UTF-8 conversion (replaces invalid sequences)
#[allow(dead_code)]
pub fn process_text_content_lossy(raw_bytes: &[u8]) -> String {
    String::from_utf8_lossy(raw_bytes).into_owned()
}

/// Decode UTF-8 at a specific position with error reporting
pub fn decode_utf8_at_position(bytes: &[u8], position: usize) -> Result<String, ParseError> {
    std::str::from_utf8(bytes)
        .map(|s| s.to_string())
        .map_err(|e| ParseError::InvalidUtf8 {
            message: format!("UTF-8 decoding error at position {}: {}", position, e),
        })
}

/// Handle text node from XML event
#[allow(dead_code)]
pub fn handle_text_node(event: &BytesText, position: usize) -> Result<String, ParseError> {
    let unescaped = event.unescape().map_err(|e| {
        ParseError::SimpleXmlError(format!("Unescape error at {}: {}", position, e))
    })?;

    process_text_content(unescaped.as_bytes())
}

/// Decode attribute name ensuring valid UTF-8
#[allow(dead_code)]
pub fn decode_attribute_name(bytes: &[u8], position: usize) -> Result<String, ParseError> {
    decode_utf8_at_position(bytes, position)
}

/// Decode attribute value with unescaping
#[allow(dead_code)]
pub fn decode_attribute_value(bytes: &[u8], position: usize) -> Result<String, ParseError> {
    // First decode UTF-8
    let utf8_str = std::str::from_utf8(bytes).map_err(|e| ParseError::InvalidUtf8 {
        message: format!("UTF-8 decoding error at position {}: {}", position, e),
    })?;

    // Then unescape XML entities
    quick_xml::escape::unescape(utf8_str)
        .map(|cow| cow.into_owned())
        .map_err(|e| ParseError::SimpleXmlError(format!("Attribute unescape error: {}", e)))
}

/// Validate UTF-8 string without copying
pub fn validate_utf8(bytes: &[u8]) -> Result<&str, ParseError> {
    std::str::from_utf8(bytes).map_err(|e| ParseError::InvalidUtf8 {
        message: format!("UTF-8 validation error: {}", e),
    })
}

/// Validate that a string contains only valid UTF-8 characters
pub fn validate_utf8_string(text: &str) -> Result<(), ParseError> {
    // Check if the string is valid UTF-8 (this should always pass for &str)
    // But we also check for any invalid Unicode scalar values
    for (pos, ch) in text.char_indices() {
        if ch == '\u{FFFD}' {
            // Replacement character indicates invalid UTF-8 was present
            return Err(ParseError::InvalidUtf8 {
                message: format!(
                    "Found Unicode replacement character at position {} indicating invalid UTF-8",
                    pos
                ),
            });
        }

        // Check for other problematic characters that might indicate encoding issues
        if ch.is_control() && ch != '\t' && ch != '\n' && ch != '\r' {
            // Allow common whitespace control characters but reject others
            return Err(ParseError::InvalidUtf8 {
                message: format!(
                    "Found invalid control character at position {}: U+{:04X}",
                    pos, ch as u32
                ),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_utf8() {
        let text = "Hello, 世界! 🎵".as_bytes();
        assert_eq!(process_text_content(text).unwrap(), "Hello, 世界! 🎵");
    }

    #[test]
    fn test_invalid_utf8() {
        let invalid = vec![0xFF, 0xFE, 0xFD];
        assert!(process_text_content(&invalid).is_err());
    }

    #[test]
    fn test_lossy_conversion() {
        let mixed = vec![72, 101, 108, 108, 111, 0xFF, 0xFE];
        let result = process_text_content_lossy(&mixed);
        assert!(result.starts_with("Hello"));
    }
}
