//! Extension preservation system for perfect round-trip fidelity
//!
//! This module provides comprehensive XML fragment preservation to maintain
//! unknown elements, namespaces, and attributes that are not part of the
//! DDEX schema. This ensures that proprietary extensions from music companies
//! or custom implementations are preserved during parse → modify → build cycles.

use serde::{Deserialize, Serialize};
use indexmap::IndexMap;
use std::collections::HashMap;

/// Comprehensive XML fragment preservation for round-trip fidelity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XmlFragment {
    /// The raw XML content as a string
    pub raw_content: String,
    
    /// Element name (local name without namespace prefix)
    pub element_name: String,
    
    /// Namespace URI for this element
    pub namespace_uri: Option<String>,
    
    /// Namespace prefix used in the original XML
    pub namespace_prefix: Option<String>,
    
    /// All namespace declarations on this element
    pub namespace_declarations: IndexMap<String, String>, // prefix -> uri
    
    /// All attributes on this element (including namespaced ones)
    pub attributes: IndexMap<String, String>, // qualified name -> value
    
    /// Child XML fragments (for nested unknown elements)
    pub children: Vec<XmlFragment>,
    
    /// Text content (if this element contains only text)
    pub text_content: Option<String>,
    
    /// Processing instructions within this fragment
    pub processing_instructions: Vec<ProcessingInstruction>,
    
    /// Comments within this fragment
    pub comments: Vec<String>,
    
    /// Position hint for canonical ordering
    pub position_hint: Option<usize>,
    
    /// Whether this fragment should be preserved as-is (no canonicalization)
    pub preserve_formatting: bool,
}

/// XML Processing Instruction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessingInstruction {
    /// The target of the processing instruction
    pub target: String,
    
    /// The data content of the processing instruction
    pub data: Option<String>,
}

/// Extension container with location-aware storage
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Extensions {
    /// Extensions organized by their location in the DDEX structure
    pub fragments: IndexMap<String, XmlFragment>,
    
    /// Global namespace declarations that should be preserved at document level
    pub global_namespaces: IndexMap<String, String>, // prefix -> uri
    
    /// Document-level processing instructions
    pub document_processing_instructions: Vec<ProcessingInstruction>,
    
    /// Document-level comments
    pub document_comments: Vec<String>,
    
    /// Legacy simple extensions (for backward compatibility)
    #[serde(flatten)]
    pub legacy_data: HashMap<String, serde_json::Value>,
}

impl XmlFragment {
    /// Create a new XML fragment
    pub fn new(element_name: String, raw_content: String) -> Self {
        Self {
            raw_content,
            element_name,
            namespace_uri: None,
            namespace_prefix: None,
            namespace_declarations: IndexMap::new(),
            attributes: IndexMap::new(),
            children: Vec::new(),
            text_content: None,
            processing_instructions: Vec::new(),
            comments: Vec::new(),
            position_hint: None,
            preserve_formatting: false,
        }
    }
    
    /// Create a fragment with namespace information
    pub fn with_namespace(
        element_name: String,
        namespace_uri: Option<String>,
        namespace_prefix: Option<String>,
        raw_content: String,
    ) -> Self {
        Self {
            raw_content,
            element_name,
            namespace_uri,
            namespace_prefix,
            namespace_declarations: IndexMap::new(),
            attributes: IndexMap::new(),
            children: Vec::new(),
            text_content: None,
            processing_instructions: Vec::new(),
            comments: Vec::new(),
            position_hint: None,
            preserve_formatting: false,
        }
    }
    
    /// Get the qualified name for this element
    pub fn qualified_name(&self) -> String {
        if let Some(ref prefix) = self.namespace_prefix {
            format!("{}:{}", prefix, self.element_name)
        } else {
            self.element_name.clone()
        }
    }
    
    /// Check if this fragment is from a specific namespace
    pub fn is_from_namespace(&self, namespace_uri: &str) -> bool {
        self.namespace_uri.as_ref().map_or(false, |uri| uri == namespace_uri)
    }
    
    /// Add a child fragment
    pub fn add_child(&mut self, child: XmlFragment) {
        self.children.push(child);
    }
    
    /// Add an attribute
    pub fn add_attribute(&mut self, name: String, value: String) {
        self.attributes.insert(name, value);
    }
    
    /// Add a namespace declaration
    pub fn add_namespace_declaration(&mut self, prefix: String, uri: String) {
        self.namespace_declarations.insert(prefix, uri);
    }
    
    /// Set position hint for canonical ordering
    pub fn set_position_hint(&mut self, position: usize) {
        self.position_hint = Some(position);
    }
    
    /// Mark this fragment to preserve original formatting
    pub fn preserve_formatting(&mut self) {
        self.preserve_formatting = true;
    }
    
    /// Get the canonical XML representation with proper formatting
    pub fn to_canonical_xml(&self, indent_level: usize) -> String {
        if self.preserve_formatting {
            return self.raw_content.clone();
        }
        
        let indent = "  ".repeat(indent_level);
        let mut xml = String::new();
        
        // Opening tag
        xml.push_str(&format!("{}<{}", indent, self.qualified_name()));
        
        // Namespace declarations (sorted for determinism)
        let mut sorted_ns: Vec<_> = self.namespace_declarations.iter().collect();
        sorted_ns.sort_by_key(|(prefix, _)| prefix.as_str());
        
        for (prefix, uri) in sorted_ns {
            if prefix.is_empty() {
                xml.push_str(&format!(" xmlns=\"{}\"", uri));
            } else {
                xml.push_str(&format!(" xmlns:{}=\"{}\"", prefix, uri));
            }
        }
        
        // Attributes (sorted for determinism)
        let mut sorted_attrs: Vec<_> = self.attributes.iter().collect();
        sorted_attrs.sort_by_key(|(name, _)| name.as_str());
        
        for (name, value) in sorted_attrs {
            xml.push_str(&format!(" {}=\"{}\"", name, html_escape::encode_double_quoted_attribute(value)));
        }
        
        if let Some(ref text) = self.text_content {
            // Element with text content
            xml.push('>');
            xml.push_str(&html_escape::encode_text(text));
            xml.push_str(&format!("</{}>", self.qualified_name()));
        } else if self.children.is_empty() && self.processing_instructions.is_empty() && self.comments.is_empty() {
            // Self-closing element
            xml.push_str("/>");
        } else {
            // Element with children
            xml.push_str(">\n");
            
            // Processing instructions
            for pi in &self.processing_instructions {
                xml.push_str(&format!("{}  <?{}", indent, pi.target));
                if let Some(ref data) = pi.data {
                    xml.push(' ');
                    xml.push_str(data);
                }
                xml.push_str("?>\n");
            }
            
            // Comments
            for comment in &self.comments {
                xml.push_str(&format!("{}  <!--{}-->\n", indent, comment));
            }
            
            // Child elements
            for child in &self.children {
                xml.push_str(&child.to_canonical_xml(indent_level + 1));
                xml.push('\n');
            }
            
            xml.push_str(&format!("{}</{}>", indent, self.qualified_name()));
        }
        
        xml
    }
}

impl ProcessingInstruction {
    /// Create a new processing instruction
    pub fn new(target: String, data: Option<String>) -> Self {
        Self { target, data }
    }
}

impl Extensions {
    /// Create a new extensions container
    pub fn new() -> Self {
        Self {
            fragments: IndexMap::new(),
            global_namespaces: IndexMap::new(),
            document_processing_instructions: Vec::new(),
            document_comments: Vec::new(),
            legacy_data: HashMap::new(),
        }
    }
    
    /// Add an XML fragment at a specific location
    pub fn add_fragment(&mut self, location: String, fragment: XmlFragment) {
        self.fragments.insert(location, fragment);
    }
    
    /// Get a fragment by location
    pub fn get_fragment(&self, location: &str) -> Option<&XmlFragment> {
        self.fragments.get(location)
    }
    
    /// Get all fragments for a location pattern
    pub fn get_fragments_matching(&self, pattern: &str) -> Vec<(&String, &XmlFragment)> {
        self.fragments
            .iter()
            .filter(|(location, _)| location.starts_with(pattern))
            .collect()
    }
    
    /// Add a global namespace declaration
    pub fn add_global_namespace(&mut self, prefix: String, uri: String) {
        self.global_namespaces.insert(prefix, uri);
    }
    
    /// Add a document-level processing instruction
    pub fn add_document_processing_instruction(&mut self, pi: ProcessingInstruction) {
        self.document_processing_instructions.push(pi);
    }
    
    /// Add a document-level comment
    pub fn add_document_comment(&mut self, comment: String) {
        self.document_comments.push(comment);
    }
    
    /// Legacy method for backward compatibility
    pub fn insert(&mut self, key: String, value: serde_json::Value) {
        self.legacy_data.insert(key, value);
    }
    
    /// Legacy method for backward compatibility
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.legacy_data.get(key)
    }
    
    /// Check if there are any extensions
    pub fn is_empty(&self) -> bool {
        self.fragments.is_empty() 
            && self.global_namespaces.is_empty()
            && self.document_processing_instructions.is_empty()
            && self.document_comments.is_empty()
            && self.legacy_data.is_empty()
    }
    
    /// Get the total number of preserved extensions
    pub fn count(&self) -> usize {
        self.fragments.len() 
            + self.global_namespaces.len()
            + self.document_processing_instructions.len()
            + self.document_comments.len()
            + self.legacy_data.len()
    }
    
    /// Merge another Extensions instance into this one
    pub fn merge(&mut self, other: Extensions) {
        for (location, fragment) in other.fragments {
            self.fragments.insert(location, fragment);
        }
        
        for (prefix, uri) in other.global_namespaces {
            self.global_namespaces.insert(prefix, uri);
        }
        
        self.document_processing_instructions.extend(other.document_processing_instructions);
        self.document_comments.extend(other.document_comments);
        
        for (key, value) in other.legacy_data {
            self.legacy_data.insert(key, value);
        }
    }
    
    /// Clear all extensions
    pub fn clear(&mut self) {
        self.fragments.clear();
        self.global_namespaces.clear();
        self.document_processing_instructions.clear();
        self.document_comments.clear();
        self.legacy_data.clear();
    }
}

/// Helper functions for extension management
pub mod utils {
    use super::*;
    
    /// Generate a location key for an extension
    /// Format: "element_path/namespace_uri/element_name"
    pub fn generate_location_key(
        element_path: &[&str],
        namespace_uri: Option<&str>,
        element_name: &str,
    ) -> String {
        let path = element_path.join("/");
        match namespace_uri {
            Some(ns) => format!("{}/{}/{}", path, ns, element_name),
            None => format!("{}/{}", path, element_name),
        }
    }
    
    /// Check if a namespace URI is a known DDEX namespace
    pub fn is_ddex_namespace(namespace_uri: &str) -> bool {
        const DDEX_NAMESPACES: &[&str] = &[
            "http://ddex.net/xml/ern/382",
            "http://ddex.net/xml/ern/42",
            "http://ddex.net/xml/ern/43",
            "http://ddex.net/xml/avs",
            "http://www.w3.org/2001/XMLSchema-instance",
        ];
        
        DDEX_NAMESPACES.contains(&namespace_uri)
    }
    
    /// Extract namespace prefix from a qualified name
    pub fn extract_namespace_prefix(qualified_name: &str) -> Option<&str> {
        qualified_name.split(':').next().filter(|prefix| !prefix.is_empty())
    }
    
    /// Extract local name from a qualified name
    pub fn extract_local_name(qualified_name: &str) -> &str {
        qualified_name.split(':').last().unwrap_or(qualified_name)
    }
    
    /// Validate XML fragment content
    pub fn validate_xml_fragment(fragment: &XmlFragment) -> Result<(), String> {
        if fragment.element_name.is_empty() {
            return Err("Element name cannot be empty".to_string());
        }
        
        if fragment.raw_content.is_empty() {
            return Err("Raw content cannot be empty".to_string());
        }
        
        // Additional validation can be added here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_xml_fragment_creation() {
        let fragment = XmlFragment::new(
            "customElement".to_string(),
            "<customElement>content</customElement>".to_string(),
        );
        
        assert_eq!(fragment.element_name, "customElement");
        assert_eq!(fragment.raw_content, "<customElement>content</customElement>");
        assert_eq!(fragment.qualified_name(), "customElement");
    }
    
    #[test]
    fn test_xml_fragment_with_namespace() {
        let fragment = XmlFragment::with_namespace(
            "customElement".to_string(),
            Some("http://example.com/custom".to_string()),
            Some("custom".to_string()),
            "<custom:customElement>content</custom:customElement>".to_string(),
        );
        
        assert_eq!(fragment.qualified_name(), "custom:customElement");
        assert!(fragment.is_from_namespace("http://example.com/custom"));
    }
    
    #[test]
    fn test_extensions_container() {
        let mut extensions = Extensions::new();
        assert!(extensions.is_empty());
        
        let fragment = XmlFragment::new(
            "test".to_string(),
            "<test/>".to_string(),
        );
        
        extensions.add_fragment("message/test".to_string(), fragment);
        assert!(!extensions.is_empty());
        assert_eq!(extensions.count(), 1);
    }
    
    #[test]
    fn test_canonical_xml_generation() {
        let mut fragment = XmlFragment::new(
            "customElement".to_string(),
            "<customElement attr=\"value\">text</customElement>".to_string(),
        );
        
        fragment.add_attribute("attr".to_string(), "value".to_string());
        fragment.text_content = Some("text".to_string());
        
        let xml = fragment.to_canonical_xml(0);
        assert!(xml.contains("<customElement attr=\"value\">text</customElement>"));
    }
    
    #[test]
    fn test_location_key_generation() {
        let key = utils::generate_location_key(
            &["message", "header"],
            Some("http://example.com/ns"),
            "customElement",
        );
        
        assert_eq!(key, "message/header/http://example.com/ns/customElement");
    }
    
    #[test]
    fn test_ddex_namespace_detection() {
        assert!(utils::is_ddex_namespace("http://ddex.net/xml/ern/43"));
        assert!(utils::is_ddex_namespace("http://ddex.net/xml/avs"));
        assert!(!utils::is_ddex_namespace("http://example.com/custom"));
    }
}

#[cfg(test)]
pub mod test_data;

#[cfg(test)]
mod comprehensive_tests;