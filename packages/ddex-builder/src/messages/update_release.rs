//! UpdateReleaseMessage implementation for DDEX Builder
//! 
//! This module provides support for generating and applying incremental DDEX updates
//! using the UpdateReleaseMessage format, allowing efficient partial updates to
//! existing DDEX messages without resending entire catalogs.

use crate::error::BuildError;
use crate::builder::MessageHeaderRequest;
use crate::diff::DiffEngine;
use crate::diff::types::{ChangeSet, SemanticChange, ChangeType};
use serde::{Serialize, Deserialize};
use indexmap::{IndexMap, IndexSet};
use chrono::{DateTime, Utc};

/// Complete UpdateReleaseMessage structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReleaseMessage {
    /// Message header information
    pub header: MessageHeaderRequest,
    
    /// List of update operations to perform
    pub update_list: Vec<UpdateOperation>,
    
    /// Resources that need to be updated
    pub resource_updates: IndexMap<String, ResourceUpdate>,
    
    /// Releases that need to be updated  
    pub release_updates: IndexMap<String, ReleaseUpdate>,
    
    /// Deal updates (if any)
    pub deal_updates: IndexMap<String, DealUpdate>,
    
    /// Metadata about this update
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

/// Resource data for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceData {
    pub resource_type: String,
    pub title: String,
    pub artist: String,
    pub isrc: Option<String>,
    pub duration: Option<String>,
    pub file_path: Option<String>,
    pub technical_details: Option<TechnicalDetails>,
}

/// Release data for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseData {
    pub release_type: String,
    pub title: String,
    pub artist: String,
    pub label: Option<String>,
    pub upc: Option<String>,
    pub release_date: Option<String>,
    pub genre: Option<String>,
    pub resource_references: Vec<String>,
}

/// Deal data for updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealData {
    pub commercial_model_type: String,
    pub territory_codes: Vec<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub price: Option<PriceData>,
}

/// Technical update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalUpdate {
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub update_action: UpdateAction,
}

/// Track update within a release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackUpdate {
    pub track_id: String,
    pub action: UpdateAction,
    pub old_resource_reference: Option<String>,
    pub new_resource_reference: Option<String>,
    pub position_change: Option<PositionChange>,
}

/// Reference update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceUpdate {
    pub old_reference: String,
    pub new_reference: String,
    pub reference_type: String,
    pub update_reason: String,
}

/// Terms update for deals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermsUpdate {
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub effective_date: Option<String>,
}

/// Position change information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionChange {
    pub old_position: usize,
    pub new_position: usize,
}

/// Technical details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDetails {
    pub file_name: Option<String>,
    pub codec_type: Option<String>,
    pub bit_rate: Option<String>,
    pub sample_rate: Option<String>,
}

/// Price data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub amount: String,
    pub currency_code: String,
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
        let metadata = self.create_update_metadata(
            original_message_id,
            &update_operations,
            &changeset,
        );
        
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
    
    /// Apply an update to a base message to produce a new complete message
    pub fn apply_update(
        &self,
        base_xml: &str,
        update: &UpdateReleaseMessage,
    ) -> Result<String, BuildError> {
        // Parse base message
        let mut base_ast = self.parse_xml_to_ast(base_xml)?;
        
        // Apply each operation in dependency order
        let ordered_operations = self.order_operations_by_dependencies(&update.update_list)?;
        
        for operation in &ordered_operations {
            self.apply_operation_to_ast(&mut base_ast, operation)?;
        }
        
        // Apply entity-level updates
        self.apply_resource_updates(&mut base_ast, &update.resource_updates)?;
        self.apply_release_updates(&mut base_ast, &update.release_updates)?;
        self.apply_deal_updates(&mut base_ast, &update.deal_updates)?;
        
        // Serialize back to XML
        self.ast_to_xml(&base_ast)
    }
    
    /// Validate an update for consistency and safety
    pub fn validate_update(&self, update: &UpdateReleaseMessage) -> Result<ValidationStatus, BuildError> {
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
    
    fn changeset_to_operations(&mut self, changeset: &ChangeSet) -> Result<Vec<UpdateOperation>, BuildError> {
        let mut operations = Vec::new();
        
        for change in &changeset.changes {
            let operation = self.semantic_change_to_operation(change)?;
            operations.push(operation);
        }
        
        Ok(operations)
    }
    
    fn semantic_change_to_operation(&mut self, change: &SemanticChange) -> Result<UpdateOperation, BuildError> {
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
    ) -> Result<(IndexMap<String, ResourceUpdate>, IndexMap<String, ReleaseUpdate>, IndexMap<String, DealUpdate>), BuildError> {
        let mut resource_updates = IndexMap::new();
        let mut release_updates = IndexMap::new();
        let mut deal_updates = IndexMap::new();
        
        for operation in operations {
            match operation.entity_type {
                EntityType::Resource => {
                    let resource_update = self.operation_to_resource_update(operation)?;
                    resource_updates.insert(operation.entity_id.clone(), resource_update);
                },
                EntityType::Release => {
                    let release_update = self.operation_to_release_update(operation)?;
                    release_updates.insert(operation.entity_id.clone(), release_update);
                },
                EntityType::Deal => {
                    let deal_update = self.operation_to_deal_update(operation)?;
                    deal_updates.insert(operation.entity_id.clone(), deal_update);
                },
                _ => {}, // Handle metadata and party updates separately if needed
            }
        }
        
        Ok((resource_updates, release_updates, deal_updates))
    }
    
    fn operation_to_resource_update(&self, operation: &UpdateOperation) -> Result<ResourceUpdate, BuildError> {
        Ok(ResourceUpdate {
            resource_id: operation.entity_id.clone(),
            resource_reference: format!("R{:06}", operation.operation_id[2..].parse::<u32>().unwrap_or(0)),
            action: operation.action,
            resource_data: None, // Would be populated from operation details
            technical_updates: Vec::new(),
            metadata_updates: IndexMap::new(),
        })
    }
    
    fn operation_to_release_update(&self, operation: &UpdateOperation) -> Result<ReleaseUpdate, BuildError> {
        Ok(ReleaseUpdate {
            release_id: operation.entity_id.clone(),
            release_reference: format!("REL{:06}", operation.operation_id[2..].parse::<u32>().unwrap_or(0)),
            action: operation.action,
            release_data: None, // Would be populated from operation details
            track_updates: Vec::new(),
            resource_reference_updates: Vec::new(),
            metadata_updates: IndexMap::new(),
        })
    }
    
    fn operation_to_deal_update(&self, operation: &UpdateOperation) -> Result<DealUpdate, BuildError> {
        Ok(DealUpdate {
            deal_id: operation.entity_id.clone(),
            deal_reference: format!("D{:06}", operation.operation_id[2..].parse::<u32>().unwrap_or(0)),
            action: operation.action,
            deal_data: None, // Would be populated from operation details
            terms_updates: Vec::new(),
        })
    }
    
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
    
    fn create_update_header(&self, original_message_id: &str, metadata: &UpdateMetadata) -> MessageHeaderRequest {
        MessageHeaderRequest {
            message_id: Some(format!("UPD-{}-{:04}", original_message_id, metadata.update_sequence)),
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
    
    fn order_operations_by_dependencies(&self, operations: &[UpdateOperation]) -> Result<Vec<UpdateOperation>, BuildError> {
        // Simplified topological sort - in production, implement proper dependency resolution
        let mut ordered = operations.to_vec();
        
        // Sort by operation ID as a simple ordering
        ordered.sort_by(|a, b| a.operation_id.cmp(&b.operation_id));
        
        Ok(ordered)
    }
    
    fn apply_operation_to_ast(&self, _ast: &mut crate::ast::AST, operation: &UpdateOperation) -> Result<(), BuildError> {
        // Simplified AST modification - in production, implement proper AST updates
        match operation.action {
            UpdateAction::Add => {
                // Add logic here
            },
            UpdateAction::Delete => {
                // Delete logic here
            },
            UpdateAction::Replace => {
                // Replace logic here
            },
            UpdateAction::Move => {
                // Move logic here
            },
        }
        Ok(())
    }
    
    fn apply_resource_updates(&self, _ast: &mut crate::ast::AST, updates: &IndexMap<String, ResourceUpdate>) -> Result<(), BuildError> {
        // Apply resource-specific updates
        for (_resource_id, _update) in updates {
            // Implementation would modify AST based on resource update
        }
        Ok(())
    }
    
    fn apply_release_updates(&self, _ast: &mut crate::ast::AST, updates: &IndexMap<String, ReleaseUpdate>) -> Result<(), BuildError> {
        // Apply release-specific updates
        for (_release_id, _update) in updates {
            // Implementation would modify AST based on release update
        }
        Ok(())
    }
    
    fn apply_deal_updates(&self, _ast: &mut crate::ast::AST, updates: &IndexMap<String, DealUpdate>) -> Result<(), BuildError> {
        // Apply deal-specific updates
        for (_deal_id, _update) in updates {
            // Implementation would modify AST based on deal update
        }
        Ok(())
    }
    
    fn ast_to_xml(&self, _ast: &crate::ast::AST) -> Result<String, BuildError> {
        // Simplified XML serialization - in production, use proper XML generation
        Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!-- Updated DDEX Message -->\n"))
    }
    
    fn validate_operation(&self, operation: &UpdateOperation, _update: &UpdateReleaseMessage) -> Result<(), BuildError> {
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
            },
            UpdateAction::Delete => {
                if operation.old_value.is_none() {
                    return Err(BuildError::InvalidFormat {
                        field: "old_value".to_string(),
                        message: "Delete operation requires old_value".to_string(),
                    });
                }
            },
            UpdateAction::Replace => {
                if operation.old_value.is_none() || operation.new_value.is_none() {
                    return Err(BuildError::InvalidFormat {
                        field: "values".to_string(),
                        message: "Replace operation requires both old_value and new_value".to_string(),
                    });
                }
            },
            UpdateAction::Move => {
                // Move operations have specific validation requirements
            },
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
                },
                EntityType::Release => {
                    referenced_releases.insert(operation.entity_id.clone());
                },
                _ => {},
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
            path_operations.entry(operation.target_path.clone())
                .or_default()
                .push(operation);
        }
        
        for (path, ops) in path_operations {
            if ops.len() > 1 {
                let conflicting_ops: Vec<_> = ops.iter().map(|op| &op.operation_id).collect();
                conflicts.push(format!("Path {} has conflicting operations: {:?}", path, conflicting_ops));
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
        
        assert_eq!(generator.determine_entity_type(&resource_path), EntityType::Resource);
        assert_eq!(generator.determine_entity_type(&release_path), EntityType::Release);
    }
}