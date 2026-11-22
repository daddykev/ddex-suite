// core/src/parser/mod.rs
//! Parser module

pub mod attribute_extractor;
pub mod detector;
pub mod dom;
pub mod extension_capture;
pub mod mode;
pub mod multi_release_parser;
pub mod namespace_detector;
pub mod security;
pub mod selective_parser;
pub mod stream;
pub mod xml_validator;
pub mod xpath_selector;

#[cfg(test)]
mod tests;

use crate::error::ParseError;
use ddex_core::models::flat::ParsedERNMessage;
use std::io::BufRead;

/// Main parser options
#[derive(Debug, Clone, PartialEq)]
pub struct ParseOptions {
    /// `Dom` (default, complete) or the experimental, opt-in `Stream` path.
    pub mode: mode::ParseMode,
    /// Unused since 0.5.0: `Auto` no longer switches to `Stream` above a size threshold.
    pub auto_threshold: u64,
    pub resolve_references: bool,
    pub include_raw: bool,
    pub max_memory: usize,
    pub timeout_ms: u64,
    pub allow_blocking: bool,
    pub include_raw_extensions: bool,
    pub include_comments: bool,
    pub preserve_unknown_elements: bool,
    pub chunk_size: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            mode: mode::ParseMode::Dom,
            auto_threshold: 10 * 1024 * 1024, // 10MB
            resolve_references: true,
            include_raw: false,
            max_memory: 100 * 1024 * 1024, // 100MB
            timeout_ms: 30000,             // 30 seconds
            allow_blocking: false,
            chunk_size: 100,
            include_raw_extensions: false,
            include_comments: false,
            preserve_unknown_elements: false,
        }
    }
}

/// Parse DDEX XML with the complete (DOM) parser, or with the experimental streaming path
/// when `options.mode` is `ParseMode::Stream`.
pub fn parse<R: BufRead + std::io::Seek>(
    mut reader: R,
    options: ParseOptions,
    security_config: &security::SecurityConfig,
) -> Result<ParsedERNMessage, ParseError> {
    // Detect version first - this now validates XML
    let version = detector::VersionDetector::detect(&mut reader)?;
    reader.seek(std::io::SeekFrom::Start(0))?;

    // Select parsing mode
    let mode_selector = mode::ModeSelector::new(options.auto_threshold);
    let selected_mode = mode_selector.select_mode(&mut reader, options.mode)?;
    reader.seek(std::io::SeekFrom::Start(0))?;

    match selected_mode {
        mode::ParseMode::Dom => {
            // Complete parser, for any input size
            dom::parse_dom(reader, version, options, security_config)
        }
        mode::ParseMode::Stream => {
            // Experimental streaming path, only when requested explicitly
            stream::parse_streaming(reader, version, options, security_config)
        }
        mode::ParseMode::Auto => unreachable!(), // Already resolved
    }
}

pub mod version_ext;

impl ParseOptions {
    pub fn with_extensions() -> Self {
        Self {
            include_raw_extensions: true,
            include_comments: true,
            preserve_unknown_elements: true,
            ..Default::default()
        }
    }

    pub fn for_round_trip() -> Self {
        Self {
            include_raw_extensions: true,
            include_comments: true,
            preserve_unknown_elements: true,
            resolve_references: false,
            ..Default::default()
        }
    }
}
