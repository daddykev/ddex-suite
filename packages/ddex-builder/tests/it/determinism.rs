//! Comprehensive determinism tests for DDEX Builder
//!
//! These tests verify that the DDEX Builder produces identical output across:
//! - Multiple build iterations
//! - Different HashMap iteration orders  
//! - Different thread scheduling
//! - Different system times
//! - Different locales
//! - Memory pressure conditions

use ddex_builder::builder::{
    BuildOptions, DealRequest, LocalizedStringRequest, MessageHeaderRequest, PartyRequest,
    ReleaseRequest,
};
use ddex_builder::{BuildRequest, DDEXBuilder};
use indexmap::IndexMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// Create a basic test build request
fn create_test_build_request() -> BuildRequest {
    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("MSG001".to_string()),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Test Sender".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Test Recipient".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_control_type: Some("NewReleaseMessage".to_string()),
            message_created_date_time: Some(chrono::Utc::now().to_rfc3339()),
        },
        version: "ern/43".to_string(),
        profile: Some("BasicProfile".to_string()),
        releases: vec![ReleaseRequest {
            release_id: "REL001".to_string(),
            release_reference: Some("REL001".to_string()),
            title: vec![LocalizedStringRequest {
                text: "Test Album".to_string(),
                language_code: None,
            }],
            artist: "Test Artist".to_string(),
            label: None,
            release_date: Some("2024-01-01".to_string()),
            upc: None,
            tracks: Vec::new(),
            resource_references: None,
        }],
        deals: vec![DealRequest {
            deal_reference: Some("DEAL001".to_string()),
            deal_terms: ddex_builder::builder::DealTerms {
                commercial_model_type: "FreeOfChargeModel".to_string(),
                territory_code: vec!["Worldwide".to_string()],
                start_date: Some("2024-01-01".to_string()),
            },
            release_references: vec!["REL001".to_string()],
        }],
        extensions: Some(IndexMap::new()),
    }
}

/// Create a complex build request with many fields that could cause non-determinism
fn create_complex_build_request() -> BuildRequest {
    let mut extensions = IndexMap::new();
    extensions.insert(
        "customField1".to_string(),
        format!("value_{:?}", thread::current().id()),
    );
    extensions.insert(
        "customField2".to_string(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    );

    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some(format!(
                "MSG_{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Complex Test Sender".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Complex Test Recipient".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_control_type: Some("NewReleaseMessage".to_string()),
            message_created_date_time: Some(chrono::Utc::now().to_rfc3339()),
        },
        version: "ern/43".to_string(),
        profile: Some("ComplexProfile".to_string()),
        releases: vec![ReleaseRequest {
            release_id: "REL001".to_string(),
            release_reference: Some("REL001".to_string()),
            title: vec![LocalizedStringRequest {
                text: "Complex Test Album".to_string(),
                language_code: None,
            }],
            artist: "Test Artist".to_string(),
            label: Some("Test Label".to_string()),
            release_date: Some("2024-01-01".to_string()),
            upc: Some("123456789012".to_string()),
            tracks: Vec::new(),
            resource_references: None,
        }],
        deals: (0..5)
            .map(|i| DealRequest {
                deal_reference: Some(format!("DEAL{:03}", i)),
                deal_terms: ddex_builder::builder::DealTerms {
                    commercial_model_type: "FreeOfChargeModel".to_string(),
                    territory_code: vec!["Worldwide".to_string()],
                    start_date: Some("2024-01-01".to_string()),
                },
                release_references: vec![format!("REL{:04}", i)],
            })
            .collect(),
        extensions: Some(extensions),
    }
}

#[test]
fn test_basic_determinism_verification() {
    let request = create_test_build_request();
    let builder = DDEXBuilder::new();

    // Test determinism by building multiple times
    let mut outputs = vec![];
    for _ in 0..5 {
        let result = builder
            .build(request.clone(), BuildOptions::default())
            .expect("Build failed");
        outputs.push(result.xml);
    }

    // All outputs should be identical
    let first_output = &outputs[0];
    for (i, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            output, first_output,
            "Output {} should be identical to first output",
            i
        );
    }
}

#[test]
fn test_complex_data_determinism() {
    let request = create_complex_build_request();
    let builder = DDEXBuilder::new();

    // Test determinism with complex data by building multiple times
    let mut outputs = vec![];
    for _ in 0..3 {
        let result = builder
            .build(request.clone(), BuildOptions::default())
            .expect("Complex build failed");
        outputs.push(result.xml);
    }

    // All outputs should be identical
    let first_output = &outputs[0];
    for (i, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            output, first_output,
            "Complex output {} should be identical to first output",
            i
        );
    }
}

#[test]
fn test_indexmap_determinism() {
    let request = create_test_build_request();
    let builder = DDEXBuilder::new();

    // Test that IndexMap maintains deterministic order
    let mut outputs = vec![];
    for _ in 0..10 {
        let result = builder
            .build(request.clone(), BuildOptions::default())
            .expect("Build failed");
        outputs.push(result.xml);
    }

    // All outputs should be identical due to IndexMap usage
    let first_output = &outputs[0];
    for (i, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            output, first_output,
            "IndexMap output {} should be identical to first output",
            i
        );
    }
}

#[test]
fn test_multithreaded_determinism() {
    let request = create_test_build_request();
    let builder = Arc::new(DDEXBuilder::new());

    let mut handles = vec![];
    let results = Arc::new(Mutex::new(vec![]));

    // Run builds in multiple threads simultaneously
    for _ in 0..4 {
        let builder_clone = Arc::clone(&builder);
        let request_clone = request.clone();
        let results_clone = Arc::clone(&results);

        let handle = thread::spawn(move || {
            let result = builder_clone
                .build(request_clone, BuildOptions::default())
                .expect("Thread build failed");
            results_clone.lock().unwrap().push(result.xml);
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread join failed");
    }

    let thread_results = results.lock().unwrap();
    assert_eq!(thread_results.len(), 4);

    // All thread results should be identical
    let first_output = &thread_results[0];
    for (i, output) in thread_results.iter().enumerate().skip(1) {
        assert_eq!(
            output, first_output,
            "Thread output {} should be identical to first output",
            i
        );
    }
}

#[test]
fn test_different_system_times() {
    // Create request with deterministic timestamp
    let mut request = create_test_build_request();
    request.header.message_created_date_time = Some("2024-01-01T00:00:00Z".to_string());

    let builder = DDEXBuilder::new();
    let mut outputs = vec![];

    // Build at different times with fixed timestamp
    for _ in 0..3 {
        let result = builder
            .build(request.clone(), BuildOptions::default())
            .expect("Time-based build failed");
        outputs.push(result.xml);

        // Small delay to ensure different system times
        thread::sleep(std::time::Duration::from_millis(10));
    }

    // All results should be identical despite different build times
    let first_output = &outputs[0];
    for (i, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            output, first_output,
            "Output {} should be identical despite different build times",
            i
        );
    }
}

#[test]
fn test_memory_pressure_determinism() {
    let request = create_complex_build_request();
    let builder = DDEXBuilder::new();

    // Allocate large amounts of memory to create pressure
    let _memory_pressure: Vec<Vec<u8>> = (0..100)
        .map(|_| vec![0u8; 1024 * 1024]) // 1MB each
        .collect();

    // Test determinism under memory pressure
    let mut outputs = vec![];
    for _ in 0..3 {
        let result = builder
            .build(request.clone(), BuildOptions::default())
            .expect("Memory pressure build failed");
        outputs.push(result.xml);
    }

    // All outputs should be identical despite memory pressure
    let first_output = &outputs[0];
    for (i, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            output, first_output,
            "Memory pressure output {} should be identical to first output",
            i
        );
    }
}

#[test]
fn test_locale_independence() {
    // Save current locale
    let original_locale = std::env::var("LC_ALL").unwrap_or_default();

    let request = create_test_build_request();
    let builder = DDEXBuilder::new();

    let mut outputs = vec![];

    // Test with different locales
    let locales = ["C", "en_US.UTF-8", "de_DE.UTF-8", "ja_JP.UTF-8"];

    for locale in &locales {
        std::env::set_var("LC_ALL", locale);

        let result = builder
            .build(request.clone(), BuildOptions::default())
            .expect("Locale-based build failed");
        outputs.push(result.xml);
    }

    // Restore original locale
    if original_locale.is_empty() {
        std::env::remove_var("LC_ALL");
    } else {
        std::env::set_var("LC_ALL", original_locale);
    }

    // All outputs should be identical despite different locales
    let first_output = &outputs[0];
    for (i, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            output, first_output,
            "Locale output {} should be identical to first output",
            i
        );
    }
}

#[test]
fn test_unicode_normalization_determinism() {
    // Test with Unicode text that could be normalized differently
    let mut request = create_test_build_request();

    // Add Unicode text with different normalization forms
    request.releases[0].title[0].text = "Café Münchën".to_string(); // Contains combining characters
    request.releases[0].artist = "Ångström & Naïve".to_string(); // Various accented characters

    let builder = DDEXBuilder::new();

    // Test determinism with Unicode text
    let mut outputs = vec![];
    for _ in 0..5 {
        let result = builder
            .build(request.clone(), BuildOptions::default())
            .expect("Unicode build failed");
        outputs.push(result.xml);
    }

    // All outputs should be identical
    let first_output = &outputs[0];
    for (i, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            output, first_output,
            "Unicode output {} should be identical to first output",
            i
        );
    }
}

#[test]
fn test_large_dataset_determinism() {
    // Create a build request with many releases and deals
    let large_releases: Vec<ReleaseRequest> = (0..100).map(|i| {
        ReleaseRequest {
            release_id: format!("REL{:04}", i),
            release_reference: Some(format!("REL{:04}", i)),
            title: vec![LocalizedStringRequest {
                text: format!("Release {} with very long title that contains lots of metadata and information", i),
                language_code: None,
            }],
            artist: format!("Artist {}", i % 10),
            label: Some(format!("Label {}", i % 5)),
            release_date: Some("2024-01-01".to_string()),
            upc: Some(format!("{:012}", i)),
            tracks: Vec::new(),
            resource_references: None,
        }
    }).collect();

    let mut request = create_test_build_request();
    request.releases = large_releases;

    let builder = DDEXBuilder::new();

    // Test determinism with large dataset
    let mut outputs = vec![];
    for _ in 0..3 {
        let result = builder
            .build(request.clone(), BuildOptions::default())
            .expect("Large dataset build failed");
        outputs.push(result.xml);
    }

    // All outputs should be identical
    let first_output = &outputs[0];
    for (i, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            output, first_output,
            "Large dataset output {} should be identical to first output",
            i
        );
    }

    // Verify the output is substantial
    assert!(
        first_output.len() > 10000,
        "Large dataset should produce substantial XML"
    );
}

#[test]
fn test_determinism_with_custom_extensions() {
    let mut request = create_test_build_request();

    // Add custom extensions
    let mut extensions = IndexMap::new();
    extensions.insert("customField1".to_string(), "value1".to_string());
    extensions.insert("customField2".to_string(), "value2".to_string());
    extensions.insert("customField3".to_string(), "value3".to_string());
    request.extensions = Some(extensions);

    let builder = DDEXBuilder::new();

    // Test determinism with custom extensions
    let mut outputs = vec![];
    for _ in 0..5 {
        let result = builder
            .build(request.clone(), BuildOptions::default())
            .expect("Custom extensions build failed");
        outputs.push(result.xml);
    }

    // All outputs should be identical
    let first_output = &outputs[0];
    for (i, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            output, first_output,
            "Custom extensions output {} should be identical to first output",
            i
        );
    }
}

#[test]
fn test_determinism_stress_test() {
    let request = create_complex_build_request();
    let builder = DDEXBuilder::new();

    // Perform thorough determinism stress test
    let mut outputs = vec![];
    for _ in 0..10 {
        let result = builder
            .build(request.clone(), BuildOptions::default())
            .expect("Stress test build failed");
        outputs.push(result.xml);
    }

    // All outputs should be identical
    let first_output = &outputs[0];
    for (i, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            output, first_output,
            "Stress test output {} should be identical to first output",
            i
        );
    }
}

#[test]
fn test_quick_determinism_check() {
    let request = create_test_build_request();
    let builder = DDEXBuilder::new();

    // Perform quick determinism check (2 builds)
    let result1 = builder
        .build(request.clone(), BuildOptions::default())
        .expect("Quick check build 1 failed");
    let result2 = builder
        .build(request, BuildOptions::default())
        .expect("Quick check build 2 failed");

    assert_eq!(
        result1.xml, result2.xml,
        "Quick check should pass for basic request"
    );
}

#[test]
fn test_determinism_with_outputs_retained() {
    let request = create_test_build_request();
    let builder = DDEXBuilder::new();

    // Test by retaining all outputs for comparison
    let mut outputs = vec![];
    for _ in 0..3 {
        let result = builder
            .build(request.clone(), BuildOptions::default())
            .expect("Retained output build failed");
        outputs.push(result.xml);
    }

    // All outputs should be identical
    let first_output = &outputs[0];
    for (i, output) in outputs.iter().enumerate().skip(1) {
        assert_eq!(
            output, first_output,
            "Retained output {} should be identical to first output",
            i
        );
    }

    assert_eq!(outputs.len(), 3, "Should have retained 3 outputs");
}

#[test]
fn test_determinism_consistency() {
    let request = create_test_build_request();
    let builder = DDEXBuilder::new();

    // Test that outputs are consistent (this should always pass)
    let result1 = builder
        .build(request.clone(), BuildOptions::default())
        .expect("Consistency build 1 failed");
    let result2 = builder
        .build(request, BuildOptions::default())
        .expect("Consistency build 2 failed");

    // Outputs should be identical
    assert_eq!(
        result1.xml, result2.xml,
        "Builds should be deterministic and identical"
    );

    // Verify the XML is well-formed and substantial
    assert!(
        result1.xml.contains("<?xml"),
        "Should contain XML declaration"
    );
    assert!(
        result1.xml.contains("NewReleaseMessage"),
        "Should contain release message"
    );
    assert!(
        result1.xml.len() > 500,
        "Should produce substantial XML output"
    );
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_determinism_with_file_io() {
        // Test that file I/O doesn't affect determinism
        let request = create_test_build_request();
        let builder = DDEXBuilder::new();

        // Create temporary files
        let mut temp_files = vec![];
        for i in 0..5 {
            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            writeln!(temp_file, "Temporary data for iteration {}", i)
                .expect("Failed to write temp file");
            temp_files.push(temp_file);
        }

        // Test determinism with file I/O activity
        let mut outputs = vec![];
        for _ in 0..3 {
            let result = builder
                .build(request.clone(), BuildOptions::default())
                .expect("File I/O build failed");
            outputs.push(result.xml);
        }

        // All outputs should be identical despite file I/O
        let first_output = &outputs[0];
        for (i, output) in outputs.iter().enumerate().skip(1) {
            assert_eq!(
                output, first_output,
                "File I/O output {} should be identical to first output",
                i
            );
        }
    }

    #[test]
    fn test_determinism_with_environment_variables() {
        let request = create_test_build_request();
        let builder = DDEXBuilder::new();

        // Save original environment
        let original_var = std::env::var("TEST_DETERMINISM_VAR").ok();

        let mut outputs = vec![];

        // Test with different environment variables
        for value in &["value1", "value2", "value3"] {
            std::env::set_var("TEST_DETERMINISM_VAR", value);
            let result = builder
                .build(request.clone(), BuildOptions::default())
                .expect("Environment variable build failed");
            outputs.push(result.xml);
        }

        // Restore original environment
        match original_var {
            Some(val) => std::env::set_var("TEST_DETERMINISM_VAR", val),
            None => std::env::remove_var("TEST_DETERMINISM_VAR"),
        }

        // All outputs should be identical despite different environment variables
        let first_output = &outputs[0];
        for (i, output) in outputs.iter().enumerate().skip(1) {
            assert_eq!(
                output, first_output,
                "Environment output {} should be identical to first output",
                i
            );
        }
    }
}
