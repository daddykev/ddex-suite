//! Deterministic reference generation with configurable styles

use super::types::{EntityType, ReferenceStyle};
use indexmap::IndexMap;

/// Generates deterministic references for DDEX entities
#[derive(Debug, Clone)]
pub struct ReferenceGenerator {
    /// Style configuration for references
    style: ReferenceStyle,
    
    /// Counters for each entity type (deterministic ordering with IndexMap)
    counters: IndexMap<EntityType, u32>,
    
    /// Custom prefixes for entity types
    prefixes: IndexMap<EntityType, String>,
}

impl ReferenceGenerator {
    /// Create a new reference generator
    pub fn new(style: ReferenceStyle) -> Self {
        let mut prefixes = IndexMap::new();
        
        // Default DDEX prefixes
        prefixes.insert(EntityType::Release, "R".to_string());
        prefixes.insert(EntityType::Resource, "A".to_string());
        prefixes.insert(EntityType::Party, "P".to_string());
        prefixes.insert(EntityType::Deal, "D".to_string());
        prefixes.insert(EntityType::TechnicalDetails, "T".to_string());
        prefixes.insert(EntityType::RightsController, "RC".to_string());
        
        Self {
            style,
            counters: IndexMap::new(),
            prefixes,
        }
    }
    
    /// Generate a new reference for an entity type
    pub fn generate(&mut self, entity_type: EntityType) -> String {
        match self.style.clone() {  // Clone the style to avoid borrow issues
            ReferenceStyle::Sequential => self.generate_sequential(entity_type),
            ReferenceStyle::Custom(formatter) => formatter(entity_type, self.next_counter(entity_type)),
            ReferenceStyle::Prefixed { separator } => self.generate_prefixed(entity_type, &separator),
        }
    }
    
    /// Generate sequential reference (e.g., "A1", "A2", "R1", "R2")
    fn generate_sequential(&mut self, entity_type: EntityType) -> String {
        let prefix = self.prefixes.get(&entity_type)
            .expect("Unknown entity type")
            .clone();  // Clone to avoid borrow issues
        let counter = self.next_counter(entity_type);
        format!("{}{}", prefix, counter)
    }
    
    /// Generate prefixed reference with custom separator
    fn generate_prefixed(&mut self, entity_type: EntityType, separator: &str) -> String {
        let prefix = self.prefixes.get(&entity_type)
            .expect("Unknown entity type")
            .clone();  // Clone to avoid borrow issues
        let counter = self.next_counter(entity_type);
        format!("{}{}{}", prefix, separator, counter)
    }
    
    /// Get next counter value for entity type
    fn next_counter(&mut self, entity_type: EntityType) -> u32 {
        let counter = self.counters.entry(entity_type).or_insert(0);
        *counter += 1;
        *counter
    }
    
    /// Set custom prefix for an entity type
    pub fn set_prefix(&mut self, entity_type: EntityType, prefix: String) {
        self.prefixes.insert(entity_type, prefix);
    }
    
    /// Reset counter for an entity type
    pub fn reset_counter(&mut self, entity_type: EntityType) {
        self.counters.insert(entity_type, 0);
    }
    
    /// Reset all counters
    pub fn reset_all_counters(&mut self) {
        self.counters.clear();
    }
    
    /// Get current counter value for entity type
    pub fn get_counter(&self, entity_type: EntityType) -> u32 {
        self.counters.get(&entity_type).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sequential_generation() {
        let mut gen = ReferenceGenerator::new(ReferenceStyle::Sequential);
        
        assert_eq!(gen.generate(EntityType::Resource), "A1");
        assert_eq!(gen.generate(EntityType::Resource), "A2");
        assert_eq!(gen.generate(EntityType::Release), "R1");
        assert_eq!(gen.generate(EntityType::Resource), "A3");
        assert_eq!(gen.generate(EntityType::Release), "R2");
    }
    
    #[test]
    fn test_deterministic_ordering() {
        let mut gen1 = ReferenceGenerator::new(ReferenceStyle::Sequential);
        let mut gen2 = ReferenceGenerator::new(ReferenceStyle::Sequential);
        
        // Same sequence should produce same results
        for _ in 0..5 {
            assert_eq!(
                gen1.generate(EntityType::Resource),
                gen2.generate(EntityType::Resource)
            );
        }
    }
}