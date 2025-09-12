//! # XML Attribute Preservation System
//! 
//! This module provides comprehensive support for preserving all XML attributes,
//! including unknown/proprietary ones, with proper namespace handling and
//! deterministic ordering for canonical XML generation.

use indexmap::{IndexMap, IndexSet};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::fmt::{self, Display, Debug};
use std::str::FromStr;
use thiserror::Error;

/// Qualified Name (QName) representing a namespace-qualified XML name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QName {
    /// Local name part
    pub local_name: String,
    /// Namespace URI (empty string for default namespace, None for no namespace)
    pub namespace_uri: Option<String>,
    /// Namespace prefix (empty string for default namespace, None for no prefix)
    pub prefix: Option<String>,
}

impl QName {
    /// Create a new QName with no namespace
    pub fn new(local_name: impl Into<String>) -> Self {
        Self {
            local_name: local_name.into(),
            namespace_uri: None,
            prefix: None,
        }
    }

    /// Create a new QName with namespace URI
    pub fn with_namespace(local_name: impl Into<String>, namespace_uri: impl Into<String>) -> Self {
        Self {
            local_name: local_name.into(),
            namespace_uri: Some(namespace_uri.into()),
            prefix: None,
        }
    }

    /// Create a new QName with prefix and namespace URI
    pub fn with_prefix_and_namespace(
        local_name: impl Into<String>, 
        prefix: impl Into<String>, 
        namespace_uri: impl Into<String>
    ) -> Self {
        Self {
            local_name: local_name.into(),
            namespace_uri: Some(namespace_uri.into()),
            prefix: Some(prefix.into()),
        }
    }

    /// Get the qualified name as it would appear in XML
    pub fn to_xml_name(&self) -> String {
        match &self.prefix {
            Some(prefix) if !prefix.is_empty() => format!("{}:{}", prefix, self.local_name),
            _ => self.local_name.clone(),
        }
    }

    /// Check if this is a namespace declaration attribute
    pub fn is_namespace_declaration(&self) -> bool {
        self.local_name == "xmlns" || 
        (self.prefix.as_deref() == Some("xmlns"))
    }

    /// Check if this is a standard DDEX attribute
    pub fn is_ddex_standard(&self) -> bool {
        match &self.namespace_uri {
            Some(uri) => {
                uri.contains("ddex.net") || 
                uri.contains("w3.org/2001/XMLSchema") ||
                self.is_namespace_declaration()
            },
            None => {
                // Common DDEX attributes without namespace
                matches!(self.local_name.as_str(), 
                    "LanguageAndScriptCode" | "ApplicableTerritoryCode" | 
                    "IsDefault" | "SequenceNumber" | "Namespace"
                )
            }
        }
    }

    /// Get the sorting key for canonical ordering
    pub fn canonical_sort_key(&self) -> String {
        // Namespace declarations come first, then alphabetical by QName
        if self.is_namespace_declaration() {
            if self.local_name == "xmlns" {
                "0:xmlns".to_string()
            } else {
                format!("0:xmlns:{}", self.local_name)
            }
        } else {
            format!("1:{}", self.to_xml_name())
        }
    }
}

impl Display for QName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_xml_name())
    }
}

impl FromStr for QName {
    type Err = AttributeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((prefix, local_name)) = s.split_once(':') {
            Ok(QName {
                local_name: local_name.to_string(),
                namespace_uri: None, // Will be resolved later with namespace context
                prefix: Some(prefix.to_string()),
            })
        } else {
            Ok(QName::new(s))
        }
    }
}

impl PartialOrd for QName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.canonical_sort_key().cmp(&other.canonical_sort_key())
    }
}

/// Typed attribute value supporting various XML Schema types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    /// String value (most common)
    String(String),
    /// Boolean value (true/false)
    Boolean(bool),
    /// Integer value
    Integer(i64),
    /// Decimal value
    Decimal(f64),
    /// Date value (ISO 8601)
    Date(chrono::NaiveDate),
    /// DateTime value (ISO 8601)
    DateTime(chrono::DateTime<chrono::Utc>),
    /// Duration value (ISO 8601)
    Duration(chrono::Duration),
    /// URI/URL value
    Uri(String),
    /// Language code (RFC 5646)
    Language(String),
    /// Token value (normalized string)
    Token(String),
    /// Enumerated value (for known enums)
    Enum(String, Vec<String>), // (value, allowed_values)
    /// Raw string value for unknown types
    Raw(String),
}

impl AttributeValue {
    /// Create a new string attribute value
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Create a new boolean attribute value
    pub fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }

    /// Create a new integer attribute value
    pub fn integer(value: i64) -> Self {
        Self::Integer(value)
    }

    /// Create a new decimal attribute value
    pub fn decimal(value: f64) -> Self {
        Self::Decimal(value)
    }

    /// Create a new URI attribute value
    pub fn uri(value: impl Into<String>) -> Self {
        Self::Uri(value.into())
    }

    /// Create a new raw attribute value
    pub fn raw(value: impl Into<String>) -> Self {
        Self::Raw(value.into())
    }

    /// Get the attribute value as a string for XML serialization
    pub fn to_xml_value(&self) -> String {
        match self {
            AttributeValue::String(s) => s.clone(),
            AttributeValue::Boolean(b) => b.to_string(),
            AttributeValue::Integer(i) => i.to_string(),
            AttributeValue::Decimal(d) => d.to_string(),
            AttributeValue::Date(d) => d.format("%Y-%m-%d").to_string(),
            AttributeValue::DateTime(dt) => dt.to_rfc3339(),
            AttributeValue::Duration(dur) => {
                // Convert to ISO 8601 duration format
                let secs = dur.num_seconds();
                format!("PT{}S", secs)
            },
            AttributeValue::Uri(uri) => uri.clone(),
            AttributeValue::Language(lang) => lang.clone(),
            AttributeValue::Token(token) => token.clone(),
            AttributeValue::Enum(value, _) => value.clone(),
            AttributeValue::Raw(raw) => raw.clone(),
        }
    }

    /// Parse an attribute value from string with type hint
    pub fn parse_with_type(value: &str, type_hint: AttributeType) -> Result<Self, AttributeError> {
        match type_hint {
            AttributeType::String => Ok(AttributeValue::String(value.to_string())),
            AttributeType::Boolean => {
                match value.to_lowercase().as_str() {
                    "true" | "1" => Ok(AttributeValue::Boolean(true)),
                    "false" | "0" => Ok(AttributeValue::Boolean(false)),
                    _ => Err(AttributeError::InvalidBoolean(value.to_string())),
                }
            },
            AttributeType::Integer => {
                value.parse::<i64>()
                    .map(AttributeValue::Integer)
                    .map_err(|_| AttributeError::InvalidInteger(value.to_string()))
            },
            AttributeType::Decimal => {
                value.parse::<f64>()
                    .map(AttributeValue::Decimal)
                    .map_err(|_| AttributeError::InvalidDecimal(value.to_string()))
            },
            AttributeType::Date => {
                chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                    .map(AttributeValue::Date)
                    .map_err(|_| AttributeError::InvalidDate(value.to_string()))
            },
            AttributeType::DateTime => {
                chrono::DateTime::parse_from_rfc3339(value)
                    .map(|dt| AttributeValue::DateTime(dt.with_timezone(&chrono::Utc)))
                    .map_err(|_| AttributeError::InvalidDateTime(value.to_string()))
            },
            AttributeType::Uri => Ok(AttributeValue::Uri(value.to_string())),
            AttributeType::Language => Ok(AttributeValue::Language(value.to_string())),
            AttributeType::Token => Ok(AttributeValue::Token(value.trim().to_string())),
            AttributeType::Raw => Ok(AttributeValue::Raw(value.to_string())),
        }
    }

    /// Validate the attribute value
    pub fn validate(&self) -> Result<(), AttributeError> {
        match self {
            AttributeValue::Enum(value, allowed_values) => {
                if allowed_values.contains(value) {
                    Ok(())
                } else {
                    Err(AttributeError::InvalidEnumValue {
                        value: value.clone(),
                        allowed: allowed_values.clone(),
                    })
                }
            },
            AttributeValue::Uri(uri) => {
                // Basic URI validation
                if uri.contains(' ') || uri.is_empty() {
                    Err(AttributeError::InvalidUri(uri.clone()))
                } else {
                    Ok(())
                }
            },
            AttributeValue::Language(lang) => {
                // Basic language code validation (simplified)
                if lang.len() < 2 || lang.len() > 8 {
                    Err(AttributeError::InvalidLanguage(lang.clone()))
                } else {
                    Ok(())
                }
            },
            _ => Ok(()),
        }
    }
}

impl Display for AttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_xml_value())
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> Self {
        AttributeValue::String(value)
    }
}

impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        AttributeValue::String(value.to_string())
    }
}

impl From<bool> for AttributeValue {
    fn from(value: bool) -> Self {
        AttributeValue::Boolean(value)
    }
}

impl From<i64> for AttributeValue {
    fn from(value: i64) -> Self {
        AttributeValue::Integer(value)
    }
}

impl From<f64> for AttributeValue {
    fn from(value: f64) -> Self {
        AttributeValue::Decimal(value)
    }
}

/// Attribute type hints for parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttributeType {
    String,
    Boolean,
    Integer,
    Decimal,
    Date,
    DateTime,
    Uri,
    Language,
    Token,
    Raw,
}

impl std::fmt::Display for AttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeType::String => write!(f, "string"),
            AttributeType::Boolean => write!(f, "boolean"),
            AttributeType::Integer => write!(f, "integer"),
            AttributeType::Decimal => write!(f, "decimal"),
            AttributeType::Date => write!(f, "date"),
            AttributeType::DateTime => write!(f, "dateTime"),
            AttributeType::Uri => write!(f, "anyURI"),
            AttributeType::Language => write!(f, "language"),
            AttributeType::Token => write!(f, "token"),
            AttributeType::Raw => write!(f, "raw"),
        }
    }
}

/// Comprehensive attribute map with deterministic ordering
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeMap {
    /// Attributes stored with deterministic ordering
    attributes: IndexMap<QName, AttributeValue>,
}

impl AttributeMap {
    /// Create a new empty attribute map
    pub fn new() -> Self {
        Self {
            attributes: IndexMap::new(),
        }
    }

    /// Insert an attribute
    pub fn insert(&mut self, name: QName, value: AttributeValue) -> Option<AttributeValue> {
        self.attributes.insert(name, value)
    }

    /// Insert an attribute by string name
    pub fn insert_str(&mut self, name: &str, value: impl Into<AttributeValue>) -> Option<AttributeValue> {
        let qname = QName::from_str(name).unwrap_or_else(|_| QName::new(name));
        self.insert(qname, value.into())
    }

    /// Get an attribute value
    pub fn get(&self, name: &QName) -> Option<&AttributeValue> {
        self.attributes.get(name)
    }

    /// Get an attribute value by string name
    pub fn get_str(&self, name: &str) -> Option<&AttributeValue> {
        let qname = QName::from_str(name).unwrap_or_else(|_| QName::new(name));
        self.get(&qname)
    }

    /// Remove an attribute
    pub fn remove(&mut self, name: &QName) -> Option<AttributeValue> {
        self.attributes.shift_remove(name)
    }

    /// Check if an attribute exists
    pub fn contains_key(&self, name: &QName) -> bool {
        self.attributes.contains_key(name)
    }

    /// Get all attributes in canonical order
    pub fn iter_canonical(&self) -> impl Iterator<Item = (&QName, &AttributeValue)> {
        let mut sorted: Vec<_> = self.attributes.iter().collect();
        sorted.sort_by(|(a, _), (b, _)| a.cmp(b));
        sorted.into_iter()
    }

    /// Get all attributes (in insertion order)
    pub fn iter(&self) -> impl Iterator<Item = (&QName, &AttributeValue)> {
        self.attributes.iter()
    }

    /// Get mutable iterator over all attributes
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&QName, &mut AttributeValue)> {
        self.attributes.iter_mut()
    }

    /// Get the number of attributes
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    /// Clear all attributes
    pub fn clear(&mut self) {
        self.attributes.clear();
    }

    /// Get all DDEX standard attributes
    pub fn standard_attributes(&self) -> IndexMap<QName, AttributeValue> {
        self.attributes.iter()
            .filter(|(qname, _)| qname.is_ddex_standard())
            .map(|(qname, value)| (qname.clone(), value.clone()))
            .collect()
    }

    /// Get all extension/custom attributes
    pub fn extension_attributes(&self) -> IndexMap<QName, AttributeValue> {
        self.attributes.iter()
            .filter(|(qname, _)| !qname.is_ddex_standard())
            .map(|(qname, value)| (qname.clone(), value.clone()))
            .collect()
    }

    /// Get all namespace declaration attributes
    pub fn namespace_declarations(&self) -> IndexMap<QName, AttributeValue> {
        self.attributes.iter()
            .filter(|(qname, _)| qname.is_namespace_declaration())
            .map(|(qname, value)| (qname.clone(), value.clone()))
            .collect()
    }

    /// Merge attributes from another map, with conflict resolution
    pub fn merge(&mut self, other: &AttributeMap, strategy: AttributeMergeStrategy) {
        for (qname, value) in &other.attributes {
            if let Some(_existing) = self.attributes.get(qname) {
                match strategy {
                    AttributeMergeStrategy::PreferThis => continue,
                    AttributeMergeStrategy::PreferOther => {
                        self.attributes.insert(qname.clone(), value.clone());
                    },
                    AttributeMergeStrategy::Error => {
                        // In a real implementation, this would return a Result
                        eprintln!("Attribute conflict: {}", qname);
                    },
                }
            } else {
                self.attributes.insert(qname.clone(), value.clone());
            }
        }
    }

    /// Validate all attributes
    pub fn validate(&self) -> Vec<AttributeError> {
        let mut errors = Vec::new();
        for (_qname, value) in &self.attributes {
            if let Err(error) = value.validate() {
                errors.push(error);
            }
        }
        errors
    }

    /// Convert to a simple string map for backwards compatibility
    pub fn to_string_map(&self) -> IndexMap<String, String> {
        self.attributes.iter()
            .map(|(qname, value)| (qname.to_xml_name(), value.to_xml_value()))
            .collect()
    }

    /// Create from a simple string map
    pub fn from_string_map(map: IndexMap<String, String>) -> Self {
        let mut attributes = IndexMap::new();
        for (name, value) in map {
            let qname = QName::from_str(&name).unwrap_or_else(|_| QName::new(name));
            attributes.insert(qname, AttributeValue::String(value));
        }
        Self { attributes }
    }
    
    /// Get iterator over attribute keys
    pub fn keys(&self) -> indexmap::map::Keys<'_, QName, AttributeValue> {
        self.attributes.keys()
    }
    
    /// Get all attributes in canonical order (namespace declarations first, then alphabetical)
    pub fn to_canonical_ordered(&self) -> IndexMap<QName, AttributeValue> {
        let mut namespace_attrs = IndexMap::new();
        let mut regular_attrs = IndexMap::new();
        
        // Separate namespace declarations from regular attributes
        for (qname, value) in &self.attributes {
            if qname.is_namespace_declaration() {
                namespace_attrs.insert(qname.clone(), value.clone());
            } else {
                regular_attrs.insert(qname.clone(), value.clone());
            }
        }
        
        // Sort both collections by canonical sort key
        namespace_attrs.sort_by(|a, _, b, _| a.canonical_sort_key().cmp(&b.canonical_sort_key()));
        regular_attrs.sort_by(|a, _, b, _| a.canonical_sort_key().cmp(&b.canonical_sort_key()));
        
        // Combine namespace declarations first, then regular attributes
        let mut result = IndexMap::new();
        result.extend(namespace_attrs);
        result.extend(regular_attrs);
        
        result
    }
    
}

impl Default for AttributeMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for AttributeMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as a map of string -> string for JSON compatibility
        let string_map = self.to_string_map();
        string_map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AttributeMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string_map = IndexMap::<String, String>::deserialize(deserializer)?;
        Ok(Self::from_string_map(string_map))
    }
}

impl<'a> IntoIterator for &'a AttributeMap {
    type Item = (&'a QName, &'a AttributeValue);
    type IntoIter = indexmap::map::Iter<'a, QName, AttributeValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.attributes.iter()
    }
}

/// Strategy for merging attribute maps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeMergeStrategy {
    /// Keep the existing attribute (prefer this map)
    PreferThis,
    /// Use the new attribute (prefer other map)
    PreferOther,
    /// Raise an error on conflicts
    Error,
}

/// Attribute inheritance model for nested elements
#[derive(Debug, Clone)]
pub struct AttributeInheritance {
    /// Attributes that should be inherited from parent elements
    inheritable_attributes: IndexSet<QName>,
    /// Attributes that should never be inherited
    non_inheritable_attributes: IndexSet<QName>,
}

impl AttributeInheritance {
    /// Create new attribute inheritance rules
    pub fn new() -> Self {
        let mut inheritable = IndexSet::new();
        let mut non_inheritable = IndexSet::new();

        // Common inheritable attributes
        inheritable.insert(QName::new("LanguageAndScriptCode"));
        inheritable.insert(QName::new("ApplicableTerritoryCode"));
        inheritable.insert(QName::with_namespace("lang", "http://www.w3.org/XML/1998/namespace"));

        // Non-inheritable attributes (element-specific)
        non_inheritable.insert(QName::new("SequenceNumber"));
        non_inheritable.insert(QName::with_prefix_and_namespace("xsi", "type", "http://www.w3.org/2001/XMLSchema-instance"));

        Self {
            inheritable_attributes: inheritable,
            non_inheritable_attributes: non_inheritable,
        }
    }

    /// Check if an attribute should be inherited
    pub fn should_inherit(&self, qname: &QName) -> bool {
        if self.non_inheritable_attributes.contains(qname) {
            false
        } else if self.inheritable_attributes.contains(qname) {
            true
        } else {
            // Default: don't inherit unknown attributes
            false
        }
    }

    /// Apply inheritance from parent to child attributes
    pub fn apply_inheritance(&self, parent: &AttributeMap, child: &mut AttributeMap) {
        for (qname, value) in parent.iter() {
            if self.should_inherit(qname) && !child.contains_key(qname) {
                child.insert(qname.clone(), value.clone());
            }
        }
    }
}

impl Default for AttributeInheritance {
    fn default() -> Self {
        Self::new()
    }
}

/// Attribute-related errors
#[derive(Debug, Clone, Error, PartialEq)]
pub enum AttributeError {
    #[error("Invalid boolean value: {0}")]
    InvalidBoolean(String),
    
    #[error("Invalid integer value: {0}")]
    InvalidInteger(String),
    
    #[error("Invalid decimal value: {0}")]
    InvalidDecimal(String),
    
    #[error("Invalid date value: {0}")]
    InvalidDate(String),
    
    #[error("Invalid datetime value: {0}")]
    InvalidDateTime(String),
    
    #[error("Invalid URI value: {0}")]
    InvalidUri(String),
    
    #[error("Invalid language code: {0}")]
    InvalidLanguage(String),
    
    #[error("Invalid enum value '{value}', allowed values: {}", allowed.join(", "))]
    InvalidEnumValue {
        value: String,
        allowed: Vec<String>,
    },
    
    #[error("Missing required attribute: {0}")]
    MissingRequired(String),
    
    #[error("Conflicting attribute values for: {0}")]
    ConflictingValues(String),
    
    #[error("Invalid QName format: {0}")]
    InvalidQName(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qname_creation() {
        let qname = QName::new("title");
        assert_eq!(qname.local_name, "title");
        assert_eq!(qname.namespace_uri, None);
        assert_eq!(qname.prefix, None);

        let qname_ns = QName::with_namespace("title", "http://ddex.net/xml/ern/43");
        assert_eq!(qname_ns.namespace_uri, Some("http://ddex.net/xml/ern/43".to_string()));

        let qname_prefix = QName::with_prefix_and_namespace("title", "ern", "http://ddex.net/xml/ern/43");
        assert_eq!(qname_prefix.prefix, Some("ern".to_string()));
        assert_eq!(qname_prefix.to_xml_name(), "ern:title");
    }

    #[test]
    fn test_qname_parsing() {
        let qname: QName = "ern:title".parse().unwrap();
        assert_eq!(qname.local_name, "title");
        assert_eq!(qname.prefix, Some("ern".to_string()));

        let simple_qname: QName = "title".parse().unwrap();
        assert_eq!(simple_qname.local_name, "title");
        assert_eq!(simple_qname.prefix, None);
    }

    #[test]
    fn test_qname_canonical_ordering() {
        let xmlns = QName::new("xmlns");
        let xmlns_ern = QName::from_str("xmlns:ern").unwrap();
        let regular = QName::new("title");
        let prefixed = QName::from_str("ern:title").unwrap();

        let mut qnames = vec![&regular, &prefixed, &xmlns_ern, &xmlns];
        qnames.sort();

        // Namespace declarations should come first
        assert_eq!(qnames[0], &xmlns);
        assert_eq!(qnames[1], &xmlns_ern);
    }

    #[test]
    fn test_attribute_value_types() {
        let string_val = AttributeValue::string("test");
        assert_eq!(string_val.to_xml_value(), "test");

        let bool_val = AttributeValue::boolean(true);
        assert_eq!(bool_val.to_xml_value(), "true");

        let int_val = AttributeValue::integer(42);
        assert_eq!(int_val.to_xml_value(), "42");

        // Test parsing with type hints
        let parsed = AttributeValue::parse_with_type("true", AttributeType::Boolean).unwrap();
        assert_eq!(parsed, AttributeValue::Boolean(true));

        let parsed_int = AttributeValue::parse_with_type("123", AttributeType::Integer).unwrap();
        assert_eq!(parsed_int, AttributeValue::Integer(123));
    }

    #[test]
    fn test_attribute_map() {
        let mut map = AttributeMap::new();
        
        map.insert_str("title", "Test Title");
        map.insert_str("ern:version", "4.3");
        map.insert_str("xmlns:ern", "http://ddex.net/xml/ern/43");

        assert_eq!(map.len(), 3);
        assert_eq!(map.get_str("title").unwrap().to_xml_value(), "Test Title");

        // Test canonical ordering
        let canonical: Vec<_> = map.iter_canonical().collect();
        assert_eq!(canonical.len(), 3);
        
        // xmlns attributes should come first
        let first_attr = &canonical[0];
        assert!(first_attr.0.is_namespace_declaration());
    }

    #[test]
    fn test_attribute_inheritance() {
        let inheritance = AttributeInheritance::new();
        
        let lang_attr = QName::new("LanguageAndScriptCode");
        let seq_attr = QName::new("SequenceNumber");
        
        assert!(inheritance.should_inherit(&lang_attr));
        assert!(!inheritance.should_inherit(&seq_attr));
    }

    #[test]
    fn test_attribute_validation() {
        let mut enum_val = AttributeValue::Enum(
            "invalid".to_string(), 
            vec!["valid1".to_string(), "valid2".to_string()]
        );
        assert!(enum_val.validate().is_err());

        enum_val = AttributeValue::Enum(
            "valid1".to_string(), 
            vec!["valid1".to_string(), "valid2".to_string()]
        );
        assert!(enum_val.validate().is_ok());
    }

    #[test]
    fn test_ddex_standard_detection() {
        let ddex_attr = QName::with_namespace("title", "http://ddex.net/xml/ern/43");
        assert!(ddex_attr.is_ddex_standard());

        let xmlns_attr = QName::new("xmlns");
        assert!(xmlns_attr.is_ddex_standard());

        let custom_attr = QName::with_namespace("custom", "http://example.com/custom");
        assert!(!custom_attr.is_ddex_standard());

        let lang_attr = QName::new("LanguageAndScriptCode");
        assert!(lang_attr.is_ddex_standard());
    }

    #[test]
    fn test_attribute_map_serialization() {
        let mut map = AttributeMap::new();
        map.insert_str("title", "Test Title");
        map.insert_str("version", "4.3");

        // Test conversion to string map
        let string_map = map.to_string_map();
        assert_eq!(string_map.len(), 2);
        assert_eq!(string_map.get("title"), Some(&"Test Title".to_string()));

        // Test round-trip through string map
        let restored = AttributeMap::from_string_map(string_map);
        assert_eq!(restored.len(), 2);
        assert_eq!(restored.get_str("title").unwrap().to_xml_value(), "Test Title");
    }
}