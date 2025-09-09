//! Determinism configuration and enforcement
//! 
//! CRITICAL: This module ensures deterministic output by using IndexMap
//! everywhere instead of HashMap/HashSet.

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

/// Determinism configuration for XML generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismConfig {
    /// Canonicalization mode
    pub canon_mode: CanonMode,
    
    /// Element ordering strategy
    pub sort_strategy: SortStrategy,
    
    /// Custom sort order (uses IndexMap for determinism)
    pub custom_sort_order: Option<IndexMap<String, Vec<String>>>,
    
    /// Namespace handling
    pub namespace_strategy: NamespaceStrategy,
    
    /// Locked namespace prefixes (uses IndexMap for determinism)
    pub locked_prefixes: IndexMap<String, String>,
    
    /// Formatting options
    pub output_mode: OutputMode,
    pub line_ending: LineEnding,
    pub indent_char: IndentChar,
    pub indent_width: usize,
    
    /// String normalization
    pub unicode_normalization: UnicodeNormalization,
    pub xml_character_policy: XmlCharacterPolicy,
    pub quote_style: QuoteStyle,
    
    /// Date/Time handling
    pub time_zone_policy: TimeZonePolicy,
    pub date_time_format: DateTimeFormat,
    
    /// Reproducibility options
    pub emit_reproducibility_banner: bool,
    pub verify_determinism: Option<usize>,
}

impl Default for DeterminismConfig {
    fn default() -> Self {
        Self {
            canon_mode: CanonMode::DbC14n,
            sort_strategy: SortStrategy::Canonical,
            custom_sort_order: None,
            namespace_strategy: NamespaceStrategy::Locked,
            locked_prefixes: Self::default_namespace_prefixes(),
            output_mode: OutputMode::DbC14n,
            line_ending: LineEnding::LF,
            indent_char: IndentChar::Space,
            indent_width: 2,
            unicode_normalization: UnicodeNormalization::NFC,
            xml_character_policy: XmlCharacterPolicy::Escape,
            quote_style: QuoteStyle::Double,
            time_zone_policy: TimeZonePolicy::UTC,
            date_time_format: DateTimeFormat::ISO8601Z,
            emit_reproducibility_banner: false,
            verify_determinism: None,
        }
    }
}

impl DeterminismConfig {
    fn default_namespace_prefixes() -> IndexMap<String, String> {
        let mut prefixes = IndexMap::new();
        prefixes.insert("http://ddex.net/xml/ern/43".to_string(), "ern".to_string());
        prefixes.insert("http://ddex.net/xml/ern/42".to_string(), "ern".to_string());
        prefixes.insert("http://ddex.net/xml/ern/382".to_string(), "ern".to_string());
        prefixes.insert("http://ddex.net/xml/avs".to_string(), "avs".to_string());
        prefixes.insert("http://www.w3.org/2001/XMLSchema-instance".to_string(), "xsi".to_string());
        prefixes
    }
}

/// Canonicalization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonMode {
    /// DB-C14N/1.0 canonicalization
    DbC14n,
    /// Pretty printing (non-canonical)
    Pretty,
    /// Compact output (no whitespace)
    Compact,
}

/// Element ordering strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortStrategy {
    /// Canonical order from XSD
    Canonical,
    /// Preserve input order
    InputOrder,
    /// Custom order
    Custom,
}

/// Namespace handling strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamespaceStrategy {
    /// Use locked prefixes
    Locked,
    /// Inherit from input
    Inherit,
}

/// Output formatting mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputMode {
    /// DB-C14N formatted
    DbC14n,
    /// Pretty printed
    Pretty,
    /// Compact (no whitespace)
    Compact,
}

/// Line ending style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineEnding {
    /// Unix line endings
    LF,
    /// Windows line endings
    CRLF,
}

/// Indentation character
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndentChar {
    /// Space indentation
    Space,
    /// Tab indentation
    Tab,
}

/// Unicode normalization form
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnicodeNormalization {
    /// NFC (Canonical Decomposition, Canonical Composition)
    NFC,
    /// NFD (Canonical Decomposition)
    NFD,
    /// NFKC (Compatibility Decomposition, Canonical Composition)
    NFKC,
    /// NFKD (Compatibility Decomposition)
    NFKD,
}

/// XML character handling policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XmlCharacterPolicy {
    /// Escape special characters
    Escape,
    /// Use CDATA sections
    CData,
    /// Reject invalid characters
    Reject,
}

/// Quote style for attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuoteStyle {
    /// Double quotes
    Double,
    /// Single quotes
    Single,
}

/// Time zone policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeZonePolicy {
    /// Convert to UTC
    UTC,
    /// Preserve original
    Preserve,
    /// Use local time zone
    Local,
}

/// Date/time format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateTimeFormat {
    /// ISO 8601 with Z suffix
    ISO8601Z,
    /// ISO 8601 with offset
    ISO8601,
    /// Custom format
    Custom,
}

/// Determinism verifier
pub struct DeterminismVerifier;

impl DeterminismVerifier {
    /// Verify that output is deterministic by building multiple times
    pub fn verify(
        request: &super::builder::BuildRequest,
        config: &DeterminismConfig,
        iterations: usize,
    ) -> Result<bool, super::error::BuildError> {
        if iterations < 2 {
            return Ok(true);
        }
        
        let mut results = Vec::with_capacity(iterations);
        
        for _ in 0..iterations {
            let builder = super::Builder::with_config(config.clone());
            let result = builder.build_internal(request)?;
            results.push(result.xml);
        }
        
        // Check all results are identical
        let first = &results[0];
        for result in &results[1..] {
            if result != first {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}