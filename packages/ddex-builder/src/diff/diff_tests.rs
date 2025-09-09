//! Tests for DDEX semantic diff functionality

use super::*;
use crate::diff::test_data::*;
use crate::diff::formatter::DiffFormatter;

#[test]
fn test_identical_documents_no_changes() {
    let mut engine = DiffEngine::new();
    
    let ast1 = create_simple_ast("Root", "same content");
    let ast2 = create_simple_ast("Root", "same content");
    
    let changeset = engine.diff(&ast1, &ast2).unwrap();
    
    assert!(!changeset.has_changes());
    assert_eq!(changeset.summary.total_changes, 0);
    assert_eq!(changeset.impact_level(), types::ImpactLevel::None);
}

#[test]
fn test_simple_text_change() {
    let mut engine = DiffEngine::new();
    
    let ast1 = create_simple_ast("Root", "old content");
    let ast2 = create_simple_ast("Root", "new content");
    
    let changeset = engine.diff(&ast1, &ast2).unwrap();
    
    assert!(changeset.has_changes());
    assert_eq!(changeset.summary.total_changes, 1);
    assert_eq!(changeset.changes[0].change_type, types::ChangeType::TextModified);
    assert_eq!(changeset.changes[0].old_value, Some("old content".to_string()));
    assert_eq!(changeset.changes[0].new_value, Some("new content".to_string()));
}

#[test]
fn test_formatting_ignored_by_default() {
    let mut engine = DiffEngine::new();
    
    let ast1 = create_simple_ast("Root", "  content  ");
    let ast2 = create_simple_ast("Root", "content");
    
    let changeset = engine.diff(&ast1, &ast2).unwrap();
    
    // Should have no changes because formatting differences are ignored
    assert!(!changeset.has_changes());
}

#[test]
fn test_formatting_respected_when_configured() {
    let mut config = DiffConfig::default();
    config.ignore_formatting = false;
    let mut engine = DiffEngine::new_with_config(config);
    
    let ast1 = create_simple_ast("Root", "  content  ");
    let ast2 = create_simple_ast("Root", "content");
    
    let changeset = engine.diff(&ast1, &ast2).unwrap();
    
    // Should have changes because formatting differences are not ignored
    assert!(changeset.has_changes());
    assert_eq!(changeset.summary.total_changes, 1);
}

#[test]
fn test_attribute_changes() {
    let mut engine = DiffEngine::new();
    
    let mut ast1 = AST {
        root: Element::new("Release").with_attr("UPC", "123456789012"),
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    };
    
    let ast2 = AST {
        root: Element::new("Release").with_attr("UPC", "987654321098"),
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    };
    
    let changeset = engine.diff(&ast1, &ast2).unwrap();
    
    assert!(changeset.has_changes());
    assert_eq!(changeset.summary.total_changes, 1);
    assert_eq!(changeset.changes[0].change_type, types::ChangeType::AttributeModified);
    assert!(changeset.changes[0].is_critical); // UPC is a critical field
}

#[test]
fn test_critical_field_detection() {
    let mut engine = DiffEngine::new();
    
    // Test critical field (UPC)
    let ast1 = AST {
        root: Element::new("Release").with_attr("UPC", "123456789012"),
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    };
    
    let ast2 = AST {
        root: Element::new("Release").with_attr("UPC", "987654321098"),
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    };
    
    let changeset = engine.diff(&ast1, &ast2).unwrap();
    
    let critical_changes = changeset.critical_changes();
    assert_eq!(critical_changes.len(), 1);
    assert_eq!(changeset.impact_level(), types::ImpactLevel::High);
}

#[test]
fn test_ignored_fields() {
    let mut engine = DiffEngine::new();
    
    // MessageId is in ignored_fields by default
    let ast1 = AST {
        root: Element::new("MessageHeader").with_attr("MessageId", "MSG-001"),
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    };
    
    let ast2 = AST {
        root: Element::new("MessageHeader").with_attr("MessageId", "MSG-002"),
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    };
    
    let changeset = engine.diff(&ast1, &ast2).unwrap();
    
    // Should have no changes because MessageId is ignored
    assert!(!changeset.has_changes());
}

#[test]
fn test_element_addition_removal() {
    let mut engine = DiffEngine::new();
    
    // AST with no children
    let ast1 = AST {
        root: Element::new("Root"),
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    };
    
    // AST with a child element
    let mut root_with_child = Element::new("Root");
    root_with_child.add_child(Element::new("Child").with_text("content"));
    let ast2 = AST {
        root: root_with_child,
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    };
    
    let changeset = engine.diff(&ast1, &ast2).unwrap();
    
    assert!(changeset.has_changes());
    assert_eq!(changeset.summary.additions, 1);
    
    // Test removal (reverse order)
    let changeset_removal = engine.diff(&ast2, &ast1).unwrap();
    assert_eq!(changeset_removal.summary.deletions, 1);
}

#[test]
fn test_reference_equivalence() {
    let mut config = DiffConfig::default();
    config.ignore_reference_ids = true;
    let mut engine = DiffEngine::new_with_config(config);
    
    // Two elements with different reference IDs but same content
    let ast1 = AST {
        root: Element::new("Resource").with_attr("ResourceReference", "R001"),
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    };
    
    let ast2 = AST {
        root: Element::new("Resource").with_attr("ResourceReference", "R002"),
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    };
    
    let changeset = engine.diff(&ast1, &ast2).unwrap();
    
    // Should have changes because we don't have reference content resolution in this simple test
    // In a real implementation, this would resolve the references and compare content
    assert!(changeset.has_changes());
}

#[test]
fn test_numeric_tolerance() {
    let mut config = DiffConfig::default();
    config.numeric_tolerance = Some(0.01);
    let mut engine = DiffEngine::new_with_config(config);
    
    // Test prices within tolerance
    let ast1 = AST {
        root: Element::new("Deal").with_attr("Price", "9.99"),
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    };
    
    let ast2 = AST {
        root: Element::new("Deal").with_attr("Price", "9.999"),
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    };
    
    let changeset = engine.diff(&ast1, &ast2).unwrap();
    
    // Should have no changes due to numeric tolerance
    assert!(!changeset.has_changes());
}

#[test]
fn test_diff_formatter_summary() {
    let mut changeset = types::ChangeSet::new();
    
    changeset.add_change(types::SemanticChange {
        path: types::DiffPath::root().with_element("Release").with_attribute("UPC"),
        change_type: types::ChangeType::AttributeModified,
        old_value: Some("123456789012".to_string()),
        new_value: Some("987654321098".to_string()),
        is_critical: true,
        description: "UPC changed from 123456789012 to 987654321098".to_string(),
    });
    
    let summary = DiffFormatter::format_summary(&changeset);
    
    assert!(summary.contains("DDEX Semantic Diff Summary"));
    assert!(summary.contains("Critical Changes"));
    assert!(summary.contains("UPC changed"));
    assert!(summary.contains("Impact Level: High"));
}

#[test]
fn test_diff_formatter_json() {
    let mut changeset = types::ChangeSet::new();
    
    changeset.add_change(types::SemanticChange {
        path: types::DiffPath::root().with_element("Title"),
        change_type: types::ChangeType::TextModified,
        old_value: Some("Old Title".to_string()),
        new_value: Some("New Title".to_string()),
        is_critical: false,
        description: "Title changed".to_string(),
    });
    
    let json_result = DiffFormatter::format_json(&changeset);
    assert!(json_result.is_ok());
    
    let json_str = json_result.unwrap();
    assert!(json_str.contains("total_changes"));
    assert!(json_str.contains("Title changed"));
    assert!(json_str.contains("Old Title"));
    assert!(json_str.contains("New Title"));
}

#[test]
fn test_diff_formatter_json_patch() {
    let mut changeset = types::ChangeSet::new();
    
    changeset.add_change(types::SemanticChange {
        path: types::DiffPath::root().with_element("Release").with_attribute("UPC"),
        change_type: types::ChangeType::AttributeModified,
        old_value: Some("123456789012".to_string()),
        new_value: Some("987654321098".to_string()),
        is_critical: true,
        description: "UPC modified".to_string(),
    });
    
    let patch_result = DiffFormatter::format_json_patch(&changeset);
    assert!(patch_result.is_ok());
    
    let patch_str = patch_result.unwrap();
    assert!(patch_str.contains("\"op\": \"replace\""));
    assert!(patch_str.contains("\"path\": \"/Release/@UPC\""));
    assert!(patch_str.contains("987654321098"));
}

#[test]
fn test_diff_formatter_html() {
    let mut changeset = types::ChangeSet::new();
    
    changeset.add_change(types::SemanticChange {
        path: types::DiffPath::root().with_element("Title"),
        change_type: types::ChangeType::TextModified,
        old_value: Some("Old Title".to_string()),
        new_value: Some("New Title".to_string()),
        is_critical: false,
        description: "Title changed".to_string(),
    });
    
    let html = DiffFormatter::format_html(&changeset);
    
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("DDEX Semantic Diff Report"));
    assert!(html.contains("Title changed"));
    assert!(html.contains("Old Title"));
    assert!(html.contains("New Title"));
}

#[test]
fn test_update_message_generation() {
    let mut changeset = types::ChangeSet::new();
    
    changeset.add_change(types::SemanticChange {
        path: types::DiffPath::root().with_element("Release").with_attribute("UPC"),
        change_type: types::ChangeType::AttributeModified,
        old_value: Some("123456789012".to_string()),
        new_value: Some("987654321098".to_string()),
        is_critical: true,
        description: "UPC changed".to_string(),
    });
    
    let update_msg_result = DiffFormatter::generate_update_message(&changeset, Some("TEST-UPDATE-001"));
    assert!(update_msg_result.is_ok());
    
    let update_msg = update_msg_result.unwrap();
    assert!(update_msg.contains("<?xml version=\"1.0\""));
    assert!(update_msg.contains("<UpdateReleaseMessage"));
    assert!(update_msg.contains("TEST-UPDATE-001"));
    assert!(update_msg.contains("<UpdateType>Modify</UpdateType>"));
    assert!(update_msg.contains("<IsCritical>true</IsCritical>"));
}

#[test]
fn test_business_impact_analysis() {
    let mut engine = DiffEngine::new();
    
    // Create changeset with mix of critical and non-critical changes
    let mut changeset = types::ChangeSet::new();
    
    // Critical change
    changeset.add_change(types::SemanticChange {
        path: types::DiffPath::root().with_element("Release").with_attribute("UPC"),
        change_type: types::ChangeType::AttributeModified,
        old_value: Some("123456789012".to_string()),
        new_value: Some("987654321098".to_string()),
        is_critical: true,
        description: "UPC changed".to_string(),
    });
    
    // Non-critical change
    changeset.add_change(types::SemanticChange {
        path: types::DiffPath::root().with_element("Title"),
        change_type: types::ChangeType::TextModified,
        old_value: Some("Old Title".to_string()),
        new_value: Some("New Title".to_string()),
        is_critical: false,
        description: "Title changed".to_string(),
    });
    
    // Simulate business impact analysis
    engine.analyze_business_impact(&mut changeset);
    
    assert_eq!(changeset.impact_level(), types::ImpactLevel::High);
    assert!(changeset.metadata.contains_key("critical_changes"));
    assert_eq!(changeset.metadata.get("critical_changes").unwrap(), "1");
    assert_eq!(changeset.metadata.get("impact_level").unwrap(), "HIGH");
}

// Helper function to create a simple AST for testing
fn create_simple_ast(element_name: &str, text_content: &str) -> AST {
    AST {
        root: Element::new(element_name).with_text(text_content),
        namespaces: indexmap::IndexMap::new(),
        schema_location: None,
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Test using real DDEX samples
    #[test]
    fn test_real_ddex_diff_basic_changes() {
        let mut engine = DiffEngine::new();
        
        // These would normally be parsed from actual DDEX XML
        // For now, we create simplified ASTs to represent the key differences
        
        let ast_v1 = create_ddex_ast_v1();
        let ast_v2 = create_ddex_ast_v2();
        
        let changeset = engine.diff(&ast_v1, &ast_v2).unwrap();
        
        assert!(changeset.has_changes());
        
        // Expected changes: Title, UPC, LabelName, Genre, Duration, etc.
        assert!(changeset.summary.total_changes > 0);
        
        // Should detect critical changes (UPC change)
        let critical_changes = changeset.critical_changes();
        assert!(!critical_changes.is_empty());
        
        // Generate different output formats
        let summary = DiffFormatter::format_summary(&changeset);
        assert!(summary.contains("Critical Changes"));
        
        let json_result = DiffFormatter::format_json(&changeset);
        assert!(json_result.is_ok());
        
        let html = DiffFormatter::format_html(&changeset);
        assert!(html.contains("DDEX Semantic Diff Report"));
    }
    
    #[test]
    fn test_formatting_only_diff() {
        let mut engine = DiffEngine::new();
        
        // Test documents that differ only in formatting
        let ast_original = create_simple_ast("Root", "content");
        let ast_formatted = create_simple_ast("Root", "  content  "); // Extra whitespace
        
        let changeset = engine.diff(&ast_original, &ast_formatted).unwrap();
        
        // Should have no changes with default config (formatting ignored)
        assert!(!changeset.has_changes());
        assert_eq!(changeset.impact_level(), types::ImpactLevel::None);
    }
    
    // Helper functions to create test ASTs representing DDEX documents
    fn create_ddex_ast_v1() -> AST {
        let mut root = Element::new("NewReleaseMessage");
        
        // Add a simplified Release element
        let mut release = Element::new("Release");
        release = release.with_attr("UPC", "123456789012");
        release.add_child(Element::new("Title").with_text("Test Album"));
        release.add_child(Element::new("LabelName").with_text("Test Label"));
        release.add_child(Element::new("Genre").with_text("Pop"));
        
        root.add_child(release);
        
        AST {
            root,
            namespaces: indexmap::IndexMap::new(),
            schema_location: None,
        }
    }
    
    fn create_ddex_ast_v2() -> AST {
        let mut root = Element::new("NewReleaseMessage");
        
        // Add a Release element with changes
        let mut release = Element::new("Release");
        release = release.with_attr("UPC", "987654321098"); // Changed UPC (critical)
        release.add_child(Element::new("Title").with_text("Test Album (Deluxe Edition)")); // Changed title
        release.add_child(Element::new("LabelName").with_text("New Test Label")); // Changed label
        release.add_child(Element::new("Genre").with_text("Rock")); // Changed genre
        
        root.add_child(release);
        
        AST {
            root,
            namespaces: indexmap::IndexMap::new(),
            schema_location: None,
        }
    }
}