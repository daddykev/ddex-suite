//! # Comprehensive Attribute Validation System
//! 
//! This module provides validation for XML attributes including:
//! - Required attribute validation
//! - Format validation (URIs, dates, enums, etc.)
//! - Custom attribute policy enforcement
//! - DDEX-specific validation rules
//! - Cross-attribute validation
//! - Namespace-aware validation

use crate::models::{AttributeMap, AttributeValue, QName, AttributeType};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use chrono::{DateTime, Utc, NaiveDate};
use url::Url;
use regex::Regex;

/// Comprehensive validation errors for attributes
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeValidationError {
    #[error("Missing required attribute: {attribute}")]
    MissingRequired { attribute: QName },
    
    #[error("Invalid format for attribute {attribute}: {reason}")]
    InvalidFormat { attribute: QName, reason: String },
    
    #[error("Invalid enum value for attribute {attribute}: '{value}', expected one of: {allowed_values:?}")]
    InvalidEnumValue { 
        attribute: QName, 
        value: String, 
        allowed_values: Vec<String> 
    },
    
    #[error("Value out of range for attribute {attribute}: {value}, expected {range}")]
    ValueOutOfRange { 
        attribute: QName, 
        value: String, 
        range: String 
    },
    
    #[error("Invalid type for attribute {attribute}: expected {expected}, got {actual}")]
    InvalidType { 
        attribute: QName, 
        expected: AttributeType, 
        actual: AttributeType 
    },
    
    #[error("Custom validation failed for attribute {attribute}: {rule}")]
    CustomValidationFailed { 
        attribute: QName, 
        rule: String 
    },
    
    #[error("Cross-attribute validation failed: {message}")]
    CrossAttributeValidationFailed { message: String },
    
    #[error("Namespace validation failed for attribute {attribute}: {reason}")]
    NamespaceValidationFailed { 
        attribute: QName, 
        reason: String 
    },
    
    #[error("Policy violation for attribute {attribute}: {policy}")]
    PolicyViolation { 
        attribute: QName, 
        policy: String 
    },
}

/// Validation result containing errors and warnings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationResult {
    pub errors: Vec<AttributeValidationError>,
    pub warnings: Vec<String>,
    pub is_valid: bool,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            is_valid: true,
        }
    }
    
    pub fn add_error(&mut self, error: AttributeValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        if !other.is_valid {
            self.is_valid = false;
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Attribute validation rule definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationRule {
    /// Attribute is required
    Required,
    /// Must match specific type
    Type(AttributeType),
    /// Must be one of specified enum values
    Enum(Vec<String>),
    /// Must match regular expression
    Regex(String),
    /// Must be valid URI
    Uri,
    /// Must be valid email address
    Email,
    /// Must be valid ISO date (YYYY-MM-DD)
    Date,
    /// Must be valid ISO datetime
    DateTime,
    /// Numeric range validation (using i64 for Eq compliance)
    Range { min: Option<i64>, max: Option<i64> },
    /// String length validation
    Length { min: Option<usize>, max: Option<usize> },
    /// Custom validation function (by name)
    Custom(String),
    /// Cross-attribute dependency
    Dependency { 
        depends_on: QName, 
        condition: DependencyCondition 
    },
}

/// Dependency condition for cross-attribute validation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyCondition {
    /// Required when dependent attribute exists
    RequiredWhenExists,
    /// Required when dependent attribute has specific value
    RequiredWhenEquals(String),
    /// Forbidden when dependent attribute exists
    ForbiddenWhenExists,
    /// Must have same value as dependent attribute
    MustMatch,
    /// Must have different value from dependent attribute
    MustDiffer,
}

/// Validation policy for controlling attribute validation behavior
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationPolicy {
    /// Whether to allow unknown attributes
    pub allow_unknown_attributes: bool,
    /// Whether to treat warnings as errors
    pub strict_mode: bool,
    /// Whether to validate namespace declarations
    pub validate_namespaces: bool,
    /// Custom policies by namespace
    pub namespace_policies: IndexMap<String, NamespacePolicy>,
    /// Element-specific validation overrides
    pub element_overrides: IndexMap<QName, ElementValidationConfig>,
}

impl Default for ValidationPolicy {
    fn default() -> Self {
        Self {
            allow_unknown_attributes: true,
            strict_mode: false,
            validate_namespaces: true,
            namespace_policies: IndexMap::new(),
            element_overrides: IndexMap::new(),
        }
    }
}

/// Namespace-specific validation policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamespacePolicy {
    /// Whether attributes in this namespace are allowed
    pub allowed: bool,
    /// Whether to validate format for this namespace
    pub validate_format: bool,
    /// Custom validation rules for this namespace
    pub custom_rules: IndexMap<String, Vec<ValidationRule>>,
}

/// Element-specific validation configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElementValidationConfig {
    /// Required attributes for this element
    pub required_attributes: IndexSet<QName>,
    /// Forbidden attributes for this element
    pub forbidden_attributes: IndexSet<QName>,
    /// Custom validation rules
    pub custom_rules: IndexMap<QName, Vec<ValidationRule>>,
}

/// Comprehensive attribute validator
#[derive(Debug, Clone)]
pub struct AttributeValidator {
    /// Global validation rules by attribute name
    global_rules: IndexMap<QName, Vec<ValidationRule>>,
    /// Element-specific validation rules
    element_rules: IndexMap<QName, IndexMap<QName, Vec<ValidationRule>>>,
    /// Validation policy
    policy: ValidationPolicy,
    /// Custom validation functions
    custom_validators: HashMap<String, fn(&AttributeValue) -> Result<(), String>>,
    /// Compiled regex cache
    regex_cache: HashMap<String, Regex>,
}

impl AttributeValidator {
    pub fn new() -> Self {
        let mut validator = Self {
            global_rules: IndexMap::new(),
            element_rules: IndexMap::new(),
            policy: ValidationPolicy::default(),
            custom_validators: HashMap::new(),
            regex_cache: HashMap::new(),
        };
        
        validator.setup_ddex_rules();
        validator.setup_xml_schema_rules();
        validator.setup_custom_validators();
        validator
    }
    
    /// Create validator with specific policy
    pub fn with_policy(policy: ValidationPolicy) -> Self {
        let mut validator = Self::new();
        validator.policy = policy;
        validator
    }
    
    /// Add global validation rule for an attribute
    pub fn add_global_rule(&mut self, attribute: QName, rule: ValidationRule) {
        self.global_rules
            .entry(attribute)
            .or_insert_with(Vec::new)
            .push(rule);
    }
    
    /// Add element-specific validation rule
    pub fn add_element_rule(&mut self, element: QName, attribute: QName, rule: ValidationRule) {
        self.element_rules
            .entry(element)
            .or_insert_with(IndexMap::new)
            .entry(attribute)
            .or_insert_with(Vec::new)
            .push(rule);
    }
    
    /// Add custom validation function
    pub fn add_custom_validator<F>(&mut self, name: String, _validator: F)
    where
        F: Fn(&AttributeValue) -> Result<(), String> + 'static,
    {
        // For now, we'll store the name and implement specific validators
        // In a real implementation, we'd need a more complex system for dynamic functions
        match name.as_str() {
            "ddex_territory_code" => {
                self.custom_validators.insert(name, Self::validate_territory_code);
            },
            "ddex_language_code" => {
                self.custom_validators.insert(name, Self::validate_language_code);
            },
            "ddex_currency_code" => {
                self.custom_validators.insert(name, Self::validate_currency_code);
            },
            _ => {
                // Store placeholder - in real implementation would store actual function
            }
        }
    }
    
    /// Validate attributes for a specific element
    pub fn validate_element_attributes(
        &mut self,
        element_name: &QName,
        attributes: &AttributeMap,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Get applicable rules
        let element_rules = self.element_rules.get(element_name).cloned().unwrap_or_default();
        
        // Check required attributes first
        self.validate_required_attributes(&element_rules, attributes, &mut result);
        
        // Validate each attribute
        for (attr_qname, attr_value) in attributes {
            self.validate_single_attribute(
                element_name,
                attr_qname,
                attr_value,
                &element_rules,
                &mut result,
            );
        }
        
        // Cross-attribute validation
        self.validate_cross_attributes(element_name, attributes, &mut result);
        
        // Policy validation
        self.validate_policy_compliance(element_name, attributes, &mut result);
        
        result
    }
    
    /// Validate all attributes globally (no element context)
    pub fn validate_global_attributes(&mut self, attributes: &AttributeMap) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        for (attr_qname, attr_value) in attributes {
            if let Some(rules) = self.global_rules.get(attr_qname).cloned() {
                for rule in &rules {
                    if let Err(error) = self.apply_validation_rule(attr_qname, attr_value, rule) {
                        result.add_error(error);
                    }
                }
            }
        }
        
        result
    }
    
    fn validate_required_attributes(
        &self,
        element_rules: &IndexMap<QName, Vec<ValidationRule>>,
        attributes: &AttributeMap,
        result: &mut ValidationResult,
    ) {
        for (attr_qname, rules) in element_rules {
            if rules.contains(&ValidationRule::Required) && !attributes.contains_key(attr_qname) {
                result.add_error(AttributeValidationError::MissingRequired {
                    attribute: attr_qname.clone(),
                });
            }
        }
    }
    
    fn validate_single_attribute(
        &mut self,
        _element_name: &QName,
        attr_qname: &QName,
        attr_value: &AttributeValue,
        element_rules: &IndexMap<QName, Vec<ValidationRule>>,
        result: &mut ValidationResult,
    ) {
        // Apply element-specific rules
        if let Some(rules) = element_rules.get(attr_qname).cloned() {
            for rule in &rules {
                if let Err(error) = self.apply_validation_rule(attr_qname, attr_value, rule) {
                    result.add_error(error);
                }
            }
        }
        
        // Apply global rules
        if let Some(rules) = self.global_rules.get(attr_qname).cloned() {
            for rule in &rules {
                if let Err(error) = self.apply_validation_rule(attr_qname, attr_value, rule) {
                    result.add_error(error);
                }
            }
        }
        
        // Namespace validation
        if self.policy.validate_namespaces {
            self.validate_namespace_compliance(attr_qname, result);
        }
    }
    
    fn apply_validation_rule(
        &mut self,
        attr_qname: &QName,
        attr_value: &AttributeValue,
        rule: &ValidationRule,
    ) -> Result<(), AttributeValidationError> {
        match rule {
            ValidationRule::Required => {
                // This is handled separately in validate_required_attributes
                Ok(())
            },
            ValidationRule::Type(expected_type) => {
                let actual_type = self.get_attribute_type(attr_value);
                if actual_type != *expected_type {
                    Err(AttributeValidationError::InvalidType {
                        attribute: attr_qname.clone(),
                        expected: *expected_type,
                        actual: actual_type,
                    })
                } else {
                    Ok(())
                }
            },
            ValidationRule::Enum(allowed_values) => {
                let value_str = attr_value.to_string();
                if !allowed_values.contains(&value_str) {
                    Err(AttributeValidationError::InvalidEnumValue {
                        attribute: attr_qname.clone(),
                        value: value_str,
                        allowed_values: allowed_values.clone(),
                    })
                } else {
                    Ok(())
                }
            },
            ValidationRule::Regex(pattern) => {
                let regex = self.get_or_compile_regex(pattern)?;
                let value_str = attr_value.to_string();
                if !regex.is_match(&value_str) {
                    Err(AttributeValidationError::InvalidFormat {
                        attribute: attr_qname.clone(),
                        reason: format!("does not match pattern: {}", pattern),
                    })
                } else {
                    Ok(())
                }
            },
            ValidationRule::Uri => {
                let value_str = attr_value.to_string();
                if Url::parse(&value_str).is_err() {
                    Err(AttributeValidationError::InvalidFormat {
                        attribute: attr_qname.clone(),
                        reason: "invalid URI format".to_string(),
                    })
                } else {
                    Ok(())
                }
            },
            ValidationRule::Email => {
                let value_str = attr_value.to_string();
                let email_regex = self.get_or_compile_regex(r"^[^\s@]+@[^\s@]+\.[^\s@]+$")?;
                if !email_regex.is_match(&value_str) {
                    Err(AttributeValidationError::InvalidFormat {
                        attribute: attr_qname.clone(),
                        reason: "invalid email format".to_string(),
                    })
                } else {
                    Ok(())
                }
            },
            ValidationRule::Date => {
                let value_str = attr_value.to_string();
                if NaiveDate::parse_from_str(&value_str, "%Y-%m-%d").is_err() {
                    Err(AttributeValidationError::InvalidFormat {
                        attribute: attr_qname.clone(),
                        reason: "invalid date format, expected YYYY-MM-DD".to_string(),
                    })
                } else {
                    Ok(())
                }
            },
            ValidationRule::DateTime => {
                let value_str = attr_value.to_string();
                if value_str.parse::<DateTime<Utc>>().is_err() {
                    Err(AttributeValidationError::InvalidFormat {
                        attribute: attr_qname.clone(),
                        reason: "invalid datetime format, expected RFC3339".to_string(),
                    })
                } else {
                    Ok(())
                }
            },
            ValidationRule::Range { min, max } => {
                self.validate_numeric_range(attr_qname, attr_value, *min, *max)
            },
            ValidationRule::Length { min, max } => {
                self.validate_string_length(attr_qname, attr_value, *min, *max)
            },
            ValidationRule::Custom(validator_name) => {
                if let Some(validator) = self.custom_validators.get(validator_name) {
                    match validator(attr_value) {
                        Ok(()) => Ok(()),
                        Err(reason) => Err(AttributeValidationError::CustomValidationFailed {
                            attribute: attr_qname.clone(),
                            rule: reason,
                        }),
                    }
                } else {
                    Err(AttributeValidationError::CustomValidationFailed {
                        attribute: attr_qname.clone(),
                        rule: format!("Unknown validator: {}", validator_name),
                    })
                }
            },
            ValidationRule::Dependency { depends_on: _, condition: _ } => {
                // Dependencies are handled in validate_cross_attributes
                Ok(())
            },
        }
    }
    
    fn validate_cross_attributes(
        &self,
        _element_name: &QName,
        attributes: &AttributeMap,
        result: &mut ValidationResult,
    ) {
        // Collect all dependency rules
        let mut dependencies = Vec::new();
        
        for rules in self.global_rules.values() {
            for rule in rules {
                if let ValidationRule::Dependency { depends_on, condition } = rule {
                    dependencies.push((depends_on, condition));
                }
            }
        }
        
        // Apply dependency rules
        for (attr_qname, attr_rules) in &self.global_rules {
            for rule in attr_rules {
                if let ValidationRule::Dependency { depends_on, condition } = rule {
                    if let Err(error) = self.validate_dependency(
                        attr_qname,
                        depends_on,
                        condition,
                        attributes,
                    ) {
                        result.add_error(error);
                    }
                }
            }
        }
    }
    
    fn validate_dependency(
        &self,
        attr_qname: &QName,
        depends_on: &QName,
        condition: &DependencyCondition,
        attributes: &AttributeMap,
    ) -> Result<(), AttributeValidationError> {
        let has_attr = attributes.contains_key(attr_qname);
        let has_dependency = attributes.contains_key(depends_on);
        
        match condition {
            DependencyCondition::RequiredWhenExists => {
                if has_dependency && !has_attr {
                    Err(AttributeValidationError::CrossAttributeValidationFailed {
                        message: format!(
                            "Attribute {} is required when {} exists",
                            attr_qname.local_name, depends_on.local_name
                        ),
                    })
                } else {
                    Ok(())
                }
            },
            DependencyCondition::RequiredWhenEquals(value) => {
                if let Some(dep_value) = attributes.get(depends_on) {
                    if dep_value.to_string() == *value && !has_attr {
                        Err(AttributeValidationError::CrossAttributeValidationFailed {
                            message: format!(
                                "Attribute {} is required when {} equals '{}'",
                                attr_qname.local_name, depends_on.local_name, value
                            ),
                        })
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            },
            DependencyCondition::ForbiddenWhenExists => {
                if has_dependency && has_attr {
                    Err(AttributeValidationError::CrossAttributeValidationFailed {
                        message: format!(
                            "Attribute {} is forbidden when {} exists",
                            attr_qname.local_name, depends_on.local_name
                        ),
                    })
                } else {
                    Ok(())
                }
            },
            DependencyCondition::MustMatch => {
                if let (Some(attr_value), Some(dep_value)) = 
                    (attributes.get(attr_qname), attributes.get(depends_on)) {
                    if attr_value.to_string() != dep_value.to_string() {
                        Err(AttributeValidationError::CrossAttributeValidationFailed {
                            message: format!(
                                "Attribute {} must match {}",
                                attr_qname.local_name, depends_on.local_name
                            ),
                        })
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            },
            DependencyCondition::MustDiffer => {
                if let (Some(attr_value), Some(dep_value)) = 
                    (attributes.get(attr_qname), attributes.get(depends_on)) {
                    if attr_value.to_string() == dep_value.to_string() {
                        Err(AttributeValidationError::CrossAttributeValidationFailed {
                            message: format!(
                                "Attribute {} must differ from {}",
                                attr_qname.local_name, depends_on.local_name
                            ),
                        })
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            },
        }
    }
    
    fn validate_policy_compliance(
        &self,
        _element_name: &QName,
        attributes: &AttributeMap,
        result: &mut ValidationResult,
    ) {
        if !self.policy.allow_unknown_attributes {
            for attr_qname in attributes.keys() {
                if !self.is_known_attribute(attr_qname) {
                    let message = format!("Unknown attribute not allowed: {}", attr_qname.local_name);
                    if self.policy.strict_mode {
                        result.add_error(AttributeValidationError::PolicyViolation {
                            attribute: attr_qname.clone(),
                            policy: message.clone(),
                        });
                    } else {
                        result.add_warning(message);
                    }
                }
            }
        }
    }
    
    fn validate_namespace_compliance(&self, attr_qname: &QName, result: &mut ValidationResult) {
        if let Some(namespace_uri) = &attr_qname.namespace_uri {
            if let Some(policy) = self.policy.namespace_policies.get(namespace_uri) {
                if !policy.allowed {
                    result.add_error(AttributeValidationError::NamespaceValidationFailed {
                        attribute: attr_qname.clone(),
                        reason: format!("Namespace {} not allowed", namespace_uri),
                    });
                }
            }
        }
    }
    
    // Helper methods
    
    fn get_attribute_type(&self, value: &AttributeValue) -> AttributeType {
        match value {
            AttributeValue::String(_) => AttributeType::String,
            AttributeValue::Boolean(_) => AttributeType::Boolean,
            AttributeValue::Integer(_) => AttributeType::Integer,
            AttributeValue::Decimal(_) => AttributeType::Decimal,
            AttributeValue::Date(_) => AttributeType::Date,
            AttributeValue::DateTime(_) => AttributeType::DateTime,
            AttributeValue::Uri(_) => AttributeType::Uri,
            AttributeValue::Language(_) => AttributeType::Language,
            AttributeValue::Token(_) => AttributeType::Token,
            AttributeValue::Duration(_) => AttributeType::Raw,
            AttributeValue::Enum(_, _) => AttributeType::String,
            AttributeValue::Raw(_) => AttributeType::Raw,
        }
    }
    
    fn get_or_compile_regex(&mut self, pattern: &str) -> Result<&Regex, AttributeValidationError> {
        if !self.regex_cache.contains_key(pattern) {
            let regex = Regex::new(pattern).map_err(|e| {
                AttributeValidationError::InvalidFormat {
                    attribute: QName::new("regex".to_string()),
                    reason: format!("Invalid regex pattern: {}", e),
                }
            })?;
            self.regex_cache.insert(pattern.to_string(), regex);
        }
        Ok(self.regex_cache.get(pattern).unwrap())
    }
    
    fn validate_numeric_range(
        &self,
        attr_qname: &QName,
        attr_value: &AttributeValue,
        min: Option<i64>,
        max: Option<i64>,
    ) -> Result<(), AttributeValidationError> {
        let numeric_value = match attr_value {
            AttributeValue::Integer(i) => *i,
            AttributeValue::Decimal(d) => *d as i64,
            _ => return Err(AttributeValidationError::InvalidFormat {
                attribute: attr_qname.clone(),
                reason: "not a numeric value".to_string(),
            }),
        };
        
        if let Some(min_val) = min {
            if numeric_value < min_val {
                return Err(AttributeValidationError::ValueOutOfRange {
                    attribute: attr_qname.clone(),
                    value: numeric_value.to_string(),
                    range: format!(">= {}", min_val),
                });
            }
        }
        
        if let Some(max_val) = max {
            if numeric_value > max_val {
                return Err(AttributeValidationError::ValueOutOfRange {
                    attribute: attr_qname.clone(),
                    value: numeric_value.to_string(),
                    range: format!("<= {}", max_val),
                });
            }
        }
        
        Ok(())
    }
    
    fn validate_string_length(
        &self,
        attr_qname: &QName,
        attr_value: &AttributeValue,
        min: Option<usize>,
        max: Option<usize>,
    ) -> Result<(), AttributeValidationError> {
        let string_value = attr_value.to_string();
        let length = string_value.len();
        
        if let Some(min_len) = min {
            if length < min_len {
                return Err(AttributeValidationError::ValueOutOfRange {
                    attribute: attr_qname.clone(),
                    value: length.to_string(),
                    range: format!(">= {} characters", min_len),
                });
            }
        }
        
        if let Some(max_len) = max {
            if length > max_len {
                return Err(AttributeValidationError::ValueOutOfRange {
                    attribute: attr_qname.clone(),
                    value: length.to_string(),
                    range: format!("<= {} characters", max_len),
                });
            }
        }
        
        Ok(())
    }
    
    fn is_known_attribute(&self, attr_qname: &QName) -> bool {
        self.global_rules.contains_key(attr_qname) ||
            self.element_rules.values()
                .any(|element_rules| element_rules.contains_key(attr_qname))
    }
    
    // Setup methods for DDEX and XML Schema rules
    
    fn setup_ddex_rules(&mut self) {
        // DDEX-specific validation rules
        self.add_global_rule(
            QName::new("TerritoryCode".to_string()),
            ValidationRule::Custom("ddex_territory_code".to_string()),
        );
        
        self.add_global_rule(
            QName::new("LanguageAndScriptCode".to_string()),
            ValidationRule::Custom("ddex_language_code".to_string()),
        );
        
        self.add_global_rule(
            QName::new("CurrencyCode".to_string()),
            ValidationRule::Custom("ddex_currency_code".to_string()),
        );
        
        // Sequence number validation
        self.add_global_rule(
            QName::new("SequenceNumber".to_string()),
            ValidationRule::Range { min: Some(1), max: Some(999999) },
        );
        
        // DDEX identifier patterns
        self.add_global_rule(
            QName::new("ISRC".to_string()),
            ValidationRule::Regex(r"^[A-Z]{2}[A-Z0-9]{3}[0-9]{7}$".to_string()),
        );
        
        self.add_global_rule(
            QName::new("ISWC".to_string()),
            ValidationRule::Regex(r"^T-[0-9]{9}-[0-9]$".to_string()),
        );
    }
    
    fn setup_xml_schema_rules(&mut self) {
        // XML Schema instance attributes
        let xsi_ns = "http://www.w3.org/2001/XMLSchema-instance";
        
        self.add_global_rule(
            QName::with_namespace("type".to_string(), xsi_ns.to_string()),
            ValidationRule::Type(AttributeType::Token),
        );
        
        self.add_global_rule(
            QName::with_namespace("nil".to_string(), xsi_ns.to_string()),
            ValidationRule::Type(AttributeType::Boolean),
        );
        
        self.add_global_rule(
            QName::with_namespace("schemaLocation".to_string(), xsi_ns.to_string()),
            ValidationRule::Type(AttributeType::Uri),
        );
    }
    
    fn setup_custom_validators(&mut self) {
        self.add_custom_validator("ddex_territory_code".to_string(), Self::validate_territory_code);
        self.add_custom_validator("ddex_language_code".to_string(), Self::validate_language_code);
        self.add_custom_validator("ddex_currency_code".to_string(), Self::validate_currency_code);
    }
    
    // Custom validation functions
    
    fn validate_territory_code(value: &AttributeValue) -> Result<(), String> {
        let code = value.to_string();
        // ISO 3166-1 alpha-2 country codes (simplified validation)
        if code.len() != 2 || !code.chars().all(|c| c.is_ascii_uppercase()) {
            Err("Invalid territory code format, expected 2 uppercase letters".to_string())
        } else {
            Ok(())
        }
    }
    
    fn validate_language_code(value: &AttributeValue) -> Result<(), String> {
        let code = value.to_string();
        // Simplified language code validation (ISO 639-1)
        if code.len() < 2 || code.len() > 8 {
            Err("Invalid language code format".to_string())
        } else {
            Ok(())
        }
    }
    
    fn validate_currency_code(value: &AttributeValue) -> Result<(), String> {
        let code = value.to_string();
        // ISO 4217 currency codes
        if code.len() != 3 || !code.chars().all(|c| c.is_ascii_uppercase()) {
            Err("Invalid currency code format, expected 3 uppercase letters".to_string())
        } else {
            Ok(())
        }
    }
}

impl Default for AttributeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_validation() {
        let mut validator = AttributeValidator::new();
        
        // Add required attribute rule
        let attr_name = QName::new("required_attr".to_string());
        validator.add_global_rule(attr_name.clone(), ValidationRule::Required);
        
        let mut attributes = AttributeMap::new();
        let result = validator.validate_global_attributes(&attributes);
        
        // Should not fail since we're not testing element-specific validation
        assert!(result.is_valid);
    }
    
    #[test]
    fn test_enum_validation() {
        let mut validator = AttributeValidator::new();
        
        let attr_name = QName::new("enum_attr".to_string());
        validator.add_global_rule(
            attr_name.clone(),
            ValidationRule::Enum(vec!["value1".to_string(), "value2".to_string()]),
        );
        
        let mut attributes = AttributeMap::new();
        attributes.insert(attr_name.clone(), AttributeValue::String("value1".to_string()));
        
        let result = validator.validate_global_attributes(&attributes);
        assert!(result.is_valid);
        
        // Test invalid enum value
        attributes.insert(attr_name.clone(), AttributeValue::String("invalid".to_string()));
        let result = validator.validate_global_attributes(&attributes);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
    }
    
    #[test]
    fn test_regex_validation() {
        let mut validator = AttributeValidator::new();
        
        let attr_name = QName::new("pattern_attr".to_string());
        validator.add_global_rule(
            attr_name.clone(),
            ValidationRule::Regex(r"^\d{4}-\d{2}-\d{2}$".to_string()),
        );
        
        let mut attributes = AttributeMap::new();
        attributes.insert(attr_name.clone(), AttributeValue::String("2023-12-25".to_string()));
        
        let result = validator.validate_global_attributes(&attributes);
        assert!(result.is_valid);
        
        // Test invalid pattern
        attributes.insert(attr_name.clone(), AttributeValue::String("invalid-date".to_string()));
        let result = validator.validate_global_attributes(&attributes);
        assert!(!result.is_valid);
    }
    
    #[test]
    fn test_uri_validation() {
        let mut validator = AttributeValidator::new();
        
        let attr_name = QName::new("uri_attr".to_string());
        validator.add_global_rule(attr_name.clone(), ValidationRule::Uri);
        
        let mut attributes = AttributeMap::new();
        attributes.insert(attr_name.clone(), AttributeValue::String("https://example.com".to_string()));
        
        let result = validator.validate_global_attributes(&attributes);
        assert!(result.is_valid);
        
        // Test invalid URI
        attributes.insert(attr_name.clone(), AttributeValue::String("not-a-uri".to_string()));
        let result = validator.validate_global_attributes(&attributes);
        assert!(!result.is_valid);
    }
    
    #[test]
    fn test_range_validation() {
        let mut validator = AttributeValidator::new();
        
        let attr_name = QName::new("numeric_attr".to_string());
        validator.add_global_rule(
            attr_name.clone(),
            ValidationRule::Range { min: Some(1), max: Some(100) },
        );
        
        let mut attributes = AttributeMap::new();
        attributes.insert(attr_name.clone(), AttributeValue::Integer(50));
        
        let result = validator.validate_global_attributes(&attributes);
        assert!(result.is_valid);
        
        // Test out of range
        attributes.insert(attr_name.clone(), AttributeValue::Integer(150));
        let result = validator.validate_global_attributes(&attributes);
        assert!(!result.is_valid);
    }
    
    #[test]
    fn test_custom_validation() {
        let mut validator = AttributeValidator::new();
        
        let attr_name = QName::new("TerritoryCode".to_string());
        // Rule already added in setup_ddex_rules
        
        let mut attributes = AttributeMap::new();
        attributes.insert(attr_name.clone(), AttributeValue::String("US".to_string()));
        
        let result = validator.validate_global_attributes(&attributes);
        assert!(result.is_valid);
        
        // Test invalid territory code
        attributes.insert(attr_name.clone(), AttributeValue::String("invalid".to_string()));
        let result = validator.validate_global_attributes(&attributes);
        assert!(!result.is_valid);
    }
}