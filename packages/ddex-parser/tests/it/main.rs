//! ddex-parser integration tests, compiled as a single test binary.
//!
//! Feature-gated modules:
//! - `perf`: throughput/latency targets; run with `cargo test --release --features perf-tests`.
//! - `legacy_streaming`: streaming implementations slated for removal; run with
//!   `cargo test --features legacy-streaming-tests`.

mod common;

mod community_prs_regression;
mod depth_limit;
mod ern4_tracks;
mod error_contract;
mod fast_streaming;
mod malformed_xml;
mod parse_options;
mod playground_regression;
mod utf8_handling;
mod vendor_quirks;
mod version_detection;
mod version_parsing;

#[cfg(feature = "legacy-streaming-tests")]
mod legacy_streaming;

#[cfg(feature = "perf-tests")]
mod perf;
