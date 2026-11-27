//! # JSON Schema validation
//!
//! [`SchemaValidator`] validates a JSON value against a [`JsonSchema`]: `type`, `required`,
//! `properties`, `additionalProperties`, `enum`, `pattern`, string and array lengths,
//! `items`, local `$defs` references, `if` / `then` / `else`, `allOf` / `anyOf` / `oneOf`,
//! and the formats `date`, `date-time` and `uri`. Numbers are not checked.
//!
//! Before 0.6.0 this module also generated JSON Schema, TypeScript and Python definitions
//! for build requests. The schemas were hand-written, left out required `BuildRequest`
//! fields, and did not depend on the version or the profile, so the generator (and the CLI
//! `schema` command) was removed. Use the documented `BuildRequest` types instead.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

mod validation;

pub use validation::{
    SchemaValidator, ValidationConfig as SchemaValidationConfig,
    ValidationResult as SchemaValidationResult,
};

/// Message profile type (moved here from `presets` in 0.6.0; also exported at the crate root)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageProfile {
    /// Audio album release
    AudioAlbum,
    /// Audio single release
    AudioSingle,
    /// Video album release
    VideoAlbum,
    /// Video single release
    VideoSingle,
    /// Mixed content release
    Mixed,
}

/// Complete JSON Schema representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    /// Schema metadata
    #[serde(rename = "$schema")]
    pub schema: String,
    /// Schema ID/URI
    #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Schema title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Schema description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Schema type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub schema_type: Option<String>,
    /// Object properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<IndexMap<String, JsonSchema>>,
    /// Required properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    /// Additional properties allowed
    #[serde(
        rename = "additionalProperties",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_properties: Option<bool>,
    /// Array items schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<JsonSchema>>,
    /// Enum values
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<JsonValue>>,
    /// String pattern validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    /// String format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// Minimum length
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    /// Maximum length
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    /// Examples
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<JsonValue>>,
    /// Schema definitions
    #[serde(rename = "$defs", skip_serializing_if = "Option::is_none")]
    pub definitions: Option<IndexMap<String, JsonSchema>>,
    /// Reference to another schema
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    /// All of (intersection)
    #[serde(rename = "allOf", skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<JsonSchema>>,
    /// Any of (union)
    #[serde(rename = "anyOf", skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<JsonSchema>>,
    /// One of (exclusive union)
    #[serde(rename = "oneOf", skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<JsonSchema>>,
    /// Conditional schema
    #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
    pub if_schema: Option<Box<JsonSchema>>,
    /// Then schema (if condition is true)
    #[serde(rename = "then", skip_serializing_if = "Option::is_none")]
    pub then_schema: Option<Box<JsonSchema>>,
    /// Else schema (if condition is false)
    #[serde(rename = "else", skip_serializing_if = "Option::is_none")]
    pub else_schema: Option<Box<JsonSchema>>,
    /// Custom annotations
    #[serde(flatten)]
    pub annotations: IndexMap<String, JsonValue>,
}

impl Default for JsonSchema {
    fn default() -> Self {
        Self {
            schema: String::new(),
            id: None,
            title: None,
            description: None,
            schema_type: None,
            properties: None,
            required: None,
            additional_properties: None,
            items: None,
            enum_values: None,
            pattern: None,
            format: None,
            min_length: None,
            max_length: None,
            examples: None,
            definitions: None,
            reference: None,
            all_of: None,
            any_of: None,
            one_of: None,
            if_schema: None,
            then_schema: None,
            else_schema: None,
            annotations: IndexMap::new(),
        }
    }
}
