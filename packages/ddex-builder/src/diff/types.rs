//! Type definitions for DDEX semantic diffing

use serde::{Serialize, Deserialize};
use indexmap::IndexMap;
use std::fmt;

/// Result of a semantic diff operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffResult {
    /// No changes detected
    Unchanged,
    /// Content was added
    Added,
    /// Content was modified  
    Modified,
    /// Content was removed
    Removed,
    /// Content was moved to a different location
    Moved { from: DiffPath, to: DiffPath },
}

/// Complete set of changes between two DDEX documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    /// Individual semantic changes
    pub changes: Vec<SemanticChange>,
    
    /// Summary statistics
    pub summary: ChangeSummary,
    
    /// Additional metadata about the diff
    pub metadata: IndexMap<String, String>,
    
    /// Timestamp when diff was performed
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ChangeSet {
    /// Create a new empty changeset
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            summary: ChangeSummary::default(),
            metadata: IndexMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Add a semantic change to the changeset
    pub fn add_change(&mut self, change: SemanticChange) {
        // Update summary statistics
        match change.change_type {
            ChangeType::ElementAdded | ChangeType::AttributeAdded => {
                self.summary.additions += 1;
            },
            ChangeType::ElementRemoved | ChangeType::AttributeRemoved => {
                self.summary.deletions += 1;
            },
            ChangeType::ElementModified | ChangeType::AttributeModified | 
            ChangeType::TextModified | ChangeType::ElementRenamed => {
                self.summary.modifications += 1;
            },
            ChangeType::ElementMoved => {
                self.summary.moves += 1;
            },
        }
        
        if change.is_critical {
            self.summary.critical_changes += 1;
        }
        
        self.changes.push(change);
        self.summary.total_changes = self.changes.len();
    }
    
    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }
    
    /// Get changes by criticality
    pub fn critical_changes(&self) -> Vec<&SemanticChange> {
        self.changes.iter().filter(|c| c.is_critical).collect()
    }
    
    /// Get changes by type
    pub fn changes_by_type(&self, change_type: ChangeType) -> Vec<&SemanticChange> {
        self.changes.iter().filter(|c| c.change_type == change_type).collect()
    }
    
    /// Get overall impact level
    pub fn impact_level(&self) -> ImpactLevel {
        if self.summary.critical_changes > 0 {
            ImpactLevel::High
        } else if self.summary.total_changes > 10 {
            ImpactLevel::Medium
        } else if self.summary.total_changes > 0 {
            ImpactLevel::Low
        } else {
            ImpactLevel::None
        }
    }
}

impl Default for ChangeSet {
    fn default() -> Self {
        Self::new()
    }
}

/// A single semantic change in a DDEX document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticChange {
    /// Path to the changed element/attribute
    pub path: DiffPath,
    
    /// Type of change
    pub change_type: ChangeType,
    
    /// Previous value (if any)
    pub old_value: Option<String>,
    
    /// New value (if any)
    pub new_value: Option<String>,
    
    /// Whether this change is business-critical
    pub is_critical: bool,
    
    /// Human-readable description of the change
    pub description: String,
}

/// Path to a specific location in a DDEX document
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiffPath {
    /// Path segments (element names, attribute names)
    pub segments: Vec<PathSegment>,
}

impl DiffPath {
    /// Create root path
    pub fn root() -> Self {
        Self { segments: Vec::new() }
    }
    
    /// Create path with a single element
    pub fn element(name: &str) -> Self {
        Self {
            segments: vec![PathSegment::Element(name.to_string())]
        }
    }
    
    /// Add an element to the path
    pub fn with_element(&self, name: &str) -> Self {
        let mut segments = self.segments.clone();
        segments.push(PathSegment::Element(name.to_string()));
        Self { segments }
    }
    
    /// Add an attribute to the path
    pub fn with_attribute(&self, name: &str) -> Self {
        let mut segments = self.segments.clone();
        segments.push(PathSegment::Attribute(name.to_string()));
        Self { segments }
    }
    
    /// Add text content to the path
    pub fn with_text(&self) -> Self {
        let mut segments = self.segments.clone();
        segments.push(PathSegment::Text);
        Self { segments }
    }
    
    /// Add an index to the path for array elements
    pub fn with_index(&self, index: usize) -> Self {
        let mut segments = self.segments.clone();
        segments.push(PathSegment::Index(index));
        Self { segments }
    }
    
    /// Get the path as a slash-separated string
    pub fn to_string(&self) -> String {
        if self.segments.is_empty() {
            return "/".to_string();
        }
        
        let mut path = String::new();
        for segment in &self.segments {
            path.push('/');
            match segment {
                PathSegment::Element(name) => path.push_str(name),
                PathSegment::Attribute(name) => {
                    path.push('@');
                    path.push_str(name);
                },
                PathSegment::Text => path.push_str("#text"),
                PathSegment::Index(idx) => path.push_str(&format!("[{}]", idx)),
            }
        }
        path
    }
}

impl fmt::Display for DiffPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Individual segment of a diff path
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PathSegment {
    /// XML element name
    Element(String),
    /// XML attribute name
    Attribute(String),
    /// Text content
    Text,
    /// Array index for repeated elements
    Index(usize),
}

/// Type of semantic change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeType {
    /// Element was added
    ElementAdded,
    /// Element was removed
    ElementRemoved,
    /// Element was modified
    ElementModified,
    /// Element was renamed
    ElementRenamed,
    /// Element was moved
    ElementMoved,
    /// Attribute was added
    AttributeAdded,
    /// Attribute was removed
    AttributeRemoved,
    /// Attribute was modified
    AttributeModified,
    /// Text content was modified
    TextModified,
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ChangeType::ElementAdded => "Element Added",
            ChangeType::ElementRemoved => "Element Removed", 
            ChangeType::ElementModified => "Element Modified",
            ChangeType::ElementRenamed => "Element Renamed",
            ChangeType::ElementMoved => "Element Moved",
            ChangeType::AttributeAdded => "Attribute Added",
            ChangeType::AttributeRemoved => "Attribute Removed",
            ChangeType::AttributeModified => "Attribute Modified",
            ChangeType::TextModified => "Text Modified",
        };
        write!(f, "{}", s)
    }
}

/// Summary of changes in a changeset
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChangeSummary {
    /// Total number of changes
    pub total_changes: usize,
    /// Number of additions
    pub additions: usize,
    /// Number of deletions
    pub deletions: usize,
    /// Number of modifications
    pub modifications: usize,
    /// Number of moves
    pub moves: usize,
    /// Number of critical changes
    pub critical_changes: usize,
}

impl ChangeSummary {
    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        self.total_changes > 0
    }
    
    /// Get a brief summary string
    pub fn summary_string(&self) -> String {
        if !self.has_changes() {
            return "No changes".to_string();
        }
        
        let mut parts = Vec::new();
        
        if self.additions > 0 {
            parts.push(format!("{} added", self.additions));
        }
        if self.deletions > 0 {
            parts.push(format!("{} deleted", self.deletions));
        }
        if self.modifications > 0 {
            parts.push(format!("{} modified", self.modifications));
        }
        if self.moves > 0 {
            parts.push(format!("{} moved", self.moves));
        }
        
        let summary = parts.join(", ");
        
        if self.critical_changes > 0 {
            format!("{} ({} critical)", summary, self.critical_changes)
        } else {
            summary
        }
    }
}

/// Impact level of changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactLevel {
    /// No changes
    None,
    /// Low impact changes (formatting, minor additions)
    Low,
    /// Medium impact changes (significant additions/modifications)
    Medium,
    /// High impact changes (critical business fields affected)
    High,
}

impl fmt::Display for ImpactLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ImpactLevel::None => "None",
            ImpactLevel::Low => "Low",
            ImpactLevel::Medium => "Medium",
            ImpactLevel::High => "High",
        };
        write!(f, "{}", s)
    }
}

/// Context information for understanding a change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeContext {
    /// Related DDEX entity (Release, Resource, Deal, etc.)
    pub entity_type: Option<String>,
    
    /// Entity identifier if available
    pub entity_id: Option<String>,
    
    /// Business context (pricing, territory, rights, etc.)
    pub business_context: Option<String>,
    
    /// Technical context (schema version, etc.)
    pub technical_context: IndexMap<String, String>,
}

/// Configuration for what constitutes a significant change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSignificance {
    /// Fields that are considered critical
    pub critical_fields: Vec<String>,
    
    /// Fields that should be ignored
    pub ignored_fields: Vec<String>,
    
    /// Numeric tolerance for comparisons
    pub numeric_tolerance: f64,
    
    /// Whether to ignore order changes
    pub ignore_order: bool,
}

impl Default for ChangeSignificance {
    fn default() -> Self {
        Self {
            critical_fields: vec![
                "CommercialModelType".to_string(),
                "TerritoryCode".to_string(),
                "Price".to_string(),
                "ValidityPeriod".to_string(),
                "ReleaseDate".to_string(),
                "UPC".to_string(),
                "ISRC".to_string(),
            ],
            ignored_fields: vec![
                "MessageId".to_string(),
                "MessageCreatedDateTime".to_string(),
            ],
            numeric_tolerance: 0.01,
            ignore_order: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_diff_path() {
        let path = DiffPath::root()
            .with_element("Release")
            .with_attribute("ReleaseId");
        
        assert_eq!(path.to_string(), "/Release/@ReleaseId");
    }
    
    #[test]
    fn test_changeset() {
        let mut changeset = ChangeSet::new();
        
        changeset.add_change(SemanticChange {
            path: DiffPath::element("Test"),
            change_type: ChangeType::ElementAdded,
            old_value: None,
            new_value: Some("new".to_string()),
            is_critical: true,
            description: "Test change".to_string(),
        });
        
        assert!(changeset.has_changes());
        assert_eq!(changeset.summary.total_changes, 1);
        assert_eq!(changeset.summary.critical_changes, 1);
        assert_eq!(changeset.impact_level(), ImpactLevel::High);
    }
    
    #[test]
    fn test_change_summary() {
        let mut summary = ChangeSummary::default();
        assert!(!summary.has_changes());
        assert_eq!(summary.summary_string(), "No changes");
        
        summary.additions = 2;
        summary.modifications = 1;
        summary.critical_changes = 1;
        summary.total_changes = 3;
        
        assert!(summary.has_changes());
        let summary_str = summary.summary_string();
        assert!(summary_str.contains("2 added"));
        assert!(summary_str.contains("1 modified"));
        assert!(summary_str.contains("1 critical"));
    }
}