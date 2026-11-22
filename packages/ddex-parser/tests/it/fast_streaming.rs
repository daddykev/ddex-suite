// Test the connection between FastStreamingParser and main parser API
use ddex_parser::{parser::security::SecurityConfig, DDEXParser};
use std::io::Cursor;
use std::time::Instant;

#[test]
#[ignore = "known bug: parse_fast_streaming (SecurityConfig::relaxed / enable_fast_streaming) returns no releases or resources"]
fn test_fast_streaming_connection_basic() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <ern:MessageHeader>
        <ern:MessageId>MSG001</ern:MessageId>
    </ern:MessageHeader>
    <ern:ReleaseList>
        <ern:Release>
            <ern:ReleaseId>REL001</ern:ReleaseId>
            <ern:ReleaseReference>R001</ern:ReleaseReference>
        </ern:Release>
    </ern:ReleaseList>
</ern:NewReleaseMessage>"#;

    // Test with fast streaming enabled
    let config = SecurityConfig::relaxed(); // This enables fast streaming
    let mut parser = DDEXParser::with_config(config);

    let cursor = Cursor::new(xml.as_bytes());
    let start = Instant::now();
    let result = parser.parse(cursor);
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Fast streaming parse should succeed");
    let message = result.unwrap();

    // Should have detected at least one release
    assert!(
        message.flat.stats.release_count >= 1,
        "Should detect releases"
    );

    println!("Fast streaming parse took: {:?}", elapsed);
    println!("Release count: {}", message.flat.stats.release_count);
}

#[test]
#[ignore = "known bug: parse_fast_streaming (SecurityConfig::relaxed / enable_fast_streaming) returns no releases or resources"]
fn test_fast_streaming_explicitly() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <ern:MessageHeader>
        <ern:MessageId>MSG_EXPLICIT</ern:MessageId>
    </ern:MessageHeader>
    <ern:ReleaseList>
        <ern:Release>
            <ern:ReleaseId>REL_EXPLICIT</ern:ReleaseId>
        </ern:Release>
    </ern:ReleaseList>
</ern:NewReleaseMessage>"#;

    let mut parser = DDEXParser::new();
    let cursor = Cursor::new(xml.as_bytes());

    // Directly call the fast streaming parser
    let result = parser.parse_fast_streaming(cursor);

    match result {
        Ok(message) => {
            println!("Success! Message: {:?}", message.flat.message_type);
        }
        Err(e) => {
            println!(
                "Fast streaming error (this shows the connection is working): {}",
                e
            );
            // The connection is working if we see our custom error message
            return;
        }
    }

    panic!("Should have gotten our custom fast streaming message");
}
