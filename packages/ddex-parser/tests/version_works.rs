// core/tests/version_works.rs
use ddex_parser::DDEXParser;
use ddex_core::models::versions::ERNVersion;

#[test]
fn test_version_detection_from_namespace() {
    let parser = DDEXParser::new();
    
    let test_cases = vec![
        (r#"<?xml version="1.0"?><ern:ReleaseList xmlns:ern="http://ddex.net/xml/ern/382"/>"#, ERNVersion::V3_8_2),
        (r#"<?xml version="1.0"?><ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/42"/>"#, ERNVersion::V4_2),
        (r#"<?xml version="1.0"?><ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43"/>"#, ERNVersion::V4_3),
    ];
    
    for (xml, expected) in test_cases {
        let version = parser.detect_version(xml.as_bytes()).unwrap();
        assert_eq!(version, expected);
        println!("[OK] Detected version: {:?}", expected);
    }
}

#[test]
fn test_minimal_message_parsing() {
    let parser = DDEXParser::new();
    
    let minimal_xml = r#"<?xml version="1.0"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader>
                <MessageId>MSG123</MessageId>
            </MessageHeader>
        </ern:NewReleaseMessage>"#;
    
    let result = parser.parse(std::io::Cursor::new(minimal_xml.as_bytes()));
    assert!(result.is_ok());
    
    let parsed = result.unwrap();
    println!("[OK] Successfully parsed minimal ERN message");
    println!("  - Version: {:?}", parsed.flat.version);
}