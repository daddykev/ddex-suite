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
use streaming::WorkingStreamIterator;

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
        self.parse_with_options(reader, Default::default())
    }

    /// Parse with options
    pub fn parse_with_options<R: std::io::BufRead + std::io::Seek>(
        &mut self,
        reader: R,
        options: parser::ParseOptions,
    ) -> Result<ddex_core::models::flat::ParsedERNMessage, error::ParseError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = DDEXParser::new();
        assert_eq!(parser.config.max_element_depth, 100);
    }
}

#[cfg(test)]
mod api_integration_test;
