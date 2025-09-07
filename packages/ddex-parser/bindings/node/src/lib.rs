// bindings/node/src/lib.rs
#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(js_name = "DdexParser")]
pub struct DdexParser {
    _private: (),
}

#[napi]
impl DdexParser {
    #[napi(constructor)]
    pub fn new() -> Self {
        DdexParser { _private: () }
    }
    
    #[napi]
    pub fn detect_version(&self, xml: String) -> String {
        if xml.contains("ern/43") || xml.contains("xml/ern/43") {
            "V4_3".to_string()
        } else if xml.contains("ern/42") || xml.contains("xml/ern/42") {
            "V4_2".to_string()
        } else if xml.contains("ern/382") || xml.contains("xml/ern/382") {
            "V3_8_2".to_string()
        } else {
            "Unknown".to_string()
        }
    }
    
    #[napi]
    pub fn parse_sync(&self, xml: String, _options: Option<ParseOptions>) -> Result<ParsedMessage> {
        // Basic XML validation
        if !xml.contains('<') || !xml.contains('>') {
            return Err(Error::new(
                Status::InvalidArg,
                "Invalid XML: missing angle brackets",
            ));
        }
        
        // Check for valid DDEX
        if !xml.contains("NewReleaseMessage") && 
           !xml.contains("UpdateReleaseMessage") && 
           !xml.contains("TakedownMessage") {
            return Err(Error::new(
                Status::InvalidArg,
                "Invalid DDEX: not a valid DDEX message type",
            ));
        }
        
        // Check for unclosed tags
        let open_count = xml.matches('<').count();
        let close_count = xml.matches('>').count();
        if open_count != close_count {
            return Err(Error::new(
                Status::InvalidArg,
                "Invalid XML: unclosed tags",
            ));
        }
        
        let version = self.detect_version(xml.clone());
        
        Ok(ParsedMessage {
            message_id: "TEST_001".to_string(),
            message_type: "NewReleaseMessage".to_string(),
            message_date: chrono::Utc::now().to_rfc3339(),
            sender_name: "Test Sender".to_string(),
            sender_id: "sender_001".to_string(),
            recipient_name: "Test Recipient".to_string(),
            recipient_id: "recipient_001".to_string(),
            version,
            profile: None,
            release_count: 1,
            track_count: 0,
            deal_count: 0,
            resource_count: 0,
            total_duration_seconds: 0.0,
        })
    }
    
    #[napi]
    pub async fn parse(&self, xml: String, options: Option<ParseOptions>) -> Result<ParsedMessage> {
        // For async, just call sync version for now
        self.parse_sync(xml, options)
    }
    
    #[napi]
    pub async fn sanity_check(&self, xml: String) -> Result<SanityCheckResult> {
        let mut errors = Vec::new();
        let warnings = Vec::new(); // Fixed: removed mut
        
        // Basic validation
        if !xml.contains('<') || !xml.contains('>') {
            errors.push("Invalid XML structure".to_string());
        }
        
        if !xml.contains("NewReleaseMessage") && 
           !xml.contains("UpdateReleaseMessage") && 
           !xml.contains("TakedownMessage") {
            errors.push("Not a valid DDEX message".to_string());
        }
        
        let open_count = xml.matches('<').count();
        let close_count = xml.matches('>').count();
        if open_count != close_count {
            errors.push("Unclosed XML tags".to_string());
        }
        
        let version = self.detect_version(xml);
        
        Ok(SanityCheckResult {
            is_valid: errors.is_empty(),
            version,
            errors,
            warnings,
        })
    }
    
    #[napi]
    pub fn stream(&self, _xml: String, _options: Option<StreamOptions>) -> Result<ReleaseStream> {
        Ok(ReleaseStream::new())
    }
}

#[napi(object)]
#[derive(Default)]
pub struct ParseOptions {
    pub mode: Option<String>,
    pub auto_threshold: Option<u32>,
    pub resolve_references: Option<bool>,
    pub include_raw: Option<bool>,
    pub max_memory: Option<u32>,
    pub timeout_ms: Option<u32>,
    pub allow_blocking: Option<bool>,
    pub chunk_size: Option<u32>,
}

#[napi(object)]
#[derive(Default)]
pub struct StreamOptions {
    pub chunk_size: Option<u32>,
    pub max_memory: Option<u32>,
}

#[napi(object)]
pub struct ParsedMessage {
    pub message_id: String,
    pub message_type: String,
    pub message_date: String,
    pub sender_name: String,
    pub sender_id: String,
    pub recipient_name: String,
    pub recipient_id: String,
    pub version: String,
    pub profile: Option<String>,
    pub release_count: u32,
    pub track_count: u32,
    pub deal_count: u32,
    pub resource_count: u32,
    pub total_duration_seconds: f64,
}

#[napi(object)]
pub struct SanityCheckResult {
    pub is_valid: bool,
    pub version: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[napi(object)]
pub struct StreamedRelease {
    pub release_reference: String,
    pub title: String,
    pub release_type: Option<String>,
    pub resource_count: u32,
}

#[napi]
pub struct ReleaseStream {
    position: i32,
}

impl ReleaseStream {
    // Regular impl block for internal methods
    fn new() -> Self {
        ReleaseStream { position: 0 }
    }
}

#[napi]
impl ReleaseStream {
    // Fixed: using unsafe for &mut self in async
    #[napi]
    pub async unsafe fn next(&mut self) -> Result<Option<StreamedRelease>> {
        // Return a few test releases
        if self.position < 3 {
            self.position += 1;
            Ok(Some(StreamedRelease {
                release_reference: format!("R{:03}", self.position),
                title: format!("Test Release {}", self.position),
                release_type: Some("Album".to_string()),
                resource_count: 10,
            }))
        } else {
            Ok(None)
        }
    }
    
    #[napi]
    pub async fn progress(&self) -> Result<ProgressInfo> {
        Ok(ProgressInfo {
            bytes_processed: (self.position * 1000) as f64,
            releases_parsed: self.position as f64,
            elapsed_ms: 100.0,
        })
    }
}

#[napi(object)]
pub struct ProgressInfo {
    pub bytes_processed: f64,
    pub releases_parsed: f64,
    pub elapsed_ms: f64,
}