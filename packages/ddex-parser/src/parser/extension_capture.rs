//! Extension capture system for preserving unknown XML elements and namespaces
//!
//! This module provides functionality to capture and preserve XML fragments that are
//! not part of the standard DDEX schema, enabling perfect round-trip fidelity for
//! documents containing proprietary extensions.

use ddex_core::models::{Extensions, XmlFragment, ProcessingInstruction, Comment, CommentPosition, extensions::utils};
use quick_xml::{Reader, events::{Event, BytesStart, BytesEnd, BytesText}};
use indexmap::IndexMap;

/// Extension capture context during parsing
#[derive(Debug, Clone)]
pub struct ExtensionCaptureContext {
    /// Current element path (for location tracking)
    pub element_path: Vec<String>,
    
    /// Namespace context (prefix -> URI mappings)
    pub namespace_context: IndexMap<String, String>,
    
    /// Whether we're currently inside an unknown element
    pub in_extension: bool,
    
    /// Depth of unknown element nesting
    pub extension_depth: usize,
    
    /// Buffer for accumulating unknown XML content
    pub extension_buffer: String,
    
    /// Current extension being built
    pub current_extension: Option<XmlFragment>,
    
    /// Extensions collected during parsing
    pub extensions: Extensions,
    
    /// Current line number for position tracking
    pub current_line: usize,
    
    /// Current column number for position tracking
    pub current_column: usize,
}

impl ExtensionCaptureContext {
    /// Create a new extension capture context
    pub fn new() -> Self {
        Self {
            element_path: Vec::new(),
            namespace_context: IndexMap::new(),
            in_extension: false,
            extension_depth: 0,
            extension_buffer: String::new(),
            current_extension: None,
            extensions: Extensions::new(),
            current_line: 1,
            current_column: 1,
        }
    }

    /// Enter an element during parsing
    pub fn enter_element(&mut self, element_name: &str) {
        self.element_path.push(element_name.to_string());
    }

    /// Exit an element during parsing
    pub fn exit_element(&mut self) -> Option<String> {
        self.element_path.pop()
    }

    /// Get the current element path as a string
    pub fn current_path(&self) -> String {
        self.element_path.join("/")
    }

    /// Update namespace context with new declarations
    pub fn add_namespace_declaration(&mut self, prefix: String, uri: String) {
        self.namespace_context.insert(prefix.clone(), uri.clone());
        
        // Also add to global namespaces if it's not a DDEX namespace
        if !utils::is_ddex_namespace(&uri) {
            self.extensions.add_global_namespace(prefix, uri);
        }
    }

    /// Check if an element should be captured as an extension
    pub fn should_capture_element(&self, _element_name: &str, namespace_uri: Option<&str>) -> bool {
        // If we're already in an extension, capture everything
        if self.in_extension {
            return true;
        }

        // Check if this element is from a non-DDEX namespace
        if let Some(ns_uri) = namespace_uri {
            return !utils::is_ddex_namespace(ns_uri);
        }

        // Check if it's an unknown element in the DDEX namespace
        // This would require schema validation, for now we'll be conservative
        false
    }

    /// Start capturing an extension element
    pub fn start_extension_capture(
        &mut self,
        element_name: &str,
        namespace_uri: Option<&str>,
        namespace_prefix: Option<&str>,
    ) {
        self.in_extension = true;
        self.extension_depth = 1;
        self.extension_buffer.clear();

        self.current_extension = Some(XmlFragment::with_namespace(
            element_name.to_string(),
            namespace_uri.map(String::from),
            namespace_prefix.map(String::from),
            String::new(), // Will be filled as we parse
        ));
    }

    /// Add content to the current extension
    pub fn add_extension_content(&mut self, content: &str) {
        if self.in_extension {
            self.extension_buffer.push_str(content);
        }
    }

    /// Process an opening tag during extension capture
    pub fn process_extension_start_tag(&mut self, event: &BytesStart) {
        if !self.in_extension {
            return;
        }

        self.extension_depth += 1;
        self.extension_buffer.push('<');
        self.extension_buffer.push_str(std::str::from_utf8(event.name().as_ref()).unwrap_or("unknown"));

        // Add attributes
        for attr_result in event.attributes() {
            if let Ok(attr) = attr_result {
                self.extension_buffer.push(' ');
                self.extension_buffer.push_str(std::str::from_utf8(attr.key.as_ref()).unwrap_or("unknown"));
                self.extension_buffer.push_str("=\"");
                self.extension_buffer.push_str(&String::from_utf8_lossy(&attr.value));
                self.extension_buffer.push('"');
                
                // Store attribute in current extension
                if let Some(ref mut ext) = self.current_extension {
                    let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("unknown").to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    ext.add_attribute(key, value);
                }
            }
        }

        self.extension_buffer.push('>');
    }

    /// Process a closing tag during extension capture
    pub fn process_extension_end_tag(&mut self, event: &BytesEnd) {
        if !self.in_extension {
            return;
        }

        self.extension_buffer.push_str("</");
        self.extension_buffer.push_str(std::str::from_utf8(event.name().as_ref()).unwrap_or("unknown"));
        self.extension_buffer.push('>');

        self.extension_depth -= 1;

        // If we're back to depth 0, finish capturing this extension
        if self.extension_depth == 0 {
            self.finish_extension_capture();
        }
    }

    /// Process text content during extension capture
    pub fn process_extension_text(&mut self, event: &BytesText) {
        if !self.in_extension {
            return;
        }

        let text = event.unescape().unwrap_or_default();
        self.extension_buffer.push_str(&text);
        
        // If this is simple text content, store it in the fragment
        if let Some(ref mut ext) = self.current_extension {
            if ext.children.is_empty() {
                ext.text_content = Some(text.to_string());
            }
        }
    }

    /// Finish capturing the current extension
    pub fn finish_extension_capture(&mut self) {
        if let Some(mut extension) = self.current_extension.take() {
            extension.raw_content = self.extension_buffer.clone();
            
            // Generate location key
            let namespace_uri = extension.namespace_uri.as_deref();
            let location_key = utils::generate_location_key(
                &self.element_path.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                namespace_uri,
                &extension.element_name,
            );

            self.extensions.add_fragment(location_key, extension);
        }

        self.in_extension = false;
        self.extension_depth = 0;
        self.extension_buffer.clear();
    }

    /// Add a document-level processing instruction
    pub fn add_processing_instruction(&mut self, target: String, data: Option<String>) {
        let pi = ProcessingInstruction::new(target, data);
        self.extensions.add_document_processing_instruction(pi);
    }

    /// Add a document-level comment
    pub fn add_comment(&mut self, comment: String) {
        self.extensions.add_document_comment(comment);
    }
    
    /// Add a position-aware comment
    pub fn add_comment_with_position(
        &mut self, 
        comment: String, 
        position: CommentPosition,
        line_number: Option<usize>,
        column_number: Option<usize>
    ) {
        let xpath = if !self.element_path.is_empty() {
            Some(format!("/{}", self.element_path.join("/")))
        } else {
            None
        };
        
        let comment_struct = Comment::with_location(
            comment,
            position,
            xpath,
            line_number,
            column_number
        );
        
        if self.element_path.is_empty() || matches!(position, CommentPosition::Before | CommentPosition::After) {
            // Document-level or standalone comment
            self.extensions.add_document_comment_structured(comment_struct);
        } else {
            // Element-level comment - add to current extension or buffer for later association
            if let Some(ref mut ext) = self.current_extension {
                ext.comments.push(comment_struct);
            } else {
                // Store for later association with the next element
                self.extensions.add_document_comment_structured(comment_struct);
            }
        }
    }

    /// Get the accumulated extensions
    pub fn into_extensions(self) -> Extensions {
        self.extensions
    }
}

/// Extension-aware XML parser
pub struct ExtensionAwareParser {
    /// Extension capture context
    pub context: ExtensionCaptureContext,
    
    /// Whether to capture extensions
    pub capture_extensions: bool,
}

impl ExtensionAwareParser {
    /// Create a new extension-aware parser
    pub fn new(capture_extensions: bool) -> Self {
        Self {
            context: ExtensionCaptureContext::new(),
            capture_extensions,
        }
    }

    /// Parse XML with extension capture
    pub fn parse_with_extensions(&mut self, xml_content: &str) -> Result<Extensions, Box<dyn std::error::Error>> {
        if !self.capture_extensions {
            return Ok(Extensions::new());
        }

        let mut reader = Reader::from_str(xml_content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let element_name_bytes = e.name();
                    let element_name = std::str::from_utf8(element_name_bytes.as_ref()).unwrap_or("unknown");
                    
                    // Extract namespace information
                    let (namespace_uri, namespace_prefix) = self.extract_namespace_info(e);
                    
                    // Update namespace context with any new declarations
                    for attr_result in e.attributes() {
                        if let Ok(attr) = attr_result {
                            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                            if key.starts_with("xmlns") {
                                let prefix = if key == "xmlns" {
                                    "".to_string()
                                } else {
                                    key.strip_prefix("xmlns:").unwrap_or("").to_string()
                                };
                                let uri = String::from_utf8_lossy(&attr.value).to_string();
                                self.context.add_namespace_declaration(prefix, uri);
                            }
                        }
                    }

                    // Check if we should capture this element
                    if self.context.should_capture_element(element_name, namespace_uri.as_deref()) {
                        if !self.context.in_extension {
                            self.context.start_extension_capture(
                                element_name,
                                namespace_uri.as_deref(),
                                namespace_prefix.as_deref(),
                            );
                        }
                        self.context.process_extension_start_tag(e);
                    } else {
                        self.context.enter_element(element_name);
                    }
                },
                Ok(Event::End(ref e)) => {
                    if self.context.in_extension {
                        self.context.process_extension_end_tag(e);
                    } else {
                        self.context.exit_element();
                    }
                },
                Ok(Event::Text(ref e)) => {
                    if self.context.in_extension {
                        self.context.process_extension_text(e);
                    }
                },
                Ok(Event::Comment(ref e)) => {
                    let comment = String::from_utf8_lossy(e);
                    if self.context.in_extension {
                        self.context.add_extension_content(&format!("<!--{}-->", comment));
                    } else {
                        // Determine comment position based on context
                        let position = if self.context.element_path.is_empty() {
                            CommentPosition::Before
                        } else {
                            CommentPosition::FirstChild
                        };
                        
                        self.context.add_comment_with_position(
                            comment.trim().to_string(),
                            position,
                            Some(self.context.current_line),
                            Some(self.context.current_column)
                        );
                    }
                },
                Ok(Event::PI(ref e)) => {
                    let content = String::from_utf8_lossy(e);
                    // Split processing instruction content into target and data
                    if let Some(space_pos) = content.find(char::is_whitespace) {
                        let target = content[..space_pos].to_string();
                        let data = content[space_pos..].trim().to_string();
                        let data = if data.is_empty() { None } else { Some(data) };
                        self.context.add_processing_instruction(target, data);
                    } else {
                        self.context.add_processing_instruction(content.to_string(), None);
                    }
                },
                Ok(Event::Eof) => break,
                Err(e) => {
                    // Log the error but continue parsing to capture as much as possible
                    eprintln!("Warning: XML parsing error during extension capture: {}", e);
                },
                _ => {}
            }
            buf.clear();
        }

        Ok(self.context.extensions.clone())
    }

    /// Extract namespace information from a start tag
    fn extract_namespace_info(&self, event: &BytesStart) -> (Option<String>, Option<String>) {
        let name_bytes = event.name();
        let name = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("unknown");
        
        if let Some(colon_pos) = name.find(':') {
            let prefix = &name[..colon_pos];
            let namespace_uri = self.context.namespace_context.get(prefix).cloned();
            (namespace_uri, Some(prefix.to_string()))
        } else {
            // Check for default namespace
            let default_ns = self.context.namespace_context.get("").cloned();
            (default_ns, None)
        }
    }
}

/// Utility functions for extension capture
pub mod capture_utils {
    use super::*;

    /// Extract all extensions from XML content
    pub fn extract_extensions(xml_content: &str) -> Result<Extensions, Box<dyn std::error::Error>> {
        let mut parser = ExtensionAwareParser::new(true);
        parser.parse_with_extensions(xml_content)
    }

    /// Check if XML content contains extensions
    pub fn has_extensions(xml_content: &str) -> bool {
        match extract_extensions(xml_content) {
            Ok(extensions) => !extensions.is_empty(),
            Err(_) => false,
        }
    }

    /// Get extension statistics from XML content
    pub fn get_extension_stats(xml_content: &str) -> ExtensionStats {
        match extract_extensions(xml_content) {
            Ok(extensions) => ExtensionStats::from_extensions(&extensions),
            Err(_) => ExtensionStats::default(),
        }
    }

    /// Extension statistics
    #[derive(Debug, Clone, Default)]
    pub struct ExtensionStats {
        pub fragment_count: usize,
        pub namespace_count: usize,
        pub comment_count: usize,
        pub processing_instruction_count: usize,
        pub unique_namespaces: Vec<String>,
    }

    impl ExtensionStats {
        fn from_extensions(extensions: &Extensions) -> Self {
            let unique_namespaces = extensions.global_namespaces.values().cloned().collect();
            
            Self {
                fragment_count: extensions.fragments.len(),
                namespace_count: extensions.global_namespaces.len(),
                comment_count: extensions.document_comments.len(),
                processing_instruction_count: extensions.document_processing_instructions.len(),
                unique_namespaces,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_capture_context() {
        let mut context = ExtensionCaptureContext::new();
        
        context.enter_element("message");
        context.enter_element("header");
        assert_eq!(context.current_path(), "message/header");
        
        context.exit_element();
        assert_eq!(context.current_path(), "message");
    }

    #[test]
    fn test_namespace_detection() {
        let context = ExtensionCaptureContext::new();
        
        // Should not capture DDEX elements
        assert!(!context.should_capture_element("Release", Some("http://ddex.net/xml/ern/43")));
        
        // Should capture non-DDEX elements
        assert!(context.should_capture_element("customElement", Some("http://example.com/custom")));
    }

    #[test]
    fn test_extension_parsing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" xmlns:custom="http://example.com/custom">
  <MessageHeader>
    <MessageId>MSG123</MessageId>
    <custom:CustomField>Custom Value</custom:CustomField>
  </MessageHeader>
  <custom:CustomSection attr="value">
    <custom:NestedElement>Nested Content</custom:NestedElement>
  </custom:CustomSection>
</ern:NewReleaseMessage>"#;

        let extensions = capture_utils::extract_extensions(xml).unwrap();
        assert!(!extensions.is_empty());
        assert!(extensions.global_namespaces.contains_key("custom"));
        assert_eq!(extensions.global_namespaces["custom"], "http://example.com/custom");
    }

    #[test]
    fn test_processing_instruction_capture() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<?custom-instruction data="value"?>
<root>content</root>"#;

        let extensions = capture_utils::extract_extensions(xml).unwrap();
        assert!(!extensions.document_processing_instructions.is_empty());
        assert_eq!(extensions.document_processing_instructions[0].target, "custom-instruction");
    }

    #[test]
    fn test_comment_capture() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- This is a document comment -->
<root>
  <!-- This is an element comment -->
  content
</root>"#;

        let extensions = capture_utils::extract_extensions(xml).unwrap();
        assert!(!extensions.document_comments.is_empty());
    }
}