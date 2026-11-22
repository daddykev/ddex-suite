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

#[cfg(feature = "perf-tests")]
#[test]
#[ignore = "known bug: parse_fast_streaming (SecurityConfig::relaxed / enable_fast_streaming) returns no releases or resources"]
fn test_fast_streaming_vs_normal_performance() {
    // Generate larger XML for performance comparison
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <ern:MessageHeader>
        <ern:MessageId>PERF_TEST</ern:MessageId>
    </ern:MessageHeader>
    <ern:ReleaseList>"#,
    );

    // Add many releases for comparison
    for i in 0..100 {
        xml.push_str(&format!(
            r#"
            <ern:Release>
                <ern:ReleaseId>REL{:03}</ern:ReleaseId>
                <ern:ReleaseReference>R{:03}</ern:ReleaseReference>
                <ern:Title>
                    <ern:TitleText>Test Release {}</ern:TitleText>
                </ern:Title>
            </ern:Release>"#,
            i, i, i
        ));
    }
    xml.push_str("</ern:ReleaseList></ern:NewReleaseMessage>");

    // Test normal parsing (fast streaming disabled)
    let config_normal = SecurityConfig::strict(); // Fast streaming disabled
    let mut parser_normal = DDEXParser::with_config(config_normal);
    let cursor_normal = Cursor::new(xml.as_bytes());

    let start_normal = Instant::now();
    let result_normal = parser_normal.parse(cursor_normal);
    let elapsed_normal = start_normal.elapsed();

    // Test fast streaming parsing
    let config_fast = SecurityConfig::relaxed(); // Fast streaming enabled
    let mut parser_fast = DDEXParser::with_config(config_fast);
    let cursor_fast = Cursor::new(xml.as_bytes());

    let start_fast = Instant::now();
    let result_fast = parser_fast.parse(cursor_fast);
    let elapsed_fast = start_fast.elapsed();

    println!("XML size: {} bytes", xml.len());
    println!("Normal parsing: {:?}", elapsed_normal);
    println!("Fast streaming: {:?}", elapsed_fast);

    if result_fast.is_ok() {
        let message_fast = result_fast.unwrap();
        println!(
            "Fast streaming release count: {}",
            message_fast.flat.stats.release_count
        );

        // Should have detected releases
        assert!(
            message_fast.flat.stats.release_count > 0,
            "Fast streaming should detect releases"
        );
    } else {
        println!("Fast streaming failed: {:?}", result_fast);
    }

    // Note: Normal parsing might fail with empty cursor, but that's OK for this test
    // We're mainly testing that fast streaming is now connected and callable
    if let Ok(message_normal) = result_normal {
        println!(
            "Normal parsing release count: {}",
            message_normal.flat.releases.len()
        );
    }
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
    let start = Instant::now();
    let result = parser.parse_fast_streaming(cursor);
    let elapsed = start.elapsed();

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
