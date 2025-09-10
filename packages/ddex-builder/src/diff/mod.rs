//! Semantic diff engine for DDEX messages
//! 
//! This module provides intelligent diffing that understands DDEX business semantics,
//! not just XML structure. It can detect meaningful changes while ignoring formatting
//! differences, reference variations, and insignificant ordering changes.

pub mod types;
pub mod formatter;

#[cfg(test)]
pub mod test_data;

#[cfg(test)]
mod diff_tests;

use crate::error::BuildError;
use crate::ast::{AST, Element, Node};
use types::{ChangeSet, SemanticChange, DiffPath, ChangeType};
use indexmap::{IndexMap, IndexSet};
use serde::{Serialize, Deserialize};

/// Configuration for semantic diffing behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffConfig {
    /// Ignore formatting differences (whitespace, indentation)
    pub ignore_formatting: bool,
    
    /// Ignore reference ID differences if content is same
    pub ignore_reference_ids: bool,
    
    /// Ignore insignificant ordering changes
    pub ignore_order_changes: bool,
    
    /// DDEX version compatibility mode
    pub version_compatibility: VersionCompatibility,
    
    /// Fields to ignore during comparison
    pub ignored_fields: IndexSet<String>,
    
    /// Business-critical fields that should be highlighted
    pub critical_fields: IndexSet<String>,
    
    /// Tolerance for numeric differences (e.g., 0.01 for currency)
    pub numeric_tolerance: Option<f64>,
}

impl Default for DiffConfig {
    fn default() -> Self {
        let mut critical_fields = IndexSet::new();
        critical_fields.insert("CommercialModelType".to_string());
        critical_fields.insert("TerritoryCode".to_string());
        critical_fields.insert("ValidityPeriod".to_string());
        critical_fields.insert("ReleaseDate".to_string());
        critical_fields.insert("UPC".to_string());
        critical_fields.insert("ISRC".to_string());
        critical_fields.insert("Price".to_string());
        
        let mut ignored_fields = IndexSet::new();
        ignored_fields.insert("MessageId".to_string());
        ignored_fields.insert("MessageCreatedDateTime".to_string());
        
        Self {
            ignore_formatting: true,
            ignore_reference_ids: true,
            ignore_order_changes: true,
            version_compatibility: VersionCompatibility::Strict,
            ignored_fields,
            critical_fields,
            numeric_tolerance: Some(0.01),
        }
    }
}

/// Version compatibility modes for DDEX diffing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionCompatibility {
    /// Strict - versions must match exactly
    Strict,
    /// Compatible - allow compatible versions (4.2 <-> 4.3)
    Compatible,
    /// Lenient - ignore version differences entirely
    Lenient,
}

/// Semantic diff engine for DDEX messages
pub struct DiffEngine {
    config: DiffConfig,
    // Cache for reference resolution
    reference_cache: IndexMap<String, Element>,
}

impl DiffEngine {
    /// Create a new diff engine with default configuration
    pub fn new() -> Self {
        Self {
            config: DiffConfig::default(),
            reference_cache: IndexMap::new(),
        }
    }
    
    /// Create a new diff engine with custom configuration
    pub fn new_with_config(config: DiffConfig) -> Self {
        Self {
            config,
            reference_cache: IndexMap::new(),
        }
    }
    
    /// Compare two DDEX ASTs and return a semantic diff
    pub fn diff(&mut self, old: &AST, new: &AST) -> Result<ChangeSet, BuildError> {
        // Clear reference cache for this comparison
        self.reference_cache.clear();
        
        // Build reference maps for both documents
        self.build_reference_cache(&old.root, "old");
        self.build_reference_cache(&new.root, "new");
        
        let mut changeset = ChangeSet::new();
        
        // Compare root elements
        self.compare_elements(&old.root, &new.root, DiffPath::root(), &mut changeset)?;
        
        // Analyze changes for business impact
        self.analyze_business_impact(&mut changeset);
        
        Ok(changeset)
    }
    
    /// Compare two elements semantically
    fn compare_elements(
        &self, 
        old: &Element, 
        new: &Element, 
        path: DiffPath, 
        changeset: &mut ChangeSet
    ) -> Result<(), BuildError> {
        // Check if elements represent the same logical entity
        if old.name != new.name {
            changeset.add_change(SemanticChange {
                path: path.clone(),
                change_type: ChangeType::ElementRenamed,
                old_value: Some(old.name.clone()),
                new_value: Some(new.name.clone()),
                is_critical: self.is_critical_field(&old.name),
                description: format!("Element renamed from '{}' to '{}'", old.name, new.name),
            });
            return Ok(());
        }
        
        // Compare attributes
        self.compare_attributes(&old.attributes, &new.attributes, &path, changeset);
        
        // Compare children with semantic understanding
        self.compare_children(&old.children, &new.children, &path, changeset)?;
        
        Ok(())
    }
    
    /// Compare attributes with semantic understanding
    fn compare_attributes(
        &self,
        old: &IndexMap<String, String>,
        new: &IndexMap<String, String>,
        path: &DiffPath,
        changeset: &mut ChangeSet,
    ) {
        // Find added, removed, and modified attributes
        let old_keys: IndexSet<_> = old.keys().collect();
        let new_keys: IndexSet<_> = new.keys().collect();
        
        // Removed attributes
        for &key in old_keys.difference(&new_keys) {
            if !self.should_ignore_field(key) {
                changeset.add_change(SemanticChange {
                    path: path.with_attribute(key),
                    change_type: ChangeType::AttributeRemoved,
                    old_value: old.get(key).cloned(),
                    new_value: None,
                    is_critical: self.is_critical_field(key),
                    description: format!("Attribute '{}' removed", key),
                });
            }
        }
        
        // Added attributes
        for &key in new_keys.difference(&old_keys) {
            if !self.should_ignore_field(key) {
                changeset.add_change(SemanticChange {
                    path: path.with_attribute(key),
                    change_type: ChangeType::AttributeAdded,
                    old_value: None,
                    new_value: new.get(key).cloned(),
                    is_critical: self.is_critical_field(key),
                    description: format!("Attribute '{}' added", key),
                });
            }
        }
        
        // Modified attributes
        for &key in old_keys.intersection(&new_keys) {
            if !self.should_ignore_field(key) {
                let old_val = &old[key];
                let new_val = &new[key];
                
                if !self.are_values_equivalent(old_val, new_val, key) {
                    changeset.add_change(SemanticChange {
                        path: path.with_attribute(key),
                        change_type: ChangeType::AttributeModified,
                        old_value: Some(old_val.clone()),
                        new_value: Some(new_val.clone()),
                        is_critical: self.is_critical_field(key),
                        description: format!("Attribute '{}' changed from '{}' to '{}'", key, old_val, new_val),
                    });
                }
            }
        }
    }
    
    /// Compare children with semantic understanding
    fn compare_children(
        &self,
        old: &[Node],
        new: &[Node],
        path: &DiffPath,
        changeset: &mut ChangeSet,
    ) -> Result<(), BuildError> {
        // Separate elements from text nodes
        let old_elements: Vec<&Element> = old.iter()
            .filter_map(|n| if let Node::Element(e) = n { Some(e) } else { None })
            .collect();
        let new_elements: Vec<&Element> = new.iter()
            .filter_map(|n| if let Node::Element(e) = n { Some(e) } else { None })
            .collect();
        
        // Compare text content
        let old_text = self.extract_text_content(old);
        let new_text = self.extract_text_content(new);
        if old_text != new_text && !old_text.trim().is_empty() || !new_text.trim().is_empty() {
            changeset.add_change(SemanticChange {
                path: path.with_text(),
                change_type: ChangeType::TextModified,
                old_value: if old_text.trim().is_empty() { None } else { Some(old_text) },
                new_value: if new_text.trim().is_empty() { None } else { Some(new_text) },
                is_critical: false,
                description: "Text content changed".to_string(),
            });
        }
        
        // Group elements by semantic identity for comparison
        let old_groups = self.group_elements_by_identity(&old_elements);
        let new_groups = self.group_elements_by_identity(&new_elements);
        
        // Compare element groups
        self.compare_element_groups(&old_groups, &new_groups, path, changeset)?;
        
        Ok(())
    }
    
    /// Group elements by their semantic identity (name + key attributes)
    fn group_elements_by_identity<'a>(&self, elements: &[&'a Element]) -> IndexMap<String, Vec<&'a Element>> {
        let mut groups = IndexMap::new();
        
        for element in elements {
            let identity = self.get_element_identity(element);
            groups.entry(identity).or_insert_with(Vec::new).push(*element);
        }
        
        groups
    }
    
    /// Get semantic identity key for an element
    fn get_element_identity(&self, element: &Element) -> String {
        // Use element name and key identifying attributes
        let mut identity = element.name.clone();
        
        // Add key attributes that identify this element uniquely
        let key_attrs = match element.name.as_str() {
            "Release" => vec!["ReleaseId", "ReleaseReference"],
            "SoundRecording" | "VideoRecording" => vec!["ResourceId", "ResourceReference"],
            "Deal" => vec!["DealReference"],
            "Party" => vec!["PartyId", "PartyReference"],
            _ => vec!["Id", "Reference"], // Generic fallback
        };
        
        for attr in key_attrs {
            if let Some(value) = element.attributes.get(attr) {
                identity.push_str(&format!(":{}", value));
                break; // Use first found key attribute
            }
        }
        
        identity
    }
    
    /// Compare groups of elements
    fn compare_element_groups(
        &self,
        old_groups: &IndexMap<String, Vec<&Element>>,
        new_groups: &IndexMap<String, Vec<&Element>>,
        path: &DiffPath,
        changeset: &mut ChangeSet,
    ) -> Result<(), BuildError> {
        let old_keys: IndexSet<_> = old_groups.keys().collect();
        let new_keys: IndexSet<_> = new_groups.keys().collect();
        
        // Removed element groups
        for &key in old_keys.difference(&new_keys) {
            for element in &old_groups[key] {
                changeset.add_change(SemanticChange {
                    path: path.with_element(&element.name),
                    change_type: ChangeType::ElementRemoved,
                    old_value: Some(self.element_to_string(element)),
                    new_value: None,
                    is_critical: self.is_critical_field(&element.name),
                    description: format!("Element '{}' removed", element.name),
                });
            }
        }
        
        // Added element groups
        for &key in new_keys.difference(&old_keys) {
            for element in &new_groups[key] {
                changeset.add_change(SemanticChange {
                    path: path.with_element(&element.name),
                    change_type: ChangeType::ElementAdded,
                    old_value: None,
                    new_value: Some(self.element_to_string(element)),
                    is_critical: self.is_critical_field(&element.name),
                    description: format!("Element '{}' added", element.name),
                });
            }
        }
        
        // Compare matching element groups
        for &key in old_keys.intersection(&new_keys) {
            let old_elements = &old_groups[key];
            let new_elements = &new_groups[key];
            
            // For now, compare first element of each group
            // In a more sophisticated implementation, we'd do optimal matching
            if let (Some(&old_elem), Some(&new_elem)) = (old_elements.first(), new_elements.first()) {
                self.compare_elements(old_elem, new_elem, path.with_element(&old_elem.name), changeset)?;
            }
        }
        
        Ok(())
    }
    
    /// Extract text content from nodes, ignoring formatting
    fn extract_text_content(&self, nodes: &[Node]) -> String {
        let mut text = String::new();
        for node in nodes {
            if let Node::Text(t) = node {
                if self.config.ignore_formatting {
                    text.push_str(t.trim());
                } else {
                    text.push_str(t);
                }
            }
        }
        text
    }
    
    /// Check if two values are semantically equivalent
    fn are_values_equivalent(&self, old: &str, new: &str, field_name: &str) -> bool {
        // Reference equivalence - if we're ignoring reference IDs
        if self.config.ignore_reference_ids && self.is_reference_field(field_name) {
            return self.are_references_equivalent(old, new);
        }
        
        // Numeric tolerance for prices and monetary values
        if let Some(tolerance) = self.config.numeric_tolerance {
            if field_name.contains("Price") || field_name.contains("Amount") {
                if let (Ok(old_num), Ok(new_num)) = (old.parse::<f64>(), new.parse::<f64>()) {
                    return (old_num - new_num).abs() < tolerance;
                }
            }
        }
        
        // Formatting equivalence
        if self.config.ignore_formatting {
            return old.trim() == new.trim();
        }
        
        old == new
    }
    
    /// Check if a field represents a reference
    fn is_reference_field(&self, field_name: &str) -> bool {
        field_name.ends_with("Reference") || 
        field_name.ends_with("Ref") ||
        field_name == "ResourceId" ||
        field_name == "ReleaseId" ||
        field_name == "DealId"
    }
    
    /// Check if two references are equivalent by content
    fn are_references_equivalent(&self, old_ref: &str, new_ref: &str) -> bool {
        // If they're the same, they're equivalent
        if old_ref == new_ref {
            return true;
        }
        
        // Look up referenced content in cache
        let old_key = format!("old:{}", old_ref);
        let new_key = format!("new:{}", new_ref);
        
        if let (Some(old_elem), Some(new_elem)) = 
            (self.reference_cache.get(&old_key), self.reference_cache.get(&new_key)) {
            // Compare the referenced elements for semantic equivalence
            self.elements_semantically_equal(old_elem, new_elem)
        } else {
            false
        }
    }
    
    /// Check if two elements are semantically equal
    fn elements_semantically_equal(&self, old: &Element, new: &Element) -> bool {
        // This is a simplified check - in practice, you'd want recursive comparison
        // excluding the reference IDs themselves
        old.name == new.name &&
        self.text_content_equal(&old.children, &new.children)
    }
    
    /// Compare text content of children for equality
    fn text_content_equal(&self, old: &[Node], new: &[Node]) -> bool {
        self.extract_text_content(old) == self.extract_text_content(new)
    }
    
    /// Build reference cache for resolving reference equivalence
    fn build_reference_cache(&mut self, element: &Element, prefix: &str) {
        // Store elements that can be referenced
        if let Some(ref_id) = self.get_reference_id(element) {
            let cache_key = format!("{}:{}", prefix, ref_id);
            self.reference_cache.insert(cache_key, element.clone());
        }
        
        // Recursively build cache for children
        for child in &element.children {
            if let Node::Element(child_elem) = child {
                self.build_reference_cache(child_elem, prefix);
            }
        }
    }
    
    /// Get reference ID from element if it has one
    fn get_reference_id(&self, element: &Element) -> Option<String> {
        // Look for common reference attributes
        let ref_attrs = ["ResourceReference", "ReleaseReference", "DealReference", 
                        "PartyReference", "Reference", "ResourceId", "ReleaseId"];
        
        for attr in &ref_attrs {
            if let Some(value) = element.attributes.get(*attr) {
                return Some(value.clone());
            }
        }
        
        None
    }
    
    /// Check if a field should be ignored during comparison
    fn should_ignore_field(&self, field_name: &str) -> bool {
        self.config.ignored_fields.contains(field_name)
    }
    
    /// Check if a field is business-critical
    fn is_critical_field(&self, field_name: &str) -> bool {
        self.config.critical_fields.contains(field_name)
    }
    
    /// Convert element to string representation
    fn element_to_string(&self, element: &Element) -> String {
        // Simplified string representation - in practice you'd want proper XML serialization
        format!("<{}>", element.name)
    }
    
    /// Analyze changes for business impact
    fn analyze_business_impact(&self, changeset: &mut ChangeSet) {
        // Count critical changes
        let critical_changes = changeset.changes.iter()
            .filter(|c| c.is_critical)
            .count();
        
        changeset.metadata.insert("critical_changes".to_string(), critical_changes.to_string());
        
        // Determine overall impact level
        let impact = if critical_changes > 0 {
            "HIGH"
        } else if changeset.changes.len() > 10 {
            "MEDIUM"
        } else {
            "LOW"
        };
        
        changeset.metadata.insert("impact_level".to_string(), impact.to_string());
    }
}

impl Default for DiffEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Element, Node};
    
    fn create_test_element(name: &str, text: &str) -> Element {
        Element::new(name).with_text(text)
    }
    
    #[test]
    fn test_basic_diff() {
        let mut engine = DiffEngine::new();
        
        let old_ast = AST {
            root: create_test_element("Root", "old content"),
            namespaces: IndexMap::new(),
            schema_location: None,
        };
        
        let new_ast = AST {
            root: create_test_element("Root", "new content"),
            namespaces: IndexMap::new(),
            schema_location: None,
        };
        
        let changeset = engine.diff(&old_ast, &new_ast).unwrap();
        assert!(!changeset.changes.is_empty());
    }
    
    #[test]
    fn test_ignore_formatting() {
        let mut engine = DiffEngine::new();
        
        let old_ast = AST {
            root: create_test_element("Root", "  content  "),
            namespaces: IndexMap::new(),
            schema_location: None,
        };
        
        let new_ast = AST {
            root: create_test_element("Root", "content"),
            namespaces: IndexMap::new(),
            schema_location: None,
        };
        
        let changeset = engine.diff(&old_ast, &new_ast).unwrap();
        // Should have no changes due to formatting being ignored
        let text_changes: Vec<_> = changeset.changes.iter()
            .filter(|c| matches!(c.change_type, ChangeType::TextModified))
            .collect();
        assert!(text_changes.is_empty());
    }
}