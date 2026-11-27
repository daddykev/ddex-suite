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

/// Main parser options. (Before 0.6.0 it also had `auto_threshold`, `include_raw`,
/// `allow_blocking`, `include_raw_extensions`, `include_comments` and
/// `preserve_unknown_elements`, which no parse path read.)
#[derive(Debug, Clone, PartialEq)]
pub struct ParseOptions {
    /// `Dom` (default, complete) or the experimental, opt-in `Stream` path.
    pub mode: mode::ParseMode,
    /// Check references after a DOM parse; unresolved ones are logged through `tracing`
    /// (the parsed message is the same either way).
    pub resolve_references: bool,
    /// `ParseMode::Stream` only: fail when the estimated memory use exceeds this (bytes).
    pub max_memory: usize,
    /// DOM parse: fail with `ParseError::Timeout` when the parse took longer than this
    /// (milliseconds; checked once the parse is done).
    pub timeout_ms: u64,
    /// `ParseMode::Stream` only: yield the thread (`std::thread::yield_now`) every
    /// `chunk_size` releases.
    pub chunk_size: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            mode: mode::ParseMode::Dom,
            resolve_references: true,
            max_memory: 100 * 1024 * 1024, // 100MB
            timeout_ms: 30000,             // 30 seconds
            chunk_size: 100,
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

    match options.mode {
        mode::ParseMode::Dom => {
            // Complete parser, for any input size
            dom::parse_dom(reader, version, options, security_config)
        }
        mode::ParseMode::Stream => {
            // Experimental streaming path, only when requested explicitly
            stream::parse_streaming(reader, version, options, security_config)
        }
    }
}

pub mod version_ext;
