#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod api_security;
pub mod ast;
pub mod builder;
pub mod canonical;
pub mod determinism;
pub mod error;
pub mod generator;
pub mod id_generator;
pub mod linker;
pub mod preflight;
pub mod round_trip;
pub mod schema;
pub mod security;
pub mod streaming;
pub mod versions;
pub(crate) mod xml_escape;

// Re-export main types
pub use builder::{BuildOptions, BuildRequest, BuildResult, DDEXBuilder};
pub use canonical::DB_C14N;
pub use determinism::DeterminismConfig;
pub use error::{BuildError, BuildWarning};
pub use id_generator::{HashAlgorithm, StableHashConfig, StableHashGenerator};
pub use linker::{EntityType, LinkerConfig, LinkingError, ReferenceLinker};
pub use preflight::{PreflightLevel, PreflightValidator, ValidationConfig, ValidationResult};
pub use schema::{JsonSchema, MessageProfile, SchemaValidator};
pub use versions::DdexVersion;

// Security module exports
pub use api_security::{ApiSecurityManager, BatchStats, FfiDataType};
pub use security::{InputValidator, OutputSanitizer, RateLimiter, SecureTempFile, SecurityConfig};

// Round trips
pub use round_trip::RoundTripTester;

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Version of the DB-C14N specification
pub const DB_C14N_VERSION: &str = "1.0";

/// Options for [`Builder::build_with_fidelity`] and [`Builder::canonicalize`]. (Before 0.6.0
/// it also had preservation and verification options, which nothing applied.)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FidelityOptions {
    /// Algorithm used by [`Builder::canonicalize`]. `build_with_fidelity` always writes
    /// DB-C14N/1.0 output, as `DDEXBuilder::build` does.
    pub canonicalization: CanonicalizationAlgorithm,
    /// Return [`BuildStatistics`] from `build_with_fidelity`
    pub collect_statistics: bool,
}

/// Canonicalization algorithms
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalizationAlgorithm {
    /// No canonicalization (the input is returned unchanged)
    None,
    /// DDEX-specific DB-C14N/1.0 algorithm (default)
    #[default]
    DbC14N,
}

/// Statistics counted from the output of [`Builder::build_with_fidelity`]
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

/// Builder facade: target version, statistics, canonicalization and round trips.
///
/// To generate XML, use [`DDEXBuilder::build`]. The target version set here does not
/// affect `DDEXBuilder::build` output (KNOWN_ISSUES.md).
///
/// ## Features
///
/// - **Canonical Output**: DB-C14N/1.0 ordering and formatting; byte-for-byte
///   determinism is not complete yet (see the [`determinism`] module; planned for 0.6.0)
/// - **Versions**: ERN 3.8.2, 4.2 and 4.3 (`supported_versions`)
/// - **Security**: Built-in validation, rate limiting, and XXE protection
/// - **Streaming**: an experimental streaming writer (`streaming::StreamingBuilder`),
///   which writes only the old ERN 3.8-style tree (KNOWN_ISSUES.md)
///
/// ## Usage Patterns
///
/// ### Basic Usage
///
/// ```rust
/// use ddex_builder::Builder;
///
/// let builder = Builder::new();
/// println!("Supported versions: {:?}", builder.supported_versions());
/// ```
///
/// ## Thread Safety
///
/// `Builder` is `Send + Sync` and can be safely shared between threads.
/// Each thread should create its own instance for best performance.
///
/// ## Memory Usage
///
/// Memory use has not been measured. Build times are in `dev-docs/PERFORMANCE.md`.
#[derive(Debug, Clone)]
pub struct Builder {
    target_version: Option<DdexVersion>,
    fidelity_options: FidelityOptions,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Creates a new DDEX Builder with default options and no target version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ddex_builder::Builder;
    ///
    /// let builder = Builder::new();
    /// assert!(builder.target_version().is_none());
    /// ```
    pub fn new() -> Self {
        Self::with_fidelity_options(FidelityOptions::default())
    }

    /// Create builder with the given options
    pub fn with_fidelity_options(fidelity_options: FidelityOptions) -> Self {
        Self {
            target_version: None,
            fidelity_options,
        }
    }

    /// Get the current options
    pub fn fidelity_options(&self) -> &FidelityOptions {
        &self.fidelity_options
    }

    /// Set the options
    pub fn set_fidelity_options(&mut self, options: FidelityOptions) -> &mut Self {
        self.fidelity_options = options;
        self
    }

    /// Set the canonicalization algorithm used by [`Builder::canonicalize`]
    pub fn with_canonicalization(&mut self, algorithm: CanonicalizationAlgorithm) -> &mut Self {
        self.fidelity_options.canonicalization = algorithm;
        self
    }

    /// Use DB-C14N/1.0 canonicalization (the default)
    pub fn with_db_c14n(&mut self) -> &mut Self {
        self.fidelity_options.canonicalization = CanonicalizationAlgorithm::DbC14N;
        self
    }

    /// Enable statistics collection
    pub fn with_statistics(&mut self) -> &mut Self {
        self.fidelity_options.collect_statistics = true;
        self
    }

    /// Get current canonicalization algorithm
    pub fn canonicalization_algorithm(&self) -> &CanonicalizationAlgorithm {
        &self.fidelity_options.canonicalization
    }

    /// Set target DDEX version for building
    pub fn with_version(&mut self, version: DdexVersion) -> &mut Self {
        self.target_version = Some(version);
        self
    }

    /// Get the target DDEX version
    pub fn target_version(&self) -> Option<DdexVersion> {
        self.target_version
    }

    /// Get supported DDEX versions
    pub fn supported_versions(&self) -> Vec<DdexVersion> {
        versions::utils::supported_versions()
    }

    /// Build DDEX XML (DB-C14N/1.0 canonical output, as `DDEXBuilder::build`) and, with
    /// `collect_statistics`, count the output's elements, attributes, namespace declarations,
    /// comments and processing instructions.
    pub fn build_with_fidelity(
        &self,
        request: &builder::BuildRequest,
    ) -> Result<FidelityBuildResult, error::BuildError> {
        let start_time = std::time::Instant::now();
        let build_result =
            builder::DDEXBuilder::new().build(request.clone(), builder::BuildOptions::default())?;

        let statistics = self.fidelity_options.collect_statistics.then(|| {
            let mut statistics = BuildStatistics {
                build_time: start_time.elapsed(),
                output_size_bytes: build_result.xml.len(),
                ..Default::default()
            };
            count_xml(&build_result.xml, &mut statistics);
            statistics
        });

        Ok(FidelityBuildResult {
            xml: build_result.xml,
            statistics,
            // `DDEXBuilder::build` always writes DB-C14N/1.0 canonical output.
            canonicalization_applied: true,
            db_c14n_version: Some(DB_C14N_VERSION.to_string()),
        })
    }

    /// Round trip XML → Parse → Build → Parse → compare, with `parse` the parser's parse
    /// function ([`round_trip::verify_round_trip`]).
    pub fn test_round_trip_fidelity_with<F>(
        &self,
        original_xml: &str,
        parse: F,
    ) -> Result<RoundTripResult, error::BuildError>
    where
        F: Fn(&str) -> Result<ddex_core::models::flat::ParsedERNMessage, String>,
    {
        let result =
            round_trip::RoundTripTester::new().test_round_trip_with(original_xml, parse)?;
        Ok(RoundTripResult {
            success: result.success,
            original_xml: result.original_xml,
            rebuilt_xml: result.rebuilt_xml,
            byte_identical: result.byte_identical,
            differences: result.differences,
            test_time: result.test_time,
        })
    }

    /// Canonicalize XML using the configured algorithm: DB-C14N/1.0 (`DbC14N`, with
    /// [`DB_C14N`]) or nothing (`None`).
    pub fn canonicalize(&self, xml_content: &str) -> Result<String, error::BuildError> {
        match self.fidelity_options.canonicalization {
            CanonicalizationAlgorithm::None => Ok(xml_content.to_string()),
            CanonicalizationAlgorithm::DbC14N => DB_C14N::new().canonicalize(xml_content),
        }
    }
}

/// Result of [`Builder::build_with_fidelity`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityBuildResult {
    /// Generated XML output
    pub xml: String,
    /// Build statistics (if enabled)
    pub statistics: Option<BuildStatistics>,
    /// Whether canonicalization was applied (always true: the output is DB-C14N/1.0)
    pub canonicalization_applied: bool,
    /// DB-C14N version used
    pub db_c14n_version: Option<String>,
}

/// Round-trip test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundTripResult {
    /// Whether round-trip was successful
    pub success: bool,
    /// Original XML input
    pub original_xml: String,
    /// XML after build process
    pub rebuilt_xml: String,
    /// Whether XMLs are byte-identical after canonicalization
    pub byte_identical: bool,
    /// Differences found (if any)
    pub differences: Vec<String>,
    /// Time taken for round-trip test
    pub test_time: Duration,
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
fn count_xml(xml: &str, stats: &mut BuildStatistics) {
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
    fn test_builder_creation() {
        let builder = Builder::new();
        assert!(builder.target_version().is_none());
    }

    #[test]
    fn test_version_info() {
        let info = version_info();
        assert!(info.contains("DDEX Builder"));
        assert!(info.contains("DB-C14N/1.0"));
    }
}
