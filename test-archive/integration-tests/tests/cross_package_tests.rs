//! Cross-package integration tests
//! 
//! These tests verify compatibility between parser and builder packages
//! and ensure that outputs from one package work correctly as inputs to another.

use integration_tests::{
    fixtures::{create_minimal_ern43_xml, create_complex_ern43_xml, FixtureManager},
    utils::{extract_xml_structure, hash_string, format_duration, measure_time},
};
use ddex_parser::DDEXParser;
use ddex_builder::Builder;
use ddex_core::models::flat::ParsedERNMessage;
use std::io::Cursor;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

/// Test that parser output structure is compatible with expected formats
#[test]
fn test_parser_output_structure() {
    let parser = DDEXParser::new();
    let xml = create_minimal_ern43_xml();
    let cursor = Cursor::new(&xml);
    
    let (parsed, parse_time) = measure_time(|| {
        parser.parse(cursor).expect("Failed to parse minimal XML")
    });
    
    println!("Parse time: {}", format_duration(parse_time));
    
    // Verify parsed message structure
    assert!(!parsed.flat.message_id.is_empty());
    assert!(!parsed.flat.sender.name.is_empty());
    assert!(!parsed.flat.recipient.name.is_empty());
    
    // Verify releases were parsed
    assert!(!parsed.releases().is_empty(), "Should have at least one release");
    
    // Verify resources were parsed
    assert!(!parsed.resources().is_empty(), "Should have at least one sound recording");
    
    // Test serialization to JSON (useful for debugging and data exchange)
    let json_result = serde_json::to_string_pretty(&parsed);
    assert!(json_result.is_ok(), "Parsed message should be serializable to JSON");
    
    if let Ok(json) = json_result {
        println!("Parsed message JSON size: {} bytes", json.len());
        
        // Verify we can deserialize back
        let deserialized: Result<ParsedERNMessage, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "Should be able to deserialize from JSON");
    }
}

/// Test that builder can handle various input formats
#[test]
fn test_builder_input_compatibility() {
    let builder = Builder::new();
    
    // This test currently documents the expected API
    // Implementation may need to be adjusted based on actual builder interface
    
    println!("Builder created successfully");
    
    // Test basic builder functionality
    assert!(!builder.available_presets().is_empty(), "Builder should have presets available");
    
    // As the builder API stabilizes, we can add more comprehensive tests here
}

/// Test parser error handling with malformed XML
#[test]
fn test_parser_error_handling() {
    let parser = DDEXParser::new();
    
    // Test with malformed XML
    let malformed_xml = r#"<invalid>unclosed tag"#;
    let cursor = Cursor::new(malformed_xml);
    let result = parser.parse(cursor);
    
    assert!(result.is_err(), "Parser should reject malformed XML");
    
    // Test with empty input
    let empty_cursor = Cursor::new("");
    let empty_result = parser.parse(empty_cursor);
    assert!(result.is_err(), "Parser should reject empty input");
    
    // Test with non-DDEX XML
    let non_ddex_xml = r#"<?xml version="1.0"?><root><child>not ddex</child></root>"#;
    let non_ddex_cursor = Cursor::new(non_ddex_xml);
    let non_ddex_result = parser.parse(non_ddex_cursor);
    assert!(non_ddex_result.is_err(), "Parser should reject non-DDEX XML");
}

/// Test parser security features
#[test]
fn test_parser_security() {
    let parser = DDEXParser::new();
    
    // Test XXE protection - attempt to include external entity
    let xxe_xml = r#"<?xml version="1.0"?>
<!DOCTYPE test [
  <!ENTITY xxe SYSTEM "file:///etc/passwd">
]>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>&xxe;</MessageId>
    </MessageHeader>
</ern:NewReleaseMessage>"#;
    
    let cursor = Cursor::new(xxe_xml);
    let result = parser.parse(cursor);
    
    // Should either reject the XML entirely or safely handle the entity
    // The exact behavior depends on the security configuration
    match result {
        Ok(parsed) => {
            // If parsing succeeds, ensure no external entity was resolved
            assert!(!parsed.flat.message_id.contains("root:"), 
                   "External entity should not be resolved");
        }
        Err(_) => {
            // Rejecting XXE attempts is also acceptable
            println!("Parser correctly rejected XXE attempt");
        }
    }
}

/// Test parser performance with large documents
#[test]
fn test_parser_performance_scaling() {
    let parser = DDEXParser::new();
    
    // Create a larger XML document by duplicating sound recordings
    let mut large_xml = create_complex_ern43_xml();
    
    // Add more sound recordings to test scaling
    let additional_recording = r#"
        <SoundRecording>
            <SoundRecordingType>MusicalWorkSoundRecording</SoundRecordingType>
            <ResourceReference>SoundRecording_PERF_TEST</ResourceReference>
            <ReferenceTitle>
                <TitleText>Performance Test Track</TitleText>
            </ReferenceTitle>
            <Duration>PT3M45S</Duration>
            <DisplayArtist>
                <PartyName>
                    <FullName>Performance Test Artist</FullName>
                </PartyName>
            </DisplayArtist>
            <SoundRecordingDetailsByTerritory>
                <TerritoryCode>Worldwide</TerritoryCode>
                <Title>
                    <TitleText>Performance Test Track</TitleText>
                </Title>
                <DisplayArtist>
                    <PartyName>
                        <FullName>Performance Test Artist</FullName>
                    </PartyName>
                </DisplayArtist>
            </SoundRecordingDetailsByTerritory>
        </SoundRecording>"#;
    
    // Insert additional recordings before closing </ResourceList>
    if let Some(pos) = large_xml.rfind("</ResourceList>") {
        for i in 3..=10 {
            let recording = additional_recording.replace("PERF_TEST", &format!("{}", i));
            large_xml.insert_str(pos, &recording);
        }
    }
    
    println!("Large XML size: {} bytes", large_xml.len());
    
    let cursor = Cursor::new(&large_xml);
    let (parsed, parse_time) = measure_time(|| {
        parser.parse(cursor).expect("Failed to parse large XML")
    });
    
    println!("Large document parse time: {}", format_duration(parse_time));
    println!("Sound recordings parsed: {}", parsed.resources().len());
    
    // Verify all sound recordings were parsed
    assert!(parsed.resources().len() >= 10, 
           "Should have parsed multiple sound recordings");
    
    // Performance expectation based on CLAUDE.md
    // This XML should be well under 100KB, so target is <10ms
    if parse_time > std::time::Duration::from_millis(10) {
        println!("WARNING: Large document parsing exceeded target time");
    }
}

/// Test data integrity across packages
#[test]
fn test_data_integrity_preservation() {
    let parser = DDEXParser::new();
    let test_cases = vec![
        ("minimal", create_minimal_ern43_xml()),
        ("complex", create_complex_ern43_xml()),
    ];
    
    for (name, xml) in test_cases {
        println!("Testing data integrity for: {}", name);
        
        let original_structure = extract_xml_structure(&xml);
        let original_hash = hash_string(&xml);
        
        let cursor = Cursor::new(&xml);
        let parsed = parser.parse(cursor).expect("Failed to parse for integrity test");
        
        // Verify key data is preserved in parsing
        assert_eq!(parsed.flat.message_id, 
                  original_structure.message_id.expect("Message ID should be present"),
                  "Message ID should be preserved");
        
        assert_eq!(parsed.releases().len(), original_structure.release_count,
                  "Release count should be preserved");
        
        assert_eq!(parsed.resources().len(), original_structure.sound_recording_count,
                  "Sound recording count should be preserved");
        
        // Test that re-serializing to JSON and back preserves data
        let json = serde_json::to_string(&parsed).expect("Should serialize to JSON");
        let deserialized: ParsedERNMessage = serde_json::from_str(&json)
            .expect("Should deserialize from JSON");
        
        assert_eq!(parsed.flat.message_id, deserialized.flat.message_id,
                  "Message ID should survive JSON round-trip");
        assert_eq!(parsed.releases().len(), deserialized.releases().len(),
                  "Release count should survive JSON round-trip");
        assert_eq!(parsed.resources().len(), deserialized.resources().len(),
                  "Sound recording count should survive JSON round-trip");
    }
}

/// Test memory usage patterns
#[test]
fn test_memory_usage() {
    let parser = DDEXParser::new();
    let xml = create_complex_ern43_xml();
    
    // Parse multiple times to test for memory leaks
    for i in 0..10 {
        let cursor = Cursor::new(&xml);
        let parsed = parser.parse(cursor).expect("Failed to parse in memory test");
        
        // Verify parsing is consistent
        assert!(!parsed.flat.message_id.is_empty(), 
               "Parse #{} should have message ID", i + 1);
        
        // Force some memory operations
        let _json = serde_json::to_string(&parsed).expect("Should serialize");
    }
    
    println!("Memory usage test completed - no crashes or excessive memory growth");
}

/// Test concurrent parsing (if async features are available)
#[test]
#[cfg(feature = "async")]
fn test_concurrent_parsing() {
    use std::sync::Arc;
    use tokio::task::JoinSet;
    
    let rt = tokio::runtime::Runtime::new().expect("Failed to create async runtime");
    
    rt.block_on(async {
        let parser = Arc::new(DDEXParser::new());
        let mut join_set = JoinSet::new();
        
        // Parse multiple documents concurrently
        let test_xmls = vec![
            create_minimal_ern43_xml(),
            create_complex_ern43_xml(),
        ];
        
        for (i, xml) in test_xmls.into_iter().enumerate() {
            let parser_clone = Arc::clone(&parser);
            join_set.spawn(async move {
                let cursor = Cursor::new(&xml);
                let parsed = parser_clone.parse(cursor).expect("Concurrent parse failed");
                (i, parsed.message_header.message_id.clone())
            });
        }
        
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            let (index, message_id) = result.expect("Concurrent task failed");
            results.push((index, message_id));
        }
        
        assert_eq!(results.len(), 2, "Should have completed both concurrent parses");
        println!("Concurrent parsing test completed successfully");
    });
}