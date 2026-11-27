#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod api_security;
pub mod ast;
pub mod builder;
pub mod canonical;
pub mod error;
pub mod from_parsed;
pub mod generator;
pub mod linker;
pub mod output;
pub mod preflight;
pub mod round_trip;
pub mod schema;
pub mod security;
pub mod streaming;
pub mod versions;
pub(crate) mod xml_escape;

// Re-export main types
pub use builder::{BuildOptions, BuildRequest, BuildResult, DDEXBuilder, StatisticsBuildResult};
pub use canonical::DB_C14N;
pub use error::{BuildError, BuildWarning};
pub use from_parsed::FromParsed;
pub use linker::{EntityType, LinkerConfig, LinkingError, ReferenceLinker};
pub use output::{CanonMode, IndentChar, OutputConfig};
pub use preflight::{PreflightLevel, PreflightValidator, ValidationConfig, ValidationResult};
pub use schema::{JsonSchema, SchemaValidator};
pub use versions::DdexVersion;

// Security module exports
pub use api_security::{ApiSecurityManager, BatchStats, FfiDataType};
pub use security::{InputValidator, OutputSanitizer, RateLimiter, SecureTempFile, SecurityConfig};

// Round trips
pub use round_trip::{RoundTripResult, RoundTripTester};

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Version of the DB-C14N specification
pub const DB_C14N_VERSION: &str = "1.0";

/// Canonicalization algorithms
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalizationAlgorithm {
    /// No canonicalization (the input is returned unchanged)
    None,
    /// DDEX-specific DB-C14N/1.0 algorithm (default)
    #[default]
    DbC14N,
}

/// Statistics counted from the output of [`DDEXBuilder::build_with_statistics`]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildStatistics {
    /// Total build time
    pub build_time: Duration,
    /// Number of elements
    pub element_count: usize,
    /// Number of attributes (namespace declarations excluded)
    pub attribute_count: usize,
    /// Size of the output in bytes
    pub output_size_bytes: usize,
    /// Number of namespace declarations
    pub namespace_count: usize,
    /// Number of comments
    pub comment_count: usize,
    /// Number of processing instructions
    pub processing_instruction_count: usize,
}

/// Version information for the builder
pub fn version_info() -> String {
    format!(
        "DDEX Builder v{} • DB-C14N/{} • Rust {}",
        env!("CARGO_PKG_VERSION"),
        DB_C14N_VERSION,
        env!("CARGO_PKG_RUST_VERSION", "unknown")
    )
}

/// Counts the elements, attributes (namespace declarations excluded), namespace
/// declarations, comments and processing instructions of `xml` into `stats`.
pub(crate) fn count_xml(xml: &str, stats: &mut BuildStatistics) {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                stats.element_count += 1;
                for attr in e.attributes().flatten() {
                    let key = attr.key.as_ref();
                    if key == b"xmlns" || key.starts_with(b"xmlns:") {
                        stats.namespace_count += 1;
                    } else {
                        stats.attribute_count += 1;
                    }
                }
            }
            Ok(Event::Comment(_)) => stats.comment_count += 1,
            Ok(Event::PI(_)) => stats.processing_instruction_count += 1,
            Ok(Event::Eof) | Err(_) => break,
            Ok(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let info = version_info();
        assert!(info.contains("DDEX Builder"));
        assert!(info.contains("DB-C14N/1.0"));
    }
}
