use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;

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
    /// ERN 4.x `ParentalWarningType` (required for 4.x), e.g. "NotExplicit"
    pub parental_warning_type: Option<String>,
    pub track_ids: Vec<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub resource_id: String,
    /// "SoundRecording" or "Video"
    pub resource_type: String,
    /// ERN 4.x: the resource's DDEX `Type` (required for 4.x), e.g. "MusicalWorkSoundRecording"
    pub ddex_type: Option<String>,
    /// ERN 4.x `ParentalWarningType` (required for 4.x)
    pub parental_warning_type: Option<String>,
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

/// Options of `buildWithStatistics` (before 0.6.0, `FidelityOptions`)
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOptions {
    /// "none" or "db_c14n" (anything else is an error); the output is DB-C14N/1.0 either way
    pub canonicalization: Option<String>,
    /// Run a real Parse → Build → Parse round trip of the output (`verification`)
    pub enable_verification: Option<bool>,
    /// Count the output (`statistics`)
    pub collect_statistics: Option<bool>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStatistics {
    pub build_time_ms: f64,
    /// Not measured: always null (before 0.5.1, twice the XML size).
    pub memory_used_bytes: Option<u32>,
    pub xml_size_bytes: u32,
    /// Elements in the output.
    pub element_count: u32,
    /// Attributes in the output, namespace declarations excluded.
    pub attribute_count: u32,
    /// Namespace declarations in the output.
    pub namespace_count: u32,
    /// Not measured: always null.
    pub extension_count: Option<u32>,
    /// Not measured separately (canonicalization is part of the build): always null
    /// (before 0.5.1, a constant 2.0).
    pub canonicalization_time_ms: Option<f64>,
    pub verification_time_ms: Option<f64>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Parse → build → parse found no difference on the compared fields.
    pub round_trip_success: bool,
    /// No score is computed: always null (before 0.5.1, a made-up 0.98 or 1.0).
    pub fidelity_score: Option<f64>,
    pub canonicalization_consistent: bool,
    /// Building the same request twice gave byte-identical XML.
    pub builds_identical: bool,
    pub issues: Vec<String>,
    pub checksums_match: Option<bool>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub xml: String,
    pub statistics: Option<BuildStatistics>,
    pub verification: Option<VerificationResult>,
    pub fidelity_info: Option<FidelityInfo>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityInfo {
    pub fidelity_level: String,
    pub canonicalization_algorithm: String,
    pub comments_preserved: bool,
    pub extensions_preserved: bool,
    pub processing_instructions_preserved: bool,
    pub attribute_order_preserved: bool,
    pub namespace_prefixes_preserved: bool,
}

/// Parse with ddex-parser (the parse function the round trip needs).
fn parse_xml(xml: &str) -> std::result::Result<ddex_core::models::flat::ParsedERNMessage, String> {
    ddex_parser::DDEXParser::new()
        .parse(Cursor::new(xml.as_bytes()))
        .map_err(|e| e.to_string())
}

/// A real Parse → Build → Parse round trip of `xml` (ddex_builder::round_trip). A failure to
/// parse or build is reported as `roundTripSuccess: false` with the error as the issue.
fn round_trip(xml: &str) -> VerificationResult {
    match ddex_builder::round_trip::verify_round_trip(xml, parse_xml) {
        Ok(result) => VerificationResult {
            round_trip_success: result.success,
            fidelity_score: None,
            canonicalization_consistent: result.canonical,
            builds_identical: result.deterministic,
            issues: result.differences,
            checksums_match: Some(result.checksums_match),
        },
        Err(e) => VerificationResult {
            round_trip_success: false,
            fidelity_score: None,
            canonicalization_consistent: false,
            builds_identical: false,
            issues: vec![format!("round trip failed: {e}")],
            checksums_match: None,
        },
    }
}

/// (elements, attributes without namespace declarations, namespace declarations) of `xml`.
fn count_xml(xml: &str) -> (u32, u32, u32) {
    use quick_xml::events::Event;
    let (mut elements, mut attributes, mut namespaces) = (0, 0, 0);
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                elements += 1;
                for attr in e.attributes().flatten() {
                    let key = attr.key.as_ref();
                    if key == b"xmlns" || key.starts_with(b"xmlns:") {
                        namespaces += 1;
                    } else {
                        attributes += 1;
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            Ok(_) => {}
        }
    }
    (elements, attributes, namespaces)
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

    /// Build DDEX XML with the core builder.
    ///
    /// `request` is a build request (the core `BuildRequest` fields, snake_case; see
    /// build-request.d.ts). `header` and `version` are required. When it has no
    /// `releases`, the releases added with `addRelease`/`addResource` are used.
    #[napi(ts_args_type = "request?: import('./build-request').BuildRequest | undefined | null")]
    pub async unsafe fn build(&mut self, data: Option<serde_json::Value>) -> Result<String> {
        let start_time = std::time::Instant::now();

        let build_request = self.build_request(data)?;

        // Use the actual DDEX builder
        let builder = ddex_builder::builder::DDEXBuilder::new();
        let options = ddex_builder::builder::BuildOptions::default();

        let result = builder
            .build(build_request, options)
            .map_err(|e| Error::new(Status::Unknown, format!("Build failed: {}", e)))?;

        self.stats.last_build_size_bytes = result.xml.len() as f64;
        self.stats.total_build_time_ms += start_time.elapsed().as_millis() as f64;

        Ok(result.xml)
    }

    #[napi(
        ts_args_type = "request?: import('./build-request').BuildRequest | undefined | null, options?: BuildOptions | undefined | null"
    )]
    pub async unsafe fn build_with_statistics(
        &mut self,
        data: Option<serde_json::Value>,
        fidelity_options: Option<BuildOptions>,
    ) -> Result<BuildResult> {
        // The output is always DB-C14N/1.0; other algorithms are not implemented.
        if let Some(algorithm) = fidelity_options
            .as_ref()
            .and_then(|o| o.canonicalization.as_deref())
            .filter(|a| !matches!(*a, "none" | "db_c14n"))
        {
            return Err(Error::new(
                Status::InvalidArg,
                format!(
                    "canonicalization '{algorithm}' is not implemented in 0.5.x (only db_c14n)"
                ),
            ));
        }
        let start_time = std::time::Instant::now();

        let build_request = self.build_request(data)?;

        // Use the actual DDEX builder
        let builder = ddex_builder::builder::DDEXBuilder::new();
        let options = ddex_builder::builder::BuildOptions::default();

        let result = builder
            .build(build_request, options)
            .map_err(|e| Error::new(Status::Unknown, format!("Build failed: {}", e)))?;

        self.stats.last_build_size_bytes = result.xml.len() as f64;
        let build_time = start_time.elapsed().as_millis() as f64;
        self.stats.total_build_time_ms += build_time;

        // Statistics computed from the output; what is not measured is null.
        let statistics = if fidelity_options
            .as_ref()
            .and_then(|o| o.collect_statistics)
            .unwrap_or(false)
        {
            let (element_count, attribute_count, namespace_count) = count_xml(&result.xml);
            Some(BuildStatistics {
                build_time_ms: build_time,
                memory_used_bytes: None,
                xml_size_bytes: result.xml.len() as u32,
                element_count,
                attribute_count,
                namespace_count,
                extension_count: None,
                canonicalization_time_ms: None,
                verification_time_ms: None,
            })
        } else {
            None
        };

        // A real round trip of the built XML (before 0.5.1: constants, all passing).
        let mut statistics = statistics;
        let verification = if fidelity_options
            .as_ref()
            .and_then(|o| o.enable_verification)
            .unwrap_or(false)
        {
            let started = std::time::Instant::now();
            let verification = round_trip(&result.xml);
            if let Some(stats) = statistics.as_mut() {
                stats.verification_time_ms = Some(started.elapsed().as_secs_f64() * 1000.0);
            }
            Some(verification)
        } else {
            None
        };

        let fidelity_info = fidelity_options.as_ref().map(applied_fidelity);

        Ok(BuildResult {
            xml: result.xml,
            statistics,
            verification,
            fidelity_info,
        })
    }

    /// Parse → build → parse `originalXml` and compare the two parsed messages on the
    /// fields a build request carries (ddex_builder::round_trip). `issues` lists the
    /// differences; `fidelityScore` is null (no score is computed). Before 0.5.1 this
    /// returned a made-up success. (Before 0.6.0, `testRoundTripFidelity`.)
    #[napi]
    pub async unsafe fn check_round_trip(
        &mut self,
        original_xml: String,
    ) -> Result<VerificationResult> {
        Ok(round_trip(&original_xml))
    }

    /// Runs the build's preflight validation on the added releases and resources (before
    /// 0.5.1 it only checked that a release had been added).
    #[napi]
    pub async unsafe fn validate(&mut self) -> Result<ValidationResult> {
        let builder = ddex_builder::builder::DDEXBuilder::new();
        let built = self
            .build_request(None)
            .map_err(|e| e.reason)
            .and_then(|request| {
                builder
                    .build(request, ddex_builder::builder::BuildOptions::default())
                    .map_err(|e| e.to_string())
            });
        let result = match built {
            Ok(built) => ValidationResult {
                is_valid: true,
                errors: vec![],
                warnings: built.warnings.iter().map(|w| w.message.clone()).collect(),
            },
            Err(e) => ValidationResult {
                is_valid: false,
                errors: vec![e],
                warnings: vec![],
            },
        };
        self.stats.validation_errors = result.errors.len() as u32;
        self.stats.validation_warnings = result.warnings.len() as u32;
        Ok(result)
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

    /// The core build request: `data` as given (missing `releases`/`deals` mean none), with
    /// the stored releases when `data` has no releases. Nothing is filled in: a missing
    /// value is reported by the core builder like any other missing value.
    fn build_request(
        &self,
        data: Option<serde_json::Value>,
    ) -> Result<ddex_builder::builder::BuildRequest> {
        let mut value = data.unwrap_or_else(|| serde_json::json!({}));
        let object = value
            .as_object_mut()
            .ok_or_else(|| Error::new(Status::InvalidArg, "the build request must be an object"))?;
        object
            .entry("releases")
            .or_insert_with(|| serde_json::json!([]));
        object
            .entry("deals")
            .or_insert_with(|| serde_json::json!([]));
        let mut request: ddex_builder::builder::BuildRequest = serde_json::from_value(value)
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid build request: {}", e)))?;
        if request.releases.is_empty() {
            request.releases = self.stored_releases()?;
        }
        Ok(request)
    }

    /// Releases added with `addRelease`/`addResource`, using only the values given.
    fn stored_releases(&self) -> Result<Vec<ddex_builder::builder::ReleaseRequest>> {
        use ddex_builder::builder::{
            LocalizedStringRequest, ReleaseRequest, ResourceKind, TrackRequest,
        };
        let mut releases = Vec::new();
        for release in &self.releases {
            let mut tracks = Vec::new();
            for track_id in &release.track_ids {
                let resource = self
                    .resources
                    .iter()
                    .find(|r| &r.resource_id == track_id)
                    .ok_or_else(|| {
                        Error::new(
                            Status::InvalidArg,
                            format!(
                                "release {} lists track {} but no resource with that id was added",
                                release.release_id, track_id
                            ),
                        )
                    })?;
                let resource_kind = match resource.resource_type.as_str() {
                    "SoundRecording" => ResourceKind::SoundRecording,
                    "Video" => ResourceKind::Video,
                    other => {
                        return Err(Error::new(
                            Status::InvalidArg,
                            format!(
                                "resource {}: resourceType must be \"SoundRecording\" or \"Video\", got {:?}",
                                resource.resource_id, other
                            ),
                        ))
                    }
                };
                tracks.push(TrackRequest {
                    track_id: resource.resource_id.clone(),
                    resource_reference: Some(resource.resource_id.clone()),
                    isrc: resource.isrc.clone(),
                    title: resource.title.clone(),
                    duration: resource.duration.clone().unwrap_or_default(),
                    artist: Some(resource.artist.clone()),
                    resource_kind,
                    resource_type: resource.ddex_type.clone().map(Into::into),
                    parental_warning_type: resource.parental_warning_type.clone(),
                    disc_number: resource.volume_number.and_then(|n| u32::try_from(n).ok()),
                    sequence_number: resource.track_number.and_then(|n| u32::try_from(n).ok()),
                    ..Default::default()
                });
            }
            releases.push(ReleaseRequest {
                release_id: Some(release.release_id.clone()),
                // Generated by the core's ID strategy (a release ID such as a GRid is not a
                // valid ERN 4.x ReleaseReference)
                release_reference: None,
                title: vec![LocalizedStringRequest {
                    text: release.title.clone(),
                    language_code: None,
                }],
                artist: Some(release.artist.clone()),
                label: release.label.clone(),
                release_date: release.release_date.clone(),
                upc: release.upc.clone(),
                tracks,
                resource_references: None,
                release_type: Some(release.release_type.clone().into()),
                genre: release.genre.clone(),
                parental_warning_type: release.parental_warning_type.clone(),
                ..Default::default()
            });
        }
        Ok(releases)
    }
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    pub max_buffer_size: u32,
    pub progress_callback_frequency: u32,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingProgress {
    pub releases_written: u32,
    pub resources_written: u32,
    pub bytes_written: u32,
    pub current_memory_usage: u32,
    pub estimated_completion_percent: Option<f64>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingStats {
    pub releases_written: u32,
    pub resources_written: u32,
    pub bytes_written: u32,
    pub peak_memory_usage: u32,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Required (the writer never generates it); missing fails `startMessage` naming the field
    #[napi(ts_type = "string")]
    pub message_id: Option<String>,
    pub message_sender_name: String,
    pub message_recipient_name: String,
    /// Required xs:dateTime (the writer reads no clock); missing fails `startMessage` naming
    /// the field
    #[napi(ts_type = "string")]
    pub message_created_date_time: Option<String>,
}

#[napi]
pub struct StreamingDdexBuilder {
    inner: Option<ddex_builder::streaming::StreamingBuilder<Cursor<Vec<u8>>>>,
    buffer: Cursor<Vec<u8>>,
    config: StreamingConfig,
    progress_callback: Option<napi::threadsafe_function::ThreadsafeFunction<StreamingProgress>>,
}

#[napi]
impl StreamingDdexBuilder {
    #[napi(constructor)]
    pub fn new(config: Option<StreamingConfig>) -> Result<Self> {
        let config = config.unwrap_or(StreamingConfig {
            max_buffer_size: 10 * 1024 * 1024, // 10MB
            progress_callback_frequency: 100,
        });

        let buffer = Cursor::new(Vec::new());

        Ok(StreamingDdexBuilder {
            inner: None,
            buffer,
            config,
            progress_callback: None,
        })
    }

    #[napi]
    pub fn set_progress_callback(&mut self, callback: napi::JsFunction) -> Result<()> {
        let tsfn: napi::threadsafe_function::ThreadsafeFunction<StreamingProgress> =
            callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

        self.progress_callback = Some(tsfn);
        Ok(())
    }

    #[napi]
    pub fn set_estimated_total(&mut self, total: u32) -> Result<()> {
        if let Some(ref mut builder) = self.inner {
            builder.set_estimated_total(total as usize);
        }
        Ok(())
    }

    #[napi]
    pub fn start_message(&mut self, header: MessageHeader, version: String) -> Result<()> {
        // Create a new buffer and streaming builder
        self.buffer = Cursor::new(Vec::new());

        // Convert config to Rust types
        let rust_config = ddex_builder::streaming::StreamingConfig {
            max_buffer_size: self.config.max_buffer_size as usize,
            progress_callback_frequency: self.config.progress_callback_frequency as usize,
        };

        let mut streaming_builder = ddex_builder::streaming::StreamingBuilder::new_with_config(
            std::mem::replace(&mut self.buffer, Cursor::new(Vec::new())),
            rust_config,
        )
        .map_err(|e| {
            Error::new(
                Status::Unknown,
                format!("Failed to create streaming builder: {}", e),
            )
        })?;

        // Set up progress callback if provided
        if let Some(ref callback) = self.progress_callback {
            let callback_clone = callback.clone();
            streaming_builder.set_progress_callback(Box::new(
                move |progress: ddex_builder::streaming::StreamingProgress| {
                    let js_progress = StreamingProgress {
                        releases_written: progress.releases_written as u32,
                        resources_written: progress.resources_written as u32,
                        bytes_written: progress.bytes_written as u32,
                        current_memory_usage: progress.current_memory_usage as u32,
                        estimated_completion_percent: progress.estimated_completion_percent,
                    };

                    let _ = callback_clone.call(
                        Ok(js_progress),
                        napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
                    );
                },
            ));
        }

        // Convert header to Rust type
        let rust_header = ddex_builder::builder::MessageHeaderRequest {
            // A missing value becomes "", which start_message reports, naming the field.
            message_id: header.message_id.unwrap_or_default(),
            message_sender: ddex_builder::builder::PartyRequest {
                party_name: vec![ddex_builder::builder::LocalizedStringRequest {
                    text: header.message_sender_name,
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_recipient: ddex_builder::builder::PartyRequest {
                party_name: vec![ddex_builder::builder::LocalizedStringRequest {
                    text: header.message_recipient_name,
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_control_type: None,
            message_created_date_time: header.message_created_date_time.unwrap_or_default(),
        };

        streaming_builder
            .start_message(&rust_header, &version)
            .map_err(|e| Error::new(Status::Unknown, format!("Failed to start message: {}", e)))?;

        self.inner = Some(streaming_builder);
        Ok(())
    }

    #[napi]
    pub fn write_resource(
        &mut self,
        resource_id: String,
        title: String,
        artist: String,
        isrc: Option<String>,
        duration: Option<String>,
        file_path: Option<String>,
    ) -> Result<String> {
        let builder = self.inner.as_mut().ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                "Message not started. Call start_message first.",
            )
        })?;

        builder
            .write_resource(
                &resource_id,
                &title,
                &artist,
                isrc.as_deref(),
                duration.as_deref(),
                file_path.as_deref(),
            )
            .map_err(|e| Error::new(Status::Unknown, format!("Failed to write resource: {}", e)))
    }

    #[napi]
    pub fn finish_resources_start_releases(&mut self) -> Result<()> {
        let builder = self.inner.as_mut().ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                "Message not started. Call start_message first.",
            )
        })?;

        builder.finish_resources_start_releases().map_err(|e| {
            Error::new(
                Status::Unknown,
                format!("Failed to transition to releases: {}", e),
            )
        })
    }

    #[napi]
    pub fn write_release(
        &mut self,
        release_id: String,
        title: String,
        artist: String,
        label: Option<String>,
        upc: Option<String>,
        release_date: Option<String>,
        genre: Option<String>,
        resource_references: Vec<String>,
    ) -> Result<String> {
        let builder = self.inner.as_mut().ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                "Message not started. Call start_message first.",
            )
        })?;

        builder
            .write_release(
                &release_id,
                &title,
                &artist,
                label.as_deref(),
                upc.as_deref(),
                release_date.as_deref(),
                genre.as_deref(),
                &resource_references,
            )
            .map_err(|e| Error::new(Status::Unknown, format!("Failed to write release: {}", e)))
    }

    #[napi]
    pub fn finish_message(&mut self) -> Result<StreamingStats> {
        let mut builder = self.inner.take().ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                "Message not started. Call start_message first.",
            )
        })?;

        let stats = builder
            .finish_message()
            .map_err(|e| Error::new(Status::Unknown, format!("Failed to finish message: {}", e)))?;
        // Keep the written document for get_xml (before 0.5.1 it was dropped with the
        // builder, so get_xml returned "").
        self.buffer = builder
            .into_inner()
            .map_err(|e| Error::new(Status::Unknown, format!("Failed to finish message: {}", e)))?;

        Ok(StreamingStats {
            releases_written: stats.releases_written as u32,
            resources_written: stats.resources_written as u32,
            bytes_written: stats.bytes_written as u32,
            peak_memory_usage: stats.peak_memory_usage as u32,
        })
    }

    #[napi]
    pub fn get_xml(&mut self) -> Result<String> {
        if self.inner.is_some() {
            return Err(Error::new(
                Status::InvalidArg,
                "Message not finished. Call finish_message first.",
            ));
        }

        // Retrieve the cursor from the completed builder
        let data = self.buffer.get_ref();
        String::from_utf8(data.clone()).map_err(|e| {
            Error::new(
                Status::Unknown,
                format!("Failed to convert to UTF-8: {}", e),
            )
        })
    }

    #[napi]
    pub fn reset(&mut self) -> Result<()> {
        self.inner = None;
        self.buffer = Cursor::new(Vec::new());
        Ok(())
    }
}

#[napi]
pub async fn validate_structure(xml: String) -> Result<ValidationResult> {
    // Reads the whole document (before 0.5.1, only its first event).
    let errors: Vec<String> = structure_error(&xml).into_iter().collect();
    Ok(ValidationResult {
        is_valid: errors.is_empty(),
        errors,
        warnings: vec![],
    })
}

/// The first problem that makes `xml` not well-formed: a reference to an undefined entity
/// (`&foo;`, there is no DTD), with its line and column, or a syntax error anywhere in the
/// document.
fn structure_error(xml: &str) -> Option<String> {
    if let Err(entity) = ddex_core::xml_refs::check_document(xml.as_bytes()) {
        return Some(format!("XML parsing error: {entity}"));
    }
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Eof) => return None,
            Ok(_) => {}
            Err(e) => return Some(format!("XML parsing error: {e}")),
        }
    }
}

/// What `buildWithStatistics` applied: DB-C14N/1.0 canonical output; the preservation
/// options are not applied in 0.5.x (before 0.5.1 the options were echoed back as if they
/// had been).
fn applied_fidelity(_options: &BuildOptions) -> FidelityInfo {
    FidelityInfo {
        fidelity_level: "not applied in 0.5.x".to_string(),
        canonicalization_algorithm: "db_c14n".to_string(),
        comments_preserved: false,
        extensions_preserved: false,
        processing_instructions_preserved: false,
        attribute_order_preserved: false,
        namespace_prefixes_preserved: false,
    }
}
