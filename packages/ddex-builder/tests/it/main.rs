//! ddex-builder integration tests, compiled as a single test binary.
//!
//! `perf` holds latency/overhead targets; run it with
//! `cargo test -p ddex-builder --release --features perf-tests`.

mod basic;
mod canonical_debug;
mod cross_platform_determinism;
mod determinism;
mod entity_classification;
mod ern382_output;
mod error_sanitizer;
mod golden_files;
mod linker;
mod linker_xml_integration;
mod presets;
mod round_trip;
mod security;
mod security_fixes;
mod security_integration;
mod target_version;
mod xml_generation;

#[cfg(feature = "perf-tests")]
mod perf;
