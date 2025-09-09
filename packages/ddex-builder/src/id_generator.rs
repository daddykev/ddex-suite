// packages/ddex-builder/src/id_generator.rs
//! Stable hash-based ID generation for deterministic DDEX messages

use sha2::{Sha256, Digest};
use blake3;
use serde::{Deserialize, Serialize};
use indexmap::IndexMap;
use unicode_normalization::UnicodeNormalization;

/// Stable hash configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StableHashConfig {
    /// Recipe version to use
    pub recipe: String,
    
    /// Hash algorithm
    pub algorithm: HashAlgorithm,
    
    /// Whether to cache generated IDs
    pub use_cache: bool,
    
    /// Salt for hash generation
    pub salt: Option<String>,
}

impl Default for StableHashConfig {
    fn default() -> Self {
        Self {
            recipe: "v1".to_string(),
            algorithm: HashAlgorithm::Blake3,
            use_cache: true,
            salt: None,
        }
    }
}

/// Hash algorithm for stable ID generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    /// SHA-256
    Sha256,
    /// Blake3 (faster, more secure)
    Blake3,
}

/// Recipe for stable hash generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashRecipe {
    /// Fields to include in hash
    pub fields: Vec<String>,
    
    /// Normalization options
    pub normalize: NormalizeOptions,
    
    /// Salt for this entity type
    pub salt: String,
}

/// Normalization options for stable hashing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizeOptions {
    /// Unicode normalization form
    pub unicode: UnicodeForm,
    
    /// Whether to trim whitespace
    pub trim: bool,
    
    /// Case normalization
    pub case: CaseNormalization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnicodeForm {
    NFC,
    NFD,
    NFKC,
    NFKD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseNormalization {
    AsIs,
    Lower,
    Upper,
}

/// Stable hash ID generator
pub struct StableHashGenerator {
    config: StableHashConfig,
    recipes: IndexMap<String, HashRecipe>,
    cache: IndexMap<String, String>,
}

impl StableHashGenerator {
    /// Create new generator with config
    pub fn new(config: StableHashConfig) -> Self {
        Self {
            config,
            recipes: Self::load_recipes(),
            cache: IndexMap::new(),
        }
    }
    
    /// Generate stable ID for a release
    pub fn generate_release_id(
        &mut self,
        upc: &str,
        release_type: &str,
        track_isrcs: &[String],
        territory_set: &[String],
    ) -> Result<String, super::error::BuildError> {
        let materials = ReleaseHashMaterials {
            upc: upc.to_string(),
            release_type: release_type.to_string(),
            track_isrcs: track_isrcs.to_vec(),
            territory_set: territory_set.to_vec(),
        };
        
        self.generate("Release", &materials)
    }
    
    /// Generate stable ID for a resource
    pub fn generate_resource_id(
        &mut self,
        isrc: &str,
        duration: u32,
        file_hash: Option<&str>,
    ) -> Result<String, super::error::BuildError> {
        let materials = ResourceHashMaterials {
            isrc: isrc.to_string(),
            duration,
            file_hash: file_hash.map(|s| s.to_string()),
        };
        
        self.generate("Resource", &materials)
    }
    
    /// Generate stable ID for a party
    pub fn generate_party_id(
        &mut self,
        name: &str,
        role: &str,
        identifiers: &[String],
    ) -> Result<String, super::error::BuildError> {
        let materials = PartyHashMaterials {
            name: name.to_string(),
            role: role.to_string(),
            identifiers: identifiers.to_vec(),
        };
        
        self.generate("Party", &materials)
    }
    
    /// Generic stable ID generation
    fn generate<T: Serialize>(
        &mut self,
        entity_type: &str,
        materials: &T,
    ) -> Result<String, super::error::BuildError> {
        // Create cache key
        let cache_key = format!("{}:{}", entity_type, 
            serde_json::to_string(materials)?);
        
        // Check cache
        if self.config.use_cache {
            if let Some(cached) = self.cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }
        
        // Get recipe
        let recipe = self.recipes.get(&format!("{}.{}", entity_type, self.config.recipe))
            .ok_or_else(|| super::error::BuildError::InvalidFormat {
                field: "recipe".to_string(),
                message: format!("No recipe for {}.{}", entity_type, self.config.recipe),
            })?;
        
        // Normalize and concatenate fields
        let normalized = self.normalize_materials(materials, recipe)?;
        
        // Generate hash
        let id = match self.config.algorithm {
            HashAlgorithm::Sha256 => self.hash_sha256(&normalized, &recipe.salt),
            HashAlgorithm::Blake3 => self.hash_blake3(&normalized, &recipe.salt),
        };
        
        // Cache result
        if self.config.use_cache {
            self.cache.insert(cache_key, id.clone());
        }
        
        Ok(id)
    }
    
    fn normalize_materials<T: Serialize>(
        &self,
        materials: &T,
        recipe: &HashRecipe,
    ) -> Result<String, super::error::BuildError> {
        let json = serde_json::to_value(materials)?;
        let mut parts = Vec::new();
        
        for field in &recipe.fields {
            if let Some(value) = json.get(field) {
                let normalized = self.normalize_value(value, &recipe.normalize)?;
                parts.push(normalized);
            }
        }
        
        Ok(parts.join("|"))
    }
    
    fn normalize_value(
        &self,
        value: &serde_json::Value,
        options: &NormalizeOptions,
    ) -> Result<String, super::error::BuildError> {
        let text = match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(arr) => {
                let strings: Vec<String> = arr.iter()
                    .map(|v| self.normalize_value(v, options))
                    .collect::<Result<Vec<_>, _>>()?;
                strings.join(",")
            },
            _ => serde_json::to_string(value)?,
        };
        
        // Apply normalization
        let mut normalized = text;
        
        // Unicode normalization
        normalized = match options.unicode {
            UnicodeForm::NFC => normalized.nfc().collect(),
            UnicodeForm::NFD => normalized.nfd().collect(),
            UnicodeForm::NFKC => normalized.nfkc().collect(),
            UnicodeForm::NFKD => normalized.nfkd().collect(),
        };
        
        // Trim
        if options.trim {
            normalized = normalized.trim().to_string();
        }
        
        // Case normalization
        normalized = match options.case {
            CaseNormalization::AsIs => normalized,
            CaseNormalization::Lower => normalized.to_lowercase(),
            CaseNormalization::Upper => normalized.to_uppercase(),
        };
        
        Ok(normalized)
    }
    
    fn hash_sha256(&self, input: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(input.as_bytes());
        if let Some(global_salt) = &self.config.salt {
            hasher.update(global_salt.as_bytes());
        }
        let result = hasher.finalize();
        format!("SHA256:{:x}", result)
    }
    
    fn hash_blake3(&self, input: &str, salt: &str) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(salt.as_bytes());
        hasher.update(input.as_bytes());
        if let Some(global_salt) = &self.config.salt {
            hasher.update(global_salt.as_bytes());
        }
        let hash = hasher.finalize();
        format!("B3:{}", hash.to_hex())
    }
    
    fn load_recipes() -> IndexMap<String, HashRecipe> {
        let mut recipes = IndexMap::new();
        
        // Release v1 recipe
        recipes.insert("Release.v1".to_string(), HashRecipe {
            fields: vec![
                "upc".to_string(),
                "release_type".to_string(),
                "track_isrcs".to_string(),
                "territory_set".to_string(),
            ],
            normalize: NormalizeOptions {
                unicode: UnicodeForm::NFC,
                trim: true,
                case: CaseNormalization::AsIs,
            },
            salt: "REL@1".to_string(),
        });
        
        // Resource v1 recipe
        recipes.insert("Resource.v1".to_string(), HashRecipe {
            fields: vec![
                "isrc".to_string(),
                "duration".to_string(),
                "file_hash".to_string(),
            ],
            normalize: NormalizeOptions {
                unicode: UnicodeForm::NFC,
                trim: true,
                case: CaseNormalization::AsIs,
            },
            salt: "RES@1".to_string(),
        });
        
        // Party v1 recipe
        recipes.insert("Party.v1".to_string(), HashRecipe {
            fields: vec![
                "name".to_string(),
                "role".to_string(),
                "identifiers".to_string(),
            ],
            normalize: NormalizeOptions {
                unicode: UnicodeForm::NFC,
                trim: true,
                case: CaseNormalization::Lower,
            },
            salt: "PTY@1".to_string(),
        });
        
        recipes
    }
}

// Hash material structures
#[derive(Debug, Serialize)]
struct ReleaseHashMaterials {
    upc: String,
    release_type: String,
    track_isrcs: Vec<String>,
    territory_set: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ResourceHashMaterials {
    isrc: String,
    duration: u32,
    file_hash: Option<String>,
}

#[derive(Debug, Serialize)]
struct PartyHashMaterials {
    name: String,
    role: String,
    identifiers: Vec<String>,
}