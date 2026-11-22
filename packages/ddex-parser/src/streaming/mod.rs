// src/streaming/mod.rs
//! High-performance streaming DDEX parser implementation

pub mod element;
pub mod fast_streaming_parser;
pub mod fast_zero_copy;
pub mod parallel_parser;
pub mod working_impl;

#[cfg(test)]
mod comprehensive_tests;

#[cfg(test)]
mod debug_test;

#[cfg(test)]
mod consistency_test;

pub use element::ParsedElement;
pub use fast_streaming_parser::{
    create_fast_parser, FastElementType, FastParsingStats, FastStreamingElement,
    FastStreamingIterator, FastStreamingParser,
};
pub use fast_zero_copy::{FastZeroCopyIterator, FastZeroCopyParser};
pub use parallel_parser::{ParallelBenchmark, ParallelStreamingIterator, ParallelStreamingParser};
pub use working_impl::{
    WorkingStreamIterator, WorkingStreamingElement, WorkingStreamingParser, WorkingStreamingStats,
};

use crate::parser::security::SecurityConfig;

/// Configuration for streaming parser
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Security configuration
    pub security: SecurityConfig,
    /// Buffer size for XML reading
    pub buffer_size: usize,
    /// Maximum memory usage before yielding elements
    pub max_memory: usize,
    /// Chunk size for processing
    pub chunk_size: usize,
    /// Enable progress callbacks
    pub enable_progress: bool,
    /// Progress callback interval (bytes)
    pub progress_interval: u64,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            security: SecurityConfig::default(),
            buffer_size: 8192,
            max_memory: 100 * 1024 * 1024, // 100MB
            chunk_size: 100,
            enable_progress: false,
            progress_interval: 1024 * 1024, // 1MB
        }
    }
}

/// Progress information for streaming parsing
#[derive(Debug, Clone)]
pub struct StreamingProgress {
    pub bytes_processed: u64,
    pub elements_parsed: usize,
    pub releases_parsed: usize,
    pub resources_parsed: usize,
    pub parties_parsed: usize,
    pub deals_parsed: usize,
    pub elapsed: std::time::Duration,
    pub estimated_total_bytes: Option<u64>,
    pub current_depth: usize,
    pub memory_usage: usize,
}
