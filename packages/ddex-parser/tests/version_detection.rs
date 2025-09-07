// core/tests/version_detection.rs
use ddex_parser::DDEXParser;
use ddex_core::models::versions::ERNVersion;
use std::io::Cursor;

#[test]
fn test_detect_version_382() {
    let parser = DDEXParser::new();
    let xml = r#"<?xml version="1.0"?><ern:ReleaseList xmlns:ern="http://ddex.net/xml/ern/382"/>"#;
    let version = parser.detect_version(Cursor::new(xml)).unwrap();
    assert_eq!(version, ERNVersion::V3_8_2);
}

#[test]
fn test_detect_version_42() {
    let parser = DDEXParser::new();
    let xml = r#"<?xml version="1.0"?><ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/42"/>"#;
    let version = parser.detect_version(Cursor::new(xml)).unwrap();
    assert_eq!(version, ERNVersion::V4_2);
}

#[test]
fn test_detect_version_43() {
    let parser = DDEXParser::new();
    let xml = r#"<?xml version="1.0"?><ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43"/>"#;
    let version = parser.detect_version(Cursor::new(xml)).unwrap();
    assert_eq!(version, ERNVersion::V4_3);
}

#[test]
fn test_detect_version_with_bom() {
    let parser = DDEXParser::new();
    let xml_with_bom = b"\xef\xbb\xbf<?xml version=\"1.0\"?><ern:NewReleaseMessage xmlns:ern=\"http://ddex.net/xml/ern/43\"/>";
    let version = parser.detect_version(&xml_with_bom[..]).unwrap();
    assert_eq!(version, ERNVersion::V4_3);
}