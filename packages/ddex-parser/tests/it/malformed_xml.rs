//! Malformed-XML detection and acceptance of well-formed edge cases.

use ddex_parser::parser::security::SecurityConfig;
use ddex_parser::{error::ParseError, DDEXParser};
use std::io::Cursor;

#[test]
#[ignore = "known bug: quick-xml's own end-tag/attribute checks fire first and surface as a generic XmlError, pre-empting the typed variants from parser::xml_validator"]
fn test_mismatched_opening_closing_tags() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
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
            </ern:WrongTag>
        </ern:ReleaseList>
    </ern:NewReleaseMessage>"#;

    let mut parser = DDEXParser::new();
    let cursor = Cursor::new(xml.as_bytes());
    let result = parser.parse(cursor);

    match result {
        Err(ParseError::MismatchedTags {
            expected, found, ..
        }) => {
            assert_eq!(expected, "Release");
            assert_eq!(found, "WrongTag");
        }
        other => panic!("Expected MismatchedTags error, got: {:?}", other),
    }
}

#[test]
#[ignore = "known bug: quick-xml's own end-tag/attribute checks fire first and surface as a generic XmlError, pre-empting the typed variants from parser::xml_validator"]
fn test_unexpected_closing_tag() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
        <ern:MessageHeader>
            <ern:MessageId>MSG002</ern:MessageId>
            <ern:MessageSender>
                <ern:PartyId>SENDER002</ern:PartyId>
            </ern:MessageSender>
            <ern:MessageRecipient>
                <ern:PartyId>RECIPIENT002</ern:PartyId>
            </ern:MessageRecipient>
        </ern:MessageHeader>
        <ern:ReleaseList>
            <ern:Release>
                <ern:ReleaseId>REL002</ern:ReleaseId>
            </ern:Release>
        </ern:UnexpectedTag>
        </ern:ReleaseList>
    </ern:NewReleaseMessage>"#;

    let mut parser = DDEXParser::new();
    let cursor = Cursor::new(xml.as_bytes());
    let result = parser.parse(cursor);

    match result {
        Err(ParseError::UnexpectedClosingTag { tag, .. }) => {
            assert_eq!(tag, "UnexpectedTag");
        }
        other => panic!("Expected UnexpectedClosingTag error, got: {:?}", other),
    }
}

#[test]
fn test_unclosed_tags() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
        <ern:MessageHeader>
            <ern:MessageId>MSG003</ern:MessageId>
            <ern:MessageSender>
                <ern:PartyId>SENDER003</ern:PartyId>
            </ern:MessageSender>
            <ern:MessageRecipient>
                <ern:PartyId>RECIPIENT003</ern:PartyId>
            </ern:MessageRecipient>
        </ern:MessageHeader>
        <ern:ReleaseList>
            <ern:Release>
                <ern:ReleaseId>REL003</ern:ReleaseId>
    <!-- Missing closing tags -->"#;

    let mut parser = DDEXParser::new();
    let cursor = Cursor::new(xml.as_bytes());
    let result = parser.parse(cursor);

    match result {
        Err(ParseError::UnclosedTags { tags, .. }) => {
            assert!(tags.contains(&"Release".to_string()));
            assert!(tags.contains(&"ReleaseList".to_string()));
            assert!(tags.contains(&"NewReleaseMessage".to_string()));
        }
        other => panic!("Expected UnclosedTags error, got: {:?}", other),
    }
}

#[test]
fn test_invalid_element_name() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
        <ern:MessageHeader>
            <ern:MessageId>MSG004</ern:MessageId>
            <123InvalidName>Invalid</123InvalidName>
        </ern:MessageHeader>
    </ern:NewReleaseMessage>"#;

    let mut parser = DDEXParser::new();
    let cursor = Cursor::new(xml.as_bytes());
    let result = parser.parse(cursor);

    match result {
        Err(ParseError::MalformedXml { message, .. }) => {
            assert!(message.contains("Invalid element name"));
            assert!(message.contains("123InvalidName"));
        }
        other => panic!("Expected MalformedXml error, got: {:?}", other),
    }
}

#[test]
#[ignore = "known bug: quick-xml's own end-tag/attribute checks fire first and surface as a generic XmlError, pre-empting the typed variants from parser::xml_validator"]
fn test_duplicate_attributes() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
        <ern:MessageHeader>
            <ern:MessageId id="first" id="second">MSG005</ern:MessageId>
        </ern:MessageHeader>
    </ern:NewReleaseMessage>"#;

    let mut parser = DDEXParser::new();
    let cursor = Cursor::new(xml.as_bytes());
    let result = parser.parse(cursor);

    match result {
        Err(ParseError::InvalidAttribute { message, .. }) => {
            assert!(message.contains("Duplicate attribute"));
            assert!(message.contains("id"));
        }
        other => panic!("Expected InvalidAttribute error, got: {:?}", other),
    }
}

#[test]
fn test_invalid_attribute_characters() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
        <ern:MessageHeader>
            <ern:MessageId value="test < invalid">MSG006</ern:MessageId>
        </ern:MessageHeader>
    </ern:NewReleaseMessage>"#;

    let mut parser = DDEXParser::new();
    let cursor = Cursor::new(xml.as_bytes());
    let result = parser.parse(cursor);

    match result {
        Err(ParseError::InvalidAttribute { message, .. }) => {
            assert!(message.contains("Invalid character in attribute value"));
        }
        other => panic!("Expected InvalidAttribute error, got: {:?}", other),
    }
}

#[test]
#[ignore = "known bug: ']]>' in character data is accepted (not well-formed XML); the text is also mangled"]
fn test_cdata_terminator_in_text_is_rejected() {
    // "]]>" may not appear in character data (XML 1.0 §2.4); here it follows a CDATA
    // section that ends early. The header is otherwise valid.
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    {}
    <ReleaseList>
        <Release>
            <ReleaseReference>R1</ReleaseReference>
            <ReleaseType>Album</ReleaseType>
            <DisplayTitleText><![CDATA[This contains ]]> in the middle]]></DisplayTitleText>
        </Release>
    </ReleaseList>
</ern:NewReleaseMessage>"#,
        crate::common::ERN43_HEADER
    );

    let result = DDEXParser::new().parse(Cursor::new(xml.as_bytes()));
    assert!(
        result.is_err(),
        "']]>' in character data must be rejected, got: {result:?}"
    );
}

#[test]
fn test_empty_element_name() {
    // This test uses a manually constructed byte array to test empty element names
    // since it's hard to construct with valid XML syntax
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
        <ern:MessageHeader>
            <ern:MessageId>MSG008</ern:MessageId>
            <>Empty</>
        </ern:MessageHeader>
    </ern:NewReleaseMessage>"#;

    let mut parser = DDEXParser::new();
    let cursor = Cursor::new(xml.as_bytes());
    let result = parser.parse(cursor);

    // This should fail during XML parsing itself, not reach our validator
    match result {
        Err(ParseError::XmlError { .. }) | Err(ParseError::MalformedXml { .. }) => {
            // Either error is acceptable for this malformed case
        }
        other => panic!(
            "Expected parsing error for empty element name, got: {:?}",
            other
        ),
    }
}

#[test]
fn test_well_formed_nested_structure_parses() {
    let xml = crate::common::ern43_message(&format!(
        "<ReleaseList>{}</ReleaseList>",
        crate::common::ern43_release("R009", "Valid Nested Structure", "Artist")
    ));
    let parsed = crate::common::parse_str(&xml).expect("well-formed nested XML must parse");
    assert_eq!(
        parsed.flat.releases[0].default_title.as_deref(),
        Some("Valid Nested Structure")
    );
}

#[test]
fn test_xml_with_comments_and_processing_instructions() {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- This is a comment -->
<?processing-instruction data?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <!-- Another comment -->
    {}
    <ReleaseList>{}</ReleaseList>
</ern:NewReleaseMessage>"#,
        crate::common::ERN43_HEADER,
        crate::common::ern43_release("R1", "Title", "Artist")
    );
    let parsed =
        crate::common::parse_str(&xml).expect("comments and processing instructions are allowed");
    assert_eq!(parsed.flat.message_id, "MSG001");
}

#[test]
fn test_self_closing_elements() {
    let release = r#"<Release>
            <ReleaseReference>R1</ReleaseReference>
            <ReleaseType>Album</ReleaseType>
            <DisplayTitleText>Title</DisplayTitleText>
            <ResourceGroup/>
        </Release>"#;
    let xml =
        crate::common::ern43_message(&format!("<ReleaseList>{release}</ReleaseList><DealList/>"));
    let parsed = crate::common::parse_str(&xml).expect("self-closing elements must parse");
    assert_eq!(parsed.flat.releases.len(), 1);
    assert!(parsed.flat.deals.is_empty());
}

#[test]
#[ignore = "known bug: quick-xml's own end-tag/attribute checks fire first and surface as a generic XmlError, pre-empting the typed variants from parser::xml_validator"]
fn test_complex_malformed_structure() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
        <ern:MessageHeader>
            <ern:MessageId>MSG012</ern:MessageId>
            <ern:MessageSender>
                <ern:PartyId>SENDER012</ern:PartyId>
            </ern:MessageRecipient> <!-- Wrong closing tag -->
            <ern:MessageRecipient>
                <ern:PartyId>RECIPIENT012</ern:PartyId>
            </ern:MessageSender> <!-- Wrong closing tag -->
        </ern:MessageHeader>
    </ern:NewReleaseMessage>"#;

    let mut parser = DDEXParser::new();
    let cursor = Cursor::new(xml.as_bytes());
    let result = parser.parse(cursor);

    // Should catch the first mismatched tag
    match result {
        Err(ParseError::MismatchedTags {
            expected, found, ..
        }) => {
            assert_eq!(expected, "MessageSender");
            assert_eq!(found, "MessageRecipient");
        }
        other => panic!("Expected MismatchedTags error, got: {:?}", other),
    }
}

#[test]
fn test_strict_and_default_configs_accept_well_formed_xml() {
    let xml = crate::common::ern43_message(&format!(
        "<ReleaseList>{}</ReleaseList>",
        crate::common::ern43_release("R1", "Title", "Artist")
    ));
    for (label, mut parser) in [
        ("strict", DDEXParser::with_config(SecurityConfig::strict())),
        ("default", DDEXParser::new()),
    ] {
        parser
            .parse(Cursor::new(xml.as_bytes()))
            .unwrap_or_else(|e| panic!("{label} config must accept well-formed XML: {e:?}"));
    }
}
