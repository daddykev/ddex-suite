//! Perfect Fidelity Engine for DDEX Builder
//!
//! This module provides comprehensive fidelity preservation features for the DDEX Builder,
//! ensuring perfect round-trip compatibility with the DDEX Parser and maintaining
//! byte-perfect XML reproduction when required.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Fidelity configuration for perfect XML preservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityConfig {
    /// Preservation level for different XML components
    pub preservation_level: PreservationLevel,
    /// Comment preservation settings
    pub comment_preservation: CommentPreservationConfig,
    /// Processing instruction preservation
    pub processing_instruction_preservation: bool,
    /// Extension element preservation settings
    pub extension_preservation: ExtensionPreservationConfig,
    /// Attribute preservation settings
    pub attribute_preservation: AttributePreservationConfig,
    /// Namespace preservation settings
    pub namespace_preservation: NamespacePreservationConfig,
    /// Whitespace preservation settings
    pub whitespace_preservation: WhitespacePreservationConfig,
}

impl Default for FidelityConfig {
    fn default() -> Self {
        Self {
            preservation_level: PreservationLevel::Standard,
            comment_preservation: CommentPreservationConfig::default(),
            processing_instruction_preservation: false,
            extension_preservation: ExtensionPreservationConfig::default(),
            attribute_preservation: AttributePreservationConfig::default(),
            namespace_preservation: NamespacePreservationConfig::default(),
            whitespace_preservation: WhitespacePreservationConfig::default(),
        }
    }
}

/// Preservation levels for different use cases
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreservationLevel {
    /// Basic preservation (DDEX elements only)
    Basic,
    /// Standard preservation (DDEX + known extensions)
    Standard,
    /// Perfect preservation (everything including comments, whitespace)
    Perfect,
    /// Custom preservation with fine-grained control
    Custom,
}

/// Comment preservation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentPreservationConfig {
    /// Whether to preserve comments at all
    pub enabled: bool,
    /// Preserve document-level comments (outside root element)
    pub preserve_document_comments: bool,
    /// Preserve element-level comments (between elements)
    pub preserve_element_comments: bool,
    /// Preserve inline comments (within elements)
    pub preserve_inline_comments: bool,
    /// Preserve comment positioning relative to elements
    pub preserve_comment_positioning: bool,
    /// Maximum comment length to preserve (0 = unlimited)
    pub max_comment_length: usize,
}

impl Default for CommentPreservationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preserve_document_comments: true,
            preserve_element_comments: true,
            preserve_inline_comments: false,
            preserve_comment_positioning: true,
            max_comment_length: 0,
        }
    }
}

/// Extension preservation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPreservationConfig {
    /// Whether to preserve extension elements
    pub enabled: bool,
    /// Known extension namespaces to always preserve
    pub known_extensions: Vec<String>,
    /// Unknown extension handling
    pub unknown_extension_handling: UnknownExtensionHandling,
    /// Preserve extension attributes
    pub preserve_extension_attributes: bool,
    /// Extension validation rules
    pub extension_validation: ExtensionValidationConfig,
}

impl Default for ExtensionPreservationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            known_extensions: vec![
                "http://spotify.com/ddex".to_string(),
                "http://apple.com/ddex".to_string(),
                "http://youtube.com/ddex".to_string(),
                "http://amazon.com/ddex".to_string(),
            ],
            unknown_extension_handling: UnknownExtensionHandling::Preserve,
            preserve_extension_attributes: true,
            extension_validation: ExtensionValidationConfig::default(),
        }
    }
}

/// How to handle unknown extensions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnknownExtensionHandling {
    /// Preserve unknown extensions as-is
    Preserve,
    /// Drop unknown extensions
    Drop,
    /// Validate against schema and preserve if valid
    ValidateAndPreserve,
    /// Convert to generic extension format
    Generalize,
}

/// Extension validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionValidationConfig {
    /// Validate extension URIs
    pub validate_uris: bool,
    /// Validate extension schema compliance
    pub validate_schema: bool,
    /// Maximum extension nesting depth
    pub max_nesting_depth: usize,
    /// Maximum extension element count per document
    pub max_extension_count: usize,
}

impl Default for ExtensionValidationConfig {
    fn default() -> Self {
        Self {
            validate_uris: true,
            validate_schema: false,
            max_nesting_depth: 10,
            max_extension_count: 1000,
        }
    }
}

/// Attribute preservation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributePreservationConfig {
    /// Preserve original attribute ordering
    pub preserve_ordering: bool,
    /// Preserve unknown attributes
    pub preserve_unknown_attributes: bool,
    /// Preserve empty attributes
    pub preserve_empty_attributes: bool,
    /// Attribute normalization rules
    pub normalization: AttributeNormalizationConfig,
    /// Attribute validation rules
    pub validation: AttributeValidationConfig,
}

impl Default for AttributePreservationConfig {
    fn default() -> Self {
        Self {
            preserve_ordering: false,
            preserve_unknown_attributes: true,
            preserve_empty_attributes: false,
            normalization: AttributeNormalizationConfig::default(),
            validation: AttributeValidationConfig::default(),
        }
    }
}

/// Attribute normalization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeNormalizationConfig {
    /// Normalize boolean attributes (true/false vs 1/0)
    pub normalize_booleans: bool,
    /// Normalize numeric formats
    pub normalize_numbers: bool,
    /// Normalize date/time formats
    pub normalize_datetime: bool,
    /// Normalize whitespace in attribute values
    pub normalize_whitespace: bool,
}

impl Default for AttributeNormalizationConfig {
    fn default() -> Self {
        Self {
            normalize_booleans: true,
            normalize_numbers: true,
            normalize_datetime: true,
            normalize_whitespace: true,
        }
    }
}

/// Attribute validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeValidationConfig {
    /// Validate attribute value formats
    pub validate_formats: bool,
    /// Validate attribute value constraints
    pub validate_constraints: bool,
    /// Custom validation rules per attribute
    pub custom_rules: HashMap<String, AttributeValidationRule>,
}

impl Default for AttributeValidationConfig {
    fn default() -> Self {
        Self {
            validate_formats: true,
            validate_constraints: true,
            custom_rules: HashMap::new(),
        }
    }
}

/// Custom attribute validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeValidationRule {
    /// Regular expression pattern for validation
    pub pattern: Option<String>,
    /// Minimum value (for numeric attributes)
    pub min_value: Option<f64>,
    /// Maximum value (for numeric attributes)
    pub max_value: Option<f64>,
    /// Allowed values (enum validation)
    pub allowed_values: Option<Vec<String>>,
    /// Custom validation message
    pub validation_message: Option<String>,
}

/// Namespace preservation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespacePreservationConfig {
    /// Preserve original namespace prefixes
    pub preserve_prefixes: bool,
    /// Preserve unused namespace declarations
    pub preserve_unused_declarations: bool,
    /// Namespace minimization strategy
    pub minimization_strategy: NamespaceMinimizationStrategy,
    /// Default namespace handling
    pub default_namespace_handling: DefaultNamespaceHandling,
}

impl Default for NamespacePreservationConfig {
    fn default() -> Self {
        Self {
            preserve_prefixes: false,
            preserve_unused_declarations: false,
            minimization_strategy: NamespaceMinimizationStrategy::Aggressive,
            default_namespace_handling: DefaultNamespaceHandling::Preserve,
        }
    }
}

/// Namespace minimization strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamespaceMinimizationStrategy {
    /// No minimization (preserve all declarations)
    None,
    /// Conservative minimization (remove only clearly unused)
    Conservative,
    /// Aggressive minimization (minimize to essential declarations)
    Aggressive,
    /// Custom minimization based on rules
    Custom(NamespaceMinimizationRules),
}

/// Custom namespace minimization rules
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamespaceMinimizationRules {
    /// Namespaces to always preserve
    pub always_preserve: Vec<String>,
    /// Namespaces that can be safely removed if unused
    pub can_remove: Vec<String>,
    /// Preferred prefix mappings
    pub preferred_prefixes: HashMap<String, String>,
}

/// Default namespace handling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefaultNamespaceHandling {
    /// Preserve default namespace as-is
    Preserve,
    /// Remove default namespace if possible
    Remove,
    /// Convert to explicit prefix
    ConvertToPrefix,
}

/// Whitespace preservation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitespacePreservationConfig {
    /// Preserve significant whitespace
    pub preserve_significant: bool,
    /// Preserve formatting whitespace (indentation, etc.)
    pub preserve_formatting: bool,
    /// Normalize line endings
    pub normalize_line_endings: bool,
    /// Target line ending style
    pub line_ending_style: LineEndingStyle,
    /// Indentation style for formatted output
    pub indentation_style: IndentationStyle,
}

impl Default for WhitespacePreservationConfig {
    fn default() -> Self {
        Self {
            preserve_significant: true,
            preserve_formatting: false,
            normalize_line_endings: true,
            line_ending_style: LineEndingStyle::Unix,
            indentation_style: IndentationStyle::Spaces(2),
        }
    }
}

/// Line ending styles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineEndingStyle {
    /// Unix style (\n)
    Unix,
    /// Windows style (\r\n)
    Windows,
    /// Classic Mac style (\r)
    Mac,
    /// Preserve original
    Preserve,
}

/// Indentation styles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndentationStyle {
    /// Spaces with specified count
    Spaces(usize),
    /// Tabs
    Tabs,
    /// No indentation
    None,
}

/// Fidelity statistics collected during processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityStatistics {
    /// Elements preserved by category
    pub elements_preserved: HashMap<String, usize>,
    /// Attributes preserved by category
    pub attributes_preserved: HashMap<String, usize>,
    /// Comments preserved
    pub comments_preserved: usize,
    /// Processing instructions preserved
    pub processing_instructions_preserved: usize,
    /// Extensions preserved by namespace
    pub extensions_preserved: HashMap<String, usize>,
    /// Namespaces processed
    pub namespaces_processed: usize,
    /// Fidelity processing time
    pub processing_time: Duration,
    /// Memory usage for fidelity features
    pub memory_usage: usize,
}

impl Default for FidelityStatistics {
    fn default() -> Self {
        Self {
            elements_preserved: HashMap::new(),
            attributes_preserved: HashMap::new(),
            comments_preserved: 0,
            processing_instructions_preserved: 0,
            extensions_preserved: HashMap::new(),
            namespaces_processed: 0,
            processing_time: Duration::ZERO,
            memory_usage: 0,
        }
    }
}

/// Fidelity processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityResult {
    /// Whether fidelity processing was successful
    pub success: bool,
    /// Fidelity level achieved
    pub achieved_level: PreservationLevel,
    /// Processing statistics
    pub statistics: FidelityStatistics,
    /// Issues encountered during processing
    pub issues: Vec<FidelityIssue>,
    /// Recommendations for improving fidelity
    pub recommendations: Vec<String>,
}

/// Fidelity processing issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityIssue {
    /// Issue severity
    pub severity: FidelitySeverity,
    /// Issue category
    pub category: String,
    /// Issue description
    pub message: String,
    /// XML path where issue occurred
    pub path: Option<String>,
    /// Suggested resolution
    pub suggestion: Option<String>,
}

/// Fidelity issue severity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FidelitySeverity {
    /// Critical issue that prevents fidelity
    Critical,
    /// Major issue that significantly impacts fidelity
    Major,
    /// Minor issue with minimal fidelity impact
    Minor,
    /// Informational message
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fidelity_config_default() {
        let config = FidelityConfig::default();
        assert_eq!(config.preservation_level, PreservationLevel::Standard);
        assert!(!config.comment_preservation.enabled);
        assert!(!config.processing_instruction_preservation);
        assert!(config.extension_preservation.enabled);
    }

    #[test]
    fn test_preservation_levels() {
        let basic = PreservationLevel::Basic;
        let standard = PreservationLevel::Standard;
        let perfect = PreservationLevel::Perfect;
        let custom = PreservationLevel::Custom;

        assert_ne!(basic, standard);
        assert_ne!(standard, perfect);
        assert_ne!(perfect, custom);
    }

    #[test]
    fn test_comment_preservation_config() {
        let mut config = CommentPreservationConfig::default();
        assert!(!config.enabled);

        config.enabled = true;
        assert!(config.enabled);
        assert!(config.preserve_document_comments);
        assert!(config.preserve_element_comments);
        assert!(!config.preserve_inline_comments);
    }

    #[test]
    fn test_extension_preservation() {
        let config = ExtensionPreservationConfig::default();
        assert!(config.enabled);
        assert!(!config.known_extensions.is_empty());
        assert_eq!(config.unknown_extension_handling, UnknownExtensionHandling::Preserve);
    }

    #[test]
    fn test_namespace_minimization() {
        let strategy = NamespaceMinimizationStrategy::Aggressive;
        match strategy {
            NamespaceMinimizationStrategy::Aggressive => {},
            _ => panic!("Expected Aggressive strategy"),
        }
    }
}