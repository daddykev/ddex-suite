//! Schema-based validation for DDEX structures

use super::*;
use serde_json::Value as JsonValue;
use regex::Regex;
use once_cell::sync::Lazy;

/// Schema-based validator for DDEX data
#[derive(Debug, Clone)]
pub struct SchemaValidator {
    /// The schema to validate against
    schema: JsonSchema,
    /// Validation configuration
    config: ValidationConfig,
}

/// Configuration for schema validation
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Strict mode - fail on warnings
    pub strict_mode: bool,
    /// Validate formats (dates, URIs, etc.)
    pub validate_formats: bool,
    /// Maximum validation depth
    pub max_depth: usize,
    /// Allow additional properties not in schema
    pub allow_additional_properties: bool,
}

/// Result of schema validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Validation errors (schema violations)
    pub errors: Vec<ValidationError>,
    /// Validation warnings (recommendations)
    pub warnings: Vec<ValidationWarning>,
    /// Validation metadata
    pub metadata: ValidationMetadata,
}

/// Schema validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// JSON path where error occurred
    pub instance_path: String,
    /// Schema path that failed
    pub schema_path: String,
    /// The value that caused the error
    pub invalid_value: Option<JsonValue>,
}

/// Schema validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning code
    pub code: String,
    /// Warning message
    pub message: String,
    /// JSON path where warning occurred
    pub instance_path: String,
    /// Suggestion for improvement
    pub suggestion: Option<String>,
}

/// Validation metadata
#[derive(Debug, Clone)]
pub struct ValidationMetadata {
    /// Number of properties validated
    pub properties_validated: usize,
    /// Validation duration
    pub validation_time: std::time::Duration,
    /// Schema complexity
    pub schema_complexity: f64,
    /// Data complexity
    pub data_complexity: f64,
}

// Common DDEX validation patterns
static ISRC_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Z]{2}[A-Z0-9]{3}\d{7}$").unwrap()
});

static UPC_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d{12}$").unwrap()
});

static LANGUAGE_CODE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z]{2}(-[A-Z]{2})?$").unwrap()
});

static TERRITORY_CODE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Z]{2}|Worldwide$").unwrap()
});

static DURATION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^PT(?:\d+H)?(?:\d+M)?(?:\d+(?:\.\d+)?S)?$").unwrap()
});

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            validate_formats: true,
            max_depth: 100,
            allow_additional_properties: false,
        }
    }
}

impl SchemaValidator {
    /// Create a new schema validator
    pub fn new(schema: JsonSchema) -> Self {
        Self {
            schema,
            config: ValidationConfig::default(),
        }
    }
    
    /// Create validator with custom configuration
    pub fn with_config(schema: JsonSchema, config: ValidationConfig) -> Self {
        Self { schema, config }
    }
    
    /// Validate JSON data against the schema
    pub fn validate(&self, data: &JsonValue) -> ValidationResult {
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut properties_validated = 0;
        
        self.validate_recursive(
            data, 
            &self.schema, 
            "", 
            "", 
            0, 
            &mut errors, 
            &mut warnings,
            &mut properties_validated
        );
        
        let validation_time = start_time.elapsed();
        let valid = errors.is_empty() && (!self.config.strict_mode || warnings.is_empty());
        
        ValidationResult {
            valid,
            errors,
            warnings,
            metadata: ValidationMetadata {
                properties_validated,
                validation_time,
                schema_complexity: self.calculate_schema_complexity(&self.schema),
                data_complexity: self.calculate_data_complexity(data),
            },
        }
    }
    
    /// Validate and return detailed path-based errors
    pub fn validate_with_paths(&self, data: &JsonValue) -> ValidationResult {
        // Same as validate but with more detailed path tracking
        self.validate(data)
    }
    
    // Private validation methods
    
    fn validate_recursive(
        &self,
        data: &JsonValue,
        schema: &JsonSchema,
        instance_path: &str,
        schema_path: &str,
        depth: usize,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
        properties_validated: &mut usize,
    ) {
        if depth > self.config.max_depth {
            errors.push(ValidationError {
                code: "MAX_DEPTH_EXCEEDED".to_string(),
                message: format!("Maximum validation depth {} exceeded", self.config.max_depth),
                instance_path: instance_path.to_string(),
                schema_path: schema_path.to_string(),
                invalid_value: Some(data.clone()),
            });
            return;
        }
        
        *properties_validated += 1;
        
        // Handle schema references
        if let Some(ref reference) = schema.reference {
            if let Some(resolved_schema) = self.resolve_reference(reference) {
                return self.validate_recursive(
                    data, &resolved_schema, instance_path, 
                    &format!("{}/{}", schema_path, reference),
                    depth + 1, errors, warnings, properties_validated
                );
            } else {
                errors.push(ValidationError {
                    code: "UNRESOLVED_REFERENCE".to_string(),
                    message: format!("Cannot resolve schema reference: {}", reference),
                    instance_path: instance_path.to_string(),
                    schema_path: schema_path.to_string(),
                    invalid_value: None,
                });
                return;
            }
        }
        
        // Validate type
        if let Some(ref expected_type) = schema.schema_type {
            self.validate_type(data, expected_type, instance_path, schema_path, errors);
        }
        
        match data {
            JsonValue::Object(obj) => {
                self.validate_object(obj, schema, instance_path, schema_path, depth, errors, warnings, properties_validated);
            },
            JsonValue::Array(arr) => {
                self.validate_array(arr, schema, instance_path, schema_path, depth, errors, warnings, properties_validated);
            },
            JsonValue::String(s) => {
                self.validate_string(s, schema, instance_path, schema_path, errors, warnings);
            },
            JsonValue::Number(_) => {
                self.validate_number(data, schema, instance_path, schema_path, errors, warnings);
            },
            _ => {}
        }
        
        // Validate enum values
        if let Some(ref enum_values) = schema.enum_values {
            if !enum_values.contains(data) {
                errors.push(ValidationError {
                    code: "ENUM_VIOLATION".to_string(),
                    message: format!("Value must be one of: {}", 
                        enum_values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
                    instance_path: instance_path.to_string(),
                    schema_path: format!("{}/enum", schema_path),
                    invalid_value: Some(data.clone()),
                });
            }
        }
        
        // Handle conditional schemas
        self.validate_conditionals(data, schema, instance_path, schema_path, depth, errors, warnings, properties_validated);
    }
    
    fn validate_object(
        &self,
        obj: &serde_json::Map<String, JsonValue>,
        schema: &JsonSchema,
        instance_path: &str,
        schema_path: &str,
        depth: usize,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
        properties_validated: &mut usize,
    ) {
        // Check required properties
        if let Some(ref required) = schema.required {
            for required_prop in required {
                if !obj.contains_key(required_prop) {
                    errors.push(ValidationError {
                        code: "REQUIRED_PROPERTY_MISSING".to_string(),
                        message: format!("Required property '{}' is missing", required_prop),
                        instance_path: instance_path.to_string(),
                        schema_path: format!("{}/required", schema_path),
                        invalid_value: None,
                    });
                }
            }
        }
        
        // Validate properties
        if let Some(ref properties) = schema.properties {
            for (prop_name, prop_value) in obj {
                let new_instance_path = if instance_path.is_empty() {
                    prop_name.clone()
                } else {
                    format!("{}/{}", instance_path, prop_name)
                };
                
                if let Some(prop_schema) = properties.get(prop_name) {
                    self.validate_recursive(
                        prop_value, prop_schema, &new_instance_path,
                        &format!("{}/properties/{}", schema_path, prop_name),
                        depth + 1, errors, warnings, properties_validated
                    );
                } else if !self.config.allow_additional_properties && 
                    schema.additional_properties.unwrap_or(true) == false {
                    errors.push(ValidationError {
                        code: "ADDITIONAL_PROPERTY_NOT_ALLOWED".to_string(),
                        message: format!("Additional property '{}' is not allowed", prop_name),
                        instance_path: new_instance_path,
                        schema_path: format!("{}/additionalProperties", schema_path),
                        invalid_value: Some(prop_value.clone()),
                    });
                }
            }
        }
    }
    
    fn validate_array(
        &self,
        arr: &Vec<JsonValue>,
        schema: &JsonSchema,
        instance_path: &str,
        schema_path: &str,
        depth: usize,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
        properties_validated: &mut usize,
    ) {
        // Check min/max length
        if let Some(min_length) = schema.min_length {
            if arr.len() < min_length {
                errors.push(ValidationError {
                    code: "ARRAY_TOO_SHORT".to_string(),
                    message: format!("Array must have at least {} items, has {}", min_length, arr.len()),
                    instance_path: instance_path.to_string(),
                    schema_path: format!("{}/minLength", schema_path),
                    invalid_value: Some(JsonValue::Array(arr.clone())),
                });
            }
        }
        
        if let Some(max_length) = schema.max_length {
            if arr.len() > max_length {
                errors.push(ValidationError {
                    code: "ARRAY_TOO_LONG".to_string(),
                    message: format!("Array must have at most {} items, has {}", max_length, arr.len()),
                    instance_path: instance_path.to_string(),
                    schema_path: format!("{}/maxLength", schema_path),
                    invalid_value: Some(JsonValue::Array(arr.clone())),
                });
            }
        }
        
        // Validate items
        if let Some(ref items_schema) = schema.items {
            for (index, item) in arr.iter().enumerate() {
                let new_instance_path = format!("{}/{}", instance_path, index);
                self.validate_recursive(
                    item, items_schema, &new_instance_path,
                    &format!("{}/items", schema_path),
                    depth + 1, errors, warnings, properties_validated
                );
            }
        }
    }
    
    fn validate_string(
        &self,
        s: &str,
        schema: &JsonSchema,
        instance_path: &str,
        schema_path: &str,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // Check length constraints
        if let Some(min_length) = schema.min_length {
            if s.len() < min_length {
                errors.push(ValidationError {
                    code: "STRING_TOO_SHORT".to_string(),
                    message: format!("String must be at least {} characters, is {}", min_length, s.len()),
                    instance_path: instance_path.to_string(),
                    schema_path: format!("{}/minLength", schema_path),
                    invalid_value: Some(JsonValue::String(s.to_string())),
                });
            }
        }
        
        if let Some(max_length) = schema.max_length {
            if s.len() > max_length {
                errors.push(ValidationError {
                    code: "STRING_TOO_LONG".to_string(),
                    message: format!("String must be at most {} characters, is {}", max_length, s.len()),
                    instance_path: instance_path.to_string(),
                    schema_path: format!("{}/maxLength", schema_path),
                    invalid_value: Some(JsonValue::String(s.to_string())),
                });
            }
        }
        
        // Check pattern
        if let Some(ref pattern) = schema.pattern {
            if let Ok(regex) = Regex::new(pattern) {
                if !regex.is_match(s) {
                    errors.push(ValidationError {
                        code: "PATTERN_MISMATCH".to_string(),
                        message: format!("String does not match required pattern: {}", pattern),
                        instance_path: instance_path.to_string(),
                        schema_path: format!("{}/pattern", schema_path),
                        invalid_value: Some(JsonValue::String(s.to_string())),
                    });
                }
            }
        }
        
        // Validate format
        if let Some(ref format) = schema.format {
            self.validate_format(s, format, instance_path, schema_path, errors, warnings);
        }
        
        // Special DDEX validations
        self.validate_ddex_codes(s, instance_path, errors, warnings);
    }
    
    fn validate_number(
        &self,
        _num: &JsonValue,
        _schema: &JsonSchema,
        _instance_path: &str,
        _schema_path: &str,
        _errors: &mut Vec<ValidationError>,
        _warnings: &mut Vec<ValidationWarning>,
    ) {
        // Number validation logic would go here
        // (min, max, multipleOf, etc.)
    }
    
    fn validate_type(
        &self,
        data: &JsonValue,
        expected_type: &str,
        instance_path: &str,
        schema_path: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        let actual_type = match data {
            JsonValue::Null => "null",
            JsonValue::Bool(_) => "boolean",
            JsonValue::Number(_) => "number",
            JsonValue::String(_) => "string",
            JsonValue::Array(_) => "array",
            JsonValue::Object(_) => "object",
        };
        
        if actual_type != expected_type {
            errors.push(ValidationError {
                code: "TYPE_MISMATCH".to_string(),
                message: format!("Expected type '{}', got '{}'", expected_type, actual_type),
                instance_path: instance_path.to_string(),
                schema_path: format!("{}/type", schema_path),
                invalid_value: Some(data.clone()),
            });
        }
    }
    
    fn validate_format(
        &self,
        s: &str,
        format: &str,
        instance_path: &str,
        schema_path: &str,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        if !self.config.validate_formats {
            return;
        }
        
        match format {
            "date" => {
                if chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_err() {
                    errors.push(ValidationError {
                        code: "INVALID_DATE_FORMAT".to_string(),
                        message: "Invalid date format, expected YYYY-MM-DD".to_string(),
                        instance_path: instance_path.to_string(),
                        schema_path: format!("{}/format", schema_path),
                        invalid_value: Some(JsonValue::String(s.to_string())),
                    });
                }
            },
            "date-time" => {
                if chrono::DateTime::parse_from_rfc3339(s).is_err() {
                    errors.push(ValidationError {
                        code: "INVALID_DATETIME_FORMAT".to_string(),
                        message: "Invalid date-time format, expected ISO 8601/RFC 3339".to_string(),
                        instance_path: instance_path.to_string(),
                        schema_path: format!("{}/format", schema_path),
                        invalid_value: Some(JsonValue::String(s.to_string())),
                    });
                }
            },
            "uri" => {
                if url::Url::parse(s).is_err() {
                    errors.push(ValidationError {
                        code: "INVALID_URI_FORMAT".to_string(),
                        message: "Invalid URI format".to_string(),
                        instance_path: instance_path.to_string(),
                        schema_path: format!("{}/format", schema_path),
                        invalid_value: Some(JsonValue::String(s.to_string())),
                    });
                }
            },
            _ => {
                warnings.push(ValidationWarning {
                    code: "UNKNOWN_FORMAT".to_string(),
                    message: format!("Unknown format specifier: {}", format),
                    instance_path: instance_path.to_string(),
                    suggestion: Some("Check schema for supported format types".to_string()),
                });
            }
        }
    }
    
    fn validate_ddex_codes(
        &self,
        s: &str,
        instance_path: &str,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // Check for ISRC patterns in relevant fields
        if instance_path.contains("isrc") {
            if !ISRC_PATTERN.is_match(s) {
                errors.push(ValidationError {
                    code: "INVALID_ISRC".to_string(),
                    message: "Invalid ISRC format, expected format: CC-XXX-YY-NNNNN".to_string(),
                    instance_path: instance_path.to_string(),
                    schema_path: "pattern".to_string(),
                    invalid_value: Some(JsonValue::String(s.to_string())),
                });
            }
        }
        
        // Check for UPC patterns
        if instance_path.contains("upc") {
            if !UPC_PATTERN.is_match(s) {
                errors.push(ValidationError {
                    code: "INVALID_UPC".to_string(),
                    message: "Invalid UPC format, expected 12 digits".to_string(),
                    instance_path: instance_path.to_string(),
                    schema_path: "pattern".to_string(),
                    invalid_value: Some(JsonValue::String(s.to_string())),
                });
            }
        }
        
        // Check duration format
        if instance_path.contains("duration") {
            if !DURATION_PATTERN.is_match(s) {
                errors.push(ValidationError {
                    code: "INVALID_DURATION".to_string(),
                    message: "Invalid duration format, expected ISO 8601 duration (PT#M#S)".to_string(),
                    instance_path: instance_path.to_string(),
                    schema_path: "pattern".to_string(),
                    invalid_value: Some(JsonValue::String(s.to_string())),
                });
            }
        }
        
        // Check language codes
        if instance_path.contains("language_code") {
            if !LANGUAGE_CODE_PATTERN.is_match(s) {
                warnings.push(ValidationWarning {
                    code: "SUSPICIOUS_LANGUAGE_CODE".to_string(),
                    message: "Language code does not match ISO 639 format".to_string(),
                    instance_path: instance_path.to_string(),
                    suggestion: Some("Use ISO 639-1 language codes (e.g., 'en', 'fr', 'en-US')".to_string()),
                });
            }
        }
        
        // Check territory codes
        if instance_path.contains("territory_code") {
            if !TERRITORY_CODE_PATTERN.is_match(s) {
                warnings.push(ValidationWarning {
                    code: "SUSPICIOUS_TERRITORY_CODE".to_string(),
                    message: "Territory code should be ISO 3166 country code or 'Worldwide'".to_string(),
                    instance_path: instance_path.to_string(),
                    suggestion: Some("Use ISO 3166-1 alpha-2 country codes (e.g., 'US', 'GB') or 'Worldwide'".to_string()),
                });
            }
        }
    }
    
    fn validate_conditionals(
        &self,
        data: &JsonValue,
        schema: &JsonSchema,
        instance_path: &str,
        schema_path: &str,
        depth: usize,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
        properties_validated: &mut usize,
    ) {
        // Handle if/then/else conditions
        if let Some(ref if_schema) = schema.if_schema {
            let condition_result = self.test_condition(data, if_schema);
            
            if condition_result {
                if let Some(ref then_schema) = schema.then_schema {
                    self.validate_recursive(
                        data, then_schema, instance_path,
                        &format!("{}/then", schema_path),
                        depth + 1, errors, warnings, properties_validated
                    );
                }
            } else if let Some(ref else_schema) = schema.else_schema {
                self.validate_recursive(
                    data, else_schema, instance_path,
                    &format!("{}/else", schema_path),
                    depth + 1, errors, warnings, properties_validated
                );
            }
        }
        
        // Handle allOf, anyOf, oneOf
        if let Some(ref all_of) = schema.all_of {
            for (index, sub_schema) in all_of.iter().enumerate() {
                self.validate_recursive(
                    data, sub_schema, instance_path,
                    &format!("{}/allOf/{}", schema_path, index),
                    depth + 1, errors, warnings, properties_validated
                );
            }
        }
        
        if let Some(ref any_of) = schema.any_of {
            let mut any_valid = false;
            for sub_schema in any_of {
                let mut temp_errors = Vec::new();
                let mut temp_warnings = Vec::new();
                let mut temp_count = 0;
                
                self.validate_recursive(
                    data, sub_schema, instance_path, schema_path,
                    depth + 1, &mut temp_errors, &mut temp_warnings, &mut temp_count
                );
                
                if temp_errors.is_empty() {
                    any_valid = true;
                    break;
                }
            }
            
            if !any_valid {
                errors.push(ValidationError {
                    code: "ANY_OF_FAILED".to_string(),
                    message: "Data does not match any of the specified schemas".to_string(),
                    instance_path: instance_path.to_string(),
                    schema_path: format!("{}/anyOf", schema_path),
                    invalid_value: Some(data.clone()),
                });
            }
        }
        
        if let Some(ref one_of) = schema.one_of {
            let mut valid_count = 0;
            for sub_schema in one_of {
                let mut temp_errors = Vec::new();
                let mut temp_warnings = Vec::new();
                let mut temp_count = 0;
                
                self.validate_recursive(
                    data, sub_schema, instance_path, schema_path,
                    depth + 1, &mut temp_errors, &mut temp_warnings, &mut temp_count
                );
                
                if temp_errors.is_empty() {
                    valid_count += 1;
                }
            }
            
            if valid_count != 1 {
                errors.push(ValidationError {
                    code: "ONE_OF_FAILED".to_string(),
                    message: format!("Data matches {} schemas, expected exactly 1", valid_count),
                    instance_path: instance_path.to_string(),
                    schema_path: format!("{}/oneOf", schema_path),
                    invalid_value: Some(data.clone()),
                });
            }
        }
    }
    
    fn test_condition(&self, _data: &JsonValue, _if_schema: &JsonSchema) -> bool {
        // Simplified condition testing - would need full implementation
        true
    }
    
    fn resolve_reference(&self, reference: &str) -> Option<JsonSchema> {
        // Resolve JSON Schema $ref - simplified implementation
        if reference.starts_with("#/$defs/") {
            let def_name = &reference[8..];
            if let Some(ref definitions) = self.schema.definitions {
                return definitions.get(def_name).cloned();
            }
        }
        None
    }
    
    fn calculate_schema_complexity(&self, schema: &JsonSchema) -> f64 {
        let mut complexity = 0.0;
        
        if let Some(ref properties) = schema.properties {
            complexity += properties.len() as f64;
        }
        if let Some(ref definitions) = schema.definitions {
            complexity += definitions.len() as f64 * 2.0;
        }
        if schema.all_of.is_some() { complexity += 3.0; }
        if schema.any_of.is_some() { complexity += 4.0; }
        if schema.one_of.is_some() { complexity += 5.0; }
        if schema.if_schema.is_some() { complexity += 6.0; }
        
        complexity
    }
    
    fn calculate_data_complexity(&self, data: &JsonValue) -> f64 {
        match data {
            JsonValue::Object(obj) => {
                obj.len() as f64 + obj.values().map(|v| self.calculate_data_complexity(v) * 0.5).sum::<f64>()
            },
            JsonValue::Array(arr) => {
                arr.len() as f64 + arr.iter().map(|v| self.calculate_data_complexity(v) * 0.3).sum::<f64>()
            },
            _ => 1.0,
        }
    }
}

/// Utility functions for schema validation
impl ValidationResult {
    /// Check if validation passed completely
    pub fn is_valid(&self) -> bool {
        self.valid
    }
    
    /// Get all errors as formatted strings
    pub fn error_messages(&self) -> Vec<String> {
        self.errors.iter()
            .map(|e| format!("{}: {} (at {})", e.code, e.message, e.instance_path))
            .collect()
    }
    
    /// Get all warnings as formatted strings
    pub fn warning_messages(&self) -> Vec<String> {
        self.warnings.iter()
            .map(|w| format!("{}: {} (at {})", w.code, w.message, w.instance_path))
            .collect()
    }
    
    /// Get validation summary
    pub fn summary(&self) -> String {
        format!(
            "Validation {}: {} errors, {} warnings, {} properties validated in {:?}",
            if self.valid { "PASSED" } else { "FAILED" },
            self.errors.len(),
            self.warnings.len(),
            self.metadata.properties_validated,
            self.metadata.validation_time
        )
    }
}