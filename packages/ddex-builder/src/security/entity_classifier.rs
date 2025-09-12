//! Entity Classification System for DDEX Builder
//!
//! This module provides a comprehensive multi-layer classification system to distinguish
//! between legitimate DDEX entities and malicious attempts. It implements defense against
//! XXE attacks, entity expansion attacks, and other XML-based security threats.
//!
//! ## Features
//!
//! - Multi-layer entity classification (SafeBuiltin, SafeDdex, CustomLocal, Suspicious, Malicious)
//! - Recursive depth tracking and expansion ratio calculation
//! - DDEX-specific entity whitelist from official schemas
//! - Pattern matching for known attack vectors
//! - Metrics collection for security monitoring
//! - Performance-optimized caching system

use crate::error::BuildError;
use indexmap::{IndexMap, IndexSet};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use tracing::{debug, warn};

/// Maximum allowed recursive depth for entity expansion
const MAX_RECURSIVE_DEPTH: usize = 3;

/// Maximum allowed expansion ratio (output size / input size)
const MAX_EXPANSION_RATIO: f64 = 10.0;

/// Maximum total expanded size in bytes
const MAX_EXPANDED_SIZE: usize = 1_000_000; // 1MB

/// Maximum number of entities in a chain
const MAX_ENTITY_CHAIN_LENGTH: usize = 50;

/// Standard XML built-in entity patterns
static BUILTIN_ENTITIES: Lazy<IndexSet<&str>> = Lazy::new(|| {
    let mut set = IndexSet::new();
    set.insert("lt");
    set.insert("gt");
    set.insert("amp");
    set.insert("quot");
    set.insert("apos");
    set
});

/// Known malicious entity patterns
static MALICIOUS_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(lol|lol[2-9]|billion|bomb|evil|attack|exploit|payload|xxe|external|system|public)").unwrap()
});

/// External reference patterns
static EXTERNAL_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(SYSTEM|PUBLIC)\s+['"][^'"]*['"]"#).unwrap()
});

/// Network URL patterns
static NETWORK_URL_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(https?://|ftp://|file://|ftps://|smb://|\\\\)").unwrap()
});

/// Recursive entity reference patterns
static RECURSIVE_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"&[a-zA-Z_][a-zA-Z0-9._-]*;").unwrap()
});

/// Entity classification levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityClass {
    /// Standard XML built-in entities (&lt;, &gt;, &amp;, &quot;, &apos;)
    SafeBuiltin,
    /// DDEX-specific entities from official schemas
    SafeDdex,
    /// User-defined entities that need validation
    CustomLocal,
    /// Entities that match suspicious patterns but aren't confirmed malicious
    Suspicious {
        reason: String,
        confidence: f64,
    },
    /// Confirmed malicious entities
    Malicious {
        attack_type: AttackType,
        reason: String,
    },
}

/// Types of XML entity attacks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackType {
    /// XXE (XML External Entity) attack
    ExternalEntity,
    /// Billion laughs / exponential expansion attack
    ExponentialExpansion,
    /// Recursive entity definition
    RecursiveEntity,
    /// Network request attempt
    NetworkRequest,
    /// File access attempt
    FileAccess,
    /// Parameter entity attack
    ParameterEntity,
    /// Generic entity bomb
    EntityBomb,
}

/// Entity definition for analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    /// Entity name (without & and ;)
    pub name: String,
    /// Entity value/definition
    pub value: String,
    /// Whether this is a parameter entity
    pub is_parameter: bool,
    /// External system ID if present
    pub system_id: Option<String>,
    /// Public ID if present
    pub public_id: Option<String>,
    /// Nesting level in entity chain
    pub depth: usize,
    /// Size of the entity value in bytes
    pub size: usize,
}

/// Result of entity chain validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the entity chain is safe
    pub is_safe: bool,
    /// Classification of the most dangerous entity
    pub classification: EntityClass,
    /// Detailed analysis metrics
    pub metrics: EntityMetrics,
    /// Warning messages for suspicious but allowed entities
    pub warnings: Vec<String>,
    /// Errors for blocked entities
    pub errors: Vec<String>,
}

/// Security metrics for entity analysis
#[derive(Debug, Clone, Default)]
pub struct EntityMetrics {
    /// Total number of entities analyzed
    pub entity_count: usize,
    /// Maximum recursive depth found
    pub max_depth: usize,
    /// Total expansion size
    pub total_expanded_size: usize,
    /// Expansion ratio (output/input)
    pub expansion_ratio: f64,
    /// Number of external references
    pub external_references: usize,
    /// Number of network URLs found
    pub network_urls: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Configuration for entity classification
#[derive(Debug, Clone)]
pub struct ClassifierConfig {
    /// Maximum allowed recursive depth
    pub max_depth: usize,
    /// Maximum expansion ratio
    pub max_expansion_ratio: f64,
    /// Maximum total expanded size
    pub max_expanded_size: usize,
    /// Whether to allow external entities
    pub allow_external_entities: bool,
    /// Whether to allow parameter entities
    pub allow_parameter_entities: bool,
    /// Custom safe entities (in addition to DDEX whitelist)
    pub custom_safe_entities: IndexSet<String>,
    /// Whether to collect detailed metrics
    pub collect_metrics: bool,
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        Self {
            max_depth: MAX_RECURSIVE_DEPTH,
            max_expansion_ratio: MAX_EXPANSION_RATIO,
            max_expanded_size: MAX_EXPANDED_SIZE,
            allow_external_entities: false,
            allow_parameter_entities: false,
            custom_safe_entities: IndexSet::new(),
            collect_metrics: true,
        }
    }
}

/// DDEX Entity Classifier
pub struct EntityClassifier {
    config: ClassifierConfig,
    ddex_whitelist: IndexSet<String>,
    entity_cache: HashMap<String, EntityClass>,
    metrics_history: VecDeque<EntityMetrics>,
}

impl EntityClassifier {
    /// Create a new entity classifier with default configuration
    pub fn new() -> Self {
        Self::with_config(ClassifierConfig::default())
    }
    
    /// Create a new entity classifier with custom configuration
    pub fn with_config(config: ClassifierConfig) -> Self {
        let ddex_whitelist = Self::load_ddex_whitelist();
        
        Self {
            config,
            ddex_whitelist,
            entity_cache: HashMap::new(),
            metrics_history: VecDeque::with_capacity(100), // Keep last 100 analyses
        }
    }
    
    /// Classify a single entity by name and value
    pub fn classify_entity(&mut self, name: &str, value: &str) -> EntityClass {
        let cache_key = format!("{}:{}", name, value);
        
        // Check cache first
        if let Some(cached) = self.entity_cache.get(&cache_key) {
            return cached.clone();
        }
        
        let classification = self.classify_entity_internal(name, value);
        
        // Cache the result
        self.entity_cache.insert(cache_key, classification.clone());
        
        classification
    }
    
    /// Internal classification logic
    fn classify_entity_internal(&self, name: &str, value: &str) -> EntityClass {
        // 1. Check if it's a standard XML built-in entity
        if BUILTIN_ENTITIES.contains(name) {
            return EntityClass::SafeBuiltin;
        }
        
        // 2. Check if it's in the DDEX whitelist
        if self.ddex_whitelist.contains(name) {
            return EntityClass::SafeDdex;
        }
        
        // 3. Check if it's in custom safe entities
        if self.config.custom_safe_entities.contains(name) {
            return EntityClass::SafeDdex; // Treat custom safe as DDEX-level
        }
        
        // 4. Check for external references in value (highest priority)
        if EXTERNAL_PATTERNS.is_match(value) {
            return EntityClass::Malicious {
                attack_type: AttackType::ExternalEntity,
                reason: "Entity contains SYSTEM or PUBLIC external reference".to_string(),
            };
        }
        
        // 5. Check for network URLs
        if NETWORK_URL_PATTERNS.is_match(value) {
            return EntityClass::Malicious {
                attack_type: AttackType::NetworkRequest,
                reason: "Entity contains network URL".to_string(),
            };
        }
        
        // 6. Check for malicious patterns in name (lower priority)
        if MALICIOUS_PATTERNS.is_match(name) {
            return EntityClass::Malicious {
                attack_type: AttackType::EntityBomb,
                reason: format!("Entity name '{}' matches known attack patterns", name),
            };
        }
        
        // 7. Check for recursive references
        let entity_refs = RECURSIVE_PATTERNS.find_iter(value).count();
        if entity_refs > 5 {
            return EntityClass::Suspicious {
                reason: format!("Entity contains {} recursive references", entity_refs),
                confidence: (entity_refs as f64 / 10.0).min(1.0),
            };
        }
        
        // 8. Check value size
        if value.len() > 10000 {
            return EntityClass::Suspicious {
                reason: format!("Entity value is very large ({} bytes)", value.len()),
                confidence: 0.7,
            };
        }
        
        // 9. Check for repetitive patterns (possible expansion bomb)
        if self.has_repetitive_pattern(value) {
            return EntityClass::Suspicious {
                reason: "Entity contains repetitive patterns".to_string(),
                confidence: 0.6,
            };
        }
        
        // Default to custom local (needs validation)
        EntityClass::CustomLocal
    }
    
    /// Check if an entity is safe for use
    pub fn is_safe_entity(&mut self, entity: &Entity) -> bool {
        let classification = self.classify_entity(&entity.name, &entity.value);
        
        match classification {
            EntityClass::SafeBuiltin | EntityClass::SafeDdex => true,
            EntityClass::CustomLocal => {
                // Additional validation for custom entities
                entity.depth <= self.config.max_depth && 
                entity.size <= self.config.max_expanded_size &&
                !entity.is_parameter // Be strict about parameter entities
            },
            EntityClass::Suspicious { confidence, .. } => {
                // Allow suspicious entities with low confidence
                confidence < 0.5
            },
            EntityClass::Malicious { .. } => false,
        }
    }
    
    /// Validate a complete entity chain
    pub fn validate_entity_chain(&mut self, entities: &[Entity]) -> ValidationResult {
        let start_time = Instant::now();
        let mut metrics = EntityMetrics::default();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut most_dangerous = EntityClass::SafeBuiltin;
        let mut is_safe = true;
        
        // Basic chain validation
        if entities.len() > MAX_ENTITY_CHAIN_LENGTH {
            errors.push(format!(
                "Entity chain too long: {} entities (max: {})", 
                entities.len(), 
                MAX_ENTITY_CHAIN_LENGTH
            ));
            is_safe = false;
        }
        
        // Track entity expansion and depth
        let mut total_input_size = 0;
        let mut total_output_size = 0;
        let mut max_depth = 0;
        let mut external_refs = 0;
        let mut network_urls = 0;
        
        // Analyze each entity
        for entity in entities {
            let classification = self.classify_entity(&entity.name, &entity.value);
            
            // Update metrics
            total_input_size += entity.name.len() + 2; // &name;
            total_output_size += entity.size;
            max_depth = max_depth.max(entity.depth);
            
            if entity.system_id.is_some() || entity.public_id.is_some() {
                external_refs += 1;
            }
            
            if NETWORK_URL_PATTERNS.is_match(&entity.value) {
                network_urls += 1;
            }
            
            // Check individual entity safety
            match &classification {
                EntityClass::SafeBuiltin | EntityClass::SafeDdex => {
                    // These are always safe
                },
                EntityClass::CustomLocal => {
                    if entity.depth > self.config.max_depth {
                        errors.push(format!(
                            "Entity '{}' exceeds maximum depth: {} > {}", 
                            entity.name, entity.depth, self.config.max_depth
                        ));
                        is_safe = false;
                    }
                    
                    if entity.is_parameter && !self.config.allow_parameter_entities {
                        errors.push(format!(
                            "Parameter entity '{}' not allowed", 
                            entity.name
                        ));
                        is_safe = false;
                    }
                },
                EntityClass::Suspicious { reason, confidence } => {
                    warnings.push(format!(
                        "Suspicious entity '{}': {} (confidence: {:.2})", 
                        entity.name, reason, confidence
                    ));
                    
                    if *confidence > 0.7 {
                        is_safe = false;
                        most_dangerous = classification.clone();
                    }
                },
                EntityClass::Malicious { attack_type, reason } => {
                    errors.push(format!(
                        "Malicious entity '{}' ({:?}): {}", 
                        entity.name, attack_type, reason
                    ));
                    is_safe = false;
                    most_dangerous = classification.clone();
                },
            }
        }
        
        // Calculate expansion ratio
        let expansion_ratio = if total_input_size > 0 {
            total_output_size as f64 / total_input_size as f64
        } else {
            1.0
        };
        
        // Check overall limits
        if expansion_ratio > self.config.max_expansion_ratio {
            errors.push(format!(
                "Expansion ratio too high: {:.2} > {}", 
                expansion_ratio, self.config.max_expansion_ratio
            ));
            is_safe = false;
        }
        
        if total_output_size > self.config.max_expanded_size {
            errors.push(format!(
                "Total expanded size too large: {} > {}", 
                total_output_size, self.config.max_expanded_size
            ));
            is_safe = false;
        }
        
        if external_refs > 0 && !self.config.allow_external_entities {
            errors.push(format!(
                "External entities not allowed ({} found)", 
                external_refs
            ));
            is_safe = false;
        }
        
        // Populate metrics
        metrics.entity_count = entities.len();
        metrics.max_depth = max_depth;
        metrics.total_expanded_size = total_output_size;
        metrics.expansion_ratio = expansion_ratio;
        metrics.external_references = external_refs;
        metrics.network_urls = network_urls;
        metrics.processing_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Store metrics for analysis
        if self.config.collect_metrics {
            self.metrics_history.push_back(metrics.clone());
            if self.metrics_history.len() > 100 {
                self.metrics_history.pop_front();
            }
        }
        
        // Log security events
        if !is_safe {
            warn!(
                "Entity chain validation failed: {} errors, {} warnings", 
                errors.len(), 
                warnings.len()
            );
        } else if !warnings.is_empty() {
            debug!(
                "Entity chain validation passed with {} warnings", 
                warnings.len()
            );
        }
        
        ValidationResult {
            is_safe,
            classification: most_dangerous,
            metrics,
            warnings,
            errors,
        }
    }
    
    /// Get recent security metrics for analysis
    pub fn get_metrics_history(&self) -> &VecDeque<EntityMetrics> {
        &self.metrics_history
    }
    
    /// Clear the entity classification cache
    pub fn clear_cache(&mut self) {
        self.entity_cache.clear();
    }
    
    /// Load DDEX entity whitelist from official schemas
    fn load_ddex_whitelist() -> IndexSet<String> {
        let mut whitelist = IndexSet::new();
        
        // Standard DDEX entities that are commonly used and safe
        // These would typically be loaded from DDEX schema files
        whitelist.insert("ddex".to_string());
        whitelist.insert("ern".to_string());
        whitelist.insert("avs".to_string());
        whitelist.insert("iso".to_string());
        whitelist.insert("musicbrainz".to_string());
        whitelist.insert("isrc".to_string());
        whitelist.insert("iswc".to_string());
        whitelist.insert("isni".to_string());
        whitelist.insert("dpid".to_string());
        whitelist.insert("grid".to_string());
        whitelist.insert("mwli".to_string());
        whitelist.insert("spar".to_string());
        
        // Common DDEX namespace prefixes
        whitelist.insert("NewReleaseMessage".to_string());
        whitelist.insert("MessageHeader".to_string());
        whitelist.insert("MessageId".to_string());
        whitelist.insert("MessageSender".to_string());
        whitelist.insert("SentOnBehalfOf".to_string());
        whitelist.insert("MessageRecipient".to_string());
        whitelist.insert("MessageCreatedDateTime".to_string());
        whitelist.insert("MessageAuditTrail".to_string());
        
        // Release-specific entities
        whitelist.insert("ReleaseList".to_string());
        whitelist.insert("Release".to_string());
        whitelist.insert("ReleaseId".to_string());
        whitelist.insert("ReleaseReference".to_string());
        whitelist.insert("ReferenceTitle".to_string());
        whitelist.insert("ReleaseDetailsByTerritory".to_string());
        
        // Resource entities
        whitelist.insert("ResourceList".to_string());
        whitelist.insert("SoundRecording".to_string());
        whitelist.insert("MusicalWork".to_string());
        whitelist.insert("Image".to_string());
        whitelist.insert("Text".to_string());
        whitelist.insert("Video".to_string());
        
        // Deal/Commercial entities
        whitelist.insert("DealList".to_string());
        whitelist.insert("ReleaseDeal".to_string());
        whitelist.insert("Deal".to_string());
        whitelist.insert("DealTerms".to_string());
        whitelist.insert("CommercialModelType".to_string());
        whitelist.insert("Usage".to_string());
        whitelist.insert("Territory".to_string());
        
        debug!("Loaded {} DDEX entities to whitelist", whitelist.len());
        
        whitelist
    }
    
    /// Check if a value has repetitive patterns that might indicate an expansion bomb
    fn has_repetitive_pattern(&self, value: &str) -> bool {
        if value.len() < 20 {
            return false;
        }
        
        // Look for repeated substrings
        let chars: Vec<char> = value.chars().collect();
        let len = chars.len();
        
        // Check for patterns of length 2-10
        for pattern_len in 2..=10.min(len / 4) {
            let mut matches = 0;
            let pattern = &chars[0..pattern_len];
            
            for i in (0..len).step_by(pattern_len) {
                if i + pattern_len <= len && &chars[i..i + pattern_len] == pattern {
                    matches += 1;
                }
            }
            
            // If more than 50% of the string is the same pattern, it's suspicious
            if matches * pattern_len > len / 2 {
                return true;
            }
        }
        
        false
    }
}

impl Default for EntityClassifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create an Entity from name and value
pub fn create_entity(name: &str, value: &str) -> Entity {
    Entity {
        name: name.to_string(),
        value: value.to_string(),
        is_parameter: false,
        system_id: None,
        public_id: None,
        depth: 0,
        size: value.len(),
    }
}

/// Helper function to create a parameter entity
pub fn create_parameter_entity(name: &str, value: &str) -> Entity {
    Entity {
        name: name.to_string(),
        value: value.to_string(),
        is_parameter: true,
        system_id: None,
        public_id: None,
        depth: 0,
        size: value.len(),
    }
}

/// Helper function to create an external entity
pub fn create_external_entity(name: &str, system_id: &str) -> Entity {
    Entity {
        name: name.to_string(),
        value: String::new(),
        is_parameter: false,
        system_id: Some(system_id.to_string()),
        public_id: None,
        depth: 0,
        size: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builtin_entity_classification() {
        let mut classifier = EntityClassifier::new();
        
        assert_eq!(
            classifier.classify_entity("lt", "<"),
            EntityClass::SafeBuiltin
        );
        
        assert_eq!(
            classifier.classify_entity("amp", "&"),
            EntityClass::SafeBuiltin
        );
    }
    
    #[test]
    fn test_ddex_entity_classification() {
        let mut classifier = EntityClassifier::new();
        
        assert_eq!(
            classifier.classify_entity("ddex", "http://ddex.net/xml/ern/43"),
            EntityClass::SafeDdex
        );
    }
    
    #[test]
    fn test_malicious_entity_detection() {
        let mut classifier = EntityClassifier::new();
        
        // Test external entity
        let result = classifier.classify_entity(
            "xxe", 
            "<!ENTITY xxe SYSTEM \"file:///etc/passwd\">"
        );
        
        match result {
            EntityClass::Malicious { attack_type: AttackType::ExternalEntity, .. } => {},
            _ => panic!("Should detect external entity attack"),
        }
        
        // Test network URL
        let result = classifier.classify_entity(
            "evil", 
            "http://attacker.com/evil.xml"
        );
        
        match result {
            EntityClass::Malicious { attack_type: AttackType::NetworkRequest, .. } => {},
            _ => panic!("Should detect network request attack"),
        }
    }
    
    #[test]
    fn test_entity_chain_validation() {
        let mut classifier = EntityClassifier::new();
        
        let entities = vec![
            create_entity("safe", "content"),
            create_entity("lol", "&lol2;&lol2;&lol2;"),
            create_entity("lol2", "&lol3;&lol3;&lol3;"),
            create_entity("lol3", "haha"),
        ];
        
        let result = classifier.validate_entity_chain(&entities);
        assert!(!result.is_safe);
        assert!(!result.errors.is_empty());
    }
    
    #[test]
    fn test_safe_entity_chain() {
        let mut classifier = EntityClassifier::new();
        
        let entities = vec![
            create_entity("title", "My Song"),
            create_entity("artist", "My Artist"),
        ];
        
        let result = classifier.validate_entity_chain(&entities);
        assert!(result.is_safe);
        assert!(result.errors.is_empty());
    }
    
    #[test]
    fn test_expansion_ratio_detection() {
        let mut classifier = EntityClassifier::new();
        
        // Create entities that expand significantly
        let entities = vec![
            Entity {
                name: "bomb".to_string(),
                value: "A".repeat(1000),
                is_parameter: false,
                system_id: None,
                public_id: None,
                depth: 0,
                size: 1000,
            },
        ];
        
        let result = classifier.validate_entity_chain(&entities);
        
        // Should trigger expansion ratio warning
        assert!(result.metrics.expansion_ratio > 50.0);
    }
}