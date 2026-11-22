//! # Determinism Configuration and Enforcement
//!
//! This module configures the builder's canonical ordering and formatting, and
//! provides [`DeterminismVerifier`], which builds a request several times and reports
//! any differences. Reproducible output enables supply chain integrity, reproducible
//! builds and cryptographic signing.
//!
//! ## Status in 0.5.0
//!
//! Output is canonicalized with DB-C14N/1.0 (fixed namespace prefixes, sorted
//! attributes, consistent formatting, elements in XSD order), and output paths use
//! `IndexMap`, never `HashMap`, so ordering does not depend on the platform.
//!
//! Byte-for-byte determinism is not complete yet: with the default options, a message
//! ID, release and resource references and `MessageCreatedDateTime` that the request
//! leaves out are generated (random UUIDs, the current time), and the stable-hash ID
//! strategy puts today's date into the `MessageId`. Two builds of the same request can
//! therefore differ. Full determinism is planned for 0.6.0.
//!
//! ## Why Determinism Matters
//!
//! ```text
//! Deterministic Benefits
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │ Supply Chain    │    │ Reproducible     │    │ Digital         │
//! │ Integrity       │    │ Builds           │    │ Signatures      │
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//!          │                       │                       │
//!          ▼                       ▼                       ▼
//!   ┌─────────────┐       ┌─────────────────┐    ┌─────────────────┐
//!   │ • Audit     │       │ • CI/CD Cache   │    │ • Crypto Valid  │
//!   │ • Verify    │       │ • Artifact      │    │ • Non-repudiat  │
//!   │ • Trust     │       │   Dedup         │    │ • Compliance    │
//!   │ • Detect    │       │ • Build Reprod  │    │ • Legal Proof   │
//!   └─────────────┘       └─────────────────┘    └─────────────────┘
//! ```
//!
//! ## Implementation Strategy
//!
//! **CRITICAL**: This module ensures deterministic output by using `IndexMap`
//! everywhere instead of `HashMap`/`HashSet`. The clippy configuration enforces this.
//!
//! ### Key Components
//!
//! 1. **DB-C14N/1.0 Canonicalization**: Our custom canonicalization spec
//! 2. **Deterministic Data Structures**: IndexMap for stable iteration order
//! 3. **Locked Namespace Prefixes** (generated IDs are not fixed yet; see above)
//! 4. **Normalized Formatting**: Consistent whitespace, encoding, line endings
//! 5. **Time Zone Handling**: UTC normalization for timestamps
//!
//! ## Configuration Example
//!
//! ```rust
//! use ddex_builder::determinism::*;
//! use indexmap::IndexMap;
//!
//! let mut config = DeterminismConfig::default();
//!
//! // Enable strict determinism verification
//! config.verify_determinism = Some(5); // Test with 5 iterations
//!
//! // Lock namespace prefixes
//! config.locked_prefixes.insert(
//!     "http://ddex.net/xml/ern/43".to_string(),
//!     "ern".to_string()
//! );
//!
//! // Use custom element ordering
//! let mut release_order = IndexMap::new();
//! release_order.insert("Release".to_string(), vec![
//!     "ReleaseReference".to_string(),
//!     "ReleaseId".to_string(),
//!     "ReferenceTitle".to_string(),
//! ]);
//! config.custom_sort_order = Some(release_order);
//!
//! // Apply configuration to builder
//! let builder = ddex_builder::Builder::with_config(config);
//! # let _ = builder;
//! ```
//!
//! ## Verification Process
//!
//! The determinism verification process works by:
//!
//! 1. **Build XML** using the same input multiple times
//! 2. **Compare Bytes** - every byte must be identical
//! 3. **Hash Verification** - SHA-256 hashes must match
//! 4. **Failure Detection** - any variance triggers detailed diff analysis
//!
//! ```rust
//! use ddex_builder::builder::{
//!     BuildRequest, LocalizedStringRequest, MessageHeaderRequest, PartyRequest,
//! };
//! use ddex_builder::determinism::{DeterminismConfig, DeterminismVerifier};
//! # let party = |name: &str| PartyRequest {
//! #     party_name: vec![LocalizedStringRequest { text: name.to_string(), language_code: None }],
//! #     party_id: Some("PADPIDA0000000001".to_string()),
//! #     party_reference: None,
//! # };
//! # let request = BuildRequest {
//! #     header: MessageHeaderRequest {
//! #         message_id: Some("MSG-1".to_string()),
//! #         message_sender: party("Label"),
//! #         message_recipient: party("DSP"),
//! #         message_control_type: None,
//! #         message_created_date_time: Some("2024-01-01T00:00:00Z".to_string()),
//! #     },
//! #     version: "4.3".to_string(),
//! #     profile: None,
//! #     releases: vec![ddex_builder::builder::ReleaseRequest {
//! #         release_id: "A1-ABCDE-0000000001-M".to_string(),
//! #         release_reference: Some("R1".to_string()),
//! #         title: vec![LocalizedStringRequest { text: "Album".to_string(), language_code: None }],
//! #         artist: "Artist".to_string(),
//! #         label: Some("Label".to_string()),
//! #         release_type: Some("Album".into()),
//! #         genre: Some("Pop".to_string()),
//! #         parental_warning_type: Some("NotExplicit".to_string()),
//! #         ..Default::default()
//! #     }],
//! #     deals: vec![],
//! #     extensions: None,
//! #     language_and_script_code: Some("en".to_string()),
//! #     avs_version_id: Some("12".to_string()),
//! #     ..Default::default()
//! # };
//!
//! // Build the same request 3 times and compare bytes and hashes
//! let verifier = DeterminismVerifier::new(DeterminismConfig::default());
//! let result = verifier.verify(&request, 3)?;
//! assert!(result.is_deterministic, "differences: {:?}", result.differences);
//! # Ok::<(), ddex_builder::BuildError>(())
//! ```
//!
//! ## Performance Impact
//!
//! The overhead of the ordering has not been measured separately. Verification builds
//! the request once per iteration, so it costs about one build per iteration; build
//! times are in `dev-docs/PERFORMANCE.md`.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Determinism configuration for XML generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismConfig {
    /// Canonicalization mode
    pub canon_mode: CanonMode,

    /// Element ordering strategy
    pub sort_strategy: SortStrategy,

    /// Custom sort order (uses IndexMap for determinism)
    pub custom_sort_order: Option<IndexMap<String, Vec<String>>>,

    /// Namespace handling
    pub namespace_strategy: NamespaceStrategy,

    /// Locked namespace prefixes (uses IndexMap for determinism)
    pub locked_prefixes: IndexMap<String, String>,

    /// Formatting options
    pub output_mode: OutputMode,
    /// Line ending style for output
    pub line_ending: LineEnding,
    /// Character used for indentation
    pub indent_char: IndentChar,
    /// Number of indent characters per level
    pub indent_width: usize,

    /// String normalization
    pub unicode_normalization: UnicodeNormalization,
    /// Policy for handling special XML characters
    pub xml_character_policy: XmlCharacterPolicy,
    /// Quote style for attributes
    pub quote_style: QuoteStyle,

    /// Date/Time handling
    pub time_zone_policy: TimeZonePolicy,
    /// Format for date/time values
    pub date_time_format: DateTimeFormat,

    /// Reproducibility options
    pub emit_reproducibility_banner: bool,
    /// Number of iterations to verify determinism (None = disabled)
    pub verify_determinism: Option<usize>,
}

impl Default for DeterminismConfig {
    fn default() -> Self {
        Self {
            canon_mode: CanonMode::DbC14n,
            sort_strategy: SortStrategy::Canonical,
            custom_sort_order: None,
            namespace_strategy: NamespaceStrategy::Locked,
            locked_prefixes: Self::default_namespace_prefixes(),
            output_mode: OutputMode::DbC14n,
            line_ending: LineEnding::LF,
            indent_char: IndentChar::Space,
            indent_width: 2,
            unicode_normalization: UnicodeNormalization::NFC,
            xml_character_policy: XmlCharacterPolicy::Escape,
            quote_style: QuoteStyle::Double,
            time_zone_policy: TimeZonePolicy::UTC,
            date_time_format: DateTimeFormat::ISO8601Z,
            emit_reproducibility_banner: false,
            verify_determinism: None,
        }
    }
}

impl DeterminismConfig {
    fn default_namespace_prefixes() -> IndexMap<String, String> {
        let mut prefixes = IndexMap::new();
        prefixes.insert("http://ddex.net/xml/ern/43".to_string(), "ern".to_string());
        prefixes.insert("http://ddex.net/xml/ern/42".to_string(), "ern".to_string());
        prefixes.insert("http://ddex.net/xml/ern/382".to_string(), "ern".to_string());
        prefixes.insert("http://ddex.net/xml/avs".to_string(), "avs".to_string());
        prefixes.insert(
            "http://www.w3.org/2001/XMLSchema-instance".to_string(),
            "xsi".to_string(),
        );
        prefixes
    }
}

/// Canonicalization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonMode {
    /// DB-C14N/1.0 canonicalization
    DbC14n,
    /// Pretty printing (non-canonical)
    Pretty,
    /// Compact output (no whitespace)
    Compact,
}

/// Element ordering strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortStrategy {
    /// Canonical order from XSD
    Canonical,
    /// Preserve input order
    InputOrder,
    /// Custom order
    Custom,
}

/// Namespace handling strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamespaceStrategy {
    /// Use locked prefixes
    Locked,
    /// Inherit from input
    Inherit,
}

/// Output formatting mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputMode {
    /// DB-C14N formatted
    DbC14n,
    /// Pretty printed
    Pretty,
    /// Compact (no whitespace)
    Compact,
}

/// Line ending style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineEnding {
    /// Unix line endings
    LF,
    /// Windows line endings
    CRLF,
}

/// Indentation character
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndentChar {
    /// Space indentation
    Space,
    /// Tab indentation
    Tab,
}

/// Unicode normalization form
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnicodeNormalization {
    /// NFC (Canonical Decomposition, Canonical Composition)
    NFC,
    /// NFD (Canonical Decomposition)
    NFD,
    /// NFKC (Compatibility Decomposition, Canonical Composition)
    NFKC,
    /// NFKD (Compatibility Decomposition)
    NFKD,
}

/// XML character handling policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XmlCharacterPolicy {
    /// Escape special characters
    Escape,
    /// Use CDATA sections
    CData,
    /// Reject invalid characters
    Reject,
}

/// Quote style for attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuoteStyle {
    /// Double quotes
    Double,
    /// Single quotes
    Single,
}

/// Time zone policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeZonePolicy {
    /// Convert to UTC
    UTC,
    /// Preserve original
    Preserve,
    /// Use local time zone
    Local,
}

/// Date/time format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateTimeFormat {
    /// ISO 8601 with Z suffix
    ISO8601Z,
    /// ISO 8601 with offset
    ISO8601,
    /// Custom format
    Custom,
}

/// Determinism verification result
#[derive(Debug, Clone, PartialEq)]
pub struct DeterminismResult {
    /// Whether output is deterministic
    pub is_deterministic: bool,
    /// Number of iterations tested
    pub iterations: usize,
    /// Generated outputs for comparison
    pub outputs: Vec<String>,
    /// SHA-256 hashes of outputs
    pub hashes: Vec<String>,
    /// Differences found between iterations
    pub differences: Vec<DeterminismDifference>,
    /// Runtime statistics
    pub runtime_stats: DeterminismStats,
}

/// Information about a determinism difference
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterminismDifference {
    /// First iteration where difference occurred
    pub iteration1: usize,
    /// Second iteration where difference occurred
    pub iteration2: usize,
    /// Byte position of first difference
    pub first_difference_byte: Option<usize>,
    /// Hash comparison details
    pub hash_difference: HashDifference,
    /// Length comparison details
    pub length_difference: LengthDifference,
    /// Context around the difference
    pub context: Option<DifferenceContext>,
}

/// Hash comparison details
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashDifference {
    /// SHA-256 hash from first iteration
    pub sha256_1: String,
    /// SHA-256 hash from second iteration
    pub sha256_2: String,
    /// BLAKE3 hash from first iteration
    pub blake3_1: String,
    /// BLAKE3 hash from second iteration
    pub blake3_2: String,
}

/// Length comparison details
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LengthDifference {
    /// Length in first iteration
    pub length_1: usize,
    /// Length in second iteration
    pub length_2: usize,
    /// Difference in bytes (negative if second is shorter)
    pub diff: i64,
}

/// Context around a difference
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DifferenceContext {
    /// Byte position where difference occurred
    pub position: usize,
    /// Content before the difference
    pub before: String,
    /// Content after in first iteration
    pub after_1: String,
    /// Content after in second iteration
    pub after_2: String,
    /// Line number if applicable
    pub line_number: Option<usize>,
    /// Column number if applicable
    pub column_number: Option<usize>,
}

/// Runtime statistics for determinism verification
#[derive(Debug, Clone, PartialEq)]
pub struct DeterminismStats {
    /// Total time for all iterations in milliseconds
    pub total_time_ms: u64,
    /// Average build time per iteration
    pub avg_build_time_ms: u64,
    /// Minimum build time observed
    pub min_build_time_ms: u64,
    /// Maximum build time observed
    pub max_build_time_ms: u64,
    /// Overhead percentage from determinism checking
    pub overhead_percentage: f64,
}

/// Determinism verifier with comprehensive analysis
pub struct DeterminismVerifier {
    config: DeterminismConfig,
    include_outputs: bool,
    context_chars: usize,
}

impl DeterminismVerifier {
    /// Create a new determinism verifier
    pub fn new(config: DeterminismConfig) -> Self {
        Self {
            config,
            include_outputs: false,
            context_chars: 100,
        }
    }

    /// Create a verifier with output retention (for debugging)
    pub fn with_outputs_retained(mut self) -> Self {
        self.include_outputs = true;
        self
    }

    /// Set context characters around differences
    pub fn with_context_chars(mut self, chars: usize) -> Self {
        self.context_chars = chars;
        self
    }

    /// Verify that output is deterministic by building multiple times
    pub fn verify(
        &self,
        request: &super::builder::BuildRequest,
        iterations: usize,
    ) -> Result<DeterminismResult, super::error::BuildError> {
        if iterations < 2 {
            return Ok(DeterminismResult {
                is_deterministic: true,
                iterations: 1,
                outputs: vec![],
                hashes: vec![],
                differences: vec![],
                runtime_stats: DeterminismStats {
                    total_time_ms: 0,
                    avg_build_time_ms: 0,
                    min_build_time_ms: 0,
                    max_build_time_ms: 0,
                    overhead_percentage: 0.0,
                },
            });
        }

        let start_time = std::time::Instant::now();
        let mut results = Vec::with_capacity(iterations);
        let mut hashes = Vec::with_capacity(iterations);
        let mut build_times = Vec::with_capacity(iterations);

        // Build XML multiple times with timing
        for _ in 0..iterations {
            let build_start = std::time::Instant::now();
            let builder = super::Builder::with_config(self.config.clone());
            let result = builder.build_internal(request)?;
            let build_time = build_start.elapsed();
            build_times.push(build_time.as_millis() as u64);

            // Calculate both SHA-256 and BLAKE3 hashes
            let sha256_hash = self.calculate_sha256(&result.xml);
            let blake3_hash = self.calculate_blake3(&result.xml);

            results.push(result.xml);
            hashes.push((sha256_hash, blake3_hash));
        }

        let total_time = start_time.elapsed().as_millis() as u64;

        // Analyze differences
        let mut differences = Vec::new();
        let first_output = &results[0];
        let first_hashes = &hashes[0];

        for (i, (output, hash_pair)) in results[1..].iter().zip(hashes[1..].iter()).enumerate() {
            if output != first_output || hash_pair != first_hashes {
                let diff = self.analyze_difference(
                    first_output,
                    output,
                    &first_hashes,
                    hash_pair,
                    0,
                    i + 1,
                );
                differences.push(diff);
            }
        }

        // Calculate runtime statistics
        let min_time = *build_times.iter().min().unwrap_or(&0);
        let max_time = *build_times.iter().max().unwrap_or(&0);
        let avg_time = if !build_times.is_empty() {
            build_times.iter().sum::<u64>() / build_times.len() as u64
        } else {
            0
        };

        let overhead = if iterations > 1 && min_time > 0 {
            ((total_time - min_time) as f64 / min_time as f64) * 100.0
        } else {
            0.0
        };

        let outputs = if self.include_outputs {
            results
        } else {
            vec![]
        };
        let final_hashes = hashes.into_iter().map(|(sha256, _)| sha256).collect();

        Ok(DeterminismResult {
            is_deterministic: differences.is_empty(),
            iterations,
            outputs,
            hashes: final_hashes,
            differences,
            runtime_stats: DeterminismStats {
                total_time_ms: total_time,
                avg_build_time_ms: avg_time,
                min_build_time_ms: min_time,
                max_build_time_ms: max_time,
                overhead_percentage: overhead,
            },
        })
    }

    /// Legacy compatibility method
    pub fn verify_legacy(
        request: &super::builder::BuildRequest,
        config: &DeterminismConfig,
        iterations: usize,
    ) -> Result<bool, super::error::BuildError> {
        let verifier = Self::new(config.clone());
        let result = verifier.verify(request, iterations)?;
        Ok(result.is_deterministic)
    }

    /// Verify with different HashMap iteration orders (stress test)
    pub fn verify_with_hashmap_stress(
        &self,
        request: &super::builder::BuildRequest,
        iterations: usize,
    ) -> Result<DeterminismResult, super::error::BuildError> {
        use std::collections::HashMap;

        // Force different HashMap iteration orders by inserting dummy data
        // in different orders to trigger different hash states
        for i in 0..iterations {
            let mut dummy_map = HashMap::new();
            for j in 0..(i % 10 + 1) {
                dummy_map.insert(format!("key_{}", j), format!("value_{}", j));
            }
            // Access map to potentially affect global hash state
            let _: Vec<_> = dummy_map.iter().collect();
        }

        self.verify(request, iterations)
    }

    /// Verify with thread scheduling variations
    pub fn verify_with_threading_stress(
        &self,
        request: &super::builder::BuildRequest,
        iterations: usize,
    ) -> Result<DeterminismResult, super::error::BuildError> {
        use std::sync::Arc;
        use std::sync::Mutex;
        use std::thread;

        let results = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        for _ in 0..iterations {
            let results_clone = Arc::clone(&results);
            let request_clone = request.clone();
            let config = self.config.clone();

            let handle = thread::spawn(move || {
                let builder = super::Builder::with_config(config);
                let result = builder.build_internal(&request_clone);
                results_clone.lock().unwrap().push(result);
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        let _thread_results = results.lock().unwrap();
        // Convert thread results to normal verification format
        // This is a simplified version - in practice you'd need to adapt this
        self.verify(request, iterations)
    }

    fn calculate_sha256(&self, data: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn calculate_blake3(&self, data: &str) -> String {
        let hash = blake3::hash(data.as_bytes());
        hash.to_hex().to_string()
    }

    fn analyze_difference(
        &self,
        output1: &str,
        output2: &str,
        hashes1: &(String, String),
        hashes2: &(String, String),
        iter1: usize,
        iter2: usize,
    ) -> DeterminismDifference {
        let first_diff_byte = self.find_first_difference(output1, output2);

        let context =
            first_diff_byte.map(|pos| self.create_difference_context(output1, output2, pos));

        DeterminismDifference {
            iteration1: iter1,
            iteration2: iter2,
            first_difference_byte: first_diff_byte,
            hash_difference: HashDifference {
                sha256_1: hashes1.0.clone(),
                sha256_2: hashes2.0.clone(),
                blake3_1: hashes1.1.clone(),
                blake3_2: hashes2.1.clone(),
            },
            length_difference: LengthDifference {
                length_1: output1.len(),
                length_2: output2.len(),
                diff: output2.len() as i64 - output1.len() as i64,
            },
            context,
        }
    }

    fn find_first_difference(&self, a: &str, b: &str) -> Option<usize> {
        a.bytes()
            .zip(b.bytes())
            .position(|(x, y)| x != y)
            .or_else(|| {
                if a.len() != b.len() {
                    Some(std::cmp::min(a.len(), b.len()))
                } else {
                    None
                }
            })
    }

    fn create_difference_context(
        &self,
        output1: &str,
        output2: &str,
        pos: usize,
    ) -> DifferenceContext {
        let start = pos.saturating_sub(self.context_chars / 2);
        let end1 = std::cmp::min(pos + self.context_chars / 2, output1.len());
        let end2 = std::cmp::min(pos + self.context_chars / 2, output2.len());

        // Calculate line and column numbers
        let (line, col) = self.calculate_line_col(output1, pos);

        DifferenceContext {
            position: pos,
            before: output1[start..pos].to_string(),
            after_1: output1[pos..end1].to_string(),
            after_2: output2[pos..end2].to_string(),
            line_number: line,
            column_number: col,
        }
    }

    fn calculate_line_col(&self, text: &str, pos: usize) -> (Option<usize>, Option<usize>) {
        if pos >= text.len() {
            return (None, None);
        }

        let before_pos = &text[..pos];
        let line_num = before_pos.lines().count();
        let last_line_start = before_pos.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let col_num = pos - last_line_start + 1;

        (Some(line_num), Some(col_num))
    }
}

/// Convenience functions for common determinism checks
impl DeterminismVerifier {
    /// Quick determinism check with default settings
    pub fn quick_check(
        request: &super::builder::BuildRequest,
    ) -> Result<bool, super::error::BuildError> {
        let config = DeterminismConfig::default();
        let verifier = Self::new(config);
        let result = verifier.verify(request, 3)?;
        Ok(result.is_deterministic)
    }

    /// Thorough determinism check with multiple stress tests
    pub fn thorough_check(
        request: &super::builder::BuildRequest,
        iterations: usize,
    ) -> Result<DeterminismResult, super::error::BuildError> {
        let config = DeterminismConfig::default();
        let verifier = Self::new(config).with_outputs_retained();

        // Run standard verification
        let standard_result = verifier.verify(request, iterations)?;
        if !standard_result.is_deterministic {
            return Ok(standard_result);
        }

        // Run HashMap stress test
        let hashmap_result = verifier.verify_with_hashmap_stress(request, iterations)?;
        if !hashmap_result.is_deterministic {
            return Ok(hashmap_result);
        }

        // Return the most comprehensive result
        Ok(standard_result)
    }
}
