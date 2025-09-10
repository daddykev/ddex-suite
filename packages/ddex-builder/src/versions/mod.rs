//! Multi-version DDEX support and conversion
//!
//! This module provides comprehensive support for multiple DDEX ERN versions
//! including 3.8.2, 4.2, and 4.3 with automatic conversion capabilities.
//!
//! # Supported Versions
//! 
//! - **ERN 3.8.2**: Legacy version with different namespaces and element structures
//! - **ERN 4.2**: Intermediate version with some modern features
//! - **ERN 4.3**: Current recommended version with full feature set
//!
//! # Version Conversion
//!
//! The system supports both upgrade and downgrade paths:
//! - Upgrade: 3.8.2 → 4.2 → 4.3 (with feature enhancement)
//! - Downgrade: 4.3 → 4.2 → 3.8.2 (with compatibility warnings)
//!
//! # Examples
//!
//! ```rust
//! use ddex_builder::versions::{VersionConverter, DdexVersion};
//! 
//! let converter = VersionConverter::new();
//! let result = converter.convert_version(ddex_xml, DdexVersion::Ern382, DdexVersion::Ern43)?;
//! ```

use crate::error::BuildError;
use crate::presets::DdexVersion;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

mod ern_382;
mod ern_42;
mod ern_43;
mod converter;

pub use ern_382::*;
pub use ern_42::*;
pub use ern_43::*;
pub use converter::{VersionConverter, ConversionResult as ConverterResult, ConversionReport as ConverterReport, ConversionWarning as ConverterWarning, ConversionWarningType};

/// Version-specific DDEX metadata and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSpec {
    /// Version identifier
    pub version: DdexVersion,
    /// XML namespace URI
    pub namespace: String,
    /// Schema location hint
    pub schema_location: Option<String>,
    /// Message schema version ID
    pub message_schema_version_id: String,
    /// Supported message types
    pub supported_message_types: Vec<String>,
    /// Version-specific element mappings
    pub element_mappings: IndexMap<String, String>,
    /// Required elements for this version
    pub required_elements: Vec<String>,
    /// Deprecated elements (for downgrades)
    pub deprecated_elements: Vec<String>,
    /// New elements (not in older versions)
    pub new_elements: Vec<String>,
    /// Namespace prefix mappings
    pub namespace_prefixes: IndexMap<String, String>,
}

/// Version conversion result with detailed reporting
#[derive(Debug, Clone)]
pub struct ConversionResult {
    /// Converted XML content
    pub converted_xml: String,
    /// Source version
    pub source_version: DdexVersion,
    /// Target version
    pub target_version: DdexVersion,
    /// Conversion report
    pub report: ConversionReport,
    /// Conversion metadata
    pub metadata: ConversionMetadata,
}

/// Detailed conversion report
#[derive(Debug, Clone)]
pub struct ConversionReport {
    /// Successful conversions
    pub conversions: Vec<ElementConversion>,
    /// Warnings generated during conversion
    pub warnings: Vec<ConversionWarning>,
    /// Errors encountered (non-fatal)
    pub errors: Vec<ConversionError>,
    /// Elements that couldn't be converted
    pub unconvertible_elements: Vec<String>,
    /// Data loss warnings
    pub data_loss_warnings: Vec<String>,
    /// Feature compatibility notes
    pub compatibility_notes: Vec<String>,
}

/// Individual element conversion record
#[derive(Debug, Clone)]
pub struct ElementConversion {
    /// Original element path
    pub source_path: String,
    /// Converted element path
    pub target_path: String,
    /// Conversion type performed
    pub conversion_type: ConversionType,
    /// Additional notes
    pub notes: Option<String>,
}

/// Type of conversion performed
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionType {
    /// Direct mapping (same element name)
    DirectMapping,
    /// Element renamed
    Renamed { old_name: String, new_name: String },
    /// Element structure changed
    Restructured { description: String },
    /// New element added (upgrade)
    Added { default_value: Option<String> },
    /// Element removed (downgrade)
    Removed { reason: String },
    /// Element moved to different location
    Moved { old_path: String, new_path: String },
    /// Complex transformation
    Transformed { description: String },
}

/// Conversion warning
#[derive(Debug, Clone)]
pub struct ConversionWarning {
    /// Warning code
    pub code: String,
    /// Warning message
    pub message: String,
    /// Element path that caused warning
    pub element_path: Option<String>,
    /// Suggested action
    pub suggestion: Option<String>,
    /// Impact level
    pub impact: ImpactLevel,
}

/// Conversion error (non-fatal)
#[derive(Debug, Clone)]
pub struct ConversionError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Element path that caused error
    pub element_path: String,
    /// Fallback action taken
    pub fallback: Option<String>,
}

/// Impact level of warnings/changes
#[derive(Debug, Clone, PartialEq)]
pub enum ImpactLevel {
    /// Low impact, cosmetic changes
    Low,
    /// Medium impact, functional changes
    Medium,
    /// High impact, potential data loss
    High,
    /// Critical impact, breaking changes
    Critical,
}

/// Conversion metadata
#[derive(Debug, Clone)]
pub struct ConversionMetadata {
    /// Conversion timestamp
    pub converted_at: chrono::DateTime<chrono::Utc>,
    /// Conversion duration
    pub conversion_time: std::time::Duration,
    /// Number of elements processed
    pub elements_processed: usize,
    /// Number of warnings generated
    pub warning_count: usize,
    /// Number of errors encountered
    pub error_count: usize,
    /// Estimated fidelity percentage
    pub fidelity_percentage: f64,
}

/// Version detection result
#[derive(Debug, Clone)]
pub struct VersionDetection {
    /// Detected version
    pub detected_version: DdexVersion,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
    /// Detection clues found
    pub clues: Vec<DetectionClue>,
    /// Ambiguities encountered
    pub ambiguities: Vec<String>,
}

/// Clue used for version detection
#[derive(Debug, Clone)]
pub struct DetectionClue {
    /// Type of clue
    pub clue_type: ClueType,
    /// Evidence found
    pub evidence: String,
    /// Confidence weight
    pub weight: f64,
}

/// Type of version detection clue
#[derive(Debug, Clone)]
pub enum ClueType {
    /// XML namespace
    Namespace,
    /// Schema location
    SchemaLocation,
    /// Message schema version ID
    MessageSchemaVersionId,
    /// Presence of version-specific elements
    VersionSpecificElement,
    /// Element structure patterns
    StructuralPattern,
    /// Namespace prefix usage
    NamespacePrefix,
}

/// Version compatibility matrix
#[derive(Debug, Clone)]
pub struct CompatibilityMatrix {
    /// Supported conversion paths
    pub conversion_paths: Vec<ConversionPath>,
    /// Feature compatibility table
    pub feature_compatibility: IndexMap<String, FeatureSupport>,
    /// Recommended conversion strategies
    pub recommended_strategies: Vec<ConversionStrategy>,
}

/// Single conversion path between versions
#[derive(Debug, Clone)]
pub struct ConversionPath {
    /// Source version
    pub from: DdexVersion,
    /// Target version
    pub to: DdexVersion,
    /// Conversion difficulty
    pub difficulty: ConversionDifficulty,
    /// Expected fidelity
    pub fidelity: f64,
    /// Major changes involved
    pub major_changes: Vec<String>,
    /// Recommended for production use
    pub production_ready: bool,
}

/// Difficulty level of conversion
#[derive(Debug, Clone, PartialEq)]
pub enum ConversionDifficulty {
    /// Simple mapping changes
    Trivial,
    /// Moderate structural changes
    Moderate,
    /// Complex transformations required
    Complex,
    /// Significant data model changes
    Challenging,
}

/// Feature support across versions
#[derive(Debug, Clone)]
pub struct FeatureSupport {
    /// Feature name
    pub feature: String,
    /// Support in ERN 3.8.2
    pub ern_382: SupportLevel,
    /// Support in ERN 4.2
    pub ern_42: SupportLevel,
    /// Support in ERN 4.3
    pub ern_43: SupportLevel,
    /// Migration notes
    pub migration_notes: Option<String>,
}

/// Level of feature support
#[derive(Debug, Clone, PartialEq)]
pub enum SupportLevel {
    /// Fully supported
    Full,
    /// Partially supported
    Partial,
    /// Not supported
    None,
    /// Deprecated
    Deprecated,
    /// New in this version
    New,
}

/// Conversion strategy recommendation
#[derive(Debug, Clone)]
pub struct ConversionStrategy {
    /// Strategy name
    pub name: String,
    /// Description
    pub description: String,
    /// Applicable scenarios
    pub scenarios: Vec<String>,
    /// Steps involved
    pub steps: Vec<String>,
    /// Expected outcomes
    pub outcomes: Vec<String>,
    /// Risk level
    pub risk_level: RiskLevel,
}

/// Risk level of conversion strategy
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    /// Low risk, safe for production
    Low,
    /// Medium risk, testing recommended
    Medium,
    /// High risk, careful validation needed
    High,
    /// Very high risk, not recommended
    VeryHigh,
}

/// Main version management interface
#[derive(Debug, Clone)]
pub struct VersionManager {
    /// Available version specifications
    version_specs: IndexMap<DdexVersion, VersionSpec>,
    /// Compatibility matrix
    compatibility: CompatibilityMatrix,
    /// Default conversion options
    default_options: ConversionOptions,
}

/// Options for version conversion
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// Allow lossy conversions
    pub allow_lossy: bool,
    /// Generate detailed reports
    pub detailed_reports: bool,
    /// Preserve unknown elements
    pub preserve_unknown: bool,
    /// Add conversion metadata
    pub add_metadata: bool,
    /// Preserve XML comments during conversion
    pub preserve_comments: bool,
    /// Validation level after conversion
    pub validation_level: ValidationLevel,
    /// Custom element mappings
    pub custom_mappings: IndexMap<String, String>,
}

/// Validation level for converted content
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationLevel {
    /// No validation
    None,
    /// Basic structure validation
    Basic,
    /// Schema validation
    Schema,
    /// Full semantic validation
    Full,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            allow_lossy: false,
            detailed_reports: true,
            preserve_unknown: false,
            add_metadata: true,
            preserve_comments: false,
            validation_level: ValidationLevel::Schema,
            custom_mappings: IndexMap::new(),
        }
    }
}

impl VersionManager {
    /// Create a new version manager with default specifications
    pub fn new() -> Self {
        Self {
            version_specs: Self::load_default_specs(),
            compatibility: Self::build_compatibility_matrix(),
            default_options: ConversionOptions::default(),
        }
    }
    
    /// Get version specification
    pub fn get_version_spec(&self, version: DdexVersion) -> Option<&VersionSpec> {
        self.version_specs.get(&version)
    }
    
    /// Detect version from XML content
    pub fn detect_version(&self, xml_content: &str) -> Result<VersionDetection, BuildError> {
        let mut clues = Vec::new();
        let mut version_scores = IndexMap::new();
        
        // Initialize scores
        for version in [DdexVersion::Ern382, DdexVersion::Ern42, DdexVersion::Ern43] {
            version_scores.insert(version, 0.0);
        }
        
        // Analyze namespace
        if let Some(namespace) = self.extract_namespace(xml_content) {
            clues.push(DetectionClue {
                clue_type: ClueType::Namespace,
                evidence: namespace.clone(),
                weight: 0.8,
            });
            
            // Score based on namespace
            for (version, spec) in &self.version_specs {
                if spec.namespace == namespace {
                    *version_scores.get_mut(version).unwrap() += 0.8;
                }
            }
        }
        
        // Analyze message schema version ID
        if let Some(schema_version) = self.extract_message_schema_version(xml_content) {
            clues.push(DetectionClue {
                clue_type: ClueType::MessageSchemaVersionId,
                evidence: schema_version.clone(),
                weight: 0.9,
            });
            
            // Score based on schema version
            for (version, spec) in &self.version_specs {
                if spec.message_schema_version_id == schema_version {
                    *version_scores.get_mut(version).unwrap() += 0.9;
                }
            }
        }
        
        // Look for version-specific elements
        for (version, spec) in &self.version_specs {
            for element in &spec.new_elements {
                if xml_content.contains(&format!("<{}", element)) {
                    clues.push(DetectionClue {
                        clue_type: ClueType::VersionSpecificElement,
                        evidence: element.clone(),
                        weight: 0.6,
                    });
                    *version_scores.get_mut(version).unwrap() += 0.6;
                }
            }
        }
        
        // Determine best match
        let (detected_version, confidence) = version_scores
            .into_iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        
        let normalized_confidence = (confidence / 2.5_f64).min(1.0_f64); // Normalize to 0-1
        
        Ok(VersionDetection {
            detected_version,
            confidence: normalized_confidence,
            clues,
            ambiguities: Vec::new(), // TODO: Implement ambiguity detection
        })
    }
    
    /// Check if conversion is supported between versions
    pub fn is_conversion_supported(&self, from: DdexVersion, to: DdexVersion) -> bool {
        self.compatibility.conversion_paths.iter()
            .any(|path| path.from == from && path.to == to)
    }
    
    /// Get conversion path information
    pub fn get_conversion_path(&self, from: DdexVersion, to: DdexVersion) -> Option<&ConversionPath> {
        self.compatibility.conversion_paths.iter()
            .find(|path| path.from == from && path.to == to)
    }
    
    /// Get feature compatibility information
    pub fn get_feature_compatibility(&self, feature: &str) -> Option<&FeatureSupport> {
        self.compatibility.feature_compatibility.get(feature)
    }
    
    /// Get recommended conversion strategy
    pub fn get_recommended_strategy(&self, from: DdexVersion, to: DdexVersion) -> Option<&ConversionStrategy> {
        let scenario = format!("{:?} to {:?}", from, to);
        self.compatibility.recommended_strategies.iter()
            .find(|strategy| strategy.scenarios.contains(&scenario))
    }
    
    // Private helper methods
    
    fn load_default_specs() -> IndexMap<DdexVersion, VersionSpec> {
        let mut specs = IndexMap::new();
        
        specs.insert(DdexVersion::Ern382, ern_382::get_version_spec());
        specs.insert(DdexVersion::Ern42, ern_42::get_version_spec());
        specs.insert(DdexVersion::Ern43, ern_43::get_version_spec());
        
        specs
    }
    
    fn build_compatibility_matrix() -> CompatibilityMatrix {
        let conversion_paths = vec![
            // Upgrade paths
            ConversionPath {
                from: DdexVersion::Ern382,
                to: DdexVersion::Ern42,
                difficulty: ConversionDifficulty::Moderate,
                fidelity: 0.85,
                major_changes: vec![
                    "Namespace migration".to_string(),
                    "Element structure updates".to_string(),
                    "New optional elements".to_string(),
                ],
                production_ready: true,
            },
            ConversionPath {
                from: DdexVersion::Ern42,
                to: DdexVersion::Ern43,
                difficulty: ConversionDifficulty::Trivial,
                fidelity: 0.95,
                major_changes: vec![
                    "Minor element additions".to_string(),
                    "Enhanced validation rules".to_string(),
                ],
                production_ready: true,
            },
            ConversionPath {
                from: DdexVersion::Ern382,
                to: DdexVersion::Ern43,
                difficulty: ConversionDifficulty::Complex,
                fidelity: 0.80,
                major_changes: vec![
                    "Major namespace changes".to_string(),
                    "Significant structural updates".to_string(),
                    "New required elements".to_string(),
                ],
                production_ready: true,
            },
            // Downgrade paths
            ConversionPath {
                from: DdexVersion::Ern43,
                to: DdexVersion::Ern42,
                difficulty: ConversionDifficulty::Moderate,
                fidelity: 0.90,
                major_changes: vec![
                    "Remove newer elements".to_string(),
                    "Downgrade validation rules".to_string(),
                ],
                production_ready: true,
            },
            ConversionPath {
                from: DdexVersion::Ern42,
                to: DdexVersion::Ern382,
                difficulty: ConversionDifficulty::Challenging,
                fidelity: 0.75,
                major_changes: vec![
                    "Legacy namespace mapping".to_string(),
                    "Remove modern elements".to_string(),
                    "Structural downgrade".to_string(),
                ],
                production_ready: false,
            },
            ConversionPath {
                from: DdexVersion::Ern43,
                to: DdexVersion::Ern382,
                difficulty: ConversionDifficulty::Challenging,
                fidelity: 0.70,
                major_changes: vec![
                    "Major structural downgrade".to_string(),
                    "Significant feature removal".to_string(),
                    "Legacy compatibility layer".to_string(),
                ],
                production_ready: false,
            },
        ];
        
        let feature_compatibility = Self::build_feature_compatibility();
        let recommended_strategies = Self::build_recommended_strategies();
        
        CompatibilityMatrix {
            conversion_paths,
            feature_compatibility,
            recommended_strategies,
        }
    }
    
    fn build_feature_compatibility() -> IndexMap<String, FeatureSupport> {
        let mut features = IndexMap::new();
        
        features.insert("ResourceReference".to_string(), FeatureSupport {
            feature: "Resource Reference Elements".to_string(),
            ern_382: SupportLevel::Partial,
            ern_42: SupportLevel::Full,
            ern_43: SupportLevel::Full,
            migration_notes: Some("Enhanced in 4.2 with better linking".to_string()),
        });
        
        features.insert("DetailedDealTerms".to_string(), FeatureSupport {
            feature: "Detailed Deal Terms".to_string(),
            ern_382: SupportLevel::None,
            ern_42: SupportLevel::Partial,
            ern_43: SupportLevel::Full,
            migration_notes: Some("New detailed terms structure in 4.2+".to_string()),
        });
        
        features.insert("EnhancedMetadata".to_string(), FeatureSupport {
            feature: "Enhanced Metadata Fields".to_string(),
            ern_382: SupportLevel::None,
            ern_42: SupportLevel::None,
            ern_43: SupportLevel::New,
            migration_notes: Some("Completely new in 4.3".to_string()),
        });
        
        features.insert("DeprecatedElements".to_string(), FeatureSupport {
            feature: "Legacy Deprecated Elements".to_string(),
            ern_382: SupportLevel::Full,
            ern_42: SupportLevel::Deprecated,
            ern_43: SupportLevel::None,
            migration_notes: Some("Removed in 4.3, use modern equivalents".to_string()),
        });
        
        features
    }
    
    fn build_recommended_strategies() -> Vec<ConversionStrategy> {
        vec![
            ConversionStrategy {
                name: "Conservative Upgrade".to_string(),
                description: "Step-by-step version upgrade with validation".to_string(),
                scenarios: vec!["Ern382 to Ern43".to_string()],
                steps: vec![
                    "Validate source ERN 3.8.2 message".to_string(),
                    "Convert 3.8.2 → 4.2 with warnings".to_string(),
                    "Validate intermediate 4.2 message".to_string(),
                    "Convert 4.2 → 4.3 with enhancements".to_string(),
                    "Final validation and report".to_string(),
                ],
                outcomes: vec![
                    "High-fidelity conversion".to_string(),
                    "Detailed conversion report".to_string(),
                    "Step-by-step validation".to_string(),
                ],
                risk_level: RiskLevel::Low,
            },
            ConversionStrategy {
                name: "Direct Upgrade".to_string(),
                description: "Direct conversion between versions".to_string(),
                scenarios: vec!["Ern42 to Ern43".to_string()],
                steps: vec![
                    "Validate source message".to_string(),
                    "Apply direct conversion mappings".to_string(),
                    "Add new optional elements".to_string(),
                    "Validate target message".to_string(),
                ],
                outcomes: vec![
                    "Fast conversion".to_string(),
                    "Minimal data transformation".to_string(),
                ],
                risk_level: RiskLevel::Low,
            },
        ]
    }
    
    fn extract_namespace(&self, xml_content: &str) -> Option<String> {
        // Simple regex to extract namespace from XML
        let re = regex::Regex::new(r#"xmlns="([^"]+)""#).ok()?;
        re.captures(xml_content)?
            .get(1)
            .map(|m| m.as_str().to_string())
    }
    
    fn extract_message_schema_version(&self, xml_content: &str) -> Option<String> {
        let re = regex::Regex::new(r#"MessageSchemaVersionId="([^"]+)""#).ok()?;
        re.captures(xml_content)?
            .get(1)
            .map(|m| m.as_str().to_string())
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}


impl std::fmt::Display for ConversionDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionDifficulty::Trivial => write!(f, "Trivial"),
            ConversionDifficulty::Moderate => write!(f, "Moderate"),
            ConversionDifficulty::Complex => write!(f, "Complex"),
            ConversionDifficulty::Challenging => write!(f, "Challenging"),
        }
    }
}

impl std::fmt::Display for SupportLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportLevel::Full => write!(f, "Full"),
            SupportLevel::Partial => write!(f, "Partial"),
            SupportLevel::None => write!(f, "None"),
            SupportLevel::Deprecated => write!(f, "Deprecated"),
            SupportLevel::New => write!(f, "New"),
        }
    }
}

impl std::fmt::Display for ImpactLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImpactLevel::Low => write!(f, "Low"),
            ImpactLevel::Medium => write!(f, "Medium"),
            ImpactLevel::High => write!(f, "High"),
            ImpactLevel::Critical => write!(f, "Critical"),
        }
    }
}

/// Utility functions for version handling
pub mod utils {
    use super::*;
    
    /// Get all supported versions
    pub fn supported_versions() -> Vec<DdexVersion> {
        vec![DdexVersion::Ern382, DdexVersion::Ern42, DdexVersion::Ern43]
    }
    
    /// Check if version is legacy
    pub fn is_legacy_version(version: DdexVersion) -> bool {
        matches!(version, DdexVersion::Ern382)
    }
    
    /// Check if version is modern
    pub fn is_modern_version(version: DdexVersion) -> bool {
        matches!(version, DdexVersion::Ern43)
    }
    
    /// Get version release date
    pub fn get_version_release_date(version: DdexVersion) -> chrono::NaiveDate {
        match version {
            DdexVersion::Ern382 => chrono::NaiveDate::from_ymd_opt(2018, 5, 1).unwrap(),
            DdexVersion::Ern42 => chrono::NaiveDate::from_ymd_opt(2020, 8, 15).unwrap(),
            DdexVersion::Ern43 => chrono::NaiveDate::from_ymd_opt(2023, 3, 1).unwrap(),
            DdexVersion::Ern41 => chrono::NaiveDate::from_ymd_opt(2019, 11, 15).unwrap(),
        }
    }
    
    /// Get version description
    pub fn get_version_description(version: DdexVersion) -> String {
        match version {
            DdexVersion::Ern382 => "Legacy version with basic features".to_string(),
            DdexVersion::Ern42 => "Intermediate version with enhanced features".to_string(),
            DdexVersion::Ern43 => "Current version with full feature set".to_string(),
            DdexVersion::Ern41 => "Early 4.x version".to_string(),
        }
    }
}