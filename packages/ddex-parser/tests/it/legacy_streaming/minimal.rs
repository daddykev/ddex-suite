// Integration test demonstrating working streaming parser

use ddex_core::models::versions::ERNVersion;
use ddex_parser::streaming::minimal::{MinimalElement, MinimalStreamIterator};
use std::io::Cursor;

#[test]
fn test_streaming_parser_integration() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ERNMessage xmlns="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>test-message-1</MessageId>
        <MessageCreatedDateTime>2023-01-01T00:00:00</MessageCreatedDateTime>
    </MessageHeader>
    <Release>
        <ReleaseReference>REL001</ReleaseReference>
        <ReleaseTitle>Test Release</ReleaseTitle>
    </Release>
    <Resource>
        <ResourceReference>RES001</ResourceReference>
        <Title>Test Resource</Title>
    </Resource>
</ERNMessage>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let iterator = MinimalStreamIterator::new(cursor, ERNVersion::V4_3);

    let elements: Result<Vec<_>, _> = iterator.collect();
    assert!(elements.is_ok(), "Streaming parser should not fail");

    let elements = elements.unwrap();
    println!("Parsed {} elements", elements.len());

    // Check we got the expected elements
    let has_header = elements
        .iter()
        .any(|e| matches!(e, MinimalElement::Header { .. }));
    let has_release = elements
        .iter()
        .any(|e| matches!(e, MinimalElement::Release { .. }));
    let has_resource = elements
        .iter()
        .any(|e| matches!(e, MinimalElement::Resource { .. }));
    let has_end_stream = elements
        .iter()
        .any(|e| matches!(e, MinimalElement::EndOfStream));

    assert!(has_header, "Should parse message header");
    assert!(has_release, "Should parse release");
    assert!(has_resource, "Should parse resource");
    assert!(has_end_stream, "Should have end of stream marker");

    println!(
        "✅ Streaming parser successfully parsed DDEX XML with {} elements",
        elements.len()
    );
}

#[test]
fn test_streaming_security_limits() {
    // Test the security limits work
    let mut xml = String::from(r#"<?xml version="1.0"?>"#);
    for i in 0..150 {
        xml.push_str(&format!("<level{}>", i));
    }
    xml.push_str("content");
    for i in (0..150).rev() {
        xml.push_str(&format!("</level{}>", i));
    }

    let cursor = Cursor::new(xml.as_bytes());
    let mut iterator = MinimalStreamIterator::new(cursor, ERNVersion::V4_3);

    // Should get a security violation
    let result = iterator.next();
    assert!(result.is_some(), "Should get a result");
    match result.unwrap() {
        Err(_) => {
            println!("✅ Security limits correctly triggered");
        }
        Ok(_) => panic!("Expected security violation for deep nesting"),
    }
}
