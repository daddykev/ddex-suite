//! Caching optimizations for DDEX Builder
//! 
//! This module provides multi-level caching for schemas, validation results,
//! hash computations, and compiled templates to eliminate redundant work.

use crate::error::BuildError;
use crate::optimized_strings::OptimizedString;
use indexmap::IndexMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;
use blake3::Hasher as Blake3Hasher;

/// Global cache instance for schema and validation data
static GLOBAL_CACHE: Lazy<Arc<RwLock<GlobalCache>>> = Lazy::new(|| {
    Arc::new(RwLock::new(GlobalCache::new()))
});

/// Multi-level cache for DDEX Builder operations
#[derive(Debug)]
pub struct GlobalCache {
    /// Schema cache
    schemas: SchemaCache,
    /// Validation results cache
    validation_cache: ValidationCache,
    /// Hash computation cache
    hash_cache: HashCache,
    /// Template cache for common patterns
    template_cache: TemplateCache,
    /// Statistics
    stats: CacheStats,
}

impl GlobalCache {
    /// Create a new global cache
    pub fn new() -> Self {
        Self {
            schemas: SchemaCache::new(),
            validation_cache: ValidationCache::new(),
            hash_cache: HashCache::new(),
            template_cache: TemplateCache::new(),
            stats: CacheStats::default(),
        }
    }
    
    /// Clear all caches
    pub fn clear_all(&mut self) {
        self.schemas.clear();
        self.validation_cache.clear();
        self.hash_cache.clear();
        self.template_cache.clear();
        self.stats = CacheStats::default();
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }
    
    /// Prune expired entries from all caches
    pub fn prune_expired(&mut self) {
        self.validation_cache.prune_expired();
        self.hash_cache.prune_expired();
        self.template_cache.prune_expired();
    }
}

/// Cache for compiled schemas
#[derive(Debug)]
pub struct SchemaCache {
    /// Compiled schemas by version and profile
    schemas: IndexMap<SchemaKey, CachedSchema>,
    /// Schema metadata
    metadata: IndexMap<SchemaKey, SchemaMetadata>,
}

impl SchemaCache {
    /// Create new schema cache
    pub fn new() -> Self {
        Self {
            schemas: IndexMap::new(),
            metadata: IndexMap::new(),
        }
    }
    
    /// Get or compile a schema
    pub fn get_or_compile(
        &mut self,
        version: &str,
        profile: Option<&str>,
        compiler: impl FnOnce() -> Result<CompiledSchema, BuildError>,
    ) -> Result<&CompiledSchema, BuildError> {
        let key = SchemaKey {
            version: version.to_string(),
            profile: profile.map(|p| p.to_string()),
        };
        
        if !self.schemas.contains_key(&key) {
            let start_time = Instant::now();
            let schema = compiler()?;
            let compile_time = start_time.elapsed();
            
            self.metadata.insert(key.clone(), SchemaMetadata {
                compile_time,
                last_used: Instant::now(),
                use_count: 0,
            });
            
            self.schemas.insert(key.clone(), CachedSchema {
                schema,
                created_at: Instant::now(),
            });
        }
        
        // Update usage statistics
        if let Some(metadata) = self.metadata.get_mut(&key) {
            metadata.last_used = Instant::now();
            metadata.use_count += 1;
        }
        
        Ok(&self.schemas.get(&key).unwrap().schema)
    }
    
    /// Check if schema is cached
    pub fn contains(&self, version: &str, profile: Option<&str>) -> bool {
        let key = SchemaKey {
            version: version.to_string(),
            profile: profile.map(|p| p.to_string()),
        };
        self.schemas.contains_key(&key)
    }
    
    /// Clear all schemas
    pub fn clear(&mut self) {
        self.schemas.clear();
        self.metadata.clear();
    }
    
    /// Get memory usage
    pub fn memory_usage(&self) -> usize {
        self.schemas.values()
            .map(|cached| cached.schema.memory_footprint())
            .sum()
    }
}

/// Cache key for schemas
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SchemaKey {
    version: String,
    profile: Option<String>,
}

/// Cached schema with metadata
#[derive(Debug)]
struct CachedSchema {
    schema: CompiledSchema,
    created_at: Instant,
}

/// Schema metadata for statistics
#[derive(Debug)]
struct SchemaMetadata {
    compile_time: Duration,
    last_used: Instant,
    use_count: usize,
}

/// Compiled schema representation
#[derive(Debug, Clone)]
pub struct CompiledSchema {
    /// Version identifier
    pub version: String,
    /// Profile identifier
    pub profile: Option<String>,
    /// Validation rules
    pub rules: Vec<ValidationRule>,
    /// Required elements
    pub required_elements: Vec<String>,
    /// Element constraints
    pub element_constraints: IndexMap<String, ElementConstraint>,
}

impl CompiledSchema {
    /// Calculate memory footprint
    pub fn memory_footprint(&self) -> usize {
        std::mem::size_of::<Self>() + 
        self.version.len() +
        self.profile.as_ref().map_or(0, |p| p.len()) +
        self.rules.len() * std::mem::size_of::<ValidationRule>() +
        self.required_elements.iter().map(|e| e.len()).sum::<usize>() +
        self.element_constraints.keys().map(|k| k.len()).sum::<usize>()
    }
}

/// Validation rule
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub element_path: String,
    pub rule_type: RuleType,
    pub parameters: Vec<String>,
}

/// Rule types
#[derive(Debug, Clone)]
pub enum RuleType {
    Required,
    Pattern(String),
    Range(f64, f64),
    Length(usize, usize),
    Custom(String),
}

/// Element constraint
#[derive(Debug, Clone)]
pub struct ElementConstraint {
    pub min_occurs: usize,
    pub max_occurs: Option<usize>,
    pub data_type: String,
}

/// Cache for validation results
#[derive(Debug)]
pub struct ValidationCache {
    /// Validation results by content hash
    results: IndexMap<String, CachedValidationResult>,
    /// Cache configuration
    config: ValidationCacheConfig,
}

impl ValidationCache {
    /// Create new validation cache
    pub fn new() -> Self {
        Self {
            results: IndexMap::new(),
            config: ValidationCacheConfig::default(),
        }
    }
    
    /// Get cached validation result
    pub fn get(&mut self, content_hash: &str) -> Option<ValidationResult> {
        if let Some(cached) = self.results.get_mut(content_hash) {
            // Check if expired
            if cached.created_at.elapsed() > self.config.ttl {
                self.results.remove(content_hash);
                return None;
            }
            
            cached.last_accessed = Instant::now();
            cached.access_count += 1;
            Some(cached.result.clone())
        } else {
            None
        }
    }
    
    /// Cache validation result
    pub fn insert(&mut self, content_hash: String, result: ValidationResult) {
        // Evict old entries if cache is full
        if self.results.len() >= self.config.max_entries {
            self.evict_lru();
        }
        
        self.results.insert(content_hash, CachedValidationResult {
            result,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
        });
    }
    
    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some((key, _)) = self.results.iter()
            .min_by_key(|(_, cached)| cached.last_accessed)
            .map(|(k, v)| (k.clone(), v.last_accessed))
        {
            self.results.remove(&key);
        }
    }
    
    /// Prune expired entries
    pub fn prune_expired(&mut self) {
        let ttl = self.config.ttl;
        self.results.retain(|_, cached| cached.created_at.elapsed() <= ttl);
    }
    
    /// Clear all validation results
    pub fn clear(&mut self) {
        self.results.clear();
    }
    
    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        if self.results.is_empty() {
            0.0
        } else {
            let total_accesses: usize = self.results.values()
                .map(|cached| cached.access_count)
                .sum();
            if total_accesses == 0 {
                0.0
            } else {
                self.results.len() as f64 / total_accesses as f64
            }
        }
    }
}

/// Cached validation result
#[derive(Debug)]
struct CachedValidationResult {
    result: ValidationResult,
    created_at: Instant,
    last_accessed: Instant,
    access_count: usize,
}

/// Validation cache configuration
#[derive(Debug)]
struct ValidationCacheConfig {
    max_entries: usize,
    ttl: Duration,
}

impl Default for ValidationCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub validation_time: Duration,
}

/// Cache for hash computations
#[derive(Debug)]
pub struct HashCache {
    /// Computed hashes
    hashes: IndexMap<HashKey, CachedHash>,
    /// Configuration
    config: HashCacheConfig,
}

impl HashCache {
    /// Create new hash cache
    pub fn new() -> Self {
        Self {
            hashes: IndexMap::new(),
            config: HashCacheConfig::default(),
        }
    }
    
    /// Get or compute hash
    pub fn get_or_compute<T: Hash>(
        &mut self,
        key: &HashKey,
        value: &T,
        hasher_fn: impl FnOnce(&T) -> String,
    ) -> String {
        if let Some(cached) = self.hashes.get_mut(key) {
            if cached.created_at.elapsed() <= self.config.ttl {
                cached.access_count += 1;
                return cached.hash.clone();
            } else {
                // Expired, remove and recompute
                self.hashes.remove(key);
            }
        }
        
        // Compute new hash
        let hash = hasher_fn(value);
        
        // Evict if necessary
        if self.hashes.len() >= self.config.max_entries {
            self.evict_random();
        }
        
        self.hashes.insert(key.clone(), CachedHash {
            hash: hash.clone(),
            created_at: Instant::now(),
            access_count: 1,
        });
        
        hash
    }
    
    /// Evict a random entry (simple eviction strategy)
    fn evict_random(&mut self) {
        if let Some(key) = self.hashes.keys().next().cloned() {
            self.hashes.remove(&key);
        }
    }
    
    /// Prune expired entries
    pub fn prune_expired(&mut self) {
        let ttl = self.config.ttl;
        self.hashes.retain(|_, cached| cached.created_at.elapsed() <= ttl);
    }
    
    /// Clear all hashes
    pub fn clear(&mut self) {
        self.hashes.clear();
    }
}

/// Hash key for caching
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HashKey {
    pub algorithm: String,
    pub content_type: String,
    pub content_id: String,
}

/// Cached hash result
#[derive(Debug)]
struct CachedHash {
    hash: String,
    created_at: Instant,
    access_count: usize,
}

/// Hash cache configuration
#[derive(Debug)]
struct HashCacheConfig {
    max_entries: usize,
    ttl: Duration,
}

impl Default for HashCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 500,
            ttl: Duration::from_secs(600), // 10 minutes
        }
    }
}

/// Cache for XML templates and patterns
#[derive(Debug)]
pub struct TemplateCache {
    /// Compiled templates
    templates: IndexMap<TemplateKey, CachedTemplate>,
    /// Configuration
    config: TemplateCacheConfig,
}

impl TemplateCache {
    /// Create new template cache
    pub fn new() -> Self {
        Self {
            templates: IndexMap::new(),
            config: TemplateCacheConfig::default(),
        }
    }
    
    /// Get or compile template
    pub fn get_or_compile(
        &mut self,
        key: &TemplateKey,
        compiler: impl FnOnce() -> CompiledTemplate,
    ) -> &CompiledTemplate {
        if !self.templates.contains_key(key) {
            if self.templates.len() >= self.config.max_entries {
                self.evict_lru();
            }
            
            let template = compiler();
            self.templates.insert(key.clone(), CachedTemplate {
                template,
                created_at: Instant::now(),
                last_used: Instant::now(),
                use_count: 0,
            });
        }
        
        // Update usage
        if let Some(cached) = self.templates.get_mut(key) {
            cached.last_used = Instant::now();
            cached.use_count += 1;
        }
        
        &self.templates.get(key).unwrap().template
    }
    
    /// Evict least recently used template
    fn evict_lru(&mut self) {
        if let Some((key, _)) = self.templates.iter()
            .min_by_key(|(_, cached)| cached.last_used)
            .map(|(k, v)| (k.clone(), v.last_used))
        {
            self.templates.remove(&key);
        }
    }
    
    /// Prune expired templates
    pub fn prune_expired(&mut self) {
        let ttl = self.config.ttl;
        self.templates.retain(|_, cached| cached.created_at.elapsed() <= ttl);
    }
    
    /// Clear all templates
    pub fn clear(&mut self) {
        self.templates.clear();
    }
}

/// Template key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemplateKey {
    pub element_type: String,
    pub version: String,
    pub variant: Option<String>,
}

/// Cached template
#[derive(Debug)]
struct CachedTemplate {
    template: CompiledTemplate,
    created_at: Instant,
    last_used: Instant,
    use_count: usize,
}

/// Compiled template for fast XML generation
#[derive(Debug, Clone)]
pub struct CompiledTemplate {
    /// Template parts (static strings and placeholders)
    pub parts: Vec<TemplatePart>,
    /// Required fields
    pub required_fields: Vec<String>,
    /// Estimated output size
    pub estimated_size: usize,
}

/// Template part (static or dynamic)
#[derive(Debug, Clone)]
pub enum TemplatePart {
    Static(OptimizedString),
    Placeholder(String), // Field name
}

/// Template cache configuration
#[derive(Debug)]
struct TemplateCacheConfig {
    max_entries: usize,
    ttl: Duration,
}

impl Default for TemplateCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 100,
            ttl: Duration::from_secs(1800), // 30 minutes
        }
    }
}

/// Cache statistics
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub schema_hits: usize,
    pub schema_misses: usize,
    pub validation_hits: usize,
    pub validation_misses: usize,
    pub hash_hits: usize,
    pub hash_misses: usize,
    pub template_hits: usize,
    pub template_misses: usize,
}

impl CacheStats {
    /// Calculate overall hit rate
    pub fn overall_hit_rate(&self) -> f64 {
        let total_hits = self.schema_hits + self.validation_hits + self.hash_hits + self.template_hits;
        let total_requests = total_hits + self.schema_misses + self.validation_misses + 
                           self.hash_misses + self.template_misses;
        
        if total_requests == 0 {
            0.0
        } else {
            total_hits as f64 / total_requests as f64
        }
    }
    
    /// Get cache efficiency summary
    pub fn summary(&self) -> String {
        format!(
            "Cache Hit Rate: {:.1}% (Schema: {:.1}%, Validation: {:.1}%, Hash: {:.1}%, Template: {:.1}%)",
            self.overall_hit_rate() * 100.0,
            self.schema_hit_rate() * 100.0,
            self.validation_hit_rate() * 100.0,
            self.hash_hit_rate() * 100.0,
            self.template_hit_rate() * 100.0,
        )
    }
    
    fn schema_hit_rate(&self) -> f64 {
        let total = self.schema_hits + self.schema_misses;
        if total == 0 { 0.0 } else { self.schema_hits as f64 / total as f64 }
    }
    
    fn validation_hit_rate(&self) -> f64 {
        let total = self.validation_hits + self.validation_misses;
        if total == 0 { 0.0 } else { self.validation_hits as f64 / total as f64 }
    }
    
    fn hash_hit_rate(&self) -> f64 {
        let total = self.hash_hits + self.hash_misses;
        if total == 0 { 0.0 } else { self.hash_hits as f64 / total as f64 }
    }
    
    fn template_hit_rate(&self) -> f64 {
        let total = self.template_hits + self.template_misses;
        if total == 0 { 0.0 } else { self.template_hits as f64 / total as f64 }
    }
}

/// Public API for cache operations
pub struct CacheManager;

impl CacheManager {
    /// Get global cache statistics
    pub fn stats() -> CacheStats {
        GLOBAL_CACHE.read().unwrap().stats().clone()
    }
    
    /// Clear all global caches
    pub fn clear_all() {
        GLOBAL_CACHE.write().unwrap().clear_all();
    }
    
    /// Prune expired entries
    pub fn prune_expired() {
        GLOBAL_CACHE.write().unwrap().prune_expired();
    }
    
    /// Get schema from cache or compile
    pub fn get_schema(
        version: &str,
        profile: Option<&str>,
        compiler: impl FnOnce() -> Result<CompiledSchema, BuildError>,
    ) -> Result<CompiledSchema, BuildError> {
        let mut cache = GLOBAL_CACHE.write().unwrap();
        let schema = cache.schemas.get_or_compile(version, profile, compiler)?;
        Ok(schema.clone())
    }
    
    /// Fast hash computation with caching
    pub fn fast_hash<T: Hash + std::fmt::Debug>(algorithm: &str, content_type: &str, content_id: &str, value: &T) -> String {
        let key = HashKey {
            algorithm: algorithm.to_string(),
            content_type: content_type.to_string(),
            content_id: content_id.to_string(),
        };
        
        let mut cache = GLOBAL_CACHE.write().unwrap();
        cache.hash_cache.get_or_compute(&key, value, |v| {
            match algorithm {
                "blake3" => {
                    let mut hasher = Blake3Hasher::new();
                    let bytes = format!("{:?}", v); // Simple serialization for hashing
                    hasher.update(bytes.as_bytes());
                    hasher.finalize().to_hex().to_string()
                }
                _ => {
                    // Fallback to default hasher
                    let mut hasher = std::hash::DefaultHasher::new();
                    v.hash(&mut hasher);
                    format!("{:016x}", hasher.finish())
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_cache() {
        let mut cache = SchemaCache::new();
        
        // Test cache miss and compilation
        let schema = cache.get_or_compile("4.3", None, || {
            Ok(CompiledSchema {
                version: "4.3".to_string(),
                profile: None,
                rules: vec![],
                required_elements: vec!["MessageHeader".to_string()],
                element_constraints: IndexMap::new(),
            })
        }).unwrap();
        
        assert_eq!(schema.version, "4.3");
        assert!(cache.contains("4.3", None));
        
        // Test cache hit
        let schema2 = cache.get_or_compile("4.3", None, || {
            panic!("Should not compile again")
        }).unwrap();
        
        assert_eq!(schema2.version, "4.3");
    }
    
    #[test]
    fn test_validation_cache() {
        let mut cache = ValidationCache::new();
        let hash = "test_hash".to_string();
        
        // Cache miss
        assert!(cache.get(&hash).is_none());
        
        // Insert result
        let result = ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
            validation_time: Duration::from_millis(10),
        };
        cache.insert(hash.clone(), result);
        
        // Cache hit
        let cached = cache.get(&hash).unwrap();
        assert!(cached.is_valid);
    }
    
    #[test]
    fn test_hash_cache() {
        let mut cache = HashCache::new();
        let key = HashKey {
            algorithm: "blake3".to_string(),
            content_type: "track".to_string(),
            content_id: "T001".to_string(),
        };
        
        let test_value = "test content";
        
        // First computation
        let hash1 = cache.get_or_compute(&key, &test_value, |v| {
            format!("hash_{}", v)
        });
        
        // Second computation (should be cached)
        let hash2 = cache.get_or_compute(&key, &test_value, |_| {
            panic!("Should not compute again")
        });
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1, "hash_test content");
    }
    
    #[test]
    fn test_cache_manager() {
        CacheManager::clear_all();
        let stats = CacheManager::stats();
        assert_eq!(stats.overall_hit_rate(), 0.0);
        
        // Test fast hash
        let hash1 = CacheManager::fast_hash("blake3", "test", "item1", &"content");
        let hash2 = CacheManager::fast_hash("blake3", "test", "item1", &"content");
        assert_eq!(hash1, hash2); // Should be cached
    }
    
    #[test]
    fn test_template_cache() {
        let mut cache = TemplateCache::new();
        let key = TemplateKey {
            element_type: "SoundRecording".to_string(),
            version: "4.3".to_string(),
            variant: None,
        };
        
        // First access - compile template
        let template = cache.get_or_compile(&key, || {
            CompiledTemplate {
                parts: vec![
                    TemplatePart::Static(OptimizedString::new("<SoundRecording>")),
                    TemplatePart::Placeholder("title".to_string()),
                    TemplatePart::Static(OptimizedString::new("</SoundRecording>")),
                ],
                required_fields: vec!["title".to_string()],
                estimated_size: 100,
            }
        });
        
        assert_eq!(template.required_fields.len(), 1);
        
        // Second access - should use cached
        let template2 = cache.get_or_compile(&key, || {
            panic!("Should not compile again")
        });
        
        assert_eq!(template2.required_fields.len(), 1);
    }
}