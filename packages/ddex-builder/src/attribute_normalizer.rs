//! # XML Attribute Normalization and Canonical Ordering
//! 
//! This module provides comprehensive attribute normalization for XML generation,
//! including canonical ordering, value escaping, namespace prefix application,
//! and special attribute handling for deterministic XML output.

use ddex_core::models::{AttributeMap, AttributeValue, QName};
use ddex_core::namespace::NamespaceRegistry;
use indexmap::IndexMap;
use std::collections::HashMap;
use html_escape;
use tracing::{debug, warn};

/// Attribute normalization configuration
#[derive(Debug, Clone)]
pub struct AttributeNormalizationConfig {
    /// Apply canonical attribute ordering (alphabetical by QName)
    pub canonical_ordering: bool,
    /// Normalize attribute values (escape, trim, etc.)
    pub normalize_values: bool,
    /// Apply namespace prefixes to qualified names
    pub apply_namespace_prefixes: bool,
    /// Handle special attributes (xsi:*, xmlns:*)
    pub handle_special_attributes: bool,
    /// Remove duplicate attributes
    pub remove_duplicates: bool,
    /// Validate attribute values
    pub validate_values: bool,
}

impl Default for AttributeNormalizationConfig {
    fn default() -> Self {
        Self {
            canonical_ordering: true,
            normalize_values: true,
            apply_namespace_prefixes: true,
            handle_special_attributes: true,
            remove_duplicates: true,
            validate_values: true,
        }
    }
}

/// Canonical attribute normalizer
#[derive(Debug, Clone)]
pub struct AttributeNormalizer {
    /// Configuration for normalization
    config: AttributeNormalizationConfig,
    /// Namespace registry for prefix resolution
    namespace_registry: NamespaceRegistry,
    /// Special attribute handlers
    special_handlers: HashMap<String, SpecialAttributeNormalizer>,
}

/// Special attribute normalization handlers
#[derive(Debug, Clone)]
pub enum SpecialAttributeNormalizer {
    /// Namespace declarations (xmlns, xmlns:*)
    NamespaceDeclaration,
    /// XML Schema Instance attributes (xsi:*)
    XmlSchemaInstance,
    /// Language and territory codes
    LanguageTerritory,
    /// Boolean attributes
    BooleanAttribute,
    /// Numeric attributes
    NumericAttribute,
    /// URI attributes
    UriAttribute,
    /// Date/time attributes
    DateTimeAttribute,
}

/// Normalized attribute output
#[derive(Debug, Clone)]
pub struct NormalizedAttributes {
    /// Attributes in canonical order ready for XML output
    pub ordered_attributes: IndexMap<String, String>,
    /// Namespace declarations (separate for hoisting)
    pub namespace_declarations: IndexMap<String, String>,
    /// Warnings during normalization
    pub warnings: Vec<String>,
    /// Validation errors
    pub validation_errors: Vec<String>,
}

impl AttributeNormalizer {
    /// Create new attribute normalizer with default configuration
    pub fn new() -> Self {
        Self::with_config(AttributeNormalizationConfig::default())
    }

    /// Create new attribute normalizer with custom configuration
    pub fn with_config(config: AttributeNormalizationConfig) -> Self {
        let mut normalizer = Self {
            config,
            namespace_registry: NamespaceRegistry::new(),
            special_handlers: HashMap::new(),
        };
        
        normalizer.initialize_special_handlers();
        normalizer
    }

    /// Initialize special attribute handlers
    fn initialize_special_handlers(&mut self) {
        // Namespace declarations
        self.special_handlers.insert("xmlns".to_string(), SpecialAttributeNormalizer::NamespaceDeclaration);
        
        // XML Schema Instance attributes
        self.special_handlers.insert("xsi:type".to_string(), SpecialAttributeNormalizer::XmlSchemaInstance);
        self.special_handlers.insert("xsi:schemaLocation".to_string(), SpecialAttributeNormalizer::XmlSchemaInstance);
        self.special_handlers.insert("xsi:noNamespaceSchemaLocation".to_string(), SpecialAttributeNormalizer::XmlSchemaInstance);
        self.special_handlers.insert("xsi:nil".to_string(), SpecialAttributeNormalizer::XmlSchemaInstance);
        
        // DDEX specific attributes
        self.special_handlers.insert("LanguageAndScriptCode".to_string(), SpecialAttributeNormalizer::LanguageTerritory);
        self.special_handlers.insert("ApplicableTerritoryCode".to_string(), SpecialAttributeNormalizer::LanguageTerritory);
        
        // Boolean attributes
        self.special_handlers.insert("IsDefault".to_string(), SpecialAttributeNormalizer::BooleanAttribute);
        self.special_handlers.insert("IsMainArtist".to_string(), SpecialAttributeNormalizer::BooleanAttribute);
        self.special_handlers.insert("HasChanged".to_string(), SpecialAttributeNormalizer::BooleanAttribute);
        
        // Numeric attributes
        self.special_handlers.insert("SequenceNumber".to_string(), SpecialAttributeNormalizer::NumericAttribute);
        
        // URI attributes
        self.special_handlers.insert("Namespace".to_string(), SpecialAttributeNormalizer::UriAttribute);
        
        // Date/time attributes
        self.special_handlers.insert("CreatedDateTime".to_string(), SpecialAttributeNormalizer::DateTimeAttribute);
        self.special_handlers.insert("UpdatedDateTime".to_string(), SpecialAttributeNormalizer::DateTimeAttribute);
    }

    /// Normalize attributes for canonical XML output
    pub fn normalize_attributes(
        &self,
        attributes: &AttributeMap,
        element_namespace_context: &IndexMap<String, String>, // prefix -> uri mappings
    ) -> NormalizedAttributes {
        debug!("Normalizing {} attributes", attributes.len());
        
        let mut ordered_attributes = IndexMap::new();
        let mut namespace_declarations = IndexMap::new();
        let mut warnings = Vec::new();
        let mut validation_errors = Vec::new();

        // Step 1: Validate attributes if configured
        if self.config.validate_values {
            let errors = attributes.validate();
            for error in errors {
                validation_errors.push(error.to_string());
            }
        }

        // Step 2: Process attributes and separate namespace declarations
        let mut processed_attributes = IndexMap::new();
        
        for (qname, value) in attributes.iter_canonical() {
            debug!("Processing attribute: {} = {}", qname, value);
            
            // Handle namespace declarations separately
            if qname.is_namespace_declaration() {
                let prefix = if qname.local_name == "xmlns" {
                    "".to_string() // Default namespace
                } else {
                    qname.local_name.clone()
                };
                namespace_declarations.insert(prefix, value.to_xml_value());
                continue;
            }

            // Normalize the attribute
            let (normalized_name, normalized_value) = self.normalize_single_attribute(
                qname, 
                value, 
                element_namespace_context, 
                &mut warnings
            );
            
            // Check for duplicates if configured
            if self.config.remove_duplicates && processed_attributes.contains_key(&normalized_name) {
                warnings.push(format!("Duplicate attribute removed: {}", normalized_name));
                continue;
            }
            
            processed_attributes.insert(normalized_name, normalized_value);
        }

        // Step 3: Apply canonical ordering if configured
        if self.config.canonical_ordering {
            // Sort by attribute name (already in canonical order from iter_canonical)
            ordered_attributes = processed_attributes;
        } else {
            // Preserve original order
            ordered_attributes = processed_attributes;
        }

        // Step 4: Sort namespace declarations
        let mut sorted_namespace_declarations: Vec<_> = namespace_declarations.into_iter().collect();
        sorted_namespace_declarations.sort_by(|(a, _), (b, _)| {
            // xmlns comes first, then xmlns:prefix in alphabetical order
            match (a.is_empty(), b.is_empty()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.cmp(b),
            }
        });
        
        let namespace_declarations: IndexMap<String, String> = sorted_namespace_declarations.into_iter().collect();

        debug!("Normalized {} attributes, {} namespace declarations", 
               ordered_attributes.len(), namespace_declarations.len());

        NormalizedAttributes {
            ordered_attributes,
            namespace_declarations,
            warnings,
            validation_errors,
        }
    }

    /// Normalize a single attribute
    fn normalize_single_attribute(
        &self,
        qname: &QName,
        value: &AttributeValue,
        namespace_context: &IndexMap<String, String>,
        warnings: &mut Vec<String>,
    ) -> (String, String) {
        // Step 1: Determine the output name
        let output_name = self.resolve_attribute_name(qname, namespace_context, warnings);
        
        // Step 2: Normalize the value
        let normalized_value = if self.config.normalize_values {
            self.normalize_attribute_value(qname, value, warnings)
        } else {
            value.to_xml_value()
        };

        (output_name, normalized_value)
    }

    /// Resolve attribute name with proper namespace prefix
    fn resolve_attribute_name(
        &self,
        qname: &QName,
        namespace_context: &IndexMap<String, String>,
        warnings: &mut Vec<String>,
    ) -> String {
        if !self.config.apply_namespace_prefixes {
            return qname.local_name.clone();
        }

        // If the attribute has a namespace URI, try to find the appropriate prefix
        if let Some(namespace_uri) = &qname.namespace_uri {
            // First check if we have a prefix from parsing
            if let Some(ref prefix) = qname.prefix {
                // Verify the prefix is valid in the current context
                if let Some(context_uri) = namespace_context.get(prefix) {
                    if context_uri == namespace_uri {
                        return format!("{}:{}", prefix, qname.local_name);
                    } else {
                        warnings.push(format!(
                            "Prefix '{}' maps to different namespace in context", 
                            prefix
                        ));
                    }
                }
            }
            
            // Try to find a matching prefix in the context
            for (prefix, uri) in namespace_context {
                if uri == namespace_uri {
                    if prefix.is_empty() {
                        // Default namespace - don't prefix attributes
                        return qname.local_name.clone();
                    } else {
                        return format!("{}:{}", prefix, qname.local_name);
                    }
                }
            }
            
            // Try to get a preferred prefix from the registry
            if let Some(preferred_prefix) = self.namespace_registry.get_preferred_prefix(namespace_uri) {
                warnings.push(format!(
                    "Using preferred prefix '{}' for namespace '{}'", 
                    preferred_prefix, namespace_uri
                ));
                return format!("{}:{}", preferred_prefix, qname.local_name);
            }
            
            warnings.push(format!("Could not resolve namespace for attribute: {}", qname));
        }

        // Fallback to local name
        qname.local_name.clone()
    }

    /// Normalize attribute value based on type and special handling
    fn normalize_attribute_value(
        &self,
        qname: &QName,
        value: &AttributeValue,
        warnings: &mut Vec<String>,
    ) -> String {
        let attr_name = qname.to_xml_name();
        
        // Check for special attribute handling
        if let Some(handler) = self.special_handlers.get(&attr_name) {
            return self.apply_special_normalization(handler, value, warnings);
        }
        
        // Check by local name if no full match
        if let Some(handler) = self.special_handlers.get(&qname.local_name) {
            return self.apply_special_normalization(handler, value, warnings);
        }

        // Apply general normalization
        self.apply_general_normalization(value)
    }

    /// Apply special normalization for specific attribute types
    fn apply_special_normalization(
        &self,
        handler: &SpecialAttributeNormalizer,
        value: &AttributeValue,
        warnings: &mut Vec<String>,
    ) -> String {
        match handler {
            SpecialAttributeNormalizer::NamespaceDeclaration => {
                // Namespace URIs should not be escaped
                value.to_xml_value()
            },
            SpecialAttributeNormalizer::XmlSchemaInstance => {
                // XSI attributes need special handling
                match value {
                    AttributeValue::Boolean(b) => b.to_string().to_lowercase(),
                    _ => value.to_xml_value(),
                }
            },
            SpecialAttributeNormalizer::LanguageTerritory => {
                // Normalize language/territory codes
                let lang_value = value.to_xml_value();
                // Could add validation for RFC 5646 compliance
                lang_value
            },
            SpecialAttributeNormalizer::BooleanAttribute => {
                // Ensure consistent boolean representation
                match value {
                    AttributeValue::Boolean(b) => b.to_string().to_lowercase(),
                    _ => {
                        let str_val = value.to_xml_value();
                        match str_val.to_lowercase().as_str() {
                            "true" | "1" => "true".to_string(),
                            "false" | "0" => "false".to_string(),
                            _ => {
                                warnings.push(format!("Invalid boolean value: {}", str_val));
                                str_val
                            }
                        }
                    }
                }
            },
            SpecialAttributeNormalizer::NumericAttribute => {
                // Normalize numeric representation
                match value {
                    AttributeValue::Integer(i) => i.to_string(),
                    AttributeValue::Decimal(d) => {
                        // Remove unnecessary decimal places
                        if d.fract() == 0.0 {
                            format!("{:.0}", d)
                        } else {
                            d.to_string()
                        }
                    },
                    _ => value.to_xml_value(),
                }
            },
            SpecialAttributeNormalizer::UriAttribute => {
                // Normalize URI representation
                let uri = value.to_xml_value();
                // Could add URI validation and normalization
                uri
            },
            SpecialAttributeNormalizer::DateTimeAttribute => {
                // Normalize date/time representation
                match value {
                    AttributeValue::DateTime(dt) => dt.to_rfc3339(),
                    AttributeValue::Date(d) => d.format("%Y-%m-%d").to_string(),
                    _ => value.to_xml_value(),
                }
            },
        }
    }

    /// Apply general attribute value normalization
    fn apply_general_normalization(&self, value: &AttributeValue) -> String {
        let raw_value = value.to_xml_value();
        
        // Trim whitespace
        let trimmed = raw_value.trim();
        
        // Apply XML attribute escaping
        html_escape::encode_double_quoted_attribute(trimmed).to_string()
    }

    /// Generate XML attribute string from normalized attributes
    pub fn generate_attribute_string(&self, normalized: &NormalizedAttributes) -> String {
        let mut parts = Vec::new();
        
        // Add namespace declarations first
        for (prefix, uri) in &normalized.namespace_declarations {
            let attr_name = if prefix.is_empty() {
                "xmlns".to_string()
            } else {
                format!("xmlns:{}", prefix)
            };
            parts.push(format!("{}=\"{}\"", attr_name, html_escape::encode_double_quoted_attribute(uri)));
        }
        
        // Add other attributes
        for (name, value) in &normalized.ordered_attributes {
            parts.push(format!("{}=\"{}\"", name, value));
        }
        
        if parts.is_empty() {
            String::new()
        } else {
            format!(" {}", parts.join(" "))
        }
    }

    /// Merge multiple attribute maps with conflict resolution
    pub fn merge_attribute_maps(
        &self,
        base_attributes: &AttributeMap,
        override_attributes: &AttributeMap,
    ) -> AttributeMap {
        let mut merged = base_attributes.clone();
        merged.merge(override_attributes, ddex_core::models::attributes::AttributeMergeStrategy::PreferOther);
        merged
    }

    /// Validate that required attributes are present
    pub fn validate_required_attributes(
        &self,
        attributes: &AttributeMap,
        required_attrs: &[&str],
    ) -> Vec<String> {
        let mut errors = Vec::new();
        
        for required in required_attrs {
            if !attributes.iter().any(|(qname, _)| qname.local_name == *required || qname.to_xml_name() == *required) {
                errors.push(format!("Missing required attribute: {}", required));
            }
        }
        
        errors
    }
}

impl Default for AttributeNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Attribute inheritance processor
pub struct AttributeInheritanceProcessor {
    /// Rules for attribute inheritance
    inheritance_rules: ddex_core::models::AttributeInheritance,
}

impl AttributeInheritanceProcessor {
    pub fn new() -> Self {
        Self {
            inheritance_rules: ddex_core::models::AttributeInheritance::new(),
        }
    }

    /// Process attribute inheritance from parent to child
    pub fn process_inheritance(
        &self,
        parent_attributes: &AttributeMap,
        child_attributes: &mut AttributeMap,
    ) {
        self.inheritance_rules.apply_inheritance(parent_attributes, child_attributes);
    }

    /// Get inherited attributes for an element
    pub fn get_inherited_attributes(
        &self,
        parent_attributes: &AttributeMap,
        child_attributes: &AttributeMap,
    ) -> AttributeMap {
        let mut result = child_attributes.clone();
        self.inheritance_rules.apply_inheritance(parent_attributes, &mut result);
        result
    }
}

impl Default for AttributeInheritanceProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ddex_core::models::attributes::{QName, AttributeValue, AttributeMap};

    #[test]
    fn test_basic_attribute_normalization() {
        let normalizer = AttributeNormalizer::new();
        let mut attributes = AttributeMap::new();
        
        attributes.insert(QName::new("title"), AttributeValue::string("Test Album"));
        attributes.insert(QName::new("IsDefault"), AttributeValue::boolean(true));
        attributes.insert(QName::new("SequenceNumber"), AttributeValue::integer(42));
        
        let context = IndexMap::new();
        let result = normalizer.normalize_attributes(&attributes, &context);
        
        assert_eq!(result.ordered_attributes.len(), 3);
        assert_eq!(result.ordered_attributes.get("title"), Some(&"Test Album".to_string()));
        assert_eq!(result.ordered_attributes.get("IsDefault"), Some(&"true".to_string()));
        assert_eq!(result.ordered_attributes.get("SequenceNumber"), Some(&"42".to_string()));
    }

    #[test]
    fn test_namespace_attribute_separation() {
        let normalizer = AttributeNormalizer::new();
        let mut attributes = AttributeMap::new();
        
        attributes.insert(QName::new("xmlns"), AttributeValue::string("http://ddex.net/xml/ern/43"));
        attributes.insert(QName::from_str("xmlns:avs").unwrap(), AttributeValue::string("http://ddex.net/xml/avs"));
        attributes.insert(QName::new("title"), AttributeValue::string("Test"));
        
        let context = IndexMap::new();
        let result = normalizer.normalize_attributes(&attributes, &context);
        
        assert_eq!(result.namespace_declarations.len(), 2);
        assert_eq!(result.ordered_attributes.len(), 1);
        assert!(result.namespace_declarations.contains_key(""));
        assert!(result.namespace_declarations.contains_key("avs"));
    }

    #[test]
    fn test_special_attribute_normalization() {
        let normalizer = AttributeNormalizer::new();
        let mut attributes = AttributeMap::new();
        
        // Boolean normalization
        attributes.insert(QName::new("IsDefault"), AttributeValue::string("1"));
        
        // Numeric normalization
        attributes.insert(QName::new("SequenceNumber"), AttributeValue::string("007"));
        
        let context = IndexMap::new();
        let result = normalizer.normalize_attributes(&attributes, &context);
        
        assert_eq!(result.ordered_attributes.get("IsDefault"), Some(&"true".to_string()));
        assert_eq!(result.ordered_attributes.get("SequenceNumber"), Some(&"007".to_string()));
    }

    #[test]
    fn test_canonical_ordering() {
        let normalizer = AttributeNormalizer::new();
        let mut attributes = AttributeMap::new();
        
        // Add attributes in non-alphabetical order
        attributes.insert(QName::new("zebra"), AttributeValue::string("z"));
        attributes.insert(QName::new("alpha"), AttributeValue::string("a"));
        attributes.insert(QName::new("beta"), AttributeValue::string("b"));
        
        let context = IndexMap::new();
        let result = normalizer.normalize_attributes(&attributes, &context);
        
        let keys: Vec<_> = result.ordered_attributes.keys().collect();
        // Should be in alphabetical order due to canonical ordering
        assert_eq!(keys, vec!["alpha", "beta", "zebra"]);
    }

    #[test]
    fn test_xml_attribute_string_generation() {
        let normalizer = AttributeNormalizer::new();
        let mut attributes = AttributeMap::new();
        
        attributes.insert(QName::new("title"), AttributeValue::string("Test & Demo"));
        attributes.insert(QName::new("xmlns"), AttributeValue::string("http://ddex.net/xml/ern/43"));
        
        let context = IndexMap::new();
        let result = normalizer.normalize_attributes(&attributes, &context);
        let attr_string = normalizer.generate_attribute_string(&result);
        
        // Should include namespace declarations first, then other attributes
        // Should properly escape the ampersand
        assert!(attr_string.contains("xmlns="));
        assert!(attr_string.contains("title="));
        assert!(attr_string.contains("Test &amp; Demo"));
    }

    #[test]
    fn test_attribute_inheritance() {
        let processor = AttributeInheritanceProcessor::new();
        
        let mut parent_attrs = AttributeMap::new();
        parent_attrs.insert(QName::new("LanguageAndScriptCode"), AttributeValue::string("en-US"));
        parent_attrs.insert(QName::new("ApplicableTerritoryCode"), AttributeValue::string("Worldwide"));
        
        let mut child_attrs = AttributeMap::new();
        child_attrs.insert(QName::new("title"), AttributeValue::string("Child Title"));
        
        processor.process_inheritance(&parent_attrs, &mut child_attrs);
        
        // Child should inherit language and territory
        assert!(child_attrs.get_str("LanguageAndScriptCode").is_some());
        assert!(child_attrs.get_str("ApplicableTerritoryCode").is_some());
        assert!(child_attrs.get_str("title").is_some());
    }

    #[test]
    fn test_required_attribute_validation() {
        let normalizer = AttributeNormalizer::new();
        let mut attributes = AttributeMap::new();
        
        attributes.insert(QName::new("title"), AttributeValue::string("Test"));
        // Missing "id" attribute
        
        let required = vec!["title", "id"];
        let errors = normalizer.validate_required_attributes(&attributes, &required);
        
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("id"));
    }
}