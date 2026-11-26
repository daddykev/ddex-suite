//! Determinism guarantees and validation for DDEX Builder
//!
//! This module defines the builder's determinism guarantees (ordering, formatting, ...)
//! and a validator that checks a build request against them. They do not yet make the
//! whole message byte-for-byte reproducible: generated IDs and timestamps can differ
//! between builds (see the [`determinism`](crate::determinism) module; planned for 0.6.0).

use crate::determinism::{DeterminismConfig, DeterminismVerifier};
use crate::error::BuildError;
use serde::{Deserialize, Serialize};

/// Core determinism guarantees provided by DDEX Builder
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeterminismGuarantee {
    /// Always use IndexMap instead of HashMap for deterministic iteration order
    DeterministicCollections,
    /// Sort all collections consistently using stable algorithms
    StableSorting,
    /// Use fixed timestamps or make them configurable inputs
    FixedTimestamps,
    /// Normalize all strings using Unicode NFC normalization
    UnicodeNormalization,
    /// Use stable hash algorithm (SHA-256) for content hashing
    StableHashing,
    /// Element ordering follows canonical XSD order
    CanonicalOrdering,
    /// Namespace prefixes are locked and consistent
    LockedNamespacePrefixes,
    /// Output format is DB-C14N/1.0 canonicalized
    CanonicalXmlOutput,
    /// Thread-safe with identical output across parallel builds
    ThreadSafety,
    /// Platform-independent output (OS, architecture, locale)
    PlatformIndependence,
    /// Memory usage patterns don't affect output
    MemoryIndependence,
}

impl DeterminismGuarantee {
    /// Get human-readable description of the guarantee
    pub fn description(&self) -> &'static str {
        match self {
            Self::DeterministicCollections => {
                "All internal data structures use IndexMap instead of HashMap to ensure deterministic iteration order"
            }
            Self::StableSorting => {
                "All collections are sorted using stable algorithms with consistent comparison functions"
            }
            Self::FixedTimestamps => {
                "Timestamps are either fixed at build time or explicitly provided as inputs"
            }
            Self::UnicodeNormalization => {
                "All string content is normalized using Unicode NFC form"
            }
            Self::StableHashing => {
                "SHA-256 is used for all content hashing to ensure stable, reproducible hashes"
            }
            Self::CanonicalOrdering => {
                "XML elements are ordered according to canonical XSD sequence definitions"
            }
            Self::LockedNamespacePrefixes => {
                "Namespace prefixes are predefined and locked to prevent variation"
            }
            Self::CanonicalXmlOutput => {
                "XML output follows DB-C14N/1.0 canonicalization specification"
            }
            Self::ThreadSafety => {
                "Multiple parallel builds of the same content produce identical output"
            }
            Self::PlatformIndependence => {
                "Output is identical across different operating systems, architectures, and locales"
            }
            Self::MemoryIndependence => {
                "Memory usage patterns and garbage collection do not affect output content"
            }
        }
    }

    /// Get the validation method for this guarantee
    pub fn validator(&self) -> GuaranteeValidator {
        match self {
            Self::DeterministicCollections => GuaranteeValidator::CodeAnalysis,
            Self::StableSorting => GuaranteeValidator::CodeAnalysis,
            Self::FixedTimestamps => GuaranteeValidator::RuntimeVerification,
            Self::UnicodeNormalization => GuaranteeValidator::RuntimeVerification,
            Self::StableHashing => GuaranteeValidator::RuntimeVerification,
            Self::CanonicalOrdering => GuaranteeValidator::RuntimeVerification,
            Self::LockedNamespacePrefixes => GuaranteeValidator::RuntimeVerification,
            Self::CanonicalXmlOutput => GuaranteeValidator::RuntimeVerification,
            Self::ThreadSafety => GuaranteeValidator::ConcurrencyTest,
            Self::PlatformIndependence => GuaranteeValidator::CrossPlatformTest,
            Self::MemoryIndependence => GuaranteeValidator::StressTest,
        }
    }

    /// Get the priority level of this guarantee
    pub fn priority(&self) -> GuaranteePriority {
        match self {
            Self::DeterministicCollections => GuaranteePriority::Critical,
            Self::StableSorting => GuaranteePriority::Critical,
            Self::FixedTimestamps => GuaranteePriority::High,
            Self::UnicodeNormalization => GuaranteePriority::High,
            Self::StableHashing => GuaranteePriority::High,
            Self::CanonicalOrdering => GuaranteePriority::Critical,
            Self::LockedNamespacePrefixes => GuaranteePriority::High,
            Self::CanonicalXmlOutput => GuaranteePriority::Critical,
            Self::ThreadSafety => GuaranteePriority::High,
            Self::PlatformIndependence => GuaranteePriority::Medium,
            Self::MemoryIndependence => GuaranteePriority::Medium,
        }
    }
}

/// Validation method for guarantees
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuaranteeValidator {
    /// Static code analysis (clippy rules, etc.)
    CodeAnalysis,
    /// Runtime verification during build
    RuntimeVerification,
    /// Concurrency/threading tests
    ConcurrencyTest,
    /// Cross-platform tests
    CrossPlatformTest,
    /// Stress tests with varying conditions
    StressTest,
}

/// Guarantee priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GuaranteePriority {
    /// Critical - must pass
    Critical,
    /// High priority
    High,
    /// Medium priority
    Medium,
    /// Low priority
    Low,
}

/// Result of guarantee validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuaranteeValidationResult {
    /// The guarantee being validated
    pub guarantee: DeterminismGuarantee,
    /// Whether the guarantee passed
    pub passed: bool,
    /// Details about the validation
    pub details: String,
    /// Supporting evidence if available
    pub evidence: Option<String>,
    /// Timestamp when guarantee was made
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Comprehensive guarantee validator
pub struct DeterminismGuaranteeValidator {
    config: DeterminismConfig,
}

impl DeterminismGuaranteeValidator {
    /// Create a new guarantee validator
    pub fn new(config: DeterminismConfig) -> Self {
        Self { config }
    }

    /// Validate all guarantees for a build request
    pub fn validate_all_guarantees(
        &self,
        request: &crate::builder::BuildRequest,
    ) -> Result<Vec<GuaranteeValidationResult>, BuildError> {
        let all_guarantees = vec![
            DeterminismGuarantee::DeterministicCollections,
            DeterminismGuarantee::StableSorting,
            DeterminismGuarantee::FixedTimestamps,
            DeterminismGuarantee::UnicodeNormalization,
            DeterminismGuarantee::StableHashing,
            DeterminismGuarantee::CanonicalOrdering,
            DeterminismGuarantee::LockedNamespacePrefixes,
            DeterminismGuarantee::CanonicalXmlOutput,
            DeterminismGuarantee::ThreadSafety,
            DeterminismGuarantee::PlatformIndependence,
            DeterminismGuarantee::MemoryIndependence,
        ];

        let mut results = Vec::new();
        for guarantee in all_guarantees {
            let result = self.validate_guarantee(&guarantee, request)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Validate a specific guarantee
    pub fn validate_guarantee(
        &self,
        guarantee: &DeterminismGuarantee,
        request: &crate::builder::BuildRequest,
    ) -> Result<GuaranteeValidationResult, BuildError> {
        let timestamp = chrono::Utc::now();

        match guarantee.validator() {
            GuaranteeValidator::CodeAnalysis => {
                self.validate_code_analysis_guarantee(guarantee, timestamp)
            }
            GuaranteeValidator::RuntimeVerification => {
                self.validate_runtime_guarantee(guarantee, request, timestamp)
            }
            GuaranteeValidator::ConcurrencyTest => {
                self.validate_concurrency_guarantee(guarantee, request, timestamp)
            }
            GuaranteeValidator::CrossPlatformTest => {
                self.validate_cross_platform_guarantee(guarantee, request, timestamp)
            }
            GuaranteeValidator::StressTest => {
                self.validate_stress_test_guarantee(guarantee, request, timestamp)
            }
        }
    }

    fn validate_code_analysis_guarantee(
        &self,
        guarantee: &DeterminismGuarantee,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<GuaranteeValidationResult, BuildError> {
        let (passed, details, evidence) = match guarantee {
            // Not verified: there is no code analysis at runtime. Before 0.5.1 both passed
            // with canned evidence.
            DeterminismGuarantee::DeterministicCollections => (
                false,
                "Not verified in 0.5.x, and not true yet: ddex-builder still uses HashMap and \
                 HashSet (clippy.toml disallows them, but that lint is allowed to fail in CI \
                 until 0.6.0)"
                    .to_string(),
                None,
            ),
            DeterminismGuarantee::StableSorting => (
                false,
                "Not verified in 0.5.x (no code analysis at runtime)".to_string(),
                None,
            ),
            _ => {
                return Err(BuildError::DeterminismGuaranteeViolated {
                    guarantee: format!("{:?}", guarantee),
                    details: "Code analysis validation not supported for this guarantee type"
                        .to_string(),
                });
            }
        };

        Ok(GuaranteeValidationResult {
            guarantee: guarantee.clone(),
            passed,
            details,
            evidence,
            timestamp,
        })
    }

    fn validate_runtime_guarantee(
        &self,
        guarantee: &DeterminismGuarantee,
        request: &crate::builder::BuildRequest,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<GuaranteeValidationResult, BuildError> {
        let verifier = DeterminismVerifier::new(self.config.clone());
        let result = verifier.verify(request, 3)?;

        // What is checked is that repeated builds are byte-identical; the specific property
        // named by the guarantee is not checked separately.
        let (passed, details, evidence) = if result.is_deterministic {
            (
                true,
                format!(
                    "{} builds of the request produced byte-identical XML; {:?} itself is not \
                     checked separately",
                    result.iterations, guarantee
                ),
                Some(format!("SHA-256: {:?}", result.hashes)),
            )
        } else {
            let details = format!(
                "Determinism verification failed: {} differences found",
                result.differences.len()
            );
            let evidence = if let Some(diff) = result.differences.first() {
                Some(format!(
                    "First difference at byte {}: SHA-256 {} vs {}",
                    diff.first_difference_byte.unwrap_or(0),
                    diff.hash_difference.sha256_1,
                    diff.hash_difference.sha256_2
                ))
            } else {
                None
            };
            (false, details, evidence)
        };

        Ok(GuaranteeValidationResult {
            guarantee: guarantee.clone(),
            passed,
            details,
            evidence,
            timestamp,
        })
    }

    fn validate_concurrency_guarantee(
        &self,
        guarantee: &DeterminismGuarantee,
        request: &crate::builder::BuildRequest,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<GuaranteeValidationResult, BuildError> {
        use std::sync::Arc;
        use std::thread;

        if !matches!(guarantee, DeterminismGuarantee::ThreadSafety) {
            return Err(BuildError::DeterminismGuaranteeViolated {
                guarantee: format!("{:?}", guarantee),
                details: "Concurrency validation only supports ThreadSafety guarantee".to_string(),
            });
        }

        let verifier = Arc::new(DeterminismVerifier::new(self.config.clone()));
        let mut handles = vec![];
        let results = Arc::new(std::sync::Mutex::new(vec![]));

        // Run builds in parallel threads
        for _ in 0..4 {
            let verifier_clone = Arc::clone(&verifier);
            let request_clone = request.clone();
            let results_clone = Arc::clone(&results);

            let handle = thread::spawn(move || {
                let result = verifier_clone.verify(&request_clone, 2);
                results_clone.lock().unwrap().push(result);
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle
                .join()
                .map_err(|_| BuildError::Other("Thread join failed".to_string()))?;
        }

        let thread_results = results.lock().unwrap();
        let all_deterministic = thread_results
            .iter()
            .all(|r| r.as_ref().map_or(false, |res| res.is_deterministic));

        if all_deterministic && thread_results.len() == 4 {
            // Verify all threads produced identical outputs
            let first_hash = &thread_results[0].as_ref().unwrap().hashes[0];
            let all_identical = thread_results
                .iter()
                .skip(1)
                .all(|r| r.as_ref().map_or(false, |res| &res.hashes[0] == first_hash));

            if all_identical {
                Ok(GuaranteeValidationResult {
                    guarantee: guarantee.clone(),
                    passed: true,
                    details: "All parallel builds produced identical output".to_string(),
                    evidence: Some(format!("4 threads all produced hash: {}", first_hash)),
                    timestamp,
                })
            } else {
                Ok(GuaranteeValidationResult {
                    guarantee: guarantee.clone(),
                    passed: false,
                    details: "Parallel builds produced different outputs".to_string(),
                    evidence: Some("Hash mismatch between threads".to_string()),
                    timestamp,
                })
            }
        } else {
            Ok(GuaranteeValidationResult {
                guarantee: guarantee.clone(),
                passed: false,
                details: format!(
                    "Thread safety test failed: {}/{} threads succeeded",
                    thread_results.iter().filter(|r| r.is_ok()).count(),
                    thread_results.len()
                ),
                evidence: None,
                timestamp,
            })
        }
    }

    fn validate_cross_platform_guarantee(
        &self,
        guarantee: &DeterminismGuarantee,
        request: &crate::builder::BuildRequest,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<GuaranteeValidationResult, BuildError> {
        if !matches!(guarantee, DeterminismGuarantee::PlatformIndependence) {
            return Err(BuildError::DeterminismGuaranteeViolated {
                guarantee: format!("{:?}", guarantee),
                details: "Cross-platform validation only supports PlatformIndependence guarantee"
                    .to_string(),
            });
        }

        // Test with different locale settings
        let original_locale = std::env::var("LC_ALL").unwrap_or_default();
        let verifier = DeterminismVerifier::new(self.config.clone());
        let mut results = vec![];

        let test_locales = ["C", "en_US.UTF-8"];
        for locale in &test_locales {
            std::env::set_var("LC_ALL", locale);
            let result = verifier.verify(request, 2)?;
            results.push(result);
        }

        // Restore original locale
        if original_locale.is_empty() {
            std::env::remove_var("LC_ALL");
        } else {
            std::env::set_var("LC_ALL", original_locale);
        }

        let all_deterministic = results.iter().all(|r| r.is_deterministic);
        if all_deterministic && results.len() > 1 {
            let first_hash = &results[0].hashes[0];
            let all_identical = results.iter().skip(1).all(|r| &r.hashes[0] == first_hash);

            Ok(GuaranteeValidationResult {
                guarantee: guarantee.clone(),
                passed: all_identical,
                details: if all_identical {
                    "Output is identical across different locales".to_string()
                } else {
                    "Output varies across different locales".to_string()
                },
                evidence: Some(format!("Tested locales: {:?}", test_locales)),
                timestamp,
            })
        } else {
            Ok(GuaranteeValidationResult {
                guarantee: guarantee.clone(),
                passed: false,
                details: "Cross-platform test failed".to_string(),
                evidence: None,
                timestamp,
            })
        }
    }

    fn validate_stress_test_guarantee(
        &self,
        guarantee: &DeterminismGuarantee,
        request: &crate::builder::BuildRequest,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<GuaranteeValidationResult, BuildError> {
        if !matches!(guarantee, DeterminismGuarantee::MemoryIndependence) {
            return Err(BuildError::DeterminismGuaranteeViolated {
                guarantee: format!("{:?}", guarantee),
                details: "Stress test validation only supports MemoryIndependence guarantee"
                    .to_string(),
            });
        }

        let verifier = DeterminismVerifier::new(self.config.clone());

        // Test under memory pressure
        let _memory_pressure: Vec<Vec<u8>> = (0..50)
            .map(|_| vec![0u8; 1024 * 1024]) // 50MB total
            .collect();

        let stressed_result = verifier.verify(request, 3)?;

        // Test without memory pressure (after pressure is dropped)
        drop(_memory_pressure);
        std::thread::sleep(std::time::Duration::from_millis(100)); // Allow GC

        let normal_result = verifier.verify(request, 3)?;

        let both_deterministic = stressed_result.is_deterministic && normal_result.is_deterministic;
        let outputs_identical =
            both_deterministic && stressed_result.hashes[0] == normal_result.hashes[0];

        Ok(GuaranteeValidationResult {
            guarantee: guarantee.clone(),
            passed: outputs_identical,
            details: if outputs_identical {
                "Output is identical under memory pressure and normal conditions".to_string()
            } else {
                "Output differs between memory pressure and normal conditions".to_string()
            },
            evidence: Some(format!(
                "Stressed hash: {}, Normal hash: {}",
                stressed_result.hashes.get(0).unwrap_or(&"N/A".to_string()),
                normal_result.hashes.get(0).unwrap_or(&"N/A".to_string())
            )),
            timestamp,
        })
    }
}

/// Not implemented in 0.5.x: returns an error (before 0.5.1 it returned `Ok(())` without
/// checking; ddex-builder still uses HashMap/HashSet, see clippy.toml).
pub fn validate_no_hashmap_usage() -> Result<(), BuildError> {
    Err(crate::round_trip::not_implemented(
        "validate_no_hashmap_usage",
        "there is no code analysis at runtime; the clippy determinism lint is the check",
    ))
}

/// Not implemented in 0.5.x: returns an error (before 0.5.1 it returned `Ok(())` without
/// checking).
pub fn validate_deterministic_sorting() -> Result<(), BuildError> {
    Err(crate::round_trip::not_implemented(
        "validate_deterministic_sorting",
        "there is no code analysis at runtime",
    ))
}

/// Generate a determinism guarantee report
pub fn generate_guarantee_report(
    request: &crate::builder::BuildRequest,
    config: &DeterminismConfig,
) -> Result<GuaranteeReport, BuildError> {
    let validator = DeterminismGuaranteeValidator::new(config.clone());
    let results = validator.validate_all_guarantees(request)?;

    let passed_count = results.iter().filter(|r| r.passed).count();
    let total_count = results.len();
    let success_rate = if total_count > 0 {
        (passed_count as f64 / total_count as f64) * 100.0
    } else {
        0.0
    };

    Ok(GuaranteeReport {
        timestamp: chrono::Utc::now(),
        total_guarantees: total_count,
        passed_guarantees: passed_count,
        success_rate,
        results,
        overall_pass: passed_count == total_count,
    })
}

/// Complete report of guarantee validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuaranteeReport {
    /// Timestamp of the validation report
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Total number of guarantees checked
    pub total_guarantees: usize,
    /// Number of guarantees that passed
    pub passed_guarantees: usize,
    /// Success rate as a percentage (0.0-100.0)
    pub success_rate: f64,
    /// Detailed results for each guarantee
    pub results: Vec<GuaranteeValidationResult>,
    /// Whether all guarantees passed
    pub overall_pass: bool,
}

impl GuaranteeReport {
    /// Get failed guarantees only
    pub fn failed_guarantees(&self) -> Vec<&GuaranteeValidationResult> {
        self.results.iter().filter(|r| !r.passed).collect()
    }

    /// Get critical failures only
    pub fn critical_failures(&self) -> Vec<&GuaranteeValidationResult> {
        self.results
            .iter()
            .filter(|r| !r.passed && r.guarantee.priority() == GuaranteePriority::Critical)
            .collect()
    }

    /// Generate human-readable summary
    pub fn summary(&self) -> String {
        if self.overall_pass {
            format!(
                "✓ All {} determinism guarantees passed (100%)",
                self.total_guarantees
            )
        } else {
            let failed = self.total_guarantees - self.passed_guarantees;
            let critical_failed = self.critical_failures().len();

            if critical_failed > 0 {
                format!(
                    "✗ {}/{} guarantees failed ({:.1}%) - {} CRITICAL failures",
                    failed,
                    self.total_guarantees,
                    100.0 - self.success_rate,
                    critical_failed
                )
            } else {
                format!(
                    "⚠ {}/{} guarantees failed ({:.1}%) - no critical failures",
                    failed,
                    self.total_guarantees,
                    100.0 - self.success_rate
                )
            }
        }
    }
}
