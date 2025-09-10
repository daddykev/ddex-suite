// packages/ddex-builder/src/preflight.rs
//! Comprehensive preflight validation for DDEX messages

use regex::Regex;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

// Validation regex patterns
static ISRC_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Z]{2}[A-Z0-9]{3}\d{2}\d{5}$").unwrap()
});

static UPC_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d{12,14}$").unwrap()
});

static ISWC_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^T\d{10}$").unwrap()
});

static ISNI_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\d{15}[\dX]$").unwrap()
});

/// Preflight validator for DDEX messages
pub struct PreflightValidator {
    config: ValidationConfig,
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Validation level
    pub level: PreflightLevel,
    
    /// Validate identifier formats
    pub validate_identifiers: bool,
    
    /// Validate checksums
    pub validate_checksums: bool,
    
    /// Check for required fields
    pub check_required_fields: bool,
    
    /// Validate dates
    pub validate_dates: bool,
    
    /// Check references
    pub validate_references: bool,
    
    /// Profile-specific validation
    pub profile: Option<String>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            level: PreflightLevel::Warn,
            validate_identifiers: true,
            validate_checksums: true,
            check_required_fields: true,
            validate_dates: true,
            validate_references: true,
            profile: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreflightLevel {
    /// Strict - fail on any warning
    Strict,
    /// Warn - collect warnings but continue
    Warn,
    /// None - skip validation
    None,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub info: Vec<ValidationInfo>,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub field: String,
    pub message: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub field: String,
    pub message: String,
    pub location: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationInfo {
    pub code: String,
    pub message: String,
}

impl PreflightValidator {
    /// Create new validator
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }
    
    /// Validate a build request
    pub fn validate(
        &self,
        request: &super::builder::BuildRequest,
    ) -> Result<ValidationResult, super::error::BuildError> {
        let mut result = ValidationResult {
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            passed: true,
        };
        
        if self.config.level == PreflightLevel::None {
            return Ok(result);
        }
        
        // Validate releases
        for (idx, release) in request.releases.iter().enumerate() {
            self.validate_release(release, idx, &mut result)?;
        }
        
        // Validate deals
        for (idx, deal) in request.deals.iter().enumerate() {
            self.validate_deal(deal, idx, &mut result)?;
        }
        
        // Check cross-references if enabled
        if self.config.validate_references {
            self.validate_references(request, &mut result)?;
        }
        
        // Apply profile-specific validation
        if let Some(profile) = &self.config.profile {
            self.validate_profile(request, profile, &mut result)?;
        }
        
        // Determine if validation passed
        result.passed = result.errors.is_empty() && 
            (self.config.level != PreflightLevel::Strict || result.warnings.is_empty());
        
        Ok(result)
    }
    
    fn validate_release(
        &self,
        release: &super::builder::ReleaseRequest,
        idx: usize,
        result: &mut ValidationResult,
    ) -> Result<(), super::error::BuildError> {
        let location = format!("/releases[{}]", idx);
        
        // Check required fields
        if self.config.check_required_fields {
            if release.title.is_empty() {
                result.errors.push(ValidationError {
                    code: "MISSING_TITLE".to_string(),
                    field: "title".to_string(),
                    message: "Release title is required".to_string(),
                    location: format!("{}/title", location),
                });
            }
            
            if release.artist.is_empty() {
                result.warnings.push(ValidationWarning {
                    code: "MISSING_ARTIST".to_string(),
                    field: "artist".to_string(),
                    message: "Release artist is recommended".to_string(),
                    location: format!("{}/artist", location),
                    suggestion: Some("Add display artist name".to_string()),
                });
            }
        }
        
        // Validate UPC
        if self.config.validate_identifiers {
            if let Some(upc) = &release.upc {
                if !self.validate_upc(upc) {
                    result.errors.push(ValidationError {
                        code: "INVALID_UPC".to_string(),
                        field: "upc".to_string(),
                        message: format!("Invalid UPC format: {}", upc),
                        location: format!("{}/upc", location),
                    });
                }
            }
        }
        
        // Validate tracks
        for (track_idx, track) in release.tracks.iter().enumerate() {
            self.validate_track(track, idx, track_idx, result)?;
        }
        
        Ok(())
    }
    
    fn validate_track(
        &self,
        track: &super::builder::TrackRequest,
        release_idx: usize,
        track_idx: usize,
        result: &mut ValidationResult,
    ) -> Result<(), super::error::BuildError> {
        let location = format!("/releases[{}]/tracks[{}]", release_idx, track_idx);
        
        // Validate ISRC
        if self.config.validate_identifiers {
            if !self.validate_isrc(&track.isrc) {
                result.errors.push(ValidationError {
                    code: "INVALID_ISRC".to_string(),
                    field: "isrc".to_string(),
                    message: format!("Invalid ISRC format: {}", track.isrc),
                    location: format!("{}/isrc", location),
                });
            }
        }
        
        // Validate duration format
        if !track.duration.is_empty() && !self.validate_duration(&track.duration) {
            result.warnings.push(ValidationWarning {
                code: "INVALID_DURATION".to_string(),
                field: "duration".to_string(),
                message: format!("Invalid ISO 8601 duration: {}", track.duration),
                location: format!("{}/duration", location),
                suggestion: Some("Use format PT3M45S for 3:45".to_string()),
            });
        }
        
        Ok(())
    }
    
    fn validate_deal(
        &self,
        deal: &super::builder::DealRequest,
        idx: usize,
        result: &mut ValidationResult,
    ) -> Result<(), super::error::BuildError> {
        let location = format!("/deals[{}]", idx);
        
        // Validate territory codes
        for (t_idx, territory) in deal.deal_terms.territory_code.iter().enumerate() {
            if !self.validate_territory_code(territory) {
                result.warnings.push(ValidationWarning {
                    code: "INVALID_TERRITORY".to_string(),
                    field: "territory_code".to_string(),
                    message: format!("Invalid territory code: {}", territory),
                    location: format!("{}/territory_code[{}]", location, t_idx),
                    suggestion: Some("Use ISO 3166-1 alpha-2 codes".to_string()),
                });
            }
        }
        
        Ok(())
    }
    
    fn validate_references(
        &self,
        request: &super::builder::BuildRequest,
        result: &mut ValidationResult,
    ) -> Result<(), super::error::BuildError> {
        // Collect all references
        let mut release_refs = indexmap::IndexSet::new();
        let mut resource_refs = indexmap::IndexSet::new();
        
        for release in &request.releases {
            if let Some(ref_val) = &release.release_reference {
                release_refs.insert(ref_val.clone());
            }
            
            for track in &release.tracks {
                if let Some(ref_val) = &track.resource_reference {
                    resource_refs.insert(ref_val.clone());
                }
            }
        }
        
        // Check deal references
        for (idx, deal) in request.deals.iter().enumerate() {
            for (r_idx, release_ref) in deal.release_references.iter().enumerate() {
                if !release_refs.contains(release_ref) {
                    result.errors.push(ValidationError {
                        code: "UNKNOWN_REFERENCE".to_string(),
                        field: "release_reference".to_string(),
                        message: format!("Unknown release reference: {}", release_ref),
                        location: format!("/deals[{}]/release_references[{}]", idx, r_idx),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    fn validate_profile(
        &self,
        request: &super::builder::BuildRequest,
        profile: &str,
        result: &mut ValidationResult,
    ) -> Result<(), super::error::BuildError> {
        match profile {
            "AudioAlbum" => self.validate_audio_album_profile(request, result),
            "AudioSingle" => self.validate_audio_single_profile(request, result),
            _ => {
                result.info.push(ValidationInfo {
                    code: "UNKNOWN_PROFILE".to_string(),
                    message: format!("Profile '{}' validation not implemented", profile),
                });
                Ok(())
            }
        }
    }
    
    fn validate_audio_album_profile(
        &self,
        request: &super::builder::BuildRequest,
        result: &mut ValidationResult,
    ) -> Result<(), super::error::BuildError> {
        // AudioAlbum specific requirements
        for (idx, release) in request.releases.iter().enumerate() {
            // Must have at least 2 tracks for album
            if release.tracks.len() < 2 {
                result.warnings.push(ValidationWarning {
                    code: "ALBUM_TRACK_COUNT".to_string(),
                    field: "tracks".to_string(),
                    message: format!("AudioAlbum typically has 2+ tracks, found {}", 
                        release.tracks.len()),
                    location: format!("/releases[{}]/tracks", idx),
                    suggestion: Some("Consider using AudioSingle profile".to_string()),
                });
            }
            
            // Should have UPC
            if release.upc.is_none() {
                result.errors.push(ValidationError {
                    code: "MISSING_UPC".to_string(),
                    field: "upc".to_string(),
                    message: "UPC is required for AudioAlbum profile".to_string(),
                    location: format!("/releases[{}]/upc", idx),
                });
            }
        }
        
        Ok(())
    }
    
    fn validate_audio_single_profile(
        &self,
        request: &super::builder::BuildRequest,
        result: &mut ValidationResult,
    ) -> Result<(), super::error::BuildError> {
        // AudioSingle specific requirements
        for (idx, release) in request.releases.iter().enumerate() {
            // Should have 1-3 tracks for single
            if release.tracks.len() > 3 {
                result.warnings.push(ValidationWarning {
                    code: "SINGLE_TRACK_COUNT".to_string(),
                    field: "tracks".to_string(),
                    message: format!("AudioSingle typically has 1-3 tracks, found {}", 
                        release.tracks.len()),
                    location: format!("/releases[{}]/tracks", idx),
                    suggestion: Some("Consider using AudioAlbum profile".to_string()),
                });
            }
        }
        
        Ok(())
    }
    
    // Identifier validation methods
    fn validate_isrc(&self, isrc: &str) -> bool {
        ISRC_PATTERN.is_match(isrc)
    }
    
    fn validate_upc(&self, upc: &str) -> bool {
        if !UPC_PATTERN.is_match(upc) {
            return false;
        }
        
        // Validate check digit
        self.validate_upc_checksum(upc)
    }
    
    fn validate_upc_checksum(&self, upc: &str) -> bool {
        let digits: Vec<u32> = upc.chars()
            .filter_map(|c| c.to_digit(10))
            .collect();
        
        if digits.len() < 12 {
            return false;
        }
        
        let mut sum = 0;
        for (i, &digit) in digits.iter().take(digits.len() - 1).enumerate() {
            if i % 2 == 0 {
                sum += digit;
            } else {
                sum += digit * 3;
            }
        }
        
        let check_digit = (10 - (sum % 10)) % 10;
        digits[digits.len() - 1] == check_digit
    }
    
    fn validate_duration(&self, duration: &str) -> bool {
        // Basic ISO 8601 duration validation
        duration.starts_with("PT") && 
            (duration.contains('M') || duration.contains('S'))
    }
    
    fn validate_territory_code(&self, code: &str) -> bool {
        // Basic ISO 3166-1 alpha-2 validation
        code.len() == 2 && code.chars().all(|c| c.is_ascii_uppercase())
    }
}