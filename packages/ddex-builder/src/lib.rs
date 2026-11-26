#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod api_security;
pub mod ast;
pub mod builder;
pub mod caching;
pub mod canonical;
pub mod determinism;
pub mod diff;
pub mod error;
pub mod fidelity;
pub mod generator;
pub mod guarantees;
pub mod id_generator;
pub mod linker;
pub mod memory_optimization;
pub mod messages;
pub mod namespace_minimizer;
pub mod optimized_strings;
pub mod parallel_processing;
pub mod preflight;
pub mod presets;
pub mod round_trip;
pub mod schema;
pub mod security;
pub mod streaming;
pub mod verification;
pub mod versions;
pub(crate) mod xml_escape;

// Re-export main types
pub use builder::{BuildOptions, BuildRequest, BuildResult, DDEXBuilder};
pub use canonical::DB_C14N;
pub use determinism::DeterminismConfig;
pub use diff::formatter::DiffFormatter;
pub use diff::types::{ChangeSet, ChangeType, DiffPath, ImpactLevel, SemanticChange};
pub use diff::{DiffConfig, DiffEngine, VersionCompatibility};
pub use error::{BuildError, BuildWarning};
pub use guarantees::{DeterminismGuarantee, DeterminismGuaranteeValidator, GuaranteeReport};
pub use id_generator::{HashAlgorithm, StableHashConfig, StableHashGenerator};
pub use linker::{EntityType, LinkerConfig, LinkingError, ReferenceLinker};
pub use messages::{
    UpdateAction, UpdateConfig, UpdateGenerator, UpdateReleaseMessage, ValidationStatus,
};
pub use preflight::{PreflightLevel, PreflightValidator, ValidationConfig, ValidationResult};
pub use presets::DdexVersion;
pub use presets::PartnerPreset;
pub use schema::{JsonSchema, SchemaCommand, SchemaConfig, SchemaDraft, SchemaGenerator};
pub use versions::{
    ConversionOptions, ConverterResult as ConversionResult, VersionConverter, VersionManager,
};

// Security module exports
pub use api_security::{ApiSecurityConfig, ApiSecurityManager, BatchStats, FfiDataType};
pub use security::{InputValidator, OutputSanitizer, RateLimiter, SecureTempFile, SecurityConfig};

// Perfect Fidelity Engine exports
pub use fidelity::{FidelityConfig, FidelityStatistics, PreservationLevel};
pub use round_trip::{FidelityAnalysis, RoundTripTester};
pub use verification::{BuildVerifier, VerificationStatistics};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
#[allow(
    clippy::disallowed_types,
    reason = "H4: custom canonicalization is not implemented (returns an error), so this map is never read"
)]
use std::collections::HashMap;

/// `HashMap` for the serde-derived structs below, which a field-level allow cannot cover
/// (the derived impls are separate items). Same type as `HashMap<K, V>`.
#[allow(
    clippy::disallowed_types,
    reason = "H4: custom canonicalization is not implemented (returns an error), so this map is never read"
)]
type UnorderedMap<K, V> = HashMap<K, V>;
use std::time::Duration;

/// Version of the DB-C14N specification
pub const DB_C14N_VERSION: &str = "1.0";

/// Fidelity preservation options (comments, processing instructions, extensions, ...)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityOptions {
    /// Enable fidelity preservation (the flag behind `Builder::enable_perfect_fidelity`)
    pub enable_perfect_fidelity: bool,
    /// Preserve all XML comments in their original positions
    pub preserve_comments: bool,
    /// Preserve processing instructions
    pub preserve_processing_instructions: bool,
    /// Preserve unknown/extension elements and attributes
    pub preserve_extensions: bool,
    /// Preserve original attribute ordering when possible
    pub preserve_attribute_order: bool,
    /// Preserve namespace prefixes from input
    pub preserve_namespace_prefixes: bool,
    /// Canonicalization algorithm to use
    pub canonicalization: CanonicalizationAlgorithm,
    /// Custom canonicalization rules (used with Custom algorithm)
    pub custom_canonicalization_rules: Option<CustomCanonicalizationRules>,
    /// Enable deterministic element ordering
    pub enable_deterministic_ordering: bool,
    /// Collect detailed building statistics
    pub collect_statistics: bool,
    /// Enable build verification (double-check output)
    pub enable_verification: bool,
    /// Verification configuration
    pub verification_config: VerificationConfig,
}

impl Default for FidelityOptions {
    fn default() -> Self {
        Self {
            enable_perfect_fidelity: false,
            preserve_comments: false,
            preserve_processing_instructions: false,
            preserve_extensions: true,
            preserve_attribute_order: false,
            preserve_namespace_prefixes: false,
            canonicalization: CanonicalizationAlgorithm::DbC14N,
            custom_canonicalization_rules: None,
            enable_deterministic_ordering: true,
            collect_statistics: false,
            enable_verification: false,
            verification_config: VerificationConfig::default(),
        }
    }
}

/// Canonicalization algorithms supported by the fidelity options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalizationAlgorithm {
    /// No canonicalization (preserves exact formatting)
    None,
    /// XML Canonicalization Version 1.0 (W3C C14N)
    C14N,
    /// XML Canonicalization Version 1.1 (W3C C14N11)
    C14N11,
    /// DDEX-specific DB-C14N/1.0 algorithm (default)
    DbC14N,
    /// Custom canonicalization with user-defined rules
    Custom(CustomCanonicalizationRules),
}

/// Custom canonicalization rules for specialized use cases
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomCanonicalizationRules {
    /// Preserve whitespace between elements
    pub preserve_whitespace: bool,
    /// Sort attributes alphabetically
    pub sort_attributes: bool,
    /// Normalize line endings to LF
    pub normalize_line_endings: bool,
    /// Remove redundant namespace declarations
    pub minimize_namespaces: bool,
    /// Custom attribute ordering rules
    pub attribute_ordering: Vec<String>,
    /// Custom element ordering rules
    pub element_ordering: UnorderedMap<String, Vec<String>>,
}

impl Default for CustomCanonicalizationRules {
    #[allow(
        clippy::disallowed_types,
        reason = "H4: custom canonicalization is not implemented (returns an error), so this map is never read"
    )]
    fn default() -> Self {
        Self {
            preserve_whitespace: false,
            sort_attributes: true,
            normalize_line_endings: true,
            minimize_namespaces: true,
            attribute_ordering: Vec::new(),
            element_ordering: HashMap::new(),
        }
    }
}

/// Build verification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    /// Enable round-trip verification (build -> parse -> build)
    pub enable_round_trip_verification: bool,
    /// Enable canonicalization verification
    pub enable_canonicalization_verification: bool,
    /// Enable schema validation after build
    pub enable_schema_validation: bool,
    /// Enable determinism verification (multiple builds identical)
    pub enable_determinism_verification: bool,
    /// Number of builds for determinism verification
    pub determinism_test_iterations: usize,
    /// Timeout for verification operations
    pub verification_timeout: Duration,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            enable_round_trip_verification: true,
            enable_canonicalization_verification: true,
            enable_schema_validation: false,
            enable_determinism_verification: true,
            determinism_test_iterations: 3,
            verification_timeout: Duration::from_secs(30),
        }
    }
}

/// Build verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Overall verification success
    pub success: bool,
    /// Round-trip verification result
    pub round_trip_success: bool,
    /// Canonicalization verification result
    pub canonicalization_success: bool,
    /// Schema validation result
    pub schema_validation_success: bool,
    /// Determinism verification result
    pub determinism_success: bool,
    /// Verification errors and warnings
    pub issues: Vec<VerificationIssue>,
    /// Time taken for verification
    pub verification_time: Duration,
}

/// Verification issue (error or warning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationIssue {
    /// Issue severity
    pub severity: VerificationSeverity,
    /// Issue category
    pub category: String,
    /// Human-readable message
    pub message: String,
    /// Optional path to the problematic element
    pub path: Option<String>,
    /// Optional suggestion for fixing
    pub suggestion: Option<String>,
}

/// Verification issue severity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationSeverity {
    /// Error that prevents successful verification
    Error,
    /// Warning that may indicate a problem
    Warning,
    /// Informational message
    Info,
}

/// Building statistics collected during the build process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStatistics {
    /// Total build time
    pub build_time: Duration,
    /// Time spent on canonicalization (not measured in 0.5.x: 0)
    pub canonicalization_time: Duration,
    /// Time spent on validation (not measured in 0.5.x: 0)
    pub validation_time: Duration,
    /// Time spent on verification
    pub verification_time: Duration,
    /// Peak memory usage during build (not measured in 0.5.x: 0)
    pub peak_memory_bytes: usize,
    /// Number of elements processed
    pub element_count: usize,
    /// Number of attributes processed
    pub attribute_count: usize,
    /// Size of input data (not measured in 0.5.x: 0)
    pub input_size_bytes: usize,
    /// Size of output data
    pub output_size_bytes: usize,
    /// Number of namespaces processed
    pub namespace_count: usize,
    /// Number of comments preserved
    pub comment_count: usize,
    /// Number of processing instructions preserved
    pub processing_instruction_count: usize,
}

impl Default for BuildStatistics {
    fn default() -> Self {
        Self {
            build_time: Duration::ZERO,
            canonicalization_time: Duration::ZERO,
            validation_time: Duration::ZERO,
            verification_time: Duration::ZERO,
            peak_memory_bytes: 0,
            element_count: 0,
            attribute_count: 0,
            input_size_bytes: 0,
            output_size_bytes: 0,
            namespace_count: 0,
            comment_count: 0,
            processing_instruction_count: 0,
        }
    }
}

/// Builder configuration: determinism settings, partner presets, target version and
/// fidelity options.
///
/// To generate XML, use [`DDEXBuilder::build`]. In 0.5.0 the presets and the target
/// version set here do not affect `DDEXBuilder::build` output (KNOWN_ISSUES.md).
///
/// ## Features
///
/// - **Canonical Output**: DB-C14N/1.0 ordering and formatting; byte-for-byte
///   determinism is not complete yet (see the [`determinism`] module; planned for 0.6.0)
/// - **Partner Presets**: Pre-configured settings for music platforms (they do not
///   change the output yet)
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
/// let available_presets = builder.available_presets();
/// println!("Available presets: {:?}", available_presets);
/// ```
///
/// ### With Partner Preset
///
/// ```rust
/// use ddex_builder::Builder;
///
/// let mut builder = Builder::new();
/// builder.preset("audio_album")?;
///
/// // Builder is now configured for audio album releases
/// assert!(builder.is_preset_locked() == false); // Unlocked for further customization
/// # Ok::<(), ddex_builder::BuildError>(())
/// ```
///
/// ### Locked Preset Configuration
///
/// ```rust
/// use ddex_builder::Builder;
///
/// let mut builder = Builder::new();
/// builder.apply_preset("audio_album", true)?; // Lock the preset
///
/// assert!(builder.is_preset_locked());
/// # Ok::<(), ddex_builder::BuildError>(())
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
    config: DeterminismConfig,
    presets: IndexMap<String, PartnerPreset>,
    locked_preset: Option<String>,
    version_manager: versions::VersionManager,
    target_version: Option<DdexVersion>,
    fidelity_options: FidelityOptions,
    verification_config: VerificationConfig,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Creates a new DDEX Builder with default configuration.
    ///
    /// The builder is initialized with:
    /// - Default determinism configuration (canonical ordering and formatting)
    /// - All available partner presets loaded
    /// - No preset locked (can be changed)
    /// - Latest supported DDEX version as target
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ddex_builder::Builder;
    ///
    /// let builder = Builder::new();
    /// assert!(!builder.is_preset_locked());
    /// assert!(builder.available_presets().len() > 0);
    /// ```
    ///
    /// # Performance
    ///
    /// Creating a new builder is fast (~1μs) as presets are loaded from
    /// embedded configuration data.
    pub fn new() -> Self {
        Self {
            config: DeterminismConfig::default(),
            presets: Self::load_default_presets(),
            locked_preset: None,
            version_manager: versions::VersionManager::new(),
            target_version: None,
            fidelity_options: FidelityOptions::default(),
            verification_config: VerificationConfig::default(),
        }
    }

    /// Create builder with custom configuration
    pub fn with_config(config: DeterminismConfig) -> Self {
        Self {
            config,
            presets: Self::load_default_presets(),
            locked_preset: None,
            version_manager: versions::VersionManager::new(),
            target_version: None,
            fidelity_options: FidelityOptions::default(),
            verification_config: VerificationConfig::default(),
        }
    }

    /// Create a builder with every fidelity option enabled
    pub fn with_perfect_fidelity() -> Self {
        let mut fidelity_options = FidelityOptions::default();
        fidelity_options.enable_perfect_fidelity = true;
        fidelity_options.preserve_comments = true;
        fidelity_options.preserve_processing_instructions = true;
        fidelity_options.preserve_extensions = true;
        fidelity_options.preserve_attribute_order = true;
        fidelity_options.preserve_namespace_prefixes = true;
        fidelity_options.enable_verification = true;

        Self {
            config: DeterminismConfig::default(),
            presets: Self::load_default_presets(),
            locked_preset: None,
            version_manager: versions::VersionManager::new(),
            target_version: None,
            fidelity_options,
            verification_config: VerificationConfig::default(),
        }
    }

    /// Create builder with custom fidelity options
    pub fn with_fidelity_options(fidelity_options: FidelityOptions) -> Self {
        Self {
            config: DeterminismConfig::default(),
            presets: Self::load_default_presets(),
            locked_preset: None,
            version_manager: versions::VersionManager::new(),
            target_version: None,
            fidelity_options,
            verification_config: VerificationConfig::default(),
        }
    }

    /// Create builder optimized for round-trip operations
    pub fn for_round_trip() -> Self {
        let mut fidelity_options = FidelityOptions::default();
        fidelity_options.enable_perfect_fidelity = true;
        fidelity_options.preserve_comments = true;
        fidelity_options.preserve_processing_instructions = true;
        fidelity_options.preserve_extensions = true;
        fidelity_options.preserve_attribute_order = true;
        fidelity_options.preserve_namespace_prefixes = true;
        fidelity_options.canonicalization = CanonicalizationAlgorithm::DbC14N;
        fidelity_options.enable_verification = true;
        fidelity_options.collect_statistics = true;

        let mut verification_config = VerificationConfig::default();
        verification_config.enable_round_trip_verification = true;
        verification_config.enable_canonicalization_verification = true;
        verification_config.enable_determinism_verification = true;

        Self {
            config: DeterminismConfig::default(),
            presets: Self::load_default_presets(),
            locked_preset: None,
            version_manager: versions::VersionManager::new(),
            target_version: None,
            fidelity_options,
            verification_config,
        }
    }

    /// Applies a partner preset configuration to the builder.
    ///
    /// Presets contain pre-configured settings for specific music platforms and
    /// distribution partners (determinism settings, validation rules, format preferences).
    /// In 0.5.0 they do not change `DDEXBuilder::build` output (KNOWN_ISSUES.md).
    ///
    /// # Arguments
    ///
    /// * `preset_name` - Name of the preset to apply (see [`available_presets`])
    /// * `lock` - Whether to lock the preset to prevent further modifications
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Preset applied successfully
    /// * `Err(BuildError::InvalidFormat)` - Unknown preset name
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ddex_builder::Builder;
    ///
    /// let mut builder = Builder::new();
    ///
    /// // Apply the generic audio album preset without locking
    /// builder.apply_preset("audio_album", false)?;
    /// assert!(!builder.is_preset_locked());
    ///
    /// // Apply and lock the YouTube video preset
    /// builder.apply_preset("youtube_video", true)?;
    /// assert!(builder.is_preset_locked());
    /// # Ok::<(), ddex_builder::BuildError>(())
    /// ```
    ///
    /// # Available Presets
    ///
    /// Built-in presets: `audio_album`, `audio_single`, `video_single`, `compilation`,
    /// `youtube_album`, `youtube_video`, `youtube_single`.
    /// - `universal_basic` - Universal Music basic preset
    /// - `sony_enhanced` - Sony Music enhanced features
    ///
    /// Use [`available_presets`] to get the complete list.
    ///
    /// [`available_presets`]: Self::available_presets
    pub fn apply_preset(&mut self, preset_name: &str, lock: bool) -> Result<(), error::BuildError> {
        let preset = self
            .presets
            .get(preset_name)
            .ok_or_else(|| error::BuildError::InvalidFormat {
                field: "preset".to_string(),
                message: format!("Unknown preset: {}", preset_name),
            })?
            .clone();

        // Apply the preset's determinism config
        self.config = preset.determinism;

        // Lock the preset if requested
        if lock {
            self.locked_preset = Some(preset_name.to_string());
        }

        Ok(())
    }

    /// Apply a preset configuration (alias for apply_preset for convenience)
    pub fn preset(&mut self, preset_name: &str) -> Result<&mut Self, error::BuildError> {
        self.apply_preset(preset_name, false)?;
        Ok(self)
    }

    /// Get available preset names
    pub fn available_presets(&self) -> Vec<String> {
        self.presets.keys().cloned().collect()
    }

    /// Get preset details
    pub fn get_preset(&self, preset_name: &str) -> Option<&PartnerPreset> {
        self.presets.get(preset_name)
    }

    /// Check if a preset is locked
    pub fn is_preset_locked(&self) -> bool {
        self.locked_preset.is_some()
    }

    /// Get the current configuration
    pub fn config(&self) -> &DeterminismConfig {
        &self.config
    }

    /// Get the current fidelity options
    pub fn fidelity_options(&self) -> &FidelityOptions {
        &self.fidelity_options
    }

    /// Set fidelity options
    pub fn set_fidelity_options(&mut self, options: FidelityOptions) -> &mut Self {
        self.fidelity_options = options;
        self
    }

    /// Enable every fidelity option
    pub fn enable_perfect_fidelity(&mut self) -> &mut Self {
        self.fidelity_options.enable_perfect_fidelity = true;
        self.fidelity_options.preserve_comments = true;
        self.fidelity_options.preserve_processing_instructions = true;
        self.fidelity_options.preserve_extensions = true;
        self.fidelity_options.preserve_attribute_order = true;
        self.fidelity_options.preserve_namespace_prefixes = true;
        self.fidelity_options.enable_verification = true;
        self
    }

    /// Disable every fidelity option
    pub fn disable_perfect_fidelity(&mut self) -> &mut Self {
        self.fidelity_options.enable_perfect_fidelity = false;
        self.fidelity_options.preserve_comments = false;
        self.fidelity_options.preserve_processing_instructions = false;
        self.fidelity_options.preserve_attribute_order = false;
        self.fidelity_options.preserve_namespace_prefixes = false;
        self.fidelity_options.enable_verification = false;
        self
    }

    /// Set canonicalization algorithm
    pub fn with_canonicalization(&mut self, algorithm: CanonicalizationAlgorithm) -> &mut Self {
        self.fidelity_options.canonicalization = algorithm;
        self
    }

    /// Enable DB-C14N/1.0 canonicalization (default for DDEX)
    pub fn with_db_c14n(&mut self) -> &mut Self {
        self.fidelity_options.canonicalization = CanonicalizationAlgorithm::DbC14N;
        self
    }

    /// Enable standard XML C14N canonicalization
    pub fn with_c14n(&mut self) -> &mut Self {
        self.fidelity_options.canonicalization = CanonicalizationAlgorithm::C14N;
        self
    }

    /// Enable XML C14N 1.1 canonicalization
    pub fn with_c14n11(&mut self) -> &mut Self {
        self.fidelity_options.canonicalization = CanonicalizationAlgorithm::C14N11;
        self
    }

    /// Set custom canonicalization rules
    pub fn with_custom_canonicalization(
        &mut self,
        rules: CustomCanonicalizationRules,
    ) -> &mut Self {
        self.fidelity_options.canonicalization = CanonicalizationAlgorithm::Custom(rules.clone());
        self.fidelity_options.custom_canonicalization_rules = Some(rules);
        self
    }

    /// Enable build verification
    pub fn with_verification(&mut self, config: VerificationConfig) -> &mut Self {
        self.fidelity_options.enable_verification = true;
        self.verification_config = config;
        self
    }

    /// Enable statistics collection
    pub fn with_statistics(&mut self) -> &mut Self {
        self.fidelity_options.collect_statistics = true;
        self
    }

    /// Check whether fidelity preservation is enabled
    pub fn is_perfect_fidelity_enabled(&self) -> bool {
        self.fidelity_options.enable_perfect_fidelity
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

    fn load_default_presets() -> IndexMap<String, PartnerPreset> {
        presets::all_presets()
    }

    /// Build DDEX XML (DB-C14N/1.0 canonical output, as `DDEXBuilder::build`) with optional
    /// statistics and verification. The other fidelity options are not applied in 0.5.x;
    /// a canonicalization algorithm other than `DbC14N` or `None` is an error (it is not
    /// implemented). The statistics count the output's elements, attributes, namespace
    /// declarations, comments and processing instructions; the times other than
    /// `build_time` / `verification_time`, `peak_memory_bytes` and `input_size_bytes` are
    /// not measured and stay 0.
    pub fn build_with_fidelity(
        &self,
        request: &builder::BuildRequest,
    ) -> Result<FidelityBuildResult, error::BuildError> {
        if !matches!(
            self.fidelity_options.canonicalization,
            CanonicalizationAlgorithm::DbC14N | CanonicalizationAlgorithm::None
        ) {
            canonicalize_with("", &self.fidelity_options.canonicalization)?;
        }
        let start_time = std::time::Instant::now();
        let mut statistics = BuildStatistics::default();

        // Use the existing build options structure
        let build_options = builder::BuildOptions::default();

        // Build the XML using existing builder
        let ddex_builder = builder::DDEXBuilder::new();
        let build_result = ddex_builder.build(request.clone(), build_options)?;

        statistics.build_time = start_time.elapsed();
        statistics.output_size_bytes = build_result.xml.len();
        count_xml(&build_result.xml, &mut statistics);

        // Perform verification if enabled
        let verification_result = if self.fidelity_options.enable_verification {
            // Convert lib VerificationConfig to verification VerificationConfig
            let verification_config = verification::VerificationConfig {
                enable_round_trip_verification: self
                    .verification_config
                    .enable_round_trip_verification,
                enable_canonicalization_verification: self
                    .verification_config
                    .enable_canonicalization_verification,
                enable_schema_validation: self.verification_config.enable_schema_validation,
                enable_determinism_verification: self
                    .verification_config
                    .enable_determinism_verification,
                determinism_test_iterations: self.verification_config.determinism_test_iterations,
                verification_timeout: self.verification_config.verification_timeout,
            };
            let verifier = verification::BuildVerifier::new(verification_config);
            let result = verifier.verify(&build_result.xml, &self.fidelity_options)?;
            statistics.verification_time = result.verification_time;

            // Convert verification::VerificationResult to VerificationResult
            Some(VerificationResult {
                success: result.success,
                round_trip_success: result.round_trip_success,
                canonicalization_success: result.canonicalization_success,
                determinism_success: result.determinism_success,
                schema_validation_success: result.schema_validation_success,
                issues: result
                    .issues
                    .into_iter()
                    .map(|issue| VerificationIssue {
                        category: issue.category,
                        severity: match issue.severity {
                            verification::VerificationSeverity::Error => {
                                VerificationSeverity::Error
                            }
                            verification::VerificationSeverity::Warning => {
                                VerificationSeverity::Warning
                            }
                            verification::VerificationSeverity::Info => VerificationSeverity::Info,
                        },
                        message: issue.message,
                        path: issue.path,
                        suggestion: issue.suggestion,
                    })
                    .collect(),
                verification_time: result.verification_time,
            })
        } else {
            None
        };

        Ok(FidelityBuildResult {
            xml: build_result.xml,
            statistics: if self.fidelity_options.collect_statistics {
                Some(statistics)
            } else {
                None
            },
            verification_result,
            // `DDEXBuilder::build` always writes DB-C14N/1.0 canonical output.
            canonicalization_applied: true,
            db_c14n_version: Some(DB_C14N_VERSION.to_string()),
        })
    }

    /// Verify build output meets fidelity requirements
    pub fn verify_build(&self, xml_output: &str) -> Result<VerificationResult, error::BuildError> {
        // Convert lib VerificationConfig to verification VerificationConfig
        let verification_config = verification::VerificationConfig {
            enable_round_trip_verification: self.verification_config.enable_round_trip_verification,
            enable_canonicalization_verification: self
                .verification_config
                .enable_canonicalization_verification,
            enable_schema_validation: self.verification_config.enable_schema_validation,
            enable_determinism_verification: self
                .verification_config
                .enable_determinism_verification,
            determinism_test_iterations: self.verification_config.determinism_test_iterations,
            verification_timeout: self.verification_config.verification_timeout,
        };

        let verifier = verification::BuildVerifier::new(verification_config);
        let result = verifier.verify(xml_output, &self.fidelity_options)?;

        // Convert verification::VerificationResult to VerificationResult
        Ok(VerificationResult {
            success: result.success,
            round_trip_success: result.round_trip_success,
            canonicalization_success: result.canonicalization_success,
            determinism_success: result.determinism_success,
            schema_validation_success: result.schema_validation_success,
            issues: result
                .issues
                .into_iter()
                .map(|issue| VerificationIssue {
                    category: issue.category,
                    severity: match issue.severity {
                        verification::VerificationSeverity::Error => VerificationSeverity::Error,
                        verification::VerificationSeverity::Warning => {
                            VerificationSeverity::Warning
                        }
                        verification::VerificationSeverity::Info => VerificationSeverity::Info,
                    },
                    message: issue.message,
                    path: issue.path,
                    suggestion: issue.suggestion,
                })
                .collect(),
            verification_time: result.verification_time,
        })
    }

    /// Not implemented in 0.5.x (the builder has no parser): returns an error. Use
    /// [`Builder::test_round_trip_fidelity_with`] with the parser's parse function. Before
    /// 0.5.1 this reported success without building anything.
    pub fn test_round_trip_fidelity(
        &self,
        original_xml: &str,
    ) -> Result<RoundTripResult, error::BuildError> {
        let round_trip = round_trip::RoundTripTester::new(self.fidelity_options.clone());
        let result = round_trip.test_round_trip(original_xml)?;
        Ok(Self::round_trip_result(result))
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
        let round_trip = round_trip::RoundTripTester::new(self.fidelity_options.clone());
        let result = round_trip.test_round_trip_with(original_xml, parse)?;
        Ok(Self::round_trip_result(result))
    }

    fn round_trip_result(result: round_trip::RoundTripResult) -> RoundTripResult {
        RoundTripResult {
            success: result.success,
            original_xml: result.original_xml,
            rebuilt_xml: result.rebuilt_xml,
            byte_identical: result.byte_identical,
            differences: result.differences,
            test_time: result.test_time,
        }
    }

    /// Canonicalize XML using the configured algorithm: DB-C14N/1.0 (`DbC14N`, with
    /// [`DB_C14N`]) or nothing (`None`). C14N, C14N 1.1 and custom rules are not
    /// implemented in 0.5.x and return an error (before 0.5.1 every algorithm returned the
    /// input unchanged).
    pub fn canonicalize(&self, xml_content: &str) -> Result<String, error::BuildError> {
        canonicalize_with(xml_content, &self.fidelity_options.canonicalization)
    }

    /// Get DB-C14N/1.0 configuration details
    pub fn db_c14n_config(&self) -> DbC14NConfig {
        DbC14NConfig {
            version: DB_C14N_VERSION.to_string(),
            algorithm: self.fidelity_options.canonicalization.clone(),
            deterministic_ordering: self.fidelity_options.enable_deterministic_ordering,
            preserve_comments: self.fidelity_options.preserve_comments,
            preserve_processing_instructions: self
                .fidelity_options
                .preserve_processing_instructions,
            namespace_handling: if self.fidelity_options.preserve_namespace_prefixes {
                NamespaceHandling::Preserve
            } else {
                NamespaceHandling::Minimize
            },
        }
    }

    /// Internal build method used by determinism verifier
    pub(crate) fn build_internal(
        &self,
        request: &builder::BuildRequest,
    ) -> Result<builder::BuildResult, error::BuildError> {
        let ddex_builder = builder::DDEXBuilder::new();
        let build_options = builder::BuildOptions::default();

        ddex_builder.build(request.clone(), build_options)
    }
}

/// Result of [`Builder::build_with_fidelity`]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityBuildResult {
    /// Generated XML output
    pub xml: String,
    /// Build statistics (if enabled)
    pub statistics: Option<BuildStatistics>,
    /// Verification result (if enabled)
    pub verification_result: Option<VerificationResult>,
    /// Whether canonicalization was applied
    pub canonicalization_applied: bool,
    /// DB-C14N version used (if applicable)
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

/// DB-C14N/1.0 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbC14NConfig {
    /// DB-C14N specification version
    pub version: String,
    /// Canonicalization algorithm in use
    pub algorithm: CanonicalizationAlgorithm,
    /// Whether deterministic ordering is enabled
    pub deterministic_ordering: bool,
    /// Whether comments are preserved
    pub preserve_comments: bool,
    /// Whether processing instructions are preserved
    pub preserve_processing_instructions: bool,
    /// Namespace handling strategy
    pub namespace_handling: NamespaceHandling,
}

/// Namespace handling strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamespaceHandling {
    /// Preserve original namespace prefixes
    Preserve,
    /// Minimize namespace declarations
    Minimize,
    /// Normalize namespace prefixes
    Normalize,
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

/// A one-line description of the fidelity features as they are in this version (before
/// 0.5.1 it claimed round-trip and extension support).
pub fn fidelity_engine_info() -> String {
    format!(
        "Fidelity features v{} • DB-C14N/{} output • Round-trip: verify_round_trip compares \
         the fields the build request carries (not lossless) • Extensions: not rebuilt",
        env!("CARGO_PKG_VERSION"),
        DB_C14N_VERSION
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

/// `xml` canonicalized with `algorithm`: DB-C14N/1.0 or nothing; the other algorithms are
/// not implemented in 0.5.x.
pub(crate) fn canonicalize_with(
    xml: &str,
    algorithm: &CanonicalizationAlgorithm,
) -> Result<String, error::BuildError> {
    match algorithm {
        CanonicalizationAlgorithm::None => Ok(xml.to_string()),
        CanonicalizationAlgorithm::DbC14N => {
            DB_C14N::new(determinism::DeterminismConfig::default()).canonicalize(xml)
        }
        CanonicalizationAlgorithm::C14N => Err(round_trip::not_implemented(
            "C14N canonicalization",
            "only DB-C14N/1.0 (CanonicalizationAlgorithm::DbC14N) is implemented",
        )),
        CanonicalizationAlgorithm::C14N11 => Err(round_trip::not_implemented(
            "C14N 1.1 canonicalization",
            "only DB-C14N/1.0 (CanonicalizationAlgorithm::DbC14N) is implemented",
        )),
        CanonicalizationAlgorithm::Custom(_) => Err(round_trip::not_implemented(
            "custom canonicalization",
            "only DB-C14N/1.0 (CanonicalizationAlgorithm::DbC14N) is implemented",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = Builder::new();
        assert!(!builder.is_preset_locked());
    }

    #[test]
    fn test_preset_application() {
        let mut builder = Builder::new();
        assert!(builder.apply_preset("audio_album", false).is_ok());
        assert!(!builder.is_preset_locked());

        assert!(builder.apply_preset("audio_album", true).is_ok());
        assert!(builder.is_preset_locked());
    }

    #[test]
    fn test_unknown_preset() {
        let mut builder = Builder::new();
        assert!(builder.apply_preset("unknown_preset", false).is_err());
    }

    #[test]
    fn test_version_info() {
        let info = version_info();
        assert!(info.contains("DDEX Builder"));
        assert!(info.contains("DB-C14N/1.0"));
    }
}
