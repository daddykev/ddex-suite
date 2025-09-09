//! Reference linker for automatic ID generation and relationship management
//!
//! The linker module ensures deterministic reference generation and maintains
//! relationships between DDEX entities (releases, resources, parties, deals).

mod auto_linker;
mod reference_generator;
mod relationship_manager;
mod types;

pub use auto_linker::AutoLinker;
pub use reference_generator::ReferenceGenerator;
pub use relationship_manager::RelationshipManager;
pub use types::*;

use indexmap::IndexMap;  // Removed IndexSet (unused)
// Removed std::fmt (unused)

/// Main reference linker that coordinates all linking operations
#[derive(Debug, Clone)]
pub struct ReferenceLinker {
    /// Reference generator for creating deterministic IDs
    generator: ReferenceGenerator,
    
    /// Relationship manager for tracking entity connections
    relationships: RelationshipManager,
    
    /// Auto-linker for automatic relationship creation
    auto_linker: AutoLinker,
    
    /// Configuration for the linker
    config: LinkerConfig,
}

impl ReferenceLinker {
    /// Create a new reference linker with default configuration
    pub fn new() -> Self {
        Self::with_config(LinkerConfig::default())
    }
    
    /// Create a new reference linker with custom configuration
    pub fn with_config(config: LinkerConfig) -> Self {
        Self {
            generator: ReferenceGenerator::new(config.reference_style.clone()),
            relationships: RelationshipManager::new(),
            auto_linker: AutoLinker::new(),
            config,
        }
    }
    
    /// Generate a new reference for an entity type
    pub fn generate_reference(&mut self, entity_type: EntityType) -> String {
        self.generator.generate(entity_type)
    }
    
    /// Register an entity with its reference
    pub fn register_entity(&mut self, entity_type: EntityType, id: String, reference: String) {
        self.relationships.register(entity_type, id, reference);
    }
    
    /// Link a release to multiple resources
    pub fn link_release_to_resources(
        &mut self,
        release_id: &str,
        resource_ids: &[String],
    ) -> Result<Vec<ReleaseResourceReference>, LinkerError> {
        // Get or generate release reference
        let release_ref = self.relationships
            .get_reference(EntityType::Release, release_id)
            .unwrap_or_else(|| {
                let ref_val = self.generator.generate(EntityType::Release);
                self.relationships.register(
                    EntityType::Release, 
                    release_id.to_string(), 
                    ref_val.clone()
                );
                ref_val
            });
        
        // Build release-resource references
        let mut references = Vec::new();
        for (sequence_no, resource_id) in resource_ids.iter().enumerate() {
            let resource_ref = self.relationships
                .get_reference(EntityType::Resource, resource_id)
                .ok_or_else(|| LinkerError::UnknownResource(resource_id.clone()))?;
            
            references.push(ReleaseResourceReference {
                release_reference: release_ref.clone(),
                resource_reference: resource_ref,
                sequence_number: sequence_no as u32 + 1,
            });
        }
        
        Ok(references)
    }
    
    /// Auto-link all entities in a build request
    pub fn auto_link_request(
        &mut self,
        request: &mut crate::builder::BuildRequest,
    ) -> Result<LinkingReport, LinkerError> {
        self.auto_linker.process_request(request, &mut self.generator, &mut self.relationships)
    }
    
    /// Get all registered references for debugging
    pub fn get_all_references(&self) -> IndexMap<EntityType, IndexMap<String, String>> {
        self.relationships.get_all()
    }
    
    /// Validate all references in the system
    pub fn validate_references(&self) -> Result<(), Vec<LinkerError>> {
        self.relationships.validate()
    }
}

impl Default for ReferenceLinker {
    fn default() -> Self {
        Self::new()
    }
}