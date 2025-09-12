// core/src/parser/mod.rs
//! Parser module

pub mod detector;
pub mod security;
pub mod stream;
pub mod mode;
pub mod dom;
pub mod extension_capture;
pub mod namespace_detector;
pub mod attribute_extractor;

#[cfg(test)]
mod tests;

use crate::error::ParseError;
use ddex_core::models::flat::ParsedERNMessage;
use std::io::BufRead;

/// Main parser options
#[derive(Debug, Clone)]
pub struct ParseOptions {
    pub mode: mode::ParseMode,
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
            mode: mode::ParseMode::Auto,
            auto_threshold: 10 * 1024 * 1024, // 10MB
            resolve_references: true,
            include_raw: false,
            max_memory: 100 * 1024 * 1024, // 100MB
            timeout_ms: 30000, // 30 seconds
            allow_blocking: false,
            chunk_size: 100,
            include_raw_extensions: false,
            include_comments: false,
            preserve_unknown_elements: false,
        }
    }
}

/// Parse DDEX XML with automatic mode selection
pub fn parse<R: BufRead + std::io::Seek>(
    mut reader: R,
    options: ParseOptions,
) -> Result<ParsedERNMessage, ParseError> {
    // Detect version first
    let version = detector::VersionDetector::detect(&mut reader)?;
    reader.seek(std::io::SeekFrom::Start(0))?;
    
    // Select parsing mode
    let mode_selector = mode::ModeSelector::new(options.auto_threshold);
    let selected_mode = mode_selector.select_mode(&mut reader, options.mode)?;
    reader.seek(std::io::SeekFrom::Start(0))?;
    
    match selected_mode {
        mode::ParseMode::Dom => {
            // Use DOM parser for smaller files
            dom::parse_dom(reader, version, options)
        }
        mode::ParseMode::Stream => {
            // Use streaming parser for larger files
            stream::parse_streaming(reader, version, options)
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
