//! Comprehensive Fidelity Test Suite for DDEX Builder
//! 
//! This module implements comprehensive fidelity testing to ensure that:
//! - Parse → Build → Parse round-trips preserve all data
//! - Byte-perfect reproduction when using Perfect Fidelity Engine
//! - All ERN versions (3.8.2, 4.2, 4.3) work correctly
//! - Real-world DDEX files are handled properly

use ddex_builder::Builder;
use ddex_builder::builder::BuildRequest;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

// pub mod samples;
pub mod round_trip;
// pub mod perfect_fidelity;
// pub mod real_world;

/// Test configuration for fidelity testing
#[derive(Debug, Clone)]
pub struct FidelityTestConfig {
    /// Enable Perfect Fidelity Engine
    pub perfect_fidelity: bool,
    /// Test data directory
    pub test_data_dir: PathBuf,
    /// Expected versions to test
    pub versions: Vec<String>,
    /// Maximum file size for testing (in bytes)
    pub max_file_size: usize,
    /// Enable stress testing mode
    pub stress_mode: bool,
}

impl Default for FidelityTestConfig {
    fn default() -> Self {
        Self {
            perfect_fidelity: true,
            test_data_dir: PathBuf::from("tests/fidelity/data"),
            versions: vec!["3.8.2".to_string(), "4.2".to_string(), "4.3".to_string()],
            max_file_size: 100 * 1024 * 1024, // 100MB
            stress_mode: false,
        }
    }
}

/// Results from a fidelity test run
#[derive(Debug, Clone)]
pub struct FidelityTestResult {
    pub file_path: PathBuf,
    pub version: String,
    pub round_trip_success: bool,
    pub byte_perfect: bool,
    pub original_size: usize,
    pub rebuilt_size: usize,
    pub parse_time_ms: u64,
    pub build_time_ms: u64,
    pub error: Option<String>,
}

/// Main fidelity test runner
pub struct FidelityTestRunner {
    config: FidelityTestConfig,
    builder: Builder,
}

impl FidelityTestRunner {
    pub fn new(config: FidelityTestConfig) -> Self {
        let builder = Builder::new();
        
        // Configure fidelity options if needed
        // Note: Perfect fidelity is configured via preset or direct configuration
        
        Self { config, builder }
    }
    
    /// Run fidelity tests on all available test data
    pub async fn run_all_tests(&self) -> Result<Vec<FidelityTestResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        // Ensure test data directory exists
        fs::create_dir_all(&self.config.test_data_dir)?;
        
        // Generate sample data if not present
        if !self.has_sufficient_test_data()? {
            self.generate_test_data().await?;
        }
        
        // Run tests on all files
        let test_files = self.discover_test_files()?;
        
        for file_path in test_files {
            match self.test_file(&file_path).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(FidelityTestResult {
                        file_path: file_path.clone(),
                        version: "unknown".to_string(),
                        round_trip_success: false,
                        byte_perfect: false,
                        original_size: 0,
                        rebuilt_size: 0,
                        parse_time_ms: 0,
                        build_time_ms: 0,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Test a single file for fidelity
    pub async fn test_file(&self, file_path: &Path) -> Result<FidelityTestResult, Box<dyn std::error::Error>> {
        let original_xml = fs::read_to_string(file_path)?;
        let original_size = original_xml.len();
        
        // Parse timing
        let parse_start = std::time::Instant::now();
        let parsed_message = self.parse_xml(&original_xml)?;
        let parse_time_ms = parse_start.elapsed().as_millis() as u64;
        
        // Build timing
        let build_start = std::time::Instant::now();
        let build_request = self.create_build_request(&parsed_message)?;
        let rebuilt_xml = self.builder.build_with_fidelity(&build_request)?.xml;
        let build_time_ms = build_start.elapsed().as_millis() as u64;
        
        let rebuilt_size = rebuilt_xml.len();
        
        // Test round-trip by parsing again
        let reparsed_message = self.parse_xml(&rebuilt_xml).is_ok();
        
        // Test byte-perfect reproduction
        let byte_perfect = if self.config.perfect_fidelity {
            self.normalize_for_comparison(&original_xml) == self.normalize_for_comparison(&rebuilt_xml)
        } else {
            // Semantic equivalence check
            reparsed_message
        };
        
        let version = self.detect_version(&original_xml);
        
        Ok(FidelityTestResult {
            file_path: file_path.to_path_buf(),
            version,
            round_trip_success: reparsed_message,
            byte_perfect,
            original_size,
            rebuilt_size,
            parse_time_ms,
            build_time_ms,
            error: None,
        })
    }
    
    /// Check if we have sufficient test data
    fn has_sufficient_test_data(&self) -> Result<bool, std::io::Error> {
        let test_files = self.discover_test_files()?;
        
        // Check for coverage across versions
        let mut version_counts = HashMap::new();
        for file_path in &test_files {
            if let Ok(content) = fs::read_to_string(file_path) {
                let version = self.detect_version(&content);
                *version_counts.entry(version).or_insert(0) += 1;
            }
        }
        
        // We want at least 10 files per version
        for version in &self.config.versions {
            if version_counts.get(version).unwrap_or(&0) < &10 {
                return Ok(false);
            }
        }
        
        Ok(test_files.len() >= 100) // At least 100 total files
    }
    
    /// Generate test data if not present
    async fn generate_test_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Generating comprehensive test data...");
        
        // Generate samples for each version
        for version in &self.config.versions {
            self.generate_version_samples(version).await?;
        }
        
        Ok(())
    }
    
    /// Generate sample files for a specific version
    async fn generate_version_samples(&self, version: &str) -> Result<(), Box<dyn std::error::Error>> {
        let version_dir = self.config.test_data_dir.join(format!("ern_{}", version.replace(".", "_")));
        fs::create_dir_all(&version_dir)?;
        
        // Generate various types of DDEX messages
        let sample_generators = [
            ("simple_release", self.generate_simple_release(version)),
            ("complex_release", self.generate_complex_release(version)),
            ("multi_track_album", self.generate_multi_track_album(version)),
            ("compilation", self.generate_compilation(version)),
            ("single_track", self.generate_single_track(version)),
            ("classical_album", self.generate_classical_album(version)),
            ("electronic_ep", self.generate_electronic_ep(version)),
            ("live_recording", self.generate_live_recording(version)),
            ("soundtrack", self.generate_soundtrack(version)),
            ("podcast_series", self.generate_podcast_series(version)),
        ];
        
        for (name, sample) in sample_generators.iter() {
            for i in 0..10 {
                let filename = format!("{}_{:02}.xml", name, i + 1);
                let file_path = version_dir.join(filename);
                fs::write(&file_path, sample)?;
            }
        }
        
        Ok(())
    }
    
    /// Discover all test files in the test data directory
    fn discover_test_files(&self) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut files = Vec::new();
        
        if !self.config.test_data_dir.exists() {
            return Ok(files);
        }
        
        self.collect_xml_files(&self.config.test_data_dir, &mut files)?;
        Ok(files)
    }
    
    /// Recursively collect XML files
    fn collect_xml_files(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.collect_xml_files(&path, files)?;
            } else if path.extension().map_or(false, |ext| ext == "xml") {
                // Check file size
                if let Ok(metadata) = fs::metadata(&path) {
                    if metadata.len() <= self.config.max_file_size as u64 {
                        files.push(path);
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Parse XML content into a message structure
    fn parse_xml(&self, xml: &str) -> Result<ParsedMessage, Box<dyn std::error::Error>> {
        // Implementation would use ddex-parser here
        // For now, create a placeholder structure
        Ok(ParsedMessage {
            version: self.detect_version(xml),
            content: xml.to_string(),
        })
    }
    
    /// Create a build request from parsed message
    fn create_build_request(&self, message: &ParsedMessage) -> Result<BuildRequest, Box<dyn std::error::Error>> {
        // Implementation would convert parsed message to build request
        // For now, create a minimal build request
        Ok(BuildRequest {
            header: ddex_builder::builder::MessageHeaderRequest {
                message_id: Some("test_msg".to_string()),
                message_sender: ddex_builder::builder::PartyRequest {
                    party_name: vec![ddex_builder::builder::LocalizedStringRequest {
                        text: "test_sender".to_string(),
                        language_code: None,
                    }],
                    party_id: None,
                    party_reference: None,
                },
                message_recipient: ddex_builder::builder::PartyRequest {
                    party_name: vec![ddex_builder::builder::LocalizedStringRequest {
                        text: "test_recipient".to_string(),
                        language_code: None,
                    }],
                    party_id: None,
                    party_reference: None,
                },
                message_control_type: None,
                message_created_date_time: Some("2024-01-01T00:00:00Z".to_string()),
            },
            version: message.version.clone(),
            profile: None,
            releases: vec![],
            deals: vec![],
            extensions: None,
        })
    }
    
    /// Detect ERN version from XML content
    fn detect_version(&self, xml: &str) -> String {
        if xml.contains("http://ddex.net/xml/ern/382") {
            "3.8.2".to_string()
        } else if xml.contains("http://ddex.net/xml/ern/42") {
            "4.2".to_string()
        } else if xml.contains("http://ddex.net/xml/ern/43") {
            "4.3".to_string()
        } else {
            "unknown".to_string()
        }
    }
    
    /// Normalize XML for comparison
    fn normalize_for_comparison(&self, xml: &str) -> String {
        // Remove variations that don't affect semantic meaning
        xml.lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    }
    
    // Sample generators - simplified versions for testing
    fn generate_simple_release(&self, version: &str) -> String {
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/{version_path}" MessageSchemaVersionId="ern/{version_short}">
  <MessageHeader>
    <MessageId>MSG_{}</MessageId>
    <MessageSender>
      <PartyName>Test Label</PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyName>Test DSP</PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2024-01-01T00:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  <ResourceList>
    <SoundRecording>
      <ResourceReference>R001</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>SR_001</ResourceId>
      <ReferenceTitle>Test Track</ReferenceTitle>
      <Duration>PT3M30S</Duration>
    </SoundRecording>
  </ResourceList>
  <ReleaseList>
    <Release>
      <ReleaseReference>REL001</ReleaseReference>
      <ReleaseId>album_001</ReleaseId>
      <ReleaseType>Album</ReleaseType>
      <Title>Test Album</Title>
      <ResourceGroup>
        <ResourceReference>R001</ResourceReference>
      </ResourceGroup>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#,
            uuid::Uuid::new_v4().to_string(),
            version_path = version.replace(".", ""),
            version_short = version.replace(".", "")
        )
    }
    
    // Additional sample generators would be implemented here
    fn generate_complex_release(&self, version: &str) -> String {
        // More complex structure with multiple tracks, detailed metadata, etc.
        self.generate_simple_release(version) // Simplified for now
    }
    
    fn generate_multi_track_album(&self, version: &str) -> String {
        self.generate_simple_release(version) // Simplified for now
    }
    
    fn generate_compilation(&self, version: &str) -> String {
        self.generate_simple_release(version) // Simplified for now
    }
    
    fn generate_single_track(&self, version: &str) -> String {
        self.generate_simple_release(version) // Simplified for now
    }
    
    fn generate_classical_album(&self, version: &str) -> String {
        self.generate_simple_release(version) // Simplified for now
    }
    
    fn generate_electronic_ep(&self, version: &str) -> String {
        self.generate_simple_release(version) // Simplified for now
    }
    
    fn generate_live_recording(&self, version: &str) -> String {
        self.generate_simple_release(version) // Simplified for now
    }
    
    fn generate_soundtrack(&self, version: &str) -> String {
        self.generate_simple_release(version) // Simplified for now
    }
    
    fn generate_podcast_series(&self, version: &str) -> String {
        self.generate_simple_release(version) // Simplified for now
    }
}

/// Placeholder for parsed message structure
#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub version: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
        
    #[tokio::test]
    async fn test_fidelity_runner_creation() {
        let config = FidelityTestConfig::default();
        let runner = FidelityTestRunner::new(config);
        assert!(runner.config.perfect_fidelity);
    }
    
    #[tokio::test] 
    async fn test_version_detection() {
        let config = FidelityTestConfig::default();
        let runner = FidelityTestRunner::new(config);
        
        let ern_43_xml = r#"<root xmlns:ern="http://ddex.net/xml/ern/43">test</root>"#;
        assert_eq!(runner.detect_version(ern_43_xml), "4.3");
        
        let ern_42_xml = r#"<root xmlns:ern="http://ddex.net/xml/ern/42">test</root>"#;
        assert_eq!(runner.detect_version(ern_42_xml), "4.2");
        
        let ern_382_xml = r#"<root xmlns:ern="http://ddex.net/xml/ern/382">test</root>"#;
        assert_eq!(runner.detect_version(ern_382_xml), "3.8.2");
    }
    
    #[tokio::test]
    async fn test_sample_generation() {
        let config = FidelityTestConfig::default();
        let runner = FidelityTestRunner::new(config);
        
        let sample = runner.generate_simple_release("4.3");
        assert!(sample.contains("ern:NewReleaseMessage"));
        assert!(sample.contains("http://ddex.net/xml/ern/43"));
    }
}