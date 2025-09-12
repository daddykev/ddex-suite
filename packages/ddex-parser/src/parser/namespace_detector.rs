//! # Namespace Detection and Management for DDEX Parser
//! 
//! This module provides comprehensive namespace detection and storage for DDEX XML parsing,
//! handling scope inheritance, default namespaces, and custom extensions.

use ddex_core::namespace::{NamespaceRegistry, NamespaceScope, NamespaceInfo, DDEXStandard, NamespaceWarning};
use ddex_core::models::versions::ERNVersion;
use indexmap::IndexMap;
use quick_xml::events::{Event, BytesStart};
use quick_xml::Reader;
use std::io::BufRead;
use crate::error::ParseError;
use tracing::{debug, warn};

/// Comprehensive namespace detection and management
#[derive(Debug, Clone)]
pub struct NamespaceDetector {
    /// Registry of known namespaces
    registry: NamespaceRegistry,
    /// Stack of namespace scopes for element-level inheritance
    scope_stack: Vec<NamespaceScope>,
    /// All detected namespace declarations in document order
    detected_namespaces: IndexMap<String, String>, // prefix -> uri
    /// Namespace aliases found in the document
    namespace_aliases: IndexMap<String, Vec<String>>, // uri -> [prefixes]
    /// Default namespace stack
    default_namespace_stack: Vec<Option<String>>,
    /// Detected ERN version from namespaces
    detected_version: Option<ERNVersion>,
    /// Warnings collected during namespace processing
    warnings: Vec<NamespaceWarning>,
}

/// Namespace detection result
#[derive(Debug, Clone)]
pub struct NamespaceDetectionResult {
    /// All namespace declarations found
    pub declarations: IndexMap<String, String>,
    /// Detected ERN version
    pub version: Option<ERNVersion>,
    /// Namespace scope tree
    pub root_scope: NamespaceScope,
    /// Warnings about namespace usage
    pub warnings: Vec<NamespaceWarning>,
    /// Default namespace at root level
    pub default_namespace: Option<String>,
    /// Custom namespaces detected
    pub custom_namespaces: Vec<NamespaceInfo>,
}

impl NamespaceDetector {
    /// Create new namespace detector
    pub fn new() -> Self {
        Self {
            registry: NamespaceRegistry::new(),
            scope_stack: vec![NamespaceScope::new()],
            detected_namespaces: IndexMap::new(),
            namespace_aliases: IndexMap::new(),
            default_namespace_stack: vec![None],
            detected_version: None,
            warnings: Vec::new(),
        }
    }

    /// Detect namespaces from XML content
    pub fn detect_from_xml<R: BufRead>(&mut self, reader: R) -> Result<NamespaceDetectionResult, ParseError> {
        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);
        
        let mut buf = Vec::new();
        
        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    self.process_start_element(e)?;
                }
                Ok(Event::End(_)) => {
                    self.pop_namespace_scope();
                }
                Ok(Event::Eof) => break,
                Ok(_) => {} // Ignore other events for namespace detection
                Err(e) => return Err(ParseError::XmlError { 
                    message: format!("XML parsing error: {}", e),
                    location: crate::error::ErrorLocation::default(),
                }),
            }
            buf.clear();
        }

        // Validate detected namespaces
        self.validate_namespaces();
        
        Ok(self.build_result())
    }

    /// Process a start element for namespace declarations
    fn process_start_element(&mut self, element: &BytesStart) -> Result<(), ParseError> {
        // Create new scope for this element
        let current_scope = self.scope_stack.last().unwrap().clone();
        let mut new_scope = current_scope.new_child();
        
        // Extract namespace declarations from attributes
        let mut has_namespace_declarations = false;
        let mut new_default_namespace = self.default_namespace_stack.last().cloned().unwrap_or(None);
        
        for attr_result in element.attributes() {
            let attr = attr_result.map_err(|e| ParseError::XmlError { 
                message: format!("Attribute error: {}", e),
                location: crate::error::ErrorLocation::default(),
            })?;
            let key = String::from_utf8_lossy(attr.key.as_ref());
            let value = String::from_utf8_lossy(&attr.value).to_string();
            
            if key == "xmlns" {
                // Default namespace declaration
                debug!("Found default namespace declaration: {}", value);
                new_default_namespace = Some(value.clone());
                new_scope.declare_namespace("".to_string(), value.clone());
                self.detected_namespaces.insert("".to_string(), value.clone());
                has_namespace_declarations = true;
                
                // Try to detect ERN version
                if let Some(version) = self.registry.detect_version(&value) {
                    if self.detected_version.is_none() {
                        self.detected_version = Some(version);
                        debug!("Detected ERN version: {:?} from namespace: {}", version, value);
                    }
                }
            } else if key.starts_with("xmlns:") {
                // Prefixed namespace declaration
                let prefix = key.strip_prefix("xmlns:").unwrap_or("");
                debug!("Found namespace declaration: {}={}", prefix, value);
                
                new_scope.declare_namespace(prefix.to_string(), value.clone());
                self.detected_namespaces.insert(prefix.to_string(), value.clone());
                has_namespace_declarations = true;
                
                // Track namespace aliases
                self.namespace_aliases
                    .entry(value.clone())
                    .or_insert_with(Vec::new)
                    .push(prefix.to_string());
                
                // Try to detect ERN version
                if let Some(version) = self.registry.detect_version(&value) {
                    if self.detected_version.is_none() {
                        self.detected_version = Some(version);
                        debug!("Detected ERN version: {:?} from namespace: {}", version, value);
                    }
                }
            }
        }
        
        // Push new scope and default namespace
        self.scope_stack.push(new_scope);
        self.default_namespace_stack.push(new_default_namespace);
        
        Ok(())
    }

    /// Pop namespace scope when closing an element
    fn pop_namespace_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
        if self.default_namespace_stack.len() > 1 {
            self.default_namespace_stack.pop();
        }
    }

    /// Validate detected namespaces against known standards
    fn validate_namespaces(&mut self) {
        let validation_warnings = self.registry.validate_declarations(&self.detected_namespaces);
        self.warnings.extend(validation_warnings);
    }

    /// Build the final detection result
    fn build_result(&self) -> NamespaceDetectionResult {
        // Identify custom namespaces
        let mut custom_namespaces = Vec::new();
        for (prefix, uri) in &self.detected_namespaces {
            if self.registry.get_namespace_info(uri).is_none() {
                // This is a custom namespace
                let custom_info = NamespaceInfo {
                    uri: uri.clone(),
                    preferred_prefix: prefix.clone(),
                    alternative_prefixes: self.namespace_aliases.get(uri)
                        .cloned()
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|p| p != prefix)
                        .collect(),
                    standard: DDEXStandard::Custom("Unknown".to_string()),
                    version: None,
                    required: false,
                };
                custom_namespaces.push(custom_info);
            }
        }

        NamespaceDetectionResult {
            declarations: self.detected_namespaces.clone(),
            version: self.detected_version,
            root_scope: self.scope_stack.first().cloned().unwrap_or_default(),
            warnings: self.warnings.clone(),
            default_namespace: self.default_namespace_stack.first().cloned().unwrap_or(None),
            custom_namespaces,
        }
    }

    /// Get current namespace scope
    pub fn current_scope(&self) -> &NamespaceScope {
        self.scope_stack.last().unwrap()
    }

    /// Resolve a prefix to its URI in current scope
    pub fn resolve_prefix(&self, prefix: &str) -> Option<String> {
        self.current_scope().resolve_prefix(prefix)
    }

    /// Get default namespace in current scope
    pub fn get_default_namespace(&self) -> Option<&String> {
        self.default_namespace_stack.last().unwrap().as_ref()
    }

    /// Check if a namespace is declared in current scope
    pub fn is_namespace_declared(&self, uri: &str) -> bool {
        self.current_scope().is_namespace_declared(uri)
    }

    /// Find prefix for a namespace URI in current scope
    pub fn find_prefix_for_uri(&self, uri: &str) -> Option<String> {
        self.current_scope().find_prefix_for_uri(uri)
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: NamespaceWarning) {
        warn!("Namespace warning: {}", warning);
        self.warnings.push(warning);
    }

    /// Get detected ERN version
    pub fn get_detected_version(&self) -> Option<ERNVersion> {
        self.detected_version
    }

    /// Get all detected namespace declarations
    pub fn get_detected_namespaces(&self) -> &IndexMap<String, String> {
        &self.detected_namespaces
    }

    /// Get namespace aliases
    pub fn get_namespace_aliases(&self) -> &IndexMap<String, Vec<String>> {
        &self.namespace_aliases
    }
}

/// Namespace context for maintaining state during parsing
#[derive(Debug, Clone)]
pub struct NamespaceContext {
    /// Current namespace scope
    pub current_scope: NamespaceScope,
    /// Detected namespaces at document level
    pub document_namespaces: IndexMap<String, String>,
    /// Current default namespace
    pub default_namespace: Option<String>,
    /// Detected ERN version
    pub ern_version: Option<ERNVersion>,
}

impl NamespaceContext {
    /// Create new namespace context from detection result
    pub fn from_detection_result(result: NamespaceDetectionResult) -> Self {
        Self {
            current_scope: result.root_scope,
            document_namespaces: result.declarations,
            default_namespace: result.default_namespace,
            ern_version: result.version,
        }
    }

    /// Create a new child context for nested elements
    pub fn create_child(&self) -> Self {
        Self {
            current_scope: self.current_scope.new_child(),
            document_namespaces: self.document_namespaces.clone(),
            default_namespace: self.default_namespace.clone(),
            ern_version: self.ern_version,
        }
    }

    /// Declare a namespace in current scope
    pub fn declare_namespace(&mut self, prefix: String, uri: String) {
        self.current_scope.declare_namespace(prefix, uri);
    }

    /// Resolve element name with namespace
    pub fn resolve_element_name(&self, local_name: &str, prefix: Option<&str>) -> ResolvedName {
        match prefix {
            Some(p) => {
                if let Some(uri) = self.current_scope.resolve_prefix(p) {
                    ResolvedName::Qualified {
                        local_name: local_name.to_string(),
                        namespace_uri: uri,
                        prefix: p.to_string(),
                    }
                } else {
                    ResolvedName::Unresolved {
                        local_name: local_name.to_string(),
                        prefix: Some(p.to_string()),
                    }
                }
            }
            None => {
                // Use default namespace if available
                if let Some(uri) = &self.default_namespace {
                    ResolvedName::Qualified {
                        local_name: local_name.to_string(),
                        namespace_uri: uri.clone(),
                        prefix: "".to_string(),
                    }
                } else {
                    ResolvedName::Unqualified {
                        local_name: local_name.to_string(),
                    }
                }
            }
        }
    }
}

/// Resolved element or attribute name
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedName {
    /// Fully qualified name with namespace
    Qualified {
        local_name: String,
        namespace_uri: String,
        prefix: String,
    },
    /// Unqualified name (no namespace)
    Unqualified {
        local_name: String,
    },
    /// Unresolved prefix
    Unresolved {
        local_name: String,
        prefix: Option<String>,
    },
}

impl Default for NamespaceDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_namespace_detection_ern_43() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" 
                       xmlns:avs="http://ddex.net/xml/avs"
                       xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <ern:MessageHeader>
        <ern:MessageId>MSG001</ern:MessageId>
    </ern:MessageHeader>
</ern:NewReleaseMessage>"#;
        
        let mut detector = NamespaceDetector::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = detector.detect_from_xml(cursor).unwrap();
        
        assert_eq!(result.version, Some(ERNVersion::V4_3));
        assert!(result.declarations.contains_key("ern"));
        assert!(result.declarations.contains_key("avs"));
        assert!(result.declarations.contains_key("xsi"));
        assert_eq!(result.declarations.get("ern"), Some(&"http://ddex.net/xml/ern/43".to_string()));
    }

    #[test]
    fn test_default_namespace_detection() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/42" 
                   xmlns:avs="http://ddex.net/xml/avs">
    <MessageHeader>
        <MessageId>MSG001</MessageId>
    </MessageHeader>
</NewReleaseMessage>"#;
        
        let mut detector = NamespaceDetector::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = detector.detect_from_xml(cursor).unwrap();
        
        assert_eq!(result.version, Some(ERNVersion::V4_2));
        assert_eq!(result.default_namespace, Some("http://ddex.net/xml/ern/42".to_string()));
        assert!(result.declarations.contains_key(""));
    }

    #[test]
    fn test_custom_namespace_detection() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" 
                       xmlns:custom="http://example.com/custom">
    <ern:MessageHeader>
        <custom:CustomElement>Test</custom:CustomElement>
    </ern:MessageHeader>
</ern:NewReleaseMessage>"#;
        
        let mut detector = NamespaceDetector::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = detector.detect_from_xml(cursor).unwrap();
        
        assert_eq!(result.custom_namespaces.len(), 1);
        assert_eq!(result.custom_namespaces[0].uri, "http://example.com/custom");
        assert_eq!(result.custom_namespaces[0].preferred_prefix, "custom");
    }

    #[test]
    fn test_namespace_scope_inheritance() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <ern:MessageHeader xmlns:local="http://example.com/local">
        <local:LocalElement>
            <ern:ErnElement />
        </local:LocalElement>
    </ern:MessageHeader>
</ern:NewReleaseMessage>"#;
        
        let mut detector = NamespaceDetector::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = detector.detect_from_xml(cursor).unwrap();
        
        // Both namespaces should be detected
        assert!(result.declarations.contains_key("ern"));
        assert!(result.declarations.contains_key("local"));
    }

    #[test]
    fn test_namespace_context() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43" 
                       xmlns:avs="http://ddex.net/xml/avs">
</ern:NewReleaseMessage>"#;
        
        let mut detector = NamespaceDetector::new();
        let cursor = Cursor::new(xml.as_bytes());
        let result = detector.detect_from_xml(cursor).unwrap();
        
        let context = NamespaceContext::from_detection_result(result);
        
        let resolved = context.resolve_element_name("MessageHeader", Some("ern"));
        match resolved {
            ResolvedName::Qualified { local_name, namespace_uri, prefix } => {
                assert_eq!(local_name, "MessageHeader");
                assert_eq!(namespace_uri, "http://ddex.net/xml/ern/43");
                assert_eq!(prefix, "ern");
            }
            _ => panic!("Expected qualified name"),
        }
    }
}