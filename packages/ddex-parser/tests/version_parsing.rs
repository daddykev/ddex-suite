// core/tests/version_parsing.rs
use ddex_parser::DDEXParser;
use ddex_core::models::versions::ERNVersion;

#[test]
fn test_parse_ern_382() {
    let parser = DDEXParser::new();
    let xml = include_str!("../../../test-suite/valid/ern-382/basic_release.xml");
    
    let result = parser.parse(std::io::Cursor::new(xml.as_bytes()))
        .expect("Failed to parse ERN 3.8.2");
    
    assert_eq!(result.graph.version, ERNVersion::V3_8_2);
}

#[test]
fn test_parse_ern_42() {
    let parser = DDEXParser::new();
    let xml = include_str!("../../../test-suite/valid/ern-42/basic_release.xml");
    
    let result = parser.parse(std::io::Cursor::new(xml.as_bytes()))
        .expect("Failed to parse ERN 4.2");
    
    assert_eq!(result.graph.version, ERNVersion::V4_2);
}

#[test]
fn test_parse_ern_43() {
    let parser = DDEXParser::new();
    let xml = include_str!("../../../test-suite/valid/ern-4.3/simple_release.xml");
    
    let result = parser.parse(std::io::Cursor::new(xml.as_bytes()))
        .expect("Failed to parse ERN 4.3");
    
    assert_eq!(result.graph.version, ERNVersion::V4_3);
    
    if let Some(deal) = result.graph.deals.first() {
        println!("Deal found in 4.3 file");
    }
}