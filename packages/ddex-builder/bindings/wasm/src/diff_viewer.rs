//! Interactive HTML diff viewer for WASM

use wasm_bindgen::prelude::*;
use ddex_builder::diff::{DiffEngine, DiffConfig};
use ddex_builder::diff::formatter::DiffFormatter;
use ddex_builder::ast::{AST, Element};
use serde::{Serialize, Deserialize};

#[wasm_bindgen]
pub struct DdexDiffViewer {
    engine: DiffEngine,
}

#[wasm_bindgen]
impl DdexDiffViewer {
    /// Create a new diff viewer
    #[wasm_bindgen(constructor)]
    pub fn new() -> DdexDiffViewer {
        console_error_panic_hook::set_once();
        
        DdexDiffViewer {
            engine: DiffEngine::new(),
        }
    }
    
    /// Create a new diff viewer with custom configuration
    #[wasm_bindgen]
    pub fn with_config(config_json: &str) -> Result<DdexDiffViewer, JsError> {
        console_error_panic_hook::set_once();
        
        let config: DiffConfig = serde_json::from_str(config_json)
            .map_err(|e| JsError::new(&format!("Invalid config JSON: {}", e)))?;
        
        Ok(DdexDiffViewer {
            engine: DiffEngine::new_with_config(config),
        })
    }
    
    /// Compare two DDEX XML strings and return HTML diff viewer
    #[wasm_bindgen]
    pub fn diff_to_html(&mut self, old_xml: &str, new_xml: &str) -> Result<String, JsError> {
        // Parse XML to AST (simplified for WASM demo)
        let old_ast = self.parse_xml_simple(old_xml)?;
        let new_ast = self.parse_xml_simple(new_xml)?;
        
        // Perform diff
        let changeset = self.engine.diff(&old_ast, &new_ast)
            .map_err(|e| JsError::new(&format!("Diff error: {}", e)))?;
        
        // Generate interactive HTML
        Ok(self.generate_interactive_html(&changeset, old_xml, new_xml))
    }
    
    /// Compare two DDEX XML strings and return JSON diff
    #[wasm_bindgen]
    pub fn diff_to_json(&mut self, old_xml: &str, new_xml: &str) -> Result<String, JsError> {
        let old_ast = self.parse_xml_simple(old_xml)?;
        let new_ast = self.parse_xml_simple(new_xml)?;
        
        let changeset = self.engine.diff(&old_ast, &new_ast)
            .map_err(|e| JsError::new(&format!("Diff error: {}", e)))?;
        
        DiffFormatter::format_json(&changeset)
            .map_err(|e| JsError::new(&format!("JSON formatting error: {}", e)))
    }
    
    /// Get diff summary as text
    #[wasm_bindgen]
    pub fn diff_to_summary(&mut self, old_xml: &str, new_xml: &str) -> Result<String, JsError> {
        let old_ast = self.parse_xml_simple(old_xml)?;
        let new_ast = self.parse_xml_simple(new_xml)?;
        
        let changeset = self.engine.diff(&old_ast, &new_ast)
            .map_err(|e| JsError::new(&format!("Diff error: {}", e)))?;
        
        Ok(DiffFormatter::format_summary(&changeset))
    }
    
    /// Generate JSON Patch from diff
    #[wasm_bindgen]
    pub fn diff_to_json_patch(&mut self, old_xml: &str, new_xml: &str) -> Result<String, JsError> {
        let old_ast = self.parse_xml_simple(old_xml)?;
        let new_ast = self.parse_xml_simple(new_xml)?;
        
        let changeset = self.engine.diff(&old_ast, &new_ast)
            .map_err(|e| JsError::new(&format!("Diff error: {}", e)))?;
        
        DiffFormatter::format_json_patch(&changeset)
            .map_err(|e| JsError::new(&format!("JSON Patch formatting error: {}", e)))
    }
    
    // Private helper methods
    
    fn parse_xml_simple(&self, xml: &str) -> Result<AST, JsError> {
        // Simplified XML parsing for WASM demo
        // In production, you'd want proper XML parsing
        let root = Element::new("Root").with_text(xml);
        
        Ok(AST {
            root,
            namespaces: indexmap::IndexMap::new(),
            schema_location: None,
        })
    }
    
    fn generate_interactive_html(&self, changeset: &ddex_builder::diff::types::ChangeSet, old_xml: &str, new_xml: &str) -> String {
        let mut html = String::new();
        
        // Enhanced HTML with JavaScript interactivity
        html.push_str(r#"<!DOCTYPE html>
<html>
<head>
    <title>DDEX Interactive Diff Viewer</title>
    <style>
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; 
            margin: 0; padding: 20px; background: #f5f5f5; 
        }
        .header { 
            background: white; padding: 20px; border-radius: 8px; 
            margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .stats { display: flex; gap: 20px; margin: 15px 0; }
        .stat { background: #f8f9fa; padding: 10px; border-radius: 4px; }
        .stat-value { font-size: 1.5em; font-weight: bold; }
        .controls { margin-bottom: 20px; }
        .btn { 
            background: #007bff; color: white; border: none; 
            padding: 8px 16px; border-radius: 4px; margin-right: 10px; 
            cursor: pointer; 
        }
        .btn:hover { background: #0056b3; }
        .btn.active { background: #28a745; }
        .diff-container { 
            display: flex; gap: 20px; margin-bottom: 20px; 
        }
        .diff-panel { 
            flex: 1; background: white; border-radius: 8px; 
            box-shadow: 0 2px 4px rgba(0,0,0,0.1); 
        }
        .panel-header { 
            background: #f8f9fa; padding: 15px; border-bottom: 1px solid #ddd;
            font-weight: bold; border-radius: 8px 8px 0 0;
        }
        .panel-content { padding: 15px; }
        .xml-content { 
            font-family: 'Monaco', 'Consolas', monospace; 
            white-space: pre-wrap; font-size: 12px;
            background: #f8f8f8; padding: 15px; border-radius: 4px;
            overflow-x: auto; max-height: 400px;
        }
        .changes-list { max-height: 500px; overflow-y: auto; }
        .change-item { 
            border: 1px solid #ddd; border-radius: 4px; 
            margin-bottom: 10px; padding: 15px; background: white;
        }
        .change-critical { border-left: 4px solid #dc3545; }
        .change-added { border-left: 4px solid #28a745; }
        .change-removed { border-left: 4px solid #dc3545; }
        .change-modified { border-left: 4px solid #ffc107; }
        .change-header { font-weight: bold; margin-bottom: 8px; }
        .change-path { 
            font-family: monospace; background: #f1f1f1; 
            padding: 2px 6px; border-radius: 3px; font-size: 11px;
        }
        .change-values { margin-top: 10px; }
        .old-value, .new-value { 
            font-family: monospace; padding: 8px; border-radius: 3px; 
            margin: 5px 0; font-size: 12px;
        }
        .old-value { background-color: #ffebee; }
        .new-value { background-color: #e8f5e8; }
        .highlight-line { background-color: #fff3cd !important; }
        .no-changes { 
            text-align: center; padding: 40px; color: #6c757d;
            background: white; border-radius: 8px;
        }
        .filter-controls { margin-bottom: 15px; }
        .filter-btn { 
            background: #6c757d; color: white; border: none;
            padding: 5px 10px; border-radius: 3px; margin-right: 5px;
            font-size: 12px; cursor: pointer;
        }
        .filter-btn.active { background: #007bff; }
        
        /* Impact indicators */
        .impact-high { color: #dc3545; }
        .impact-medium { color: #fd7e14; }
        .impact-low { color: #28a745; }
        .impact-none { color: #6c757d; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🔍 DDEX Interactive Diff Viewer</h1>
        <p>Generated: <span id="timestamp">"#);
        
        html.push_str(&changeset.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string());
        html.push_str(r#"</span></p>
        <div class="stats">
            <div class="stat">
                <div class="stat-value">"#);
        html.push_str(&changeset.summary.total_changes.to_string());
        html.push_str(r#"</div>
                <div>Total Changes</div>
            </div>
            <div class="stat">
                <div class="stat-value impact-"#);
        html.push_str(&changeset.impact_level().to_string().to_lowercase());
        html.push_str(r#"">"#);
        html.push_str(&changeset.impact_level().to_string());
        html.push_str(r#"</div>
                <div>Impact Level</div>
            </div>
            <div class="stat">
                <div class="stat-value">"#);
        html.push_str(&changeset.summary.critical_changes.to_string());
        html.push_str(r#"</div>
                <div>Critical</div>
            </div>
        </div>
    </div>
    
    <div class="controls">
        <button class="btn active" onclick="showView('side-by-side')">Side by Side</button>
        <button class="btn" onclick="showView('changes-only')">Changes Only</button>
        <button class="btn" onclick="showView('summary')">Summary</button>
        <button class="btn" onclick="exportDiff('json')">Export JSON</button>
    </div>
"#);

        if changeset.has_changes() {
            // Side-by-side view
            html.push_str(r#"
    <div id="side-by-side-view" class="view">
        <div class="diff-container">
            <div class="diff-panel">
                <div class="panel-header">📄 Original DDEX</div>
                <div class="panel-content">
                    <div class="xml-content" id="old-xml">"#);
            html.push_str(&html_escape::encode_text(old_xml));
            html.push_str(r#"</div>
                </div>
            </div>
            <div class="diff-panel">
                <div class="panel-header">📝 Modified DDEX</div>
                <div class="panel-content">
                    <div class="xml-content" id="new-xml">"#);
            html.push_str(&html_escape::encode_text(new_xml));
            html.push_str(r#"</div>
                </div>
            </div>
        </div>
    </div>
    
    <div id="changes-only-view" class="view" style="display: none;">
        <div class="diff-panel">
            <div class="panel-header">
                🔄 Detected Changes
                <div class="filter-controls" style="float: right;">
                    <button class="filter-btn active" onclick="filterChanges('all')">All</button>
                    <button class="filter-btn" onclick="filterChanges('critical')">Critical</button>
                    <button class="filter-btn" onclick="filterChanges('added')">Added</button>
                    <button class="filter-btn" onclick="filterChanges('removed')">Removed</button>
                    <button class="filter-btn" onclick="filterChanges('modified')">Modified</button>
                </div>
            </div>
            <div class="panel-content">
                <div class="changes-list" id="changes-list">
"#);
            
            // Add changes
            for change in &changeset.changes {
                let change_class = match change.change_type {
                    ddex_builder::diff::types::ChangeType::ElementAdded | 
                    ddex_builder::diff::types::ChangeType::AttributeAdded => "change-added",
                    ddex_builder::diff::types::ChangeType::ElementRemoved | 
                    ddex_builder::diff::types::ChangeType::AttributeRemoved => "change-removed",
                    _ => "change-modified",
                };
                
                let critical_class = if change.is_critical { " change-critical" } else { "" };
                
                html.push_str(&format!(r#"
                    <div class="change-item {}{}" data-type="{}" data-critical="{}">
                        <div class="change-header">{} {}</div>
                        <div class="change-path">{}</div>
"#, 
                    change_class, critical_class,
                    change.change_type.to_string().to_lowercase(),
                    change.is_critical,
                    Self::change_type_icon(change.change_type),
                    html_escape::encode_text(&change.description),
                    html_escape::encode_text(&change.path.to_string())
                ));
                
                if let Some(old_val) = &change.old_value {
                    html.push_str(&format!(r#"
                        <div class="change-values">
                            <div class="old-value">Old: {}</div>
"#, html_escape::encode_text(old_val)));
                }
                
                if let Some(new_val) = &change.new_value {
                    html.push_str(&format!(r#"
                            <div class="new-value">New: {}</div>
                        </div>
"#, html_escape::encode_text(new_val)));
                } else if change.old_value.is_some() {
                    html.push_str("</div>");
                }
                
                html.push_str("</div>");
            }
            
            html.push_str(r#"
                </div>
            </div>
        </div>
    </div>
"#);
        } else {
            html.push_str(r#"
    <div class="no-changes">
        <h2>✅ No Semantic Changes Detected</h2>
        <p>The DDEX documents are semantically identical.</p>
    </div>
"#);
        }
        
        // Summary view
        html.push_str(r#"
    <div id="summary-view" class="view" style="display: none;">
        <div class="diff-panel">
            <div class="panel-header">📊 Diff Summary</div>
            <div class="panel-content">
                <pre style="white-space: pre-wrap; font-family: monospace;">"#);
        html.push_str(&html_escape::encode_text(&DiffFormatter::format_summary(changeset)));
        html.push_str(r#"</pre>
            </div>
        </div>
    </div>
    
    <script>
        // View switching
        function showView(viewName) {
            // Hide all views
            document.querySelectorAll('.view').forEach(view => view.style.display = 'none');
            
            // Show selected view
            document.getElementById(viewName + '-view').style.display = 'block';
            
            // Update button states
            document.querySelectorAll('.btn').forEach(btn => btn.classList.remove('active'));
            event.target.classList.add('active');
        }
        
        // Change filtering
        function filterChanges(type) {
            const changeItems = document.querySelectorAll('.change-item');
            
            changeItems.forEach(item => {
                if (type === 'all') {
                    item.style.display = 'block';
                } else if (type === 'critical') {
                    item.style.display = item.dataset.critical === 'true' ? 'block' : 'none';
                } else {
                    item.style.display = item.dataset.type.includes(type) ? 'block' : 'none';
                }
            });
            
            // Update filter button states
            document.querySelectorAll('.filter-btn').forEach(btn => btn.classList.remove('active'));
            event.target.classList.add('active');
        }
        
        // Export functionality
        function exportDiff(format) {
            if (format === 'json') {
                // This would call back to WASM to get JSON format
                console.log('Export JSON not yet implemented in this demo');
            }
        }
        
        // Initialize
        document.addEventListener('DOMContentLoaded', function() {
            // Any initialization code
        });
    </script>
</body>
</html>
"#);
        
        html
    }
    
    fn change_type_icon(change_type: ddex_builder::diff::types::ChangeType) -> &'static str {
        match change_type {
            ddex_builder::diff::types::ChangeType::ElementAdded | 
            ddex_builder::diff::types::ChangeType::AttributeAdded => "➕",
            ddex_builder::diff::types::ChangeType::ElementRemoved | 
            ddex_builder::diff::types::ChangeType::AttributeRemoved => "➖",
            ddex_builder::diff::types::ChangeType::ElementModified | 
            ddex_builder::diff::types::ChangeType::AttributeModified => "✏️",
            ddex_builder::diff::types::ChangeType::TextModified => "📝",
            ddex_builder::diff::types::ChangeType::ElementRenamed => "🔄",
            ddex_builder::diff::types::ChangeType::ElementMoved => "🔄",
        }
    }
}

impl Default for DdexDiffViewer {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for the diff viewer (serializable for JS)
#[derive(Serialize, Deserialize)]
pub struct DiffViewerConfig {
    pub ignore_formatting: bool,
    pub ignore_reference_ids: bool,
    pub ignore_order_changes: bool,
    pub show_line_numbers: bool,
    pub highlight_critical_changes: bool,
    pub theme: String, // "light" or "dark"
}

impl Default for DiffViewerConfig {
    fn default() -> Self {
        Self {
            ignore_formatting: true,
            ignore_reference_ids: true, 
            ignore_order_changes: true,
            show_line_numbers: true,
            highlight_critical_changes: true,
            theme: "light".to_string(),
        }
    }
}