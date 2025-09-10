//! Reference management for streaming DDEX XML generation
//! 
//! Manages stable reference generation and validation during streaming
//! to ensure proper linking between releases and resources.

use crate::error::BuildError;
use crate::id_generator::{StableHashGenerator, StableHashConfig};
use indexmap::{IndexMap, IndexSet};

/// Configuration for reference management during streaming
#[derive(Debug, Clone)]
pub struct ReferenceConfig {
    /// Use deterministic reference generation
    pub deterministic: bool,
    /// Prefix for resource references
    pub resource_prefix: String,
    /// Prefix for release references  
    pub release_prefix: String,
    /// Prefix for deal references
    pub deal_prefix: String,
    /// Maximum number of references to cache in memory
    pub max_cache_size: usize,
}

impl Default for ReferenceConfig {
    fn default() -> Self {
        Self {
            deterministic: true,
            resource_prefix: "R".to_string(),
            release_prefix: "REL".to_string(),
            deal_prefix: "D".to_string(),
            max_cache_size: 100_000, // Should handle large catalogs
        }
    }
}

/// Represents a resource reference with metadata
#[derive(Debug, Clone)]
pub struct ResourceReference {
    pub reference_id: String,
    pub resource_id: String,
    pub title: String,
    pub artist: String,
    pub resource_type: String,
    pub sequence_number: usize,
}

/// Represents a release reference with metadata
#[derive(Debug, Clone)]
pub struct ReleaseReference {
    pub reference_id: String,
    pub release_id: String,
    pub title: String,
    pub artist: String,
    pub resource_references: Vec<String>,
    pub sequence_number: usize,
}

/// Manages references during streaming operations
pub struct StreamingReferenceManager {
    config: ReferenceConfig,
    hash_generator: StableHashGenerator,
    
    // Resource tracking
    resource_references: IndexMap<String, String>, // resource_id -> reference_id
    resource_metadata: IndexMap<String, ResourceReference>,
    resource_sequence: usize,
    
    // Release tracking
    release_references: IndexMap<String, String>, // release_id -> reference_id  
    release_metadata: IndexMap<String, ReleaseReference>,
    release_sequence: usize,
    
    // Deal tracking
    deal_references: IndexMap<String, String>, // deal_id -> reference_id
    deal_sequence: usize,
    
    // Validation tracking
    used_references: IndexSet<String>,
    orphaned_references: Vec<String>,
    duplicate_resource_ids: IndexSet<String>,
    duplicate_release_ids: IndexSet<String>,
    
    // Memory management
    references_generated: usize,
}

impl StreamingReferenceManager {
    /// Create a new streaming reference manager
    pub fn new() -> Self {
        Self::new_with_config(ReferenceConfig::default())
    }
    
    /// Create a new streaming reference manager with custom configuration
    pub fn new_with_config(config: ReferenceConfig) -> Self {
        let hash_config = StableHashConfig::default();
        
        StreamingReferenceManager {
            config,
            hash_generator: StableHashGenerator::new(hash_config),
            resource_references: IndexMap::new(),
            resource_metadata: IndexMap::new(),
            resource_sequence: 1,
            release_references: IndexMap::new(),
            release_metadata: IndexMap::new(),
            release_sequence: 1,
            deal_references: IndexMap::new(),
            deal_sequence: 1,
            used_references: IndexSet::new(),
            orphaned_references: Vec::new(),
            duplicate_resource_ids: IndexSet::new(),
            duplicate_release_ids: IndexSet::new(),
            references_generated: 0,
        }
    }
    
    /// Generate a stable reference for a resource
    pub fn generate_resource_reference(&mut self, resource_id: &str) -> Result<String, BuildError> {
        // Check for duplicate resource ID
        if self.resource_references.contains_key(resource_id) {
            self.duplicate_resource_ids.insert(resource_id.to_string());
            return Ok(self.resource_references[resource_id].clone());
        }
        
        // Generate stable reference
        let reference_id = if self.config.deterministic {
            // Use a simplified hash approach for now
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(resource_id.as_bytes());
            let hash = format!("{:x}", hasher.finalize());
            format!("{}{}", self.config.resource_prefix, &hash[..8])
        } else {
            format!("{}{:06}", self.config.resource_prefix, self.resource_sequence)
        };
        
        // Check for reference collision
        if self.used_references.contains(&reference_id) {
            return Err(BuildError::InvalidReference { 
                reference: reference_id,
            });
        }
        
        // Store the mapping
        self.resource_references.insert(resource_id.to_string(), reference_id.clone());
        self.used_references.insert(reference_id.clone());
        self.resource_sequence += 1;
        self.references_generated += 1;
        
        // Manage memory usage
        self.manage_cache_size()?;
        
        Ok(reference_id)
    }
    
    /// Generate a stable reference for a release
    pub fn generate_release_reference(&mut self, release_id: &str) -> Result<String, BuildError> {
        // Check for duplicate release ID
        if self.release_references.contains_key(release_id) {
            self.duplicate_release_ids.insert(release_id.to_string());
            return Ok(self.release_references[release_id].clone());
        }
        
        // Generate stable reference
        let reference_id = if self.config.deterministic {
            // Use a simplified hash approach for now
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(release_id.as_bytes());
            let hash = format!("{:x}", hasher.finalize());
            format!("{}{}", self.config.release_prefix, &hash[..8])
        } else {
            format!("{}{:06}", self.config.release_prefix, self.release_sequence)
        };
        
        // Check for reference collision
        if self.used_references.contains(&reference_id) {
            return Err(BuildError::InvalidReference { 
                reference: reference_id,
            });
        }
        
        // Store the mapping
        self.release_references.insert(release_id.to_string(), reference_id.clone());
        self.used_references.insert(reference_id.clone());
        self.release_sequence += 1;
        self.references_generated += 1;
        
        // Manage memory usage
        self.manage_cache_size()?;
        
        Ok(reference_id)
    }
    
    /// Generate a stable reference for a deal
    pub fn generate_deal_reference(&mut self, deal_id: &str) -> Result<String, BuildError> {
        // Check for existing mapping
        if let Some(existing_ref) = self.deal_references.get(deal_id) {
            return Ok(existing_ref.clone());
        }
        
        // Generate stable reference
        let reference_id = if self.config.deterministic {
            // Use a simplified hash approach for now
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(deal_id.as_bytes());
            let hash = format!("{:x}", hasher.finalize());
            format!("{}{}", self.config.deal_prefix, &hash[..8])
        } else {
            format!("{}{:06}", self.config.deal_prefix, self.deal_sequence)
        };
        
        // Check for reference collision
        if self.used_references.contains(&reference_id) {
            return Err(BuildError::InvalidReference { 
                reference: reference_id,
            });
        }
        
        // Store the mapping
        self.deal_references.insert(deal_id.to_string(), reference_id.clone());
        self.used_references.insert(reference_id.clone());
        self.deal_sequence += 1;
        self.references_generated += 1;
        
        // Manage memory usage
        self.manage_cache_size()?;
        
        Ok(reference_id)
    }
    
    /// Store metadata for a resource reference
    pub fn store_resource_metadata(&mut self, 
                                   resource_id: &str,
                                   title: &str,
                                   artist: &str,
                                   resource_type: &str) -> Result<(), BuildError> {
        let reference_id = self.resource_references.get(resource_id)
            .ok_or_else(|| BuildError::InvalidReference { 
                reference: format!("Resource {} not found", resource_id) 
            })?
            .clone();
        
        let metadata = ResourceReference {
            reference_id: reference_id.clone(),
            resource_id: resource_id.to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            resource_type: resource_type.to_string(),
            sequence_number: self.resource_metadata.len() + 1,
        };
        
        self.resource_metadata.insert(reference_id, metadata);
        Ok(())
    }
    
    /// Store metadata for a release reference
    pub fn store_release_metadata(&mut self,
                                  release_id: &str,
                                  title: &str, 
                                  artist: &str,
                                  resource_references: Vec<String>) -> Result<(), BuildError> {
        let reference_id = self.release_references.get(release_id)
            .ok_or_else(|| BuildError::InvalidReference { 
                reference: format!("Release {} not found", release_id) 
            })?
            .clone();
        
        // Validate that all resource references exist
        for resource_ref in &resource_references {
            if !self.used_references.contains(resource_ref) {
                self.orphaned_references.push(resource_ref.clone());
            }
        }
        
        let metadata = ReleaseReference {
            reference_id: reference_id.clone(),
            release_id: release_id.to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            resource_references,
            sequence_number: self.release_metadata.len() + 1,
        };
        
        self.release_metadata.insert(reference_id, metadata);
        Ok(())
    }
    
    /// Validate all references at the end of streaming
    pub fn validate_references(&self) -> ReferenceValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check for orphaned references
        if !self.orphaned_references.is_empty() {
            warnings.push(format!("Found {} orphaned resource references", 
                                 self.orphaned_references.len()));
        }
        
        // Check for duplicate resource IDs
        if !self.duplicate_resource_ids.is_empty() {
            warnings.push(format!("Found {} duplicate resource IDs", 
                                 self.duplicate_resource_ids.len()));
        }
        
        // Check for duplicate release IDs
        if !self.duplicate_release_ids.is_empty() {
            warnings.push(format!("Found {} duplicate release IDs", 
                                 self.duplicate_release_ids.len()));
        }
        
        // Check reference consistency
        for (resource_id, reference_id) in &self.resource_references {
            if !self.used_references.contains(reference_id) {
                errors.push(format!("Resource reference {} for resource {} not properly tracked", 
                                   reference_id, resource_id));
            }
        }
        
        for (release_id, reference_id) in &self.release_references {
            if !self.used_references.contains(reference_id) {
                errors.push(format!("Release reference {} for release {} not properly tracked", 
                                   reference_id, release_id));
            }
        }
        
        ReferenceValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            total_references: self.references_generated,
            resource_count: self.resource_references.len(),
            release_count: self.release_references.len(),
            deal_count: self.deal_references.len(),
        }
    }
    
    /// Get statistics about reference management
    pub fn get_stats(&self) -> ReferenceStats {
        ReferenceStats {
            total_references_generated: self.references_generated,
            resource_references: self.resource_references.len(),
            release_references: self.release_references.len(),
            deal_references: self.deal_references.len(),
            cache_size: self.current_cache_size(),
            duplicate_resource_ids: self.duplicate_resource_ids.len(),
            duplicate_release_ids: self.duplicate_release_ids.len(),
            orphaned_references: self.orphaned_references.len(),
        }
    }
    
    /// Get a resource reference by resource ID
    pub fn get_resource_reference(&self, resource_id: &str) -> Option<&str> {
        self.resource_references.get(resource_id).map(|s| s.as_str())
    }
    
    /// Get a release reference by release ID
    pub fn get_release_reference(&self, release_id: &str) -> Option<&str> {
        self.release_references.get(release_id).map(|s| s.as_str())
    }
    
    /// Clear old references to manage memory usage
    fn manage_cache_size(&mut self) -> Result<(), BuildError> {
        let current_size = self.current_cache_size();
        
        if current_size > self.config.max_cache_size {
            // Remove oldest 25% of entries to free up memory
            let to_remove = current_size / 4;
            
            // Remove oldest resource references
            let resource_to_remove = std::cmp::min(to_remove / 2, self.resource_references.len() / 2);
            for _ in 0..resource_to_remove {
                if let Some((_resource_id, reference_id)) = self.resource_references.shift_remove_index(0) {
                    self.resource_metadata.remove(&reference_id);
                    self.used_references.remove(&reference_id);
                }
            }
            
            // Remove oldest release references  
            let release_to_remove = std::cmp::min(to_remove / 2, self.release_references.len() / 2);
            for _ in 0..release_to_remove {
                if let Some((_release_id, reference_id)) = self.release_references.shift_remove_index(0) {
                    self.release_metadata.remove(&reference_id);
                    self.used_references.remove(&reference_id);
                }
            }
        }
        
        Ok(())
    }
    
    fn current_cache_size(&self) -> usize {
        self.resource_references.len() + 
        self.release_references.len() + 
        self.deal_references.len() + 
        self.resource_metadata.len() + 
        self.release_metadata.len()
    }
}

impl Default for StreamingReferenceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of reference validation
#[derive(Debug, Clone)]
pub struct ReferenceValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub total_references: usize,
    pub resource_count: usize,
    pub release_count: usize,
    pub deal_count: usize,
}

/// Statistics about reference management
#[derive(Debug, Clone)]
pub struct ReferenceStats {
    pub total_references_generated: usize,
    pub resource_references: usize,
    pub release_references: usize,
    pub deal_references: usize,
    pub cache_size: usize,
    pub duplicate_resource_ids: usize,
    pub duplicate_release_ids: usize,
    pub orphaned_references: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_reference_generation() {
        let mut manager = StreamingReferenceManager::new();
        
        let ref1 = manager.generate_resource_reference("resource1").unwrap();
        let ref2 = manager.generate_resource_reference("resource2").unwrap();
        
        assert_ne!(ref1, ref2);
        assert!(ref1.starts_with("R"));
        assert!(ref2.starts_with("R"));
        
        // Test duplicate handling
        let ref3 = manager.generate_resource_reference("resource1").unwrap();
        assert_eq!(ref1, ref3);
    }
    
    #[test]
    fn test_release_reference_generation() {
        let mut manager = StreamingReferenceManager::new();
        
        let ref1 = manager.generate_release_reference("release1").unwrap();
        let ref2 = manager.generate_release_reference("release2").unwrap();
        
        assert_ne!(ref1, ref2);
        assert!(ref1.starts_with("REL"));
        assert!(ref2.starts_with("REL"));
    }
    
    #[test]
    fn test_metadata_storage() {
        let mut manager = StreamingReferenceManager::new();
        
        let resource_ref = manager.generate_resource_reference("resource1").unwrap();
        manager.store_resource_metadata("resource1", "Title", "Artist", "SoundRecording").unwrap();
        
        let metadata = manager.resource_metadata.get(&resource_ref).unwrap();
        assert_eq!(metadata.title, "Title");
        assert_eq!(metadata.artist, "Artist");
    }
    
    #[test]
    fn test_reference_validation() {
        let mut manager = StreamingReferenceManager::new();
        
        // Add some resources and releases
        let resource_ref = manager.generate_resource_reference("resource1").unwrap();
        let release_ref = manager.generate_release_reference("release1").unwrap();
        
        // Store metadata with valid resource reference
        manager.store_release_metadata("release1", "Album Title", "Artist", 
                                      vec![resource_ref]).unwrap();
        
        let validation = manager.validate_references();
        assert!(validation.is_valid);
        assert_eq!(validation.resource_count, 1);
        assert_eq!(validation.release_count, 1);
    }
    
    #[test]
    fn test_orphaned_references() {
        let mut manager = StreamingReferenceManager::new();
        
        let release_ref = manager.generate_release_reference("release1").unwrap();
        
        // Store metadata with invalid resource reference
        manager.store_release_metadata("release1", "Album Title", "Artist", 
                                      vec!["R999999".to_string()]).unwrap();
        
        let validation = manager.validate_references();
        assert!(!validation.warnings.is_empty());
    }
}