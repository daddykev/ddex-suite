//! # DDEX Namespace Management
//!
//! This module provides comprehensive namespace management for the DDEX Suite,
//! handling multiple DDEX versions and custom extensions with proper prefix
//! management and collision detection.

use crate::models::versions::ERNVersion;
use indexmap::{IndexMap, IndexSet};

/// DDEX namespace registry for comprehensive namespace management
#[derive(Debug, Clone)]
pub struct NamespaceRegistry {
    /// Known DDEX namespace URIs mapped to their information
    namespaces: IndexMap<String, NamespaceInfo>,
    /// Default prefixes for known namespaces
    default_prefixes: IndexMap<String, String>,
    /// Custom namespace registrations
    custom_namespaces: IndexMap<String, NamespaceInfo>,
    /// Prefix collision detection
    reserved_prefixes: IndexSet<String>,
}

/// Information about a namespace
#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceInfo {
    /// The namespace URI
    pub uri: String,
    /// Preferred prefix
    pub preferred_prefix: String,
    /// Alternative acceptable prefixes
    pub alternative_prefixes: Vec<String>,
    /// DDEX standard this namespace belongs to
    pub standard: DDEXStandard,
    /// Version information if applicable
    pub version: Option<String>,
    /// Whether this namespace is required for the standard
    pub required: bool,
}

/// DDEX standards supported
#[derive(Debug, Clone, PartialEq)]
pub enum DDEXStandard {
    /// Electronic Release Notification
    ERN,
    /// Audio Video Vocabulary
    AVS,
    /// Musical Work Electronic Delivery
    MEAD,
    /// Performer Information Exchange
    PIE,
    /// Recording Information Notification
    RIN,
    /// Rights and Remuneration Information
    RRI,
    /// Sales and Usage Reporting
    DSRF,
    /// XML Schema Instance
    XMLSchema,
    /// Custom/Unknown standard
    Custom(String),
}

/// Namespace scope tracking for inheritance
#[derive(Debug, Clone)]
pub struct NamespaceScope {
    /// Active namespace declarations at this scope
    pub declarations: IndexMap<String, String>, // prefix -> uri
    /// Parent scope for inheritance
    pub parent: Option<Box<NamespaceScope>>,
    /// Element depth for debugging
    pub depth: usize,
}

/// Namespace conflict resolution strategy
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    /// Use the first declared prefix
    PreferFirst,
    /// Use the most recently declared prefix
    PreferLatest,
    /// Generate a unique prefix
    GenerateUnique,
    /// Throw an error
    Error,
}

impl NamespaceRegistry {
    /// Create a new namespace registry with all known DDEX namespaces
    pub fn new() -> Self {
        let mut registry = Self {
            namespaces: IndexMap::new(),
            default_prefixes: IndexMap::new(),
            custom_namespaces: IndexMap::new(),
            reserved_prefixes: IndexSet::new(),
        };

        registry.initialize_ddex_namespaces();
        registry
    }

    /// Initialize all known DDEX namespaces
    fn initialize_ddex_namespaces(&mut self) {
        // ERN namespaces
        self.register_namespace(NamespaceInfo {
            uri: "http://ddex.net/xml/ern/382".to_string(),
            preferred_prefix: "ern".to_string(),
            alternative_prefixes: vec!["ern382".to_string()],
            standard: DDEXStandard::ERN,
            version: Some("3.8.2".to_string()),
            required: true,
        });

        self.register_namespace(NamespaceInfo {
            uri: "http://ddex.net/xml/ern/42".to_string(),
            preferred_prefix: "ern".to_string(),
            alternative_prefixes: vec!["ern42".to_string()],
            standard: DDEXStandard::ERN,
            version: Some("4.2".to_string()),
            required: true,
        });

        self.register_namespace(NamespaceInfo {
            uri: "http://ddex.net/xml/ern/43".to_string(),
            preferred_prefix: "ern".to_string(),
            alternative_prefixes: vec!["ern43".to_string()],
            standard: DDEXStandard::ERN,
            version: Some("4.3".to_string()),
            required: true,
        });

        // AVS namespaces
        self.register_namespace(NamespaceInfo {
            uri: "http://ddex.net/xml/avs".to_string(),
            preferred_prefix: "avs".to_string(),
            alternative_prefixes: vec!["ddexavs".to_string()],
            standard: DDEXStandard::AVS,
            version: None,
            required: false,
        });

        self.register_namespace(NamespaceInfo {
            uri: "http://ddex.net/xml/avs/avs".to_string(),
            preferred_prefix: "avs".to_string(),
            alternative_prefixes: vec!["ddexavs".to_string()],
            standard: DDEXStandard::AVS,
            version: None,
            required: false,
        });

        // MEAD namespaces
        self.register_namespace(NamespaceInfo {
            uri: "http://ddex.net/xml/mead/mead".to_string(),
            preferred_prefix: "mead".to_string(),
            alternative_prefixes: vec!["ddexmead".to_string()],
            standard: DDEXStandard::MEAD,
            version: None,
            required: false,
        });

        // PIE namespaces
        self.register_namespace(NamespaceInfo {
            uri: "http://ddex.net/xml/pie/pie".to_string(),
            preferred_prefix: "pie".to_string(),
            alternative_prefixes: vec!["ddexpie".to_string()],
            standard: DDEXStandard::PIE,
            version: None,
            required: false,
        });

        // RIN namespaces
        self.register_namespace(NamespaceInfo {
            uri: "http://ddex.net/xml/rin/rin".to_string(),
            preferred_prefix: "rin".to_string(),
            alternative_prefixes: vec!["ddexrin".to_string()],
            standard: DDEXStandard::RIN,
            version: None,
            required: false,
        });

        // XML Schema Instance
        self.register_namespace(NamespaceInfo {
            uri: "http://www.w3.org/2001/XMLSchema-instance".to_string(),
            preferred_prefix: "xsi".to_string(),
            alternative_prefixes: vec!["xmlschema".to_string()],
            standard: DDEXStandard::XMLSchema,
            version: None,
            required: false,
        });

        // XML Schema
        self.register_namespace(NamespaceInfo {
            uri: "http://www.w3.org/2001/XMLSchema".to_string(),
            preferred_prefix: "xs".to_string(),
            alternative_prefixes: vec!["xsd".to_string(), "schema".to_string()],
            standard: DDEXStandard::XMLSchema,
            version: None,
            required: false,
        });

        // Common extension namespaces
        self.register_namespace(NamespaceInfo {
            uri: "http://ddex.net/xml/gc".to_string(),
            preferred_prefix: "gc".to_string(),
            alternative_prefixes: vec!["ddexgc".to_string()],
            standard: DDEXStandard::Custom("GC".to_string()),
            version: None,
            required: false,
        });
    }

    /// Register a new namespace
    pub fn register_namespace(&mut self, info: NamespaceInfo) {
        self.default_prefixes
            .insert(info.uri.clone(), info.preferred_prefix.clone());
        self.reserved_prefixes.insert(info.preferred_prefix.clone());

        for alt_prefix in &info.alternative_prefixes {
            self.reserved_prefixes.insert(alt_prefix.clone());
        }

        self.namespaces.insert(info.uri.clone(), info);
    }

    /// Register a custom namespace
    pub fn register_custom_namespace(&mut self, info: NamespaceInfo) -> Result<(), NamespaceError> {
        // Check for URI conflicts
        if self.namespaces.contains_key(&info.uri) || self.custom_namespaces.contains_key(&info.uri)
        {
            return Err(NamespaceError::UriConflict(info.uri));
        }

        // Check for prefix conflicts
        if self.is_prefix_reserved(&info.preferred_prefix) {
            return Err(NamespaceError::PrefixConflict(info.preferred_prefix));
        }

        self.reserved_prefixes.insert(info.preferred_prefix.clone());
        self.custom_namespaces.insert(info.uri.clone(), info);
        Ok(())
    }

    /// Detect DDEX version from namespace URI
    pub fn detect_version(&self, namespace_uri: &str) -> Option<ERNVersion> {
        match namespace_uri {
            // ERN 3.8.1 is treated as 3.8.2 (minor differences, compatible structure)
            "http://ddex.net/xml/ern/381" | "http://ddex.net/xml/ern/382" => Some(ERNVersion::V3_8_2),
            "http://ddex.net/xml/ern/42" => Some(ERNVersion::V4_2),
            "http://ddex.net/xml/ern/43" => Some(ERNVersion::V4_3),
            _ => None,
        }
    }

    /// Get all namespace URIs for a specific ERN version
    pub fn get_version_namespaces(&self, version: &ERNVersion) -> Vec<String> {
        let mut namespaces = vec![];

        // Add the main ERN namespace
        match version {
            ERNVersion::V3_8_2 => namespaces.push("http://ddex.net/xml/ern/382".to_string()),
            ERNVersion::V4_2 => namespaces.push("http://ddex.net/xml/ern/42".to_string()),
            ERNVersion::V4_3 => namespaces.push("http://ddex.net/xml/ern/43".to_string()),
        }

        // Add common supporting namespaces
        namespaces.push("http://ddex.net/xml/avs".to_string());
        namespaces.push("http://www.w3.org/2001/XMLSchema-instance".to_string());

        namespaces
    }

    /// Get preferred prefix for a namespace URI
    pub fn get_preferred_prefix(&self, uri: &str) -> Option<&str> {
        self.default_prefixes
            .get(uri)
            .map(|s| s.as_str())
            .or_else(|| {
                self.custom_namespaces
                    .get(uri)
                    .map(|info| info.preferred_prefix.as_str())
            })
    }

    /// Get namespace info by URI
    pub fn get_namespace_info(&self, uri: &str) -> Option<&NamespaceInfo> {
        self.namespaces
            .get(uri)
            .or_else(|| self.custom_namespaces.get(uri))
    }

    /// Check if a prefix is reserved
    pub fn is_prefix_reserved(&self, prefix: &str) -> bool {
        self.reserved_prefixes.contains(prefix)
    }

    /// Generate a unique prefix for a namespace
    pub fn generate_unique_prefix(&self, base_prefix: &str) -> String {
        if !self.is_prefix_reserved(base_prefix) {
            return base_prefix.to_string();
        }

        let mut counter = 1;
        loop {
            let candidate = format!("{}{}", base_prefix, counter);
            if !self.is_prefix_reserved(&candidate) {
                return candidate;
            }
            counter += 1;
        }
    }

    /// Resolve namespace conflicts using the specified strategy
    pub fn resolve_prefix_conflict(
        &self,
        _uri: &str,
        existing_prefix: &str,
        new_prefix: &str,
        strategy: ConflictResolution,
    ) -> Result<String, NamespaceError> {
        match strategy {
            ConflictResolution::PreferFirst => Ok(existing_prefix.to_string()),
            ConflictResolution::PreferLatest => Ok(new_prefix.to_string()),
            ConflictResolution::GenerateUnique => Ok(self.generate_unique_prefix(new_prefix)),
            ConflictResolution::Error => {
                Err(NamespaceError::PrefixConflict(new_prefix.to_string()))
            }
        }
    }

    /// Create minimal namespace declarations for root element
    pub fn create_minimal_declarations(
        &self,
        used_namespaces: &[String],
    ) -> IndexMap<String, String> {
        let mut declarations = IndexMap::new();

        for uri in used_namespaces {
            if let Some(prefix) = self.get_preferred_prefix(uri) {
                declarations.insert(prefix.to_string(), uri.clone());
            }
        }

        declarations
    }

    /// Validate namespace declarations against known namespaces
    pub fn validate_declarations(
        &self,
        declarations: &IndexMap<String, String>,
    ) -> Vec<NamespaceWarning> {
        let mut warnings = Vec::new();

        for (prefix, uri) in declarations {
            if let Some(info) = self.get_namespace_info(uri) {
                // Check if using non-preferred prefix
                if prefix != &info.preferred_prefix && !info.alternative_prefixes.contains(prefix) {
                    warnings.push(NamespaceWarning::NonStandardPrefix {
                        uri: uri.clone(),
                        used_prefix: prefix.clone(),
                        preferred_prefix: info.preferred_prefix.clone(),
                    });
                }
            } else {
                // Unknown namespace
                warnings.push(NamespaceWarning::UnknownNamespace {
                    uri: uri.clone(),
                    prefix: prefix.clone(),
                });
            }
        }

        warnings
    }

    /// Get all registered namespaces for a standard
    pub fn get_namespaces_by_standard(&self, standard: &DDEXStandard) -> Vec<&NamespaceInfo> {
        self.namespaces
            .values()
            .chain(self.custom_namespaces.values())
            .filter(|info| &info.standard == standard)
            .collect()
    }
}

impl NamespaceScope {
    /// Create a new root scope
    pub fn new() -> Self {
        Self {
            declarations: IndexMap::new(),
            parent: None,
            depth: 0,
        }
    }

    /// Create a child scope
    pub fn new_child(&self) -> Self {
        Self {
            declarations: IndexMap::new(),
            parent: Some(Box::new(self.clone())),
            depth: self.depth + 1,
        }
    }

    /// Add a namespace declaration to this scope
    pub fn declare_namespace(&mut self, prefix: String, uri: String) {
        self.declarations.insert(prefix, uri);
    }

    /// Resolve a prefix to its URI, checking parent scopes
    pub fn resolve_prefix(&self, prefix: &str) -> Option<String> {
        if let Some(uri) = self.declarations.get(prefix) {
            Some(uri.clone())
        } else if let Some(parent) = &self.parent {
            parent.resolve_prefix(prefix)
        } else {
            None
        }
    }

    /// Get all active namespace declarations (including inherited)
    pub fn get_all_declarations(&self) -> IndexMap<String, String> {
        let mut all_declarations = IndexMap::new();

        // Start with parent declarations
        if let Some(parent) = &self.parent {
            all_declarations = parent.get_all_declarations();
        }

        // Override with current scope declarations
        for (prefix, uri) in &self.declarations {
            all_declarations.insert(prefix.clone(), uri.clone());
        }

        all_declarations
    }

    /// Check if a namespace is declared in this scope or parents
    pub fn is_namespace_declared(&self, uri: &str) -> bool {
        self.declarations
            .values()
            .any(|declared_uri| declared_uri == uri)
            || self
                .parent
                .as_ref()
                .is_some_and(|parent| parent.is_namespace_declared(uri))
    }

    /// Find the prefix for a namespace URI
    pub fn find_prefix_for_uri(&self, uri: &str) -> Option<String> {
        for (prefix, declared_uri) in &self.declarations {
            if declared_uri == uri {
                return Some(prefix.clone());
            }
        }

        if let Some(parent) = &self.parent {
            parent.find_prefix_for_uri(uri)
        } else {
            None
        }
    }
}

/// Namespace-related errors
#[derive(Debug, Clone, PartialEq)]
pub enum NamespaceError {
    /// URI conflict with existing namespace
    UriConflict(String),
    /// Prefix conflict with reserved prefix
    PrefixConflict(String),
    /// Invalid namespace URI format
    InvalidUri(String),
    /// Circular namespace dependency
    CircularDependency(Vec<String>),
}

impl std::fmt::Display for NamespaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NamespaceError::UriConflict(uri) => write!(f, "Namespace URI conflict: {}", uri),
            NamespaceError::PrefixConflict(prefix) => write!(f, "Prefix conflict: {}", prefix),
            NamespaceError::InvalidUri(uri) => write!(f, "Invalid namespace URI: {}", uri),
            NamespaceError::CircularDependency(chain) => {
                write!(f, "Circular namespace dependency: {}", chain.join(" -> "))
            }
        }
    }
}

impl std::error::Error for NamespaceError {}

/// Namespace-related warnings
#[derive(Debug, Clone, PartialEq)]
pub enum NamespaceWarning {
    /// Using non-standard prefix for known namespace
    NonStandardPrefix {
        uri: String,
        used_prefix: String,
        preferred_prefix: String,
    },
    /// Unknown/unregistered namespace
    UnknownNamespace { uri: String, prefix: String },
    /// Redundant namespace declaration
    RedundantDeclaration { uri: String, prefix: String },
}

impl std::fmt::Display for NamespaceWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NamespaceWarning::NonStandardPrefix {
                uri,
                used_prefix,
                preferred_prefix,
            } => {
                write!(
                    f,
                    "Non-standard prefix '{}' for namespace '{}', prefer '{}'",
                    used_prefix, uri, preferred_prefix
                )
            }
            NamespaceWarning::UnknownNamespace { uri, prefix } => {
                write!(f, "Unknown namespace '{}' with prefix '{}'", uri, prefix)
            }
            NamespaceWarning::RedundantDeclaration { uri, prefix } => {
                write!(
                    f,
                    "Redundant declaration of namespace '{}' with prefix '{}'",
                    uri, prefix
                )
            }
        }
    }
}

impl Default for NamespaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for NamespaceScope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_registry_creation() {
        let registry = NamespaceRegistry::new();
        assert!(registry
            .get_preferred_prefix("http://ddex.net/xml/ern/43")
            .is_some());
        assert_eq!(
            registry.get_preferred_prefix("http://ddex.net/xml/ern/43"),
            Some("ern")
        );
    }

    #[test]
    fn test_version_detection() {
        let registry = NamespaceRegistry::new();
        assert_eq!(
            registry.detect_version("http://ddex.net/xml/ern/382"),
            Some(ERNVersion::V3_8_2)
        );
        assert_eq!(
            registry.detect_version("http://ddex.net/xml/ern/42"),
            Some(ERNVersion::V4_2)
        );
        assert_eq!(
            registry.detect_version("http://ddex.net/xml/ern/43"),
            Some(ERNVersion::V4_3)
        );
        assert_eq!(
            registry.detect_version("http://unknown.com/namespace"),
            None
        );
    }

    #[test]
    fn test_custom_namespace_registration() {
        let mut registry = NamespaceRegistry::new();

        let custom_ns = NamespaceInfo {
            uri: "http://example.com/custom".to_string(),
            preferred_prefix: "ex".to_string(),
            alternative_prefixes: vec!["example".to_string()],
            standard: DDEXStandard::Custom("Example".to_string()),
            version: None,
            required: false,
        };

        assert!(registry.register_custom_namespace(custom_ns).is_ok());
        assert_eq!(
            registry.get_preferred_prefix("http://example.com/custom"),
            Some("ex")
        );
    }

    #[test]
    fn test_prefix_conflict_detection() {
        let mut registry = NamespaceRegistry::new();

        let conflicting_ns = NamespaceInfo {
            uri: "http://example.com/conflict".to_string(),
            preferred_prefix: "ern".to_string(), // Conflicts with existing ERN prefix
            alternative_prefixes: vec![],
            standard: DDEXStandard::Custom("Conflict".to_string()),
            version: None,
            required: false,
        };

        assert!(matches!(
            registry.register_custom_namespace(conflicting_ns),
            Err(NamespaceError::PrefixConflict(_))
        ));
    }

    #[test]
    fn test_namespace_scope() {
        let mut root_scope = NamespaceScope::new();
        root_scope.declare_namespace("ern".to_string(), "http://ddex.net/xml/ern/43".to_string());

        let mut child_scope = root_scope.new_child();
        child_scope.declare_namespace("avs".to_string(), "http://ddex.net/xml/avs".to_string());

        // Child should resolve both its own and parent declarations
        assert_eq!(
            child_scope.resolve_prefix("ern"),
            Some("http://ddex.net/xml/ern/43".to_string())
        );
        assert_eq!(
            child_scope.resolve_prefix("avs"),
            Some("http://ddex.net/xml/avs".to_string())
        );

        // Parent should not see child declarations
        assert_eq!(root_scope.resolve_prefix("avs"), None);
    }

    #[test]
    fn test_unique_prefix_generation() {
        let mut registry = NamespaceRegistry::new();

        // Reserve some test prefixes
        registry.reserved_prefixes.insert("test".to_string());
        registry.reserved_prefixes.insert("test1".to_string());

        let unique = registry.generate_unique_prefix("test");
        assert_eq!(unique, "test2");
    }

    #[test]
    fn test_minimal_declarations() {
        let registry = NamespaceRegistry::new();
        let used_namespaces = vec![
            "http://ddex.net/xml/ern/43".to_string(),
            "http://ddex.net/xml/avs".to_string(),
        ];

        let declarations = registry.create_minimal_declarations(&used_namespaces);
        assert_eq!(
            declarations.get("ern"),
            Some(&"http://ddex.net/xml/ern/43".to_string())
        );
        assert_eq!(
            declarations.get("avs"),
            Some(&"http://ddex.net/xml/avs".to_string())
        );
    }
}
