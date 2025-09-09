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
    
    /// Compare two DDEX XML documents and return semantic differences
    /// 
    /// This method performs semantic diffing that understands DDEX business logic,
    /// not just XML structure differences.
    pub fn diff_xml(&self, old_xml: &str, new_xml: &str) -> Result<super::diff::types::ChangeSet, super::error::BuildError> {
        self.diff_xml_with_config(old_xml, new_xml, super::diff::DiffConfig::default())
    }
    
    /// Compare two DDEX XML documents with custom diff configuration
    pub fn diff_xml_with_config(
        &self, 
        old_xml: &str, 
        new_xml: &str, 
        config: super::diff::DiffConfig
    ) -> Result<super::diff::types::ChangeSet, super::error::BuildError> {
        // Parse both XML documents to AST
        let old_ast = self.parse_xml_to_ast(old_xml)?;
        let new_ast = self.parse_xml_to_ast(new_xml)?;
        
        // Create diff engine and compare
        let mut diff_engine = super::diff::DiffEngine::new_with_config(config);
        diff_engine.diff(&old_ast, &new_ast)
    }
    
    /// Compare a BuildRequest with existing XML to see what would change
    pub fn diff_request_with_xml(
        &self, 
        request: &BuildRequest, 
        existing_xml: &str
    ) -> Result<super::diff::types::ChangeSet, super::error::BuildError> {
        // Build new XML from request
        let build_result = self.build(request.clone(), BuildOptions::default())?;
        
        // Compare existing XML with newly built XML
        self.diff_xml(existing_xml, &build_result.xml)
    }
    
    /// Helper to parse XML string to AST
    fn parse_xml_to_ast(&self, xml: &str) -> Result<super::ast::AST, super::error::BuildError> {
        use quick_xml::Reader;
        
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        
        // This is a simplified XML->AST parser
        // In a production system, you'd want to use the actual ddex-parser
        let mut root_element = super::ast::Element::new("Root");
        let namespace_map = indexmap::IndexMap::new();
        
        // For now, create a basic AST structure
        // TODO: Implement proper XML parsing or integrate with ddex-parser
        root_element = root_element.with_text(xml);
        
        Ok(super::ast::AST {
            root: root_element,
            namespaces: namespace_map,
            schema_location: None,
        })
    }
    
    /// Create an UpdateReleaseMessage from two DDEX messages
    /// 
    /// This method compares an original DDEX message with an updated version and
    /// generates a minimal UpdateReleaseMessage containing only the differences.
    pub fn create_update(
        &self,
        original_xml: &str,
        updated_xml: &str,
        original_message_id: &str,
    ) -> Result<super::messages::UpdateReleaseMessage, super::error::BuildError> {
        let mut update_generator = super::messages::UpdateGenerator::new();
        update_generator.create_update(original_xml, updated_xml, original_message_id)
    }
    
    /// Create an UpdateReleaseMessage with custom configuration
    pub fn create_update_with_config(
        &self,
        original_xml: &str,
        updated_xml: &str,
        original_message_id: &str,
        config: super::messages::UpdateConfig,
    ) -> Result<super::messages::UpdateReleaseMessage, super::error::BuildError> {
        let mut update_generator = super::messages::UpdateGenerator::new_with_config(config);
        update_generator.create_update(original_xml, updated_xml, original_message_id)
    }
    
    /// Apply an UpdateReleaseMessage to a base DDEX message
    /// 
    /// This method takes a base DDEX message and applies the operations from an
    /// UpdateReleaseMessage to produce a new complete DDEX message.
    pub fn apply_update(
        &self,
        base_xml: &str,
        update: &super::messages::UpdateReleaseMessage,
    ) -> Result<String, super::error::BuildError> {
        let update_generator = super::messages::UpdateGenerator::new();
        update_generator.apply_update(base_xml, update)
    }
    
    /// Create an update from a BuildRequest compared to existing XML
    /// 
    /// This is useful for generating updates when you have a new BuildRequest
    /// that represents the desired state and need to update an existing message.
    pub fn create_update_from_request(
        &self,
        existing_xml: &str,
        request: &BuildRequest,
        original_message_id: &str,
    ) -> Result<super::messages::UpdateReleaseMessage, super::error::BuildError> {
        // Build new XML from request
        let build_result = self.build(request.clone(), BuildOptions::default())?;
        
        // Create update between existing and new XML
        self.create_update(existing_xml, &build_result.xml, original_message_id)
    }
    
    /// Validate an UpdateReleaseMessage for safety and consistency
    pub fn validate_update(
        &self,
        update: &super::messages::UpdateReleaseMessage,
    ) -> Result<super::messages::ValidationStatus, super::error::BuildError> {
        let update_generator = super::messages::UpdateGenerator::new();
        update_generator.validate_update(update)
    }
    
    /// Generate an UpdateReleaseMessage as XML
    pub fn serialize_update(
        &self,
        update: &super::messages::UpdateReleaseMessage,
    ) -> Result<String, super::error::BuildError> {
        self.serialize_update_message_to_xml(update)
    }
    
    // Helper methods for update serialization
    
    fn serialize_update_message_to_xml(
        &self,
        update: &super::messages::UpdateReleaseMessage,
    ) -> Result<String, super::error::BuildError> {
        let mut xml = String::new();
        
        // XML declaration and root element
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<UpdateReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">"#);
        xml.push('\n');
        
        // Message header
        self.serialize_update_header(&mut xml, &update.header)?;
        
        // Update metadata
        self.serialize_update_metadata(&mut xml, &update.update_metadata)?;
        
        // Update list
        self.serialize_update_list(&mut xml, &update.update_list)?;
        
        // Resource updates
        if !update.resource_updates.is_empty() {
            self.serialize_resource_updates(&mut xml, &update.resource_updates)?;
        }
        
        // Release updates
        if !update.release_updates.is_empty() {
            self.serialize_release_updates(&mut xml, &update.release_updates)?;
        }
        
        // Deal updates
        if !update.deal_updates.is_empty() {
            self.serialize_deal_updates(&mut xml, &update.deal_updates)?;
        }
        
        // Close root element
        xml.push_str("</UpdateReleaseMessage>\n");
        
        Ok(xml)
    }
    
    fn serialize_update_header(
        &self,
        xml: &mut String,
        header: &MessageHeaderRequest,
    ) -> Result<(), super::error::BuildError> {
        xml.push_str("  <MessageHeader>\n");
        
        if let Some(ref message_id) = header.message_id {
            xml.push_str(&format!("    <MessageId>{}</MessageId>\n", self.escape_xml(message_id)));
        }
        
        // Message sender
        xml.push_str("    <MessageSender>\n");
        if !header.message_sender.party_name.is_empty() {
            xml.push_str(&format!("      <PartyName>{}</PartyName>\n", 
                self.escape_xml(&header.message_sender.party_name[0].text)));
        }
        xml.push_str("    </MessageSender>\n");
        
        // Message recipient
        xml.push_str("    <MessageRecipient>\n");
        if !header.message_recipient.party_name.is_empty() {
            xml.push_str(&format!("      <PartyName>{}</PartyName>\n", 
                self.escape_xml(&header.message_recipient.party_name[0].text)));
        }
        xml.push_str("    </MessageRecipient>\n");
        
        // Created date time
        if let Some(ref created_time) = header.message_created_date_time {
            xml.push_str(&format!("    <MessageCreatedDateTime>{}</MessageCreatedDateTime>\n", 
                self.escape_xml(created_time)));
        } else {
            let default_time = chrono::Utc::now().to_rfc3339();
            xml.push_str(&format!("    <MessageCreatedDateTime>{}</MessageCreatedDateTime>\n", 
                self.escape_xml(&default_time)));
        }
        
        xml.push_str("  </MessageHeader>\n");
        Ok(())
    }
    
    fn serialize_update_metadata(
        &self,
        xml: &mut String,
        metadata: &super::messages::UpdateMetadata,
    ) -> Result<(), super::error::BuildError> {
        xml.push_str("  <UpdateMetadata>\n");
        xml.push_str(&format!("    <OriginalMessageId>{}</OriginalMessageId>\n", 
            self.escape_xml(&metadata.original_message_id)));
        xml.push_str(&format!("    <UpdateSequence>{}</UpdateSequence>\n", metadata.update_sequence));
        xml.push_str(&format!("    <TotalOperations>{}</TotalOperations>\n", metadata.total_operations));
        xml.push_str(&format!("    <ImpactLevel>{}</ImpactLevel>\n", 
            self.escape_xml(&metadata.impact_level)));
        xml.push_str(&format!("    <ValidationStatus>{}</ValidationStatus>\n", metadata.validation_status));
        xml.push_str(&format!("    <UpdateCreatedDateTime>{}</UpdateCreatedDateTime>\n", 
            metadata.update_created_timestamp.to_rfc3339()));
        xml.push_str("  </UpdateMetadata>\n");
        Ok(())
    }
    
    fn serialize_update_list(
        &self,
        xml: &mut String,
        operations: &[super::messages::UpdateOperation],
    ) -> Result<(), super::error::BuildError> {
        xml.push_str("  <UpdateList>\n");
        
        for operation in operations {
            xml.push_str("    <UpdateOperation>\n");
            xml.push_str(&format!("      <OperationId>{}</OperationId>\n", 
                self.escape_xml(&operation.operation_id)));
            xml.push_str(&format!("      <Action>{}</Action>\n", operation.action));
            xml.push_str(&format!("      <TargetPath>{}</TargetPath>\n", 
                self.escape_xml(&operation.target_path)));
            xml.push_str(&format!("      <EntityType>{}</EntityType>\n", operation.entity_type));
            xml.push_str(&format!("      <EntityId>{}</EntityId>\n", 
                self.escape_xml(&operation.entity_id)));
            
            if let Some(ref old_value) = operation.old_value {
                xml.push_str(&format!("      <OldValue>{}</OldValue>\n", 
                    self.escape_xml(old_value)));
            }
            
            if let Some(ref new_value) = operation.new_value {
                xml.push_str(&format!("      <NewValue>{}</NewValue>\n", 
                    self.escape_xml(new_value)));
            }
            
            xml.push_str(&format!("      <IsCritical>{}</IsCritical>\n", operation.is_critical));
            xml.push_str(&format!("      <Description>{}</Description>\n", 
                self.escape_xml(&operation.description)));
            
            if !operation.dependencies.is_empty() {
                xml.push_str("      <Dependencies>\n");
                for dependency in &operation.dependencies {
                    xml.push_str(&format!("        <Dependency>{}</Dependency>\n", 
                        self.escape_xml(dependency)));
                }
                xml.push_str("      </Dependencies>\n");
            }
            
            xml.push_str("    </UpdateOperation>\n");
        }
        
        xml.push_str("  </UpdateList>\n");
        Ok(())
    }
    
    fn serialize_resource_updates(
        &self,
        xml: &mut String,
        resource_updates: &indexmap::IndexMap<String, super::messages::ResourceUpdate>,
    ) -> Result<(), super::error::BuildError> {
        xml.push_str("  <ResourceUpdates>\n");
        
        for (resource_id, update) in resource_updates {
            xml.push_str("    <ResourceUpdate>\n");
            xml.push_str(&format!("      <ResourceId>{}</ResourceId>\n", 
                self.escape_xml(resource_id)));
            xml.push_str(&format!("      <ResourceReference>{}</ResourceReference>\n", 
                self.escape_xml(&update.resource_reference)));
            xml.push_str(&format!("      <Action>{}</Action>\n", update.action));
            
            // Add resource data if present
            if let Some(ref data) = update.resource_data {
                xml.push_str("      <ResourceData>\n");
                xml.push_str(&format!("        <Type>{}</Type>\n", 
                    self.escape_xml(&data.resource_type)));
                xml.push_str(&format!("        <Title>{}</Title>\n", 
                    self.escape_xml(&data.title)));
                xml.push_str(&format!("        <Artist>{}</Artist>\n", 
                    self.escape_xml(&data.artist)));
                
                if let Some(ref isrc) = data.isrc {
                    xml.push_str(&format!("        <ISRC>{}</ISRC>\n", 
                        self.escape_xml(isrc)));
                }
                
                if let Some(ref duration) = data.duration {
                    xml.push_str(&format!("        <Duration>{}</Duration>\n", 
                        self.escape_xml(duration)));
                }
                
                xml.push_str("      </ResourceData>\n");
            }
            
            xml.push_str("    </ResourceUpdate>\n");
        }
        
        xml.push_str("  </ResourceUpdates>\n");
        Ok(())
    }
    
    fn serialize_release_updates(
        &self,
        xml: &mut String,
        release_updates: &indexmap::IndexMap<String, super::messages::ReleaseUpdate>,
    ) -> Result<(), super::error::BuildError> {
        xml.push_str("  <ReleaseUpdates>\n");
        
        for (release_id, update) in release_updates {
            xml.push_str("    <ReleaseUpdate>\n");
            xml.push_str(&format!("      <ReleaseId>{}</ReleaseId>\n", 
                self.escape_xml(release_id)));
            xml.push_str(&format!("      <ReleaseReference>{}</ReleaseReference>\n", 
                self.escape_xml(&update.release_reference)));
            xml.push_str(&format!("      <Action>{}</Action>\n", update.action));
            
            // Add release data if present
            if let Some(ref data) = update.release_data {
                xml.push_str("      <ReleaseData>\n");
                xml.push_str(&format!("        <Type>{}</Type>\n", 
                    self.escape_xml(&data.release_type)));
                xml.push_str(&format!("        <Title>{}</Title>\n", 
                    self.escape_xml(&data.title)));
                xml.push_str(&format!("        <Artist>{}</Artist>\n", 
                    self.escape_xml(&data.artist)));
                
                if let Some(ref label) = data.label {
                    xml.push_str(&format!("        <Label>{}</Label>\n", 
                        self.escape_xml(label)));
                }
                
                if let Some(ref upc) = data.upc {
                    xml.push_str(&format!("        <UPC>{}</UPC>\n", 
                        self.escape_xml(upc)));
                }
                
                xml.push_str("      </ReleaseData>\n");
            }
            
            xml.push_str("    </ReleaseUpdate>\n");
        }
        
        xml.push_str("  </ReleaseUpdates>\n");
        Ok(())
    }
    
    fn serialize_deal_updates(
        &self,
        xml: &mut String,
        deal_updates: &indexmap::IndexMap<String, super::messages::DealUpdate>,
    ) -> Result<(), super::error::BuildError> {
        xml.push_str("  <DealUpdates>\n");
        
        for (deal_id, update) in deal_updates {
            xml.push_str("    <DealUpdate>\n");
            xml.push_str(&format!("      <DealId>{}</DealId>\n", 
                self.escape_xml(deal_id)));
            xml.push_str(&format!("      <DealReference>{}</DealReference>\n", 
                self.escape_xml(&update.deal_reference)));
            xml.push_str(&format!("      <Action>{}</Action>\n", update.action));
            
            xml.push_str("    </DealUpdate>\n");
        }
        
        xml.push_str("  </DealUpdates>\n");
        Ok(())
    }
    
    fn escape_xml(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

impl Default for DDEXBuilder {
    fn default() -> Self {
        Self::new()
    }
}