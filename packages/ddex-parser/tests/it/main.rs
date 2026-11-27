//! ddex-parser integration tests, compiled as a single test binary.
//!
//! Feature-gated modules:
//! - `perf`: timing sanity limits (release only); run with `cargo test --release --features perf-tests`.

mod common;

mod attribute_flood;
mod attribute_values;
mod community_prs_regression;
mod depth_limit;
mod ern38_details;
mod ern4_build_inputs;
mod ern4_tracks;
mod error_contract;
mod honest_results;
mod large_file;
mod malformed_xml;
mod playground_regression;
mod reader_paths;
mod serialization;
mod undefined_entities;
mod utf8_handling;
mod vendor_quirks;
mod version_detection;
mod version_parsing;
mod whitespace;

#[cfg(feature = "perf-tests")]
mod perf;
