#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod api_security;
pub mod ast;
pub mod builder;
pub mod caching;
pub mod canonical;
pub mod determinism;
pub mod error;
pub mod generator;
pub mod id_generator;
pub mod linker;
pub mod memory_optimization;
pub mod namespace_minimizer;
pub mod optimized_strings;
pub mod parallel_processing;
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
pub use schema::{JsonSchema, SchemaCommand, SchemaConfig, SchemaDraft, SchemaGenerator};
pub use versions::{
    ConversionOptions, ConverterResult as ConversionResult, DdexVersion, VersionConverter,
    VersionManager,
};

// Security module exports
pub use api_security::{ApiSecurityConfig, ApiSecurityManager, BatchStats, FfiDataType};
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

/// Builder facade: target version, version detection and conversion, statistics and
/// round trips.
///
/// To generate XML, use [`DDEXBuilder::build`]. The target version set here does not
/// affect `DDEXBuilder::build` output (KNOWN_ISSUES.md).
///
/// ## Features
///
/// - **Canonical Output**: DB-C14N/1.0 ordering and formatting; byte-for-byte
///   determinism is not complete yet (see the [`determinism`] module; planned for 0.6.0)
/// - **Version Management**: Support for ERN 3.8.2, 4.2, and 4.3 with conversion
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
/// ### Version Conversion
///
/// ```rust
/// use ddex_builder::{Builder, DdexVersion};
/// use ddex_builder::versions::ConversionOptions;
///
/// let builder = Builder::new();
/// let xml_content = "<test></test>";
///
/// // Check version compatibility
/// let compatible = builder.is_version_compatible(
///     DdexVersion::Ern382,
///     DdexVersion::Ern43
/// );
///
/// if compatible {
///     let options = Some(ConversionOptions::default());
///     let result = builder.convert_version(
///         xml_content,
///         DdexVersion::Ern382,
///         DdexVersion::Ern43,
///         options
///     )?;
///     if let ddex_builder::versions::ConverterResult::Success { xml, .. } = result {
///         println!("Converted XML: {}", xml);
///     }
/// }
/// # Ok::<(), ddex_builder::BuildError>(())
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
    version_manager: versions::VersionManager,
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
            version_manager: versions::VersionManager::new(),
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

    /// Detect version from XML content
    pub fn detect_version(&self, xml_content: &str) -> Result<DdexVersion, error::BuildError> {
        self.version_manager
            .detect_version(xml_content)
            .map(|detection| detection.detected_version)
            .map_err(|e| error::BuildError::InvalidFormat {
                field: "version".to_string(),
                message: format!("Version detection failed: {}", e),
            })
    }

    /// Convert XML between DDEX versions
    pub fn convert_version(
        &self,
        xml_content: &str,
        from_version: DdexVersion,
        to_version: DdexVersion,
        options: Option<ConversionOptions>,
    ) -> Result<versions::ConverterResult, error::BuildError> {
        let converter = versions::VersionConverter::new();
        Ok(converter.convert(xml_content, from_version, to_version, options))
    }

    /// Get version compatibility information
    pub fn is_version_compatible(&self, from: DdexVersion, to: DdexVersion) -> bool {
        self.version_manager.is_conversion_supported(from, to)
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
            CanonicalizationAlgorithm::DbC14N => {
                DB_C14N::new(determinism::DeterminismConfig::default()).canonicalize(xml_content)
            }
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
