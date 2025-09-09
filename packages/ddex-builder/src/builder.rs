//! Main builder implementation

use crate::generator::{ASTGenerator, xml_writer::XmlWriter};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
pub use super::preflight::PreflightLevel;

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
    pub message_created_date_time: Option<String>,
}

/// Party request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyRequest {
    pub party_name: Vec<LocalizedStringRequest>,
    pub party_id: Option<String>,
    pub party_reference: Option<String>,
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
    pub label: Option<String>,              // Added for metadata
    pub release_date: Option<String>,       // Added for metadata
    pub upc: Option<String>,                // Added for validation
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
    pub preflight_level: super::preflight::PreflightLevel,
    
    /// ID generation strategy
    pub id_strategy: IdStrategy,
    
    /// Stable hash configuration (when using StableHash strategy)
    pub stable_hash_config: Option<super::id_generator::StableHashConfig>,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            determinism: None,
            preflight_level: super::preflight::PreflightLevel::Warn,
            id_strategy: IdStrategy::UUID,
            stable_hash_config: None,
        }
    }
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
    pub fn build(&self, mut request: BuildRequest, options: BuildOptions) -> Result<BuildResult, super::error::BuildError> {
        let start = std::time::Instant::now();
        let mut warnings = Vec::new();
        
        // 1. Enhanced preflight checks with new validator
        let validator = super::preflight::PreflightValidator::new(
            super::preflight::ValidationConfig {
                level: options.preflight_level,
                profile: request.profile.clone(),
                validate_identifiers: true,
                validate_checksums: true,
                check_required_fields: true,
                validate_dates: true,
                validate_references: true,
            }
        );
        
        let validation_result = validator.validate(&request)?;
        
        // Convert validation warnings to build warnings
        for warning in validation_result.warnings {
            warnings.push(BuildWarning {
                code: warning.code,
                message: warning.message,
                location: Some(warning.location),
            });
        }
        
        // Fail if validation didn't pass
        if !validation_result.passed {
            if options.preflight_level == super::preflight::PreflightLevel::Strict {
                return Err(super::error::BuildError::ValidationFailed {
                    errors: validation_result.errors.iter()
                        .map(|e| format!("{}: {}", e.code, e.message))
                        .collect(),
                });
            }
        }
        
        // 2. Generate IDs based on strategy
        self.generate_ids(&mut request, &options)?;
        
        // 3. Generate AST
        let mut generator = ASTGenerator::new(request.version.clone());
        let ast = generator.generate(&request)?;
        
        // 4. Apply determinism config
        let config = options.determinism.unwrap_or_default();
        
        // 5. Generate XML
        let writer = XmlWriter::new(config.clone());
        let xml = writer.write(&ast)?;
        
        // 6. Apply canonicalization if requested
        let (final_xml, canonical_hash) = if config.canon_mode == super::determinism::CanonMode::DbC14n {
            let canonicalizer = super::canonical::DB_C14N::new(config.clone());
            let canonical = canonicalizer.canonicalize(&xml)?;
            let hash = Some(canonicalizer.canonical_hash(&canonical)?);
            (canonical, hash)
        } else {
            (xml, None)
        };
        
        // 7. Generate reproducibility banner if requested
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
    
    /// Generate IDs based on the selected strategy
    fn generate_ids(&self, request: &mut BuildRequest, options: &BuildOptions) -> Result<(), super::error::BuildError> {
        match options.id_strategy {
            IdStrategy::UUID => {
                self.generate_uuid_ids(request)?;
            },
            IdStrategy::UUIDv7 => {
                self.generate_uuidv7_ids(request)?;
            },
            IdStrategy::Sequential => {
                self.generate_sequential_ids(request)?;
            },
            IdStrategy::StableHash => {
                self.generate_stable_hash_ids(request, options)?;
            },
        }
        Ok(())
    }
    
    /// Generate UUID v4 IDs
    fn generate_uuid_ids(&self, request: &mut BuildRequest) -> Result<(), super::error::BuildError> {
        use uuid::Uuid;
        
        // Generate message ID if missing
        if request.header.message_id.is_none() {
            request.header.message_id = Some(format!("MSG_{}", Uuid::new_v4()));
        }
        
        // Generate release references if missing
        for release in &mut request.releases {
            if release.release_reference.is_none() {
                release.release_reference = Some(format!("R{}", Uuid::new_v4().simple()));
            }
            
            // Generate resource references for tracks
            for track in &mut release.tracks {
                if track.resource_reference.is_none() {
                    track.resource_reference = Some(format!("A{}", Uuid::new_v4().simple()));
                }
            }
        }
        
        // Generate deal references if missing
        for (idx, deal) in request.deals.iter_mut().enumerate() {
            if deal.deal_reference.is_none() {
                deal.deal_reference = Some(format!("D{}", idx + 1));
            }
        }
        
        Ok(())
    }
    
    /// Generate UUID v7 IDs (time-ordered)
    fn generate_uuidv7_ids(&self, request: &mut BuildRequest) -> Result<(), super::error::BuildError> {
        // For now, fall back to UUID v4
        // TODO: Implement proper UUID v7 generation
        self.generate_uuid_ids(request)
    }
    
    /// Generate sequential IDs
    fn generate_sequential_ids(&self, request: &mut BuildRequest) -> Result<(), super::error::BuildError> {
        // Generate message ID if missing
        if request.header.message_id.is_none() {
            request.header.message_id = Some(format!("MSG_{}", chrono::Utc::now().timestamp()));
        }
        
        // Generate release references if missing
        for (idx, release) in request.releases.iter_mut().enumerate() {
            if release.release_reference.is_none() {
                release.release_reference = Some(format!("R{}", idx + 1));
            }
            
            // Generate resource references for tracks
            for (track_idx, track) in release.tracks.iter_mut().enumerate() {
                if track.resource_reference.is_none() {
                    track.resource_reference = Some(format!("A{}", (idx * 1000) + track_idx + 1));
                }
            }
        }
        
        // Generate deal references if missing
        for (idx, deal) in request.deals.iter_mut().enumerate() {
            if deal.deal_reference.is_none() {
                deal.deal_reference = Some(format!("D{}", idx + 1));
            }
        }
        
        Ok(())
    }
    
    /// Generate stable hash-based IDs
    fn generate_stable_hash_ids(&self, request: &mut BuildRequest, options: &BuildOptions) -> Result<(), super::error::BuildError> {
        let config = options.stable_hash_config.clone()
            .unwrap_or_default();
        let mut id_gen = super::id_generator::StableHashGenerator::new(config);
        
        // Generate message ID if missing
        if request.header.message_id.is_none() {
            // Use sender/recipient info for stable message ID
            let sender_name = request.header.message_sender.party_name
                .first()
                .map(|s| s.text.clone())
                .unwrap_or_default();
            let recipient_name = request.header.message_recipient.party_name
                .first()
                .map(|s| s.text.clone())
                .unwrap_or_default();
            
            let msg_id = id_gen.generate_party_id(
                &format!("{}-{}", sender_name, recipient_name),
                "MessageHeader",
                &[chrono::Utc::now().format("%Y%m%d").to_string()],
            )?;
            request.header.message_id = Some(msg_id);
        }
        
        // Generate stable IDs for releases
        for release in &mut request.releases {
            if release.release_reference.is_none() {
                let id = id_gen.generate_release_id(
                    release.upc.as_deref().unwrap_or(&release.release_id),
                    "Album",
                    &release.tracks.iter()
                        .map(|t| t.isrc.clone())
                        .collect::<Vec<_>>(),
                    &[], // Empty territory set for now
                )?;
                release.release_reference = Some(id);
            }
            
            // Generate stable IDs for tracks/resources
            for track in &mut release.tracks {
                if track.resource_reference.is_none() {
                    // Parse duration to seconds for stable hash
                    let duration_seconds = self.parse_duration_to_seconds(&track.duration)
                        .unwrap_or(0);
                    
                    let id = id_gen.generate_resource_id(
                        &track.isrc,
                        duration_seconds,
                        None, // No file hash available
                    )?;
                    track.resource_reference = Some(id);
                }
            }
        }
        
        // Generate deal references if missing
        for (_idx, deal) in request.deals.iter_mut().enumerate() {
            if deal.deal_reference.is_none() {
                // Create stable deal ID based on terms
                let territories = deal.deal_terms.territory_code.join(",");
                deal.deal_reference = Some(format!("DEAL_{}_{}", 
                    deal.deal_terms.commercial_model_type,
                    territories));
            }
        }
        
        Ok(())
    }
    
    /// Parse ISO 8601 duration to seconds
    fn parse_duration_to_seconds(&self, duration: &str) -> Option<u32> {
        // Simple parser for PT3M45S format
        if !duration.starts_with("PT") {
            return None;
        }
        
        let mut seconds = 0u32;
        let mut current_num = String::new();
        
        for ch in duration[2..].chars() {
            match ch {
                '0'..='9' => current_num.push(ch),
                'H' => {
                    if let Ok(hours) = current_num.parse::<u32>() {
                        seconds += hours * 3600;
                    }
                    current_num.clear();
                },
                'M' => {
                    if let Ok(minutes) = current_num.parse::<u32>() {
                        seconds += minutes * 60;
                    }
                    current_num.clear();
                },
                'S' => {
                    if let Ok(secs) = current_num.parse::<u32>() {
                        seconds += secs;
                    }
                    current_num.clear();
                },
                _ => {}
            }
        }
        
        Some(seconds)
    }
    
    /// Legacy preflight check method (kept for compatibility)
    fn preflight(&self, request: &BuildRequest, level: super::preflight::PreflightLevel) -> Result<Vec<BuildWarning>, super::error::BuildError> {
        let mut warnings = Vec::new();
        
        if level == super::preflight::PreflightLevel::None {
            return Ok(warnings);
        }
        
        // Basic checks (enhanced validation is done in main build method)
        if request.releases.is_empty() {
            warnings.push(BuildWarning {
                code: "NO_RELEASES".to_string(),
                message: "No releases in request".to_string(),
                location: Some("/releases".to_string()),
            });
        }
        
        if level == super::preflight::PreflightLevel::Strict && !warnings.is_empty() {
            return Err(super::error::BuildError::InvalidFormat {
                field: "request".to_string(),
                message: format!("{} validation warnings in strict mode", warnings.len()),
            });
        }
        
        Ok(warnings)
    }
}

impl Default for DDEXBuilder {
    fn default() -> Self {
        Self::new()
    }
}