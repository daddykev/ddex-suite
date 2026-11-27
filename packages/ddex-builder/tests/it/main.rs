//! ddex-builder integration tests, compiled as a single test binary.
//!
//! `perf` holds latency/overhead targets; run it with
//! `cargo test -p ddex-builder --release --features perf-tests`.

mod basic;
mod bridge_round_trip;
mod canonical_debug;
mod cross_platform_determinism;
mod determinism;
mod determinism_0_6;
mod ern382_output;
mod ern38_round_trip;
mod ern38_xsd;
mod ern4_escaping;
mod ern4_samples;
mod ern4_support;
mod ern4_xsd;
mod golden_files;
mod honest_results;
mod linker;
mod linker_xml_integration;
mod round_trip;
mod security;
mod security_fixes;
mod security_integration;
mod target_version;
mod undefined_entities;
mod whitespace_round_trip;
mod xml_generation;

#[cfg(feature = "perf-tests")]
mod perf;
