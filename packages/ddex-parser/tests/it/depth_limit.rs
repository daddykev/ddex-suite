use ddex_parser::parser::security::SecurityConfig;
use ddex_parser::{error::ParseError, DDEXParser};
use std::io::Cursor;

#[test]
fn test_depth_limit_enforcement() {
    // Create a deeply nested XML that exceeds the limit
    let deep_xml = create_deep_xml(105); // More than default limit of 100

    // Create parser with strict security config
    let security_config = SecurityConfig::strict();
    let mut parser = DDEXParser::with_config(security_config);

    let cursor = Cursor::new(deep_xml.as_bytes());
    let result = parser.parse(cursor);

    match result {
        Err(ParseError::DepthLimitExceeded { depth, limit }) => {
            assert!(depth > limit);
            assert_eq!(limit, 100); // Default strict limit
        }
        other => panic!("Expected DepthLimitExceeded error, got: {:?}", other),
    }
}

#[test]
fn test_depth_limit_within_bounds() {
    // Create a nested XML within the limit
    let xml = create_deep_xml(50); // Within limit of 100

    let mut parser = DDEXParser::new();
    let cursor = Cursor::new(xml.as_bytes());

    // This should not fail due to depth limits
    // (it might fail for other reasons like unsupported elements, but not depth)
    match parser.parse(cursor) {
        Err(ParseError::DepthLimitExceeded { .. }) => {
            panic!("Should not fail with depth limit error for depth 50");
        }
        _ => {
            // Other errors are acceptable for this test
        }
    }
}

#[test]
fn test_custom_depth_limit() {
    let xml = create_deep_xml(15); // Just over custom limit of 10

    let mut security_config = SecurityConfig::strict();
    security_config.max_element_depth = 10;

    let mut parser = DDEXParser::with_config(security_config);
    let cursor = Cursor::new(xml.as_bytes());
    let result = parser.parse(cursor);

    match result {
        Err(ParseError::DepthLimitExceeded { depth, limit }) => {
            assert!(depth > 10);
            assert_eq!(limit, 10);
        }
        other => panic!("Expected DepthLimitExceeded error, got: {:?}", other),
    }
}

#[test]
fn test_relaxed_depth_limit() {
    let xml = create_deep_xml(150); // More than strict limit but within relaxed

    let security_config = SecurityConfig::relaxed(); // Should have limit of 200
    let mut parser = DDEXParser::with_config(security_config);

    let cursor = Cursor::new(xml.as_bytes());

    // Should not fail due to depth limits
    match parser.parse(cursor) {
        Err(ParseError::DepthLimitExceeded { .. }) => {
            panic!("Should not fail with depth limit error for depth 150 with relaxed config");
        }
        _ => {
            // Other errors are acceptable
        }
    }
}

/// Creates a deeply nested XML structure for testing depth limits
fn create_deep_xml(depth: usize) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43"
                       xmlns:avs="http://ddex.net/xml/avs"
                       xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <ern:MessageHeader>
        <ern:MessageId>MSG001</ern:MessageId>
        <ern:MessageSender>
            <ern:PartyId>SENDER001</ern:PartyId>
        </ern:MessageSender>
        <ern:MessageRecipient>
            <ern:PartyId>RECIPIENT001</ern:PartyId>
        </ern:MessageRecipient>
    </ern:MessageHeader>
    <ern:ReleaseList>
        <ern:Release>
            <ern:ReleaseId>REL001</ern:ReleaseId>
            <ern:ReleaseReference>R001</ern:ReleaseReference>
"#,
    );

    // Add deeply nested elements
    for i in 0..depth {
        xml.push_str(&format!("            <ern:NestedElement{}>", i));
    }

    xml.push_str("<ern:Content>Deep content</ern:Content>");

    // Close deeply nested elements
    for i in (0..depth).rev() {
        xml.push_str(&format!("</ern:NestedElement{}>", i));
    }

    xml.push_str(
        r#"
        </ern:Release>
    </ern:ReleaseList>
</ern:NewReleaseMessage>"#,
    );

    xml
}

/// Regression: sibling elements must not accumulate depth (10 sibling releases sit at
/// depth 3, not 13+). Formerly tests/test_depth_fix.rs.
#[test]
fn sibling_elements_do_not_accumulate_depth() {
    let releases: String = (0..10)
        .map(|i| crate::common::ern43_release(&format!("R{i}"), &format!("Title {i}"), "Artist"))
        .collect();
    let xml = crate::common::ern43_message(&format!("<ReleaseList>{releases}</ReleaseList>"));

    let parsed = crate::common::parse_str(&xml)
        .unwrap_or_else(|e| panic!("10 sibling releases must parse: {e:?}"));
    assert_eq!(parsed.flat.releases.len(), 10);
}
