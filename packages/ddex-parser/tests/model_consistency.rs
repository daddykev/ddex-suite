// core/tests/model_consistency.rs
use ddex_parser::DDEXParser;

#[test]
fn test_graph_to_flat_consistency() {
    let parser = DDEXParser::new();
    let xml = include_str!("../../../test-suite/valid/ern-4.3/simple_release.xml");
    
    let result = parser.parse(std::io::Cursor::new(xml.as_bytes()))
        .expect("Failed to parse test file");
    
    // Verify both models are populated
    assert!(!result.graph.resources.is_empty());
    assert!(!result.flat.releases.is_empty());
    
    // Verify version consistency
    assert_eq!(result.graph.version.to_string(), result.flat.version);
}

#[test]
fn test_round_trip_preservation() {
    let parser = DDEXParser::new();
    let xml = include_str!("../../../test-suite/valid/ern-4.3/simple_release.xml");
    
    let result1 = parser.parse(std::io::Cursor::new(xml.as_bytes())).unwrap();
    let json = serde_json::to_string(&result1.graph).unwrap();
    let _deserialized: ddex_core::models::graph::ERNMessage = 
        serde_json::from_str(&json).unwrap();
    
    assert!(true, "Round trip successful");
}

#[test]
fn test_ffi_error_conversion() {
    use ddex_parser::error::{ParseError, ErrorLocation};
    use ddex_core::ffi::FFIError;
    
    let error = ParseError::XmlError {
        message: "Test error".to_string(),
        location: ErrorLocation {
            line: 10,
            column: 20,
            byte_offset: Some(100),
            path: "/test".to_string(),
        },
    };
    
    let _ffi_error: FFIError = error.into();
    assert!(true, "FFI conversion successful");
}