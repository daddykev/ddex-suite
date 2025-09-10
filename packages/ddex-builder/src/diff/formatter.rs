//! Formatters for DDEX diff output in various formats

use super::types::{ChangeSet, SemanticChange, ChangeType};
use crate::error::BuildError;
use serde_json::json;
use indexmap::IndexMap;
use std::fmt::Write;

/// Formatter for diff output in various formats
pub struct DiffFormatter;

impl DiffFormatter {
    /// Format changeset as human-readable summary
    pub fn format_summary(changeset: &ChangeSet) -> String {
        let mut output = String::new();
        
        // Header
        writeln!(output, "DDEX Semantic Diff Summary").unwrap();
        writeln!(output, "==========================").unwrap();
        writeln!(output, "Timestamp: {}", changeset.timestamp.format("%Y-%m-%d %H:%M:%S UTC")).unwrap();
        writeln!(output, "Impact Level: {}", changeset.impact_level()).unwrap();
        writeln!(output, "Changes: {}", changeset.summary.summary_string()).unwrap();
        writeln!(output).unwrap();
        
        if !changeset.has_changes() {
            writeln!(output, "✅ No semantic changes detected").unwrap();
            return output;
        }
        
        // Critical changes first
        let critical_changes = changeset.critical_changes();
        if !critical_changes.is_empty() {
            writeln!(output, "🚨 Critical Changes ({}):", critical_changes.len()).unwrap();
            for change in critical_changes {
                writeln!(output, "  {} {}", 
                    Self::change_type_icon(change.change_type), 
                    change.description).unwrap();
                writeln!(output, "    Path: {}", change.path).unwrap();
                if let (Some(old), Some(new)) = (&change.old_value, &change.new_value) {
                    writeln!(output, "    Change: '{}' → '{}'", old, new).unwrap();
                }
                writeln!(output).unwrap();
            }
        }
        
        // Group remaining changes by type
        let mut changes_by_type: IndexMap<ChangeType, Vec<&SemanticChange>> = IndexMap::new();
        for change in &changeset.changes {
            if !change.is_critical {
                changes_by_type.entry(change.change_type).or_default().push(change);
            }
        }
        
        for (change_type, changes) in changes_by_type {
            if !changes.is_empty() {
                writeln!(output, "{} {} ({}):", 
                    Self::change_type_icon(change_type), 
                    change_type, 
                    changes.len()).unwrap();
                
                for change in changes {
                    writeln!(output, "  • {} ({})", change.description, change.path).unwrap();
                }
                writeln!(output).unwrap();
            }
        }
        
        // Metadata
        if !changeset.metadata.is_empty() {
            writeln!(output, "Metadata:").unwrap();
            for (key, value) in &changeset.metadata {
                writeln!(output, "  {}: {}", key, value).unwrap();
            }
        }
        
        output
    }
    
    /// Format changeset as detailed report
    pub fn format_detailed(changeset: &ChangeSet) -> String {
        let mut output = String::new();
        
        writeln!(output, "DDEX Semantic Diff - Detailed Report").unwrap();
        writeln!(output, "====================================").unwrap();
        writeln!(output, "Generated: {}", changeset.timestamp.format("%Y-%m-%d %H:%M:%S UTC")).unwrap();
        writeln!(output).unwrap();
        
        // Statistics
        writeln!(output, "Statistics:").unwrap();
        writeln!(output, "  Total Changes: {}", changeset.summary.total_changes).unwrap();
        writeln!(output, "  Additions: {}", changeset.summary.additions).unwrap();
        writeln!(output, "  Deletions: {}", changeset.summary.deletions).unwrap();
        writeln!(output, "  Modifications: {}", changeset.summary.modifications).unwrap();
        writeln!(output, "  Moves: {}", changeset.summary.moves).unwrap();
        writeln!(output, "  Critical: {}", changeset.summary.critical_changes).unwrap();
        writeln!(output, "  Impact: {}", changeset.impact_level()).unwrap();
        writeln!(output).unwrap();
        
        if !changeset.has_changes() {
            writeln!(output, "No changes detected.").unwrap();
            return output;
        }
        
        // Detailed change list
        writeln!(output, "Detailed Changes:").unwrap();
        writeln!(output, "-----------------").unwrap();
        
        for (i, change) in changeset.changes.iter().enumerate() {
            writeln!(output, "{}. {} {}", 
                i + 1, 
                Self::change_type_icon(change.change_type), 
                change.description).unwrap();
            writeln!(output, "   Type: {}", change.change_type).unwrap();
            writeln!(output, "   Path: {}", change.path).unwrap();
            writeln!(output, "   Critical: {}", if change.is_critical { "Yes" } else { "No" }).unwrap();
            
            match (&change.old_value, &change.new_value) {
                (Some(old), Some(new)) => {
                    writeln!(output, "   Old Value: {}", Self::truncate_value(old)).unwrap();
                    writeln!(output, "   New Value: {}", Self::truncate_value(new)).unwrap();
                },
                (Some(old), None) => {
                    writeln!(output, "   Removed Value: {}", Self::truncate_value(old)).unwrap();
                },
                (None, Some(new)) => {
                    writeln!(output, "   Added Value: {}", Self::truncate_value(new)).unwrap();
                },
                (None, None) => {},
            }
            writeln!(output).unwrap();
        }
        
        output
    }
    
    /// Format changeset as JSON Patch (RFC 6902)
    pub fn format_json_patch(changeset: &ChangeSet) -> Result<String, BuildError> {
        let mut patches = Vec::new();
        
        for change in &changeset.changes {
            let path = Self::path_to_json_pointer(&change.path);
            
            let patch = match change.change_type {
                ChangeType::ElementAdded | ChangeType::AttributeAdded => {
                    json!({
                        "op": "add",
                        "path": path,
                        "value": change.new_value.clone().unwrap_or_default()
                    })
                },
                ChangeType::ElementRemoved | ChangeType::AttributeRemoved => {
                    json!({
                        "op": "remove", 
                        "path": path
                    })
                },
                ChangeType::ElementModified | ChangeType::AttributeModified | 
                ChangeType::TextModified => {
                    json!({
                        "op": "replace",
                        "path": path,
                        "value": change.new_value.clone().unwrap_or_default()
                    })
                },
                ChangeType::ElementMoved => {
                    // JSON Patch move operation would require more complex path resolution
                    json!({
                        "op": "move",
                        "from": path,
                        "path": path // Simplified - would need actual destination
                    })
                },
                ChangeType::ElementRenamed => {
                    // Handle as remove + add for JSON Patch
                    json!({
                        "op": "replace",
                        "path": path,
                        "value": change.new_value.clone().unwrap_or_default()
                    })
                },
            };
            
            patches.push(patch);
        }
        
        serde_json::to_string_pretty(&patches)
            .map_err(|e| BuildError::Serialization(e.to_string()))
    }
    
    /// Format changeset as JSON for programmatic use
    pub fn format_json(changeset: &ChangeSet) -> Result<String, BuildError> {
        let json = json!({
            "timestamp": changeset.timestamp.to_rfc3339(),
            "summary": {
                "total_changes": changeset.summary.total_changes,
                "additions": changeset.summary.additions,
                "deletions": changeset.summary.deletions,
                "modifications": changeset.summary.modifications,
                "moves": changeset.summary.moves,
                "critical_changes": changeset.summary.critical_changes,
                "impact_level": changeset.impact_level().to_string(),
                "has_changes": changeset.has_changes()
            },
            "changes": changeset.changes.iter().map(|change| json!({
                "path": change.path.to_string(),
                "type": change.change_type.to_string(),
                "critical": change.is_critical,
                "description": change.description,
                "old_value": change.old_value,
                "new_value": change.new_value
            })).collect::<Vec<_>>(),
            "metadata": changeset.metadata
        });
        
        serde_json::to_string_pretty(&json)
            .map_err(|e| BuildError::Serialization(e.to_string()))
    }
    
    /// Format changeset as HTML report
    pub fn format_html(changeset: &ChangeSet) -> String {
        let mut html = String::new();
        
        // HTML header
        html.push_str(r#"<!DOCTYPE html>
<html>
<head>
    <title>DDEX Semantic Diff Report</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; }
        .header { border-bottom: 2px solid #ddd; padding-bottom: 20px; margin-bottom: 30px; }
        .summary { background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 30px; }
        .change-group { margin-bottom: 30px; }
        .change-type { font-weight: bold; font-size: 1.2em; margin-bottom: 15px; }
        .change-item { background: white; border: 1px solid #ddd; border-radius: 4px; padding: 15px; margin-bottom: 10px; }
        .critical { border-left: 4px solid #dc3545; }
        .added { border-left: 4px solid #28a745; }
        .removed { border-left: 4px solid #dc3545; }
        .modified { border-left: 4px solid #ffc107; }
        .path { font-family: monospace; background: #f1f1f1; padding: 2px 6px; border-radius: 3px; }
        .value { font-family: monospace; background: #f8f8f8; padding: 8px; border-radius: 3px; margin: 5px 0; }
        .old-value { background-color: #ffebee; }
        .new-value { background-color: #e8f5e8; }
        .impact-high { color: #dc3545; }
        .impact-medium { color: #ffc107; }
        .impact-low { color: #28a745; }
        .impact-none { color: #6c757d; }
    </style>
</head>
<body>
"#);
        
        // Header
        html.push_str(&format!(r#"
    <div class="header">
        <h1>DDEX Semantic Diff Report</h1>
        <p>Generated: {}</p>
        <p>Impact Level: <span class="impact-{}">{}</span></p>
        <p>Summary: {}</p>
    </div>
"#, 
            changeset.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            changeset.impact_level().to_string().to_lowercase(),
            changeset.impact_level(),
            changeset.summary.summary_string()
        ));
        
        if !changeset.has_changes() {
            html.push_str("<div class='summary'><h2>✅ No Changes</h2><p>No semantic changes detected between the documents.</p></div>");
        } else {
            // Summary statistics
            html.push_str(&format!(r#"
    <div class="summary">
        <h2>Summary Statistics</h2>
        <ul>
            <li>Total Changes: {}</li>
            <li>Additions: {}</li>
            <li>Deletions: {}</li>
            <li>Modifications: {}</li>
            <li>Moves: {}</li>
            <li>Critical Changes: {}</li>
        </ul>
    </div>
"#,
                changeset.summary.total_changes,
                changeset.summary.additions,
                changeset.summary.deletions,
                changeset.summary.modifications,
                changeset.summary.moves,
                changeset.summary.critical_changes
            ));
            
            // Critical changes first
            let critical_changes = changeset.critical_changes();
            if !critical_changes.is_empty() {
                html.push_str("<div class='change-group'>");
                html.push_str("<div class='change-type'>🚨 Critical Changes</div>");
                
                for change in critical_changes {
                    html.push_str(&Self::format_change_html(change, "critical"));
                }
                
                html.push_str("</div>");
            }
            
            // Group other changes by type
            let mut changes_by_type: IndexMap<ChangeType, Vec<&SemanticChange>> = IndexMap::new();
            for change in &changeset.changes {
                if !change.is_critical {
                    changes_by_type.entry(change.change_type).or_default().push(change);
                }
            }
            
            for (change_type, changes) in changes_by_type {
                if !changes.is_empty() {
                    html.push_str("<div class='change-group'>");
                    html.push_str(&format!("<div class='change-type'>{} {} ({})</div>", 
                        Self::change_type_icon(change_type), change_type, changes.len()));
                    
                    let css_class = match change_type {
                        ChangeType::ElementAdded | ChangeType::AttributeAdded => "added",
                        ChangeType::ElementRemoved | ChangeType::AttributeRemoved => "removed", 
                        _ => "modified",
                    };
                    
                    for change in changes {
                        html.push_str(&Self::format_change_html(change, css_class));
                    }
                    
                    html.push_str("</div>");
                }
            }
        }
        
        // HTML footer
        html.push_str("</body></html>");
        
        html
    }
    
    /// Generate DDEX UpdateReleaseMessage from changeset
    pub fn generate_update_message(changeset: &ChangeSet, message_id: Option<&str>) -> Result<String, BuildError> {
        let mut xml = String::new();
        
        // XML declaration and root element
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<UpdateReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">"#);
        xml.push('\n');
        
        // Message header
        xml.push_str("  <MessageHeader>\n");
        xml.push_str(&format!("    <MessageId>{}</MessageId>\n", 
            message_id.unwrap_or(&uuid::Uuid::new_v4().to_string())));
        xml.push_str("    <MessageSender>\n");
        xml.push_str("      <PartyName>DDEX Suite Diff Engine</PartyName>\n");
        xml.push_str("    </MessageSender>\n");
        xml.push_str("    <MessageRecipient>\n");
        xml.push_str("      <PartyName>Recipient</PartyName>\n");
        xml.push_str("    </MessageRecipient>\n");
        xml.push_str(&format!("    <MessageCreatedDateTime>{}</MessageCreatedDateTime>\n", 
            changeset.timestamp.to_rfc3339()));
        xml.push_str("  </MessageHeader>\n");
        
        // Update list (simplified - real implementation would group by entity)
        xml.push_str("  <UpdateList>\n");
        
        for change in &changeset.changes {
            xml.push_str("    <Update>\n");
            xml.push_str(&format!("      <UpdateType>{}</UpdateType>\n", 
                Self::change_type_to_update_type(change.change_type)));
            xml.push_str(&format!("      <UpdatePath>{}</UpdatePath>\n", 
                html_escape::encode_text(&change.path.to_string())));
            
            if let Some(old_val) = &change.old_value {
                xml.push_str(&format!("      <OldValue>{}</OldValue>\n", 
                    html_escape::encode_text(old_val)));
            }
            if let Some(new_val) = &change.new_value {
                xml.push_str(&format!("      <NewValue>{}</NewValue>\n", 
                    html_escape::encode_text(new_val)));
            }
            
            xml.push_str(&format!("      <IsCritical>{}</IsCritical>\n", change.is_critical));
            xml.push_str("    </Update>\n");
        }
        
        xml.push_str("  </UpdateList>\n");
        xml.push_str("</UpdateReleaseMessage>\n");
        
        Ok(xml)
    }
    
    // Helper methods
    
    fn change_type_icon(change_type: ChangeType) -> &'static str {
        match change_type {
            ChangeType::ElementAdded | ChangeType::AttributeAdded => "➕",
            ChangeType::ElementRemoved | ChangeType::AttributeRemoved => "➖",
            ChangeType::ElementModified | ChangeType::AttributeModified => "✏️",
            ChangeType::TextModified => "📝",
            ChangeType::ElementRenamed => "🔄",
            ChangeType::ElementMoved => "🔄",
        }
    }
    
    fn truncate_value(value: &str) -> String {
        if value.len() > 100 {
            format!("{}...", &value[..97])
        } else {
            value.to_string()
        }
    }
    
    fn path_to_json_pointer(path: &super::types::DiffPath) -> String {
        let mut pointer = String::new();
        for segment in &path.segments {
            pointer.push('/');
            match segment {
                super::types::PathSegment::Element(name) => pointer.push_str(name),
                super::types::PathSegment::Attribute(name) => {
                    pointer.push('@');
                    pointer.push_str(name);
                },
                super::types::PathSegment::Text => pointer.push_str("text()"),
                super::types::PathSegment::Index(idx) => pointer.push_str(&idx.to_string()),
            }
        }
        if pointer.is_empty() {
            "/".to_string()
        } else {
            pointer
        }
    }
    
    fn format_change_html(change: &SemanticChange, css_class: &str) -> String {
        let mut html = format!("<div class='change-item {}'>\n", css_class);
        html.push_str(&format!("  <div><strong>{}</strong></div>\n", 
            html_escape::encode_text(&change.description)));
        html.push_str(&format!("  <div>Path: <span class='path'>{}</span></div>\n", 
            html_escape::encode_text(&change.path.to_string())));
        
        match (&change.old_value, &change.new_value) {
            (Some(old), Some(new)) => {
                html.push_str(&format!("  <div class='value old-value'>Old: {}</div>\n", 
                    html_escape::encode_text(&Self::truncate_value(old))));
                html.push_str(&format!("  <div class='value new-value'>New: {}</div>\n", 
                    html_escape::encode_text(&Self::truncate_value(new))));
            },
            (Some(old), None) => {
                html.push_str(&format!("  <div class='value old-value'>Removed: {}</div>\n", 
                    html_escape::encode_text(&Self::truncate_value(old))));
            },
            (None, Some(new)) => {
                html.push_str(&format!("  <div class='value new-value'>Added: {}</div>\n", 
                    html_escape::encode_text(&Self::truncate_value(new))));
            },
            (None, None) => {},
        }
        
        html.push_str("</div>\n");
        html
    }
    
    fn change_type_to_update_type(change_type: ChangeType) -> &'static str {
        match change_type {
            ChangeType::ElementAdded | ChangeType::AttributeAdded => "Add",
            ChangeType::ElementRemoved | ChangeType::AttributeRemoved => "Remove",
            ChangeType::ElementModified | ChangeType::AttributeModified | 
            ChangeType::TextModified => "Modify",
            ChangeType::ElementRenamed => "Rename",
            ChangeType::ElementMoved => "Move",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::types::{SemanticChange, DiffPath, ChangeType};
    
    fn create_test_changeset() -> ChangeSet {
        let mut changeset = ChangeSet::new();
        
        changeset.add_change(SemanticChange {
            path: DiffPath::root().with_element("Release").with_attribute("UPC"),
            change_type: ChangeType::AttributeModified,
            old_value: Some("123456789".to_string()),
            new_value: Some("987654321".to_string()),
            is_critical: true,
            description: "UPC changed".to_string(),
        });
        
        changeset.add_change(SemanticChange {
            path: DiffPath::root().with_element("Release").with_element("Title"),
            change_type: ChangeType::TextModified,
            old_value: Some("Old Title".to_string()),
            new_value: Some("New Title".to_string()),
            is_critical: false,
            description: "Title changed".to_string(),
        });
        
        changeset
    }
    
    #[test]
    fn test_format_summary() {
        let changeset = create_test_changeset();
        let summary = DiffFormatter::format_summary(&changeset);
        
        assert!(summary.contains("DDEX Semantic Diff Summary"));
        assert!(summary.contains("Critical Changes"));
        assert!(summary.contains("UPC changed"));
    }
    
    #[test]
    fn test_format_json() {
        let changeset = create_test_changeset();
        let json_result = DiffFormatter::format_json(&changeset);
        
        assert!(json_result.is_ok());
        let json_str = json_result.unwrap();
        assert!(json_str.contains("total_changes"));
        assert!(json_str.contains("critical_changes"));
    }
    
    #[test]
    fn test_format_json_patch() {
        let changeset = create_test_changeset();
        let patch_result = DiffFormatter::format_json_patch(&changeset);
        
        assert!(patch_result.is_ok());
        let patch_str = patch_result.unwrap();
        assert!(patch_str.contains("\"op\":"));
        assert!(patch_str.contains("\"path\":"));
    }
    
    #[test]
    fn test_format_html() {
        let changeset = create_test_changeset();
        let html = DiffFormatter::format_html(&changeset);
        
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("DDEX Semantic Diff Report"));
        assert!(html.contains("Critical Changes"));
    }
}