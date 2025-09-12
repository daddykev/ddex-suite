//! # DB-C14N/1.0 - DDEX Builder Canonicalization
//! 
//! This module implements the DB-C14N/1.0 (DDEX Builder Canonical XML 1.0)
//! specification for deterministic XML canonicalization. This ensures that
//! identical logical XML documents always produce byte-identical output.
//! 
//! ## Why Canonicalization?
//! 
//! DDEX Builder guarantees deterministic output - the same input always
//! produces identical XML bytes. This is critical for:
//! 
//! - **Supply chain integrity**: Partners can verify XML hasn't changed
//! - **Reproducible builds**: CI/CD systems produce identical artifacts
//! - **Digital signatures**: Cryptographic signatures remain valid
//! - **Caching and deduplication**: Identical content can be detected
//! 
//! ## DB-C14N/1.0 Specification
//! 
//! ```text
//! Canonicalization Process
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │   Input XML     │───▶│   Parse & Sort   │───▶│  Canonical XML  │
//! │ (any format)    │    │                  │    │ (deterministic) │
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//!                               │
//!                               ▼
//!                        ┌──────────────────┐
//!                        │ Apply Rules:     │
//!                        │ • Namespace lock │
//!                        │ • Element order  │
//!                        │ • Attribute sort │
//!                        │ • Whitespace fix │
//!                        └──────────────────┘
//! ```
//! 
//! ## Key Features
//! 
//! - **Namespace Prefix Locking**: Fixed prefixes for DDEX namespaces
//! - **Deterministic Element Ordering**: Stable child element sequences
//! - **Attribute Canonicalization**: Alphabetical attribute ordering
//! - **Whitespace Normalization**: Consistent formatting and indentation
//! - **Comment Preservation**: Optional comment handling
//! 
//! ## Usage Example
//! 
//! ```rust
//! use ddex_builder::canonical::DB_C14N;
//! use ddex_builder::determinism::DeterminismConfig;
//! 
//! let config = DeterminismConfig::default();
//! let canonicalizer = DB_C14N::new(config);
//! 
//! let input_xml = r#"<Release xmlns:ern="http://ddex.net/xml/ern/43">
//!     <ReleaseId><GRid>A12345</GRid></ReleaseId>
//! </Release>"#;
//! 
//! let canonical = canonicalizer.canonicalize(input_xml)?;
//! let hash = canonicalizer.canonical_hash(&canonical)?;
//! 
//! // Same input always produces same output
//! assert_eq!(hash, canonicalizer.canonical_hash(&canonical)?);
//! ```
//! 
//! ## Specification Rules
//! 
//! The canonicalization follows these rules in order:
//! 
//! 1. **XML Declaration**: Always `<?xml version="1.0" encoding="UTF-8"?>`
//! 2. **Namespace Prefixes**: Use locked prefix table for DDEX namespaces
//! 3. **Element Order**: Apply schema-defined canonical element ordering
//! 4. **Attribute Order**: Sort attributes alphabetically by qualified name
//! 5. **Text Normalization**: Trim whitespace, normalize line endings
//! 6. **Indentation**: Use 2-space indentation with no trailing whitespace

use indexmap::IndexMap;
use sha2::{Sha256, Digest};
use quick_xml::{Reader, events::Event};
use std::collections::BTreeMap;

pub mod rules;

/// DB-C14N/1.0 canonicalizer
#[allow(non_camel_case_types)]  // Allow non-standard naming for DB-C14N
pub struct DB_C14N {
    config: super::determinism::DeterminismConfig,
    version: String,
}

impl DB_C14N {
    /// Create a new canonicalizer
    pub fn new(config: super::determinism::DeterminismConfig) -> Self {
        Self { 
            config,
            version: "4.3".to_string(), // Default to latest
        }
    }
    
    /// Create a new canonicalizer with specific ERN version
    pub fn with_version(config: super::determinism::DeterminismConfig, version: String) -> Self {
        Self { config, version }
    }
    
    /// Detect ERN version from XML content
    fn detect_version(&self, xml: &str) -> String {
        if xml.contains("http://ddex.net/xml/ern/382") {
            "3.8.2".to_string()
        } else if xml.contains("http://ddex.net/xml/ern/42") {
            "4.2".to_string()
        } else if xml.contains("http://ddex.net/xml/ern/43") {
            "4.3".to_string()
        } else {
            self.version.clone() // Use configured version as fallback
        }
    }
    
    /// Canonicalize XML according to DB-C14N/1.0 spec
    pub fn canonicalize(&self, xml: &str) -> Result<String, super::error::BuildError> {
        // Detect ERN version from content
        let detected_version = self.detect_version(xml);
        
        // Full DB-C14N/1.0 implementation
        let doc = self.parse_xml(xml)?;
        let canonical_doc = self.canonicalize_document(doc, &detected_version)?;
        self.serialize_canonical(canonical_doc)
    }
    
    /// Calculate canonical hash
    pub fn canonical_hash(&self, xml: &str) -> Result<String, super::error::BuildError> {
        let mut hasher = Sha256::new();
        hasher.update(xml.as_bytes());
        let result = hasher.finalize();
        
        Ok(format!("{:x}", result))
    }
    
    fn parse_xml(&self, xml: &str) -> Result<XmlDocument, super::error::BuildError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        
        let mut buf = Vec::new();
        let mut element_stack: Vec<XmlElement> = Vec::new();
        let mut text_content = String::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    // Save any accumulated text
                    if !text_content.trim().is_empty() {
                        if let Some(parent) = element_stack.last_mut() {
                            parent.children.push(XmlNode::Text(text_content.trim().to_string()));
                        }
                        text_content.clear();
                    }
                    
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut attributes = IndexMap::new();
                    
                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| super::error::BuildError::XmlGeneration(format!("Attribute error: {}", e)))?;
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        attributes.insert(key, value);
                    }
                    
                    let element = XmlElement {
                        name,
                        attributes,
                        children: Vec::new(),
                    };
                    
                    // Opening element, push to stack
                    element_stack.push(element);
                },
                Ok(Event::Empty(ref e)) => {
                    // Save any accumulated text
                    if !text_content.trim().is_empty() {
                        if let Some(parent) = element_stack.last_mut() {
                            parent.children.push(XmlNode::Text(text_content.trim().to_string()));
                        }
                        text_content.clear();
                    }
                    
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let mut attributes = IndexMap::new();
                    
                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| super::error::BuildError::XmlGeneration(format!("Attribute error: {}", e)))?;
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        attributes.insert(key, value);
                    }
                    
                    let element = XmlElement {
                        name,
                        attributes,
                        children: Vec::new(),
                    };
                    
                    // Self-closing element
                    if let Some(parent) = element_stack.last_mut() {
                        parent.children.push(XmlNode::Element(element));
                    } else {
                        // Root element
                        return Ok(XmlDocument { root: element });
                    }
                },
                Ok(Event::End(_)) => {
                    // Save any accumulated text
                    if !text_content.trim().is_empty() {
                        if let Some(parent) = element_stack.last_mut() {
                            parent.children.push(XmlNode::Text(text_content.trim().to_string()));
                        }
                        text_content.clear();
                    }
                    
                    // Pop the completed element
                    if let Some(completed_element) = element_stack.pop() {
                        if let Some(parent) = element_stack.last_mut() {
                            parent.children.push(XmlNode::Element(completed_element));
                        } else {
                            // This was the root element
                            return Ok(XmlDocument { root: completed_element });
                        }
                    }
                },
                Ok(Event::Text(e)) => {
                    text_content.push_str(&e.unescape().map_err(|e| {
                        super::error::BuildError::XmlGeneration(format!("Text unescape error: {}", e))
                    })?);
                },
                Ok(Event::Comment(e)) => {
                    let comment = String::from_utf8_lossy(&e).to_string();
                    if let Some(parent) = element_stack.last_mut() {
                        parent.children.push(XmlNode::Comment(comment));
                    }
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(super::error::BuildError::XmlGeneration(format!("XML parse error: {}", e))),
                _ => {} // Ignore other events
            }
            buf.clear();
        }
        
        Err(super::error::BuildError::XmlGeneration("No root element found".to_string()))
    }
    
    fn canonicalize_document(&self, mut doc: XmlDocument, version: &str) -> Result<XmlDocument, super::error::BuildError> {
        // Apply canonicalization rules to the entire document
        self.canonicalize_element(&mut doc.root, version)?;
        Ok(doc)
    }
    
    fn canonicalize_element(&self, element: &mut XmlElement, version: &str) -> Result<(), super::error::BuildError> {
        // 1. Sort attributes alphabetically by qualified name
        let sorted_attributes: BTreeMap<String, String> = element.attributes.clone().into_iter().collect();
        element.attributes = sorted_attributes.into_iter().collect();
        
        // 2. Apply namespace prefix locking
        self.apply_namespace_prefix_locking(&mut element.attributes, version)?;
        
        // 3. Sort child elements according to schema-defined order
        self.sort_child_elements(&mut element.children, &element.name, version)?;
        
        // 4. Recursively canonicalize child elements
        for child in &mut element.children {
            match child {
                XmlNode::Element(ref mut child_element) => {
                    self.canonicalize_element(child_element, version)?;
                },
                XmlNode::Text(ref mut text) => {
                    // Normalize whitespace in text content
                    *text = self.normalize_whitespace(text);
                },
                XmlNode::Comment(_) => {
                    // Comments are preserved as-is
                }
            }
        }
        
        Ok(())
    }
    
    fn apply_namespace_prefix_locking(&self, attributes: &mut IndexMap<String, String>, version: &str) -> Result<(), super::error::BuildError> {
        // Use the new comprehensive namespace manager
        let manager = rules::CanonicalNamespaceManager::new();
        
        // Extract namespace declarations from attributes
        let mut namespace_declarations = IndexMap::new();
        let mut other_attributes = IndexMap::new();
        
        for (key, value) in attributes.iter() {
            if key.starts_with("xmlns:") {
                let prefix = key.strip_prefix("xmlns:").unwrap_or("");
                namespace_declarations.insert(prefix.to_string(), value.clone());
            } else if key == "xmlns" {
                namespace_declarations.insert("".to_string(), value.clone()); // Default namespace
            } else {
                other_attributes.insert(key.clone(), value.clone());
            }
        }
        
        // Apply canonical namespace transformation
        let canonical_declarations = manager.canonicalize_namespaces(&namespace_declarations, version);
        
        // Rebuild attributes with canonical namespaces
        let mut updated_attrs = IndexMap::new();
        
        // Add canonical namespace declarations
        for (prefix, uri) in canonical_declarations {
            let key = if prefix.is_empty() {
                "xmlns".to_string()
            } else {
                format!("xmlns:{}", prefix)
            };
            updated_attrs.insert(key, uri);
        }
        
        // Add other attributes
        for (key, value) in other_attributes {
            updated_attrs.insert(key, value);
        }
        
        *attributes = updated_attrs;
        Ok(())
    }
    
    fn sort_child_elements(&self, children: &mut Vec<XmlNode>, parent_name: &str, version: &str) -> Result<(), super::error::BuildError> {
        let manager = rules::CanonicalNamespaceManager::new();
        
        if let Some(order) = manager.get_canonical_element_order(parent_name, version) {
            // Create a map for quick lookup of element order
            let order_map: IndexMap<String, usize> = order.iter().enumerate()
                .map(|(i, name)| (name.clone(), i))
                .collect();
                
            children.sort_by(|a, b| {
                match (a, b) {
                    (XmlNode::Element(elem_a), XmlNode::Element(elem_b)) => {
                        let order_a = order_map.get(&elem_a.name).unwrap_or(&usize::MAX);
                        let order_b = order_map.get(&elem_b.name).unwrap_or(&usize::MAX);
                        order_a.cmp(order_b).then_with(|| elem_a.name.cmp(&elem_b.name))
                    },
                    (XmlNode::Element(_), _) => std::cmp::Ordering::Less,
                    (_, XmlNode::Element(_)) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                }
            });
        }
        
        Ok(())
    }
    
    fn normalize_whitespace(&self, text: &str) -> String {
        // Normalize line endings to LF and trim whitespace
        text.replace("\r\n", "\n")
            .replace("\r", "\n")
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    fn serialize_canonical(&self, doc: XmlDocument) -> Result<String, super::error::BuildError> {
        let mut output = Vec::new();
        // XML writer no longer used since we build the output manually
        
        // Write XML declaration
        output.clear();
        output.extend_from_slice(rules::XML_DECLARATION.as_bytes());
        output.push(b'\n');
        
        // Serialize the root element with 2-space indentation
        self.serialize_element(&doc.root, &mut output, 0)?;
        
        // Ensure no trailing whitespace and final newline
        let result = String::from_utf8(output)
            .map_err(|e| super::error::BuildError::XmlGeneration(format!("UTF-8 conversion error: {}", e)))?;
            
        let canonical = result.lines()
            .map(|line| line.trim_end()) // Remove trailing whitespace
            .collect::<Vec<_>>()
            .join("\n");
            
        Ok(format!("{}\n", canonical))
    }
    
    fn serialize_element(&self, element: &XmlElement, output: &mut Vec<u8>, indent_level: usize) -> Result<(), super::error::BuildError> {
        let indent = "  ".repeat(indent_level);
        
        // Start tag
        output.extend_from_slice(indent.as_bytes());
        output.push(b'<');
        output.extend_from_slice(element.name.as_bytes());
        
        // Attributes (already sorted)
        for (key, value) in &element.attributes {
            output.push(b' ');
            output.extend_from_slice(key.as_bytes());
            output.extend_from_slice(b"=\"");
            output.extend_from_slice(html_escape::encode_double_quoted_attribute(&value).as_bytes());
            output.push(b'"');
        }
        
        if element.children.is_empty() {
            // Self-closing tag
            output.extend_from_slice(b"/>");
            output.push(b'\n');
        } else {
            output.push(b'>');
            
            // Check if we have only text content (no child elements)
            let has_only_text = element.children.iter().all(|child| matches!(child, XmlNode::Text(_)));
            
            if has_only_text {
                // Inline text content
                for child in &element.children {
                    if let XmlNode::Text(text) = child {
                        output.extend_from_slice(html_escape::encode_text(text).as_bytes());
                    }
                }
            } else {
                output.push(b'\n');
                
                // Child elements with proper indentation
                for child in &element.children {
                    match child {
                        XmlNode::Element(child_element) => {
                            self.serialize_element(child_element, output, indent_level + 1)?;
                        },
                        XmlNode::Text(text) => {
                            if !text.trim().is_empty() {
                                let child_indent = "  ".repeat(indent_level + 1);
                                output.extend_from_slice(child_indent.as_bytes());
                                output.extend_from_slice(html_escape::encode_text(text.trim()).as_bytes());
                                output.push(b'\n');
                            }
                        },
                        XmlNode::Comment(comment) => {
                            let child_indent = "  ".repeat(indent_level + 1);
                            output.extend_from_slice(child_indent.as_bytes());
                            output.extend_from_slice(b"<!--");
                            output.extend_from_slice(comment.as_bytes());
                            output.extend_from_slice(b"-->");
                            output.push(b'\n');
                        }
                    }
                }
                
                output.extend_from_slice(indent.as_bytes());
            }
            
            // End tag
            output.extend_from_slice(b"</");
            output.extend_from_slice(element.name.as_bytes());
            output.push(b'>');
            output.push(b'\n');
        }
        
        Ok(())
    }
}

/// Internal XML document representation
struct XmlDocument {
    root: XmlElement,
}

/// Internal XML element representation  
struct XmlElement {
    name: String,
    attributes: IndexMap<String, String>,  // Deterministic ordering
    children: Vec<XmlNode>,
}

/// XML node types
enum XmlNode {
    Element(XmlElement),
    Text(String),
    Comment(String),
}


#[cfg(test)]
mod tests;

/// Test helper functions
#[cfg(test)]
pub fn create_test_canonicalizer() -> DB_C14N {
    let config = super::determinism::DeterminismConfig::default();
    DB_C14N::new(config)
}

#[cfg(test)]
pub fn create_test_canonicalizer_with_version(version: &str) -> DB_C14N {
    let config = super::determinism::DeterminismConfig::default();
    DB_C14N::with_version(config, version.to_string())
}