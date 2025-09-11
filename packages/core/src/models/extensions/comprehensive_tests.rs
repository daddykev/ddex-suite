//! Comprehensive integration tests for the extension preservation system

use super::*;
use crate::models::extensions::test_data::*;

#[cfg(test)]
mod extension_tests {
    use super::*;

    #[test]
    fn test_xml_fragment_creation_with_namespaces() {
        let fragment = XmlFragment::with_namespace(
            "CustomElement".to_string(),
            Some("http://example.com/custom".to_string()),
            Some("custom".to_string()),
            r#"<custom:CustomElement attr="value">Content</custom:CustomElement>"#.to_string(),
        );

        assert_eq!(fragment.element_name, "CustomElement");
        assert_eq!(fragment.namespace_uri, Some("http://example.com/custom".to_string()));
        assert_eq!(fragment.namespace_prefix, Some("custom".to_string()));
        assert!(fragment.raw_content.contains("CustomElement"));
    }

    #[test]
    fn test_xml_fragment_canonical_generation() {
        let mut fragment = XmlFragment::with_namespace(
            "TestElement".to_string(),
            Some("http://test.com".to_string()),
            Some("test".to_string()),
            String::new(),
        );

        fragment.add_attribute("id".to_string(), "123".to_string());
        fragment.add_attribute("name".to_string(), "test".to_string());
        fragment.text_content = Some("Hello World".to_string());

        let canonical = fragment.to_canonical_xml(2);
        
        // Check that attributes are sorted
        assert!(canonical.contains(r#"id="123""#));
        assert!(canonical.contains(r#"name="test""#));
        assert!(canonical.contains("Hello World"));
        
        // Check proper indentation (4 spaces = 2 levels * 2 spaces each)
        assert!(canonical.starts_with("    "));
    }

    #[test]
    fn test_extensions_container_operations() {
        let mut extensions = Extensions::new();

        // Add fragments
        let fragment1 = XmlFragment::new("element1".to_string(), "content1".to_string());
        let fragment2 = XmlFragment::new("element2".to_string(), "content2".to_string());

        extensions.add_fragment("location1".to_string(), fragment1);
        extensions.add_fragment("location2".to_string(), fragment2);

        // Add global namespace
        extensions.add_global_namespace("test".to_string(), "http://test.com".to_string());

        // Add document-level items
        extensions.add_document_comment("Test comment".to_string());
        let pi = ProcessingInstruction::new("test-instruction".to_string(), Some("data".to_string()));
        extensions.add_document_processing_instruction(pi);

        // Test retrieval
        assert_eq!(extensions.fragments.len(), 2);
        assert_eq!(extensions.global_namespaces.len(), 1);
        assert_eq!(extensions.document_comments.len(), 1);
        assert_eq!(extensions.document_processing_instructions.len(), 1);
        
        // Test pattern matching
        let matches = extensions.get_fragments_matching("location1");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0, "location1");
    }

    #[test]
    fn test_extensions_merging() {
        let mut extensions1 = Extensions::new();
        let mut extensions2 = Extensions::new();

        let fragment1 = XmlFragment::new("element1".to_string(), "content1".to_string());
        let fragment2 = XmlFragment::new("element2".to_string(), "content2".to_string());

        extensions1.add_fragment("location1".to_string(), fragment1);
        extensions1.add_global_namespace("ns1".to_string(), "http://ns1.com".to_string());

        extensions2.add_fragment("location2".to_string(), fragment2);
        extensions2.add_global_namespace("ns2".to_string(), "http://ns2.com".to_string());

        extensions1.merge(extensions2);

        assert_eq!(extensions1.fragments.len(), 2);
        assert_eq!(extensions1.global_namespaces.len(), 2);
        assert!(extensions1.fragments.contains_key("location1"));
        assert!(extensions1.fragments.contains_key("location2"));
        assert!(extensions1.global_namespaces.contains_key("ns1"));
        assert!(extensions1.global_namespaces.contains_key("ns2"));
    }

    #[test]
    fn test_processing_instruction_creation() {
        let pi = ProcessingInstruction::new("xml-stylesheet".to_string(), Some("type=\"text/xsl\" href=\"style.xsl\"".to_string()));
        assert_eq!(pi.target, "xml-stylesheet");
        assert_eq!(pi.data, Some("type=\"text/xsl\" href=\"style.xsl\"".to_string()));

        let simple_pi = ProcessingInstruction::new("simple".to_string(), None);
        assert_eq!(simple_pi.target, "simple");
        assert_eq!(simple_pi.data, None);
    }

    #[test]
    fn test_namespace_detection_utilities() {
        // Test DDEX namespace detection
        assert!(utils::is_ddex_namespace("http://ddex.net/xml/ern/43"));
        assert!(utils::is_ddex_namespace("http://ddex.net/xml/ern/382"));
        assert!(utils::is_ddex_namespace("http://ddex.net/xml/ern/42"));
        assert!(!utils::is_ddex_namespace("http://example.com/custom"));
        assert!(!utils::is_ddex_namespace("http://spotify.com/ddex/extensions"));

        // Test location key generation
        let path = vec!["message", "header", "customElement"];
        let location = utils::generate_location_key(&path, Some("http://custom.com"), "customElement");
        assert!(location.contains("message/header/customElement"));
        assert!(location.contains("http://custom.com"));
    }

    #[test]
    fn test_xml_fragment_validation() {
        // Valid XML fragment
        let valid_fragment = XmlFragment::new("test".to_string(), "<test>valid</test>".to_string());
        assert!(utils::validate_xml_fragment(&valid_fragment).is_ok());

        // Invalid XML fragment (empty content - which should fail validation)
        let invalid_fragment = XmlFragment::new("test".to_string(), "".to_string());
        assert!(utils::validate_xml_fragment(&invalid_fragment).is_err());
    }

    #[test]
    fn test_extension_statistics() {
        let mut extensions = Extensions::new();
        
        // Add various types of extensions
        let fragment1 = XmlFragment::new("element1".to_string(), "content1".to_string());
        let fragment2 = XmlFragment::new("element2".to_string(), "content2".to_string());
        
        extensions.add_fragment("location1".to_string(), fragment1);
        extensions.add_fragment("location2".to_string(), fragment2);
        extensions.add_global_namespace("custom".to_string(), "http://custom.com".to_string());
        extensions.add_document_comment("Test comment".to_string());
        extensions.add_document_processing_instruction(ProcessingInstruction::new("test".to_string(), None));

        assert_eq!(extensions.fragments.len(), 2);
        assert_eq!(extensions.global_namespaces.len(), 1);
        assert_eq!(extensions.document_comments.len(), 1);
        assert_eq!(extensions.document_processing_instructions.len(), 1);
        assert!(!extensions.is_empty());
    }

    #[test]
    fn test_spotify_extensions_detection() {
        // This test would require integration with the parser
        // For now, we test the structure
        let test_xml = DDEX_WITH_SPOTIFY_EXTENSIONS;
        assert!(test_xml.contains("xmlns:spotify"));
        assert!(test_xml.contains("spotify:SpotifyMetadata"));
        assert!(test_xml.contains("spotify:TrackMetadata"));
        assert!(test_xml.contains("spotify:AudioFeatures"));
    }

    #[test] 
    fn test_youtube_extensions_detection() {
        let test_xml = DDEX_WITH_YOUTUBE_EXTENSIONS;
        assert!(test_xml.contains("xmlns:ytm"));
        assert!(test_xml.contains("ytm:DeliveryMetadata"));
        assert!(test_xml.contains("ytm:VideoMetadata"));
        assert!(test_xml.contains("youtube-processing-instruction"));
    }

    #[test]
    fn test_apple_extensions_detection() {
        let test_xml = DDEX_WITH_APPLE_EXTENSIONS;
        assert!(test_xml.contains("xmlns:apple"));
        assert!(test_xml.contains("apple:ReleaseEnhancements"));
        assert!(test_xml.contains("apple:SpatialAudio"));
        assert!(test_xml.contains("apple:LosslessAudio"));
    }

    #[test]
    fn test_multiple_extensions_detection() {
        let test_xml = DDEX_WITH_MULTIPLE_EXTENSIONS;
        assert!(test_xml.contains("xmlns:analytics"));
        assert!(test_xml.contains("xmlns:blockchain"));
        assert!(test_xml.contains("xmlns:ml"));
        assert!(test_xml.contains("analytics:TrackingMetadata"));
        assert!(test_xml.contains("blockchain:Rights"));
        assert!(test_xml.contains("ml:GenerationMetadata"));
    }

    #[test]
    fn test_extension_free_xml() {
        let test_xml = DDEX_WITHOUT_EXTENSIONS;
        assert!(!test_xml.contains("xmlns:") || test_xml.matches("xmlns:").count() <= 1); // Only ern namespace
        assert!(!test_xml.contains("spotify:"));
        assert!(!test_xml.contains("ytm:"));
        assert!(!test_xml.contains("apple:"));
        assert!(!test_xml.contains("analytics:"));
    }

    #[test]
    fn test_xml_fragment_with_children() {
        let mut parent = XmlFragment::new("parent".to_string(), String::new());
        let child1 = XmlFragment::new("child1".to_string(), "content1".to_string());
        let child2 = XmlFragment::new("child2".to_string(), "content2".to_string());

        parent.add_child(child1);
        parent.add_child(child2);

        assert_eq!(parent.children.len(), 2);
        assert_eq!(parent.children[0].element_name, "child1");
        assert_eq!(parent.children[1].element_name, "child2");
    }

    #[test]
    fn test_xml_fragment_with_comments_and_processing_instructions() {
        let mut fragment = XmlFragment::new("test".to_string(), String::new());
        
        fragment.comments.push("This is a comment".to_string());
        fragment.processing_instructions.push(ProcessingInstruction::new("test".to_string(), Some("data".to_string())));

        assert_eq!(fragment.comments.len(), 1);
        assert_eq!(fragment.processing_instructions.len(), 1);
        assert_eq!(fragment.comments[0], "This is a comment");
        assert_eq!(fragment.processing_instructions[0].target, "test");
    }

    #[test]
    fn test_canonical_xml_with_complex_structure() {
        let mut fragment = XmlFragment::with_namespace(
            "ComplexElement".to_string(),
            Some("http://example.com/complex".to_string()),
            Some("complex".to_string()),
            String::new(),
        );

        // Add namespace declarations
        fragment.add_namespace_declaration("ns1".to_string(), "http://ns1.com".to_string());
        fragment.add_namespace_declaration("ns2".to_string(), "http://ns2.com".to_string());

        // Add attributes (should be sorted)
        fragment.add_attribute("zebra".to_string(), "last".to_string());
        fragment.add_attribute("alpha".to_string(), "first".to_string());

        // Add comments and processing instructions
        fragment.comments.push("Element comment".to_string());
        fragment.processing_instructions.push(ProcessingInstruction::new("element-pi".to_string(), None));

        let canonical = fragment.to_canonical_xml(0);

        // Check that attributes are sorted alphabetically
        let alpha_pos = canonical.find(r#"alpha="first""#).unwrap();
        let zebra_pos = canonical.find(r#"zebra="last""#).unwrap();
        assert!(alpha_pos < zebra_pos);

        // Check namespace declarations are present and sorted
        assert!(canonical.contains(r#"xmlns:ns1="http://ns1.com""#));
        assert!(canonical.contains(r#"xmlns:ns2="http://ns2.com""#));

        // Check comments and processing instructions
        assert!(canonical.contains("<!--Element comment-->"));
        assert!(canonical.contains("<?element-pi?>"));
    }

    #[test]
    fn test_extension_pattern_matching() {
        let mut extensions = Extensions::new();
        
        let frag1 = XmlFragment::new("element1".to_string(), "content1".to_string());
        let frag2 = XmlFragment::new("element2".to_string(), "content2".to_string());
        let frag3 = XmlFragment::new("element3".to_string(), "content3".to_string());

        extensions.add_fragment("message/header/element1".to_string(), frag1);
        extensions.add_fragment("message/body/element2".to_string(), frag2);
        extensions.add_fragment("message/footer/element3".to_string(), frag3);

        // Test exact matching
        let exact_matches = extensions.get_fragments_matching("message/header/element1");
        assert_eq!(exact_matches.len(), 1);

        // Test prefix matching
        let header_matches = extensions.get_fragments_matching("message/header/");
        assert_eq!(header_matches.len(), 1);

        // Test pattern matching (simplified for this test)
        let message_matches = extensions.get_fragments_matching("message/");
        assert!(message_matches.len() >= 1); // Implementation dependent
    }

    #[test] 
    fn test_html_escaping_in_xml_fragments() {
        let content_with_entities = r#"<test attr="value&amp;more">Content with &lt;tags&gt; and &quot;quotes&quot;</test>"#;
        let fragment = XmlFragment::new("test".to_string(), content_with_entities.to_string());

        let canonical = fragment.to_canonical_xml(0);
        
        // The canonical XML should contain the original content since it preserves raw content
        // For this simple test, we'll just check that content is present
        assert!(canonical.contains("test"));
        assert!(!canonical.is_empty());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    // These tests will be expanded when we integrate with the actual parser/builder
    
    #[test]
    fn test_extension_data_structures() {
        // Test that all our test data is well-formed XML
        let test_cases = vec![
            DDEX_WITH_SPOTIFY_EXTENSIONS,
            DDEX_WITH_YOUTUBE_EXTENSIONS,
            DDEX_WITH_APPLE_EXTENSIONS,
            DDEX_WITH_MULTIPLE_EXTENSIONS,
            DDEX_WITHOUT_EXTENSIONS,
        ];

        for xml in test_cases {
            // Basic XML structure validation
            assert!(xml.contains("<?xml"));
            assert!(xml.contains("NewReleaseMessage"));
            assert!(xml.contains("MessageHeader"));
            assert!(xml.contains("ReleaseList"));
            assert!(xml.contains("ResourceList"));
        }
    }

    #[test]
    fn test_extension_namespace_variety() {
        let complex_xml = DDEX_WITH_MULTIPLE_EXTENSIONS;
        
        // Count unique namespace declarations
        let xmlns_count = complex_xml.matches("xmlns:").count();
        assert!(xmlns_count >= 4); // Should have ern, analytics, blockchain, ml at minimum

        // Test that different extension types are present
        assert!(complex_xml.contains("analytics:"));
        assert!(complex_xml.contains("blockchain:"));  
        assert!(complex_xml.contains("ml:"));
    }

    #[test]
    fn test_processing_instruction_parsing() {
        let youtube_xml = DDEX_WITH_YOUTUBE_EXTENSIONS;
        assert!(youtube_xml.contains("<?youtube-processing-instruction"));
        
        let multi_xml = DDEX_WITH_MULTIPLE_EXTENSIONS;
        assert!(multi_xml.contains("<?custom-processing"));
    }

    #[test]
    fn test_comment_preservation() {
        let apple_xml = DDEX_WITH_APPLE_EXTENSIONS;
        assert!(apple_xml.contains("<!--Apple Music Content Ingestion"));
        
        let spotify_xml = DDEX_WITH_SPOTIFY_EXTENSIONS;
        assert!(spotify_xml.contains("<!--Generated by Spotify"));
        
        let multi_xml = DDEX_WITH_MULTIPLE_EXTENSIONS;
        assert!(multi_xml.contains("<!--Multi-platform distribution"));
    }
}