// src/streaming/mod.rs
//! Experimental, opt-in streaming DDEX parsers. They read a subset of the fields; the
//! parallel iterator does not currently scale with the thread count (planned for 0.6.0).
//! `DDEXParser::parse` does not use them. Every one rejects a DOCTYPE and references to
//! undefined entities, like `parse`. Measured figures are in `dev-docs/PERFORMANCE.md`.
//!
//! Removed in 0.6.0: `FastStreamingParser` (its element counts were wrong by construction),
//! `StreamingConfig` and `StreamingProgress` (never read or produced), `ParsedElement` /
//! `HeaderBuilder` (never produced) and `ParallelBenchmark` (not implemented).

pub mod fast_zero_copy;
pub mod parallel_parser;
pub mod working_impl;

#[cfg(test)]
mod comprehensive_tests;

#[cfg(test)]
mod debug_test;

#[cfg(test)]
mod consistency_test;

pub use fast_zero_copy::{FastZeroCopyIterator, FastZeroCopyParser};
pub use parallel_parser::{ParallelStreamingIterator, ParallelStreamingParser};
pub use working_impl::{
    WorkingStreamIterator, WorkingStreamingElement, WorkingStreamingParser, WorkingStreamingStats,
};
