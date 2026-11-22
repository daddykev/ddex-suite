//! Isolated test for comprehensive streaming parser with model alignment

use ddex_core::models::versions::ERNVersion;
use ddex_parser::streaming::comprehensive::{ComprehensiveStreamIterator, StreamingElement};
use std::io::Cursor;

#[test]
fn test_comprehensive_model_alignment() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ERNMessage xmlns="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>test-message-1</MessageId>
        <MessageCreatedDateTime>2023-01-01T00:00:00</MessageCreatedDateTime>
    </MessageHeader>
    <Release ReleaseReference="REL001">
        <ReleaseTitle>Test Release</ReleaseTitle>
        <Genre>Rock</Genre>
    </Release>
    <Resource ResourceReference="RES001">
        <Title>Test Resource</Title>
        <Duration>180</Duration>
    </Resource>
</ERNMessage>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let iterator = ComprehensiveStreamIterator::new(cursor, ERNVersion::V4_3);

    let elements: Result<Vec<_>, _> = iterator.collect();
    assert!(
        elements.is_ok(),
        "Parser should not fail: {:?}",
        elements.err()
    );

    let elements = elements.unwrap();
    assert!(
        elements.len() >= 3,
        "Should have at least 3 elements, got {}",
        elements.len()
    );

    let mut header_found = false;
    let mut release_found = false;
    let mut resource_found = false;

    for element in &elements {
        match element {
            StreamingElement::Header(header) => {
                println!("✅ Header found with message_id: {:?}", header.message_id);
                header_found = true;
            }
            StreamingElement::Release(release) => {
                println!("✅ Release found: {}", release.release_reference);
                println!("   Titles count: {}", release.release_title.len());
                println!("   Genres count: {}", release.genre.len());
                release_found = true;
            }
            StreamingElement::Resource(resource) => {
                println!("✅ Resource found: {}", resource.resource_reference);
                println!("   Title count: {}", resource.reference_title.len());
                if let Some(duration) = resource.duration {
                    println!("   Duration: {}s", duration.as_secs());
                }
                resource_found = true;
            }
            StreamingElement::Party(_) => {
                println!("✅ Party found");
            }
            StreamingElement::EndOfStream => {
                println!("✅ End of stream");
            }
        }
    }

    assert!(header_found, "Should find message header");
    assert!(release_found, "Should find release");
    assert!(resource_found, "Should find resource");

    println!("🎉 Comprehensive parser model alignment test PASSED!");
}
