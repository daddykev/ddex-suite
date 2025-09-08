// core/tests/error_contract_test.rs
use ddex_parser::DDEXParser;

#[test]
fn test_error_on_invalid_xml() {
    let parser = DDEXParser::new();
    let invalid_xml = "not xml";
    let result = parser.parse(std::io::Cursor::new(invalid_xml.as_bytes()));
    assert!(result.is_err());
}

#[test]
fn test_error_on_empty_input() {
    let parser = DDEXParser::new();
    let empty = "";
    let result = parser.parse(std::io::Cursor::new(empty.as_bytes()));
    assert!(result.is_err());
}

#[test]
fn test_error_on_malformed_xml() {
    let parser = DDEXParser::new();
    let malformed = "<unclosed>";
    let result = parser.parse(std::io::Cursor::new(malformed.as_bytes()));
    assert!(result.is_err());
}

#[test]
fn test_ffi_error_serialization() {
    use ddex_core::ffi::{FFIError, FFIErrorCategory, FFIErrorSeverity};
    
    let ffi_error = FFIError {
        code: "TEST_ERROR".to_string(),
        message: "Test message".to_string(),
        location: None,
        severity: FFIErrorSeverity::Error,
        hint: Some("Test hint".to_string()),
        category: FFIErrorCategory::XmlParsing,
    };
    
    let json = serde_json::to_string(&ffi_error).unwrap();
    assert!(json.contains("TEST_ERROR"));
}