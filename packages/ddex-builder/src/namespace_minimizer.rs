//! # Namespace Minimizer for DDEX Builder
//! 
//! This module provides functionality to minimize namespace declarations in generated XML,
//! hoisting declarations to the root when possible and applying locked prefixes.

use ddex_core::namespace::{NamespaceRegistry, NamespaceScope, ConflictResolution};
use ddex_core::models::versions::ERNVersion;
use crate::canonical::rules::CanonicalNamespaceManager;
use crate::ast::{AST, Element, Node};
use indexmap::{IndexMap, IndexSet};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Namespace minimization result
#[derive(Debug, Clone)]
pub struct MinimizationResult {
    /// Minimal namespace declarations for root element
    pub root_namespaces: IndexMap<String, String>,
    /// Updated AST with minimized namespace declarations
    pub optimized_ast: AST,
    /// Mapping of old prefixes to new prefixes
    pub prefix_mapping: IndexMap<String, String>,
    /// Warnings about namespace changes
    pub warnings: Vec<String>,
}

/// Namespace usage analysis
#[derive(Debug, Clone)]
pub struct NamespaceUsage {
    /// All namespaces used in the document
    pub used_namespaces: IndexSet<String>,
    /// Elements that use each namespace
    pub namespace_elements: HashMap<String, IndexSet<String>>,
    /// Attribute namespaces
    pub attribute_namespaces: IndexSet<String>,
}

/// Comprehensive namespace minimizer
pub struct NamespaceMinimizer {
    /// Namespace registry for known namespaces
    registry: NamespaceRegistry,
    /// Canonical namespace manager
    canonical_manager: CanonicalNamespaceManager,
    /// ERN version for version-specific rules
    version: ERNVersion,
    /// Conflict resolution strategy
    conflict_resolution: ConflictResolution,
}

impl NamespaceMinimizer {
    /// Create new namespace minimizer
    pub fn new(version: ERNVersion) -> Self {
        Self {
            registry: NamespaceRegistry::new(),
            canonical_manager: CanonicalNamespaceManager::new(),
            version,
            conflict_resolution: ConflictResolution::GenerateUnique,
        }
    }

    /// Create namespace minimizer with specific conflict resolution
    pub fn with_conflict_resolution(mut self, strategy: ConflictResolution) -> Self {
        self.conflict_resolution = strategy;
        self
    }

    /// Minimize namespace declarations in AST
    pub fn minimize(&self, ast: AST) -> Result<MinimizationResult, String> {
        debug!("Starting namespace minimization for ERN {:?}", self.version);
        
        // Step 1: Analyze namespace usage throughout the document
        let usage = self.analyze_namespace_usage(&ast)?;
        debug!("Found {} used namespaces", usage.used_namespaces.len());
        
        // Step 2: Create optimal namespace declarations for root
        let root_namespaces = self.create_minimal_root_declarations(&usage)?;
        
        // Step 3: Apply canonical namespace transformations
        let canonical_namespaces = self.apply_canonical_rules(&root_namespaces)?;
        
        // Step 4: Update AST with optimized namespace declarations
        let (optimized_ast, prefix_mapping) = self.apply_namespace_minimization(ast, &canonical_namespaces)?;
        
        // Step 5: Validate the result
        let warnings = self.validate_minimization(&optimized_ast, &canonical_namespaces);
        
        Ok(MinimizationResult {
            root_namespaces: canonical_namespaces,
            optimized_ast,
            prefix_mapping,
            warnings,
        })
    }

    /// Analyze namespace usage throughout the document
    fn analyze_namespace_usage(&self, ast: &AST) -> Result<NamespaceUsage, String> {
        let mut used_namespaces = IndexSet::new();
        let mut namespace_elements = HashMap::new();
        let mut attribute_namespaces = IndexSet::new();

        // Add namespaces already declared in AST
        for (prefix, uri) in &ast.namespaces {
            used_namespaces.insert(uri.clone());
            namespace_elements.entry(uri.clone()).or_insert_with(IndexSet::new);
        }

        // Analyze namespace usage in element tree
        self.analyze_element_usage(&ast.root, &mut used_namespaces, &mut namespace_elements, &mut attribute_namespaces);

        // Add required namespaces for the ERN version
        let required_namespaces = self.registry.get_version_namespaces(&self.version);
        for ns_uri in required_namespaces {
            used_namespaces.insert(ns_uri);
        }

        Ok(NamespaceUsage {
            used_namespaces,
            namespace_elements,
            attribute_namespaces,
        })
    }

    /// Recursively analyze namespace usage in elements
    fn analyze_element_usage(
        &self,
        element: &Element,
        used_namespaces: &mut IndexSet<String>,
        namespace_elements: &mut HashMap<String, IndexSet<String>>,
        attribute_namespaces: &mut IndexSet<String>,
    ) {
        // Check element namespace
        if let Some(ref ns) = element.namespace {
            used_namespaces.insert(ns.clone());
            namespace_elements
                .entry(ns.clone())
                .or_insert_with(IndexSet::new)
                .insert(element.name.clone());
        }

        // Check attribute namespaces
        for (attr_name, _) in &element.attributes {
            if attr_name.contains(':') && !attr_name.starts_with("xmlns") {
                // Extract namespace prefix from qualified attribute name
                if let Some(prefix) = attr_name.split(':').next() {
                    // This would need the namespace URI, but for now just track the usage
                    debug!("Found namespaced attribute: {} with prefix: {}", attr_name, prefix);
                }
            }
        }

        // Recursively analyze children
        for child in &element.children {
            if let Node::Element(child_element) = child {
                self.analyze_element_usage(child_element, used_namespaces, namespace_elements, attribute_namespaces);
            }
        }
    }

    /// Create minimal namespace declarations for root element
    fn create_minimal_root_declarations(&self, usage: &NamespaceUsage) -> Result<IndexMap<String, String>, String> {
        let mut declarations = IndexMap::new();
        
        // Create declarations for all used namespaces
        for uri in &usage.used_namespaces {
            if let Some(preferred_prefix) = self.registry.get_preferred_prefix(uri) {
                declarations.insert(preferred_prefix.to_string(), uri.clone());
            } else {
                // For unknown namespaces, generate a prefix
                let generated_prefix = self.generate_prefix_for_uri(uri);
                declarations.insert(generated_prefix, uri.clone());
            }
        }

        Ok(declarations)
    }

    /// Apply canonical namespace rules
    fn apply_canonical_rules(&self, declarations: &IndexMap<String, String>) -> Result<IndexMap<String, String>, String> {
        let version_str = match self.version {
            ERNVersion::V3_8_2 => "3.8.2",
            ERNVersion::V4_2 => "4.2",
            ERNVersion::V4_3 => "4.3",
        };

        Ok(self.canonical_manager.canonicalize_namespaces(declarations, version_str))
    }

    /// Apply namespace minimization to AST
    fn apply_namespace_minimization(
        &self,
        mut ast: AST,
        canonical_namespaces: &IndexMap<String, String>,
    ) -> Result<(AST, IndexMap<String, String>), String> {
        // Update AST namespaces with canonical declarations
        ast.namespaces = canonical_namespaces.clone();

        // Create prefix mapping for any changes
        let mut prefix_mapping = IndexMap::new();
        
        // For now, assume no prefix changes (would be more complex in full implementation)
        for (prefix, _) in canonical_namespaces {
            prefix_mapping.insert(prefix.clone(), prefix.clone());
        }

        // Update element prefixes if needed (recursive through tree)
        self.update_element_prefixes(&mut ast.root, &prefix_mapping);

        Ok((ast, prefix_mapping))
    }

    /// Update element prefixes based on mapping
    fn update_element_prefixes(&self, element: &mut Element, _prefix_mapping: &IndexMap<String, String>) {
        // This would update element names and attributes based on prefix changes
        // For now, keep existing prefixes
        
        // Recursively update children
        for child in &mut element.children {
            if let Node::Element(child_element) = child {
                self.update_element_prefixes(child_element, _prefix_mapping);
            }
        }
    }

    /// Generate a prefix for an unknown URI
    fn generate_prefix_for_uri(&self, uri: &str) -> String {
        // Simple heuristic: use domain name or create generic prefix
        if let Some(domain_start) = uri.find("://") {
            if let Some(domain_part) = uri[domain_start + 3..].split('/').next() {
                let domain_clean = domain_part.replace('.', "").replace('-', "");
                if !domain_clean.is_empty() && domain_clean.len() <= 8 {
                    return format!("ns{}", domain_clean.chars().take(3).collect::<String>());
                }
            }
        }
        
        // Fallback: generate based on hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        uri.hash(&mut hasher);
        let hash = hasher.finish();
        format!("ns{}", hash % 1000)
    }

    /// Validate the minimization result
    fn validate_minimization(&self, ast: &AST, _namespaces: &IndexMap<String, String>) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for unused namespace declarations
        let declared_uris: IndexSet<_> = ast.namespaces.values().cloned().collect();
        let usage = match self.analyze_namespace_usage(ast) {
            Ok(usage) => usage,
            Err(e) => {
                warnings.push(format!("Failed to re-analyze namespace usage: {}", e));
                return warnings;
            }
        };

        for uri in &declared_uris {
            if !usage.used_namespaces.contains(uri) {
                warnings.push(format!("Declared but unused namespace: {}", uri));
            }
        }

        // Check for missing required namespaces
        let required_namespaces = self.registry.get_version_namespaces(&self.version);
        for required_uri in required_namespaces {
            if !declared_uris.contains(&required_uri) {
                warnings.push(format!("Missing required namespace: {}", required_uri));
            }
        }

        warnings
    }

    /// Hoist namespace declarations to root element when beneficial
    pub fn hoist_namespaces(&self, mut ast: AST) -> Result<AST, String> {
        // This would analyze which namespace declarations can be moved to root
        // For now, assume all namespaces are already at root level
        debug!("Hoisting namespaces to root level");
        
        // Remove duplicate namespace declarations from child elements
        self.remove_duplicate_declarations(&mut ast.root, &ast.namespaces);
        
        Ok(ast)
    }

    /// Remove duplicate namespace declarations from child elements
    fn remove_duplicate_declarations(&self, element: &mut Element, root_namespaces: &IndexMap<String, String>) {
        // Remove xmlns attributes that duplicate root declarations
        let xmlns_keys: Vec<String> = element.attributes.keys()
            .filter(|k| k.starts_with("xmlns"))
            .cloned()
            .collect();
            
        for xmlns_key in xmlns_keys {
            if let Some(uri) = element.attributes.get(&xmlns_key) {
                let prefix = if xmlns_key == "xmlns" {
                    ""
                } else {
                    xmlns_key.strip_prefix("xmlns:").unwrap_or("")
                };
                
                // If this namespace is already declared at root with same prefix, remove it
                if root_namespaces.get(prefix).map(|root_uri| root_uri == uri).unwrap_or(false) {
                    element.attributes.remove(&xmlns_key);
                    debug!("Removed duplicate namespace declaration: {} from element {}", xmlns_key, element.name);
                }
            }
        }

        // Recursively process children
        for child in &mut element.children {
            if let Node::Element(child_element) = child {
                self.remove_duplicate_declarations(child_element, root_namespaces);
            }
        }
    }
}

/// Namespace optimization strategies
#[derive(Debug, Clone, Copy)]
pub enum OptimizationStrategy {
    /// Minimal declarations (default)
    Minimal,
    /// Hoist all possible declarations to root
    HoistAll,
    /// Conservative approach, keep existing structure
    Conservative,
}

/// Advanced namespace minimizer with optimization strategies
pub struct AdvancedNamespaceMinimizer {
    base_minimizer: NamespaceMinimizer,
    strategy: OptimizationStrategy,
}

impl AdvancedNamespaceMinimizer {
    pub fn new(version: ERNVersion, strategy: OptimizationStrategy) -> Self {
        Self {
            base_minimizer: NamespaceMinimizer::new(version),
            strategy,
        }
    }

    pub fn minimize(&self, ast: AST) -> Result<MinimizationResult, String> {
        match self.strategy {
            OptimizationStrategy::Minimal => self.base_minimizer.minimize(ast),
            OptimizationStrategy::HoistAll => {
                let minimized = self.base_minimizer.minimize(ast)?;
                let hoisted_ast = self.base_minimizer.hoist_namespaces(minimized.optimized_ast)?;
                Ok(MinimizationResult {
                    optimized_ast: hoisted_ast,
                    ..minimized
                })
            },
            OptimizationStrategy::Conservative => {
                // Conservative approach: minimal changes
                let mut result = self.base_minimizer.minimize(ast)?;
                result.warnings.push("Conservative mode: minimal namespace optimization applied".to_string());
                Ok(result)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Element, Node};

    #[test]
    fn test_namespace_minimizer_creation() {
        let minimizer = NamespaceMinimizer::new(ERNVersion::V4_3);
        assert!(matches!(minimizer.version, ERNVersion::V4_3));
    }

    #[test]
    fn test_namespace_usage_analysis() {
        let minimizer = NamespaceMinimizer::new(ERNVersion::V4_3);
        
        // Create test AST
        let mut root = Element::new("NewReleaseMessage")
            .with_namespace("http://ddex.net/xml/ern/43");
        root.add_child(Element::new("MessageHeader")
            .with_namespace("http://ddex.net/xml/ern/43"));
        
        let mut ast = AST {
            root,
            namespaces: {
                let mut ns = IndexMap::new();
                ns.insert("ern".to_string(), "http://ddex.net/xml/ern/43".to_string());
                ns
            },
            schema_location: None,
        };
        
        let usage = minimizer.analyze_namespace_usage(&ast).unwrap();
        assert!(usage.used_namespaces.contains("http://ddex.net/xml/ern/43"));
    }

    #[test]
    fn test_prefix_generation() {
        let minimizer = NamespaceMinimizer::new(ERNVersion::V4_3);
        
        let prefix = minimizer.generate_prefix_for_uri("http://example.com/custom");
        assert!(prefix.starts_with("ns"));
        assert!(prefix.len() <= 10); // Reasonable length
    }

    #[test]
    fn test_minimal_declarations() {
        let minimizer = NamespaceMinimizer::new(ERNVersion::V4_3);
        
        let mut usage = NamespaceUsage {
            used_namespaces: IndexSet::new(),
            namespace_elements: HashMap::new(),
            attribute_namespaces: IndexSet::new(),
        };
        
        usage.used_namespaces.insert("http://ddex.net/xml/ern/43".to_string());
        usage.used_namespaces.insert("http://ddex.net/xml/avs".to_string());
        
        let declarations = minimizer.create_minimal_root_declarations(&usage).unwrap();
        assert!(declarations.contains_key("ern"));
        assert!(declarations.contains_key("avs"));
    }
}