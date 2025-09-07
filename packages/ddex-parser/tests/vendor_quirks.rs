// core/tests/vendor_quirks.rs
use ddex_parser::DDEXParser;
use std::fs;
use std::path::Path;

#[test]
fn test_vendor_a_missing_thread_id() {
    let parser = DDEXParser::new();
    // Test parsing with missing MessageThreadId (vendor quirk)
    let xml = r#"<?xml version="1.0"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader>
                <MessageId>MSG123</MessageId>
            </MessageHeader>
        </ern:NewReleaseMessage>"#;
    
    let result = parser.parse(std::io::Cursor::new(xml.as_bytes()));
    assert!(result.is_ok(), "Should handle missing MessageThreadId");
}

#[test]
fn test_vendor_b_empty_audit_trail() {
    let parser = DDEXParser::new();
    // Test parsing with empty MessageAuditTrail (vendor quirk)
    let xml = r#"<?xml version="1.0"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/42">
            <MessageHeader>
                <MessageId>MSG123</MessageId>
                <MessageAuditTrail/>
            </MessageHeader>
        </ern:NewReleaseMessage>"#;
    
    let result = parser.parse(std::io::Cursor::new(xml.as_bytes()));
    assert!(result.is_ok(), "Should handle empty MessageAuditTrail");
}

#[test]
#[ignore]  // Ignore by default as it requires test files
fn test_all_vendor_quirks() {
    let parser = DDEXParser::new();
    let quirks_dir = Path::new("test-suite/vendor-quirks");
    
    if quirks_dir.exists() {
        for entry in fs::read_dir(quirks_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            
            if path.is_dir() {
                for file in fs::read_dir(&path).unwrap() {
                    let file = file.unwrap();
                    let file_path = file.path();
                    
                    if file_path.extension().map_or(false, |e| e == "xml") {
                        let xml = fs::read_to_string(&file_path).unwrap();
                        let result = parser.parse(std::io::Cursor::new(xml.as_bytes()));
                        assert!(result.is_ok(), "Failed to parse: {:?}", file_path);
                    }
                }
            }
        }
    }
}