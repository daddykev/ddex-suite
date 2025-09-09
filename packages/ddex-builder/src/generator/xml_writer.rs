//! XML serialization from AST

use crate::ast::{AST, Element, Node};
use crate::determinism::DeterminismConfig;
use crate::error::BuildError;
use indexmap::IndexMap;
use std::io::Write;

pub struct XmlWriter {
    config: DeterminismConfig,
}

impl XmlWriter {
    pub fn new(config: DeterminismConfig) -> Self {
        Self { config }
    }
    
    pub fn write(&self, ast: &AST) -> Result<String, BuildError> {
        let mut buffer = Vec::new();
        
        // Write XML declaration
        writeln!(&mut buffer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
        
        // Write root element with namespaces
        self.write_element(&mut buffer, &ast.root, &ast.namespaces, ast.schema_location.as_deref(), 0)?;
        
        Ok(String::from_utf8(buffer).map_err(|e| BuildError::Serialization(e.to_string()))?)
    }
    
    fn write_element(
        &self,
        writer: &mut impl Write,
        element: &Element,
        namespaces: &IndexMap<String, String>,
        schema_location: Option<&str>,
        depth: usize,
    ) -> Result<(), BuildError> {
        let indent = self.get_indent(depth);
        
        // Start tag
        write!(writer, "{}<", indent)?;
        
        // Add namespace prefix if needed
        if let Some(ns) = &element.namespace {
            if let Some(prefix) = namespaces.iter().find(|(_, v)| *v == ns).map(|(k, _)| k) {
                write!(writer, "{}:", prefix)?;
            }
        } else if depth == 0 && !namespaces.is_empty() {
            // Root element gets default namespace prefix
            if let Some((prefix, _)) = namespaces.first() {
                write!(writer, "{}:", prefix)?;
            }
        }
        
        write!(writer, "{}", element.name)?;
        
        // Add namespace declarations on root element
        if depth == 0 {
            for (prefix, uri) in namespaces {
                write!(writer, " xmlns:{}=\"{}\"", prefix, uri)?;
            }
            
            if let Some(location) = schema_location {
                write!(writer, " xsi:schemaLocation=\"{}\"", location)?;
            }
        }
        
        // Add attributes (in deterministic order)
        for (key, value) in &element.attributes {
            write!(writer, " {}=\"{}\"", key, self.escape_attribute(value))?;
        }
        
        // Check if we have children
        if element.children.is_empty() {
            writeln!(writer, "/>")?;
        } else {
            // Check if we only have text content
            let only_text = element.children.len() == 1 && 
                matches!(&element.children[0], Node::Text(_));
            
            if only_text {
                // Inline text content
                write!(writer, ">")?;
                if let Node::Text(text) = &element.children[0] {
                    write!(writer, "{}", self.escape_text(text))?;
                }
                writeln!(writer, "</{}>", self.get_element_name(element, namespaces))?;
            } else {
                // Multi-line with children
                writeln!(writer, ">")?;
                
                for child in &element.children {
                    match child {
                        Node::Element(child_elem) => {
                            self.write_element(writer, child_elem, namespaces, None, depth + 1)?;
                        }
                        Node::Text(text) => {
                            writeln!(writer, "{}{}", self.get_indent(depth + 1), self.escape_text(text))?;
                        }
                        Node::Comment(comment) => {
                            writeln!(writer, "{}<!-- {} -->", self.get_indent(depth + 1), comment)?;
                        }
                    }
                }
                
                writeln!(writer, "{}</{}>", indent, self.get_element_name(element, namespaces))?;
            }
        }
        
        Ok(())
    }
    
    fn get_element_name(&self, element: &Element, namespaces: &IndexMap<String, String>) -> String {
        if let Some(ns) = &element.namespace {
            if let Some(prefix) = namespaces.iter().find(|(_, v)| *v == ns).map(|(k, _)| k) {
                return format!("{}:{}", prefix, element.name);
            }
        } else if !namespaces.is_empty() {
            // Use default namespace prefix
            if let Some((prefix, _)) = namespaces.first() {
                return format!("{}:{}", prefix, element.name);
            }
        }
        element.name.clone()
    }
    
    fn get_indent(&self, depth: usize) -> String {
        match self.config.indent_char {
            crate::determinism::IndentChar::Space => " ".repeat(depth * self.config.indent_width),
            crate::determinism::IndentChar::Tab => "\t".repeat(depth),
        }
    }
    
    fn escape_text(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
    
    fn escape_attribute(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}