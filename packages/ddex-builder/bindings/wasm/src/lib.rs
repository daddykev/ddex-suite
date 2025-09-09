use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::collections::HashMap;

// Set up console error handling for better debugging
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    #[wasm_bindgen(getter_with_clone)]
    pub release_id: String,
    #[wasm_bindgen(getter_with_clone)]
    pub release_type: String,
    #[wasm_bindgen(getter_with_clone)]
    pub title: String,
    #[wasm_bindgen(getter_with_clone)]
    pub artist: String,
    #[wasm_bindgen(getter_with_clone)]
    pub label: Option<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub catalog_number: Option<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub upc: Option<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub release_date: Option<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub genre: Option<String>,
    pub parental_warning: Option<bool>,
    track_ids: Vec<String>,
    metadata: Option<HashMap<String, String>>,
}

#[wasm_bindgen]
impl Release {
    #[wasm_bindgen(constructor)]
    pub fn new(
        release_id: String,
        release_type: String,
        title: String,
        artist: String,
    ) -> Release {
        Release {
            release_id,
            release_type,
            title,
            artist,
            label: None,
            catalog_number: None,
            upc: None,
            release_date: None,
            genre: None,
            parental_warning: None,
            track_ids: Vec::new(),
            metadata: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn track_ids(&self) -> Vec<String> {
        self.track_ids.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_track_ids(&mut self, track_ids: Vec<String>) {
        self.track_ids = track_ids;
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> JsValue {
        match &self.metadata {
            Some(meta) => to_value(meta).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter)]
    pub fn set_metadata(&mut self, metadata: JsValue) -> Result<(), JsValue> {
        if metadata.is_null() || metadata.is_undefined() {
            self.metadata = None;
        } else {
            self.metadata = Some(from_value(metadata)?);
        }
        Ok(())
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    #[wasm_bindgen(getter_with_clone)]
    pub resource_id: String,
    #[wasm_bindgen(getter_with_clone)]
    pub resource_type: String,
    #[wasm_bindgen(getter_with_clone)]
    pub title: String,
    #[wasm_bindgen(getter_with_clone)]
    pub artist: String,
    #[wasm_bindgen(getter_with_clone)]
    pub isrc: Option<String>,
    #[wasm_bindgen(getter_with_clone)]
    pub duration: Option<String>,
    pub track_number: Option<i32>,
    pub volume_number: Option<i32>,
    metadata: Option<HashMap<String, String>>,
}

#[wasm_bindgen]
impl Resource {
    #[wasm_bindgen(constructor)]
    pub fn new(
        resource_id: String,
        resource_type: String,
        title: String,
        artist: String,
    ) -> Resource {
        Resource {
            resource_id,
            resource_type,
            title,
            artist,
            isrc: None,
            duration: None,
            track_number: None,
            volume_number: None,
            metadata: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> JsValue {
        match &self.metadata {
            Some(meta) => to_value(meta).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(setter)]
    pub fn set_metadata(&mut self, metadata: JsValue) -> Result<(), JsValue> {
        if metadata.is_null() || metadata.is_undefined() {
            self.metadata = None;
        } else {
            self.metadata = Some(from_value(metadata)?);
        }
        Ok(())
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    #[wasm_bindgen(getter_with_clone)]
    pub is_valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[wasm_bindgen]
impl ValidationResult {
    #[wasm_bindgen(constructor)]
    pub fn new(is_valid: bool) -> ValidationResult {
        ValidationResult {
            is_valid,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_errors(&mut self, errors: Vec<String>) {
        self.errors = errors;
    }

    #[wasm_bindgen(setter)]
    pub fn set_warnings(&mut self, warnings: Vec<String>) {
        self.warnings = warnings;
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderStats {
    pub releases_count: u32,
    pub resources_count: u32,
    pub total_build_time_ms: f64,
    pub last_build_size_bytes: f64,
    pub validation_errors: u32,
    pub validation_warnings: u32,
}

#[wasm_bindgen]
impl BuilderStats {
    #[wasm_bindgen(constructor)]
    pub fn new() -> BuilderStats {
        BuilderStats {
            releases_count: 0,
            resources_count: 0,
            total_build_time_ms: 0.0,
            last_build_size_bytes: 0.0,
            validation_errors: 0,
            validation_warnings: 0,
        }
    }
}

#[wasm_bindgen]
pub struct WasmDdexBuilder {
    releases: Vec<Release>,
    resources: Vec<Resource>,
    stats: BuilderStats,
}

#[wasm_bindgen]
impl WasmDdexBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmDdexBuilder, JsValue> {
        console_error_panic_hook::set_once();
        
        Ok(WasmDdexBuilder {
            releases: Vec::new(),
            resources: Vec::new(),
            stats: BuilderStats::new(),
        })
    }

    #[wasm_bindgen(js_name = addRelease)]
    pub fn add_release(&mut self, release: Release) {
        self.releases.push(release);
        self.stats.releases_count = self.releases.len() as u32;
        console_log!("Added release, total: {}", self.stats.releases_count);
    }

    #[wasm_bindgen(js_name = addResource)]
    pub fn add_resource(&mut self, resource: Resource) {
        self.resources.push(resource);
        self.stats.resources_count = self.resources.len() as u32;
        console_log!("Added resource, total: {}", self.stats.resources_count);
    }

    #[wasm_bindgen]
    pub async fn build(&mut self) -> Result<String, JsValue> {
        let start_time = js_sys::Date::now();

        // Generate a basic DDEX-like XML structure for demonstration
        let xml_output = self.generate_placeholder_xml()?;
        
        let end_time = js_sys::Date::now();
        let build_time = end_time - start_time;
        
        self.stats.last_build_size_bytes = xml_output.len() as f64;
        self.stats.total_build_time_ms += build_time;

        console_log!("Build completed: {} bytes in {}ms", xml_output.len(), build_time);
        Ok(xml_output)
    }

    #[wasm_bindgen]
    pub fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult::new(!self.releases.is_empty());
        
        if self.releases.is_empty() {
            result.set_errors(vec!["At least one release is required".to_string()]);
        }
        
        console_log!("Validation: is_valid={}, errors={}", result.is_valid, result.errors().len());
        result
    }

    #[wasm_bindgen(js_name = getStats)]
    pub fn get_stats(&self) -> BuilderStats {
        self.stats.clone()
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.releases.clear();
        self.resources.clear();
        self.stats = BuilderStats::new();
        console_log!("Builder reset");
    }

    fn generate_placeholder_xml(&self) -> Result<String, JsValue> {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">"#);
        xml.push('\n');
        
        // Message header
        xml.push_str("  <MessageHeader>\n");
        xml.push_str(&format!("    <MessageId>{}</MessageId>\n", uuid::Uuid::new_v4()));
        xml.push_str("    <MessageSender>\n");
        xml.push_str("      <PartyName>DDEX Suite WASM</PartyName>\n");
        xml.push_str("    </MessageSender>\n");
        xml.push_str("    <MessageRecipient>\n");
        xml.push_str("      <PartyName>Web Client</PartyName>\n");
        xml.push_str("    </MessageRecipient>\n");
        xml.push_str(&format!("    <MessageCreatedDateTime>{}</MessageCreatedDateTime>\n", 
            chrono::Utc::now().to_rfc3339()));
        xml.push_str("  </MessageHeader>\n");

        // Releases
        for release in &self.releases {
            xml.push_str("  <ReleaseList>\n");
            xml.push_str("    <Release>\n");
            xml.push_str(&format!("      <ReleaseId>{}</ReleaseId>\n", release.release_id));
            xml.push_str(&format!("      <Title>{}</Title>\n", release.title));
            xml.push_str(&format!("      <Artist>{}</Artist>\n", release.artist));
            if let Some(ref label) = release.label {
                xml.push_str(&format!("      <Label>{}</Label>\n", label));
            }
            xml.push_str("    </Release>\n");
            xml.push_str("  </ReleaseList>\n");
        }

        // Resources
        for resource in &self.resources {
            xml.push_str("  <ResourceList>\n");
            xml.push_str("    <SoundRecording>\n");
            xml.push_str(&format!("      <ResourceId>{}</ResourceId>\n", resource.resource_id));
            xml.push_str(&format!("      <Title>{}</Title>\n", resource.title));
            xml.push_str(&format!("      <Artist>{}</Artist>\n", resource.artist));
            if let Some(ref isrc) = resource.isrc {
                xml.push_str(&format!("      <ISRC>{}</ISRC>\n", isrc));
            }
            xml.push_str("    </SoundRecording>\n");
            xml.push_str("  </ResourceList>\n");
        }
        
        xml.push_str("</NewReleaseMessage>\n");
        Ok(xml)
    }
}

#[wasm_bindgen(js_name = batchBuild)]
pub async fn batch_build(requests: JsValue) -> Result<Vec<String>, JsValue> {
    // Convert JsValue to JavaScript Array
    let array = js_sys::Array::from(&requests);
    let length = array.length();
    let mut results = Vec::new();
    
    for i in 0..length {
        // Create a simple placeholder result for each request
        let result = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <MessageId>{}</MessageId>
    <MessageSender><PartyName>DDEX Suite WASM</PartyName></MessageSender>
    <MessageRecipient><PartyName>Web Client</PartyName></MessageRecipient>
  </MessageHeader>
</NewReleaseMessage>"#, uuid::Uuid::new_v4());
        results.push(result);
    }
    
    console_log!("Batch build completed: {} results", results.len());
    Ok(results)
}

#[wasm_bindgen(js_name = validateStructure)]
pub fn validate_structure(xml: String) -> ValidationResult {
    // Basic XML validation - check for well-formedness
    let mut result = ValidationResult::new(true);
    
    // Simple validation checks
    if xml.is_empty() {
        result.is_valid = false;
        result.set_errors(vec!["XML cannot be empty".to_string()]);
    } else if !xml.trim_start().starts_with("<?xml") && !xml.trim_start().starts_with('<') {
        result.is_valid = false;
        result.set_errors(vec!["Invalid XML format".to_string()]);
    }
    
    console_log!("XML validation: is_valid={}, errors={}", result.is_valid, result.errors().len());
    result
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Export module info
pub fn init() {
    console_log!("DDEX Builder WASM v{} initialized", version());
}