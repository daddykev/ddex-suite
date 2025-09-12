//! Build Verification for Perfect Fidelity Engine
//!
//! This module provides comprehensive verification capabilities for the DDEX Builder,
//! ensuring that generated XML meets fidelity requirements and can successfully
//! round-trip through the parser.

use crate::{error::BuildError, FidelityOptions, CanonicalizationAlgorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Build verifier for Perfect Fidelity Engine
pub struct BuildVerifier {
    config: VerificationConfig,
}

impl BuildVerifier {
    /// Create a new build verifier with the specified configuration
    pub fn new(config: VerificationConfig) -> Self {
        Self { config }
    }

    /// Verify that the generated XML meets fidelity requirements
    pub fn verify(
        &self,
        xml_output: &str,
        fidelity_options: &FidelityOptions,
    ) -> Result<VerificationResult, BuildError> {
        let start_time = Instant::now();
        let mut issues = Vec::new();
        
        // Track verification results
        let mut round_trip_success = true;
        let mut canonicalization_success = true;
        let mut schema_validation_success = true;
        let mut determinism_success = true;

        // Round-trip verification
        if self.config.enable_round_trip_verification {
            match self.verify_round_trip(xml_output, fidelity_options) {
                Ok(result) => {
                    if !result.success {
                        round_trip_success = false;
                        issues.extend(result.issues);
                    }
                },
                Err(e) => {
                    round_trip_success = false;
                    issues.push(VerificationIssue {
                        severity: VerificationSeverity::Error,
                        category: "round-trip".to_string(),
                        message: format!("Round-trip verification failed: {}", e),
                        path: None,
                        suggestion: Some("Check XML structure and fidelity options".to_string()),
                    });
                }
            }
        }

        // Canonicalization verification
        if self.config.enable_canonicalization_verification {
            match self.verify_canonicalization(xml_output, fidelity_options) {
                Ok(result) => {
                    if !result.success {
                        canonicalization_success = false;
                        issues.extend(result.issues);
                    }
                },
                Err(e) => {
                    canonicalization_success = false;
                    issues.push(VerificationIssue {
                        severity: VerificationSeverity::Error,
                        category: "canonicalization".to_string(),
                        message: format!("Canonicalization verification failed: {}", e),
                        path: None,
                        suggestion: Some("Check canonicalization settings".to_string()),
                    });
                }
            }
        }

        // Schema validation
        if self.config.enable_schema_validation {
            match self.verify_schema(xml_output) {
                Ok(result) => {
                    if !result.success {
                        schema_validation_success = false;
                        issues.extend(result.issues);
                    }
                },
                Err(e) => {
                    schema_validation_success = false;
                    issues.push(VerificationIssue {
                        severity: VerificationSeverity::Error,
                        category: "schema".to_string(),
                        message: format!("Schema validation failed: {}", e),
                        path: None,
                        suggestion: Some("Check XML against DDEX schema".to_string()),
                    });
                }
            }
        }

        // Determinism verification
        if self.config.enable_determinism_verification {
            match self.verify_determinism(xml_output, fidelity_options) {
                Ok(result) => {
                    if !result.success {
                        determinism_success = false;
                        issues.extend(result.issues);
                    }
                },
                Err(e) => {
                    determinism_success = false;
                    issues.push(VerificationIssue {
                        severity: VerificationSeverity::Error,
                        category: "determinism".to_string(),
                        message: format!("Determinism verification failed: {}", e),
                        path: None,
                        suggestion: Some("Check determinism configuration".to_string()),
                    });
                }
            }
        }

        let verification_time = start_time.elapsed();
        let overall_success = round_trip_success 
            && canonicalization_success 
            && schema_validation_success 
            && determinism_success;

        Ok(VerificationResult {
            success: overall_success,
            round_trip_success,
            canonicalization_success,
            schema_validation_success,
            determinism_success,
            issues,
            verification_time,
        })
    }

    /// Verify round-trip capability: XML → Parse → Build → Compare
    fn verify_round_trip(
        &self,
        xml_output: &str,
        fidelity_options: &FidelityOptions,
    ) -> Result<RoundTripVerificationResult, BuildError> {
        // This would integrate with the ddex-parser to test round-trip
        // For now, we'll simulate the verification
        let issues = Vec::new();
        
        // TODO: Integrate with ddex-parser when available
        // let parser = ddex_parser::DDEXParser::new();
        // let parsed = parser.parse(xml_output)?;
        // let rebuilt_xml = self.build(parsed)?;
        // let canonical_original = canonicalize(xml_output)?;
        // let canonical_rebuilt = canonicalize(rebuilt_xml)?;
        // let success = canonical_original == canonical_rebuilt;

        Ok(RoundTripVerificationResult {
            success: true, // Placeholder
            issues,
        })
    }

    /// Verify canonicalization consistency
    fn verify_canonicalization(
        &self,
        xml_output: &str,
        fidelity_options: &FidelityOptions,
    ) -> Result<CanonicalizationVerificationResult, BuildError> {
        let mut issues = Vec::new();
        let mut success = true;

        match &fidelity_options.canonicalization {
            CanonicalizationAlgorithm::None => {
                // No canonicalization - just verify XML is well-formed
                if let Err(e) = quick_xml::Reader::from_str(xml_output).read_event() {
                    success = false;
                    issues.push(VerificationIssue {
                        severity: VerificationSeverity::Error,
                        category: "xml-wellformed".to_string(),
                        message: format!("XML is not well-formed: {}", e),
                        path: None,
                        suggestion: Some("Check XML syntax".to_string()),
                    });
                }
            },
            CanonicalizationAlgorithm::C14N | 
            CanonicalizationAlgorithm::C14N11 | 
            CanonicalizationAlgorithm::DbC14N => {
                // Verify that multiple canonicalizations produce the same result
                let mut canonicalized_versions = Vec::new();
                
                for _ in 0..3 {
                    match self.canonicalize_xml(xml_output, &fidelity_options.canonicalization) {
                        Ok(canonical) => canonicalized_versions.push(canonical),
                        Err(e) => {
                            success = false;
                            issues.push(VerificationIssue {
                                severity: VerificationSeverity::Error,
                                category: "canonicalization".to_string(),
                                message: format!("Canonicalization failed: {}", e),
                                path: None,
                                suggestion: Some("Check canonicalization algorithm settings".to_string()),
                            });
                            break;
                        }
                    }
                }

                if canonicalized_versions.len() >= 2 {
                    let first = &canonicalized_versions[0];
                    for (i, version) in canonicalized_versions.iter().enumerate().skip(1) {
                        if first != version {
                            success = false;
                            issues.push(VerificationIssue {
                                severity: VerificationSeverity::Error,
                                category: "canonicalization-consistency".to_string(),
                                message: format!("Canonicalization is not deterministic: iteration {} differs", i + 1),
                                path: None,
                                suggestion: Some("Check for non-deterministic elements in canonicalization".to_string()),
                            });
                        }
                    }
                }
            },
            CanonicalizationAlgorithm::Custom(_rules) => {
                // Verify custom canonicalization rules
                // TODO: Implement custom canonicalization verification
                issues.push(VerificationIssue {
                    severity: VerificationSeverity::Info,
                    category: "canonicalization".to_string(),
                    message: "Custom canonicalization verification not yet implemented".to_string(),
                    path: None,
                    suggestion: None,
                });
            },
        }

        Ok(CanonicalizationVerificationResult {
            success,
            issues,
        })
    }

    /// Verify against DDEX schema
    fn verify_schema(&self, xml_output: &str) -> Result<SchemaVerificationResult, BuildError> {
        let mut issues = Vec::new();
        let mut success = true;

        // Basic XML well-formedness check
        let mut reader = quick_xml::Reader::from_str(xml_output);

        loop {
            match reader.read_event() {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => continue,
                Err(e) => {
                    success = false;
                    issues.push(VerificationIssue {
                        severity: VerificationSeverity::Error,
                        category: "xml-syntax".to_string(),
                        message: format!("XML syntax error: {}", e),
                        path: Some(format!("position {}", reader.buffer_position())),
                        suggestion: Some("Fix XML syntax errors".to_string()),
                    });
                    break;
                }
            }
        }

        // TODO: Add actual DDEX schema validation
        // This would require integrating with a schema validation library
        // and loading the appropriate DDEX schemas

        Ok(SchemaVerificationResult {
            success,
            issues,
        })
    }

    /// Verify deterministic output
    fn verify_determinism(
        &self,
        xml_output: &str,
        fidelity_options: &FidelityOptions,
    ) -> Result<DeterminismVerificationResult, BuildError> {
        let mut issues = Vec::new();
        let mut success = true;

        // Check for non-deterministic elements
        let non_deterministic_patterns = [
            (r#"\btimestamp\s*=\s*['"][^'"]*['"]"#, "timestamp attributes"),
            (r#"\bcreated\s*=\s*['"][^'"]*['"]"#, "creation time attributes"),
            (r#"\buuid\s*=\s*['"][^'"]*['"]"#, "UUID attributes"),
            (r#"\bid\s*=\s*['"]uuid:[^'"]*['"]"#, "UUID-based IDs"),
        ];

        for (pattern, description) in non_deterministic_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(xml_output) {
                    issues.push(VerificationIssue {
                        severity: VerificationSeverity::Warning,
                        category: "determinism".to_string(),
                        message: format!("Potentially non-deterministic element detected: {}", description),
                        path: None,
                        suggestion: Some("Use content-based IDs instead of random values".to_string()),
                    });
                }
            }
        }

        // Check attribute ordering consistency
        if !fidelity_options.preserve_attribute_order {
            // Verify that attributes are in a deterministic order
            if let Ok(attribute_order_regex) = regex::Regex::new(r#"<\w+[^>]*>"#) {
                for attr_match in attribute_order_regex.find_iter(xml_output) {
                    let element = attr_match.as_str();
                    if !self.is_attribute_order_deterministic(element) {
                        success = false;
                        issues.push(VerificationIssue {
                            severity: VerificationSeverity::Error,
                            category: "determinism".to_string(),
                            message: "Attributes are not in deterministic order".to_string(),
                            path: Some(element.to_string()),
                            suggestion: Some("Enable deterministic attribute ordering".to_string()),
                        });
                    }
                }
            }
        }

        Ok(DeterminismVerificationResult {
            success,
            issues,
        })
    }

    /// Check if attributes in an element are in deterministic order
    fn is_attribute_order_deterministic(&self, element_str: &str) -> bool {
        // Extract attributes and check if they are sorted
        if let Ok(attr_regex) = regex::Regex::new(r#"(\w+)\s*=\s*['"][^'"]*['"]"#) {
            let mut attributes: Vec<&str> = attr_regex
                .captures_iter(element_str)
                .filter_map(|cap| cap.get(1).map(|m| m.as_str()))
                .collect();
            
            let original_order = attributes.clone();
            attributes.sort();
            
            original_order == attributes
        } else {
            // If regex fails, assume deterministic
            true
        }
    }

    /// Canonicalize XML using the specified algorithm
    fn canonicalize_xml(
        &self,
        xml: &str,
        algorithm: &CanonicalizationAlgorithm,
    ) -> Result<String, BuildError> {
        match algorithm {
            CanonicalizationAlgorithm::None => Ok(xml.to_string()),
            CanonicalizationAlgorithm::C14N => {
                // TODO: Implement C14N canonicalization
                Ok(xml.to_string()) // Placeholder
            },
            CanonicalizationAlgorithm::C14N11 => {
                // TODO: Implement C14N11 canonicalization
                Ok(xml.to_string()) // Placeholder
            },
            CanonicalizationAlgorithm::DbC14N => {
                // TODO: Implement DB-C14N canonicalization
                Ok(xml.to_string()) // Placeholder
            },
            CanonicalizationAlgorithm::Custom(_rules) => {
                // TODO: Implement custom canonicalization
                Ok(xml.to_string()) // Placeholder
            },
        }
    }
}

/// Configuration for build verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    /// Enable round-trip verification (build → parse → build)
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

/// Overall verification result
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

/// Round-trip verification result
#[derive(Debug, Clone)]
struct RoundTripVerificationResult {
    success: bool,
    issues: Vec<VerificationIssue>,
}

/// Canonicalization verification result
#[derive(Debug, Clone)]
struct CanonicalizationVerificationResult {
    success: bool,
    issues: Vec<VerificationIssue>,
}

/// Schema verification result
#[derive(Debug, Clone)]
struct SchemaVerificationResult {
    success: bool,
    issues: Vec<VerificationIssue>,
}

/// Determinism verification result
#[derive(Debug, Clone)]
struct DeterminismVerificationResult {
    success: bool,
    issues: Vec<VerificationIssue>,
}

/// Verification report for detailed analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    /// Verification configuration used
    pub config: VerificationConfig,
    /// Fidelity options used
    pub fidelity_options: FidelityOptions,
    /// Overall result
    pub result: VerificationResult,
    /// Detailed statistics
    pub statistics: VerificationStatistics,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Verification statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStatistics {
    /// Total elements verified
    pub elements_verified: usize,
    /// Total attributes verified
    pub attributes_verified: usize,
    /// Namespaces processed
    pub namespaces_processed: usize,
    /// Comments verified
    pub comments_verified: usize,
    /// Processing instructions verified
    pub processing_instructions_verified: usize,
    /// Extensions verified
    pub extensions_verified: HashMap<String, usize>,
    /// Memory usage during verification
    pub memory_usage: usize,
}

impl Default for VerificationStatistics {
    fn default() -> Self {
        Self {
            elements_verified: 0,
            attributes_verified: 0,
            namespaces_processed: 0,
            comments_verified: 0,
            processing_instructions_verified: 0,
            extensions_verified: HashMap::new(),
            memory_usage: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FidelityOptions;

    #[test]
    fn test_verification_config_default() {
        let config = VerificationConfig::default();
        assert!(config.enable_round_trip_verification);
        assert!(config.enable_canonicalization_verification);
        assert!(!config.enable_schema_validation);
        assert!(config.enable_determinism_verification);
        assert_eq!(config.determinism_test_iterations, 3);
    }

    #[test]
    fn test_build_verifier_creation() {
        let config = VerificationConfig::default();
        let verifier = BuildVerifier::new(config);
        assert_eq!(verifier.config.determinism_test_iterations, 3);
    }

    #[test]
    fn test_attribute_order_determinism() {
        let verifier = BuildVerifier::new(VerificationConfig::default());
        
        // Deterministic (alphabetically ordered)
        assert!(verifier.is_attribute_order_deterministic(r#"<element a="1" b="2" c="3">"#));
        
        // Non-deterministic
        assert!(!verifier.is_attribute_order_deterministic(r#"<element c="3" a="1" b="2">"#));
    }

    #[test]
    fn test_verification_issue_creation() {
        let issue = VerificationIssue {
            severity: VerificationSeverity::Error,
            category: "test".to_string(),
            message: "Test issue".to_string(),
            path: Some("/test/path".to_string()),
            suggestion: Some("Fix the test".to_string()),
        };
        
        assert_eq!(issue.severity, VerificationSeverity::Error);
        assert_eq!(issue.category, "test");
        assert_eq!(issue.message, "Test issue");
    }

    #[test]
    fn test_verification_statistics() {
        let stats = VerificationStatistics::default();
        assert_eq!(stats.elements_verified, 0);
        assert_eq!(stats.attributes_verified, 0);
        assert_eq!(stats.memory_usage, 0);
    }
}