#![doc = include_str!("../README.md")]

pub mod error;
pub mod parser;
mod sanity;
pub mod streaming;
pub mod transform;
pub mod utf8_utils;
mod xml_text;

// Re-export commonly used types
pub use ddex_core::models::versions::ERNVersion;

use parser::security::SecurityConfig;
use serde::{Deserialize, Serialize};
use streaming::{StreamingConfig, WorkingStreamIterator};

#[cfg(feature = "zero-copy")]
use streaming::fast_zero_copy::FastZeroCopyIterator;

use streaming::parallel_parser::ParallelStreamingIterator;

/// Main DDEX Parser
#[derive(Debug, Clone)]
pub struct DDEXParser {
    config: SecurityConfig,
}

impl Default for DDEXParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DDEXParser {
    /// Create a new parser with default security configuration
    pub fn new() -> Self {
        Self {
            config: SecurityConfig::default(),
        }
    }

    /// Create parser with custom security configuration
    pub fn with_config(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// Parse DDEX XML from a reader
    pub fn parse<R: std::io::BufRead + std::io::Seek>(
        &mut self,
        reader: R,
    ) -> Result<ddex_core::models::flat::ParsedERNMessage, error::ParseError> {
        // Use fast streaming if enabled
        if self.config.enable_fast_streaming {
            return self.parse_fast_streaming(reader);
        }

        // Otherwise use standard path
        self.parse_with_options(reader, Default::default())
    }

    /// Parse with options
    pub fn parse_with_options<R: std::io::BufRead + std::io::Seek>(
        &mut self,
        reader: R,
        options: parser::ParseOptions,
    ) -> Result<ddex_core::models::flat::ParsedERNMessage, error::ParseError> {
        // Use fast streaming if enabled (we'll skip the options comparison for now)
        if self.config.enable_fast_streaming {
            return self.parse_fast_streaming(reader);
        }

        // Apply security config - check if external entities are disabled and we should block them
        // Note: This security check will be enhanced with XML bomb protection

        parser::parse(reader, options, &self.config)
    }

    /// Stream parse for large files using new streaming implementation
    pub fn stream<R: std::io::BufRead>(&self, reader: R) -> WorkingStreamIterator<R> {
        // For streaming, we can't detect version from reader without consuming it
        // So we default to V4_3
        let version = ddex_core::models::versions::ERNVersion::V4_3;

        WorkingStreamIterator::new(reader, version)
    }

    /// Stream parse with version detection (consumes some input to detect version)
    pub fn stream_with_version_detection<R: std::io::BufRead + std::io::Seek>(
        &self,
        mut reader: R,
    ) -> Result<WorkingStreamIterator<R>, error::ParseError> {
        // Detect version first
        let version = parser::detector::VersionDetector::detect(&mut reader)?;
        reader.seek(std::io::SeekFrom::Start(0))?;

        Ok(WorkingStreamIterator::new(reader, version))
    }

    /// Experimental low-level streaming iterator (`FastZeroCopyIterator`) that assumes
    /// ERN 4.3. It yields raw elements, not the release model; for measured throughput see
    /// `dev-docs/PERFORMANCE.md`.
    #[cfg(feature = "zero-copy")]
    pub fn stream_zero_copy<R: std::io::BufRead>(&self, reader: R) -> FastZeroCopyIterator<R> {
        let version = ddex_core::models::versions::ERNVersion::V4_3;
        FastZeroCopyIterator::new(reader, version)
    }

    /// Experimental low-level streaming iterator (`FastZeroCopyIterator`) with version
    /// detection
    #[cfg(feature = "zero-copy")]
    pub fn stream_zero_copy_with_version_detection<R: std::io::BufRead + std::io::Seek>(
        &self,
        mut reader: R,
    ) -> Result<FastZeroCopyIterator<R>, error::ParseError> {
        let version = parser::detector::VersionDetector::detect(&mut reader)?;
        reader.seek(std::io::SeekFrom::Start(0))?;

        Ok(FastZeroCopyIterator::new(reader, version))
    }

    /// Experimental parallel streaming iterator that assumes ERN 4.3. It yields raw
    /// elements, not the release model, and does not currently scale with the thread count
    /// (planned for 0.6.0; see `dev-docs/PERFORMANCE.md`).
    pub fn stream_parallel<R: std::io::BufRead>(&self, reader: R) -> ParallelStreamingIterator<R> {
        let version = ddex_core::models::versions::ERNVersion::V4_3;
        ParallelStreamingIterator::new(reader, version)
    }

    /// Parallel streaming with custom thread count
    pub fn stream_parallel_with_threads<R: std::io::BufRead>(
        &self,
        reader: R,
        threads: usize,
    ) -> ParallelStreamingIterator<R> {
        let version = ddex_core::models::versions::ERNVersion::V4_3;
        ParallelStreamingIterator::with_threads(reader, version, threads)
    }

    /// Parallel streaming with version detection
    pub fn stream_parallel_with_version_detection<R: std::io::BufRead + std::io::Seek>(
        &self,
        mut reader: R,
    ) -> Result<ParallelStreamingIterator<R>, error::ParseError> {
        let version = parser::detector::VersionDetector::detect(&mut reader)?;
        reader.seek(std::io::SeekFrom::Start(0))?;

        Ok(ParallelStreamingIterator::new(reader, version))
    }

    /// Parse with the experimental fast streaming path (`FastStreamingParser`), used when
    /// `SecurityConfig::relaxed()` / `enable_fast_streaming` is set. Known bug: it returns no
    /// releases or resources (KNOWN_ISSUES.md); use [`DDEXParser::parse`].
    pub fn parse_fast_streaming<R: std::io::BufRead>(
        &mut self,
        mut reader: R,
    ) -> Result<ddex_core::models::flat::ParsedERNMessage, error::ParseError> {
        use crate::streaming::fast_streaming_parser::{FastElementType, FastStreamingParser};

        // Create streaming config from security config
        let streaming_config = StreamingConfig {
            security: self.config.clone(),
            buffer_size: 64 * 1024,        // 64KB buffer
            max_memory: 200 * 1024 * 1024, // 200MB memory limit
            chunk_size: 512,               // 512KB chunks
            enable_progress: false,        // Disable for max speed
            progress_interval: 0,
        };

        // Create and use the ACTUAL fast parser
        let mut fast_parser = FastStreamingParser::new(streaming_config);

        // Parse using the fast streaming method
        let iterator = fast_parser.parse_streaming(&mut reader, None)?;

        // Count elements from the fast iterator
        let mut release_count = 0;
        let mut _resource_count = 0;

        for (_total_elements, element) in iterator.enumerate() {
            match element.element_type {
                FastElementType::Release => {
                    release_count += 1;
                }
                FastElementType::Resource => {
                    _resource_count += 1;
                }
                _ => {} // Handle other types as needed
            }
        }

        // Create a minimal ParsedERNMessage with the parsed data
        use ddex_core::models::common::{Identifier, IdentifierType, LocalizedString};
        use ddex_core::models::flat::{
            FlattenedMessage, MessageStats, Organization, ParsedERNMessage,
        };
        use ddex_core::models::graph::{
            ERNMessage, MessageControlType, MessageHeader, MessageRecipient, MessageSender,
            MessageType,
        };
        use ddex_core::models::versions::ERNVersion;
        use indexmap::IndexMap;

        // Create minimal flattened message
        let flat_message = FlattenedMessage {
            message_id: "FAST_STREAMING_MESSAGE".to_string(),
            message_type: "NewReleaseMessage".to_string(),
            message_date: chrono::Utc::now(),
            sender: Organization {
                name: Some("Fast Streaming Parser".to_string()),
                id: "FAST_PARSER".to_string(),
                extensions: None,
            },
            recipient: Organization {
                name: Some("Streaming Service Recipient".to_string()),
                id: "STREAMING_SERVICE_RECIPIENT".to_string(),
                extensions: None,
            },
            releases: Vec::new(), // TODO: Convert FastStreamingElements to ParsedReleases
            resources: IndexMap::new(), // TODO: Convert FastStreamingElements to ParsedResources
            deals: Vec::new(),
            parties: IndexMap::new(),
            version: "4.3".to_string(),
            profile: None,
            stats: MessageStats {
                release_count,
                track_count: 0,
                deal_count: 0,
                total_duration: 0,
            },
            extensions: None,
            message_control_type: None,
        };

        // Create minimal graph message (placeholder)
        let graph_message = ERNMessage {
            message_header: MessageHeader {
                message_id: "FAST_STREAMING_MESSAGE".to_string(),
                message_type: MessageType::NewReleaseMessage,
                message_created_date_time: chrono::Utc::now(),
                message_sender: MessageSender {
                    party_id: vec![Identifier {
                        id_type: IdentifierType::Proprietary,
                        value: "FAST_PARSER".to_string(),
                        namespace: Some("PADPIDA".to_string()),
                    }],
                    party_name: vec![LocalizedString {
                        text: "Fast Streaming Parser".to_string(),
                        language_code: Some("en".to_string()),
                        script: None,
                    }],
                    trading_name: None,
                    attributes: None,
                    extensions: None,
                    comments: None,
                },
                message_recipient: MessageRecipient {
                    party_id: vec![Identifier {
                        id_type: IdentifierType::Proprietary,
                        value: "STREAMING_SERVICE_RECIPIENT".to_string(),
                        namespace: Some("PADPIDA".to_string()),
                    }],
                    party_name: vec![LocalizedString {
                        text: "Streaming Service Recipient".to_string(),
                        language_code: Some("en".to_string()),
                        script: None,
                    }],
                    trading_name: None,
                    attributes: None,
                    extensions: None,
                    comments: None,
                },
                message_control_type: Some(MessageControlType::LiveMessage),
                message_thread_id: None,
                attributes: None,
                extensions: None,
                comments: None,
            },
            parties: Vec::new(),
            resources: Vec::new(),
            releases: Vec::new(),
            deals: Vec::new(),
            version: ERNVersion::V4_3,
            profile: None,
            message_audit_trail: None,
            attributes: None,
            extensions: None,
            legacy_extensions: None,
            comments: None,
            track_releases: Vec::new(),
        };

        let message = ParsedERNMessage {
            graph: graph_message,
            flat: flat_message,
            extensions: None,
        };

        Ok(message)
    }

    /// Detect DDEX version from XML
    pub fn detect_version<R: std::io::BufRead>(
        &self,
        reader: R,
    ) -> Result<ddex_core::models::versions::ERNVersion, error::ParseError> {
        parser::detector::VersionDetector::detect(reader)
    }

    /// Quick check of a document: detects the ERN version (from the root element's
    /// namespace, like [`DDEXParser::detect_version`]) and checks that the XML is
    /// well-formed (syntax, matching end tags, attributes, entity references, one root
    /// element, no DTD). It does not validate against the DDEX XSD or the AVS rules, and it
    /// does not build the message model: [`DDEXParser::parse`] can still fail on a document
    /// that passes.
    ///
    /// When a version is detected the result is `Ok`: `is_valid` is true when no problem
    /// was found, and `errors` lists the problems otherwise. When no ERN version can be
    /// detected (not XML, or not an ERN document) the result is an `Err` naming every
    /// problem found, since [`SanityCheckResult::version`] cannot express an unknown version.
    pub fn sanity_check<R: std::io::BufRead>(
        &self,
        mut reader: R,
    ) -> Result<SanityCheckResult, error::ParseError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        let report = sanity::check(&bytes);
        match report.version {
            Some(version) => Ok(SanityCheckResult {
                is_valid: report.errors.is_empty(),
                version,
                errors: report.errors,
                warnings: report.warnings,
            }),
            None => Err(error::ParseError::XmlError(report.errors.join("; "))),
        }
    }
}

// Old StreamIterator removed - now using DDEXStreamIterator from streaming module

/// Result of [`DDEXParser::sanity_check`]: the detected ERN version and the
/// well-formedness problems found (`is_valid` is true when there are none).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanityCheckResult {
    pub is_valid: bool,
    pub version: ddex_core::models::versions::ERNVersion,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Benchmark report support
#[cfg(feature = "bench")]
pub mod bench_report;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = DDEXParser::new();
        assert!(parser.config.disable_external_entities);
    }
}

#[cfg(test)]
mod api_integration_test;
