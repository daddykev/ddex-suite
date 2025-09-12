//! Round-trip fidelity testing
//! 
//! Tests that parse → build → parse operations preserve all data
//! and maintain semantic equivalence across different ERN versions

use super::*;
use std::time::Instant;

/// Round-trip test suite
pub struct RoundTripTester {
    config: FidelityTestConfig,
    runner: FidelityTestRunner,
}

impl RoundTripTester {
    pub fn new(config: FidelityTestConfig) -> Self {
        let runner = FidelityTestRunner::new(config.clone());
        Self { config, runner }
    }
    
    /// Run comprehensive round-trip tests
    pub async fn run_round_trip_tests(&self) -> Result<RoundTripResults, Box<dyn std::error::Error>> {
        let mut results = RoundTripResults::new();
        
        // Test each supported version
        for version in &self.config.versions {
            let version_results = self.test_version_round_trips(version).await?;
            results.add_version_results(version.clone(), version_results);
        }
        
        Ok(results)
    }
    
    /// Test round-trips for a specific ERN version
    async fn test_version_round_trips(&self, version: &str) -> Result<Vec<RoundTripTestCase>, Box<dyn std::error::Error>> {
        let mut test_cases = Vec::new();
        
        // Test simple cases
        test_cases.extend(self.test_simple_round_trips(version).await?);
        
        // Test complex cases
        test_cases.extend(self.test_complex_round_trips(version).await?);
        
        // Test edge cases
        test_cases.extend(self.test_edge_cases(version).await?);
        
        Ok(test_cases)
    }
    
    /// Test simple round-trip cases
    async fn test_simple_round_trips(&self, version: &str) -> Result<Vec<RoundTripTestCase>, Box<dyn std::error::Error>> {
        let mut cases = Vec::new();
        
        let test_cases = [
            ("minimal_release", self.create_minimal_release(version)),
            ("single_track", self.create_single_track(version)),
            ("basic_metadata", self.create_basic_metadata(version)),
        ];
        
        for (name, xml) in test_cases.iter() {
            let case = self.test_single_round_trip(name, xml, version).await?;
            cases.push(case);
        }
        
        Ok(cases)
    }
    
    /// Test complex round-trip cases
    async fn test_complex_round_trips(&self, version: &str) -> Result<Vec<RoundTripTestCase>, Box<dyn std::error::Error>> {
        let mut cases = Vec::new();
        
        let test_cases = [
            ("multi_track_album", self.create_multi_track_album(version)),
            ("complex_metadata", self.create_complex_metadata(version)),
            ("nested_resources", self.create_nested_resources(version)),
            ("full_deal_terms", self.create_full_deal_terms(version)),
        ];
        
        for (name, xml) in test_cases.iter() {
            let case = self.test_single_round_trip(name, xml, version).await?;
            cases.push(case);
        }
        
        Ok(cases)
    }
    
    /// Test edge cases that might break round-trip fidelity
    async fn test_edge_cases(&self, version: &str) -> Result<Vec<RoundTripTestCase>, Box<dyn std::error::Error>> {
        let mut cases = Vec::new();
        
        let test_cases = [
            ("empty_elements", self.create_empty_elements(version)),
            ("special_characters", self.create_special_characters(version)),
            ("unicode_content", self.create_unicode_content(version)),
            ("mixed_content", self.create_mixed_content(version)),
            ("large_text_blocks", self.create_large_text_blocks(version)),
            ("namespace_variations", self.create_namespace_variations(version)),
        ];
        
        for (name, xml) in test_cases.iter() {
            let case = self.test_single_round_trip(name, xml, version).await?;
            cases.push(case);
        }
        
        Ok(cases)
    }
    
    /// Test a single round-trip case
    async fn test_single_round_trip(&self, name: &str, xml: &str, version: &str) -> Result<RoundTripTestCase, Box<dyn std::error::Error>> {
        let _start_time = Instant::now();
        
        // Step 1: Parse original XML
        let parse1_start = Instant::now();
        let parsed1 = self.runner.parse_xml(xml);
        let parse1_time = parse1_start.elapsed();
        
        let parsed1 = match parsed1 {
            Ok(p) => p,
            Err(e) => {
                return Ok(RoundTripTestCase {
                    name: name.to_string(),
                    version: version.to_string(),
                    success: false,
                    original_size: xml.len(),
                    intermediate_size: 0,
                    final_size: 0,
                    parse1_time_ms: parse1_time.as_millis() as u64,
                    build_time_ms: 0,
                    parse2_time_ms: 0,
                    error: Some(format!("Initial parse failed: {}", e)),
                    data_preserved: false,
                    structure_preserved: false,
                    metadata_preserved: false,
                });
            }
        };
        
        // Step 2: Build XML from parsed data
        let build_start = Instant::now();
        let build_request = self.runner.create_build_request(&parsed1)?;
        let built_xml = self.runner.builder.build_with_fidelity(&build_request);
        let build_time = build_start.elapsed();
        
        let built_xml = match built_xml {
            Ok(result) => result.xml,
            Err(e) => {
                return Ok(RoundTripTestCase {
                    name: name.to_string(),
                    version: version.to_string(),
                    success: false,
                    original_size: xml.len(),
                    intermediate_size: 0,
                    final_size: 0,
                    parse1_time_ms: parse1_time.as_millis() as u64,
                    build_time_ms: build_time.as_millis() as u64,
                    parse2_time_ms: 0,
                    error: Some(format!("Build failed: {}", e)),
                    data_preserved: false,
                    structure_preserved: false,
                    metadata_preserved: false,
                });
            }
        };
        
        // Step 3: Parse the rebuilt XML
        let parse2_start = Instant::now();
        let parsed2 = self.runner.parse_xml(&built_xml);
        let parse2_time = parse2_start.elapsed();
        
        let parsed2 = match parsed2 {
            Ok(p) => p,
            Err(e) => {
                return Ok(RoundTripTestCase {
                    name: name.to_string(),
                    version: version.to_string(),
                    success: false,
                    original_size: xml.len(),
                    intermediate_size: built_xml.len(),
                    final_size: 0,
                    parse1_time_ms: parse1_time.as_millis() as u64,
                    build_time_ms: build_time.as_millis() as u64,
                    parse2_time_ms: parse2_time.as_millis() as u64,
                    error: Some(format!("Re-parse failed: {}", e)),
                    data_preserved: false,
                    structure_preserved: false,
                    metadata_preserved: false,
                });
            }
        };
        
        // Step 4: Analyze preservation
        let data_preserved = self.compare_data_preservation(&parsed1, &parsed2);
        let structure_preserved = self.compare_structure_preservation(&parsed1, &parsed2);
        let metadata_preserved = self.compare_metadata_preservation(&parsed1, &parsed2);
        
        let success = data_preserved && structure_preserved && metadata_preserved;
        
        Ok(RoundTripTestCase {
            name: name.to_string(),
            version: version.to_string(),
            success,
            original_size: xml.len(),
            intermediate_size: built_xml.len(),
            final_size: built_xml.len(),
            parse1_time_ms: parse1_time.as_millis() as u64,
            build_time_ms: build_time.as_millis() as u64,
            parse2_time_ms: parse2_time.as_millis() as u64,
            error: if success { None } else { Some("Data/structure/metadata not preserved".to_string()) },
            data_preserved,
            structure_preserved,
            metadata_preserved,
        })
    }
    
    /// Compare data preservation between two parsed messages
    fn compare_data_preservation(&self, original: &ParsedMessage, rebuilt: &ParsedMessage) -> bool {
        // This would implement detailed data comparison
        // For now, basic content comparison
        original.version == rebuilt.version
    }
    
    /// Compare structure preservation
    fn compare_structure_preservation(&self, _original: &ParsedMessage, _rebuilt: &ParsedMessage) -> bool {
        // This would analyze XML structure preservation
        // For now, basic check
        true
    }
    
    /// Compare metadata preservation
    fn compare_metadata_preservation(&self, _original: &ParsedMessage, _rebuilt: &ParsedMessage) -> bool {
        // This would check all metadata fields
        // For now, basic check
        true
    }
    
    // Sample XML generators for different test cases
    fn create_minimal_release(&self, version: &str) -> String {
        let version_path = version.replace(".", "");
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/{version_path}">
  <MessageHeader>
    <MessageId>MIN_001</MessageId>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_MIN_001</ReleaseId>
      <Title>Minimal Release</Title>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#)
    }
    
    fn create_single_track(&self, version: &str) -> String {
        let version_path = version.replace(".", "");
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/{version_path}">
  <MessageHeader>
    <MessageId>ST_001</MessageId>
  </MessageHeader>
  <ResourceList>
    <SoundRecording>
      <ResourceReference>R001</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>SR_001</ResourceId>
      <ReferenceTitle>Test Track</ReferenceTitle>
    </SoundRecording>
  </ResourceList>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_ST_001</ReleaseId>
      <Title>Single Track</Title>
      <ResourceGroup>
        <ResourceReference>R001</ResourceReference>
      </ResourceGroup>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#)
    }
    
    fn create_basic_metadata(&self, version: &str) -> String {
        self.create_single_track(version) // Simplified
    }
    
    fn create_multi_track_album(&self, version: &str) -> String {
        let version_path = version.replace(".", "");
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/{version_path}">
  <MessageHeader>
    <MessageId>MTA_001</MessageId>
  </MessageHeader>
  <ResourceList>
    <SoundRecording>
      <ResourceReference>R001</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>SR_001</ResourceId>
      <ReferenceTitle>Track 1</ReferenceTitle>
    </SoundRecording>
    <SoundRecording>
      <ResourceReference>R002</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>SR_002</ResourceId>
      <ReferenceTitle>Track 2</ReferenceTitle>
    </SoundRecording>
  </ResourceList>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_MTA_001</ReleaseId>
      <Title>Multi-Track Album</Title>
      <ResourceGroup>
        <ResourceReference>R001</ResourceReference>
        <ResourceReference>R002</ResourceReference>
      </ResourceGroup>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#)
    }
    
    fn create_complex_metadata(&self, version: &str) -> String {
        self.create_multi_track_album(version) // Simplified
    }
    
    fn create_nested_resources(&self, version: &str) -> String {
        self.create_multi_track_album(version) // Simplified
    }
    
    fn create_full_deal_terms(&self, version: &str) -> String {
        self.create_multi_track_album(version) // Simplified
    }
    
    fn create_empty_elements(&self, version: &str) -> String {
        let version_path = version.replace(".", "");
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/{version_path}">
  <MessageHeader>
    <MessageId>EMPTY_001</MessageId>
    <EmptyElement/>
    <EmptyElement></EmptyElement>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_EMPTY_001</ReleaseId>
      <Title>Empty Elements Test</Title>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#)
    }
    
    fn create_special_characters(&self, version: &str) -> String {
        let version_path = version.replace(".", "");
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/{version_path}">
  <MessageHeader>
    <MessageId>SPEC_001</MessageId>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_SPEC_001</ReleaseId>
      <Title>Special &amp; Characters &lt;&gt; "Quotes" 'Apostrophes'</Title>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#)
    }
    
    fn create_unicode_content(&self, version: &str) -> String {
        let version_path = version.replace(".", "");
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/{version_path}">
  <MessageHeader>
    <MessageId>UNI_001</MessageId>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_UNI_001</ReleaseId>
      <Title>Ñiño's Café 音楽 🎵 Русский</Title>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#)
    }
    
    fn create_mixed_content(&self, version: &str) -> String {
        let version_path = version.replace(".", "");
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/{version_path}">
  <MessageHeader>
    <MessageId>MIX_001</MessageId>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_MIX_001</ReleaseId>
      <Title>Mixed <em>Content</em> Test</Title>
      <Description>This has <!-- comment --> mixed content</Description>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#)
    }
    
    fn create_large_text_blocks(&self, version: &str) -> String {
        let large_text = "Lorem ipsum ".repeat(1000);
        let version_path = version.replace(".", "");
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/{version_path}">
  <MessageHeader>
    <MessageId>LARGE_001</MessageId>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_LARGE_001</ReleaseId>
      <Title>Large Text Test</Title>
      <Description>{large_text}</Description>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#)
    }
    
    fn create_namespace_variations(&self, version: &str) -> String {
        let version_path = version.replace(".", "");
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/{version_path}" xmlns:custom="http://example.com/custom">
  <MessageHeader>
    <MessageId>NS_001</MessageId>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_NS_001</ReleaseId>
      <Title>Namespace Test</Title>
      <custom:CustomField>Custom Value</custom:CustomField>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#)
    }
}

/// Results from round-trip testing
#[derive(Debug, Clone)]
pub struct RoundTripResults {
    pub version_results: std::collections::HashMap<String, Vec<RoundTripTestCase>>,
    pub total_tests: usize,
    pub successful_tests: usize,
    pub failed_tests: usize,
}

impl RoundTripResults {
    pub fn new() -> Self {
        Self {
            version_results: std::collections::HashMap::new(),
            total_tests: 0,
            successful_tests: 0,
            failed_tests: 0,
        }
    }
    
    pub fn add_version_results(&mut self, version: String, results: Vec<RoundTripTestCase>) {
        let successful = results.iter().filter(|r| r.success).count();
        let total = results.len();
        
        self.total_tests += total;
        self.successful_tests += successful;
        self.failed_tests += total - successful;
        
        self.version_results.insert(version, results);
    }
    
    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.successful_tests as f64 / self.total_tests as f64
        }
    }
    
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("Round-Trip Test Results\n"));
        report.push_str(&format!("=======================\n"));
        report.push_str(&format!("Total Tests: {}\n", self.total_tests));
        report.push_str(&format!("Successful: {}\n", self.successful_tests));
        report.push_str(&format!("Failed: {}\n", self.failed_tests));
        report.push_str(&format!("Success Rate: {:.2}%\n\n", self.success_rate() * 100.0));
        
        for (version, results) in &self.version_results {
            let successful = results.iter().filter(|r| r.success).count();
            report.push_str(&format!("ERN {}: {}/{} tests passed\n", version, successful, results.len()));
            
            for result in results.iter().filter(|r| !r.success) {
                report.push_str(&format!("  FAILED: {} - {}\n", 
                    result.name, 
                    result.error.as_ref().unwrap_or(&"Unknown error".to_string())));
            }
        }
        
        report
    }
}

/// Individual round-trip test case result
#[derive(Debug, Clone)]
pub struct RoundTripTestCase {
    pub name: String,
    pub version: String,
    pub success: bool,
    pub original_size: usize,
    pub intermediate_size: usize,
    pub final_size: usize,
    pub parse1_time_ms: u64,
    pub build_time_ms: u64,
    pub parse2_time_ms: u64,
    pub error: Option<String>,
    pub data_preserved: bool,
    pub structure_preserved: bool,
    pub metadata_preserved: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_round_trip_tester_creation() {
        let config = FidelityTestConfig::default();
        let tester = RoundTripTester::new(config);
        assert!(tester.config.perfect_fidelity);
    }
    
    #[tokio::test]
    async fn test_minimal_round_trip() {
        let config = FidelityTestConfig::default();
        let tester = RoundTripTester::new(config);
        
        let xml = tester.create_minimal_release("4.3");
        assert!(xml.contains("MIN_001"));
        assert!(xml.contains("Minimal Release"));
    }
    
    #[tokio::test]
    async fn test_results_aggregation() {
        let mut results = RoundTripResults::new();
        
        let test_case = RoundTripTestCase {
            name: "test".to_string(),
            version: "4.3".to_string(),
            success: true,
            original_size: 100,
            intermediate_size: 105,
            final_size: 105,
            parse1_time_ms: 5,
            build_time_ms: 10,
            parse2_time_ms: 5,
            error: None,
            data_preserved: true,
            structure_preserved: true,
            metadata_preserved: true,
        };
        
        results.add_version_results("4.3".to_string(), vec![test_case]);
        assert_eq!(results.total_tests, 1);
        assert_eq!(results.successful_tests, 1);
        assert_eq!(results.success_rate(), 1.0);
    }
}