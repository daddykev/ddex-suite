//! # Comprehensive Namespace Management Integration Tests
//! 
//! This module contains extensive tests for the namespace management system,
//! covering detection, canonical transformations, round-trip fidelity, and 
//! complex scenarios with multiple namespace versions and custom extensions.

use ddex_core::namespace::{
    NamespaceRegistry, NamespaceScope, NamespaceInfo, DDEXStandard, 
    NamespaceWarning, NamespaceError, ConflictResolution
};
use ddex_core::models::versions::ERNVersion;
use ddex_parser::parser::namespace_detector::{NamespaceDetector, NamespaceContext, ResolvedName};
use ddex_builder::canonical::rules::CanonicalNamespaceManager;
use ddex_builder::namespace_minimizer::{NamespaceMinimizer, OptimizationStrategy, AdvancedNamespaceMinimizer};
use ddex_builder::ast::{AST, Element, Node};
use indexmap::{IndexMap, IndexSet};
use std::io::Cursor;

/// Test the complete namespace management workflow
#[cfg(test)]
mod comprehensive_tests {
    use super::*;

    #[test]
    fn test_complete_namespace_workflow() {
        // Step 1: Create sample DDEX XML with various namespaces
        let sample_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" 
                       xmlns:avs="http://ddex.net/xml/avs"
                       xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                       xmlns:custom="http://example.com/custom">
    <ern:MessageHeader>
        <ern:MessageId>MSG-12345</ern:MessageId>
        <ern:MessageType>NewReleaseMessage</ern:MessageType>
        <ern:MessageCreatedDateTime>2024-01-01T00:00:00Z</ern:MessageCreatedDateTime>
        <custom:ExtensionData>Custom content</custom:ExtensionData>
    </ern:MessageHeader>
    <ern:ReleaseList>
        <ern:Release>
            <ern:ReleaseId>
                <ern:GRid>A12345678901</ern:GRid>
            </ern:ReleaseId>
            <ern:ReleaseDetailsByTerritory>
                <ern:TerritoryCode>Worldwide</ern:TerritoryCode>
                <ern:DisplayTitleText>Test Release</ern:DisplayTitleText>
                <avs:Genre>Rock</avs:Genre>
                <custom:ReleaseMetadata>Extended metadata</custom:ReleaseMetadata>
            </ern:ReleaseDetailsByTerritory>
        </ern:Release>
    </ern:ReleaseList>
</ern:NewReleaseMessage>"#;

        // Step 2: Parse and detect namespaces
        let mut detector = NamespaceDetector::new();
        let cursor = Cursor::new(sample_xml.as_bytes());
        let detection_result = detector.detect_from_xml(cursor).unwrap();

        // Verify detection results
        assert_eq!(detection_result.version, Some(ERNVersion::V4_3));
        assert!(detection_result.declarations.contains_key("ern"));
        assert!(detection_result.declarations.contains_key("avs"));
        assert!(detection_result.declarations.contains_key("xsi"));
        assert!(detection_result.declarations.contains_key("custom"));
        
        // Check custom namespaces
        assert_eq!(detection_result.custom_namespaces.len(), 1);
        assert_eq!(detection_result.custom_namespaces[0].uri, "http://example.com/custom");

        // Step 3: Apply canonical transformations
        let canonical_manager = CanonicalNamespaceManager::new();
        let canonical_declarations = canonical_manager.canonicalize_namespaces(
            &detection_result.declarations, 
            "4.3"
        );

        // Verify canonical transformations
        assert!(canonical_declarations.contains_key("ern"));
        assert!(canonical_declarations.contains_key("avs"));
        assert_eq!(
            canonical_declarations.get("ern"), 
            Some(&"http://ddex.net/xml/ern/43".to_string())
        );

        // Step 4: Test namespace minimization
        let minimizer = NamespaceMinimizer::new(ERNVersion::V4_3);
        
        // Create test AST
        let mut root = Element::new("NewReleaseMessage")
            .with_namespace("http://ddex.net/xml/ern/43");
        root.add_child(Element::new("MessageHeader")
            .with_namespace("http://ddex.net/xml/ern/43"));
        
        let test_ast = AST {
            root,
            namespaces: detection_result.declarations.clone(),
            schema_location: None,
        };

        let minimization_result = minimizer.minimize(test_ast).unwrap();
        assert!(!minimization_result.root_namespaces.is_empty());
        
        println!("Workflow completed successfully!");
        println!("Root namespaces: {:?}", minimization_result.root_namespaces);
        println!("Warnings: {:?}", minimization_result.warnings);
    }

    #[test]
    fn test_namespace_registry_comprehensive() {
        let mut registry = NamespaceRegistry::new();

        // Test all ERN version detection
        assert_eq!(
            registry.detect_version("http://ddex.net/xml/ern/382"),
            Some(ERNVersion::V3_8_2)
        );
        assert_eq!(
            registry.detect_version("http://ddex.net/xml/ern/42"),
            Some(ERNVersion::V4_2)
        );
        assert_eq!(
            registry.detect_version("http://ddex.net/xml/ern/43"),
            Some(ERNVersion::V4_3)
        );

        // Test version-specific namespaces
        let v43_namespaces = registry.get_version_namespaces(&ERNVersion::V4_3);
        assert!(v43_namespaces.contains(&"http://ddex.net/xml/ern/43".to_string()));
        assert!(v43_namespaces.contains(&"http://ddex.net/xml/avs".to_string()));

        // Test custom namespace registration
        let custom_ns = NamespaceInfo {
            uri: "http://musiclabel.com/extensions".to_string(),
            preferred_prefix: "label".to_string(),
            alternative_prefixes: vec!["musiclabel".to_string()],
            standard: DDEXStandard::Custom("MusicLabel".to_string()),
            version: Some("1.0".to_string()),
            required: false,
        };

        assert!(registry.register_custom_namespace(custom_ns).is_ok());
        assert_eq!(
            registry.get_preferred_prefix("http://musiclabel.com/extensions"),
            Some("label")
        );

        // Test conflict detection
        let conflicting_ns = NamespaceInfo {
            uri: "http://conflicting.com/namespace".to_string(),
            preferred_prefix: "ern".to_string(), // This should conflict
            alternative_prefixes: vec![],
            standard: DDEXStandard::Custom("Conflicting".to_string()),
            version: None,
            required: false,
        };

        assert!(matches!(
            registry.register_custom_namespace(conflicting_ns),
            Err(NamespaceError::PrefixConflict(_))
        ));

        // Test unique prefix generation
        let unique_prefix = registry.generate_unique_prefix("ern");
        assert_ne!(unique_prefix, "ern");
        assert!(unique_prefix.starts_with("ern"));
    }

    #[test]
    fn test_namespace_scope_inheritance() {
        // Create nested scope structure
        let mut root_scope = NamespaceScope::new();
        root_scope.declare_namespace("ern".to_string(), "http://ddex.net/xml/ern/43".to_string());
        root_scope.declare_namespace("avs".to_string(), "http://ddex.net/xml/avs".to_string());

        let mut header_scope = root_scope.new_child();
        header_scope.declare_namespace("local".to_string(), "http://example.com/local".to_string());

        let mut element_scope = header_scope.new_child();
        element_scope.declare_namespace("temp".to_string(), "http://temporary.com/ns".to_string());

        // Test resolution from deepest scope
        assert_eq!(
            element_scope.resolve_prefix("ern"),
            Some("http://ddex.net/xml/ern/43".to_string())
        );
        assert_eq!(
            element_scope.resolve_prefix("local"),
            Some("http://example.com/local".to_string())
        );
        assert_eq!(
            element_scope.resolve_prefix("temp"),
            Some("http://temporary.com/ns".to_string())
        );

        // Test that parent scopes don't see child declarations
        assert_eq!(header_scope.resolve_prefix("temp"), None);
        assert_eq!(root_scope.resolve_prefix("local"), None);

        // Test URI-to-prefix resolution
        assert_eq!(
            element_scope.find_prefix_for_uri("http://ddex.net/xml/avs"),
            Some("avs".to_string())
        );

        // Test all active declarations
        let all_declarations = element_scope.get_all_declarations();
        assert_eq!(all_declarations.len(), 4);
        assert!(all_declarations.contains_key("ern"));
        assert!(all_declarations.contains_key("avs"));
        assert!(all_declarations.contains_key("local"));
        assert!(all_declarations.contains_key("temp"));
    }

    #[test]
    fn test_namespace_context_and_resolution() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" 
                   xmlns:avs="http://ddex.net/xml/avs"
                   xmlns:local="http://example.com/local">
    <MessageHeader xmlns:header="http://example.com/header">
        <MessageId>MSG001</MessageId>
        <avs:Genre>Rock</avs:Genre>
        <local:CustomData>Data</local:CustomData>
        <header:HeaderExtension>Extension</header:HeaderExtension>
    </MessageHeader>
</NewReleaseMessage>"#;

        let mut detector = NamespaceDetector::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = detector.detect_from_xml(cursor).unwrap();
        
        let context = NamespaceContext::from_detection_result(result);

        // Test element name resolution
        let resolved_message = context.resolve_element_name("NewReleaseMessage", None);
        match resolved_message {
            ResolvedName::Qualified { local_name, namespace_uri, prefix } => {
                assert_eq!(local_name, "NewReleaseMessage");
                assert_eq!(namespace_uri, "http://ddex.net/xml/ern/43");
                assert_eq!(prefix, "");
            },
            _ => panic!("Expected qualified name for default namespace"),
        }

        let resolved_genre = context.resolve_element_name("Genre", Some("avs"));
        match resolved_genre {
            ResolvedName::Qualified { local_name, namespace_uri, prefix } => {
                assert_eq!(local_name, "Genre");
                assert_eq!(namespace_uri, "http://ddex.net/xml/avs");
                assert_eq!(prefix, "avs");
            },
            _ => panic!("Expected qualified name for avs:Genre"),
        }

        // Test unresolved prefix
        let unresolved = context.resolve_element_name("Element", Some("unknown"));
        match unresolved {
            ResolvedName::Unresolved { local_name, prefix } => {
                assert_eq!(local_name, "Element");
                assert_eq!(prefix, Some("unknown".to_string()));
            },
            _ => panic!("Expected unresolved name for unknown prefix"),
        }
    }

    #[test]
    fn test_canonical_namespace_transformations() {
        let manager = CanonicalNamespaceManager::new();

        // Test ERN 4.3 canonical transformations
        let mut declarations = IndexMap::new();
        declarations.insert("custom_ern".to_string(), "http://ddex.net/xml/ern/43".to_string());
        declarations.insert("my_avs".to_string(), "http://ddex.net/xml/avs".to_string());
        declarations.insert("schema_instance".to_string(), "http://www.w3.org/2001/XMLSchema-instance".to_string());

        let canonical = manager.canonicalize_namespaces(&declarations, "4.3");

        // Should use locked prefixes
        assert_eq!(canonical.get("ern"), Some(&"http://ddex.net/xml/ern/43".to_string()));
        assert_eq!(canonical.get("avs"), Some(&"http://ddex.net/xml/avs".to_string()));
        assert_eq!(canonical.get("xsi"), Some(&"http://www.w3.org/2001/XMLSchema-instance".to_string()));

        // Test with conflicting prefixes
        let mut conflicting_declarations = IndexMap::new();
        conflicting_declarations.insert("ern".to_string(), "http://ddex.net/xml/ern/43".to_string());
        conflicting_declarations.insert("ern".to_string(), "http://custom.com/namespace".to_string()); // Should overwrite

        let canonical_conflicting = manager.canonicalize_namespaces(&conflicting_declarations, "4.3");
        // The result should handle the conflict appropriately
        assert!(!canonical_conflicting.is_empty());
    }

    #[test]
    fn test_namespace_minimization_strategies() {
        // Create test AST with redundant namespace declarations
        let mut root = Element::new("NewReleaseMessage")
            .with_namespace("http://ddex.net/xml/ern/43")
            .with_attr("xmlns:ern", "http://ddex.net/xml/ern/43")
            .with_attr("xmlns:avs", "http://ddex.net/xml/avs");

        let mut header = Element::new("MessageHeader")
            .with_namespace("http://ddex.net/xml/ern/43")
            .with_attr("xmlns:ern", "http://ddex.net/xml/ern/43"); // Redundant declaration

        header.add_child(Element::new("MessageId")
            .with_namespace("http://ddex.net/xml/ern/43")
            .with_text("MSG001"));

        root.add_child(header);

        let mut namespaces = IndexMap::new();
        namespaces.insert("ern".to_string(), "http://ddex.net/xml/ern/43".to_string());
        namespaces.insert("avs".to_string(), "http://ddex.net/xml/avs".to_string());

        let test_ast = AST {
            root,
            namespaces,
            schema_location: None,
        };

        // Test minimal strategy
        let minimal_minimizer = AdvancedNamespaceMinimizer::new(
            ERNVersion::V4_3, 
            OptimizationStrategy::Minimal
        );
        let minimal_result = minimal_minimizer.minimize(test_ast.clone()).unwrap();
        assert!(!minimal_result.root_namespaces.is_empty());

        // Test hoist all strategy
        let hoist_minimizer = AdvancedNamespaceMinimizer::new(
            ERNVersion::V4_3, 
            OptimizationStrategy::HoistAll
        );
        let hoist_result = hoist_minimizer.minimize(test_ast.clone()).unwrap();
        assert!(!hoist_result.root_namespaces.is_empty());

        // Test conservative strategy
        let conservative_minimizer = AdvancedNamespaceMinimizer::new(
            ERNVersion::V4_3, 
            OptimizationStrategy::Conservative
        );
        let conservative_result = conservative_minimizer.minimize(test_ast).unwrap();
        assert!(!conservative_result.root_namespaces.is_empty());
        assert!(!conservative_result.warnings.is_empty());
    }

    #[test]
    fn test_multi_version_namespace_handling() {
        // Test handling of different ERN versions in the same workflow
        let versions = vec![ERNVersion::V3_8_2, ERNVersion::V4_2, ERNVersion::V4_3];
        
        for version in versions {
            let registry = NamespaceRegistry::new();
            let version_namespaces = registry.get_version_namespaces(&version);
            
            // All versions should have ERN and AVS namespaces
            assert!(version_namespaces.iter().any(|ns| ns.contains("/ern/")));
            assert!(version_namespaces.iter().any(|ns| ns.contains("/avs")));
            
            // Test canonical rules for each version
            let manager = CanonicalNamespaceManager::new();
            let version_str = match version {
                ERNVersion::V3_8_2 => "3.8.2",
                ERNVersion::V4_2 => "4.2",
                ERNVersion::V4_3 => "4.3",
            };
            
            let mut test_declarations = IndexMap::new();
            let ern_uri = match version {
                ERNVersion::V3_8_2 => "http://ddex.net/xml/ern/382",
                ERNVersion::V4_2 => "http://ddex.net/xml/ern/42", 
                ERNVersion::V4_3 => "http://ddex.net/xml/ern/43",
            };
            test_declarations.insert("ern".to_string(), ern_uri.to_string());
            
            let canonical = manager.canonicalize_namespaces(&test_declarations, version_str);
            assert_eq!(canonical.get("ern"), Some(&ern_uri.to_string()));
        }
    }

    #[test]
    fn test_namespace_validation_and_warnings() {
        let registry = NamespaceRegistry::new();
        
        // Test with non-standard prefixes
        let mut declarations = IndexMap::new();
        declarations.insert("custom_ern".to_string(), "http://ddex.net/xml/ern/43".to_string());
        declarations.insert("my_avs".to_string(), "http://ddex.net/xml/avs".to_string());
        declarations.insert("unknown".to_string(), "http://unknown.com/namespace".to_string());
        
        let warnings = registry.validate_declarations(&declarations);
        
        // Should have warnings for non-standard prefixes and unknown namespace
        assert!(!warnings.is_empty());
        
        let non_standard_warnings: Vec<_> = warnings.iter()
            .filter(|w| matches!(w, NamespaceWarning::NonStandardPrefix { .. }))
            .collect();
        assert!(!non_standard_warnings.is_empty());
        
        let unknown_warnings: Vec<_> = warnings.iter()
            .filter(|w| matches!(w, NamespaceWarning::UnknownNamespace { .. }))
            .collect();
        assert!(!unknown_warnings.is_empty());
    }

    #[test]
    fn test_namespace_conflict_resolution() {
        let registry = NamespaceRegistry::new();
        
        // Test different conflict resolution strategies
        let strategies = vec![
            ConflictResolution::PreferFirst,
            ConflictResolution::PreferLatest,
            ConflictResolution::GenerateUnique,
        ];
        
        for strategy in strategies {
            let result = registry.resolve_prefix_conflict(
                "http://ddex.net/xml/ern/43",
                "existing_ern",
                "new_ern",
                strategy
            );
            
            assert!(result.is_ok());
            let resolved_prefix = result.unwrap();
            
            match strategy {
                ConflictResolution::PreferFirst => assert_eq!(resolved_prefix, "existing_ern"),
                ConflictResolution::PreferLatest => assert_eq!(resolved_prefix, "new_ern"),
                ConflictResolution::GenerateUnique => {
                    assert!(resolved_prefix.starts_with("new_ern"));
                    assert_ne!(resolved_prefix, "new_ern");
                },
                ConflictResolution::Error => unreachable!(),
            }
        }
        
        // Test error strategy
        let error_result = registry.resolve_prefix_conflict(
            "http://ddex.net/xml/ern/43",
            "existing_ern",
            "new_ern",
            ConflictResolution::Error
        );
        assert!(matches!(error_result, Err(NamespaceError::PrefixConflict(_))));
    }

    #[test]
    fn test_round_trip_namespace_fidelity() {
        // Test that namespace declarations are preserved through parse -> minimize -> build cycle
        let original_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" 
                       xmlns:avs="http://ddex.net/xml/avs"
                       xmlns:custom="http://example.com/custom">
    <ern:MessageHeader>
        <ern:MessageId>MSG001</ern:MessageId>
        <custom:ExtensionData>Custom</custom:ExtensionData>
    </ern:MessageHeader>
    <ern:ReleaseList>
        <ern:Release>
            <ern:ReleaseId>
                <ern:GRid>A1234567890123456789</ern:GRid>
            </ern:ReleaseId>
        </ern:Release>
    </ern:ReleaseList>
</ern:NewReleaseMessage>"#;

        // Step 1: Parse and detect namespaces
        let mut detector = NamespaceDetector::new();
        let cursor = Cursor::new(original_xml.as_bytes());
        let detection_result = detector.detect_from_xml(cursor).unwrap();

        // Step 2: Apply minimization
        let minimizer = NamespaceMinimizer::new(ERNVersion::V4_3);
        
        let mut test_ast = AST {
            root: Element::new("NewReleaseMessage")
                .with_namespace("http://ddex.net/xml/ern/43"),
            namespaces: detection_result.declarations.clone(),
            schema_location: None,
        };
        
        let minimization_result = minimizer.minimize(test_ast).unwrap();

        // Step 3: Verify that essential namespaces are preserved
        let final_namespaces = &minimization_result.root_namespaces;
        
        // Should have ERN namespace
        assert!(final_namespaces.values().any(|uri| uri == "http://ddex.net/xml/ern/43"));
        
        // Should have custom namespace if used
        assert!(final_namespaces.values().any(|uri| uri == "http://example.com/custom"));
        
        // Should have appropriate prefixes
        assert!(final_namespaces.contains_key("ern"));
        
        println!("Round-trip test completed successfully!");
        println!("Original declarations: {:?}", detection_result.declarations);
        println!("Final declarations: {:?}", final_namespaces);
    }
}

/// Performance and stress tests for namespace management
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_namespace_document() {
        // Create XML with many namespace declarations
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" 
                       xmlns:avs="http://ddex.net/xml/avs""#);

        // Add many custom namespaces
        for i in 0..100 {
            xml.push_str(&format!(r#" xmlns:ns{}="http://example.com/ns{}""#, i, i));
        }
        xml.push_str(">\n");

        // Add elements using various namespaces
        for i in 0..50 {
            xml.push_str(&format!(r#"    <ns{}:Element{}>Content</ns{}:Element{}>"#, i, i, i, i));
            xml.push('\n');
        }

        xml.push_str("</ern:NewReleaseMessage>");

        // Test detection performance
        let start = std::time::Instant::now();
        let mut detector = NamespaceDetector::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = detector.detect_from_xml(cursor).unwrap();
        let detection_time = start.elapsed();

        assert!(detection_time.as_millis() < 100); // Should be fast
        assert!(result.declarations.len() > 100);
        
        // Test minimization performance
        let start = std::time::Instant::now();
        let minimizer = NamespaceMinimizer::new(ERNVersion::V4_3);
        
        let mut test_ast = AST {
            root: Element::new("NewReleaseMessage")
                .with_namespace("http://ddex.net/xml/ern/43"),
            namespaces: result.declarations.clone(),
            schema_location: None,
        };
        
        let minimization_result = minimizer.minimize(test_ast);
        let minimization_time = start.elapsed();
        
        assert!(minimization_result.is_ok());
        assert!(minimization_time.as_millis() < 50); // Should be very fast
        
        println!("Performance test completed:");
        println!("  Detection time: {:?}", detection_time);
        println!("  Minimization time: {:?}", minimization_time);
        println!("  Namespace count: {}", result.declarations.len());
    }

    #[test]
    fn test_deep_namespace_nesting() {
        // Test deeply nested elements with different namespace scopes
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">"#);

        // Create nested elements with new namespace declarations at each level
        for depth in 0..20 {
            xml.push_str(&format!(
                r#"<level{}:Element xmlns:level{}="http://example.com/level{}">"#,
                depth, depth, depth
            ));
        }

        // Close all elements
        for depth in (0..20).rev() {
            xml.push_str(&format!("</level{}:Element>", depth));
        }
        xml.push_str("</ern:NewReleaseMessage>");

        let mut detector = NamespaceDetector::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = detector.detect_from_xml(cursor).unwrap();

        // Should detect all namespace levels
        assert!(result.declarations.len() >= 20);
        
        // Should handle the nesting correctly without stack overflow
        assert!(result.version == Some(ERNVersion::V4_3));
        
        println!("Deep nesting test completed with {} namespaces", result.declarations.len());
    }
}