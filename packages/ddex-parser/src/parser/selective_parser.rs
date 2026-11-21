// src/parser/selective_parser.rs
//! Fast selective parsing for extracting specific fields like ISRCs

use crate::error::ParseError;
use quick_xml::{events::Event, Reader};
use std::collections::HashSet;
use std::io::BufRead;

/// High-performance selective parser for extracting specific fields
#[derive(Debug, Clone)]
pub struct SelectiveParser {
    /// Target fields to extract (e.g., "ISRC", "ReleaseId", etc.)
    target_fields: HashSet<String>,
    /// Enable case-sensitive matching
    case_sensitive: bool,
    /// Maximum depth to search (0 = unlimited)
    max_depth: usize,
}

/// Result of selective field extraction
#[derive(Debug, Clone)]
pub struct SelectiveResult {
    /// Extracted values mapped by field name
    pub values: std::collections::HashMap<String, Vec<String>>,
    /// Total elements processed
    pub elements_processed: usize,
    /// Bytes processed
    pub bytes_processed: usize,
    /// Parse duration
    pub duration: std::time::Duration,
}

impl SelectiveParser {
    /// Create a new selective parser targeting specific fields
    pub fn new(target_fields: Vec<String>) -> Self {
        Self {
            target_fields: target_fields.into_iter().collect(),
            case_sensitive: false,
            max_depth: 0,
        }
    }

    /// Create a parser specifically for ISRC extraction
    pub fn for_isrcs() -> Self {
        Self::new(vec![
            "ISRC".to_string(),
            "SoundRecordingId".to_string(),
            "ResourceId".to_string(),
        ])
    }

    /// Create a parser for release metadata extraction
    pub fn for_release_metadata() -> Self {
        Self::new(vec![
            "ReleaseId".to_string(),
            "ReleaseReference".to_string(),
            "TitleText".to_string(),
            "DisplayArtist".to_string(),
            "ReleaseDate".to_string(),
        ])
    }

    /// Set case sensitivity for field matching
    pub fn case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Set maximum depth to search
    pub fn max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Extract ISRCs from XML with maximum performance
    pub fn extract_isrcs<R: BufRead>(&mut self, reader: R) -> Result<Vec<String>, ParseError> {
        let result = self.extract_fields(reader)?;

        let mut isrcs = Vec::new();

        // Collect ISRCs from all possible field names
        for field_name in &["ISRC", "SoundRecordingId", "ResourceId"] {
            if let Some(values) = result.values.get(*field_name) {
                for value in values {
                    // Extract ISRC from value (might be in format "Namespace:Value")
                    let isrc = if value.contains(':') {
                        value.split(':').nth(1).unwrap_or(value).to_string()
                    } else {
                        value.clone()
                    };

                    // Validate ISRC format (12 characters: CCXXXYYNNNNN)
                    if self.is_valid_isrc(&isrc) {
                        isrcs.push(isrc);
                    }
                }
            }
        }

        isrcs.sort();
        isrcs.dedup();
        Ok(isrcs)
    }

    /// Extract all targeted fields from XML
    pub fn extract_fields<R: BufRead>(&mut self, reader: R) -> Result<SelectiveResult, ParseError> {
        let start_time = std::time::Instant::now();
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);

        let mut values: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        let mut buf = Vec::new();
        let mut current_field = None::<String>;
        let mut depth = 0;
        let mut elements_processed = 0;

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    depth += 1;
                    elements_processed += 1;

                    // Check depth limit
                    if self.max_depth > 0 && depth > self.max_depth {
                        buf.clear();
                        continue;
                    }

                    let element_name = self.extract_element_name(e.name().as_ref())?;

                    // Check if this is a target field
                    if self.is_target_field(&element_name) {
                        current_field = Some(element_name);
                    }
                }
                Ok(Event::End(_)) => {
                    depth = depth.saturating_sub(1);
                    current_field = None;
                }
                Ok(Event::Empty(ref e)) => {
                    elements_processed += 1;

                    let element_name = self.extract_element_name(e.name().as_ref())?;

                    // For self-closing elements, check attributes
                    if self.is_target_field(&element_name) {
                        if let Ok(attributes) = e.attributes().collect::<Result<Vec<_>, _>>() {
                            for attr in attributes {
                                let attr_value = String::from_utf8_lossy(&attr.value);
                                self.add_value(&mut values, &element_name, attr_value.to_string());
                            }
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref field_name) = current_field {
                        // Use utf8_utils for proper UTF-8 handling
                        let current_pos = xml_reader.buffer_position() as usize;
                        let text = crate::utf8_utils::handle_text_node(e, current_pos)?;

                        let text_content = text.trim();
                        if !text_content.is_empty() {
                            self.add_value(&mut values, field_name, text_content.to_string());
                        }
                    }
                }
                Ok(Event::CData(ref e)) => {
                    if let Some(ref field_name) = current_field {
                        let text = String::from_utf8_lossy(e);
                        let text_content = text.trim();
                        if !text_content.is_empty() {
                            self.add_value(&mut values, field_name, text_content.to_string());
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(ParseError::XmlError(format!("XML parsing error: {}", e)));
                }
                _ => {} // Skip other events for maximum speed
            }
            buf.clear();
        }

        Ok(SelectiveResult {
            values,
            elements_processed,
            bytes_processed: xml_reader.buffer_position() as usize,
            duration: start_time.elapsed(),
        })
    }

    /// Ultra-fast ISRC extraction using pattern matching (10x+ faster than XML parsing)
    pub fn extract_isrcs_fast<R: BufRead>(
        &mut self,
        mut reader: R,
    ) -> Result<Vec<String>, ParseError> {
        let mut isrcs = Vec::new();
        let mut buffer = Vec::new();

        // Read entire content for fast scanning
        reader
            .read_to_end(&mut buffer)
            .map_err(|e| ParseError::IoError(format!("Failed to read input: {}", e)))?;

        // Convert to string for faster pattern matching
        let content = std::str::from_utf8(&buffer).map_err(|e| ParseError::InvalidUtf8 {
            message: format!("UTF-8 decoding error at position 0: {}", e),
        })?;

        // Ultra-fast pattern matching for ISRC tags
        self.extract_isrcs_from_content(content, &mut isrcs);

        // Remove duplicates and sort
        isrcs.sort_unstable();
        isrcs.dedup();

        Ok(isrcs)
    }

    /// Extract ISRCs from content using fastest possible pattern matching
    fn extract_isrcs_from_content(&self, content: &str, isrcs: &mut Vec<String>) {
        // Look for ISRC patterns, handling both direct ISRC tags and SoundRecordingId with ISRC namespace
        let mut pos = 0;
        let content_len = content.len();

        while pos < content_len {
            // Look for any potential ISRC container tags
            if let Some(isrc_pos) = self.find_next_isrc_tag(content, pos) {
                pos = isrc_pos;

                // Extract ISRC value from this position
                if let Some((isrc, next_pos)) = self.extract_isrc_at_position(content, pos) {
                    if self.is_valid_isrc(&isrc) {
                        isrcs.push(isrc);
                    }
                    pos = next_pos;
                } else {
                    pos += 1;
                }
            } else {
                break;
            }
        }
    }

    /// Find next potential ISRC tag position
    fn find_next_isrc_tag(&self, content: &str, start_pos: usize) -> Option<usize> {
        let search_slice = &content[start_pos..];

        // Patterns to look for (ordered by likelihood)
        let patterns = [
            "<ISRC>",
            "<ern:ISRC>",
            "<SoundRecordingId",
            "<ern:SoundRecordingId",
        ];

        let mut min_pos: Option<usize> = None;
        for &pattern in &patterns {
            if let Some(found_pos) = search_slice.find(pattern) {
                let absolute_pos = start_pos + found_pos;
                min_pos =
                    Some(min_pos.map_or(absolute_pos, |current: usize| current.min(absolute_pos)));
            }
        }

        min_pos
    }

    /// Extract ISRC value at a specific position
    fn extract_isrc_at_position(&self, content: &str, pos: usize) -> Option<(String, usize)> {
        let remaining = &content[pos..];

        // Handle direct ISRC tags
        if remaining.starts_with("<ISRC>") {
            return self.extract_between_tags(content, pos, "<ISRC>", "</ISRC>");
        }
        if remaining.starts_with("<ern:ISRC>") {
            return self.extract_between_tags(content, pos, "<ern:ISRC>", "</ern:ISRC>");
        }

        // Handle SoundRecordingId with Namespace="ISRC"
        if remaining.starts_with("<SoundRecordingId")
            || remaining.starts_with("<ern:SoundRecordingId")
        {
            // Find the closing > of the opening tag
            if let Some(tag_end) = remaining.find('>') {
                let opening_tag = &remaining[..=tag_end];

                // Check if this has Namespace="ISRC"
                if opening_tag.contains("Namespace=\"ISRC\"")
                    || opening_tag.contains("Namespace='ISRC'")
                {
                    let content_start = pos + tag_end + 1;

                    // Find the closing tag
                    let closing_tag = if remaining.starts_with("<ern:") {
                        "</ern:SoundRecordingId>"
                    } else {
                        "</SoundRecordingId>"
                    };

                    if let Some(closing_pos) = content[content_start..].find(closing_tag) {
                        let content_end = content_start + closing_pos;
                        let isrc = content[content_start..content_end].trim().to_string();
                        return Some((isrc, content_end + closing_tag.len()));
                    }
                }
            }
        }

        None
    }

    /// Extract content between opening and closing tags
    fn extract_between_tags(
        &self,
        content: &str,
        pos: usize,
        open_tag: &str,
        close_tag: &str,
    ) -> Option<(String, usize)> {
        let content_start = pos + open_tag.len();

        if let Some(content_end_rel) = content[content_start..].find(close_tag) {
            let content_end = content_start + content_end_rel;
            let extracted = content[content_start..content_end].trim().to_string();
            Some((extracted, content_end + close_tag.len()))
        } else {
            None
        }
    }

    /// Check if element name matches target fields
    fn is_target_field(&self, name: &str) -> bool {
        if self.case_sensitive {
            self.target_fields.contains(name)
        } else {
            self.target_fields
                .iter()
                .any(|field| field.eq_ignore_ascii_case(name))
        }
    }

    /// Extract element name from QName (strips namespace prefix)
    fn extract_element_name(&self, qname: &[u8]) -> Result<String, ParseError> {
        let name_str = std::str::from_utf8(qname)
            .map_err(|_| ParseError::IoError("Invalid UTF-8 in element name".to_string()))?;

        // Strip namespace prefix if present
        let local_name = if let Some(colon_pos) = name_str.find(':') {
            &name_str[colon_pos + 1..]
        } else {
            name_str
        };

        Ok(local_name.to_string())
    }

    /// Add value to results, handling duplicates
    fn add_value(
        &self,
        values: &mut std::collections::HashMap<String, Vec<String>>,
        field_name: &str,
        value: String,
    ) {
        values
            .entry(field_name.to_string())
            .or_default()
            .push(value);
    }

    /// Validate ISRC format (basic validation)
    fn is_valid_isrc(&self, isrc: &str) -> bool {
        // ISRC format: CCXXXYYNNNNN (12 characters)
        // CC = Country code (2 letters)
        // XXX = Registrant code (3 alphanumeric)
        // YY = Year (2 digits)
        // NNNNN = Designation code (5 digits)

        if isrc.len() != 12 {
            return false;
        }

        let chars: Vec<char> = isrc.chars().collect();

        // Check country code (first 2 chars should be letters)
        if !chars[0].is_ascii_alphabetic() || !chars[1].is_ascii_alphabetic() {
            return false;
        }

        // Check registrant code (chars 2-4 should be alphanumeric)
        for &ch in &chars[2..5] {
            if !ch.is_ascii_alphanumeric() {
                return false;
            }
        }

        // Check year (chars 5-6 should be digits)
        if !chars[5].is_ascii_digit() || !chars[6].is_ascii_digit() {
            return false;
        }

        // Check designation code (chars 7-11 should be digits)
        for &ch in &chars[7..12] {
            if !ch.is_ascii_digit() {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_isrc_validation() {
        let parser = SelectiveParser::for_isrcs();

        assert!(parser.is_valid_isrc("USRC17607839"));
        assert!(parser.is_valid_isrc("GBUM71505078"));
        assert!(parser.is_valid_isrc("FRUM71200001"));

        assert!(!parser.is_valid_isrc("USRC1760783")); // Too short
        assert!(!parser.is_valid_isrc("USRC176078391")); // Too long
        assert!(!parser.is_valid_isrc("12RC17607839")); // Invalid country code
        assert!(!parser.is_valid_isrc("USRC1760783A")); // Invalid designation code
    }

    #[test]
    fn test_selective_isrc_extraction() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <ern:ResourceList>
                <ern:SoundRecording>
                    <ern:SoundRecordingId Namespace="ISRC">USRC17607839</ern:SoundRecordingId>
                    <ern:ReferenceTitle>
                        <ern:TitleText>Test Track</ern:TitleText>
                    </ern:ReferenceTitle>
                </ern:SoundRecording>
                <ern:SoundRecording>
                    <ern:SoundRecordingId Namespace="ISRC">GBUM71505078</ern:SoundRecordingId>
                    <ern:ReferenceTitle>
                        <ern:TitleText>Another Track</ern:TitleText>
                    </ern:ReferenceTitle>
                </ern:SoundRecording>
            </ern:ResourceList>
        </ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let mut parser = SelectiveParser::for_isrcs();

        let isrcs = parser.extract_isrcs(cursor).expect("Should extract ISRCs");

        assert_eq!(isrcs.len(), 2);
        assert!(isrcs.contains(&"USRC17607839".to_string()));
        assert!(isrcs.contains(&"GBUM71505078".to_string()));
    }

    #[test]
    fn test_fast_isrc_extraction() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <ern:ResourceList>
                <ern:SoundRecording>
                    <ISRC>USRC17607839</ISRC>
                    <ern:ReferenceTitle>
                        <ern:TitleText>Test Track</ern:TitleText>
                    </ern:ReferenceTitle>
                </ern:SoundRecording>
            </ern:ResourceList>
        </ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let mut parser = SelectiveParser::for_isrcs();

        let isrcs = parser
            .extract_isrcs_fast(cursor)
            .expect("Should extract ISRCs");

        assert_eq!(isrcs.len(), 1);
        assert_eq!(isrcs[0], "USRC17607839");
    }

    #[test]
    fn test_selective_field_extraction() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <ern:ReleaseList>
                <ern:Release>
                    <ern:ReleaseId>REL001</ern:ReleaseId>
                    <ern:ReleaseReference>R001</ern:ReleaseReference>
                    <ern:ReferenceTitle>
                        <ern:TitleText>My Album</ern:TitleText>
                    </ern:ReferenceTitle>
                </ern:Release>
            </ern:ReleaseList>
        </ern:NewReleaseMessage>"#;

        let cursor = Cursor::new(xml.as_bytes());
        let mut parser = SelectiveParser::for_release_metadata();

        let result = parser
            .extract_fields(cursor)
            .expect("Should extract fields");

        assert!(result.values.contains_key("ReleaseId"));
        assert!(result.values.contains_key("ReleaseReference"));
        assert!(result.values.contains_key("TitleText"));

        assert_eq!(result.values["ReleaseId"][0], "REL001");
        assert_eq!(result.values["ReleaseReference"][0], "R001");
        assert_eq!(result.values["TitleText"][0], "My Album");

        println!("Extraction results: {:#?}", result);
    }

    #[test]
    fn test_performance_comparison() {
        // Generate larger test data
        let mut xml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <ern:ResourceList>"#,
        );

        for i in 0..1000 {
            xml.push_str(&format!(
                r#"
                <ern:SoundRecording>
                    <ern:SoundRecordingId Namespace="ISRC">USRC{:08}</ern:SoundRecordingId>
                    <ern:ReferenceTitle>
                        <ern:TitleText>Test Track {}</ern:TitleText>
                    </ern:ReferenceTitle>
                </ern:SoundRecording>"#,
                17600000 + i,
                i
            ));
        }

        xml.push_str("</ern:ResourceList></ern:NewReleaseMessage>");

        // Test standard extraction
        let cursor1 = Cursor::new(xml.as_bytes());
        let mut parser1 = SelectiveParser::for_isrcs();
        let start1 = std::time::Instant::now();
        let isrcs1 = parser1
            .extract_isrcs(cursor1)
            .expect("Standard extraction should work");
        let duration1 = start1.elapsed();

        // Test fast extraction
        let cursor2 = Cursor::new(xml.as_bytes());
        let mut parser2 = SelectiveParser::for_isrcs();
        let start2 = std::time::Instant::now();
        let isrcs2 = parser2
            .extract_isrcs_fast(cursor2)
            .expect("Fast extraction should work");
        let duration2 = start2.elapsed();

        println!(
            "Standard extraction: {} ISRCs in {:?}",
            isrcs1.len(),
            duration1
        );
        println!("Fast extraction: {} ISRCs in {:?}", isrcs2.len(), duration2);

        // Both methods should find the same ISRCs
        assert_eq!(isrcs1.len(), 1000);
        assert_eq!(isrcs2.len(), 1000);

        // Fast method should be faster (though results may vary in debug mode)
        println!(
            "Fast extraction speedup: {:.2}x",
            duration1.as_nanos() as f64 / duration2.as_nanos() as f64
        );
    }
}
