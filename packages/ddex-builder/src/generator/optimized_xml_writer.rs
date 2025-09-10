//! High-performance XML writer optimized for DDEX Builder
//! 
//! This writer uses string interning, buffer pooling, and vectorized operations
//! to achieve target performance of <10ms for typical albums.

use crate::ast::{AST, Element, Node};
use crate::determinism::{DeterminismConfig, IndentChar};
use crate::error::BuildError;
use crate::optimized_strings::{BuildContext, OptimizedString, buffer_sizes};
use indexmap::IndexMap;
use std::io::Write;
use std::fmt::Write as FmtWrite;

/// High-performance XML writer with optimizations
pub struct OptimizedXmlWriter<'a> {
    config: DeterminismConfig,
    context: &'a mut BuildContext,
}

impl<'a> OptimizedXmlWriter<'a> {
    /// Create a new optimized XML writer
    pub fn new(config: DeterminismConfig, context: &'a mut BuildContext) -> Self {
        Self { config, context }
    }
    
    /// Write AST to XML string with performance optimizations
    pub fn write(&mut self, ast: &AST) -> Result<String, BuildError> {
        // Pre-calculate estimated size based on AST complexity
        let estimated_size = self.estimate_output_size(ast);
        
        // Get pre-sized buffer from pool
        let mut buffer = self.context.get_xml_buffer(estimated_size);
        
        // Write XML declaration (static strings for performance)
        buffer.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        
        // Write root element with namespaces
        self.write_element_optimized(&mut buffer, &ast.root, &ast.namespaces, 
                                   ast.schema_location.as_deref(), 0)?;
        
        // Return buffer to pool for reuse
        let result = buffer.clone();
        self.context.return_xml_buffer(buffer);
        
        Ok(result)
    }
    
    /// Estimate output size to pre-allocate buffers efficiently
    fn estimate_output_size(&self, ast: &AST) -> usize {
        let element_count = self.count_elements(&ast.root);
        
        // Use our buffer size estimation
        let track_count = self.estimate_track_count(&ast.root);
        buffer_sizes::estimated_xml_size(track_count)
    }
    
    /// Count total elements in AST for size estimation
    fn count_elements(&self, element: &Element) -> usize {
        1 + element.children.iter()
            .map(|child| match child {
                Node::Element(elem) => self.count_elements(elem),
                _ => 0,
            })
            .sum::<usize>()
    }
    
    /// Estimate track count for buffer sizing
    fn estimate_track_count(&self, element: &Element) -> usize {
        // Look for SoundRecording elements as proxy for track count
        self.count_sound_recordings(element)
    }
    
    /// Count SoundRecording elements
    fn count_sound_recordings(&self, element: &Element) -> usize {
        let mut count = 0;
        
        if element.name == "SoundRecording" {
            count += 1;
        }
        
        for child in &element.children {
            if let Node::Element(child_elem) = child {
                count += self.count_sound_recordings(child_elem);
            }
        }
        
        count.max(1) // At least 1 for sizing
    }
    
    /// Optimized element writing with string interning and fast paths
    fn write_element_optimized(
        &mut self,
        writer: &mut String,
        element: &Element,
        namespaces: &IndexMap<String, String>,
        schema_location: Option<&str>,
        depth: usize,
    ) -> Result<(), BuildError> {
        // Pre-calculate indent (cache common depths)
        let indent = self.get_optimized_indent(depth);
        
        // Start tag with capacity hint
        writer.reserve(128); // Common element size
        writer.push_str(&indent);
        writer.push('<');
        
        // Optimize element name with interning
        let element_name = self.optimize_element_name(element, namespaces, depth);
        writer.push_str(element_name.as_str());
        
        // Add namespace declarations on root element
        if depth == 0 {
            for (prefix, uri) in namespaces {
                writer.push_str(" xmlns:");
                writer.push_str(prefix);
                writer.push_str("=\"");
                writer.push_str(uri);
                writer.push('"');
            }
            
            if let Some(location) = schema_location {
                writer.push_str(" xsi:schemaLocation=\"");
                writer.push_str(location);
                writer.push('"');
            }
        }
        
        // Add attributes (in deterministic order)
        for (key, value) in &element.attributes {
            writer.push(' ');
            writer.push_str(key);
            writer.push_str("=\"");
            // Use optimized escaping
            self.escape_attribute_into(value, writer);
            writer.push('"');
        }
        
        // Handle children with fast paths
        if element.children.is_empty() {
            writer.push_str("/>\n");
        } else {
            // Check for common patterns
            let only_text = element.children.len() == 1 && 
                matches!(&element.children[0], Node::Text(_));
            
            if only_text {
                // Inline text content (most common case)
                writer.push('>');
                if let Node::Text(text) = &element.children[0] {
                    self.escape_text_into(text, writer);
                }
                writer.push_str("</");
                writer.push_str(element_name.as_str());
                writer.push_str(">\n");
            } else {
                // Has child elements
                writer.push_str(">\n");
                
                // Write children with batch operations when possible
                for child in &element.children {
                    match child {
                        Node::Element(child_elem) => {
                            self.write_element_optimized(writer, child_elem, namespaces, None, depth + 1)?;
                        }
                        Node::Text(text) => {
                            writer.push_str(&self.get_optimized_indent(depth + 1));
                            self.escape_text_into(text, writer);
                            writer.push('\n');
                        }
                        Node::Comment(comment) => {
                            writer.push_str(&self.get_optimized_indent(depth + 1));
                            writer.push_str("<!-- ");
                            writer.push_str(comment);
                            writer.push_str(" -->\n");
                        }
                    }
                }
                
                // Close tag
                writer.push_str(&indent);
                writer.push_str("</");
                writer.push_str(element_name.as_str());
                writer.push_str(">\n");
            }
        }
        
        Ok(())
    }
    
    /// Optimize element name with caching and interning
    fn optimize_element_name(
        &mut self,
        element: &Element,
        namespaces: &IndexMap<String, String>,
        depth: usize,
    ) -> OptimizedString {
        // Common element names are cached as static strings
        let name_with_ns = if let Some(ns) = &element.namespace {
            format!("{}:{}", ns, element.name)
        } else if depth == 0 && !namespaces.is_empty() {
            if let Some((prefix, _)) = namespaces.first() {
                format!("{}:{}", prefix, element.name)
            } else {
                element.name.clone()
            }
        } else {
            element.name.clone()
        };
        
        self.context.optimize_string(&name_with_ns)
    }
    
    /// Cache common indent patterns
    fn get_optimized_indent(&self, depth: usize) -> String {
        // Cache up to 10 levels (covers 99% of DDEX structures)
        static CACHED_SPACE_INDENTS: once_cell::sync::Lazy<Vec<String>> = 
            once_cell::sync::Lazy::new(|| {
                (0..=10).map(|d| " ".repeat(d * 2)).collect()
            });
        
        static CACHED_TAB_INDENTS: once_cell::sync::Lazy<Vec<String>> = 
            once_cell::sync::Lazy::new(|| {
                (0..=10).map(|d| "\t".repeat(d)).collect()
            });
        
        let indent_width = self.config.indent_width;
        
        match self.config.indent_char {
            IndentChar::Space => {
                if depth <= 10 && indent_width == 2 {
                    CACHED_SPACE_INDENTS[depth].clone()
                } else {
                    " ".repeat(depth * indent_width)
                }
            }
            IndentChar::Tab => {
                if depth <= 10 && indent_width == 1 {
                    CACHED_TAB_INDENTS[depth].clone()
                } else {
                    "\t".repeat(depth * indent_width)
                }
            }
        }
    }
    
    /// In-place text escaping to avoid allocations
    fn escape_text_into(&self, text: &str, writer: &mut String) {
        // Reserve space for worst-case escaping
        writer.reserve(text.len() * 6); // Worst case: all chars become &entity;
        
        for ch in text.chars() {
            match ch {
                '&' => writer.push_str("&amp;"),
                '<' => writer.push_str("&lt;"),
                '>' => writer.push_str("&gt;"),
                _ => writer.push(ch),
            }
        }
    }
    
    /// In-place attribute escaping
    fn escape_attribute_into(&self, text: &str, writer: &mut String) {
        writer.reserve(text.len() * 6);
        
        for ch in text.chars() {
            match ch {
                '&' => writer.push_str("&amp;"),
                '<' => writer.push_str("&lt;"),
                '>' => writer.push_str("&gt;"),
                '"' => writer.push_str("&quot;"),
                '\'' => writer.push_str("&apos;"),
                _ => writer.push(ch),
            }
        }
    }
}

/// Vectorized XML operations for batch processing
pub mod vectorized {
    use super::*;
    use rayon::prelude::*;
    
    /// Write multiple elements in parallel (for large collections)
    pub fn write_elements_parallel<T>(
        elements: &[T],
        context: &mut BuildContext,
        config: &DeterminismConfig,
        converter: impl Fn(&T) -> Element + Send + Sync,
    ) -> Result<Vec<String>, BuildError> 
    where
        T: Send + Sync,
    {
        // Only use parallelization for large collections
        if elements.len() < 10 {
            return write_elements_sequential(elements, context, config, converter);
        }
        
        // Process in parallel chunks
        let chunk_size = (elements.len() / num_cpus::get()).max(1);
        
        elements
            .par_chunks(chunk_size)
            .map(|chunk| {
                // Each thread needs its own context to avoid conflicts
                let mut local_context = BuildContext::new();
                let mut writer = OptimizedXmlWriter::new(config.clone(), &mut local_context);
                
                let mut results = Vec::with_capacity(chunk.len());
                for element in chunk {
                    let converted = converter(element);
                    let ast = AST {
                        root: converted,
                        namespaces: IndexMap::new(),
                        schema_location: None,
                    };
                    results.push(writer.write(&ast)?);
                }
                Ok(results)
            })
            .collect::<Result<Vec<_>, BuildError>>()
            .map(|chunks| chunks.into_iter().flatten().collect())
    }
    
    /// Sequential version for smaller collections
    fn write_elements_sequential<T>(
        elements: &[T],
        context: &mut BuildContext,
        config: &DeterminismConfig,
        converter: impl Fn(&T) -> Element,
    ) -> Result<Vec<String>, BuildError> {
        let mut writer = OptimizedXmlWriter::new(config.clone(), context);
        let mut results = Vec::with_capacity(elements.len());
        
        for element in elements {
            let converted = converter(element);
            let ast = AST {
                root: converted,
                namespaces: IndexMap::new(),
                schema_location: None,
            };
            results.push(writer.write(&ast)?);
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimized_strings::BuildContext;
    
    #[test]
    fn test_optimized_writer_performance() {
        let mut context = BuildContext::new();
        let config = DeterminismConfig::default();
        let mut writer = OptimizedXmlWriter::new(config, &mut context);
        
        // Create a simple AST
        let element = Element {
            name: "TestElement".to_string(),
            namespace: None,
            attributes: IndexMap::new(),
            children: vec![Node::Text("Test content".to_string())],
        };
        
        let ast = AST {
            root: element,
            namespaces: IndexMap::new(),
            schema_location: None,
        };
        
        let result = writer.write(&ast).unwrap();
        assert!(result.contains("<TestElement>Test content</TestElement>"));
        
        // Check that context accumulated statistics
        assert_eq!(context.stats.buffers_requested, 1);
    }
    
    #[test]
    fn test_size_estimation() {
        let mut context = BuildContext::new();
        let config = DeterminismConfig::default();
        let writer = OptimizedXmlWriter::new(config, &mut context);
        
        // Create AST with sound recordings
        let sr_element = Element {
            name: "SoundRecording".to_string(),
            namespace: None,
            attributes: IndexMap::new(),
            children: vec![],
        };
        
        let root = Element {
            name: "NewReleaseMessage".to_string(),
            namespace: None,
            attributes: IndexMap::new(),
            children: vec![Node::Element(sr_element)],
        };
        
        let ast = AST {
            root,
            namespaces: IndexMap::new(),
            schema_location: None,
        };
        
        let estimated = writer.estimate_output_size(&ast);
        assert!(estimated > buffer_sizes::SINGLE_TRACK_XML / 2);
    }
}