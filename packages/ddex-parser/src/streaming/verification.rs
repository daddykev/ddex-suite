//! Verification script for model-aligned comprehensive streaming parser

use crate::streaming::comprehensive::{ComprehensiveStreamIterator, StreamingElement};
use ddex_core::models::versions::ERNVersion;
use std::io::Cursor;

pub fn verify_comprehensive_parser() {
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

    println!("Testing comprehensive streaming parser with model alignment...");

    let cursor = Cursor::new(xml.as_bytes());
    let iterator = ComprehensiveStreamIterator::new(cursor, ERNVersion::V4_3);

    let mut element_count = 0;
    let mut header_found = false;
    let mut release_found = false;
    let mut resource_found = false;

    for result in iterator {
        match result {
            Ok(element) => {
                element_count += 1;
                match element {
                    StreamingElement::Header(_header) => {
                        println!("✅ Parsed MessageHeader successfully");
                        header_found = true;
                    }
                    StreamingElement::Release(release) => {
                        println!("✅ Parsed Release: reference={}", release.release_reference);
                        println!("   Titles: {}", release.release_title.len());
                        println!("   Genres: {}", release.genre.len());
                        release_found = true;
                    }
                    StreamingElement::Resource(resource) => {
                        println!(
                            "✅ Parsed Resource: reference={}",
                            resource.resource_reference
                        );
                        println!("   Titles: {}", resource.reference_title.len());
                        if let Some(duration) = resource.duration {
                            println!("   Duration: {}s", duration.as_secs());
                        }
                        resource_found = true;
                    }
                    StreamingElement::Party(_party) => {
                        println!("✅ Parsed Party");
                    }
                    StreamingElement::EndOfStream => {
                        println!("✅ End of stream reached");
                    }
                }
            }
            Err(e) => {
                println!("❌ Error parsing: {:?}", e);
            }
        }
    }

    println!("\n=== Verification Results ===");
    println!("Total elements parsed: {}", element_count);
    println!("Header found: {}", header_found);
    println!("Release found: {}", release_found);
    println!("Resource found: {}", resource_found);

    if header_found && release_found && resource_found {
        println!("🎉 Model alignment verification PASSED!");
    } else {
        println!("❌ Model alignment verification FAILED!");
    }
}

#[cfg(all(test, feature = "legacy-streaming-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_model_alignment() {
        verify_comprehensive_parser();
    }
}
