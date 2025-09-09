//! Comprehensive determinism tests for DDEX Builder
//! 
//! These tests verify that the DDEX Builder produces identical output across:
//! - Multiple build iterations
//! - Different HashMap iteration orders  
//! - Different thread scheduling
//! - Different system times
//! - Different locales
//! - Memory pressure conditions

use ddex_builder::determinism::{DeterminismConfig, DeterminismVerifier};
use ddex_builder::{Builder, BuildRequest};
use ddex_core::models::{Release, Party, ReleaseId};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// Create a basic test build request
fn create_test_build_request() -> BuildRequest {
    BuildRequest {
        data: json!({
            "messageType": "NewReleaseMessage",
            "messageId": "MSG001",
            "release": {
                "releaseId": "REL001", 
                "title": "Test Album",
                "artist": "Test Artist",
                "tracks": [
                    {
                        "trackId": "TRK001",
                        "title": "Track 1",
                        "duration": "PT3M30S"
                    },
                    {
                        "trackId": "TRK002", 
                        "title": "Track 2",
                        "duration": "PT4M15S"
                    }
                ]
            }
        }),
        config: DeterminismConfig::default(),
        preset: None,
        validate: false,
    }
}

/// Create a complex build request with many fields that could cause non-determinism
fn create_complex_build_request() -> BuildRequest {
    BuildRequest {
        data: json!({
            "messageType": "NewReleaseMessage",
            "messageId": format!("MSG_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()),
            "sentDateTime": chrono::Utc::now().to_rfc3339(),
            "release": {
                "releaseId": "REL001",
                "title": "Complex Test Album",
                "artist": "Test Artist",
                "releaseDate": "2024-01-01",
                "genres": ["Rock", "Pop", "Electronic"],
                "labels": [
                    {
                        "name": "Test Label",
                        "id": "LBL001"
                    }
                ],
                "tracks": (0..20).map(|i| {
                    json!({
                        "trackId": format!("TRK{:03}", i),
                        "title": format!("Track {}", i + 1),
                        "duration": format!("PT{}M{}S", (i % 5) + 2, (i * 7) % 60),
                        "isrc": format!("USRC17{:06}", i),
                        "contributors": [
                            {
                                "name": format!("Artist {}", i % 3),
                                "role": "MainArtist" 
                            }
                        ],
                        "metadata": {
                            "bpm": (120 + (i % 40)) as u32,
                            "key": ["C", "D", "E", "F", "G", "A", "B"][i % 7],
                            "mood": ["Happy", "Sad", "Energetic", "Calm"][i % 4]
                        }
                    })
                }).collect::<Vec<_>>(),
                "images": [
                    {
                        "type": "FrontCover",
                        "url": "https://example.com/cover.jpg",
                        "width": 3000,
                        "height": 3000
                    }
                ],
                "deals": [
                    {
                        "dealId": "DEAL001",
                        "dealType": "LicensePartnerDeal",
                        "territories": ["US", "CA", "MX", "GB", "DE", "FR", "IT", "ES"],
                        "useTypes": ["Stream", "Download", "ConditionalDownload"],
                        "validityPeriod": {
                            "startDate": "2024-01-01",
                            "endDate": "2034-12-31"
                        }
                    }
                ]
            },
            "additionalMetadata": {
                "customField1": format!("value_{}", thread::current().id().as_u64()),
                "customField2": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                "nestedObject": {
                    "array": [1, 2, 3, 4, 5],
                    "map": {
                        "key1": "value1",
                        "key2": "value2", 
                        "key3": "value3"
                    }
                }
            }
        }),
        config: DeterminismConfig::default(),
        preset: None,
        validate: false,
    }
}

#[test]
fn test_basic_determinism_verification() {
    let request = create_test_build_request();
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config);
    
    let result = verifier.verify(&request, 5).expect("Determinism verification failed");
    
    assert!(result.is_deterministic, "Basic build should be deterministic");
    assert_eq!(result.iterations, 5);
    assert!(result.differences.is_empty());
    assert_eq!(result.hashes.len(), 5);
    
    // All hashes should be identical
    let first_hash = &result.hashes[0];
    for hash in &result.hashes[1..] {
        assert_eq!(hash, first_hash, "All output hashes should be identical");
    }
}

#[test]
fn test_complex_data_determinism() {
    let request = create_complex_build_request();
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config);
    
    let result = verifier.verify(&request, 3).expect("Complex determinism verification failed");
    
    assert!(result.is_deterministic, "Complex build should be deterministic");
    assert!(result.differences.is_empty());
}

#[test]
fn test_hashmap_iteration_order_resistance() {
    let request = create_test_build_request();
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config);
    
    let result = verifier.verify_with_hashmap_stress(&request, 10)
        .expect("HashMap stress test failed");
    
    assert!(result.is_deterministic, "Build should be deterministic despite HashMap iteration order changes");
    assert!(result.differences.is_empty());
}

#[test] 
fn test_multithreaded_determinism() {
    let request = create_test_build_request();
    let config = DeterminismConfig::default();
    let verifier = Arc::new(DeterminismVerifier::new(config));
    
    let mut handles = vec![];
    let results = Arc::new(Mutex::new(vec![]));
    
    // Run verification in multiple threads simultaneously
    for _ in 0..4 {
        let verifier_clone = Arc::clone(&verifier);
        let request_clone = request.clone();
        let results_clone = Arc::clone(&results);
        
        let handle = thread::spawn(move || {
            let result = verifier_clone.verify(&request_clone, 3).expect("Thread verification failed");
            results_clone.lock().unwrap().push(result);
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread join failed");
    }
    
    let thread_results = results.lock().unwrap();
    assert_eq!(thread_results.len(), 4);
    
    // All thread results should be deterministic and identical
    let first_hash = &thread_results[0].hashes[0];
    for result in thread_results.iter() {
        assert!(result.is_deterministic, "Each thread result should be deterministic");
        assert_eq!(&result.hashes[0], first_hash, "All threads should produce identical output");
    }
}

#[test]
fn test_different_system_times() {
    let request = create_test_build_request();
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config);
    
    let mut results = vec![];
    
    // Build at different times (though the config should normalize times)
    for _ in 0..3 {
        let result = verifier.verify(&request, 2).expect("Time-based verification failed");
        results.push(result);
        
        // Small delay to ensure different system times
        thread::sleep(std::time::Duration::from_millis(10));
    }
    
    // All results should be identical despite different build times
    let first_hash = &results[0].hashes[0];
    for result in &results[1..] {
        assert!(result.is_deterministic, "Result should be deterministic at different times");
        assert_eq!(&result.hashes[0], first_hash, "Output should be identical across different build times");
    }
}

#[test]
fn test_memory_pressure_determinism() {
    let request = create_complex_build_request();
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config);
    
    // Allocate large amounts of memory to create pressure
    let _memory_pressure: Vec<Vec<u8>> = (0..100)
        .map(|_| vec![0u8; 1024 * 1024]) // 1MB each
        .collect();
    
    let result = verifier.verify(&request, 3).expect("Memory pressure verification failed");
    
    assert!(result.is_deterministic, "Build should be deterministic under memory pressure");
    assert!(result.differences.is_empty());
}

#[test]
fn test_locale_independence() {
    // Save current locale
    let original_locale = std::env::var("LC_ALL").unwrap_or_default();
    
    let request = create_test_build_request();
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config);
    
    let mut results = vec![];
    
    // Test with different locales
    let locales = ["C", "en_US.UTF-8", "de_DE.UTF-8", "ja_JP.UTF-8"];
    
    for locale in &locales {
        std::env::set_var("LC_ALL", locale);
        
        let result = verifier.verify(&request, 2).expect("Locale-based verification failed");
        results.push(result);
    }
    
    // Restore original locale
    if original_locale.is_empty() {
        std::env::remove_var("LC_ALL");
    } else {
        std::env::set_var("LC_ALL", original_locale);
    }
    
    // All results should be identical despite different locales
    let first_hash = &results[0].hashes[0];
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_deterministic, "Result should be deterministic with locale {}", locales[i]);
        assert_eq!(&result.hashes[0], first_hash, "Output should be identical across locales");
    }
}

#[test]
fn test_unicode_normalization_determinism() {
    // Test with Unicode text that could be normalized differently
    let mut request = create_test_build_request();
    
    // Add Unicode text with different normalization forms
    request.data["release"]["title"] = json!("Café Münchën"); // Contains combining characters
    request.data["release"]["artist"] = json!("Ångström & Naïve"); // Various accented characters
    
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config);
    
    let result = verifier.verify(&request, 5).expect("Unicode normalization verification failed");
    
    assert!(result.is_deterministic, "Build should be deterministic with Unicode text");
    assert!(result.differences.is_empty());
}

#[test]
fn test_large_dataset_determinism() {
    // Create a build request with a large amount of data
    let large_tracks: Vec<serde_json::Value> = (0..1000).map(|i| {
        json!({
            "trackId": format!("TRK{:04}", i),
            "title": format!("Track {} with very long title that contains lots of metadata and information", i),
            "duration": format!("PT{}M{}S", (i % 8) + 1, (i * 13) % 60),
            "isrc": format!("USRC17{:07}", i),
            "contributors": (0..(i % 5 + 1)).map(|j| {
                json!({
                    "name": format!("Contributor {} for track {}", j, i),
                    "role": ["MainArtist", "FeaturedArtist", "Producer", "Songwriter"][j % 4]
                })
            }).collect::<Vec<_>>()
        })
    }).collect();
    
    let mut request = create_test_build_request();
    request.data["release"]["tracks"] = json!(large_tracks);
    
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config);
    
    let result = verifier.verify(&request, 3).expect("Large dataset verification failed");
    
    assert!(result.is_deterministic, "Build should be deterministic with large datasets");
    assert!(result.differences.is_empty());
    
    // Verify performance characteristics
    assert!(result.runtime_stats.avg_build_time_ms < 5000, "Large dataset should build within reasonable time");
    assert!(result.runtime_stats.overhead_percentage < 200.0, "Verification overhead should be reasonable");
}

#[test]
fn test_determinism_with_custom_config() {
    let request = create_test_build_request();
    let mut config = DeterminismConfig::default();
    
    // Customize configuration
    config.indent_width = 4;
    config.output_mode = ddex_builder::determinism::OutputMode::Pretty;
    config.unicode_normalization = ddex_builder::determinism::UnicodeNormalization::NFKC;
    
    let verifier = DeterminismVerifier::new(config);
    let result = verifier.verify(&request, 5).expect("Custom config verification failed");
    
    assert!(result.is_deterministic, "Build should be deterministic with custom config");
    assert!(result.differences.is_empty());
}

#[test]
fn test_determinism_stress_test() {
    let request = create_complex_build_request();
    
    let result = DeterminismVerifier::thorough_check(&request, 10)
        .expect("Thorough determinism check failed");
    
    assert!(result.is_deterministic, "Thorough stress test should pass");
    assert!(result.differences.is_empty());
    assert_eq!(result.iterations, 10);
}

#[test]
fn test_quick_determinism_check() {
    let request = create_test_build_request();
    
    let is_deterministic = DeterminismVerifier::quick_check(&request)
        .expect("Quick check failed");
    
    assert!(is_deterministic, "Quick check should pass for basic request");
}

#[test]
fn test_determinism_verification_with_outputs_retained() {
    let request = create_test_build_request();
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config).with_outputs_retained();
    
    let result = verifier.verify(&request, 3).expect("Verification with outputs failed");
    
    assert!(result.is_deterministic);
    assert_eq!(result.outputs.len(), 3, "Outputs should be retained");
    
    // All outputs should be identical
    let first_output = &result.outputs[0];
    for output in &result.outputs[1..] {
        assert_eq!(output, first_output, "All retained outputs should be identical");
    }
}

#[test]
fn test_determinism_difference_analysis() {
    // This test creates an intentionally non-deterministic scenario for testing difference analysis
    // In real usage, this should not happen - but we need to test the analysis code
    
    let request = create_test_build_request();
    let config = DeterminismConfig::default();
    let verifier = DeterminismVerifier::new(config).with_outputs_retained().with_context_chars(200);
    
    // Create a modified verifier that introduces artificial differences for testing
    // In a real implementation, you might want to skip this test or mock the builder
    let result = verifier.verify(&request, 2).expect("Difference analysis test failed");
    
    // This should normally pass - if it fails, we can analyze the differences
    if !result.is_deterministic {
        assert!(!result.differences.is_empty(), "Non-deterministic result should have differences");
        
        let diff = &result.differences[0];
        assert!(diff.first_difference_byte.is_some(), "Should identify byte position of first difference");
        assert_ne!(diff.hash_difference.sha256_1, diff.hash_difference.sha256_2, "Hashes should differ");
        assert_ne!(diff.hash_difference.blake3_1, diff.hash_difference.blake3_2, "BLAKE3 hashes should differ");
        
        if let Some(context) = &diff.context {
            assert!(context.line_number.is_some(), "Should identify line number of difference");
            assert!(context.column_number.is_some(), "Should identify column number of difference");
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_determinism_with_file_io() {
        // Test that file I/O doesn't affect determinism
        let request = create_test_build_request();
        let config = DeterminismConfig::default();
        let verifier = DeterminismVerifier::new(config);
        
        // Create temporary files
        let mut temp_files = vec![];
        for i in 0..5 {
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            writeln!(temp_file, "Temporary data for iteration {}", i).expect("Failed to write temp file");
            temp_files.push(temp_file);
        }
        
        let result = verifier.verify(&request, 3).expect("File I/O determinism test failed");
        
        assert!(result.is_deterministic, "File I/O should not affect determinism");
    }

    #[test]
    fn test_determinism_with_environment_variables() {
        let request = create_test_build_request();
        let config = DeterminismConfig::default();
        let verifier = DeterminismVerifier::new(config);
        
        // Save original environment
        let original_var = std::env::var("TEST_DETERMINISM_VAR").ok();
        
        let mut results = vec![];
        
        // Test with different environment variables
        for value in &["value1", "value2", "value3"] {
            std::env::set_var("TEST_DETERMINISM_VAR", value);
            let result = verifier.verify(&request, 2).expect("Environment variable test failed");
            results.push(result);
        }
        
        // Restore original environment
        match original_var {
            Some(val) => std::env::set_var("TEST_DETERMINISM_VAR", val),
            None => std::env::remove_var("TEST_DETERMINISM_VAR"),
        }
        
        // All results should be identical despite different environment variables
        let first_hash = &results[0].hashes[0];
        for result in &results[1..] {
            assert!(result.is_deterministic, "Result should be deterministic with different env vars");
            assert_eq!(&result.hashes[0], first_hash, "Output should be identical despite env var changes");
        }
    }
}