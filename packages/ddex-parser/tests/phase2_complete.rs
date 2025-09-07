// core/tests/phase2_complete.rs
use ddex_parser::DDEXParser;
use ddex_core::models::versions::ERNVersion;

#[test]
fn test_phase2_version_support() {
    let parser = DDEXParser::new();
    
    // Test all supported versions
    for version in &[ERNVersion::V3_8_2, ERNVersion::V4_2, ERNVersion::V4_3] {
        println!("Testing version: {:?}", version);
        
        let xml = match version {
            ERNVersion::V3_8_2 => {
                include_str!("../../../test-suite/valid/ern-382/basic_release.xml")
            }
            ERNVersion::V4_2 => {
                include_str!("../../../test-suite/valid/ern-42/basic_release.xml")
            }
            ERNVersion::V4_3 => {
                include_str!("../../../test-suite/valid/ern-4.3/simple_release.xml")
            }
        };
        
        let result = parser.parse(std::io::Cursor::new(xml.as_bytes()));
        assert!(result.is_ok(), "Failed to parse version {:?}", version);
        
        let parsed = result.unwrap();
        assert_eq!(parsed.graph.version, *version);
    }
}

#[test]
fn test_streaming_support() {
    let parser = DDEXParser::new();
    let xml = include_str!("../../../test-suite/valid/ern-4.3/simple_release.xml");
    
    // Test that streaming mode works
    let result = parser.parse(std::io::Cursor::new(xml));
    assert!(result.is_ok(), "Streaming parse should work");
}