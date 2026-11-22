// bindings/node/src/lib.rs
#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::io::Cursor;

// Import the actual DDEX parser and related types
use ddex_core::models::flat::{ParsedDeal, ParsedERNMessage, ParsedRelease, ParsedResource};
use ddex_core::models::versions::ERNVersion;
use ddex_parser::{error::ParseError, DDEXParser as RustDDEXParser};

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
                format!("Parsing timeout: {}. Consider splitting the input or increasing timeout limits.", message),
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
        label_name: release.label_name,
        tracks: release.tracks.into_iter().map(convert_track).collect(),
        icpn: release.identifiers.upc,
        isrc: release.identifiers.isrc,
        grid: release.identifiers.grid,
        proprietary_ids: release
            .identifiers
            .proprietary
            .into_iter()
            .map(|p| JsProprietaryId {
                namespace: p.namespace,
                value: p.value,
            })
            .collect(),
        genres: release
            .genres
            .into_iter()
            .map(|g| JsGenre {
                genre: g.genre,
                sub_genre: g.sub_genre,
                territory_codes: g.scope.territory_codes,
                excluded_territory_codes: g.scope.excluded_territory_codes,
                is_default: g.scope.is_default,
            })
            .collect(),
        parental_warning: release.parental_warning,
        parental_warnings: release.parental_warnings,
        p_line: release.p_line.map(convert_copyright),
        c_line: release.c_line.map(convert_copyright),
        release_dates: release
            .release_dates
            .into_iter()
            .map(convert_date)
            .collect(),
        original_release_dates: release
            .original_release_dates
            .into_iter()
            .map(convert_date)
            .collect(),
    }
}

fn convert_copyright(c: ddex_core::models::common::Copyright) -> JsCopyright {
    JsCopyright {
        text: c.text,
        year: c.year,
        company: c.owner,
        territory_codes: c.scope.territory_codes,
        is_default: c.scope.is_default,
    }
}

fn convert_date(d: ddex_core::models::flat::DateInfo) -> JsDateInfo {
    JsDateInfo {
        date: d.date,
        is_approximate: d.is_approximate,
        territory_codes: d.scope.territory_codes,
        excluded_territory_codes: d.scope.excluded_territory_codes,
        is_default: d.scope.is_default,
    }
}

/// Convert ParsedTrack to JavaScript-compatible structure
fn convert_track(track: ddex_core::models::flat::ParsedTrack) -> JsTrack {
    JsTrack {
        track_id: track.track_id.clone(),
        title: track.title,
        artist: track.display_artist,
        duration: track.duration_formatted,
        position: Some(track.position as u32),
        disc_number: track.disc_number.map(|d| d as u32),
        track_number: track.track_number,
        isrc: track.isrc,
        // ParsedTrack.track_id is the track's ResourceReference.
        resource_reference: Some(track.track_id),
        track_release_reference: track.track_release_reference,
        track_release_id: track.track_release_id,
        group_path: track
            .group_path
            .into_iter()
            .map(|g| JsTrackGroup {
                group_type: g.group_type,
                sequence_number: g.sequence_number,
            })
            .collect(),
        artists: track
            .artists
            .into_iter()
            .map(|a| JsArtist {
                name: a.name,
                role: a.role,
                party_reference: a.party_id,
            })
            .collect(),
        genre: track.genre,
        sub_genre: track.sub_genre,
        parental_warning: track.parental_warning,
        p_line: track.p_line.map(convert_copyright),
        c_line: track.c_line.map(convert_copyright),
        label_name: track.label_name,
        track_release_genre: track.track_release_genre,
        track_release_sub_genre: track.track_release_sub_genre,
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
        genre: resource.genre,
        sub_genre: resource.sub_genre,
        parental_warning: resource.parental_warning,
        p_line: resource.p_line.map(convert_copyright),
        c_line: resource.c_line.map(convert_copyright),
        label_name: resource.label_name,
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

/// Every parse option was removed in 0.5.0 (the streaming ones, and the ones that had no
/// effect). They are rejected instead of silently ignored.
fn reject_removed_options(options: &ParseOptions) -> Result<()> {
    let streaming = [
        ("mode", options.mode.is_some()),
        ("autoThreshold", options.auto_threshold.is_some()),
        ("enableStreaming", options.enable_streaming.is_some()),
        ("streamingThreshold", options.streaming_threshold.is_some()),
    ];
    let no_effect = [
        ("resolveReferences", options.resolve_references.is_some()),
        ("includeRaw", options.include_raw.is_some()),
        ("maxMemory", options.max_memory.is_some()),
        ("timeoutMs", options.timeout_ms.is_some()),
        ("allowBlocking", options.allow_blocking.is_some()),
        ("chunkSize", options.chunk_size.is_some()),
        ("fidelityLevel", options.fidelity_level.is_some()),
        ("preserveComments", options.preserve_comments.is_some()),
        (
            "preserveProcessingInstructions",
            options.preserve_processing_instructions.is_some(),
        ),
        ("preserveExtensions", options.preserve_extensions.is_some()),
        (
            "preserveAttributeOrder",
            options.preserve_attribute_order.is_some(),
        ),
        (
            "preserveNamespacePrefixes",
            options.preserve_namespace_prefixes.is_some(),
        ),
        ("canonicalization", options.canonicalization.is_some()),
        ("collectStatistics", options.collect_statistics.is_some()),
        ("validationLevel", options.validation_level.is_some()),
        (
            "extensionValidation",
            options.extension_validation.is_some(),
        ),
        ("enableChecksums", options.enable_checksums.is_some()),
        ("memoryLimit", options.memory_limit.is_some()),
        (
            "enableDetailedErrors",
            options.enable_detailed_errors.is_some(),
        ),
    ];
    let names = |list: &[(&'static str, bool)]| -> Vec<&'static str> {
        list.iter()
            .filter(|(_, set)| *set)
            .map(|(name, _)| *name)
            .collect()
    };
    let (streaming, no_effect) = (names(&streaming), names(&no_effect));
    let mut reasons = Vec::new();
    if !streaming.is_empty() {
        reasons.push(format!(
            "{} were removed in 0.5.0: the Node parser always uses the complete parser, whatever \
             the input size (streaming is not available in Node)",
            streaming.join(", ")
        ));
    }
    if !no_effect.is_empty() {
        reasons.push(format!(
            "{} were removed in 0.5.0: they had no effect on parsing",
            no_effect.join(", ")
        ));
    }
    if reasons.is_empty() {
        return Ok(());
    }
    Err(napi::Error::new(
        napi::Status::InvalidArg,
        format!("Parse option(s) {}. Remove them.", reasons.join("; ")),
    ))
}

/// Convert ParsedERNMessage to Node.js ParsedMessage structure
fn convert_parsed_message(parsed: ParsedERNMessage) -> ParsedMessage {
    let flat = parsed.flat; // Take ownership instead of borrowing

    // Convert the actual data structures
    let releases: Vec<JsRelease> = flat.releases.into_iter().map(convert_release).collect();
    let resources_obj = convert_resources_to_js_object(flat.resources.clone());
    let deals: Vec<JsDeal> = flat.deals.into_iter().map(convert_deal).collect();

    // Calculate counts from actual data
    let release_count = releases.len() as u32;
    let resource_count = flat.resources.len() as u32;
    let deal_count = deals.len() as u32;

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
        message_control_type: flat.message_control_type.clone(),

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
    pub display_artist: Option<String>,
    pub release_type: String,
    pub genre: Option<String>,
    pub sub_genre: Option<String>,
    pub track_count: u32,
    pub disc_count: Option<u32>,
    /// Default release date (RFC 3339, midnight UTC); only for a full `YYYY-MM-DD` date.
    pub release_date: Option<String>,
    pub original_release_date: Option<String>,
    /// 3.8.x `LabelName`; 4.x the PartyList name of the first `ReleaseLabelReference`.
    pub label_name: Option<String>,
    pub tracks: Vec<JsTrack>,
    /// `ReleaseId/ICPN`.
    pub icpn: Option<String>,
    /// `ReleaseId/ISRC` (ERN 3.8.x).
    pub isrc: Option<String>,
    /// `ReleaseId/GRid`.
    pub grid: Option<String>,
    /// `ReleaseId/ProprietaryId`s.
    pub proprietary_ids: Vec<JsProprietaryId>,
    /// Every genre in document order (`genre` / `subGenre` hold the default one).
    pub genres: Vec<JsGenre>,
    /// Default `ParentalWarningType`.
    pub parental_warning: Option<String>,
    /// Every `ParentalWarningType` value in document order.
    pub parental_warnings: Vec<String>,
    /// Default P line / C line.
    pub p_line: Option<JsCopyright>,
    pub c_line: Option<JsCopyright>,
    /// Every release date as written (`YYYY`, `YYYY-MM` or `YYYY-MM-DD`) with its territories.
    pub release_dates: Vec<JsDateInfo>,
    pub original_release_dates: Vec<JsDateInfo>,
}

#[napi(object)]
#[derive(serde::Serialize)]
pub struct JsProprietaryId {
    pub namespace: String,
    pub value: String,
}

/// Where a value applies: 4.x `ApplicableTerritoryCode` / `IsDefault`; 3.8.x the
/// territories of the DetailsByTerritory block it came from.
#[napi(object)]
#[derive(serde::Serialize)]
pub struct JsGenre {
    pub genre: String,
    pub sub_genre: Option<String>,
    pub territory_codes: Vec<String>,
    pub excluded_territory_codes: Vec<String>,
    pub is_default: Option<bool>,
}

#[napi(object)]
#[derive(serde::Serialize)]
pub struct JsDateInfo {
    /// As written: `YYYY`, `YYYY-MM` or `YYYY-MM-DD`.
    pub date: String,
    pub is_approximate: Option<bool>,
    pub territory_codes: Vec<String>,
    pub excluded_territory_codes: Vec<String>,
    pub is_default: Option<bool>,
}

/// A P line or C line.
#[napi(object)]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsCopyright {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    /// `PLineCompany` / `CLineCompany`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    pub territory_codes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
}

#[napi(object)]
#[derive(serde::Serialize)]
pub struct JsArtist {
    pub name: String,
    pub role: String,
    pub party_reference: Option<String>,
}

#[napi(object)]
#[derive(serde::Serialize)]
pub struct JsTrack {
    pub track_id: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub duration: Option<String>,
    pub position: Option<u32>,
    pub disc_number: Option<u32>,
    /// Item `SequenceNumber` as written (ERN 4.x).
    pub track_number: Option<i32>,
    pub isrc: Option<String>,
    pub resource_reference: Option<String>,
    /// ERN 4.x: `ReleaseReference` of the TrackRelease for this track (deals use it).
    pub track_release_reference: Option<String>,
    /// ERN 4.x: first identifier of that TrackRelease's `ReleaseId`.
    pub track_release_id: Option<String>,
    /// Enclosing nested ResourceGroups, outermost first.
    pub group_path: Vec<JsTrackGroup>,
    /// Display artists with their roles (3.8.x: from the details block; 4.x: via PartyList).
    pub artists: Vec<JsArtist>,
    /// The resource's default genre (3.8.x), parental warning, P/C line and label (3.8.x).
    pub genre: Option<String>,
    pub sub_genre: Option<String>,
    pub parental_warning: Option<String>,
    pub p_line: Option<JsCopyright>,
    pub c_line: Option<JsCopyright>,
    pub label_name: Option<String>,
    /// ERN 4.x: default genre of this track's TrackRelease.
    pub track_release_genre: Option<String>,
    pub track_release_sub_genre: Option<String>,
}

#[napi(object)]
#[derive(serde::Serialize)]
pub struct JsTrackGroup {
    pub group_type: Option<String>,
    pub sequence_number: Option<i32>,
}

/// A resource (the values of `ParsedMessage.resources`). It reaches JS through serde, so the
/// serde names must match the camelCase names napi gives the other objects; absent values
/// are omitted as napi does (`test-types.js` checks the keys against index.d.ts).
#[napi(object)]
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsResource {
    pub resource_id: String,
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<String>, // Convert u64 to string for JS compatibility
    /// Default genre (3.8.x), parental warning, P/C line and label name (3.8.x).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parental_warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p_line: Option<JsCopyright>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_line: Option<JsCopyright>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_name: Option<String>,
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

impl Default for DdexParser {
    fn default() -> Self {
        Self::new()
    }
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
        if let Some(opts) = &options {
            reject_removed_options(opts)?;
        }

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
                "XML input too large (>100MB). The Node parser reads the whole document into memory.",
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
                let result = convert_parsed_message(parsed_message);
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

    /// # Safety
    ///
    /// napi-rs requires `unsafe` for an async method taking `&mut self`: the caller (JS)
    /// must not call methods of the same parser concurrently with this one.
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

/// Options accepted by `parse` / `parseSync`. None are supported in 0.5.0: every field
/// is a removed option, and passing any of them throws (they used to be ignored).
#[napi(object)]
#[derive(Default)]
pub struct ParseOptions {
    /// Removed in 0.5.0: parsing always uses the complete parser, whatever the input size
    /// (the core's experimental streaming mode is not exposed in Node). Passing it throws.
    #[napi(ts_type = "never")]
    pub mode: Option<serde_json::Value>,
    /// Removed in 0.5.0 (see `mode`). Passing it throws.
    #[napi(ts_type = "never")]
    pub auto_threshold: Option<serde_json::Value>,
    /// Removed in 0.5.0 (see `mode`). Passing it throws.
    #[napi(ts_type = "never")]
    pub enable_streaming: Option<serde_json::Value>,
    /// Removed in 0.5.0 (see `mode`). Passing it throws.
    #[napi(ts_type = "never")]
    pub streaming_threshold: Option<serde_json::Value>,
    /// Removed in 0.5.0: it had no effect on parsing. Passing it throws.
    #[napi(ts_type = "never")]
    pub resolve_references: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub include_raw: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub max_memory: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub timeout_ms: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub allow_blocking: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub chunk_size: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub fidelity_level: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub preserve_comments: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub preserve_processing_instructions: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub preserve_extensions: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub preserve_attribute_order: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub preserve_namespace_prefixes: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub canonicalization: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub collect_statistics: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub validation_level: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub extension_validation: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub enable_checksums: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub memory_limit: Option<serde_json::Value>,
    /// Removed in 0.5.0 (no effect; see `resolveReferences`). Passing it throws.
    #[napi(ts_type = "never")]
    pub enable_detailed_errors: Option<serde_json::Value>,
}

#[napi(object)]
pub struct ParsedMessage {
    pub message_id: String,
    pub message_type: String,
    pub message_date: String,
    pub sender_name: Option<String>,
    pub sender_id: String,
    pub recipient_name: Option<String>,
    pub recipient_id: String,
    pub version: String,
    pub profile: Option<String>,
    /// `MessageHeader/MessageControlType` as written.
    pub message_control_type: Option<String>,

    // Counts (for backward compatibility)
    pub release_count: u32,
    pub track_count: u32,
    pub deal_count: u32,
    pub resource_count: u32,
    pub total_duration_seconds: f64,

    // CRITICAL: Actual data arrays/objects
    pub releases: Vec<JsRelease>,
    /// Resources by resource reference.
    #[napi(ts_type = "Record<string, JsResource>")]
    pub resources: serde_json::Value,
    pub deals: Vec<JsDeal>,
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
