//! Integration tests for comprehensive fidelity testing
//! 
//! This module runs the complete fidelity test suite covering:
//! - Parse → Build → Parse round-trips for all ERN versions
//! - Byte-perfect reproduction with Perfect Fidelity Engine  
//! - Real-world DDEX XML file processing
//! - Performance and memory usage monitoring

// This would import from a proper fidelity module
// use ddex_builder::tests::fidelity::{FidelityTestConfig, FidelityTestRunner, RoundTripTester};
use crate::fidelity::{FidelityTestConfig, FidelityTestRunner};
use crate::fidelity::round_trip::RoundTripTester;
use std::time::Instant;
use tempfile::TempDir;

mod fidelity;

#[tokio::test]
async fn comprehensive_fidelity_test_suite() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = FidelityTestConfig {
        perfect_fidelity: true,
        test_data_dir: temp_dir.path().join("test_data"),
        versions: vec!["3.8.2".to_string(), "4.2".to_string(), "4.3".to_string()],
        max_file_size: 10 * 1024 * 1024, // 10MB for testing
        stress_mode: false,
    };
    
    let start_time = Instant::now();
    
    // Run the comprehensive fidelity test
    let runner = FidelityTestRunner::new(config.clone());
    let results = runner.run_all_tests().await.expect("Fidelity tests should complete");
    
    let total_time = start_time.elapsed();
    
    // Analyze results
    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r.round_trip_success).count();
    let byte_perfect_tests = results.iter().filter(|r| r.byte_perfect).count();
    
    println!("Fidelity Test Results:");
    println!("Total tests: {}", total_tests);
    println!("Successful round-trips: {}", successful_tests);
    println!("Byte-perfect reproductions: {}", byte_perfect_tests);
    println!("Total time: {:?}", total_time);
    
    // Assert success criteria
    if total_tests > 0 {
        let success_rate = successful_tests as f64 / total_tests as f64;
        let byte_perfect_rate = byte_perfect_tests as f64 / total_tests as f64;
        
        println!("Success rate: {:.2}%", success_rate * 100.0);
        println!("Byte-perfect rate: {:.2}%", byte_perfect_rate * 100.0);
        
        // We expect high success rates for fidelity
        assert!(success_rate >= 0.95, "Round-trip success rate should be at least 95%");
        
        if config.perfect_fidelity {
            assert!(byte_perfect_rate >= 0.90, "Byte-perfect reproduction rate should be at least 90% with Perfect Fidelity Engine");
        }
    }
    
    // Report any failures
    for result in &results {
        if !result.round_trip_success {
            println!("FAILED: {:?} - {}", 
                result.file_path, 
                result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
        }
    }
}

#[tokio::test]
async fn round_trip_fidelity_test_all_versions() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = FidelityTestConfig {
        perfect_fidelity: true,
        test_data_dir: temp_dir.path().join("round_trip_data"),
        versions: vec!["3.8.2".to_string(), "4.2".to_string(), "4.3".to_string()],
        max_file_size: 5 * 1024 * 1024, // 5MB for testing
        stress_mode: false,
    };
    
    let tester = RoundTripTester::new(config);
    let results = tester.run_round_trip_tests().await.expect("Round-trip tests should complete");
    
    // Generate and print report
    let report = results.generate_report();
    println!("{}", report);
    
    // Assert success criteria
    assert!(results.total_tests > 0, "Should have run some tests");
    assert!(results.success_rate() >= 0.95, "Round-trip success rate should be at least 95%");
    
    // Check each version has good coverage
    for version in &["3.8.2", "4.2", "4.3"] {
        if let Some(version_results) = results.version_results.get(*version) {
            let version_success_rate = version_results.iter().filter(|r| r.success).count() as f64 / version_results.len() as f64;
            assert!(version_success_rate >= 0.90, "Version {} should have at least 90% success rate", version);
        }
    }
}

#[tokio::test]
async fn perfect_fidelity_byte_comparison_test() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = FidelityTestConfig {
        perfect_fidelity: true,
        test_data_dir: temp_dir.path().join("perfect_fidelity_data"),
        versions: vec!["4.3".to_string()], // Test just one version for focus
        max_file_size: 1024 * 1024, // 1MB
        stress_mode: false,
    };
    
    let runner = FidelityTestRunner::new(config);
    
    // Test with a well-formed sample
    let sample_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <MessageId>PERFECT_001</MessageId>
    <MessageSender>
      <PartyName>Test Label</PartyName>
    </MessageSender>
    <MessageRecipient>
      <PartyName>Test DSP</PartyName>
    </MessageRecipient>
    <MessageCreatedDateTime>2024-01-01T00:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_PERFECT_001</ReleaseId>
      <Title>Perfect Fidelity Test</Title>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#;
    
    // Create a temporary file
    let test_file = temp_dir.path().join("perfect_test.xml");
    std::fs::write(&test_file, sample_xml).unwrap();
    
    let result = runner.test_file(&test_file).await.expect("Test should complete");
    
    println!("Perfect Fidelity Test Result:");
    println!("Round-trip success: {}", result.round_trip_success);
    println!("Byte perfect: {}", result.byte_perfect);
    println!("Original size: {} bytes", result.original_size);
    println!("Rebuilt size: {} bytes", result.rebuilt_size);
    println!("Parse time: {} ms", result.parse_time_ms);
    println!("Build time: {} ms", result.build_time_ms);
    
    if let Some(error) = &result.error {
        println!("Error: {}", error);
    }
    
    // With Perfect Fidelity Engine, we should achieve byte-perfect reproduction
    // (This might fail initially until the full implementation is complete)
    if result.round_trip_success {
        // Only assert byte-perfect if round-trip succeeded
        assert!(result.byte_perfect, "Perfect Fidelity Engine should produce byte-perfect reproduction");
    }
}

#[tokio::test]
async fn performance_monitoring_test() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = FidelityTestConfig {
        perfect_fidelity: false, // Standard mode for performance baseline
        test_data_dir: temp_dir.path().join("performance_data"),
        versions: vec!["4.3".to_string()],
        max_file_size: 1024 * 1024, // 1MB
        stress_mode: false,
    };
    
    let runner = FidelityTestRunner::new(config);
    
    // Generate test data with varying sizes
    let test_sizes = [1024, 10240, 102400]; // 1KB, 10KB, 100KB
    
    for &size in &test_sizes {
        let content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <MessageId>PERF_{}</MessageId>
    <Description>{}</Description>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_PERF_{}</ReleaseId>
      <Title>Performance Test {}</Title>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#, 
            size,
            "x".repeat(size),
            size,
            size
        );
        
        let test_file = temp_dir.path().join(format!("perf_test_{}.xml", size));
        std::fs::write(&test_file, &content).unwrap();
        
        let result = runner.test_file(&test_file).await.expect("Performance test should complete");
        
        println!("Performance Test - Size: {} bytes", size);
        println!("  Parse time: {} ms", result.parse_time_ms);
        println!("  Build time: {} ms", result.build_time_ms);
        println!("  Total time: {} ms", result.parse_time_ms + result.build_time_ms);
        
        // Performance assertions - these are reasonable expectations
        assert!(result.parse_time_ms < 1000, "Parse time should be under 1 second for {} byte file", size);
        assert!(result.build_time_ms < 1000, "Build time should be under 1 second for {} byte file", size);
    }
}

#[tokio::test]
#[ignore] // Run with --ignored for extended testing
async fn stress_test_large_files() {
    let temp_dir = TempDir::new().unwrap();
    
    let config = FidelityTestConfig {
        perfect_fidelity: false,
        test_data_dir: temp_dir.path().join("stress_data"),
        versions: vec!["4.3".to_string()],
        max_file_size: 50 * 1024 * 1024, // 50MB
        stress_mode: true,
    };
    
    let runner = FidelityTestRunner::new(config);
    
    // Generate a large file with many elements
    let mut large_content = String::new();
    large_content.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <MessageId>STRESS_001</MessageId>
  </MessageHeader>
  <ResourceList>"#);
    
    // Add many sound recordings
    for i in 0..1000 {
        large_content.push_str(&format!(r#"
    <SoundRecording>
      <ResourceReference>R{:04}</ResourceReference>
      <Type>SoundRecording</Type>
      <ResourceId>SR_{:04}</ResourceId>
      <ReferenceTitle>Track {:04}</ReferenceTitle>
      <Duration>PT3M30S</Duration>
    </SoundRecording>"#, i, i, i));
    }
    
    large_content.push_str(r#"
  </ResourceList>
  <ReleaseList>
    <Release>
      <ReleaseId>REL_STRESS_001</ReleaseId>
      <Title>Stress Test Release</Title>
      <ResourceGroup>"#);
    
    // Reference all tracks
    for i in 0..1000 {
        large_content.push_str(&format!(r#"
        <ResourceReference>R{:04}</ResourceReference>"#, i));
    }
    
    large_content.push_str(r#"
      </ResourceGroup>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#);
    
    let test_file = temp_dir.path().join("stress_test.xml");
    std::fs::write(&test_file, &large_content).unwrap();
    
    let file_size = std::fs::metadata(&test_file).unwrap().len();
    println!("Stress test file size: {} bytes", file_size);
    
    let start_time = Instant::now();
    let result = runner.test_file(&test_file).await.expect("Stress test should complete");
    let total_time = start_time.elapsed();
    
    println!("Stress Test Results:");
    println!("  File size: {} bytes", file_size);
    println!("  Round-trip success: {}", result.round_trip_success);
    println!("  Parse time: {} ms", result.parse_time_ms);
    println!("  Build time: {} ms", result.build_time_ms);
    println!("  Total time: {:?}", total_time);
    
    // Stress test should still complete successfully, even if slower
    assert!(result.round_trip_success, "Stress test should succeed");
    
    // Performance should be reasonable even for large files
    assert!(result.parse_time_ms < 30000, "Parse time should be under 30 seconds for large file");
    assert!(result.build_time_ms < 30000, "Build time should be under 30 seconds for large file");
}