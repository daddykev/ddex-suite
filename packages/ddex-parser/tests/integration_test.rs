// core/tests/integration_test.rs
use ddex_parser::DDEXParser;
use ddex_core::models::versions::ERNVersion;

#[test]
fn test_parse_all_versions() {
    let parser = DDEXParser::new();
    
    let test_cases = vec![
        (
            "3.8.2",
            include_str!("../../../test-suite/valid/ern-382/basic_release.xml"),
            ERNVersion::V3_8_2,
        ),
        (
            "4.2",
            include_str!("../../../test-suite/valid/ern-42/basic_release.xml"),
            ERNVersion::V4_2,
        ),
        (
            "4.3",
            include_str!("../../../test-suite/valid/ern-4.3/simple_release.xml"),
            ERNVersion::V4_3,
        ),
    ];
    
    for (version_name, xml, expected_version) in test_cases {
        println!("Testing ERN {}", version_name);
        let result = parser.parse(std::io::Cursor::new(xml.as_bytes()));
        assert!(result.is_ok(), "Failed to parse ERN {}", version_name);
        
        let parsed = result.unwrap();
        assert_eq!(parsed.graph.version, expected_version);
    }
}