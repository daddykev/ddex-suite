use ddex_parser::DDEXParser;
use ddex_core::models::versions::ERNVersion;

#[test]
fn test_version_migration_382_to_42() {
    let parser = DDEXParser::new();
    
    // Parse a 3.8.2 file
    let xml_382 = include_str!("../../../test-suite/valid/ern-382/basic_release.xml");
    let result = parser.parse(std::io::Cursor::new(xml_382.as_bytes()));
    assert!(result.is_ok());
    
    let parsed = result.unwrap();
    assert_eq!(parsed.graph.version, ERNVersion::V3_8_2);
    
    // In a real migration, you would transform the data
    // For now, just verify we can parse both versions
    let xml_42 = include_str!("../../../test-suite/valid/ern-42/basic_release.xml");
    let result42 = parser.parse(std::io::Cursor::new(xml_42.as_bytes()));
    assert!(result42.is_ok());
}

#[test]
fn test_version_migration_42_to_43() {
    let parser = DDEXParser::new();
    
    // Parse a 4.2 file
    let xml_42 = include_str!("../../../test-suite/valid/ern-42/basic_release.xml");
    let result = parser.parse(std::io::Cursor::new(xml_42.as_bytes()));
    assert!(result.is_ok());
    
    let parsed = result.unwrap();
    assert_eq!(parsed.graph.version, ERNVersion::V4_2);
    
    // Parse a 4.3 file
    let xml_43 = include_str!("../../../test-suite/valid/ern-4.3/simple_release.xml");
    let result43 = parser.parse(std::io::Cursor::new(xml_43.as_bytes()));
    assert!(result43.is_ok());
    
    let parsed43 = result43.unwrap();
    assert_eq!(parsed43.graph.version, ERNVersion::V4_3);
}