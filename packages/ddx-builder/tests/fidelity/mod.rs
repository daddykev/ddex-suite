//! # Perfect Fidelity Engine Test Suite
//! 
//! Comprehensive testing system for validating 100% round-trip fidelity
//! of DDEX XML processing through parse → modify → build cycles.

use std::path::{Path, PathBuf};
use std::fs;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub mod harness;
pub mod fixtures;
pub mod benchmarks;
pub mod reports;
pub mod synthetic;

/// Test result for a single DDEX file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityTestResult {
    pub test_name: String,
    pub file_path: String,
    pub ern_version: String,
    pub file_size_bytes: u64,
    pub success: bool,
    pub round_trip_success: bool,
    pub modification_success: bool,
    pub canonicalization_success: bool,
    pub extension_preservation_success: bool,
    pub comment_preservation_success: bool,
    pub parse_time_ms: u64,
    pub build_time_ms: u64,
    pub total_time_ms: u64,
    pub memory_peak_mb: f64,
    pub error_message: Option<String>,
    pub byte_differences: Vec<ByteDifference>,
    pub attribute_differences: Vec<AttributeDifference>,
    pub structure_differences: Vec<StructureDifference>,
}

/// Byte-level difference in output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteDifference {
    pub position: usize,
    pub expected: u8,
    pub actual: u8,
    pub context: String,
}

/// Attribute-level difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDifference {
    pub element_path: String,
    pub attribute_name: String,
    pub expected_value: Option<String>,
    pub actual_value: Option<String>,
    pub difference_type: AttributeDifferenceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeDifferenceType {
    Missing,
    Added,
    ValueChanged,
    OrderChanged,
}

/// Structural difference in XML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureDifference {
    pub element_path: String,
    pub difference_type: StructureDifferenceType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructureDifferenceType {
    MissingElement,
    ExtraElement,
    OrderChanged,
    ContentChanged,
    NamespaceChanged,
}

/// Test suite configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityTestConfig {
    pub test_fixtures_dir: PathBuf,
    pub output_dir: PathBuf,
    pub parallel_execution: bool,
    pub max_parallel_tests: usize,
    pub timeout_seconds: u64,
    pub memory_limit_mb: Option<u64>,
    pub generate_diff_reports: bool,
    pub save_intermediate_files: bool,
    pub test_categories: Vec<TestCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestCategory {
    BasicRoundTrip,
    ModificationRoundTrip,
    CanonicalizationConsistency,
    ExtensionPreservation,
    CommentRetention,
    PerformanceBenchmark,
    LargeFileStressTest,
    ProprietaryExtensions,
}

/// Comprehensive test suite results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityTestSuite {
    pub config: FidelityTestConfig,
    pub start_time: String,
    pub end_time: String,
    pub total_duration_ms: u64,
    pub total_files_tested: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub success_rate: f64,
    pub results: Vec<FidelityTestResult>,
    pub performance_summary: PerformanceSummary,
    pub compatibility_matrix: CompatibilityMatrix,
    pub error_patterns: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub avg_parse_time_ms: f64,
    pub avg_build_time_ms: f64,
    pub avg_memory_usage_mb: f64,
    pub max_file_size_tested: u64,
    pub throughput_files_per_second: f64,
    pub throughput_mb_per_second: f64,
    pub performance_by_file_size: Vec<PerformanceBySize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBySize {
    pub size_range: String,
    pub file_count: usize,
    pub avg_parse_time_ms: f64,
    pub avg_build_time_ms: f64,
    pub avg_memory_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityMatrix {
    pub ern_382_success_rate: f64,
    pub ern_42_success_rate: f64,
    pub ern_43_success_rate: f64,
    pub partner_extensions: HashMap<String, f64>, // partner -> success_rate
    pub feature_support: HashMap<String, f64>, // feature -> success_rate
}

impl Default for FidelityTestConfig {
    fn default() -> Self {
        Self {
            test_fixtures_dir: PathBuf::from("../../test-fixtures/real-world"),
            output_dir: PathBuf::from("target/fidelity-test-results"),
            parallel_execution: true,
            max_parallel_tests: num_cpus::get(),
            timeout_seconds: 300, // 5 minutes per test
            memory_limit_mb: Some(1024), // 1GB limit
            generate_diff_reports: true,
            save_intermediate_files: false,
            test_categories: vec![
                TestCategory::BasicRoundTrip,
                TestCategory::ModificationRoundTrip,
                TestCategory::CanonicalizationConsistency,
                TestCategory::ExtensionPreservation,
                TestCategory::CommentRetention,
                TestCategory::PerformanceBenchmark,
            ],
        }
    }
}

impl FidelityTestResult {
    pub fn new(test_name: String, file_path: String) -> Self {
        Self {
            test_name,
            file_path,
            ern_version: String::new(),
            file_size_bytes: 0,
            success: false,
            round_trip_success: false,
            modification_success: false,
            canonicalization_success: false,
            extension_preservation_success: false,
            comment_preservation_success: false,
            parse_time_ms: 0,
            build_time_ms: 0,
            total_time_ms: 0,
            memory_peak_mb: 0.0,
            error_message: None,
            byte_differences: Vec::new(),
            attribute_differences: Vec::new(),
            structure_differences: Vec::new(),
        }
    }

    pub fn mark_success(&mut self) {
        self.success = true;
    }

    pub fn mark_failure(&mut self, error: String) {
        self.success = false;
        self.error_message = Some(error);
    }

    pub fn add_timing(&mut self, parse_time: Duration, build_time: Duration) {
        self.parse_time_ms = parse_time.as_millis() as u64;
        self.build_time_ms = build_time.as_millis() as u64;
        self.total_time_ms = self.parse_time_ms + self.build_time_ms;
    }

    pub fn add_byte_difference(&mut self, pos: usize, expected: u8, actual: u8, context: String) {
        self.byte_differences.push(ByteDifference {
            position: pos,
            expected,
            actual,
            context,
        });
    }

    pub fn add_attribute_difference(&mut self, diff: AttributeDifference) {
        self.attribute_differences.push(diff);
    }

    pub fn add_structure_difference(&mut self, diff: StructureDifference) {
        self.structure_differences.push(diff);
    }

    /// Overall success is true only if all categories pass
    pub fn calculate_overall_success(&mut self) {
        self.success = self.round_trip_success
            && self.modification_success  
            && self.canonicalization_success
            && self.extension_preservation_success
            && self.comment_preservation_success
            && self.byte_differences.is_empty()
            && self.attribute_differences.is_empty()
            && self.structure_differences.is_empty();
    }
}

impl FidelityTestSuite {
    pub fn new(config: FidelityTestConfig) -> Self {
        Self {
            config,
            start_time: chrono::Utc::now().to_rfc3339(),
            end_time: String::new(),
            total_duration_ms: 0,
            total_files_tested: 0,
            success_count: 0,
            failure_count: 0,
            success_rate: 0.0,
            results: Vec::new(),
            performance_summary: PerformanceSummary::default(),
            compatibility_matrix: CompatibilityMatrix::default(),
            error_patterns: HashMap::new(),
        }
    }

    pub fn add_result(&mut self, result: FidelityTestResult) {
        self.total_files_tested += 1;
        if result.success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
            if let Some(error) = &result.error_message {
                *self.error_patterns.entry(error.clone()).or_insert(0) += 1;
            }
        }
        self.results.push(result);
    }

    pub fn finalize(&mut self, start_time: Instant) {
        self.end_time = chrono::Utc::now().to_rfc3339();
        self.total_duration_ms = start_time.elapsed().as_millis() as u64;
        self.success_rate = if self.total_files_tested > 0 {
            self.success_count as f64 / self.total_files_tested as f64 * 100.0
        } else {
            0.0
        };
        
        self.calculate_performance_summary();
        self.calculate_compatibility_matrix();
    }

    fn calculate_performance_summary(&mut self) {
        if self.results.is_empty() {
            return;
        }

        let successful_results: Vec<_> = self.results.iter().filter(|r| r.success).collect();
        
        if successful_results.is_empty() {
            return;
        }

        let total_parse_time: u64 = successful_results.iter().map(|r| r.parse_time_ms).sum();
        let total_build_time: u64 = successful_results.iter().map(|r| r.build_time_ms).sum();
        let total_memory: f64 = successful_results.iter().map(|r| r.memory_peak_mb).sum();
        let count = successful_results.len() as f64;

        self.performance_summary = PerformanceSummary {
            avg_parse_time_ms: total_parse_time as f64 / count,
            avg_build_time_ms: total_build_time as f64 / count,
            avg_memory_usage_mb: total_memory / count,
            max_file_size_tested: successful_results.iter().map(|r| r.file_size_bytes).max().unwrap_or(0),
            throughput_files_per_second: count / (self.total_duration_ms as f64 / 1000.0),
            throughput_mb_per_second: successful_results.iter().map(|r| r.file_size_bytes as f64 / 1024.0 / 1024.0).sum::<f64>() / (self.total_duration_ms as f64 / 1000.0),
            performance_by_file_size: self.calculate_performance_by_size(&successful_results),
        };
    }

    fn calculate_performance_by_size(&self, results: &[&FidelityTestResult]) -> Vec<PerformanceBySize> {
        let mut size_buckets: HashMap<String, Vec<&FidelityTestResult>> = HashMap::new();
        
        for result in results {
            let bucket = match result.file_size_bytes {
                0..=10_000 => "0-10KB".to_string(),
                10_001..=100_000 => "10-100KB".to_string(),
                100_001..=1_000_000 => "100KB-1MB".to_string(),
                1_000_001..=10_000_000 => "1-10MB".to_string(),
                _ => "10MB+".to_string(),
            };
            size_buckets.entry(bucket).or_insert_with(Vec::new).push(result);
        }

        size_buckets.into_iter().map(|(size_range, bucket_results)| {
            let count = bucket_results.len();
            let avg_parse_time = bucket_results.iter().map(|r| r.parse_time_ms).sum::<u64>() as f64 / count as f64;
            let avg_build_time = bucket_results.iter().map(|r| r.build_time_ms).sum::<u64>() as f64 / count as f64;
            let avg_memory = bucket_results.iter().map(|r| r.memory_peak_mb).sum::<f64>() / count as f64;

            PerformanceBySize {
                size_range,
                file_count: count,
                avg_parse_time_ms: avg_parse_time,
                avg_build_time_ms: avg_build_time,
                avg_memory_mb: avg_memory,
            }
        }).collect()
    }

    fn calculate_compatibility_matrix(&mut self) {
        let ern_382_results: Vec<_> = self.results.iter().filter(|r| r.ern_version.contains("3.8.2")).collect();
        let ern_42_results: Vec<_> = self.results.iter().filter(|r| r.ern_version.contains("4.2")).collect();
        let ern_43_results: Vec<_> = self.results.iter().filter(|r| r.ern_version.contains("4.3")).collect();

        self.compatibility_matrix = CompatibilityMatrix {
            ern_382_success_rate: Self::calculate_success_rate(&ern_382_results),
            ern_42_success_rate: Self::calculate_success_rate(&ern_42_results),
            ern_43_success_rate: Self::calculate_success_rate(&ern_43_results),
            partner_extensions: HashMap::new(), // To be filled by specific tests
            feature_support: HashMap::new(), // To be filled by specific tests
        };
    }

    fn calculate_success_rate(results: &[&FidelityTestResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        let success_count = results.iter().filter(|r| r.success).count();
        success_count as f64 / results.len() as f64 * 100.0
    }
}

impl Default for PerformanceSummary {
    fn default() -> Self {
        Self {
            avg_parse_time_ms: 0.0,
            avg_build_time_ms: 0.0,
            avg_memory_usage_mb: 0.0,
            max_file_size_tested: 0,
            throughput_files_per_second: 0.0,
            throughput_mb_per_second: 0.0,
            performance_by_file_size: Vec::new(),
        }
    }
}

impl Default for CompatibilityMatrix {
    fn default() -> Self {
        Self {
            ern_382_success_rate: 0.0,
            ern_42_success_rate: 0.0,
            ern_43_success_rate: 0.0,
            partner_extensions: HashMap::new(),
            feature_support: HashMap::new(),
        }
    }
}

/// Test discovery utilities
pub fn discover_test_files(fixtures_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut test_files = Vec::new();
    
    if !fixtures_dir.exists() {
        return Err(format!("Test fixtures directory does not exist: {}", fixtures_dir.display()).into());
    }

    // Recursively find all XML files
    fn find_xml_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                find_xml_files(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("xml") {
                files.push(path);
            }
        }
        Ok(())
    }

    find_xml_files(fixtures_dir, &mut test_files)?;
    test_files.sort();
    
    Ok(test_files)
}

/// Extract ERN version from XML content
pub fn detect_ern_version(xml_content: &str) -> String {
    if xml_content.contains("ern/382") {
        "ERN-3.8.2".to_string()
    } else if xml_content.contains("ern/42") {
        "ERN-4.2".to_string()
    } else if xml_content.contains("ern/43") {
        "ERN-4.3".to_string()
    } else if xml_content.contains("ern/341") {
        "ERN-3.4.1".to_string()
    } else {
        "Unknown".to_string()
    }
}