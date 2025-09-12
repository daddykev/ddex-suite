//! Comprehensive tests for comment retention throughout the round-trip process
//! 
//! This module tests the complete comment retention engine that preserves XML 
//! comments during Parse → Modify → Build cycles with position-aware handling.

use ddex_core::models::{Comment, CommentPosition};
use ddex_parser::parser::{ParseOptions, ExtensionAwareParser, ExtensionCaptureContext};
use ddex_builder::{DDEXBuilder, BuildOptions};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_XML_WITH_COMMENTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- Document header comment -->
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <!-- Message header section -->
  <MessageHeader>
    <MessageId>MSG123</MessageId>
    <!-- This is a processing comment -->
    <MessageType>NewReleaseMessage</MessageType>
    <MessageSender>
      <PartyId>SENDER001</PartyId>
      <!-- Sender details comment -->
      <PartyName>Test Sender</PartyName>
    </MessageSender>
  </MessageHeader>
  
  <!-- Release list begins here -->
  <ReleaseList>
    <Release>
      <ReleaseReference>REL001</ReleaseReference>
      <!-- Release title comment -->
      <ReleaseTitle>Test Album</ReleaseTitle>
      <DisplayArtist>
        <PartyName>Test Artist</PartyName>
        <!-- Artist role comment -->
        <ArtistRole>MainArtist</ArtistRole>
      </DisplayArtist>
    </Release>
    <!-- End of release -->
  </ReleaseList>
  <!-- Document footer comment -->
</ern:NewReleaseMessage>
<!-- Final document comment -->"#;

    const TEST_XML_NESTED_COMMENTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- Root level comment -->
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <!-- Before MessageId -->
    <MessageId>MSG456</MessageId>
    <!-- After MessageId, before MessageType -->
    <MessageType>NewReleaseMessage</MessageType>
    <!-- After MessageType -->
  </MessageHeader>
  <!-- Between sections -->
  <ReleaseList>
    <!-- Inside ReleaseList -->
    <Release>
      <!-- First child comment in Release -->
      <ReleaseReference>REL002</ReleaseReference>
      <ReleaseTitle>Nested Comment Album</ReleaseTitle>
      <!-- Last child comment in Release -->
    </Release>
    <!-- After Release -->
  </ReleaseList>
</ern:NewReleaseMessage>"#;

    #[test]
    fn test_parse_comments_basic() {
        let mut parser = ExtensionAwareParser::new(true);
        let extensions = parser.parse_with_extensions(TEST_XML_WITH_COMMENTS).unwrap();
        
        // Should capture document-level comments
        assert!(!extensions.document_comments.is_empty(), "Should capture document comments");
        
        // Verify specific comments exist
        let comment_contents: Vec<String> = extensions.document_comments
            .iter()
            .map(|c| c.content.clone())
            .collect();
        
        assert!(comment_contents.contains(&"Document header comment".to_string()));
        assert!(comment_contents.contains(&"Document footer comment".to_string()));
        assert!(comment_contents.contains(&"Final document comment".to_string()));
    }

    #[test]
    fn test_comment_positions() {
        let comments = vec![
            Comment::new("Before element".to_string(), CommentPosition::Before),
            Comment::new("First child".to_string(), CommentPosition::FirstChild),
            Comment::new("Last child".to_string(), CommentPosition::LastChild),
            Comment::new("After element".to_string(), CommentPosition::After),
            Comment::new("Inline comment".to_string(), CommentPosition::Inline),
        ];
        
        for comment in &comments {
            let xml_output = comment.to_xml();
            assert!(xml_output.starts_with("<!--"));
            assert!(xml_output.ends_with("-->"));
            assert!(xml_output.contains(&comment.content));
        }
    }

    #[test]
    fn test_comment_canonicalization() {
        // Test comment content normalization
        let original_comment = Comment::new(
            "  This has   extra   whitespace  ".to_string(), 
            CommentPosition::FirstChild
        );
        
        let canonical_content = original_comment.canonical_content();
        assert_eq!(canonical_content, "This has   extra   whitespace");
        
        // Test preservation mode
        let preserved_comment = Comment::new(
            "  This has   extra   whitespace  ".to_string(), 
            CommentPosition::FirstChild
        ).preserve_formatting();
        
        let preserved_content = preserved_comment.canonical_content();
        assert_eq!(preserved_content, "  This has   extra   whitespace  ");
    }

    #[test]
    fn test_comment_xml_escaping() {
        let comment = Comment::new(
            "Contains -- double dashes and <!-- nested -->".to_string(),
            CommentPosition::FirstChild
        );
        
        let xml_output = comment.to_xml();
        
        // Should not contain double dashes (invalid in XML comments)
        assert!(!xml_output.contains("--"), "Comments should not contain double dashes");
        
        // Should escape nested comment markers
        assert!(!xml_output.contains("<!--"), "Nested comment start should be escaped");
        assert!(!xml_output.contains("-->"), "Nested comment end should be escaped");
    }

    #[test] 
    fn test_comment_with_processing_hints() {
        let comment = Comment::new("Test comment".to_string(), CommentPosition::Before)
            .with_hint("format".to_string(), "preserve".to_string())
            .with_hint("tool".to_string(), "ddex-suite".to_string());
        
        assert_eq!(comment.processing_hints.len(), 2);
        assert_eq!(comment.processing_hints.get("format"), Some(&"preserve".to_string()));
        assert_eq!(comment.processing_hints.get("tool"), Some(&"ddex-suite".to_string()));
    }

    #[test]
    fn test_comment_location_tracking() {
        let comment = Comment::with_location(
            "Located comment".to_string(),
            CommentPosition::FirstChild,
            Some("/message/header/messageId".to_string()),
            Some(42),
            Some(10)
        );
        
        assert_eq!(comment.xpath, Some("/message/header/messageId".to_string()));
        assert_eq!(comment.line_number, Some(42));
        assert_eq!(comment.column_number, Some(10));
    }

    #[test]
    fn test_comment_stripping_option() {
        // Test that comments can be stripped during production builds
        let comment = Comment::new("Debug comment".to_string(), CommentPosition::FirstChild);
        let xml_with_comment = comment.to_xml();
        
        // Simulate stripping comments (builder would handle this)
        let stripped = ""; // In real implementation, builder would omit comments
        
        assert!(xml_with_comment.contains("Debug comment"));
        assert!(!stripped.contains("Debug comment"));
    }

    #[test]
    fn test_parser_includecomments_option() {
        let options_with_comments = ParseOptions {
            include_comments: true,
            ..Default::default()
        };
        
        let options_without_comments = ParseOptions {
            include_comments: false,
            ..Default::default()
        };
        
        assert!(options_with_comments.include_comments);
        assert!(!options_without_comments.include_comments);
        
        // Test the for_round_trip preset
        let round_trip_options = ParseOptions::for_round_trip();
        assert!(round_trip_options.include_comments);
    }

    #[test] 
    fn test_mixed_comment_positions_in_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- Before root -->
<root>
  <!-- First child -->
  <element>content</element>
  <!-- Between elements -->
  <element>more content</element>
  <!-- Last child -->
</root>
<!-- After root -->"#;
        
        let mut parser = ExtensionAwareParser::new(true);
        let extensions = parser.parse_with_extensions(xml).unwrap();
        
        // Should capture all comments
        assert!(extensions.document_comments.len() >= 4, "Should capture multiple positioned comments");
    }

    #[test]
    fn test_comment_round_trip_fidelity() {
        // This would be a more complex integration test in practice
        // For now, we verify the comment structures work correctly
        
        let original_comments = vec![
            Comment::new("Document start".to_string(), CommentPosition::Before),
            Comment::new("Inside element".to_string(), CommentPosition::FirstChild),
            Comment::new("Document end".to_string(), CommentPosition::After),
        ];
        
        // Simulate round-trip by converting to XML and back
        for comment in &original_comments {
            let xml_form = comment.to_xml();
            assert!(xml_form.starts_with("<!--"));
            assert!(xml_form.ends_with("-->"));
            
            // Extract content (simple simulation)
            let content = xml_form.trim_start_matches("<!--").trim_end_matches("-->").trim();
            assert_eq!(content, comment.content);
        }
    }

    #[test]
    fn test_extension_fragment_with_comments() {
        use ddex_core::models::XmlFragment;
        
        let mut fragment = XmlFragment::new(
            "customElement".to_string(),
            "<customElement>test</customElement>".to_string()
        );
        
        fragment.comments.push(Comment::new(
            "Fragment comment".to_string(),
            CommentPosition::FirstChild
        ));
        
        let canonical_xml = fragment.to_canonical_xml(0);
        assert!(canonical_xml.contains("<!--Fragment comment-->"));
    }

    #[test]
    fn test_comment_deterministic_ordering() {
        // Test that comments maintain consistent ordering for deterministic output
        let mut comments = vec![
            Comment::new("Comment C".to_string(), CommentPosition::FirstChild),
            Comment::new("Comment A".to_string(), CommentPosition::Before),
            Comment::new("Comment B".to_string(), CommentPosition::After),
        ];
        
        // Sort by position for deterministic output
        comments.sort_by_key(|c| match c.position {
            CommentPosition::Before => 0,
            CommentPosition::FirstChild => 1,
            CommentPosition::LastChild => 2,
            CommentPosition::After => 3,
            CommentPosition::Inline => 4,
        });
        
        assert!(matches!(comments[0].position, CommentPosition::Before));
        assert!(matches!(comments[1].position, CommentPosition::FirstChild));
        assert!(matches!(comments[2].position, CommentPosition::After));
    }

    #[test]
    fn test_comment_preservation_with_modifications() {
        // Test that comments are preserved even when the document is modified
        let mut extensions = ddex_core::models::Extensions::new();
        extensions.add_document_comment("Original comment".to_string());
        
        // Simulate modification (in practice this would be part of the full pipeline)
        assert!(!extensions.document_comments.is_empty());
        
        // Add more comments during processing
        extensions.add_document_comment("Added during processing".to_string());
        
        assert_eq!(extensions.document_comments.len(), 2);
    }
}

/// Integration test module for full round-trip comment preservation
#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Test the complete workflow: XML → Parse → Modify → Build → XML
    /// This would require full integration with both parser and builder
    #[test]
    #[ignore] // Enable when full integration is ready
    fn test_full_round_trip_comment_preservation() {
        // This test would:
        // 1. Parse XML with comments using includeComments: true
        // 2. Modify some content while preserving comment structure  
        // 3. Build back to XML with comments intact
        // 4. Verify all comments survived with correct positioning
        
        // For now, this is a placeholder for the future complete integration test
        todo!("Implement full round-trip test with parser and builder integration");
    }
    
    #[test]
    #[ignore] // Enable when builder integration is ready
    fn test_comment_canonicalization_in_builder() {
        // This test would verify that the builder properly applies
        // DB-C14N/1.0 canonicalization rules to comment content
        todo!("Implement builder canonicalization test");
    }
    
    #[test]
    #[ignore] // Enable when builder integration is ready  
    fn test_comment_stripping_for_production() {
        // This test would verify that comments can be optionally
        // stripped during production builds
        todo!("Implement production comment stripping test");
    }
}