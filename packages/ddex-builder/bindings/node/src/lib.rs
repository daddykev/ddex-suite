use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub release_id: String,
    pub release_type: String,
    pub title: String,
    pub artist: String,
    pub label: Option<String>,
    pub catalog_number: Option<String>,
    pub upc: Option<String>,
    pub release_date: Option<String>,
    pub genre: Option<String>,
    pub parental_warning: Option<bool>,
    pub track_ids: Vec<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_id: String,
    pub resource_type: String,
    pub title: String,
    pub artist: String,
    pub isrc: Option<String>,
    pub duration: Option<String>,
    pub track_number: Option<i32>,
    pub volume_number: Option<i32>,
    pub metadata: Option<HashMap<String, String>>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderStats {
    pub releases_count: u32,
    pub resources_count: u32,
    pub total_build_time_ms: f64,
    pub last_build_size_bytes: f64,
    pub validation_errors: u32,
    pub validation_warnings: u32,
}

#[napi]
pub struct DdexBuilder {
    releases: Vec<Release>,
    resources: Vec<Resource>,
    stats: BuilderStats,
}

#[napi]
impl DdexBuilder {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        Ok(DdexBuilder {
            releases: Vec::new(),
            resources: Vec::new(),
            stats: BuilderStats {
                releases_count: 0,
                resources_count: 0,
                total_build_time_ms: 0.0,
                last_build_size_bytes: 0.0,
                validation_errors: 0,
                validation_warnings: 0,
            },
        })
    }

    #[napi]
    pub fn add_release(&mut self, release: Release) -> Result<()> {
        self.releases.push(release);
        self.stats.releases_count = self.releases.len() as u32;
        Ok(())
    }

    #[napi]
    pub fn add_resource(&mut self, resource: Resource) -> Result<()> {
        self.resources.push(resource);
        self.stats.resources_count = self.resources.len() as u32;
        Ok(())
    }

    #[napi]
    pub async unsafe fn build(&mut self) -> Result<String> {
        let start_time = std::time::Instant::now();

        // For now, return a simple XML structure
        // In a complete implementation, this would use the actual DDEXBuilder
        let xml_output = self.generate_placeholder_xml()?;
        
        self.stats.last_build_size_bytes = xml_output.len() as f64;
        self.stats.total_build_time_ms += start_time.elapsed().as_millis() as f64;

        Ok(xml_output)
    }

    #[napi]
    pub async fn validate(&self) -> Result<ValidationResult> {
        Ok(ValidationResult {
            is_valid: !self.releases.is_empty(),
            errors: if self.releases.is_empty() { 
                vec!["At least one release is required".to_string()] 
            } else { 
                vec![] 
            },
            warnings: vec![],
        })
    }

    #[napi]
    pub fn get_stats(&self) -> Result<BuilderStats> {
        Ok(self.stats.clone())
    }

    #[napi]
    pub fn reset(&mut self) -> Result<()> {
        self.releases.clear();
        self.resources.clear();
        self.stats = BuilderStats {
            releases_count: 0,
            resources_count: 0,
            total_build_time_ms: 0.0,
            last_build_size_bytes: 0.0,
            validation_errors: 0,
            validation_warnings: 0,
        };
        Ok(())
    }

    fn generate_placeholder_xml(&self) -> Result<String> {
        // Generate a basic DDEX-like XML structure for demonstration
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43">"#);
        xml.push('\n');
        
        // Message header
        xml.push_str("  <MessageHeader>\n");
        xml.push_str(&format!("    <MessageId>{}</MessageId>\n", uuid::Uuid::new_v4()));
        xml.push_str("    <MessageSender>\n");
        xml.push_str("      <PartyName>DDEX Suite</PartyName>\n");
        xml.push_str("    </MessageSender>\n");
        xml.push_str("    <MessageRecipient>\n");
        xml.push_str("      <PartyName>Recipient</PartyName>\n");
        xml.push_str("    </MessageRecipient>\n");
        xml.push_str(&format!("    <MessageCreatedDateTime>{}</MessageCreatedDateTime>\n", chrono::Utc::now().to_rfc3339()));
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

#[napi]
pub async fn batch_build(requests: Vec<String>) -> Result<Vec<String>> {
    let mut results = Vec::new();
    
    for _request_json in requests {
        // Create a simple placeholder result for each request
        let result = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <MessageId>{}</MessageId>
    <MessageSender><PartyName>DDEX Suite</PartyName></MessageSender>
    <MessageRecipient><PartyName>Recipient</PartyName></MessageRecipient>
  </MessageHeader>
</NewReleaseMessage>"#, uuid::Uuid::new_v4());
        results.push(result);
    }
    
    Ok(results)
}

#[napi]
pub async fn validate_structure(xml: String) -> Result<ValidationResult> {
    // Parse and validate XML structure
    match quick_xml::Reader::from_str(&xml).read_event() {
        Ok(_) => Ok(ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
        }),
        Err(e) => Ok(ValidationResult {
            is_valid: false,
            errors: vec![format!("XML parsing error: {}", e)],
            warnings: vec![],
        }),
    }
}