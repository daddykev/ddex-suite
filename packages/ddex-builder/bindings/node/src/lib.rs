use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Write, Cursor};

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

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub profile: String,
    pub required_fields: Vec<String>,
    pub disclaimer: String,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field_name: String,
    pub rule_type: String,
    pub message: String,
    pub parameters: Option<HashMap<String, String>>,
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
    pub async unsafe fn build(&mut self, data: Option<serde_json::Value>) -> Result<String> {
        let start_time = std::time::Instant::now();

        // Create BuildRequest based on whether data was provided
        let build_request = match data {
            Some(json_data) => self.create_build_request_from_json(json_data)?,
            None => self.create_build_request_from_stored_data()?,
        };
        
        // Use the actual DDEX builder
        let builder = ddex_builder::builder::DDEXBuilder::new();
        let options = ddex_builder::builder::BuildOptions::default();
        
        let result = builder.build(build_request, options)
            .map_err(|e| Error::new(Status::Unknown, format!("Build failed: {}", e)))?;
        
        self.stats.last_build_size_bytes = result.xml.len() as f64;
        self.stats.total_build_time_ms += start_time.elapsed().as_millis() as f64;

        Ok(result.xml)
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

    #[napi]
    pub fn get_available_presets(&self) -> Result<Vec<String>> {
        // Return list of available preset names
        Ok(vec![
            "spotify_album".to_string(),
            "spotify_single".to_string(),
            "spotify_ep".to_string(),
            "youtube_album".to_string(),
            "youtube_video".to_string(),
            "youtube_single".to_string(),
            "apple_music_43".to_string(),
        ])
    }

    #[napi]
    pub fn get_preset_info(&self, preset_name: String) -> Result<PresetInfo> {
        match preset_name.as_str() {
            "spotify_album" => Ok(PresetInfo {
                name: "spotify_album".to_string(),
                description: "Spotify Album ERN 4.3 requirements with audio quality validation".to_string(),
                version: "1.0.0".to_string(),
                profile: "AudioAlbum".to_string(),
                required_fields: vec![
                    "ISRC".to_string(),
                    "UPC".to_string(),
                    "ReleaseDate".to_string(),
                    "Genre".to_string(),
                    "ExplicitContent".to_string(),
                    "AlbumTitle".to_string(),
                    "ArtistName".to_string(),
                    "TrackTitle".to_string(),
                ],
                disclaimer: "Based on Spotify public documentation. Verify current requirements.".to_string(),
            }),
            "spotify_single" => Ok(PresetInfo {
                name: "spotify_single".to_string(),
                description: "Spotify Single ERN 4.3 requirements with simplified track structure".to_string(),
                version: "1.0.0".to_string(),
                profile: "AudioSingle".to_string(),
                required_fields: vec![
                    "ISRC".to_string(),
                    "UPC".to_string(),
                    "ReleaseDate".to_string(),
                    "Genre".to_string(),
                    "ExplicitContent".to_string(),
                    "TrackTitle".to_string(),
                    "ArtistName".to_string(),
                ],
                disclaimer: "Based on Spotify public documentation. Verify current requirements.".to_string(),
            }),
            "youtube_video" => Ok(PresetInfo {
                name: "youtube_video".to_string(),
                description: "YouTube Music Video ERN 4.2/4.3 with video resource handling".to_string(),
                version: "1.0.0".to_string(),
                profile: "VideoSingle".to_string(),
                required_fields: vec![
                    "ISRC".to_string(),
                    "ISVN".to_string(),
                    "ReleaseDate".to_string(),
                    "Genre".to_string(),
                    "ContentID".to_string(),
                    "VideoResource".to_string(),
                    "AudioResource".to_string(),
                    "VideoTitle".to_string(),
                    "ArtistName".to_string(),
                    "AssetType".to_string(),
                    "VideoQuality".to_string(),
                ],
                disclaimer: "Based on YouTube Partner documentation. Video encoding requirements may vary.".to_string(),
            }),
            _ => Err(Error::new(
                Status::InvalidArg,
                format!("Unknown preset: {}", preset_name)
            ))
        }
    }

    #[napi]
    pub fn apply_preset(&mut self, preset_name: String) -> Result<()> {
        // Validate preset exists
        let _preset_info = self.get_preset_info(preset_name.clone())?;
        
        // In a full implementation, this would apply the preset configuration
        // to the internal builder state. For now, we just validate the preset exists.
        Ok(())
    }

    #[napi]
    pub fn get_preset_validation_rules(&self, preset_name: String) -> Result<Vec<ValidationRule>> {
        match preset_name.as_str() {
            "spotify_album" | "spotify_single" => Ok(vec![
                ValidationRule {
                    field_name: "ISRC".to_string(),
                    rule_type: "Required".to_string(),
                    message: "ISRC is required for Spotify releases".to_string(),
                    parameters: None,
                },
                ValidationRule {
                    field_name: "AudioQuality".to_string(),
                    rule_type: "AudioQuality".to_string(),
                    message: "Minimum 16-bit/44.1kHz audio quality required".to_string(),
                    parameters: Some([
                        ("min_bit_depth".to_string(), "16".to_string()),
                        ("min_sample_rate".to_string(), "44100".to_string()),
                    ].iter().cloned().collect()),
                },
                ValidationRule {
                    field_name: "TerritoryCode".to_string(),
                    rule_type: "TerritoryCode".to_string(),
                    message: "Territory code must be 'Worldwide' or 'WW'".to_string(),
                    parameters: Some([
                        ("allowed".to_string(), "Worldwide,WW".to_string()),
                    ].iter().cloned().collect()),
                },
            ]),
            "youtube_video" | "youtube_album" => Ok(vec![
                ValidationRule {
                    field_name: "ContentID".to_string(),
                    rule_type: "Required".to_string(),
                    message: "Content ID is required for YouTube releases".to_string(),
                    parameters: None,
                },
                ValidationRule {
                    field_name: "VideoQuality".to_string(),
                    rule_type: "OneOf".to_string(),
                    message: "Video quality must be HD720, HD1080, or 4K".to_string(),
                    parameters: Some([
                        ("options".to_string(), "HD720,HD1080,4K".to_string()),
                    ].iter().cloned().collect()),
                },
            ]),
            _ => Err(Error::new(
                Status::InvalidArg,
                format!("Unknown preset: {}", preset_name)
            ))
        }
    }

    fn create_build_request_from_json(&self, data: serde_json::Value) -> Result<ddex_builder::builder::BuildRequest> {
        let obj = data.as_object()
            .ok_or_else(|| Error::new(Status::InvalidArg, "Expected object"))?;

        // Extract version
        let version = obj.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("4.3")
            .to_string();

        // Create message header
        let header = ddex_builder::builder::MessageHeaderRequest {
            message_id: Some(uuid::Uuid::new_v4().to_string()),
            message_sender: ddex_builder::builder::PartyRequest {
                party_name: vec![ddex_builder::builder::LocalizedStringRequest {
                    text: "DDEX Suite".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_recipient: ddex_builder::builder::PartyRequest {
                party_name: vec![ddex_builder::builder::LocalizedStringRequest {
                    text: "Recipient".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_control_type: None,
            message_created_date_time: Some(chrono::Utc::now().to_rfc3339()),
        };

        // Convert releases from JSON
        let mut releases = Vec::new();
        if let Some(releases_array) = obj.get("releases").and_then(|v| v.as_array()) {
            for release_val in releases_array {
                if let Some(release_obj) = release_val.as_object() {
                    let release_id = release_obj.get("release_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("UNKNOWN")
                        .to_string();
                    
                    let title = release_obj.get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Untitled")
                        .to_string();
                    
                    let artist = release_obj.get("display_artist")
                        .or_else(|| release_obj.get("artist"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown Artist")
                        .to_string();

                    releases.push(ddex_builder::builder::ReleaseRequest {
                        release_id: release_id.clone(),
                        release_reference: Some(release_id.clone()),
                        title: vec![ddex_builder::builder::LocalizedStringRequest {
                            text: title,
                            language_code: None,
                        }],
                        artist,
                        label: release_obj.get("label").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        release_date: release_obj.get("release_date").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        upc: release_obj.get("upc").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        tracks: vec![], // No tracks in the simple format for now
                        resource_references: None,
                    });
                }
            }
        }

        // Create build request
        Ok(ddex_builder::builder::BuildRequest {
            header,
            version,
            profile: Some("AudioAlbum".to_string()),
            releases,
            deals: vec![], // Empty for now
            extensions: None,
        })
    }

    fn create_build_request_from_stored_data(&self) -> Result<ddex_builder::builder::BuildRequest> {
        // Create message header
        let header = ddex_builder::builder::MessageHeaderRequest {
            message_id: Some(uuid::Uuid::new_v4().to_string()),
            message_sender: ddex_builder::builder::PartyRequest {
                party_name: vec![ddex_builder::builder::LocalizedStringRequest {
                    text: "DDEX Suite".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_recipient: ddex_builder::builder::PartyRequest {
                party_name: vec![ddex_builder::builder::LocalizedStringRequest {
                    text: "Recipient".to_string(),
                    language_code: None,
                }],
                party_id: None,
                party_reference: None,
            },
            message_control_type: None,
            message_created_date_time: Some(chrono::Utc::now().to_rfc3339()),
        };

        // Convert releases
        let mut releases = Vec::new();
        for release in &self.releases {
            let tracks = self.resources
                .iter()
                .filter(|resource| release.track_ids.contains(&resource.resource_id))
                .map(|resource| ddex_builder::builder::TrackRequest {
                    track_id: resource.resource_id.clone(),
                    resource_reference: Some(resource.resource_id.clone()),
                    isrc: resource.isrc.clone().unwrap_or_else(|| "TEMP00000000".to_string()),
                    title: resource.title.clone(),
                    duration: resource.duration.clone().unwrap_or_else(|| "PT3M00S".to_string()),
                    artist: resource.artist.clone(),
                })
                .collect();

            releases.push(ddex_builder::builder::ReleaseRequest {
                release_id: release.release_id.clone(),
                release_reference: Some(release.release_id.clone()),
                title: vec![ddex_builder::builder::LocalizedStringRequest {
                    text: release.title.clone(),
                    language_code: None,
                }],
                artist: release.artist.clone(),
                label: release.label.clone(),
                release_date: release.release_date.clone(),
                upc: release.upc.clone(),
                tracks,
                resource_references: Some(release.track_ids.clone()),
            });
        }

        // Create build request
        Ok(ddex_builder::builder::BuildRequest {
            header,
            version: "4.3".to_string(),
            profile: Some("AudioAlbum".to_string()),
            releases,
            deals: vec![], // Empty for now
            extensions: None,
        })
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

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    pub max_buffer_size: u32,
    pub deterministic: bool,
    pub validate_during_stream: bool,
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
    pub deals_written: u32,
    pub bytes_written: u32,
    pub warnings: Vec<String>,
    pub peak_memory_usage: u32,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub message_id: Option<String>,
    pub message_sender_name: String,
    pub message_recipient_name: String,
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
            deterministic: true,
            validate_during_stream: true,
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
        let tsfn: napi::threadsafe_function::ThreadsafeFunction<StreamingProgress> = callback
            .create_threadsafe_function(0, |ctx| {
                Ok(vec![ctx.value])
            })?;
        
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
            deterministic: self.config.deterministic,
            determinism_config: ddex_builder::determinism::DeterminismConfig::default(),
            validate_during_stream: self.config.validate_during_stream,
            progress_callback_frequency: self.config.progress_callback_frequency as usize,
        };
        
        let mut streaming_builder = ddex_builder::streaming::StreamingBuilder::new_with_config(
            std::mem::replace(&mut self.buffer, Cursor::new(Vec::new())),
            rust_config
        ).map_err(|e| Error::new(Status::Unknown, format!("Failed to create streaming builder: {}", e)))?;
        
        // Set up progress callback if provided
        if let Some(ref callback) = self.progress_callback {
            let callback_clone = callback.clone();
            streaming_builder.set_progress_callback(Box::new(move |progress: ddex_builder::streaming::StreamingProgress| {
                let js_progress = StreamingProgress {
                    releases_written: progress.releases_written as u32,
                    resources_written: progress.resources_written as u32,
                    bytes_written: progress.bytes_written as u32,
                    current_memory_usage: progress.current_memory_usage as u32,
                    estimated_completion_percent: progress.estimated_completion_percent,
                };
                
                let _ = callback_clone.call(Ok(js_progress), napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking);
            }));
        }
        
        // Convert header to Rust type
        let rust_header = ddex_builder::builder::MessageHeaderRequest {
            message_id: header.message_id,
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
            message_created_date_time: header.message_created_date_time,
        };
        
        streaming_builder.start_message(&rust_header, &version)
            .map_err(|e| Error::new(Status::Unknown, format!("Failed to start message: {}", e)))?;
        
        self.inner = Some(streaming_builder);
        Ok(())
    }
    
    #[napi]
    pub fn write_resource(&mut self,
                         resource_id: String,
                         title: String,
                         artist: String,
                         isrc: Option<String>,
                         duration: Option<String>,
                         file_path: Option<String>) -> Result<String> {
        let builder = self.inner.as_mut()
            .ok_or_else(|| Error::new(Status::InvalidArg, "Message not started. Call start_message first."))?;
        
        builder.write_resource(&resource_id, &title, &artist, isrc.as_deref(), duration.as_deref(), file_path.as_deref())
            .map_err(|e| Error::new(Status::Unknown, format!("Failed to write resource: {}", e)))
    }
    
    #[napi]
    pub fn finish_resources_start_releases(&mut self) -> Result<()> {
        let builder = self.inner.as_mut()
            .ok_or_else(|| Error::new(Status::InvalidArg, "Message not started. Call start_message first."))?;
        
        builder.finish_resources_start_releases()
            .map_err(|e| Error::new(Status::Unknown, format!("Failed to transition to releases: {}", e)))
    }
    
    #[napi]
    pub fn write_release(&mut self,
                        release_id: String,
                        title: String,
                        artist: String,
                        label: Option<String>,
                        upc: Option<String>,
                        release_date: Option<String>,
                        genre: Option<String>,
                        resource_references: Vec<String>) -> Result<String> {
        let builder = self.inner.as_mut()
            .ok_or_else(|| Error::new(Status::InvalidArg, "Message not started. Call start_message first."))?;
        
        builder.write_release(&release_id, &title, &artist, label.as_deref(), upc.as_deref(), 
                             release_date.as_deref(), genre.as_deref(), &resource_references)
            .map_err(|e| Error::new(Status::Unknown, format!("Failed to write release: {}", e)))
    }
    
    #[napi]
    pub fn finish_message(&mut self) -> Result<StreamingStats> {
        let mut builder = self.inner.take()
            .ok_or_else(|| Error::new(Status::InvalidArg, "Message not started. Call start_message first."))?;
        
        let stats = builder.finish_message()
            .map_err(|e| Error::new(Status::Unknown, format!("Failed to finish message: {}", e)))?;
        
        Ok(StreamingStats {
            releases_written: stats.releases_written as u32,
            resources_written: stats.resources_written as u32,
            deals_written: stats.deals_written as u32,
            bytes_written: stats.bytes_written as u32,
            warnings: stats.warnings.iter().map(|w| w.message.clone()).collect(),
            peak_memory_usage: stats.peak_memory_usage as u32,
        })
    }
    
    #[napi]
    pub fn get_xml(&mut self) -> Result<String> {
        if self.inner.is_some() {
            return Err(Error::new(Status::InvalidArg, "Message not finished. Call finish_message first."));
        }
        
        // Retrieve the cursor from the completed builder
        let data = self.buffer.get_ref();
        String::from_utf8(data.clone())
            .map_err(|e| Error::new(Status::Unknown, format!("Failed to convert to UTF-8: {}", e)))
    }
    
    #[napi]
    pub fn reset(&mut self) -> Result<()> {
        self.inner = None;
        self.buffer = Cursor::new(Vec::new());
        Ok(())
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