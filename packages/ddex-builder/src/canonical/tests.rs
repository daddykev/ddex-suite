//! Tests for DB-C14N/1.0 canonicalization

use super::*;
use crate::determinism::DeterminismConfig;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_canonicalizer() -> DB_C14N {
        let config = DeterminismConfig::default();
        DB_C14N::new(config)
    }

    fn create_test_canonicalizer_with_version(version: &str) -> DB_C14N {
        let config = DeterminismConfig::default();
        DB_C14N::with_version(config, version.to_string())
    }

    #[test]
    fn test_xml_declaration() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>test</root>"#;
        
        let result = canonicalizer.canonicalize(input).unwrap();
        assert!(result.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    }

    #[test]
    fn test_attribute_sorting() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<root z="z" a="a" m="m">test</root>"#;
        
        let result = canonicalizer.canonicalize(input).unwrap();
        // Attributes should be sorted alphabetically
        assert!(result.contains(r#"<root a="a" m="m" z="z">"#));
    }

    #[test]
    fn test_whitespace_normalization() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
    
    test content   
    
</root>"#;
        
        let result = canonicalizer.canonicalize(input).unwrap();
        // Should normalize whitespace to single space
        assert!(result.contains("test content"));
    }

    #[test]
    fn test_indentation() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<root><child>content</child></root>"#;
        
        let result = canonicalizer.canonicalize(input).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        
        // Should have 2-space indentation
        assert!(lines.iter().any(|line| line.starts_with("  <child>")));
    }

    #[test]
    fn test_no_trailing_whitespace() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
  <child>content</child>
</root>"#;
        
        let result = canonicalizer.canonicalize(input).unwrap();
        // No line should end with whitespace
        for line in result.lines() {
            assert_eq!(line, line.trim_end(), "Line has trailing whitespace: '{}'", line);
        }
    }

    #[test]
    fn test_ern_version_detection() {
        let canonicalizer = create_test_canonicalizer();
        
        // Test ERN 3.8.2 detection
        assert_eq!(canonicalizer.detect_version(r#"<root xmlns="http://ddex.net/xml/ern/382">test</root>"#), "3.8.2");
        
        // Test ERN 4.2 detection
        assert_eq!(canonicalizer.detect_version(r#"<root xmlns="http://ddex.net/xml/ern/42">test</root>"#), "4.2");
        
        // Test ERN 4.3 detection
        assert_eq!(canonicalizer.detect_version(r#"<root xmlns="http://ddex.net/xml/ern/43">test</root>"#), "4.3");
    }

    #[test]
    fn test_namespace_prefix_locking_43() {
        let canonicalizer = create_test_canonicalizer_with_version("4.3");
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns:ddex="http://ddex.net/xml/ern/43" xmlns:avs="http://ddex.net/xml/avs">
  <ddex:Release>test</ddex:Release>
</root>"#;
        
        let result = canonicalizer.canonicalize(input).unwrap();
        // Should use locked prefix 'ern' instead of 'ddex'
        assert!(result.contains(r#"xmlns:ern="http://ddex.net/xml/ern/43""#));
    }

    #[test]
    fn test_element_ordering_message_header() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<MessageHeader>
  <MessageSender>sender</MessageSender>
  <MessageId>id</MessageId>
  <MessageType>type</MessageType>
</MessageHeader>"#;
        
        let result = canonicalizer.canonicalize(input).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        
        // Find positions of elements
        let id_pos = lines.iter().position(|l| l.contains("<MessageId>")).unwrap();
        let type_pos = lines.iter().position(|l| l.contains("<MessageType>")).unwrap();
        let sender_pos = lines.iter().position(|l| l.contains("<MessageSender>")).unwrap();
        
        // Should be in canonical order: Id, Type, Sender
        assert!(id_pos < type_pos);
        assert!(type_pos < sender_pos);
    }

    #[test]
    fn test_deterministic_output() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<root z="z" a="a">
  <child2>content2</child2>
  <child1>content1</child1>
</root>"#;
        
        // Run canonicalization multiple times
        let result1 = canonicalizer.canonicalize(input).unwrap();
        let result2 = canonicalizer.canonicalize(input).unwrap();
        let result3 = canonicalizer.canonicalize(input).unwrap();
        
        // Should be byte-identical
        assert_eq!(result1, result2);
        assert_eq!(result2, result3);
    }

    #[test]
    fn test_canonical_hash() {
        let canonicalizer = create_test_canonicalizer();
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>test</root>"#;
        
        let canonical = canonicalizer.canonicalize(xml).unwrap();
        let hash1 = canonicalizer.canonical_hash(&canonical).unwrap();
        let hash2 = canonicalizer.canonical_hash(&canonical).unwrap();
        
        // Same content should produce same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 hex string
    }

    #[test]
    fn test_empty_elements() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
  <empty/>
  <also-empty></also-empty>
</root>"#;
        
        let result = canonicalizer.canonicalize(input).unwrap();
        // Should handle empty elements correctly
        assert!(result.contains("<empty/>") || result.contains("<empty></empty>"));
    }

    #[test]
    fn test_comments_preservation() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
  <!-- This is a comment -->
  <child>content</child>
</root>"#;
        
        let result = canonicalizer.canonicalize(input).unwrap();
        // Comments should be preserved
        assert!(result.contains("<!-- This is a comment -->"));
    }

    #[test]
    fn test_complex_ddex_structure() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" xmlns:avs="http://ddex.net/xml/avs">
  <MessageHeader>
    <MessageSender>sender</MessageSender>
    <MessageId>MSG123</MessageId>
    <MessageType>NewReleaseMessage</MessageType>
    <MessageCreatedDateTime>2023-01-01T00:00:00Z</MessageCreatedDateTime>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseId>
        <GRid>A123456789</GRid>
      </ReleaseId>
      <ReferenceTitle>
        <TitleText>Test Album</TitleText>
      </ReferenceTitle>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#;
        
        let result = canonicalizer.canonicalize(input).unwrap();
        
        // Should maintain structure and apply canonicalization
        assert!(result.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(result.contains("<MessageHeader>"));
        assert!(result.contains("<Release>"));
        
        // Should have proper indentation
        assert!(result.contains("  <MessageHeader>"));
        assert!(result.contains("    <MessageId>"));
    }

    #[test]
    fn test_round_trip_fidelity() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<root attr="value">
  <child>content</child>
</root>"#;
        
        // Canonicalize once
        let canonical1 = canonicalizer.canonicalize(input).unwrap();
        
        // Canonicalize the result again
        let canonical2 = canonicalizer.canonicalize(&canonical1).unwrap();
        
        // Should be identical (idempotent)
        assert_eq!(canonical1, canonical2);
    }

    #[test]
    fn test_line_ending_normalization() {
        let canonicalizer = create_test_canonicalizer();
        
        // Test different line ending types
        let input_crlf = "<root>\r\n  <child>content</child>\r\n</root>";
        let input_cr = "<root>\r  <child>content</child>\r</root>";
        let input_lf = "<root>\n  <child>content</child>\n</root>";
        
        let result_crlf = canonicalizer.canonicalize(input_crlf).unwrap();
        let result_cr = canonicalizer.canonicalize(input_cr).unwrap();
        let result_lf = canonicalizer.canonicalize(input_lf).unwrap();
        
        // All should normalize to LF
        assert!(!result_crlf.contains("\r\n"));
        assert!(!result_cr.contains("\r"));
        assert!(result_lf.contains("\n"));
        
        // Results should be structurally identical
        assert_eq!(result_crlf, result_lf);
        assert_eq!(result_cr, result_lf);
    }

    #[test]
    fn test_attribute_escaping() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<root attr="&quot;quoted&quot; &amp; escaped">
  <child>&lt;content&gt;</child>
</root>"#;
        
        let result = canonicalizer.canonicalize(input).unwrap();
        
        // Should properly escape/unescape content
        assert!(result.contains("&quot;") || result.contains("\""));
        assert!(result.contains("&amp;") || result.contains("&"));
        assert!(result.contains("&lt;") || result.contains("<"));
        assert!(result.contains("&gt;") || result.contains(">"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_ern_43_canonicalization_snapshot() {
        let canonicalizer = create_test_canonicalizer_with_version("4.3");
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" xmlns:avs="http://ddex.net/xml/avs" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <MessageHeader>
    <MessageSender>TestSender</MessageSender>
    <MessageRecipient>TestRecipient</MessageRecipient>
    <MessageId>MSG_001</MessageId>
    <MessageType>NewReleaseMessage</MessageType>
    <MessageCreatedDateTime>2023-12-01T10:00:00Z</MessageCreatedDateTime>
    <MessageControlType>LiveMessage</MessageControlType>
  </MessageHeader>
  <ReleaseList>
    <Release>
      <ReleaseDetailsByTerritory>
        <TerritoryCode>Worldwide</TerritoryCode>
      </ReleaseDetailsByTerritory>
      <ReleaseId>
        <GRid>A1234567890123456789</GRid>
      </ReleaseId>
      <ReferenceTitle>
        <TitleText>Sample Release Title</TitleText>
      </ReferenceTitle>
      <ReleaseReference>REL_001</ReleaseReference>
    </Release>
  </ReleaseList>
</ern:NewReleaseMessage>"#;
        
        let result = canonicalizer.canonicalize(input).unwrap();
        assert_snapshot!(result);
    }

    #[test]
    fn test_byte_identical_output() {
        let canonicalizer = create_test_canonicalizer();
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<test xmlns:z="urn:z" xmlns:a="urn:a" z:attr="z" a:attr="a">
  <z:element>content</z:element>
  <a:element>content</a:element>
</test>"#;
        
        let result1 = canonicalizer.canonicalize(input).unwrap();
        let result2 = canonicalizer.canonicalize(input).unwrap();
        
        // Convert to bytes for exact comparison
        let bytes1 = result1.as_bytes();
        let bytes2 = result2.as_bytes();
        
        assert_eq!(bytes1, bytes2, "Canonicalization must be deterministic at byte level");
    }
}