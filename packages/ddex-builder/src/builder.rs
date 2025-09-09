//! Main builder implementation

use crate::ast::AST;
use crate::generator::{ASTGenerator, xml_writer::XmlWriter};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Build request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRequest {
    /// Message header
    pub header: MessageHeaderRequest,
    
    /// ERN version
    pub version: String,
    
    /// Profile
    pub profile: Option<String>,
    
    /// Releases (uses IndexMap for order preservation)
    pub releases: Vec<ReleaseRequest>,
    
    /// Deals
    pub deals: Vec<DealRequest>,
    
    /// Extensions (uses IndexMap for determinism)
    pub extensions: Option<IndexMap<String, String>>,
}

/// Message header request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeaderRequest {
    pub message_id: Option<String>,
    pub message_sender: PartyRequest,
    pub message_recipient: PartyRequest,
    pub message_control_type: Option<String>,
}

/// Party request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyRequest {
    pub party_name: Vec<LocalizedStringRequest>,
    pub party_id: Option<String>,
    pub party_reference: Option<String>,  // Added for linker
}

/// Localized string request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedStringRequest {
    pub text: String,
    pub language_code: Option<String>,
}

/// Release request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseRequest {
    pub release_id: String,
    pub release_reference: Option<String>,  // Added for linker
    pub title: Vec<LocalizedStringRequest>,
    pub artist: String,
    pub tracks: Vec<TrackRequest>,
    pub resource_references: Option<Vec<String>>,  // Added for linker
}

/// Track request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackRequest {
    pub track_id: String,                     // Added for linker
    pub resource_reference: Option<String>,   // Added for linker
    pub isrc: String,                        // Changed from Option<String>
    pub title: String,
    pub duration: String,                    // Keep as String for ISO 8601 format
    pub artist: String,
}

/// Deal request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealRequest {
    pub deal_reference: Option<String>,       // Added for linker
    pub deal_terms: DealTerms,               // Define this
    pub release_references: Vec<String>,      // Added for linker
}

/// Deal terms (simple definition for now)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealTerms {
    pub commercial_model_type: String,
    pub territory_code: Vec<String>,
    pub start_date: Option<String>,
}

/// Build options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOptions {
    /// Determinism configuration
    pub determinism: Option<super::determinism::DeterminismConfig>,
    
    /// Validation level
    pub preflight_level: PreflightLevel,
    
    /// ID generation strategy
    pub id_strategy: IdStrategy,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            determinism: None,
            preflight_level: PreflightLevel::Warn,
            id_strategy: IdStrategy::UUID,
        }
    }
}

/// Preflight validation level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreflightLevel {
    /// Strict validation - fail on warnings
    Strict,
    /// Warn but continue
    Warn,
    /// No validation
    None,
}

/// ID generation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdStrategy {
    /// UUID v4
    UUID,
    /// UUID v7 (time-ordered)
    UUIDv7,
    /// Sequential
    Sequential,
    /// Stable hash-based
    StableHash,
}

/// Build result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    /// Generated XML
    pub xml: String,
    
    /// Warnings
    pub warnings: Vec<BuildWarning>,
    
    /// Errors (if any)
    pub errors: Vec<super::error::BuildError>,
    
    /// Statistics
    pub statistics: BuildStatistics,
    
    /// Canonical hash (if deterministic)
    pub canonical_hash: Option<String>,
    
    /// Reproducibility banner (if requested)
    pub reproducibility_banner: Option<String>,
}

/// Build warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildWarning {
    pub code: String,
    pub message: String,
    pub location: Option<String>,
}

/// Build statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStatistics {
    pub releases: usize,
    pub tracks: usize,
    pub deals: usize,
    pub generation_time_ms: u64,
    pub xml_size_bytes: usize,
}

impl Default for BuildStatistics {
    fn default() -> Self {
        Self {
            releases: 0,
            tracks: 0,
            deals: 0,
            generation_time_ms: 0,
            xml_size_bytes: 0,
        }
    }
}

/// Main DDEX Builder
pub struct DDEXBuilder {
    inner: super::Builder,
}

impl DDEXBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            inner: super::Builder::new(),
        }
    }
    
    /// Build DDEX XML from request
    pub fn build(&self, request: BuildRequest, options: BuildOptions) -> Result<BuildResult, super::error::BuildError> {
        let start = std::time::Instant::now();
        
        // 1. Preflight checks
        let warnings = self.preflight(&request, options.preflight_level)?;
        
        // 2. Generate AST
        let mut generator = ASTGenerator::new(request.version.clone());
        let ast = generator.generate(&request)?;
        
        // 3. Apply determinism config
        let config = options.determinism.unwrap_or_default();
        
        // 4. Generate XML
        let writer = XmlWriter::new(config.clone());
        let xml = writer.write(&ast)?;
        
        // 5. Apply canonicalization if requested
        let (final_xml, canonical_hash) = if config.canon_mode == super::determinism::CanonMode::DbC14n {
            let canonicalizer = super::canonical::DB_C14N::new(config.clone());
            let canonical = canonicalizer.canonicalize(&xml)?;
            let hash = Some(canonicalizer.canonical_hash(&canonical)?);
            (canonical, hash)
        } else {
            (xml, None)
        };
        
        // 6. Generate reproducibility banner if requested
        let reproducibility_banner = if config.emit_reproducibility_banner {
            Some(format!(
                "Generated by DDEX Builder v{} with DB-C14N/{}",
                env!("CARGO_PKG_VERSION"),
                super::DB_C14N_VERSION
            ))
        } else {
            None
        };
        
        let elapsed = start.elapsed();
        
        Ok(BuildResult {
            xml: final_xml.clone(),
            warnings,
            errors: Vec::new(),
            statistics: BuildStatistics {
                releases: request.releases.len(),
                tracks: request.releases.iter().map(|r| r.tracks.len()).sum(),
                deals: request.deals.len(),
                generation_time_ms: elapsed.as_millis() as u64,
                xml_size_bytes: final_xml.len(),
            },
            canonical_hash,
            reproducibility_banner,
        })
    }
    
    fn preflight(&self, request: &BuildRequest, level: PreflightLevel) -> Result<Vec<BuildWarning>, super::error::BuildError> {
        let mut warnings = Vec::new();
        
        if level == PreflightLevel::None {
            return Ok(warnings);
        }
        
        // Check for required fields
        if request.releases.is_empty() {
            warnings.push(BuildWarning {
                code: "NO_RELEASES".to_string(),
                message: "No releases in request".to_string(),
                location: Some("/releases".to_string()),
            });
        }
        
        // Check each release
        for (idx, release) in request.releases.iter().enumerate() {
            if release.title.is_empty() {
                warnings.push(BuildWarning {
                    code: "MISSING_TITLE".to_string(),
                    message: format!("Release {} missing title", idx),
                    location: Some(format!("/releases/{}/title", idx)),
                });
            }
            
            if release.artist.is_empty() {
                warnings.push(BuildWarning {
                    code: "MISSING_ARTIST".to_string(),
                    message: format!("Release {} missing artist", idx),
                    location: Some(format!("/releases/{}/artist", idx)),
                });
            }
        }
        
        if level == PreflightLevel::Strict && !warnings.is_empty() {
            return Err(super::error::BuildError::InvalidFormat {
                field: "request".to_string(),
                message: format!("{} validation warnings in strict mode", warnings.len()),
            });
        }
        
        Ok(warnings)
    }
    
    fn generate_ast(&self, _request: &BuildRequest) -> Result<AST, super::error::BuildError> {
        // AST generation
        todo!("Generate AST")
    }
    
    fn apply_determinism(&self, _ast: AST, _options: &BuildOptions) -> Result<AST, super::error::BuildError> {
        // Apply determinism rules
        todo!("Apply determinism")
    }
    
    fn generate_xml(&self, _ast: AST) -> Result<String, super::error::BuildError> {
        // XML generation
        todo!("Generate XML")
    }
    
    fn calculate_hash(&self, xml: &str) -> Result<String, super::error::BuildError> {
        let canonicalizer = super::canonical::DB_C14N::new(super::determinism::DeterminismConfig::default());
        canonicalizer.canonical_hash(xml)
    }
}

impl Default for DDEXBuilder {
    fn default() -> Self {
        Self::new()
    }
}