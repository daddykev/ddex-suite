// packages/ddex-builder/src/preflight.rs
//! Comprehensive preflight validation for DDEX messages

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

// Validation regex patterns
static ISRC_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Z]{2}[A-Z0-9]{3}\d{2}\d{5}$").unwrap());

/// GTIN lengths: EAN-8, UPC-A (12), EAN-13 and GTIN-14.
static UPC_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d{8}|\d{12,14})$").unwrap());

/// Preflight validator for DDEX messages
pub struct PreflightValidator {
    config: ValidationConfig,
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Validation level
    pub level: PreflightLevel,

    /// Validate identifier formats (including the UPC check digit)
    pub validate_identifiers: bool,

    /// Check for required fields
    pub check_required_fields: bool,

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
            check_required_fields: true,
            validate_references: true,
            profile: None,
        }
    }
}

/// Validation strictness level for preflight checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreflightLevel {
    /// Strict validation - fail on any issue
    Strict,
    /// Warning level - continue with warnings
    Warn,
    /// Info only - log but don't fail
    None,
}

/// Result of preflight validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// List of validation errors that must be fixed
    pub errors: Vec<ValidationError>,
    /// List of warnings that should be reviewed
    pub warnings: Vec<ValidationWarning>,
    /// Informational messages
    pub info: Vec<ValidationInfo>,
    /// Whether validation passed
    pub passed: bool,
}

/// Validation error that prevents building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code for programmatic handling
    pub code: String,
    /// Field that failed validation
    pub field: String,
    /// Human-readable error message
    pub message: String,
    /// Location in the structure (e.g., `Release[0].Track[2]`)
    pub location: String,
}

/// Validation warning that should be reviewed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning code for programmatic handling
    pub code: String,
    /// Field that triggered warning
    pub field: String,
    /// Human-readable warning message
    pub message: String,
    /// Location in the structure
    pub location: String,
    /// Suggested fix if available
    pub suggestion: Option<String>,
}

/// Informational validation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationInfo {
    /// Info code for logging
    pub code: String,
    /// Informational message
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
        result.passed = result.errors.is_empty()
            && (self.config.level != PreflightLevel::Strict || result.warnings.is_empty());

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

            if release.artist.as_deref().unwrap_or_default().is_empty() {
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
            let isrc = track.isrc.as_deref().unwrap_or_default();
            if !self.validate_isrc(isrc) {
                result.errors.push(ValidationError {
                    code: "INVALID_ISRC".to_string(),
                    field: "isrc".to_string(),
                    message: format!("Invalid ISRC format: {}", isrc),
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
                    message: format!(
                        "AudioAlbum typically has 2+ tracks, found {}",
                        release.tracks.len()
                    ),
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
                    message: format!(
                        "AudioSingle typically has 1-3 tracks, found {}",
                        release.tracks.len()
                    ),
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

    /// The GS1 mod-10 check digit: the digits before the check digit are weighted 3, 1, 3,
    /// 1, ... from the right, so it is right for 8-, 12-, 13- and 14-digit codes. (Before
    /// 0.6.0 the weights started from the left, which is wrong for 13-digit codes.)
    fn validate_upc_checksum(&self, upc: &str) -> bool {
        let digits: Vec<u32> = upc.chars().filter_map(|c| c.to_digit(10)).collect();
        if digits.len() != upc.len() || !matches!(digits.len(), 8 | 12 | 13 | 14) {
            return false;
        }
        let Some((check, body)) = digits.split_last() else {
            return false;
        };
        let sum: u32 = body
            .iter()
            .rev()
            .enumerate()
            .map(|(i, d)| if i % 2 == 0 { d * 3 } else { *d })
            .sum();
        (10 - sum % 10) % 10 == *check
    }

    fn validate_duration(&self, duration: &str) -> bool {
        // Basic ISO 8601 duration validation
        duration.starts_with("PT") && (duration.contains('M') || duration.contains('S'))
    }

    fn validate_territory_code(&self, code: &str) -> bool {
        // Basic ISO 3166-1 alpha-2 validation
        code.len() == 2 && code.chars().all(|c| c.is_ascii_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validator() -> PreflightValidator {
        PreflightValidator::new(ValidationConfig::default())
    }

    #[test]
    fn gs1_check_digits_of_every_length() {
        // Known-valid codes: EAN-8, UPC-A, EAN-13, GTIN-14
        for valid in [
            "96385074",
            "036000291452",
            "4006381333931",
            "10012345678902",
        ] {
            assert!(validator().validate_upc(valid), "{valid} is valid");
        }
        // The same codes with a wrong check digit
        for wrong in [
            "96385075",
            "036000291453",
            "4006381333932",
            "10012345678903",
        ] {
            assert!(
                !validator().validate_upc(wrong),
                "{wrong} has a wrong check digit"
            );
        }
        // Not a GTIN length, or not digits
        for bad in ["1234567", "123456789", "03600029145X", "0360002914520000"] {
            assert!(!validator().validate_upc(bad), "{bad}");
        }
    }
}
