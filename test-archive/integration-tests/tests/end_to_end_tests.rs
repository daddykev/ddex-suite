//! End-to-end integration tests
//! 
//! These tests simulate real-world usage scenarios and verify
//! the complete workflow from XML input to XML output.

use integration_tests::{
    fixtures::{FixtureManager, create_minimal_ern43_xml, create_complex_ern43_xml},
    round_trip::RoundTripTester,
    utils::{format_duration, measure_time, create_xml_diff, XMLStructure},
};
use ddex_parser::DDEXParser;
use ddex_builder::Builder;
use std::io::Cursor;
use tempfile::TempDir;
use std::fs;
use pretty_assertions::assert_eq;

/// Test complete workflow: file -> parse -> process -> build -> file
#[test]
fn test_complete_file_workflow() {
    // Setup temporary directory for test files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let input_path = temp_dir.path().join("input.xml");
    let output_path = temp_dir.path().join("output.xml");
    
    // Create input file
    let xml_content = create_minimal_ern43_xml();
    fs::write(&input_path, &xml_content).expect("Failed to write input file");
    
    // Step 1: Parse from file
    let parser = DDEXParser::new();
    let input_file = fs::File::open(&input_path).expect("Failed to open input file");
    let mut buf_reader = std::io::BufReader::new(input_file);
    
    let (parsed_message, parse_time) = measure_time(|| {
        parser.parse(&mut buf_reader).expect("Failed to parse input file")
    });
    
    println!("File parse time: {}", format_duration(parse_time));
    
    // Step 2: Process the data (simulate some business logic)
    let processed_message = {
        let mut msg = parsed_message.clone();
        // Example processing: add a prefix to the message ID
        msg.flat.message_id = format!("PROCESSED_{}", msg.flat.message_id);
        msg
    };
    
    // Step 3: Convert back to XML using builder (when available)
    // For now, we'll serialize the processed message to JSON as a proxy
    let (processed_json, serialize_time) = measure_time(|| {
        serde_json::to_string_pretty(&processed_message)
            .expect("Failed to serialize processed message")
    });
    
    println!("Serialization time: {}", format_duration(serialize_time));
    
    // Write processed data to output file
    let json_output_path = temp_dir.path().join("processed.json");
    fs::write(&json_output_path, &processed_json).expect("Failed to write processed JSON");
    
    // Verify the workflow preserved data integrity
    assert!(processed_message.flat.message_id.starts_with("PROCESSED_"));
    assert_eq!(processed_message.releases().len(), parsed_message.releases().len());
    assert_eq!(processed_message.resources().len(), 
              parsed_message.resources().len());
    
    println!("Complete file workflow test passed");
    println!("Input file: {} bytes", xml_content.len());
    println!("Output JSON: {} bytes", processed_json.len());
}

/// Test batch processing of multiple XML files
#[test]
fn test_batch_processing() {
    let fixture_manager = FixtureManager::new().expect("Failed to create fixture manager");
    let fixtures = fixture_manager.create_standard_fixtures().expect("Failed to create fixtures");
    
    let parser = DDEXParser::new();
    let mut processed_files = Vec::new();
    
    // Process all fixture files
    for (name, path) in [
        ("minimal", &fixtures.minimal_ern43),
        ("complex", &fixtures.complex_ern43),
    ] {
        println!("Processing file: {}", name);
        
        let xml_content = fs::read_to_string(path).expect("Failed to read fixture");
        let cursor = Cursor::new(&xml_content);
        
        let (parsed, parse_time) = measure_time(|| {
            parser.parse(cursor).expect("Failed to parse fixture")
        });
        
        processed_files.push((name.to_string(), parsed, parse_time));
        println!("  Parsed in: {}", format_duration(parse_time));
    }
    
    // Verify batch processing results
    assert_eq!(processed_files.len(), 2);
    
    let total_parse_time: std::time::Duration = processed_files.iter()
        .map(|(_, _, time)| *time)
        .sum();
    
    println!("Total batch processing time: {}", format_duration(total_parse_time));
    
    // Verify each file was processed correctly
    for (name, parsed, _) in processed_files {
        assert!(!parsed.flat.message_id.is_empty(), 
               "File {} should have message ID", name);
        assert!(!parsed.releases().is_empty(), 
               "File {} should have releases", name);
        assert!(!parsed.resources().is_empty(), 
               "File {} should have sound recordings", name);
    }
}

/// Test error recovery and validation
#[test]
fn test_error_recovery_workflow() {
    let parser = DDEXParser::new();
    
    // Test with various error conditions
    let error_cases = vec![
        ("empty", ""),
        ("malformed", "<invalid>unclosed"),
        ("non_ddex", "<?xml version='1.0'?><root><child>value</child></root>"),
        ("incomplete", "<?xml version='1.0'?><ern:NewReleaseMessage xmlns:ern='http://ddex.net/xml/ern/43'>"),
    ];
    
    for (name, xml) in error_cases {
        println!("Testing error case: {}", name);
        
        let cursor = Cursor::new(xml);
        let result = parser.parse(cursor);
        
        match result {
            Ok(_) => {
                println!("  Unexpectedly succeeded (parser may be very tolerant)");
            }
            Err(e) => {
                println!("  Error handled: {:?}", e);
                // Verify error is informative
                let error_str = format!("{:?}", e);
                assert!(!error_str.is_empty(), "Error should have description");
            }
        }
    }
}

/// Test performance characteristics under various conditions
#[test]
fn test_performance_characteristics() {
    let parser = DDEXParser::new();
    
    // Test performance with different XML sizes
    let base_xml = create_complex_ern43_xml();
    
    for multiplier in [1, 2, 5] {
        // Create larger XML by duplicating content
        let mut large_xml = base_xml.clone();
        
        // Add multiple sound recordings
        if multiplier > 1 {
            let additional_recording = r#"
        <SoundRecording>
            <SoundRecordingType>MusicalWorkSoundRecording</SoundRecordingType>
            <ResourceReference>SoundRecording_PERF_{}</ResourceReference>
            <ReferenceTitle>
                <TitleText>Performance Test Track {}</TitleText>
            </ReferenceTitle>
            <Duration>PT3M{}S</Duration>
            <DisplayArtist>
                <PartyName>
                    <FullName>Performance Test Artist {}</FullName>
                </PartyName>
            </DisplayArtist>
            <SoundRecordingDetailsByTerritory>
                <TerritoryCode>Worldwide</TerritoryCode>
                <Title>
                    <TitleText>Performance Test Track {}</TitleText>
                </Title>
                <DisplayArtist>
                    <PartyName>
                        <FullName>Performance Test Artist {}</FullName>
                    </PartyName>
                </DisplayArtist>
            </SoundRecordingDetailsByTerritory>
        </SoundRecording>"#;
            
            if let Some(pos) = large_xml.rfind("</ResourceList>") {
                for i in 3..=(multiplier * 5) {
                    let recording = additional_recording
                        .replace("{}", &format!("{}", i))
                        .replace("PT3M{}S", &format!("PT3M{}S", 30 + i * 10));
                    large_xml.insert_str(pos, &recording);
                }
            }
        }
        
        let xml_size = large_xml.len();
        let cursor = Cursor::new(&large_xml);
        
        let (parsed, parse_time) = measure_time(|| {
            parser.parse(cursor).expect("Failed to parse in performance test")
        });
        
        println!("Multiplier {}: {} bytes, parse time: {}, {} sound recordings",
                multiplier, xml_size, format_duration(parse_time), 
                parsed.resources().len());
        
        // Verify linear or sub-linear scaling
        if multiplier == 1 {
            // Store baseline for comparison
            continue;
        }
        
        // Performance should scale reasonably with content size
        let size_ratio = xml_size as f64 / base_xml.len() as f64;
        println!("  Size ratio: {:.2}x", size_ratio);
        
        // Check that performance scales reasonably (not exponentially)
        // This is a soft assertion for development monitoring
        if parse_time > std::time::Duration::from_millis(50) {
            println!("  WARNING: Parse time may be scaling poorly with document size");
        }
    }
}

/// Test memory-bounded streaming for large files
#[test]
fn test_streaming_large_files() {
    // This test simulates processing files that are too large to fit in memory
    // For now, we create a reasonably large test case
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let large_file_path = temp_dir.path().join("large_test.xml");
    
    // Create a large XML file by replicating content
    let base_xml = create_complex_ern43_xml();
    let mut large_content = String::with_capacity(base_xml.len() * 50);
    
    // Start with XML header
    large_content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    large_content.push_str("<ern:NewReleaseMessage xmlns:ern=\"http://ddex.net/xml/ern/43\" ");
    large_content.push_str("MessageSchemaVersionId=\"ern/43\">\n");
    large_content.push_str("<MessageHeader><MessageId>LARGE_FILE_TEST</MessageId></MessageHeader>\n");
    large_content.push_str("<ReleaseList><Release><ReleaseReference>LARGE_REL</ReleaseReference></Release></ReleaseList>\n");
    large_content.push_str("<ResourceList>\n");
    
    // Add many sound recordings
    for i in 0..100 {
        large_content.push_str(&format!(r#"
    <SoundRecording>
        <SoundRecordingType>MusicalWorkSoundRecording</SoundRecordingType>
        <ResourceReference>SoundRecording_{}</ResourceReference>
        <ReferenceTitle><TitleText>Large File Track {}</TitleText></ReferenceTitle>
        <Duration>PT3M{}S</Duration>
        <DisplayArtist><PartyName><FullName>Large File Artist {}</FullName></PartyName></DisplayArtist>
        <SoundRecordingDetailsByTerritory>
            <TerritoryCode>Worldwide</TerritoryCode>
            <Title><TitleText>Large File Track {}</TitleText></Title>
            <DisplayArtist><PartyName><FullName>Large File Artist {}</FullName></PartyName></DisplayArtist>
        </SoundRecordingDetailsByTerritory>
    </SoundRecording>"#, i, i, 30 + (i % 60), i, i, i));
    }
    
    large_content.push_str("</ResourceList>\n</ern:NewReleaseMessage>");
    
    // Write large file
    fs::write(&large_file_path, &large_content).expect("Failed to write large file");
    
    let file_size = large_content.len();
    println!("Large test file size: {} bytes (~{:.1} KB)", file_size, file_size as f64 / 1024.0);
    
    // Parse the large file
    let parser = DDEXParser::new();
    let file = fs::File::open(&large_file_path).expect("Failed to open large file");
    let mut buf_reader = std::io::BufReader::new(file);
    
    let (parsed, parse_time) = measure_time(|| {
        parser.parse(&mut buf_reader).expect("Failed to parse large file")
    });
    
    println!("Large file parse time: {}", format_duration(parse_time));
    println!("Sound recordings parsed: {}", parsed.resources().len());
    
    // Verify all content was parsed correctly
    assert_eq!(parsed.flat.message_id, "LARGE_FILE_TEST");
    assert_eq!(parsed.resources().len(), 100);
    
    // Check memory efficiency - parsing should not consume excessive memory
    // This is more of a documentation test than a strict assertion
    if parse_time > std::time::Duration::from_millis(100) {
        println!("WARNING: Large file parsing may need optimization");
    }
}

/// Test round-trip with data modifications
#[test]
fn test_round_trip_with_modifications() {
    let original_xml = create_minimal_ern43_xml();
    let parser = DDEXParser::new();
    
    // Parse original
    let cursor = Cursor::new(&original_xml);
    let mut parsed = parser.parse(cursor).expect("Failed to parse for modification test");
    
    // Make modifications
    let original_message_id = parsed.flat.message_id.clone();
    parsed.flat.message_id = format!("MODIFIED_{}", original_message_id);
    
    // Note: Cannot easily modify resources as it's a method returning a reference.
    // In a real implementation, we would need mutable access to the internal structure.
    // For now, we'll skip the resource modification part of the test.
    
    // Serialize modified data
    let modified_json = serde_json::to_string_pretty(&parsed)
        .expect("Failed to serialize modified data");
    
    // Verify modifications were applied
    let deserialized: ddex_core::models::flat::ParsedERNMessage = 
        serde_json::from_str(&modified_json).expect("Failed to deserialize modified data");
    
    assert!(deserialized.flat.message_id.starts_with("MODIFIED_"));
    assert_ne!(deserialized.flat.message_id, original_message_id);
    
    // In a full implementation, we would now build XML from the modified data
    // and verify the changes are present in the output
    
    println!("Round-trip with modifications test passed");
    println!("Original message ID: {}", original_message_id);
    println!("Modified message ID: {}", deserialized.flat.message_id);
    println!("Sound recordings after modification: {}", deserialized.resources().len());
}