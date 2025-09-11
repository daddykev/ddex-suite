//! Extension restoration system for preserving unknown XML elements during build
//!
//! This module provides functionality to restore XML fragments that were captured
//! during parsing, ensuring perfect round-trip fidelity for documents containing
//! proprietary extensions.

use ddex_core::models::{Extensions, XmlFragment, ProcessingInstruction, Comment};
use crate::error::BuildError;
use indexmap::IndexMap;
use std::io::Write;

/// Extension restoration context during building
#[derive(Debug, Clone)]
pub struct ExtensionRestorationContext {
    /// Extensions to restore
    pub extensions: Extensions,
    
    /// Current element path (for location matching)
    pub current_path: Vec<String>,
    
    /// Namespace prefix mappings for restoration
    pub namespace_mappings: IndexMap<String, String>,
    
    /// Whether to restore extensions at the current location
    pub restore_extensions: bool,
}

impl ExtensionRestorationContext {
    /// Create a new extension restoration context
    pub fn new(extensions: Extensions) -> Self {
        let namespace_mappings = extensions.global_namespaces.clone();
        
        Self {
            extensions,
            current_path: Vec::new(),
            namespace_mappings,
            restore_extensions: true,
        }
    }

    /// Enter an element during building
    pub fn enter_element(&mut self, element_name: &str) {
        self.current_path.push(element_name.to_string());
    }

    /// Exit an element during building
    pub fn exit_element(&mut self) -> Option<String> {
        self.current_path.pop()
    }

    /// Get the current element path as a string
    pub fn current_path_string(&self) -> String {
        self.current_path.join("/")
    }

    /// Get extensions that should be restored at the current location
    pub fn get_extensions_for_current_location(&self) -> Vec<(&String, &XmlFragment)> {
        let current_path = self.current_path_string();
        self.extensions.get_fragments_matching(&current_path)
    }

    /// Get extensions that should be restored as children of the current element
    pub fn get_child_extensions(&self, child_element_name: &str) -> Vec<(&String, &XmlFragment)> {
        let child_path = if self.current_path.is_empty() {
            child_element_name.to_string()
        } else {
            format!("{}/{}", self.current_path_string(), child_element_name)
        };
        
        self.extensions.get_fragments_matching(&child_path)
    }

    /// Check if there are any extensions to restore
    pub fn has_extensions(&self) -> bool {
        !self.extensions.is_empty()
    }

    /// Get document-level processing instructions
    pub fn get_document_processing_instructions(&self) -> &[ProcessingInstruction] {
        &self.extensions.document_processing_instructions
    }

    /// Get document-level comments
    pub fn get_document_comments(&self) -> &[Comment] {
        &self.extensions.document_comments
    }

    /// Get global namespace declarations
    pub fn get_global_namespaces(&self) -> &IndexMap<String, String> {
        &self.extensions.global_namespaces
    }
}

/// Extension restoration writer that wraps an XML writer
pub struct ExtensionAwareWriter<W: Write> {
    /// Underlying writer
    writer: W,
    
    /// Extension restoration context
    context: ExtensionRestorationContext,
    
    /// Current indentation level
    indent_level: usize,
    
    /// Whether to use pretty printing
    pretty_print: bool,
}

impl<W: Write> ExtensionAwareWriter<W> {
    /// Create a new extension-aware writer
    pub fn new(writer: W, extensions: Extensions) -> Self {
        Self {
            writer,
            context: ExtensionRestorationContext::new(extensions),
            indent_level: 0,
            pretty_print: true,
        }
    }

    /// Create a new extension-aware writer with custom formatting
    pub fn with_formatting(writer: W, extensions: Extensions, pretty_print: bool) -> Self {
        Self {
            writer,
            context: ExtensionRestorationContext::new(extensions),
            indent_level: 0,
            pretty_print,
        }
    }

    /// Write document-level processing instructions
    pub fn write_document_processing_instructions(&mut self) -> Result<(), BuildError> {
        for pi in self.context.get_document_processing_instructions() {
            write!(self.writer, "<?{}", pi.target)?;
            if let Some(ref data) = pi.data {
                write!(self.writer, " {}", data)?;
            }
            writeln!(self.writer, "?>")?;
        }
        Ok(())
    }

    /// Write document-level comments
    pub fn write_document_comments(&mut self) -> Result<(), BuildError> {
        for comment in self.context.get_document_comments() {
            writeln!(self.writer, "{}", comment.to_xml())?;
        }
        Ok(())
    }

    /// Write namespace declarations for extensions
    pub fn write_extension_namespaces(&mut self, existing_namespaces: &IndexMap<String, String>) -> Result<(), BuildError> {
        for (prefix, uri) in self.context.get_global_namespaces() {
            // Only write if not already declared
            if !existing_namespaces.contains_key(prefix) {
                if prefix.is_empty() {
                    write!(self.writer, " xmlns=\"{}\"", uri)?;
                } else {
                    write!(self.writer, " xmlns:{}=\"{}\"", prefix, uri)?;
                }
            }
        }
        Ok(())
    }

    /// Enter an element and potentially write extensions
    pub fn enter_element(&mut self, element_name: &str) -> Result<(), BuildError> {
        self.context.enter_element(element_name);
        self.indent_level += 1;
        Ok(())
    }

    /// Exit an element and potentially write extensions
    pub fn exit_element(&mut self) -> Result<(), BuildError> {
        self.indent_level = self.indent_level.saturating_sub(1);
        
        // Write any extensions that should appear at this location
        self.write_extensions_at_current_location()?;
        
        self.context.exit_element();
        Ok(())
    }

    /// Write extensions that should appear at the current location
    pub fn write_extensions_at_current_location(&mut self) -> Result<(), BuildError> {
        let extensions = self.context.get_extensions_for_current_location();
        
        for (location, fragment) in extensions {
            self.write_xml_fragment(fragment)?;
        }
        
        Ok(())
    }

    /// Write extensions that should appear as children of the current element
    pub fn write_child_extensions(&mut self, child_element_name: &str) -> Result<(), BuildError> {
        let extensions = self.context.get_child_extensions(child_element_name);
        
        for (location, fragment) in extensions {
            self.write_xml_fragment(fragment)?;
        }
        
        Ok(())
    }

    /// Write a single XML fragment
    pub fn write_xml_fragment(&mut self, fragment: &XmlFragment) -> Result<(), BuildError> {
        if self.pretty_print {
            let canonical_xml = fragment.to_canonical_xml(self.indent_level);
            write!(self.writer, "{}", canonical_xml)?;
        } else {
            // Use raw content for non-pretty printing
            write!(self.writer, "{}", fragment.raw_content)?;
        }
        
        if self.pretty_print {
            writeln!(self.writer)?;
        }
        
        Ok(())
    }

    /// Write all extensions that match a specific pattern
    pub fn write_extensions_matching(&mut self, pattern: &str) -> Result<(), BuildError> {
        let extensions = self.context.extensions.get_fragments_matching(pattern);
        
        for (location, fragment) in extensions {
            self.write_xml_fragment(fragment)?;
        }
        
        Ok(())
    }

    /// Get the underlying writer
    pub fn into_inner(self) -> W {
        self.writer
    }

    /// Get a mutable reference to the underlying writer
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Check if there are extensions to restore at the current location
    pub fn has_extensions_at_current_location(&self) -> bool {
        !self.context.get_extensions_for_current_location().is_empty()
    }

    /// Check if there are child extensions for a specific element
    pub fn has_child_extensions(&self, child_element_name: &str) -> bool {
        !self.context.get_child_extensions(child_element_name).is_empty()
    }
}

impl<W: Write> Write for ExtensionAwareWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

/// Extension restoration strategies
#[derive(Debug, Clone, PartialEq)]
pub enum RestorationStrategy {
    /// Restore extensions exactly as they were captured
    Exact,
    
    /// Restore extensions with canonical formatting
    Canonical,
    
    /// Restore extensions but merge with existing content
    Merge,
    
    /// Skip extension restoration
    Skip,
}

/// Extension restoration options
#[derive(Debug, Clone)]
pub struct RestorationOptions {
    /// Restoration strategy
    pub strategy: RestorationStrategy,
    
    /// Whether to validate extensions before restoration
    pub validate: bool,
    
    /// Whether to preserve original formatting
    pub preserve_formatting: bool,
    
    /// Whether to restore comments
    pub restore_comments: bool,
    
    /// Whether to restore processing instructions
    pub restore_processing_instructions: bool,
    
    /// Custom namespace prefix mappings
    pub custom_namespace_mappings: IndexMap<String, String>,
}

impl Default for RestorationOptions {
    fn default() -> Self {
        Self {
            strategy: RestorationStrategy::Canonical,
            validate: true,
            preserve_formatting: false,
            restore_comments: true,
            restore_processing_instructions: true,
            custom_namespace_mappings: IndexMap::new(),
        }
    }
}

impl RestorationOptions {
    /// Create options for exact restoration (preserving original formatting)
    pub fn exact() -> Self {
        Self {
            strategy: RestorationStrategy::Exact,
            preserve_formatting: true,
            ..Default::default()
        }
    }

    /// Create options for canonical restoration (formatted consistently)
    pub fn canonical() -> Self {
        Self {
            strategy: RestorationStrategy::Canonical,
            preserve_formatting: false,
            ..Default::default()
        }
    }

    /// Create options that skip extension restoration
    pub fn skip() -> Self {
        Self {
            strategy: RestorationStrategy::Skip,
            restore_comments: false,
            restore_processing_instructions: false,
            ..Default::default()
        }
    }
}

/// Utility functions for extension restoration
pub mod restoration_utils {
    use super::*;

    /// Merge two extension collections
    pub fn merge_extensions(base: Extensions, additional: Extensions) -> Extensions {
        let mut merged = base;
        merged.merge(additional);
        merged
    }

    /// Filter extensions by location pattern
    pub fn filter_extensions_by_pattern(extensions: &Extensions, pattern: &str) -> Extensions {
        let mut filtered = Extensions::new();
        
        for (location, fragment) in extensions.get_fragments_matching(pattern) {
            filtered.add_fragment(location.clone(), fragment.clone());
        }
        
        // Copy other extension data
        filtered.global_namespaces = extensions.global_namespaces.clone();
        filtered.document_processing_instructions = extensions.document_processing_instructions.clone();
        filtered.document_comments = extensions.document_comments.clone();
        
        filtered
    }

    /// Validate that all extensions can be restored safely
    pub fn validate_extensions(extensions: &Extensions) -> Result<(), BuildError> {
        for (location, fragment) in &extensions.fragments {
            // Validate fragment content
            if let Err(msg) = ddex_core::models::extensions::utils::validate_xml_fragment(fragment) {
                return Err(BuildError::ValidationFailed {
                    errors: vec![format!("Extension validation failed at {}: {}", location, msg)],
                });
            }
        }
        Ok(())
    }

    /// Get restoration statistics
    pub fn get_restoration_stats(extensions: &Extensions) -> RestorationStats {
        RestorationStats {
            fragment_count: extensions.fragments.len(),
            namespace_count: extensions.global_namespaces.len(),
            comment_count: extensions.document_comments.len(),
            processing_instruction_count: extensions.document_processing_instructions.len(),
            total_size: extensions.fragments.values()
                .map(|f| f.raw_content.len())
                .sum(),
        }
    }

    /// Restoration statistics
    #[derive(Debug, Clone)]
    pub struct RestorationStats {
        pub fragment_count: usize,
        pub namespace_count: usize,
        pub comment_count: usize,
        pub processing_instruction_count: usize,
        pub total_size: usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddex_core::models::XmlFragment;

    fn create_test_extensions() -> Extensions {
        let mut extensions = Extensions::new();
        
        let fragment = XmlFragment::with_namespace(
            "customElement".to_string(),
            Some("http://example.com/custom".to_string()),
            Some("custom".to_string()),
            "<custom:customElement>test content</custom:customElement>".to_string(),
        );
        
        extensions.add_fragment("message/header/customElement".to_string(), fragment);
        extensions.add_global_namespace("custom".to_string(), "http://example.com/custom".to_string());
        extensions.add_document_comment("This is a test comment".to_string());
        
        extensions
    }

    #[test]
    fn test_restoration_context() {
        let extensions = create_test_extensions();
        let mut context = ExtensionRestorationContext::new(extensions);
        
        context.enter_element("message");
        context.enter_element("header");
        
        let current_extensions = context.get_extensions_for_current_location();
        assert!(!current_extensions.is_empty());
        
        context.exit_element();
        context.exit_element();
    }

    #[test]
    fn test_extension_aware_writer() {
        let extensions = create_test_extensions();
        let mut buffer = Vec::new();
        
        {
            let mut writer = ExtensionAwareWriter::new(&mut buffer, extensions);
            writer.write_document_comments().unwrap();
            writer.enter_element("message").unwrap();
            writer.enter_element("header").unwrap();
            writer.write_extensions_at_current_location().unwrap();
            writer.exit_element().unwrap();
            writer.exit_element().unwrap();
        }
        
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("This is a test comment"));
    }

    #[test]
    fn test_restoration_options() {
        let exact_options = RestorationOptions::exact();
        assert_eq!(exact_options.strategy, RestorationStrategy::Exact);
        assert!(exact_options.preserve_formatting);

        let canonical_options = RestorationOptions::canonical();
        assert_eq!(canonical_options.strategy, RestorationStrategy::Canonical);
        assert!(!canonical_options.preserve_formatting);

        let skip_options = RestorationOptions::skip();
        assert_eq!(skip_options.strategy, RestorationStrategy::Skip);
        assert!(!skip_options.restore_comments);
    }

    #[test]
    fn test_extension_validation() {
        let extensions = create_test_extensions();
        assert!(restoration_utils::validate_extensions(&extensions).is_ok());
    }

    #[test]
    fn test_restoration_stats() {
        let extensions = create_test_extensions();
        let stats = restoration_utils::get_restoration_stats(&extensions);
        
        assert_eq!(stats.fragment_count, 1);
        assert_eq!(stats.namespace_count, 1);
        assert_eq!(stats.comment_count, 1);
        assert!(stats.total_size > 0);
    }
}