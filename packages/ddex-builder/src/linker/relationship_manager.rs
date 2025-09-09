//! Manages relationships between DDEX entities

use super::types::{EntityType, LinkerError};
use indexmap::{IndexMap, IndexSet};

/// Manages entity relationships and reference lookups
#[derive(Debug, Clone, Default)]
pub struct RelationshipManager {
    /// Maps from entity type to (id -> reference) mappings
    /// Using IndexMap for deterministic iteration
    registry: IndexMap<EntityType, IndexMap<String, String>>,
    
    /// Reverse lookup: reference -> (entity_type, id)
    reverse_lookup: IndexMap<String, (EntityType, String)>,
    
    /// Track relationships between entities
    relationships: IndexMap<String, IndexSet<String>>,
}

impl RelationshipManager {
    /// Create a new relationship manager
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register an entity with its reference
    pub fn register(&mut self, entity_type: EntityType, id: String, reference: String) {
        // Forward lookup
        self.registry
            .entry(entity_type)
            .or_insert_with(IndexMap::new)
            .insert(id.clone(), reference.clone());
        
        // Reverse lookup
        self.reverse_lookup.insert(reference, (entity_type, id));
    }
    
    /// Get reference for an entity
    pub fn get_reference(&self, entity_type: EntityType, id: &str) -> Option<String> {
        self.registry
            .get(&entity_type)
            .and_then(|refs| refs.get(id))
            .cloned()
    }
    
    /// Get entity info by reference
    pub fn get_entity_by_reference(&self, reference: &str) -> Option<(EntityType, String)> {
        self.reverse_lookup.get(reference).cloned()
    }
    
    /// Check if a reference exists
    pub fn reference_exists(&self, reference: &str) -> bool {
        self.reverse_lookup.contains_key(reference)
    }
    
    /// Add a relationship between two entities
    pub fn add_relationship(&mut self, from_ref: String, to_ref: String) {
        self.relationships
            .entry(from_ref)
            .or_insert_with(IndexSet::new)
            .insert(to_ref);
    }
    
    /// Get all related references for an entity
    pub fn get_relationships(&self, reference: &str) -> Option<&IndexSet<String>> {
        self.relationships.get(reference)
    }
    
    /// Get all registered references (for debugging)
    pub fn get_all(&self) -> IndexMap<EntityType, IndexMap<String, String>> {
        self.registry.clone()
    }
    
    /// Validate all references
    pub fn validate(&self) -> Result<(), Vec<LinkerError>> {
        let mut errors = Vec::new();
        
        // Check for orphaned relationships
        for (from_ref, to_refs) in &self.relationships {
            if !self.reference_exists(from_ref) {
                errors.push(LinkerError::OrphanedReference(from_ref.clone()));
            }
            
            for to_ref in to_refs {
                if !self.reference_exists(to_ref) {
                    errors.push(LinkerError::BrokenReference {
                        from: from_ref.clone(),
                        to: to_ref.clone(),
                    });
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Get statistics about registered entities
    pub fn get_statistics(&self) -> RelationshipStatistics {
        RelationshipStatistics {
            total_entities: self.reverse_lookup.len(),
            releases: self.registry.get(&EntityType::Release).map(|m| m.len()).unwrap_or(0),
            resources: self.registry.get(&EntityType::Resource).map(|m| m.len()).unwrap_or(0),
            parties: self.registry.get(&EntityType::Party).map(|m| m.len()).unwrap_or(0),
            deals: self.registry.get(&EntityType::Deal).map(|m| m.len()).unwrap_or(0),
            total_relationships: self.relationships.values().map(|s| s.len()).sum(),
        }
    }
}

/// Statistics about managed relationships
#[derive(Debug, Clone)]
pub struct RelationshipStatistics {
    pub total_entities: usize,
    pub releases: usize,
    pub resources: usize,
    pub parties: usize,
    pub deals: usize,
    pub total_relationships: usize,
}