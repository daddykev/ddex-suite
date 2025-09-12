//! # XML Attribute Extraction and Processing
//! 
//! This module provides comprehensive attribute extraction from XML elements,
//! handling namespace resolution, special attributes, and proper type conversion
//! for both standard DDEX and custom/proprietary attributes.

use ddex_core::models::{AttributeMap, AttributeValue, QName, AttributeType};
use crate::parser::namespace_detector::NamespaceContext;
use crate::error::ParseError;
use indexmap::IndexMap;
use quick_xml::events::{BytesStart, attributes::Attribute};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Comprehensive attribute extractor with namespace awareness
#[derive(Debug, Clone)]
pub struct AttributeExtractor {
    /// Known DDEX attribute types for proper parsing
    ddex_attribute_types: HashMap<String, AttributeType>,
    /// Special attribute handlers
    special_attributes: IndexMap<String, SpecialAttributeHandler>,
}

/// Special attribute handler for attributes requiring custom processing
#[derive(Debug, Clone)]
pub enum SpecialAttributeHandler {
    /// xsi:type attribute (XML Schema instance type)
    XsiType,
    /// xsi:schemaLocation attribute
    XsiSchemaLocation,
    /// xsi:noNamespaceSchemaLocation attribute
    XsiNoNamespaceSchemaLocation,
    /// xsi:nil attribute (indicates null value)
    XsiNil,
    /// Namespace declaration attributes (xmlns, xmlns:*)
    NamespaceDeclaration,
    /// Language and territory codes
    LanguageAndTerritory,
    /// Sequence numbers and ordering
    SequenceNumber,
    /// Boolean flags
    BooleanFlag,
}

/// Attribute extraction result
#[derive(Debug, Clone)]
pub struct AttributeExtractionResult {
    /// All extracted attributes with proper typing
    pub attributes: AttributeMap,
    /// Standard DDEX attributes (subset of all attributes)
    pub standard_attributes: IndexMap<QName, AttributeValue>,
    /// Extension/custom attributes
    pub extension_attributes: IndexMap<QName, AttributeValue>,
    /// Namespace declarations found in this element
    pub namespace_declarations: IndexMap<String, String>,
    /// Special attributes requiring additional processing
    pub special_attributes: IndexMap<QName, SpecialAttributeValue>,
    /// Warnings about attribute processing
    pub warnings: Vec<String>,
}

/// Special attribute values requiring custom handling
#[derive(Debug, Clone, PartialEq)]
pub enum SpecialAttributeValue {
    /// xsi:type with resolved type information
    XsiType {
        type_name: String,
        namespace_uri: Option<String>,
        resolved_type: Option<String>,
    },
    /// Schema location with URI pairs
    SchemaLocation {
        locations: IndexMap<String, String>, // namespace_uri -> schema_location
    },
    /// No namespace schema location
    NoNamespaceSchemaLocation(String),
    /// Nil indicator
    Nil(bool),
    /// Language with territory code
    Language {
        language: String,
        script: Option<String>,
        territory: Option<String>,
    },
    /// Territory code list
    Territory(Vec<String>),
    /// Sequence number for ordering
    Sequence(u32),
    /// Boolean flag
    Flag(bool),
}

impl AttributeExtractor {
    /// Create a new attribute extractor with DDEX knowledge
    pub fn new() -> Self {
        let mut extractor = Self {
            ddex_attribute_types: HashMap::new(),
            special_attributes: IndexMap::new(),
        };
        
        extractor.initialize_ddex_attributes();
        extractor.initialize_special_handlers();
        extractor
    }

    /// Initialize known DDEX attribute types
    fn initialize_ddex_attributes(&mut self) {
        // Language and territory attributes
        self.ddex_attribute_types.insert("LanguageAndScriptCode".to_string(), AttributeType::Language);
        self.ddex_attribute_types.insert("ApplicableTerritoryCode".to_string(), AttributeType::String);
        
        // Boolean attributes
        self.ddex_attribute_types.insert("IsDefault".to_string(), AttributeType::Boolean);
        self.ddex_attribute_types.insert("IsMainArtist".to_string(), AttributeType::Boolean);
        self.ddex_attribute_types.insert("HasChanged".to_string(), AttributeType::Boolean);
        
        // Numeric attributes
        self.ddex_attribute_types.insert("SequenceNumber".to_string(), AttributeType::Integer);
        self.ddex_attribute_types.insert("Duration".to_string(), AttributeType::String); // ISO 8601 duration
        
        // URI attributes
        self.ddex_attribute_types.insert("Namespace".to_string(), AttributeType::Uri);
        
        // Date/time attributes
        self.ddex_attribute_types.insert("CreatedDateTime".to_string(), AttributeType::DateTime);
        self.ddex_attribute_types.insert("UpdatedDateTime".to_string(), AttributeType::DateTime);
    }

    /// Initialize special attribute handlers
    fn initialize_special_handlers(&mut self) {
        // XML Schema Instance attributes
        self.special_attributes.insert("xsi:type".to_string(), SpecialAttributeHandler::XsiType);
        self.special_attributes.insert("xsi:schemaLocation".to_string(), SpecialAttributeHandler::XsiSchemaLocation);
        self.special_attributes.insert("xsi:noNamespaceSchemaLocation".to_string(), SpecialAttributeHandler::XsiNoNamespaceSchemaLocation);
        self.special_attributes.insert("xsi:nil".to_string(), SpecialAttributeHandler::XsiNil);
        
        // Namespace declarations
        self.special_attributes.insert("xmlns".to_string(), SpecialAttributeHandler::NamespaceDeclaration);
        // Note: xmlns:* are handled dynamically
        
        // DDEX specific
        self.special_attributes.insert("LanguageAndScriptCode".to_string(), SpecialAttributeHandler::LanguageAndTerritory);
        self.special_attributes.insert("ApplicableTerritoryCode".to_string(), SpecialAttributeHandler::LanguageAndTerritory);
        self.special_attributes.insert("SequenceNumber".to_string(), SpecialAttributeHandler::SequenceNumber);
        
        // Boolean flags
        self.special_attributes.insert("IsDefault".to_string(), SpecialAttributeHandler::BooleanFlag);
        self.special_attributes.insert("IsMainArtist".to_string(), SpecialAttributeHandler::BooleanFlag);
    }

    /// Extract all attributes from an XML element
    pub fn extract_attributes(
        &self,
        element: &BytesStart,
        namespace_context: &NamespaceContext,
    ) -> Result<AttributeExtractionResult, ParseError> {
        let mut attributes = AttributeMap::new();
        let mut namespace_declarations = IndexMap::new();
        let mut special_attributes = IndexMap::new();
        let warnings = Vec::new();

        debug!("Extracting attributes from element: {}", String::from_utf8_lossy(element.name().as_ref()));

        // Process all attributes
        for attr_result in element.attributes() {
            let attr = attr_result.map_err(|e| ParseError::XmlError {
                message: format!("Failed to read attribute: {}", e),
                location: crate::error::ErrorLocation::default(),
            })?;

            let (qname, attr_value) = self.process_attribute(&attr, namespace_context)?;
            
            // Handle namespace declarations separately
            if qname.is_namespace_declaration() {
                let prefix = if qname.local_name == "xmlns" {
                    "".to_string() // Default namespace
                } else {
                    qname.local_name.clone() // Prefixed namespace
                };
                namespace_declarations.insert(prefix, attr_value.to_xml_value());
                debug!("Found namespace declaration: {}={}", qname.to_xml_name(), attr_value.to_xml_value());
            }

            // Check for special attributes
            if let Some(special_value) = self.process_special_attribute(&qname, &attr_value, namespace_context)? {
                special_attributes.insert(qname.clone(), special_value);
            }

            // Add to main attribute map
            attributes.insert(qname, attr_value);
        }

        // Separate standard and extension attributes
        let standard_attributes = attributes.standard_attributes();
        let extension_attributes = attributes.extension_attributes();

        debug!("Extracted {} total attributes ({} standard, {} extensions)", 
               attributes.len(), standard_attributes.len(), extension_attributes.len());

        Ok(AttributeExtractionResult {
            attributes,
            standard_attributes,
            extension_attributes,
            namespace_declarations,
            special_attributes,
            warnings,
        })
    }

    /// Process a single attribute
    fn process_attribute(
        &self,
        attr: &Attribute,
        namespace_context: &NamespaceContext,
    ) -> Result<(QName, AttributeValue), ParseError> {
        let attr_name = String::from_utf8_lossy(attr.key.as_ref());
        let attr_value = String::from_utf8_lossy(&attr.value);

        debug!("Processing attribute: {}={}", attr_name, attr_value);

        // Create QName with namespace resolution
        let qname = self.resolve_attribute_qname(&attr_name, namespace_context);
        
        // Determine attribute type and parse value
        let parsed_value = if let Some(attr_type) = self.get_attribute_type(&qname) {
            AttributeValue::parse_with_type(&attr_value, attr_type)
                .unwrap_or_else(|e| {
                    warn!("Failed to parse attribute {} as {:?}: {}", qname, attr_type, e);
                    AttributeValue::Raw(attr_value.to_string())
                })
        } else {
            // Default to string for unknown attributes
            AttributeValue::String(attr_value.to_string())
        };

        Ok((qname, parsed_value))
    }

    /// Resolve attribute name to QName with namespace context
    fn resolve_attribute_qname(&self, attr_name: &str, namespace_context: &NamespaceContext) -> QName {
        if let Some((prefix, local_name)) = attr_name.split_once(':') {
            // Prefixed attribute
            if let Some(namespace_uri) = namespace_context.current_scope.resolve_prefix(prefix) {
                QName::with_prefix_and_namespace(local_name, prefix, namespace_uri)
            } else {
                // Unresolved prefix - keep as is with warning
                warn!("Unresolved namespace prefix in attribute: {}", attr_name);
                QName {
                    local_name: local_name.to_string(),
                    namespace_uri: None,
                    prefix: Some(prefix.to_string()),
                }
            }
        } else {
            // Non-prefixed attribute - check if it's a namespace declaration
            if attr_name == "xmlns" || attr_name.starts_with("xmlns:") {
                QName::new(attr_name)
            } else {
                // Regular attribute without namespace
                QName::new(attr_name)
            }
        }
    }

    /// Get the expected type for an attribute
    fn get_attribute_type(&self, qname: &QName) -> Option<AttributeType> {
        // Check by full qualified name first
        if let Some(attr_type) = self.ddex_attribute_types.get(&qname.to_xml_name()) {
            return Some(*attr_type);
        }
        
        // Check by local name
        self.ddex_attribute_types.get(&qname.local_name).copied()
    }

    /// Process special attributes that require custom handling
    fn process_special_attribute(
        &self,
        qname: &QName,
        value: &AttributeValue,
        namespace_context: &NamespaceContext,
    ) -> Result<Option<SpecialAttributeValue>, ParseError> {
        let attr_name = qname.to_xml_name();
        
        if let Some(handler) = self.special_attributes.get(&attr_name) {
            match handler {
                SpecialAttributeHandler::XsiType => {
                    self.process_xsi_type(value, namespace_context)
                },
                SpecialAttributeHandler::XsiSchemaLocation => {
                    self.process_schema_location(value)
                },
                SpecialAttributeHandler::XsiNoNamespaceSchemaLocation => {
                    Ok(Some(SpecialAttributeValue::NoNamespaceSchemaLocation(value.to_xml_value())))
                },
                SpecialAttributeHandler::XsiNil => {
                    self.process_xsi_nil(value)
                },
                SpecialAttributeHandler::NamespaceDeclaration => {
                    // Already handled in main extraction
                    Ok(None)
                },
                SpecialAttributeHandler::LanguageAndTerritory => {
                    self.process_language_territory(value)
                },
                SpecialAttributeHandler::SequenceNumber => {
                    self.process_sequence_number(value)
                },
                SpecialAttributeHandler::BooleanFlag => {
                    self.process_boolean_flag(value)
                },
            }
        } else {
            Ok(None)
        }
    }

    /// Process xsi:type attribute
    fn process_xsi_type(
        &self,
        value: &AttributeValue,
        namespace_context: &NamespaceContext,
    ) -> Result<Option<SpecialAttributeValue>, ParseError> {
        let type_value = value.to_xml_value();
        
        if let Some((prefix, local_name)) = type_value.split_once(':') {
            // Prefixed type
            let namespace_uri = namespace_context.current_scope.resolve_prefix(prefix);
            Ok(Some(SpecialAttributeValue::XsiType {
                type_name: local_name.to_string(),
                namespace_uri,
                resolved_type: None, // Could be resolved later with schema information
            }))
        } else {
            // Non-prefixed type
            Ok(Some(SpecialAttributeValue::XsiType {
                type_name: type_value,
                namespace_uri: None,
                resolved_type: None,
            }))
        }
    }

    /// Process xsi:schemaLocation attribute
    fn process_schema_location(&self, value: &AttributeValue) -> Result<Option<SpecialAttributeValue>, ParseError> {
        let location_value = value.to_xml_value();
        let mut locations = IndexMap::new();
        
        // Schema locations are space-separated pairs: namespace_uri schema_url
        let tokens: Vec<&str> = location_value.split_whitespace().collect();
        for chunk in tokens.chunks(2) {
            if chunk.len() == 2 {
                locations.insert(chunk[0].to_string(), chunk[1].to_string());
            }
        }
        
        Ok(Some(SpecialAttributeValue::SchemaLocation { locations }))
    }

    /// Process xsi:nil attribute
    fn process_xsi_nil(&self, value: &AttributeValue) -> Result<Option<SpecialAttributeValue>, ParseError> {
        match value {
            AttributeValue::Boolean(b) => Ok(Some(SpecialAttributeValue::Nil(*b))),
            _ => {
                let str_val = value.to_xml_value();
                let nil_val = matches!(str_val.to_lowercase().as_str(), "true" | "1");
                Ok(Some(SpecialAttributeValue::Nil(nil_val)))
            }
        }
    }

    /// Process language and territory codes
    fn process_language_territory(&self, value: &AttributeValue) -> Result<Option<SpecialAttributeValue>, ParseError> {
        let lang_value = value.to_xml_value();
        
        // Parse RFC 5646 language tags (simplified)
        if lang_value.contains('-') {
            let parts: Vec<&str> = lang_value.split('-').collect();
            let language = parts[0].to_string();
            let territory = if parts.len() > 1 {
                Some(parts[1].to_string())
            } else {
                None
            };
            
            Ok(Some(SpecialAttributeValue::Language {
                language,
                script: None, // Could be enhanced to parse script codes
                territory,
            }))
        } else if lang_value.contains(' ') {
            // Space-separated territory codes
            let territories: Vec<String> = lang_value.split_whitespace()
                .map(|s| s.to_string())
                .collect();
            Ok(Some(SpecialAttributeValue::Territory(territories)))
        } else {
            Ok(Some(SpecialAttributeValue::Language {
                language: lang_value,
                script: None,
                territory: None,
            }))
        }
    }

    /// Process sequence number
    fn process_sequence_number(&self, value: &AttributeValue) -> Result<Option<SpecialAttributeValue>, ParseError> {
        match value {
            AttributeValue::Integer(i) => Ok(Some(SpecialAttributeValue::Sequence(*i as u32))),
            _ => {
                if let Ok(seq) = value.to_xml_value().parse::<u32>() {
                    Ok(Some(SpecialAttributeValue::Sequence(seq)))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Process boolean flag
    fn process_boolean_flag(&self, value: &AttributeValue) -> Result<Option<SpecialAttributeValue>, ParseError> {
        match value {
            AttributeValue::Boolean(b) => Ok(Some(SpecialAttributeValue::Flag(*b))),
            _ => {
                let str_val = value.to_xml_value();
                let bool_val = matches!(str_val.to_lowercase().as_str(), "true" | "1");
                Ok(Some(SpecialAttributeValue::Flag(bool_val)))
            }
        }
    }

    /// Apply attribute inheritance from parent to child
    pub fn apply_inheritance(
        &self,
        parent_attributes: &AttributeMap,
        child_attributes: &mut AttributeMap,
    ) {
        let inheritance = ddex_core::models::AttributeInheritance::new();
        inheritance.apply_inheritance(parent_attributes, child_attributes);
    }

    /// Validate extracted attributes
    pub fn validate_attributes(&self, attributes: &AttributeMap) -> Vec<String> {
        let mut errors = Vec::new();
        
        for (qname, value) in attributes.iter() {
            if let Err(e) = value.validate() {
                errors.push(format!("Invalid attribute {}: {}", qname, e));
            }
        }
        
        errors
    }
}

impl Default for AttributeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::Reader;
    use std::io::Cursor;

    #[test]
    fn test_attribute_extraction_basic() {
        let xml = r#"<Release title="Test Album" SequenceNumber="1" IsDefault="true" />"#;
        let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
        let mut buf = Vec::new();
        
        if let Ok(quick_xml::events::Event::Empty(start)) = reader.read_event_into(&mut buf) {
            let extractor = AttributeExtractor::new();
            let namespace_context = NamespaceContext {
                current_scope: ddex_core::namespace::NamespaceScope::new(),
                document_namespaces: indexmap::IndexMap::new(),
                default_namespace: None,
                ern_version: None,
            };
            
            let result = extractor.extract_attributes(&start, &namespace_context).unwrap();
            
            assert_eq!(result.attributes.len(), 3);
            assert_eq!(result.attributes.get_str("title").unwrap().to_xml_value(), "Test Album");
            assert_eq!(result.attributes.get_str("SequenceNumber").unwrap().to_xml_value(), "1");
            assert_eq!(result.attributes.get_str("IsDefault").unwrap().to_xml_value(), "true");
            
            // Check type parsing
            if let Some(AttributeValue::Integer(seq)) = result.attributes.get_str("SequenceNumber") {
                assert_eq!(*seq, 1);
            } else {
                panic!("SequenceNumber should be parsed as integer");
            }
            
            if let Some(AttributeValue::Boolean(is_default)) = result.attributes.get_str("IsDefault") {
                assert_eq!(*is_default, true);
            } else {
                panic!("IsDefault should be parsed as boolean");
            }
        }
    }

    #[test]
    fn test_namespace_attribute_extraction() {
        let xml = r#"<ern:Release xmlns:ern="http://ddex.net/xml/ern/43" 
                                  xmlns:avs="http://ddex.net/xml/avs" 
                                  ern:title="Test" />"#;
        let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
        let mut buf = Vec::new();
        
        if let Ok(quick_xml::events::Event::Empty(start)) = reader.read_event_into(&mut buf) {
            let extractor = AttributeExtractor::new();
            let namespace_context = NamespaceContext {
                current_scope: ddex_core::namespace::NamespaceScope::new(),
                document_namespaces: indexmap::IndexMap::new(),
                default_namespace: None,
                ern_version: None,
            };
            
            let result = extractor.extract_attributes(&start, &namespace_context).unwrap();
            
            assert_eq!(result.namespace_declarations.len(), 2);
            assert!(result.namespace_declarations.contains_key("ern"));
            assert!(result.namespace_declarations.contains_key("avs"));
        }
    }

    #[test]
    fn test_special_attribute_processing() {
        let xml = r#"<element xsi:type="xs:string" 
                              xsi:nil="true"
                              xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                              xmlns:xs="http://www.w3.org/2001/XMLSchema" />"#;
        let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
        let mut buf = Vec::new();
        
        if let Ok(quick_xml::events::Event::Empty(start)) = reader.read_event_into(&mut buf) {
            let extractor = AttributeExtractor::new();
            let namespace_context = NamespaceContext {
                current_scope: ddex_core::namespace::NamespaceScope::new(),
                document_namespaces: indexmap::IndexMap::new(),
                default_namespace: None,
                ern_version: None,
            };
            
            let result = extractor.extract_attributes(&start, &namespace_context).unwrap();
            
            assert!(!result.special_attributes.is_empty());
            
            // Check for xsi:nil
            let xsi_nil_qname = QName::with_prefix_and_namespace(
                "nil".to_string(),
                "xsi".to_string(), 
                "http://www.w3.org/2001/XMLSchema-instance".to_string()
            );
            if let Some(SpecialAttributeValue::Nil(nil_value)) = result.special_attributes.get(&xsi_nil_qname) {
                assert_eq!(*nil_value, true);
            }
        }
    }

    #[test]
    fn test_attribute_inheritance() {
        let mut parent_attrs = AttributeMap::new();
        parent_attrs.insert_str("LanguageAndScriptCode", "en-US");
        parent_attrs.insert_str("ApplicableTerritoryCode", "Worldwide");
        
        let mut child_attrs = AttributeMap::new();
        child_attrs.insert_str("title", "Child Title");
        
        let extractor = AttributeExtractor::new();
        extractor.apply_inheritance(&parent_attrs, &mut child_attrs);
        
        // Child should inherit language and territory
        assert!(child_attrs.get_str("LanguageAndScriptCode").is_some());
        assert!(child_attrs.get_str("ApplicableTerritoryCode").is_some());
        assert!(child_attrs.get_str("title").is_some());
    }

    #[test]
    fn test_ddex_standard_vs_extension_attributes() {
        let mut attributes = AttributeMap::new();
        attributes.insert_str("LanguageAndScriptCode", "en-US"); // Standard
        attributes.insert_str("custom:proprietary", "custom value"); // Extension
        attributes.insert_str("xmlns:custom", "http://example.com/custom"); // Namespace
        
        let standard = attributes.standard_attributes();
        let extensions = attributes.extension_attributes();
        
        assert!(standard.len() >= 1); // Should contain LanguageAndScriptCode
        assert!(extensions.len() >= 1); // Should contain custom:proprietary
    }
}