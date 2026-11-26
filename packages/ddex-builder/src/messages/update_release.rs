//! UpdateReleaseMessage implementation for DDEX Builder
//!
//! This module provides support for generating and applying incremental DDEX updates
//! using the UpdateReleaseMessage format, allowing efficient partial updates to
//! existing DDEX messages without resending entire catalogs.

use crate::builder::MessageHeaderRequest;
use crate::diff::types::{ChangeSet, ChangeType, SemanticChange};
use crate::diff::DiffEngine;
use crate::error::BuildError;
use chrono::{DateTime, Utc};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

/// Complete UpdateReleaseMessage structure
///
/// Represents a complete DDEX UpdateReleaseMessage that describes incremental
/// changes to an existing DDEX message. This allows efficient partial updates
/// without resending entire catalogs.
///
/// # Example
/// ```
/// use ddex_builder::messages::update_release::*;
/// use ddex_builder::builder::{LocalizedStringRequest, MessageHeaderRequest, PartyRequest};
/// use indexmap::IndexMap;
/// # let party = |name: &str| PartyRequest {
/// #     party_name: vec![LocalizedStringRequest { text: name.to_string(), language_code: None }],
/// #     party_id: Some("PADPIDA0000000001".to_string()),
/// #     party_reference: None,
/// # };
///
/// // Create an update message to modify a release title
/// let update_message = UpdateReleaseMessage {
///     header: MessageHeaderRequest {
///         message_id: Some("UPD-MSG-20241215-001".to_string()),
///         // ... other header fields
///         # message_sender: party("Label"),
///         # message_recipient: party("DSP"),
///         # message_control_type: Some("UpdateMessage".to_string()),
///         # message_created_date_time: None,
///     },
///     update_list: vec![
///         // UpdateOperation to change release title
///         # UpdateOperation {
///         #     operation_id: "OP000001".to_string(),
///         #     action: UpdateAction::Replace,
///         #     target_path: "/Release[@ReleaseId='R001']/Title".to_string(),
///         #     entity_type: EntityType::Release,
///         #     entity_id: "R001".to_string(),
///         #     old_value: Some("Old Title".to_string()),
///         #     new_value: Some("New Title".to_string()),
///         #     is_critical: false,
///         #     description: "Update release title".to_string(),
///         #     dependencies: vec![],
///         # }
///     ],
///     resource_updates: IndexMap::new(),
///     release_updates: IndexMap::new(),
///     deal_updates: IndexMap::new(),
///     update_metadata: UpdateMetadata {
///         original_message_id: "ORIG-MSG-001".to_string(),
///         # original_message_version: None,
///         # original_message_timestamp: None,
///         # update_created_timestamp: chrono::Utc::now(),
///         # update_sequence: 1,
///         # total_operations: 1,
///         # impact_level: "Low".to_string(),
///         # validation_status: ValidationStatus::Pending,
///         # custom_metadata: IndexMap::new(),
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReleaseMessage {
    /// Message header containing sender, recipient, and timing information
    pub header: MessageHeaderRequest,

    /// Ordered list of update operations to perform on the target message
    pub update_list: Vec<UpdateOperation>,

    /// Resource-specific updates indexed by resource ID
    pub resource_updates: IndexMap<String, ResourceUpdate>,

    /// Release-specific updates indexed by release ID
    pub release_updates: IndexMap<String, ReleaseUpdate>,

    /// Deal-specific updates indexed by deal ID (optional)
    pub deal_updates: IndexMap<String, DealUpdate>,

    /// Metadata describing this update operation
    pub update_metadata: UpdateMetadata,
}

/// Individual update operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOperation {
    /// Unique identifier for this update operation
    pub operation_id: String,

    /// Type of update action
    pub action: UpdateAction,

    /// Path to the element being updated
    pub target_path: String,

    /// Entity type being updated
    pub entity_type: EntityType,

    /// Entity ID being updated
    pub entity_id: String,

    /// Old value (for Replace operations)
    pub old_value: Option<String>,

    /// New value (for Add/Replace operations)
    pub new_value: Option<String>,

    /// Whether this update is critical
    pub is_critical: bool,

    /// Human-readable description
    pub description: String,

    /// Dependencies on other operations
    pub dependencies: Vec<String>,
}

/// Types of update actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpdateAction {
    /// Add a new element or attribute
    Add,
    /// Delete an existing element or attribute
    Delete,
    /// Replace an existing element or attribute
    Replace,
    /// Move an element to a different location
    Move,
}

/// Entity types that can be updated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    /// Sound recording resource
    Resource,
    /// Release
    Release,
    /// Deal
    Deal,
    /// Party information
    Party,
    /// Message metadata
    Metadata,
}

/// Resource update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpdate {
    /// Resource identifier
    pub resource_id: String,

    /// Resource reference (may change)
    pub resource_reference: String,

    /// Update action for this resource
    pub action: UpdateAction,

    /// Updated resource data (for Add/Replace)
    pub resource_data: Option<ResourceData>,

    /// Technical details updates
    pub technical_updates: Vec<TechnicalUpdate>,

    /// Metadata updates
    pub metadata_updates: IndexMap<String, String>,
}

/// Release update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseUpdate {
    /// Release identifier
    pub release_id: String,

    /// Release reference (may change)
    pub release_reference: String,

    /// Update action for this release
    pub action: UpdateAction,

    /// Updated release data (for Add/Replace)
    pub release_data: Option<ReleaseData>,

    /// Track updates within this release
    pub track_updates: Vec<TrackUpdate>,

    /// Resource reference updates
    pub resource_reference_updates: Vec<ReferenceUpdate>,

    /// Metadata updates
    pub metadata_updates: IndexMap<String, String>,
}

/// Deal update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealUpdate {
    /// Deal identifier
    pub deal_id: String,

    /// Deal reference
    pub deal_reference: String,

    /// Update action for this deal
    pub action: UpdateAction,

    /// Updated deal data
    pub deal_data: Option<DealData>,

    /// Terms updates
    pub terms_updates: Vec<TermsUpdate>,
}

/// Updated resource information
///
/// Represents resource data in DDEX update messages, containing all the
/// metadata needed to describe a sound recording, video, or other resource
/// that has been modified in an UpdateReleaseMessage.
///
/// # Example
/// ```
/// use ddex_builder::messages::update_release::{UpdatedResource, TechnicalDetails};
///
/// let resource = UpdatedResource {
///     resource_type: "SoundRecording".to_string(),
///     title: "Bohemian Rhapsody (Remastered)".to_string(),
///     artist: "Queen".to_string(),
///     isrc: Some("GBUM71505078".to_string()),
///     duration: Some("PT5M55S".to_string()), // 5 minutes 55 seconds
///     file_path: Some("/audio/queen/bohemian_rhapsody_remastered.flac".to_string()),
///     technical_details: Some(TechnicalDetails {
///         file_name: Some("bohemian_rhapsody_remastered.flac".to_string()),
///         codec_type: Some("FLAC".to_string()),
///         bit_rate: Some("1411".to_string()), // kbps
///         sample_rate: Some("44100".to_string()), // Hz
///     }),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct UpdatedResource {
    /// Type of resource (SoundRecording, Video, Image, Text, etc.)
    pub resource_type: String,
    /// Title of the resource/track
    pub title: String,
    /// Artist name for this resource
    pub artist: String,
    /// International Standard Recording Code (12-character alphanumeric)
    pub isrc: Option<String>,
    /// Duration in ISO 8601 format (e.g., "PT3M45S" for 3 minutes 45 seconds)
    pub duration: Option<String>,
    /// File path or URL to the resource file
    pub file_path: Option<String>,
    /// Technical details of the resource (codec, bitrate, etc.)
    pub technical_details: Option<TechnicalDetails>,
}

/// Resource data for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceData {
    /// Type of resource (SoundRecording, Video, Image, etc.)
    pub resource_type: String,
    /// Title of the resource
    pub title: String,
    /// Artist name
    pub artist: String,
    /// International Standard Recording Code
    pub isrc: Option<String>,
    /// Duration in ISO 8601 format
    pub duration: Option<String>,
    /// File path or URL
    pub file_path: Option<String>,
    /// Technical details
    pub technical_details: Option<TechnicalDetails>,
}

/// Updated release information
///
/// Represents release data in DDEX update messages, containing all the
/// metadata needed to describe a music release (album, single, EP, etc.)
/// that has been modified in an UpdateReleaseMessage.
///
/// # Example
/// ```
/// use ddex_builder::messages::update_release::UpdatedRelease;
///
/// let release = UpdatedRelease {
///     release_type: "Album".to_string(),
///     title: "The Greatest Hits (Deluxe Edition)".to_string(),
///     artist: "The Beatles".to_string(),
///     label: Some("Apple Records".to_string()),
///     upc: Some("602537518739".to_string()),
///     release_date: Some("2024-01-15".to_string()),
///     genre: Some("Rock".to_string()),
///     resource_references: vec![
///         "R12345678".to_string(),
///         "R87654321".to_string(),
///         "R11223344".to_string(),
///     ],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct UpdatedRelease {
    /// Release type (Album, Single, EP, Compilation, etc.)
    pub release_type: String,
    /// Title of the release
    pub title: String,
    /// Main artist name for the release
    pub artist: String,
    /// Record label name
    pub label: Option<String>,
    /// Universal Product Code (12-digit barcode identifier)
    pub upc: Option<String>,
    /// Release date in YYYY-MM-DD format
    pub release_date: Option<String>,
    /// Primary genre classification
    pub genre: Option<String>,
    /// References to included resources (tracks/recordings)
    pub resource_references: Vec<String>,
}

/// Release data for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseData {
    /// Release type (Single, Album, EP, etc.)
    pub release_type: String,
    /// Release title
    pub title: String,
    /// Main artist name
    pub artist: String,
    /// Record label name
    pub label: Option<String>,
    /// Universal Product Code
    pub upc: Option<String>,
    /// Release date in YYYY-MM-DD format
    pub release_date: Option<String>,
    /// Primary genre
    pub genre: Option<String>,
    /// References to included resources
    pub resource_references: Vec<String>,
}

/// Deal data for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealData {
    /// Type of commercial model (e.g., "PayAsYouGoModel")
    pub commercial_model_type: String,
    /// Territory codes where deal applies
    pub territory_codes: Vec<String>,
    /// Deal start date in YYYY-MM-DD format
    pub start_date: Option<String>,
    /// Deal end date in YYYY-MM-DD format
    pub end_date: Option<String>,
    /// Pricing information
    pub price: Option<PriceData>,
}

/// Field update details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalUpdate {
    /// Name of the field being updated
    pub field_name: String,
    /// Previous value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
    /// Type of update action
    pub update_action: UpdateAction,
}

/// Track update within a release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackUpdate {
    /// Unique track identifier
    pub track_id: String,
    /// Update action to perform
    pub action: UpdateAction,
    /// Previous resource reference
    pub old_resource_reference: Option<String>,
    /// New resource reference
    pub new_resource_reference: Option<String>,
    /// Position change information
    pub position_change: Option<PositionChange>,
}

/// Reference update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceUpdate {
    /// Previous reference value
    pub old_reference: String,
    /// New reference value
    pub new_reference: String,
    /// Type of reference being updated
    pub reference_type: String,
    /// Reason for the update
    pub update_reason: String,
}

/// Terms update for deals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermsUpdate {
    /// Name of the field being updated
    pub field_name: String,
    /// Previous value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
    /// Date when update becomes effective
    pub effective_date: Option<String>,
}

/// Position change information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionChange {
    /// Previous position in sequence
    pub old_position: usize,
    /// New position in sequence
    pub new_position: usize,
}

/// Technical details for resources
///
/// Contains technical specifications for audio/video resources,
/// including encoding information and quality parameters.
///
/// # Example
/// ```
/// use ddex_builder::messages::update_release::TechnicalDetails;
///
/// let tech_details = TechnicalDetails {
///     file_name: Some("track_01_mastered.flac".to_string()),
///     codec_type: Some("FLAC".to_string()),
///     bit_rate: Some("1411".to_string()), // kbps for lossless
///     sample_rate: Some("44100".to_string()), // Hz (CD quality)
/// };
///
/// let mp3_details = TechnicalDetails {
///     file_name: Some("track_01_compressed.mp3".to_string()),
///     codec_type: Some("MP3".to_string()),
///     bit_rate: Some("320".to_string()), // kbps
///     sample_rate: Some("44100".to_string()), // Hz
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDetails {
    /// File name of the resource
    pub file_name: Option<String>,
    /// Audio/video codec type (e.g., "MP3", "FLAC", "AAC", "H.264")
    pub codec_type: Option<String>,
    /// Bit rate in kilobits per second (kbps)
    pub bit_rate: Option<String>,
    /// Sample rate in Hz (e.g., "44100", "48000", "96000")
    pub sample_rate: Option<String>,
}

/// Pricing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    /// Price amount
    pub amount: String,
    /// ISO currency code
    pub currency_code: String,
    /// Type of price (e.g., "WholesalePrice")
    pub price_type: Option<String>,
}

/// Metadata about the update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMetadata {
    /// Original message ID being updated
    pub original_message_id: String,

    /// Version of the original message
    pub original_message_version: Option<String>,

    /// Timestamp of the original message
    pub original_message_timestamp: Option<DateTime<Utc>>,

    /// Update creation timestamp
    pub update_created_timestamp: DateTime<Utc>,

    /// Update sequence number (for ordering)
    pub update_sequence: u64,

    /// Total number of operations in this update
    pub total_operations: usize,

    /// Estimated impact level
    pub impact_level: String,

    /// Update validation status
    pub validation_status: ValidationStatus,

    /// Additional metadata
    pub custom_metadata: IndexMap<String, String>,
}

/// Validation status for updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStatus {
    /// Update has been validated and is safe to apply
    Validated,
    /// Update has validation warnings but can be applied
    WarningsOnly,
    /// Update has validation errors and should not be applied
    Invalid,
    /// Update validation is pending
    Pending,
}

/// Configuration for update generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Include non-critical changes in updates
    pub include_non_critical: bool,

    /// Maximum number of operations per update message
    pub max_operations_per_update: usize,

    /// Whether to validate references during update generation
    pub validate_references: bool,

    /// Whether to optimize reference updates
    pub optimize_references: bool,

    /// Fields to exclude from updates
    pub excluded_fields: IndexSet<String>,

    /// Custom update priorities
    pub update_priorities: IndexMap<String, u8>,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        let mut excluded_fields = IndexSet::new();
        excluded_fields.insert("MessageId".to_string());
        excluded_fields.insert("MessageCreatedDateTime".to_string());

        Self {
            include_non_critical: true,
            max_operations_per_update: 1000,
            validate_references: true,
            optimize_references: true,
            excluded_fields,
            update_priorities: IndexMap::new(),
        }
    }
}

/// Update generation engine
pub struct UpdateGenerator {
    config: UpdateConfig,
    diff_engine: DiffEngine,
    operation_counter: u64,
}

impl UpdateGenerator {
    /// Create a new update generator
    pub fn new() -> Self {
        Self {
            config: UpdateConfig::default(),
            diff_engine: DiffEngine::new(),
            operation_counter: 0,
        }
    }

    /// Create a new update generator with custom configuration
    pub fn new_with_config(config: UpdateConfig) -> Self {
        Self {
            config,
            diff_engine: DiffEngine::new(),
            operation_counter: 0,
        }
    }

    /// Generate an UpdateReleaseMessage from two DDEX messages
    pub fn create_update(
        &mut self,
        original_xml: &str,
        updated_xml: &str,
        original_message_id: &str,
    ) -> Result<UpdateReleaseMessage, BuildError> {
        // Parse both messages to ASTs
        let original_ast = self.parse_xml_to_ast(original_xml)?;
        let updated_ast = self.parse_xml_to_ast(updated_xml)?;

        // Generate semantic diff
        let changeset = self.diff_engine.diff(&original_ast, &updated_ast)?;

        // Convert changeset to update operations
        let update_operations = self.changeset_to_operations(&changeset)?;

        // Group operations by entity type
        let (resource_updates, release_updates, deal_updates) =
            self.group_operations_by_entity(&update_operations)?;

        // Create update metadata
        let metadata =
            self.create_update_metadata(original_message_id, &update_operations, &changeset);

        // Generate message header
        let header = self.create_update_header(original_message_id, &metadata);

        let update_message = UpdateReleaseMessage {
            header,
            update_list: update_operations,
            resource_updates,
            release_updates,
            deal_updates,
            update_metadata: metadata,
        };

        // Validate the update
        self.validate_update(&update_message)?;

        Ok(update_message)
    }

    /// Not implemented in 0.5.x: returns an error. Before 0.5.1 every update operation was a
    /// no-op and the "updated" message was an XML declaration and a comment, returned as
    /// success.
    pub fn apply_update(
        &self,
        _base_xml: &str,
        _update: &UpdateReleaseMessage,
    ) -> Result<String, BuildError> {
        Err(crate::round_trip::not_implemented(
            "UpdateGenerator::apply_update",
            "update operations are not applied to a base message yet",
        ))
    }

    /// Validate an update for consistency and safety
    pub fn validate_update(
        &self,
        update: &UpdateReleaseMessage,
    ) -> Result<ValidationStatus, BuildError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate operations
        for operation in &update.update_list {
            if let Err(e) = self.validate_operation(operation, update) {
                errors.push(format!("Operation {}: {}", operation.operation_id, e));
            }
        }

        // Validate references
        if self.config.validate_references {
            if let Err(e) = self.validate_references(update) {
                errors.push(format!("Reference validation: {}", e));
            }
        }

        // Validate dependencies
        if let Err(e) = self.validate_dependencies(&update.update_list) {
            errors.push(format!("Dependency validation: {}", e));
        }

        // Check for conflicts
        let conflicts = self.detect_conflicts(&update.update_list)?;
        if !conflicts.is_empty() {
            warnings.push(format!("Found {} potential conflicts", conflicts.len()));
        }

        if !errors.is_empty() {
            Err(BuildError::ValidationFailed { errors })
        } else if !warnings.is_empty() {
            Ok(ValidationStatus::WarningsOnly)
        } else {
            Ok(ValidationStatus::Validated)
        }
    }

    // Private helper methods

    fn parse_xml_to_ast(&self, xml: &str) -> Result<crate::ast::AST, BuildError> {
        // Simplified XML parsing - in production, use proper DDEX parser
        let root = crate::ast::Element::new("NewReleaseMessage").with_text(xml);
        Ok(crate::ast::AST {
            root,
            namespaces: IndexMap::new(),
            schema_location: None,
        })
    }

    fn changeset_to_operations(
        &mut self,
        changeset: &ChangeSet,
    ) -> Result<Vec<UpdateOperation>, BuildError> {
        let mut operations = Vec::new();

        for change in &changeset.changes {
            let operation = self.semantic_change_to_operation(change)?;
            operations.push(operation);
        }

        Ok(operations)
    }

    fn semantic_change_to_operation(
        &mut self,
        change: &SemanticChange,
    ) -> Result<UpdateOperation, BuildError> {
        self.operation_counter += 1;

        let action = match change.change_type {
            ChangeType::ElementAdded | ChangeType::AttributeAdded => UpdateAction::Add,
            ChangeType::ElementRemoved | ChangeType::AttributeRemoved => UpdateAction::Delete,
            ChangeType::ElementMoved => UpdateAction::Move,
            _ => UpdateAction::Replace,
        };

        let entity_type = self.determine_entity_type(&change.path);
        let entity_id = self.extract_entity_id(&change.path)?;

        Ok(UpdateOperation {
            operation_id: format!("OP{:06}", self.operation_counter),
            action,
            target_path: change.path.to_string(),
            entity_type,
            entity_id,
            old_value: change.old_value.clone(),
            new_value: change.new_value.clone(),
            is_critical: change.is_critical,
            description: change.description.clone(),
            dependencies: Vec::new(), // Will be filled in later pass
        })
    }

    fn determine_entity_type(&self, path: &crate::diff::types::DiffPath) -> EntityType {
        let path_str = path.to_string().to_lowercase();

        if path_str.contains("resource") {
            EntityType::Resource
        } else if path_str.contains("release") {
            EntityType::Release
        } else if path_str.contains("deal") {
            EntityType::Deal
        } else if path_str.contains("party") {
            EntityType::Party
        } else {
            EntityType::Metadata
        }
    }

    #[allow(
        clippy::disallowed_methods,
        reason = "known bug D11: removed in 0.6.0 determinism PR"
    )]
    fn extract_entity_id(&self, path: &crate::diff::types::DiffPath) -> Result<String, BuildError> {
        // Extract entity ID from path - simplified implementation
        let path_str = path.to_string();
        if let Some(id_start) = path_str.find("Id=") {
            let id_part = &path_str[id_start + 3..];
            if let Some(id_end) = id_part.find(&[']', '/', '@'][..]) {
                Ok(id_part[..id_end].to_string())
            } else {
                Ok(id_part.to_string())
            }
        } else {
            let uuid_str = uuid::Uuid::new_v4().to_string();
            Ok(format!("unknown_{}", &uuid_str[..8]))
        }
    }

    fn group_operations_by_entity(
        &self,
        operations: &[UpdateOperation],
    ) -> Result<
        (
            IndexMap<String, ResourceUpdate>,
            IndexMap<String, ReleaseUpdate>,
            IndexMap<String, DealUpdate>,
        ),
        BuildError,
    > {
        let mut resource_updates = IndexMap::new();
        let mut release_updates = IndexMap::new();
        let mut deal_updates = IndexMap::new();

        for operation in operations {
            match operation.entity_type {
                EntityType::Resource => {
                    let resource_update = self.operation_to_resource_update(operation)?;
                    resource_updates.insert(operation.entity_id.clone(), resource_update);
                }
                EntityType::Release => {
                    let release_update = self.operation_to_release_update(operation)?;
                    release_updates.insert(operation.entity_id.clone(), release_update);
                }
                EntityType::Deal => {
                    let deal_update = self.operation_to_deal_update(operation)?;
                    deal_updates.insert(operation.entity_id.clone(), deal_update);
                }
                _ => {} // Handle metadata and party updates separately if needed
            }
        }

        Ok((resource_updates, release_updates, deal_updates))
    }

    fn operation_to_resource_update(
        &self,
        operation: &UpdateOperation,
    ) -> Result<ResourceUpdate, BuildError> {
        Ok(ResourceUpdate {
            resource_id: operation.entity_id.clone(),
            resource_reference: format!(
                "R{:06}",
                operation.operation_id[2..].parse::<u32>().unwrap_or(0)
            ),
            action: operation.action,
            resource_data: None, // Would be populated from operation details
            technical_updates: Vec::new(),
            metadata_updates: IndexMap::new(),
        })
    }

    fn operation_to_release_update(
        &self,
        operation: &UpdateOperation,
    ) -> Result<ReleaseUpdate, BuildError> {
        Ok(ReleaseUpdate {
            release_id: operation.entity_id.clone(),
            release_reference: format!(
                "REL{:06}",
                operation.operation_id[2..].parse::<u32>().unwrap_or(0)
            ),
            action: operation.action,
            release_data: None, // Would be populated from operation details
            track_updates: Vec::new(),
            resource_reference_updates: Vec::new(),
            metadata_updates: IndexMap::new(),
        })
    }

    fn operation_to_deal_update(
        &self,
        operation: &UpdateOperation,
    ) -> Result<DealUpdate, BuildError> {
        Ok(DealUpdate {
            deal_id: operation.entity_id.clone(),
            deal_reference: format!(
                "D{:06}",
                operation.operation_id[2..].parse::<u32>().unwrap_or(0)
            ),
            action: operation.action,
            deal_data: None, // Would be populated from operation details
            terms_updates: Vec::new(),
        })
    }

    #[allow(
        clippy::disallowed_methods,
        reason = "known bug D10: removed in 0.6.0 determinism PR"
    )]
    fn create_update_metadata(
        &self,
        original_message_id: &str,
        operations: &[UpdateOperation],
        changeset: &ChangeSet,
    ) -> UpdateMetadata {
        UpdateMetadata {
            original_message_id: original_message_id.to_string(),
            original_message_version: None,
            original_message_timestamp: None,
            update_created_timestamp: Utc::now(),
            update_sequence: 1,
            total_operations: operations.len(),
            impact_level: changeset.impact_level().to_string(),
            validation_status: ValidationStatus::Pending,
            custom_metadata: IndexMap::new(),
        }
    }

    fn create_update_header(
        &self,
        original_message_id: &str,
        metadata: &UpdateMetadata,
    ) -> MessageHeaderRequest {
        MessageHeaderRequest {
            message_id: Some(format!(
                "UPD-{}-{:04}",
                original_message_id, metadata.update_sequence
            )),
            message_sender: crate::builder::PartyRequest {
                party_name: vec![crate::builder::LocalizedStringRequest {
                    text: "DDEX Builder Update Engine".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_recipient: crate::builder::PartyRequest {
                party_name: vec![crate::builder::LocalizedStringRequest {
                    text: "Update Recipient".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_control_type: Some("UpdateMessage".to_string()),
            message_created_date_time: Some(metadata.update_created_timestamp.to_rfc3339()),
        }
    }

    fn validate_operation(
        &self,
        operation: &UpdateOperation,
        _update: &UpdateReleaseMessage,
    ) -> Result<(), BuildError> {
        // Validate individual operation
        if operation.entity_id.is_empty() {
            return Err(BuildError::InvalidFormat {
                field: "entity_id".to_string(),
                message: "Entity ID cannot be empty".to_string(),
            });
        }

        // Validate action compatibility
        match operation.action {
            UpdateAction::Add => {
                if operation.new_value.is_none() {
                    return Err(BuildError::InvalidFormat {
                        field: "new_value".to_string(),
                        message: "Add operation requires new_value".to_string(),
                    });
                }
            }
            UpdateAction::Delete => {
                if operation.old_value.is_none() {
                    return Err(BuildError::InvalidFormat {
                        field: "old_value".to_string(),
                        message: "Delete operation requires old_value".to_string(),
                    });
                }
            }
            UpdateAction::Replace => {
                if operation.old_value.is_none() || operation.new_value.is_none() {
                    return Err(BuildError::InvalidFormat {
                        field: "values".to_string(),
                        message: "Replace operation requires both old_value and new_value"
                            .to_string(),
                    });
                }
            }
            UpdateAction::Move => {
                // Move operations have specific validation requirements
            }
        }

        Ok(())
    }

    fn validate_references(&self, update: &UpdateReleaseMessage) -> Result<(), BuildError> {
        // Validate that all referenced entities exist
        let mut referenced_resources = IndexSet::new();
        let mut referenced_releases = IndexSet::new();

        // Collect all references
        for operation in &update.update_list {
            match operation.entity_type {
                EntityType::Resource => {
                    referenced_resources.insert(operation.entity_id.clone());
                }
                EntityType::Release => {
                    referenced_releases.insert(operation.entity_id.clone());
                }
                _ => {}
            }
        }

        // Check that referenced entities have corresponding updates
        for resource_id in &referenced_resources {
            if !update.resource_updates.contains_key(resource_id) {
                return Err(BuildError::InvalidReference {
                    reference: resource_id.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_dependencies(&self, operations: &[UpdateOperation]) -> Result<(), BuildError> {
        let operation_ids: IndexSet<_> = operations.iter().map(|op| &op.operation_id).collect();

        for operation in operations {
            for dependency in &operation.dependencies {
                if !operation_ids.contains(&dependency) {
                    return Err(BuildError::InvalidReference {
                        reference: format!("Missing dependency: {}", dependency),
                    });
                }
            }
        }

        Ok(())
    }

    fn detect_conflicts(&self, operations: &[UpdateOperation]) -> Result<Vec<String>, BuildError> {
        let mut conflicts = Vec::new();

        // Check for operations targeting the same path
        let mut path_operations: IndexMap<String, Vec<&UpdateOperation>> = IndexMap::new();

        for operation in operations {
            path_operations
                .entry(operation.target_path.clone())
                .or_default()
                .push(operation);
        }

        for (path, ops) in path_operations {
            if ops.len() > 1 {
                let conflicting_ops: Vec<_> = ops.iter().map(|op| &op.operation_id).collect();
                conflicts.push(format!(
                    "Path {} has conflicting operations: {:?}",
                    path, conflicting_ops
                ));
            }
        }

        Ok(conflicts)
    }
}

impl Default for UpdateGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Display implementations for better debugging
impl std::fmt::Display for UpdateAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateAction::Add => write!(f, "Add"),
            UpdateAction::Delete => write!(f, "Delete"),
            UpdateAction::Replace => write!(f, "Replace"),
            UpdateAction::Move => write!(f, "Move"),
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Resource => write!(f, "Resource"),
            EntityType::Release => write!(f, "Release"),
            EntityType::Deal => write!(f, "Deal"),
            EntityType::Party => write!(f, "Party"),
            EntityType::Metadata => write!(f, "Metadata"),
        }
    }
}

impl std::fmt::Display for ValidationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationStatus::Validated => write!(f, "Validated"),
            ValidationStatus::WarningsOnly => write!(f, "Warnings Only"),
            ValidationStatus::Invalid => write!(f, "Invalid"),
            ValidationStatus::Pending => write!(f, "Pending"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_generator_creation() {
        let generator = UpdateGenerator::new();
        assert_eq!(generator.operation_counter, 0);
    }

    #[test]
    fn test_update_config_defaults() {
        let config = UpdateConfig::default();
        assert!(config.include_non_critical);
        assert_eq!(config.max_operations_per_update, 1000);
        assert!(config.validate_references);
    }

    #[test]
    #[allow(
        clippy::disallowed_methods,
        reason = "D10: unit-test fixture timestamp; not build output"
    )]
    fn test_operation_validation() {
        let generator = UpdateGenerator::new();

        let operation = UpdateOperation {
            operation_id: "OP000001".to_string(),
            action: UpdateAction::Add,
            target_path: "/Release/Title".to_string(),
            entity_type: EntityType::Release,
            entity_id: "release-001".to_string(),
            old_value: None,
            new_value: Some("New Title".to_string()),
            is_critical: false,
            description: "Update title".to_string(),
            dependencies: Vec::new(),
        };

        let update = UpdateReleaseMessage {
            header: MessageHeaderRequest {
                message_id: Some("TEST-001".to_string()),
                message_sender: crate::builder::PartyRequest {
                    party_name: vec![crate::builder::LocalizedStringRequest {
                        text: "Test".to_string(),
                        language_code: None,
                    }],
                    party_id: None,
                    party_reference: None,
                },
                message_recipient: crate::builder::PartyRequest {
                    party_name: vec![crate::builder::LocalizedStringRequest {
                        text: "Test".to_string(),
                        language_code: None,
                    }],
                    party_id: None,
                    party_reference: None,
                },
                message_control_type: None,
                message_created_date_time: None,
            },
            update_list: vec![operation.clone()],
            resource_updates: IndexMap::new(),
            release_updates: IndexMap::new(),
            deal_updates: IndexMap::new(),
            update_metadata: UpdateMetadata {
                original_message_id: "ORIG-001".to_string(),
                original_message_version: None,
                original_message_timestamp: None,
                update_created_timestamp: Utc::now(),
                update_sequence: 1,
                total_operations: 1,
                impact_level: "Low".to_string(),
                validation_status: ValidationStatus::Pending,
                custom_metadata: IndexMap::new(),
            },
        };

        assert!(generator.validate_operation(&operation, &update).is_ok());
    }

    #[test]
    fn test_entity_type_determination() {
        let generator = UpdateGenerator::new();

        let resource_path = crate::diff::types::DiffPath::root()
            .with_element("ResourceList")
            .with_element("SoundRecording");

        let release_path = crate::diff::types::DiffPath::root()
            .with_element("ReleaseList")
            .with_element("Release");

        assert_eq!(
            generator.determine_entity_type(&resource_path),
            EntityType::Resource
        );
        assert_eq!(
            generator.determine_entity_type(&release_path),
            EntityType::Release
        );
    }
}
