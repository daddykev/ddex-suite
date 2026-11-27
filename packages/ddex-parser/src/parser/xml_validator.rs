//! XML structure validation for detecting malformed XML

use crate::error::ParseError;
use crate::utf8_utils;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use std::io::BufRead;

/// XML validator that tracks element stack and validates structure
#[derive(Debug, Clone)]
pub struct XmlValidator {
    /// Stack of open XML elements for tag matching (stores element name and its depth)
    element_stack: Vec<(String, usize)>,
    /// Track current actual nesting depth (siblings don't increase depth)
    current_depth: usize,
    /// Track current byte position for error reporting
    current_position: usize,
    /// Enable strict validation (mismatched tags, unclosed elements)
    strict_validation: bool,
    /// Enable extended validation (attributes, content)
    extended_validation: bool,
}

impl Default for XmlValidator {
    fn default() -> Self {
        Self::new(true, false)
    }
}

impl XmlValidator {
    /// Create new validator with specified validation levels
    pub fn new(strict: bool, extended: bool) -> Self {
        Self {
            element_stack: Vec::new(),
            current_depth: 0,
            current_position: 0,
            strict_validation: strict,
            extended_validation: extended,
        }
    }

    /// Create a strict validator for production use
    pub fn strict() -> Self {
        Self::new(true, true)
    }

    /// Create a lenient validator for development/testing
    pub fn lenient() -> Self {
        Self::new(false, false)
    }

    /// Validate XML structure during parsing
    pub(crate) fn validate_event<R: BufRead>(
        &mut self,
        event: &Event,
        reader: &Reader<R>,
    ) -> Result<(), ParseError> {
        // Update current position for error reporting
        self.current_position = reader.buffer_position() as usize;

        match event {
            Event::Start(ref element) => {
                self.handle_start_element(element)?;
            }
            Event::End(ref element) => {
                self.handle_end_element(element)?;
            }
            Event::Empty(ref element) => {
                self.handle_empty_element(element)?;
            }
            Event::Text(ref text) => {
                if self.extended_validation {
                    self.validate_text_content(text)?;
                }
            }
            Event::GeneralRef(ref reference) => {
                // A reference in character data (its own event since quick-xml 0.38): the
                // same check as when it was part of the text.
                if self.extended_validation {
                    let raw = format!("&{};", String::from_utf8_lossy(reference));
                    utf8_utils::handle_escaped_text(raw.as_bytes(), self.current_position)?;
                }
            }
            Event::CData(ref cdata) => {
                if self.extended_validation {
                    self.validate_cdata_content(cdata)?;
                }
            }
            Event::Comment(_) => {
                // Comments are always valid
            }
            Event::Decl(_) => {
                // XML declarations are handled elsewhere
            }
            Event::PI(_) => {
                // Processing instructions are generally allowed
            }
            Event::DocType(_) => {
                // DocType validation is handled by security module
            }
            Event::Eof => {
                self.validate_document_end()?;
            }
        }

        Ok(())
    }

    /// Handle XML start element
    fn handle_start_element(&mut self, element: &BytesStart) -> Result<(), ParseError> {
        // Use local_name() to get just the element name without namespace prefix
        let element_name = utf8_utils::decode_utf8_at_position(
            element.local_name().as_ref(),
            self.current_position,
        )?;

        if self.strict_validation {
            // Validate element name
            if element_name.is_empty() {
                return Err(ParseError::MalformedXml {
                    message: "Empty element name".to_string(),
                    position: self.current_position,
                });
            }

            // Validate element name contains only valid XML name characters
            if !is_valid_xml_name(&element_name) {
                return Err(ParseError::MalformedXml {
                    message: format!("Invalid element name: '{}'", element_name),
                    position: self.current_position,
                });
            }
        }

        // Validate attributes if extended validation is enabled
        if self.extended_validation {
            self.validate_attributes(element)?;
        }

        // Calculate depth: depth = number of open ancestors + 1 (for this element)
        // Siblings have the same depth as each other
        let element_depth = self.element_stack.len() + 1;

        // Push element onto stack for tag matching with its depth
        self.element_stack
            .push((element_name.clone(), element_depth));

        // Update current depth to this element's depth
        self.current_depth = element_depth;

        // Debug logging disabled for production use

        Ok(())
    }

    /// Handle XML end element
    fn handle_end_element(&mut self, element: &BytesEnd) -> Result<(), ParseError> {
        // Use local_name() to get just the element name without namespace prefix
        let element_name = utf8_utils::decode_utf8_at_position(
            element.local_name().as_ref(),
            self.current_position,
        )?;

        if self.strict_validation {
            // Check if there's a matching start tag
            if let Some((expected, _depth)) = self.element_stack.pop() {
                if expected != element_name {
                    // Debug: print stack state when mismatch occurs
                    // Debug logging disabled for production use

                    return Err(ParseError::MismatchedTags {
                        expected,
                        found: element_name,
                        position: self.current_position,
                    });
                }
                // Update depth to parent's depth when exiting an element
                // After popping, stack size = parent depth
                self.current_depth = self.element_stack.len();
            } else {
                return Err(ParseError::UnexpectedClosingTag {
                    tag: element_name,
                    position: self.current_position,
                });
            }
        } else {
            // Even in lenient mode, we should pop from stack and update depth
            if let Some((_, _depth)) = self.element_stack.pop() {
                // After popping, current depth = remaining stack size
                self.current_depth = self.element_stack.len();
            }
        }

        Ok(())
    }

    /// Handle empty XML element (self-closing)
    fn handle_empty_element(&mut self, element: &BytesStart) -> Result<(), ParseError> {
        // Use local_name() to get just the element name without namespace prefix
        let element_name = utf8_utils::decode_utf8_at_position(
            element.local_name().as_ref(),
            self.current_position,
        )?;

        if self.strict_validation {
            // Validate element name
            if element_name.is_empty() {
                return Err(ParseError::MalformedXml {
                    message: "Empty element name".to_string(),
                    position: self.current_position,
                });
            }

            if !is_valid_xml_name(&element_name) {
                return Err(ParseError::MalformedXml {
                    message: format!("Invalid element name: '{}'", element_name),
                    position: self.current_position,
                });
            }
        }

        // Validate attributes if extended validation is enabled
        if self.extended_validation {
            self.validate_attributes(element)?;
        }

        // Empty elements don't need to be added to the stack since they're self-closing

        Ok(())
    }

    /// Validate text content
    fn validate_text_content(&self, text: &BytesText) -> Result<(), ParseError> {
        // Use UTF-8 utilities to safely decode text
        let _decoded = utf8_utils::handle_text_node(text, self.current_position)?;

        // Additional text validation could be added here
        // For example, checking for invalid control characters

        Ok(())
    }

    /// Validate CDATA content
    fn validate_cdata_content(&self, cdata: &[u8]) -> Result<(), ParseError> {
        // Validate CDATA is properly UTF-8 encoded
        let _decoded = utf8_utils::decode_utf8_at_position(cdata, self.current_position)?;

        // CDATA sections cannot contain "]]>" sequence except at the end
        let cdata_str = std::str::from_utf8(cdata).map_err(|e| ParseError::InvalidUtf8 {
            message: format!(
                "UTF-8 decoding error at position {}: {}",
                self.current_position + e.valid_up_to(),
                e
            ),
        })?;

        if cdata_str.contains("]]>") && !cdata_str.ends_with("]]>") {
            return Err(ParseError::MalformedXml {
                message: "CDATA section contains ']]>' in the middle".to_string(),
                position: self.current_position,
            });
        }

        Ok(())
    }

    /// Validate XML attributes
    fn validate_attributes(&self, element: &BytesStart) -> Result<(), ParseError> {
        let mut seen_attributes = std::collections::HashSet::new();

        for attr_result in element.attributes() {
            let attr = attr_result.map_err(|e| ParseError::MalformedXml {
                message: format!("Malformed attribute: {}", e),
                position: self.current_position,
            })?;

            // Decode attribute name and value
            let attr_name =
                utf8_utils::decode_attribute_name(attr.key.as_ref(), self.current_position)?;
            let attr_value =
                utf8_utils::decode_attribute_value(&attr.value, self.current_position)?;

            // Validate attribute name
            if attr_name.is_empty() {
                return Err(ParseError::InvalidAttribute {
                    message: "Empty attribute name".to_string(),
                    position: self.current_position,
                });
            }

            if !is_valid_xml_name(&attr_name) {
                return Err(ParseError::InvalidAttribute {
                    message: format!("Invalid attribute name: '{}'", attr_name),
                    position: self.current_position,
                });
            }

            // Check for duplicate attributes
            if !seen_attributes.insert(attr_name.clone()) {
                return Err(ParseError::InvalidAttribute {
                    message: format!("Duplicate attribute: '{}'", attr_name),
                    position: self.current_position,
                });
            }

            // Validate the raw (still escaped) value: a literal '<', or a '&' that does not
            // start a reference, is not well-formed. Escaped `&lt;` / `&amp;` are fine, so the
            // decoded value may contain '<' and '&'.
            let raw = &attr.value;
            if raw.contains(&b'<') || raw.contains(&b'&') && !raw.contains(&b';') {
                return Err(ParseError::InvalidAttribute {
                    message: format!("Invalid character in attribute value: '{}'", attr_value),
                    position: self.current_position,
                });
            }
        }

        Ok(())
    }

    /// Validate at document end that all elements are properly closed
    fn validate_document_end(&mut self) -> Result<(), ParseError> {
        if self.strict_validation && !self.element_stack.is_empty() {
            let unclosed_tags = self
                .element_stack
                .iter()
                .map(|(name, _)| name.clone())
                .collect();
            return Err(ParseError::UnclosedTags {
                tags: unclosed_tags,
                position: self.current_position,
            });
        }

        // Clear stack and reset depth for next document
        self.element_stack.clear();
        self.current_depth = 0;
        Ok(())
    }

    /// Get current element stack (for debugging)
    pub fn get_element_stack(&self) -> Vec<String> {
        self.element_stack
            .iter()
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Check if validator is currently inside any elements
    pub fn is_in_element(&self) -> bool {
        !self.element_stack.is_empty()
    }

    /// Get current nesting depth (actual depth, not stack size)
    pub fn get_depth(&self) -> usize {
        // Return actual stack depth, which represents nesting level
        // This fixes the sibling depth bug - siblings have the same depth as their parent + 1
        self.element_stack.len()
    }
}

/// Validate XML name according to XML 1.0 specification
/// <https://www.w3.org/TR/xml/#NT-Name>
fn is_valid_xml_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let chars: Vec<char> = name.chars().collect();

    // First character must be a letter, underscore, or colon
    if !is_name_start_char(chars[0]) {
        return false;
    }

    // Remaining characters must be name characters
    for &ch in chars.iter().skip(1) {
        if !is_name_char(ch) {
            return false;
        }
    }

    true
}

/// Check if character can start an XML name
fn is_name_start_char(ch: char) -> bool {
    ch.is_ascii_alphabetic()
        || ch == '_'
        || ch == ':'
        || ('\u{C0}'..='\u{D6}').contains(&ch)
        || ('\u{D8}'..='\u{F6}').contains(&ch)
        || ('\u{F8}'..='\u{2FF}').contains(&ch)
        || ('\u{370}'..='\u{37D}').contains(&ch)
        || ('\u{37F}'..='\u{1FFF}').contains(&ch)
        || ('\u{200C}'..='\u{200D}').contains(&ch)
        || ('\u{2070}'..='\u{218F}').contains(&ch)
        || ('\u{2C00}'..='\u{2FEF}').contains(&ch)
        || ('\u{3001}'..='\u{D7FF}').contains(&ch)
        || ('\u{F900}'..='\u{FDCF}').contains(&ch)
        || ('\u{FDF0}'..='\u{FFFD}').contains(&ch)
}

/// Check if character can be in an XML name
fn is_name_char(ch: char) -> bool {
    is_name_start_char(ch)
        || ch.is_ascii_digit()
        || ch == '-'
        || ch == '.'
        || ch == '\u{B7}'
        || ('\u{0300}'..='\u{036F}').contains(&ch)
        || ('\u{203F}'..='\u{2040}').contains(&ch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_valid_xml_names() {
        assert!(is_valid_xml_name("element"));
        assert!(is_valid_xml_name("_private"));
        assert!(is_valid_xml_name("ns:element"));
        assert!(is_valid_xml_name("element-1"));
        assert!(is_valid_xml_name("element.1"));
    }

    #[test]
    fn test_invalid_xml_names() {
        assert!(!is_valid_xml_name(""));
        assert!(!is_valid_xml_name("1element"));
        assert!(!is_valid_xml_name("-element"));
        assert!(!is_valid_xml_name(".element"));
        assert!(!is_valid_xml_name("element with spaces"));
    }

    #[test]
    fn test_validator_creation() {
        let validator = XmlValidator::default();
        assert_eq!(validator.get_depth(), 0);
        assert!(!validator.is_in_element());
    }

    #[test]
    fn test_element_stack_tracking() {
        let mut validator = XmlValidator::strict();
        let cursor = Cursor::new(b"test");
        let reader = Reader::from_reader(cursor);

        // Simulate start element
        let start_element = BytesStart::new("test");
        let start_event = Event::Start(start_element);

        validator.validate_event(&start_event, &reader).unwrap();
        assert_eq!(validator.get_depth(), 1);
        assert!(validator.is_in_element());

        // Simulate end element
        let end_element = BytesEnd::new("test");
        let end_event = Event::End(end_element);

        validator.validate_event(&end_event, &reader).unwrap();
        assert_eq!(validator.get_depth(), 0);
        assert!(!validator.is_in_element());
    }
}
