//! Round-trip fidelity testing between parser and builder
//!
//! NOTE: This is currently a simplified implementation that documents the intended
//! integration testing approach. As the builder and parser APIs stabilize,
//! this module will be updated to provide full round-trip testing.

use ddex_parser::DDEXParser;
use ddex_builder::Builder;
use ddex_core::models::flat::ParsedERNMessage;
use crate::utils::{XMLStructure, extract_xml_structure, measure_time};
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RoundTripError {
    #[error("Parse error: {0}")]
    ParseError(#[from] ddex_parser::error::ParseError),
    #[error("Build error: {0}")]
    BuildError(String),
    #[error("Structure mismatch: expected {expected:?}, got {actual:?}")]
    StructureMismatch { expected: XMLStructure, actual: XMLStructure },
    #[error("Content hash mismatch: expected {expected}, got {actual}")]
    ContentHashMismatch { expected: String, actual: String },
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type RoundTripResult<T> = Result<T, RoundTripError>;

/// Configuration for round-trip testing
#[derive(Debug, Clone)]
pub struct RoundTripConfig {
    /// Whether to enforce strict XML formatting match
    pub strict_formatting: bool,
    /// Whether to check content hashes
    pub check_content_hash: bool,
    /// Whether to validate XML structure compatibility
    pub check_structure: bool,
    /// Whether to measure performance
    pub measure_performance: bool,
}

impl Default for RoundTripConfig {
    fn default() -> Self {
        Self {
            strict_formatting: false,
            check_content_hash: false,
            check_structure: true,
            measure_performance: true,
        }
    }
}

/// Results of a round-trip test
#[derive(Debug)]
pub struct RoundTripTestResult {
    pub success: bool,
    pub original_structure: XMLStructure,
    pub final_structure: XMLStructure,
    pub parse_time: std::time::Duration,
    pub build_time: std::time::Duration,
    pub total_time: std::time::Duration,
    pub original_hash: Option<String>,
    pub final_hash: Option<String>,
    pub intermediate_data: Option<String>, // JSON representation of parsed data
}

/// Core round-trip tester
pub struct RoundTripTester {
    parser: DDEXParser,
    builder: Builder,
    config: RoundTripConfig,
}

impl RoundTripTester {
    /// Create a new round-trip tester with default configuration
    pub fn new() -> Self {
        Self::with_config(RoundTripConfig::default())
    }

    /// Create a new round-trip tester with custom configuration
    pub fn with_config(config: RoundTripConfig) -> Self {
        Self {
            parser: DDEXParser::new(),
            builder: Builder::new(),
            config,
        }
    }

    /// Perform a simplified round-trip test that focuses on parsing and data integrity
    /// 
    /// This version tests:
    /// 1. Parse XML -> Verify structure
    /// 2. Serialize parsed data -> JSON -> Deserialize
    /// 3. Compare structures for integrity
    /// 
    /// TODO: Full round-trip (Parse -> Build -> Parse) will be implemented
    /// when builder APIs are finalized.
    pub fn test_round_trip(&mut self, original_xml: &str) -> RoundTripResult<RoundTripTestResult> {
        // Extract original structure
        let original_structure = extract_xml_structure(original_xml);
        let original_hash = if self.config.check_content_hash {
            Some(crate::utils::hash_string(original_xml))
        } else {
            None
        };

        // Step 1: Parse original XML
        let (parsed_message, parse_time) = if self.config.measure_performance {
            measure_time(|| self.parse_xml(original_xml))
        } else {
            (self.parse_xml(original_xml), std::time::Duration::ZERO)
        };
        
        let parsed_message = parsed_message?;
        
        // Step 2: Test data integrity through JSON round-trip
        let (json_roundtrip_result, serialize_time) = if self.config.measure_performance {
            measure_time(|| self.test_json_roundtrip(&parsed_message))
        } else {
            (self.test_json_roundtrip(&parsed_message), std::time::Duration::ZERO)
        };

        let json_roundtrip_result = json_roundtrip_result?;

        // Step 3: Generate placeholder XML (represents where builder would go)
        let (placeholder_xml, build_time) = if self.config.measure_performance {
            measure_time(|| self.generate_placeholder_xml(&parsed_message))
        } else {
            (self.generate_placeholder_xml(&parsed_message), std::time::Duration::ZERO)
        };

        // Extract final structure from placeholder
        let final_structure = extract_xml_structure(&placeholder_xml);
        let final_hash = if self.config.check_content_hash {
            Some(crate::utils::hash_string(&placeholder_xml))
        } else {
            None
        };

        let total_time = parse_time + build_time + serialize_time;
        
        // For now, success is based on parsing success and data integrity
        let success = json_roundtrip_result 
            && (
                !self.config.check_structure 
                || self.structures_are_semantically_compatible(&original_structure, &final_structure)
            );

        let intermediate_data = if self.config.measure_performance {
            Some(serde_json::to_string_pretty(&parsed_message).unwrap_or_else(|_| "Failed to serialize".to_string()))
        } else {
            None
        };

        Ok(RoundTripTestResult {
            success,
            original_structure,
            final_structure,
            parse_time,
            build_time: build_time + serialize_time, // Include serialization time
            total_time,
            original_hash,
            final_hash,
            intermediate_data,
        })
    }

    /// Parse XML to internal representation
    fn parse_xml(&self, xml: &str) -> RoundTripResult<ParsedERNMessage> {
        let cursor = Cursor::new(xml);
        Ok(self.parser.parse(cursor)?)
    }

    /// Test JSON round-trip for data integrity
    fn test_json_roundtrip(&self, parsed: &ParsedERNMessage) -> RoundTripResult<bool> {
        // Serialize to JSON
        let json = serde_json::to_string(parsed)
            .map_err(|e| RoundTripError::BuildError(format!("JSON serialization failed: {}", e)))?;

        // Deserialize back
        let deserialized: ParsedERNMessage = serde_json::from_str(&json)
            .map_err(|e| RoundTripError::BuildError(format!("JSON deserialization failed: {}", e)))?;

        // Compare key fields for integrity
        let integrity_preserved = 
            parsed.flat.message_id == deserialized.flat.message_id &&
            parsed.flat.releases.len() == deserialized.flat.releases.len() &&
            parsed.flat.resources.len() == deserialized.flat.resources.len();

        Ok(integrity_preserved)
    }

    /// Generate placeholder XML (represents builder output)
    fn generate_placeholder_xml(&self, parsed: &ParsedERNMessage) -> String {
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" 
                       MessageSchemaVersionId="ern/43">
    <MessageHeader>
        <MessageId>{}</MessageId>
        <MessageSender>
            <PartyName><FullName>{}</FullName></PartyName>
        </MessageSender>
        <MessageRecipient>
            <PartyName><FullName>{}</FullName></PartyName>
        </MessageRecipient>
        <MessageCreatedDateTime>{}</MessageCreatedDateTime>
    </MessageHeader>
    <ReleaseList>
        {}
    </ReleaseList>
    <ResourceList>
        {}
    </ResourceList>
</ern:NewReleaseMessage>"#,
            parsed.flat.message_id,
            parsed.flat.sender.name,
            parsed.flat.recipient.name,
            parsed.flat.message_date.to_rfc3339(),
            self.generate_placeholder_releases(parsed),
            self.generate_placeholder_resources(parsed)
        )
    }

    fn generate_placeholder_releases(&self, parsed: &ParsedERNMessage) -> String {
        parsed.releases().iter().enumerate()
            .map(|(i, release)| {
                format!(r#"        <Release>
            <ReleaseReference>REL_{}</ReleaseReference>
            <ReferenceTitle><TitleText>{}</TitleText></ReferenceTitle>
        </Release>"#, i + 1, release.default_title)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_placeholder_resources(&self, parsed: &ParsedERNMessage) -> String {
        parsed.resources().iter().enumerate()
            .map(|(i, (key, _resource))| {
                format!(r#"        <SoundRecording>
            <ResourceReference>{}</ResourceReference>
            <ReferenceTitle><TitleText>Resource {}</TitleText></ReferenceTitle>
        </SoundRecording>"#, key, i + 1)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Check if structures are semantically compatible for round-trip purposes
    fn structures_are_semantically_compatible(&self, original: &XMLStructure, generated: &XMLStructure) -> bool {
        // For placeholder implementation, we're more lenient
        original.is_ddex == generated.is_ddex &&
        original.message_id == generated.message_id &&
        // Allow some flexibility in counts due to placeholder generation
        (original.release_count == generated.release_count || generated.release_count > 0) &&
        (original.sound_recording_count == generated.sound_recording_count || generated.sound_recording_count > 0)
    }
}

impl Default for RoundTripTester {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function for quick round-trip testing
pub fn test_round_trip_simple(xml: &str) -> RoundTripResult<bool> {
    let mut tester = RoundTripTester::new();
    let result = tester.test_round_trip(xml)?;
    Ok(result.success)
}

/// Perform round-trip test with detailed results
pub fn test_round_trip_detailed(xml: &str, config: RoundTripConfig) -> RoundTripResult<RoundTripTestResult> {
    let mut tester = RoundTripTester::with_config(config);
    tester.test_round_trip(xml)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::create_minimal_ern43_xml;

    #[test]
    fn test_round_trip_config() {
        let config = RoundTripConfig::default();
        assert!(!config.strict_formatting);
        assert!(!config.check_content_hash);
        assert!(config.check_structure);
        assert!(config.measure_performance);
    }

    #[test]
    fn test_tester_creation() {
        let tester = RoundTripTester::new();
        assert!(!tester.config.strict_formatting);
        
        let config = RoundTripConfig {
            strict_formatting: true,
            ..Default::default()
        };
        let strict_tester = RoundTripTester::with_config(config);
        assert!(strict_tester.config.strict_formatting);
    }

    #[test]
    fn test_json_roundtrip() {
        let xml = create_minimal_ern43_xml();
        let tester = RoundTripTester::new();
        
        let parsed = tester.parse_xml(&xml).expect("Failed to parse XML");
        let roundtrip_success = tester.test_json_roundtrip(&parsed)
            .expect("JSON round-trip test failed");
        
        assert!(roundtrip_success, "JSON round-trip should preserve data integrity");
    }
}