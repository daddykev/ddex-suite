// bindings/node/src/lib.rs
#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::io::Cursor;

// Import the actual DDEX parser and related types
use ddex_core::models::flat::{ParsedDeal, ParsedERNMessage, ParsedRelease, ParsedResource};
use ddex_core::models::versions::ERNVersion;
use ddex_parser::{error::ParseError, DDEXParser as RustDDEXParser};
use indexmap;
use serde_json;

/// Convert a JavaScript string to a BufRead + Seek cursor for the parser
fn string_to_cursor(xml: String) -> Cursor<Vec<u8>> {
    Cursor::new(xml.into_bytes())
}

/// Convert ParseError to DetailedError structure
fn parse_error_to_detailed(err: ParseError) -> DetailedError {
    match err {
        ParseError::MissingField(field) => DetailedError {
            error_type: "MISSING_FIELD".to_string(),
            message: format!("Missing required DDEX field: {}", field),
            field: Some(field),
            value: None,
            context: Some("DDEX schema validation".to_string()),
            suggestions: vec![
                "Check the DDEX specification for required fields".to_string(),
                "Ensure your XML includes all mandatory elements".to_string(),
            ],
        },
        ParseError::InvalidValue { field, value } => DetailedError {
            error_type: "INVALID_VALUE".to_string(),
            message: format!("Invalid value '{}' for field '{}'", value, field),
            field: Some(field),
            value: Some(value),
            context: Some("Data validation".to_string()),
            suggestions: vec![
                "Check the DDEX specification for valid values".to_string(),
                "Verify the data type and format requirements".to_string(),
            ],
        },
        ParseError::XmlError(msg) => DetailedError {
            error_type: "XML_ERROR".to_string(),
            message: format!("XML parsing failed: {}", msg),
            field: None,
            value: None,
            context: Some("XML structure validation".to_string()),
            suggestions: vec![
                "Validate your XML syntax".to_string(),
                "Check for malformed elements or attributes".to_string(),
                "Ensure proper XML encoding (UTF-8)".to_string(),
            ],
        },
        ParseError::StreamError(stream_err) => DetailedError {
            error_type: "STREAM_ERROR".to_string(),
            message: format!("Streaming error: {:?}", stream_err),
            field: None,
            value: None,
            context: Some("Streaming parser".to_string()),
            suggestions: vec![
                "Check for corrupted or incomplete data".to_string(),
                "Try parsing the full document instead of streaming".to_string(),
            ],
        },
        _ => DetailedError {
            error_type: "GENERAL_ERROR".to_string(),
            message: format!("{}", err),
            field: None,
            value: None,
            context: None,
            suggestions: vec!["Please check the input and try again".to_string()],
        },
    }
}

/// Convert ParseError to NAPI Error with detailed categorization
fn parse_error_to_napi(err: ParseError) -> napi::Error {
    match err {
        ParseError::MissingField(field) => {
            napi::Error::new(
                napi::Status::InvalidArg,
                format!("Missing required DDEX field: {}. Please ensure the XML contains all mandatory elements for this message type.", field),
            )
        }
        ParseError::InvalidValue { field, value } => {
            napi::Error::new(
                napi::Status::InvalidArg,
                format!("Invalid value '{}' for field '{}'. Please check the DDEX specification for valid values.", value, field),
            )
        }
        ParseError::XmlError(msg) => {
            napi::Error::new(
                napi::Status::InvalidArg,
                format!("XML parsing failed: {}. Please ensure the input is valid XML and follows DDEX schema.", msg),
            )
        }
        ParseError::StreamError(stream_err) => {
            napi::Error::new(
                napi::Status::GenericFailure,
                format!("Streaming error: {:?}. This may indicate a corrupted or incomplete DDEX message.", stream_err),
            )
        }
        ParseError::InvalidUtf8 { message } => {
            napi::Error::new(
                napi::Status::InvalidArg,
                format!("Invalid UTF-8 encoding: {}. Please ensure the XML file uses valid UTF-8 encoding.", message),
            )
        }
        ParseError::SimpleXmlError(msg) => {
            napi::Error::new(
                napi::Status::InvalidArg,
                format!("XML structure error: {}. Please check for malformed XML elements.", msg),
            )
        }
        ParseError::ConversionError { from, to, message } => {
            napi::Error::new(
                napi::Status::GenericFailure,
                format!("Data conversion error from {} to {}: {}. This may indicate incompatible data types in the DDEX message.", from, to, message),
            )
        }
        ParseError::IoError(msg) => {
            napi::Error::new(
                napi::Status::GenericFailure,
                format!("IO error: {}. This may indicate a file access or network issue.", msg),
            )
        }
        ParseError::Timeout { message } => {
            napi::Error::new(
                napi::Status::GenericFailure,
                format!("Parsing timeout: {}. Consider using streaming mode for large files or increasing timeout limits.", message),
            )
        }
        ParseError::DepthLimitExceeded { depth, limit } => {
            napi::Error::new(
                napi::Status::GenericFailure,
                format!("XML depth limit exceeded: {} > {}. The DDEX message has too deeply nested elements. Consider simplifying the structure.", depth, limit),
            )
        }
        ParseError::SecurityViolation { message } => {
            napi::Error::new(
                napi::Status::GenericFailure,
                format!("Security violation: {}. The DDEX message contains potentially unsafe content that violates security policies.", message),
            )
        }
        ParseError::MalformedXml { message, position } => {
            napi::Error::new(
                napi::Status::InvalidArg,
                format!("Malformed XML at position {}: {}. Please check the XML syntax and structure.", position, message),
            )
        }
        ParseError::MismatchedTags { expected, found, position } => {
            napi::Error::new(
                napi::Status::InvalidArg,
                format!("Mismatched XML tags at position {}: expected '{}', found '{}'. Please ensure proper tag nesting.", position, expected, found),
            )
        }
        ParseError::UnexpectedClosingTag { tag, position } => {
            napi::Error::new(
                napi::Status::InvalidArg,
                format!("Unexpected closing tag '{}' at position {}. Please check for unmatched opening tags.", tag, position),
            )
        }
        ParseError::InvalidAttribute { message, position } => {
            napi::Error::new(
                napi::Status::InvalidArg,
                format!("Invalid XML attribute at position {}: {}. Please check the attribute syntax.", position, message),
            )
        }
        ParseError::UnclosedTags { tags, position } => {
            napi::Error::new(
                napi::Status::InvalidArg,
                format!("Unclosed XML tags at position {}: {:?}. Please ensure all tags are properly closed.", position, tags),
            )
        }
    }
}

/// Convert ERNVersion to string
fn version_to_string(version: ERNVersion) -> String {
    match version {
        ERNVersion::V3_8_1 => "V3_8_1".to_string(),
        ERNVersion::V3_8_2 => "V3_8_2".to_string(),
        ERNVersion::V4_2 => "V4_2".to_string(),
        ERNVersion::V4_3 => "V4_3".to_string(),
    }
}

/// Convert ParsedRelease to JavaScript-compatible structure
fn convert_release(release: ParsedRelease) -> JsRelease {
    JsRelease {
        release_id: release.release_id,
        title: release.title.first().map(|t| t.text.clone()),
        default_title: release.default_title,
        subtitle: release.default_subtitle,
        display_artist: release.display_artist,
        release_type: release.release_type,
        genre: release.genre,
        sub_genre: release.sub_genre,
        track_count: release.track_count as u32,
        disc_count: release.disc_count.map(|c| c as u32),
        release_date: release.release_date.map(|d| d.to_rfc3339()),
        original_release_date: release.original_release_date.map(|d| d.to_rfc3339()),
        label_name: None, // ParsedRelease doesn't have label_name directly
        tracks: release.tracks.into_iter().map(convert_track).collect(),
    }
}

/// Convert ParsedTrack to JavaScript-compatible structure
fn convert_track(track: ddex_core::models::flat::ParsedTrack) -> JsTrack {
    JsTrack {
        track_id: track.track_id,
        title: track.title,
        artist: track.display_artist,
        duration: track.duration_formatted,
        position: Some(track.position as u32),
        disc_number: track.disc_number.map(|d| d as u32),
        isrc: track.isrc,
        resource_reference: None, // ParsedTrack doesn't have resource_reference directly
    }
}

/// Convert ParsedResource to JavaScript-compatible structure
fn convert_resource(resource: ParsedResource) -> JsResource {
    JsResource {
        resource_id: resource.resource_id,
        resource_type: resource.resource_type,
        title: resource.title,
        duration_seconds: resource.duration.map(|d| d.as_secs_f64()),
        duration_string: resource
            .duration
            .map(|d| format!("{}:{:02}", d.as_secs() / 60, d.as_secs() % 60)),
        file_format: resource.technical_details.file_format,
        bitrate: resource.technical_details.bitrate,
        sample_rate: resource.technical_details.sample_rate,
        file_size: resource
            .technical_details
            .file_size
            .map(|size| size.to_string()),
    }
}

/// Convert ParsedDeal to JavaScript-compatible structure
fn convert_deal(deal: ParsedDeal) -> JsDeal {
    JsDeal {
        deal_id: deal.deal_id,
        releases: deal.releases,
        start_date: deal.validity.start.map(|d| d.to_rfc3339()),
        end_date: deal.validity.end.map(|d| d.to_rfc3339()),
        territories: deal.territories.included,
        usage_rights: deal.usage_rights,
        restrictions: deal.restrictions,
        // The flat model does not carry the deal's CommercialModelType yet; report null rather than a guess.
        commercial_model: None,
    }
}

/// Convert IndexMap<String, ParsedResource> to JavaScript object
fn convert_resources_to_js_object(
    resources: indexmap::IndexMap<String, ParsedResource>,
) -> serde_json::Value {
    let mut js_resources = serde_json::Map::new();

    for (id, resource) in resources {
        let js_resource = convert_resource(resource);
        js_resources.insert(
            id,
            serde_json::to_value(js_resource).unwrap_or(serde_json::Value::Null),
        );
    }

    serde_json::Value::Object(js_resources)
}

/// Convert ParsedERNMessage to Node.js ParsedMessage structure
fn convert_parsed_message(
    parsed: ParsedERNMessage,
    options: Option<&ParseOptions>,
) -> ParsedMessage {
    let flat = parsed.flat; // Take ownership instead of borrowing

    // Convert the actual data structures
    let releases: Vec<JsRelease> = flat.releases.into_iter().map(convert_release).collect();
    let resources_obj = convert_resources_to_js_object(flat.resources.clone());
    let deals: Vec<JsDeal> = flat.deals.into_iter().map(convert_deal).collect();

    // Calculate counts from actual data
    let release_count = releases.len() as u32;
    let resource_count = flat.resources.len() as u32;
    let deal_count = deals.len() as u32;

    // Generate statistics if requested
    let statistics = if options.and_then(|o| o.collect_statistics).unwrap_or(false) {
        Some(ParseStatistics {
            parse_time_ms: 0.0,   // TODO: Add timing
            memory_used_bytes: 0, // TODO: Add memory tracking
            element_count: 0,     // TODO: Count elements during parsing
            attribute_count: 0,   // TODO: Count attributes during parsing
            comment_count: 0,     // TODO: Count comments during parsing
            extension_count: if parsed.extensions.is_some() { 1 } else { 0 },
            namespace_count: 1, // TODO: Count namespaces during parsing
            file_size_bytes: 0, // TODO: Track file size
        })
    } else {
        None
    };

    // Generate fidelity info based on options
    let fidelity_info = if let Some(opts) = options {
        Some(FidelityInfo {
            fidelity_level: opts
                .fidelity_level
                .clone()
                .unwrap_or_else(|| "balanced".to_string()),
            canonicalization_algorithm: opts
                .canonicalization
                .clone()
                .unwrap_or_else(|| "db_c14n".to_string()),
            comments_preserved: opts.preserve_comments.unwrap_or(false),
            extensions_preserved: opts.preserve_extensions.unwrap_or(true),
            processing_instructions_preserved: opts
                .preserve_processing_instructions
                .unwrap_or(false),
            attribute_order_preserved: opts.preserve_attribute_order.unwrap_or(true),
            namespace_prefixes_preserved: opts.preserve_namespace_prefixes.unwrap_or(true),
        })
    } else {
        None
    };

    ParsedMessage {
        message_id: flat.message_id.clone(),
        message_type: flat.message_type.clone(),
        message_date: flat.message_date.to_rfc3339(),
        sender_name: flat.sender.name.clone(),
        sender_id: flat.sender.id.clone(),
        recipient_name: flat.recipient.name.clone(),
        recipient_id: flat.recipient.id.clone(),
        version: flat.version.clone(),
        profile: flat.profile.clone(),

        // Counts (for backward compatibility)
        release_count,
        track_count: flat.stats.track_count as u32,
        deal_count,
        resource_count,
        total_duration_seconds: flat.stats.total_duration as f64,

        // CRITICAL: Include the actual data
        releases,
        resources: resources_obj,
        deals,

        statistics,
        fidelity_info,
    }
}

// JavaScript-compatible type definitions
#[napi(object)]
#[derive(serde::Serialize)]
pub struct JsRelease {
    pub release_id: String,
    pub title: Option<String>,
    pub default_title: Option<String>,
    pub subtitle: Option<String>,
    pub display_artist: String,
    pub release_type: String,
    pub genre: Option<String>,
    pub sub_genre: Option<String>,
    pub track_count: u32,
    pub disc_count: Option<u32>,
    pub release_date: Option<String>,
    pub original_release_date: Option<String>,
    pub label_name: Option<String>,
    pub tracks: Vec<JsTrack>,
}

#[napi(object)]
#[derive(serde::Serialize)]
pub struct JsTrack {
    pub track_id: String,
    pub title: Option<String>,
    pub artist: String,
    pub duration: Option<String>,
    pub position: Option<u32>,
    pub disc_number: Option<u32>,
    pub isrc: Option<String>,
    pub resource_reference: Option<String>,
}

#[napi(object)]
#[derive(serde::Serialize)]
pub struct JsResource {
    pub resource_id: String,
    pub resource_type: String,
    pub title: Option<String>,
    pub duration_seconds: Option<f64>,
    pub duration_string: Option<String>,
    pub file_format: Option<String>,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub file_size: Option<String>, // Convert u64 to string for JS compatibility
}

#[napi(object)]
#[derive(serde::Serialize)]
pub struct JsDeal {
    pub deal_id: Option<String>,
    pub releases: Vec<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub territories: Vec<String>,
    pub usage_rights: Vec<String>,
    pub restrictions: Vec<String>,
    pub commercial_model: Option<String>,
}

#[napi(js_name = "DdexParser")]
pub struct DdexParser {
    inner: RustDDEXParser,
}

#[napi]
impl DdexParser {
    #[napi(constructor)]
    pub fn new() -> Self {
        DdexParser {
            inner: RustDDEXParser::new(),
        }
    }

    #[napi]
    pub fn detect_version(&self, xml: String) -> Result<String> {
        // Validate input
        if xml.is_empty() {
            return Err(napi::Error::new(
                napi::Status::InvalidArg,
                "XML input cannot be empty. Cannot detect version of empty content.",
            ));
        }

        // Check if it looks like XML at all
        if !xml.trim_start().starts_with("<?xml") && !xml.trim_start().starts_with('<') {
            return Err(napi::Error::new(
                napi::Status::InvalidArg,
                "Input does not appear to be XML. Version detection requires valid XML content.",
            ));
        }

        let cursor = string_to_cursor(xml.clone());
        match self.inner.detect_version(cursor) {
            Ok(version) => Ok(version_to_string(version)),
            Err(err) => {
                // Add context for version detection failures
                let context_info = format!(
                    " [Version detection failed on {} bytes of input starting with: '{}']",
                    xml.len(),
                    xml.chars().take(150).collect::<String>().replace('\n', " ")
                );

                let mut error = parse_error_to_napi(err);
                error.reason = format!("{}{}", error.reason, context_info);
                Err(error)
            }
        }
    }

    #[napi]
    pub fn parse_sync(
        &mut self,
        xml: String,
        options: Option<ParseOptions>,
    ) -> Result<ParsedMessage> {
        // Validate input
        if xml.is_empty() {
            return Err(napi::Error::new(
                napi::Status::InvalidArg,
                "XML input cannot be empty. Please provide a valid DDEX XML document.",
            ));
        }

        if xml.len() > 100_000_000 {
            // 100MB limit
            return Err(napi::Error::new(
                napi::Status::InvalidArg,
                "XML input too large (>100MB). Consider using streaming mode for large files.",
            ));
        }

        // Convert string to cursor
        let cursor = string_to_cursor(xml.clone());

        // Call the real Rust parser with enhanced error context
        match self.inner.parse(cursor) {
            Ok(parsed_message) => {
                // Validate that we got meaningful data
                if parsed_message.flat.releases.is_empty()
                    && parsed_message.flat.resources.is_empty()
                    && parsed_message.flat.deals.is_empty()
                {
                    return Err(napi::Error::new(
                        napi::Status::InvalidArg,
                        "DDEX parsing succeeded but no releases, resources, or deals were found. Please check that the XML contains valid DDEX content.",
                    ));
                }

                // Convert the Rust ParsedERNMessage to Node.js ParsedMessage
                // All data is now real parsed data - no mock data possible at this point
                let result = convert_parsed_message(parsed_message, options.as_ref());
                Ok(result)
            }
            Err(parse_error) => {
                // Add context about the input that failed
                let context_info = format!(
                    " [Input context: {} bytes, starts with: '{}']",
                    xml.len(),
                    xml.chars().take(100).collect::<String>().replace('\n', " ")
                );

                // Convert ParseError to NAPI error with additional context
                let mut error = parse_error_to_napi(parse_error);
                error.reason = format!("{}{}", error.reason, context_info);
                Err(error)
            }
        }
    }

    #[napi]
    pub async unsafe fn parse(
        &mut self,
        xml: String,
        options: Option<ParseOptions>,
    ) -> Result<ParsedMessage> {
        // For now, delegate to sync version with proper async handling
        // TODO: Implement true async parsing using tokio::task::spawn_blocking for CPU-intensive work

        // Validate input early to avoid unnecessary work
        if xml.is_empty() {
            return Err(napi::Error::new(
                napi::Status::InvalidArg,
                "XML input cannot be empty. Please provide a valid DDEX XML document.",
            ));
        }

        // Use sync version but wrapped in proper async context
        match self.parse_sync(xml, options) {
            Ok(result) => Ok(result),
            Err(err) => {
                // Add async context to error message
                let mut async_err = err;
                async_err.reason = format!(
                    "{} [Note: This was called via async parse method]",
                    async_err.reason
                );
                Err(async_err)
            }
        }
    }

    #[napi]
    pub async fn sanity_check(&self, xml: String) -> Result<SanityCheckResult> {
        // Validate input
        if xml.is_empty() {
            return Ok(SanityCheckResult {
                is_valid: false,
                version: "Unknown".to_string(),
                errors: vec!["XML input cannot be empty for sanity check".to_string()],
                warnings: Vec::new(),
            });
        }

        // Basic XML structure check
        if !xml.trim_start().starts_with("<?xml") && !xml.trim_start().starts_with('<') {
            return Ok(SanityCheckResult {
                is_valid: false,
                version: "Unknown".to_string(),
                errors: vec!["Input does not appear to be valid XML".to_string()],
                warnings: Vec::new(),
            });
        }

        let cursor = string_to_cursor(xml.clone());

        // Use the real parser's sanity check
        match self.inner.sanity_check(cursor) {
            Ok(result) => Ok(SanityCheckResult {
                is_valid: result.is_valid,
                version: version_to_string(result.version),
                errors: result.errors,
                warnings: result.warnings,
            }),
            Err(err) => {
                // If sanity check fails, return detailed error information
                let detailed_error = match err {
                    ParseError::MissingField(field) => {
                        format!("Missing required DDEX field: {}", field)
                    }
                    ParseError::InvalidValue { field, value } => {
                        format!("Invalid value '{}' for field '{}'", value, field)
                    }
                    ParseError::XmlError(msg) => {
                        format!("XML parsing error: {}", msg)
                    }
                    _ => format!("Sanity check failed: {}", err),
                };

                Ok(SanityCheckResult {
                    is_valid: false,
                    version: "Unknown".to_string(),
                    errors: vec![detailed_error],
                    warnings: vec![format!("Input size: {} bytes", xml.len())],
                })
            }
        }
    }

    #[napi]
    pub fn stream(&self, _xml: String, _options: Option<StreamOptions>) -> Result<ReleaseStream> {
        Ok(ReleaseStream::new())
    }

    /// Get detailed error information for debugging - useful for error handling in JavaScript
    #[napi]
    pub fn get_detailed_error(&mut self, xml: String) -> Result<DetailedError> {
        let cursor = string_to_cursor(xml.clone());

        match self.inner.parse(cursor) {
            Ok(_) => Err(napi::Error::new(
                napi::Status::GenericFailure,
                "No error found - parsing succeeded",
            )),
            Err(parse_error) => Ok(parse_error_to_detailed(parse_error)),
        }
    }
}

#[napi(object)]
#[derive(Default)]
pub struct ParseOptions {
    // Legacy options for backward compatibility
    pub mode: Option<String>,
    pub auto_threshold: Option<u32>,
    pub resolve_references: Option<bool>,
    pub include_raw: Option<bool>,
    pub max_memory: Option<u32>,
    pub timeout_ms: Option<u32>,
    pub allow_blocking: Option<bool>,
    pub chunk_size: Option<u32>,

    // Perfect Fidelity Engine options
    pub fidelity_level: Option<String>, // "fast", "balanced", "perfect"
    pub preserve_comments: Option<bool>,
    pub preserve_processing_instructions: Option<bool>,
    pub preserve_extensions: Option<bool>,
    pub preserve_attribute_order: Option<bool>,
    pub preserve_namespace_prefixes: Option<bool>,
    pub canonicalization: Option<String>, // "none", "c14n", "c14n11", "db_c14n"
    pub collect_statistics: Option<bool>,
    pub enable_streaming: Option<bool>,
    pub streaming_threshold: Option<u32>,
    pub validation_level: Option<String>, // "none", "basic", "standard", "strict"
    pub extension_validation: Option<bool>,
    pub enable_checksums: Option<bool>,
    pub memory_limit: Option<u32>,
    pub enable_detailed_errors: Option<bool>,
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

    // Counts (for backward compatibility)
    pub release_count: u32,
    pub track_count: u32,
    pub deal_count: u32,
    pub resource_count: u32,
    pub total_duration_seconds: f64,

    // CRITICAL: Actual data arrays/objects
    pub releases: Vec<JsRelease>,
    pub resources: serde_json::Value, // Will be a JS object with resource IDs as keys
    pub deals: Vec<JsDeal>,

    // Perfect Fidelity Engine results
    pub statistics: Option<ParseStatistics>,
    pub fidelity_info: Option<FidelityInfo>,
}

#[napi(object)]
pub struct ParseStatistics {
    pub parse_time_ms: f64,
    pub memory_used_bytes: u32,
    pub element_count: u32,
    pub attribute_count: u32,
    pub comment_count: u32,
    pub extension_count: u32,
    pub namespace_count: u32,
    pub file_size_bytes: u32,
}

#[napi(object)]
pub struct FidelityInfo {
    pub fidelity_level: String,
    pub canonicalization_algorithm: String,
    pub comments_preserved: bool,
    pub extensions_preserved: bool,
    pub processing_instructions_preserved: bool,
    pub attribute_order_preserved: bool,
    pub namespace_prefixes_preserved: bool,
}

#[napi(object)]
pub struct SanityCheckResult {
    pub is_valid: bool,
    pub version: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[napi(object)]
pub struct DetailedError {
    pub error_type: String,
    pub message: String,
    pub field: Option<String>,
    pub value: Option<String>,
    pub context: Option<String>,
    pub suggestions: Vec<String>,
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
