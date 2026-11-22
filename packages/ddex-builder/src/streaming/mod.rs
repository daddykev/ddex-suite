//! Streaming DDEX XML builder for large catalogs
//!
//! This module provides a memory-efficient streaming approach to building DDEX XML
//! that can handle catalogs with thousands of releases without loading everything
//! into memory at once.

pub mod buffer_manager;
pub mod reference_manager;

use crate::builder::MessageHeaderRequest;
use crate::determinism::DeterminismConfig;
use crate::error::{BuildError, BuildWarning};
use buffer_manager::BufferManager;
use reference_manager::StreamingReferenceManager;
use std::io::Write as IoWrite;
use uuid::Uuid;

/// Configuration for streaming builder
///
/// Controls the behavior of the streaming DDEX XML builder including
/// buffer management, deterministic ordering, validation, and progress reporting.
///
/// # Example
/// ```
/// use ddex_builder::streaming::StreamingConfig;
/// use ddex_builder::determinism::DeterminismConfig;
///
/// let config = StreamingConfig {
///     max_buffer_size: 5 * 1024 * 1024, // 5MB buffer
///     deterministic: true,
///     determinism_config: DeterminismConfig::default(),
///     validate_during_stream: true,
///     progress_callback_frequency: 50, // Report every 50 items
/// };
/// ```
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Maximum buffer size in bytes before automatic flush (default: 10MB)
    pub max_buffer_size: usize,
    /// Whether to use deterministic ordering for consistent output
    pub deterministic: bool,
    /// Detailed configuration for deterministic behavior
    pub determinism_config: DeterminismConfig,
    /// Whether to validate data while streaming (slower but safer)
    pub validate_during_stream: bool,
    /// Progress callback frequency - report progress every N items processed
    pub progress_callback_frequency: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 10 * 1024 * 1024, // 10MB
            deterministic: true,
            determinism_config: DeterminismConfig::default(),
            validate_during_stream: true,
            progress_callback_frequency: 100,
        }
    }
}

/// Progress information for streaming operations
///
/// Provides real-time progress updates during streaming DDEX XML generation.
/// Used by progress callbacks to monitor the build process and estimate completion.
///
/// # Example
/// ```
/// use ddex_builder::streaming::{StreamingBuilder, StreamingProgress};
///
/// let output_writer: Vec<u8> = Vec::new();
/// let mut builder = StreamingBuilder::new(output_writer)?;
/// builder.set_progress_callback(Box::new(|progress: StreamingProgress| {
///     println!("Progress: {} items ({:.0}%), {} MB written",
///              progress.releases_written + progress.resources_written,
///              progress.estimated_completion_percent.unwrap_or(0.0),
///              progress.bytes_written / 1024 / 1024);
/// }));
/// # Ok::<(), ddex_builder::BuildError>(())
/// ```
#[derive(Debug, Clone)]
pub struct StreamingProgress {
    /// Number of releases written to the stream so far
    pub releases_written: usize,
    /// Number of resources (tracks/recordings) written to the stream so far
    pub resources_written: usize,
    /// Total bytes written to the output stream so far
    pub bytes_written: usize,
    /// Current memory usage in bytes (includes buffers)
    pub current_memory_usage: usize,
    /// Estimated completion percentage (0.0-100.0) if total items was set
    pub estimated_completion_percent: Option<f64>,
}

/// Callback type for progress updates
pub type ProgressCallback = Box<dyn Fn(StreamingProgress) + Send + Sync>;

/// Streaming DDEX XML builder
pub struct StreamingBuilder<W: IoWrite> {
    buffer_manager: BufferManager<W>,
    reference_manager: StreamingReferenceManager,
    config: StreamingConfig,
    xml_buffer: Vec<u8>,

    // State tracking
    message_started: bool,
    message_finished: bool,
    releases_written: usize,
    resources_written: usize,
    deals_written: usize,
    warnings: Vec<BuildWarning>,

    // Progress tracking
    progress_callback: Option<ProgressCallback>,
    estimated_total_items: Option<usize>,
}

impl<W: IoWrite> StreamingBuilder<W> {
    /// Create a new streaming builder with the given writer
    pub fn new(writer: W) -> Result<Self, BuildError> {
        Self::new_with_config(writer, StreamingConfig::default())
    }

    /// Create a new streaming builder with custom configuration
    pub fn new_with_config(writer: W, config: StreamingConfig) -> Result<Self, BuildError> {
        let buffer_manager = BufferManager::new(writer, config.max_buffer_size).map_err(|e| {
            BuildError::XmlGeneration(format!("Failed to create buffer manager: {}", e))
        })?;

        Ok(StreamingBuilder {
            buffer_manager,
            reference_manager: StreamingReferenceManager::new(),
            config,
            xml_buffer: Vec::new(),
            message_started: false,
            message_finished: false,
            releases_written: 0,
            resources_written: 0,
            deals_written: 0,
            warnings: Vec::new(),
            progress_callback: None,
            estimated_total_items: None,
        })
    }

    /// Set a progress callback function
    pub fn set_progress_callback(&mut self, callback: ProgressCallback) {
        self.progress_callback = Some(callback);
    }

    /// Set estimated total number of items for progress calculation
    pub fn set_estimated_total(&mut self, total: usize) {
        self.estimated_total_items = Some(total);
    }

    /// Start the DDEX message with header information
    pub fn start_message(
        &mut self,
        header: &MessageHeaderRequest,
        version: &str,
    ) -> Result<(), BuildError> {
        if self.message_started {
            return Err(BuildError::XmlGeneration(
                "Message already started".to_string(),
            ));
        }

        // The streaming writer only knows the 3.8-style tree; schema-valid ERN 4.x output
        // comes from DDEXBuilder::build.
        use ddex_core::models::versions::ERNVersion;
        if matches!(
            crate::builder::parse_target_version(version),
            Ok(ERNVersion::V4_2 | ERNVersion::V4_3)
        ) {
            return Err(BuildError::InvalidFormat {
                field: "version".to_string(),
                message: format!(
                    "ERN {} is not yet supported by the streaming writer; use DDEXBuilder::build \
                     for ERN 4.2/4.3 output",
                    version
                ),
            });
        }

        // Write XML declaration and start of message
        let xml_start = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="{}" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
"#,
            version
        );

        self.xml_buffer.extend_from_slice(xml_start.as_bytes());

        // Write message header
        self.write_message_header(header)?;

        // Start resource list container
        self.xml_buffer.extend_from_slice(b"  <ResourceList>\n");

        self.message_started = true;
        self.flush_if_needed()?;

        Ok(())
    }

    /// Write a single resource to the stream
    pub fn write_resource(
        &mut self,
        resource_id: &str,
        title: &str,
        artist: &str,
        isrc: Option<&str>,
        duration: Option<&str>,
        file_path: Option<&str>,
    ) -> Result<String, BuildError> {
        if !self.message_started || self.message_finished {
            return Err(BuildError::XmlGeneration(
                "Message not in valid state for writing resources".to_string(),
            ));
        }

        // Generate stable reference for this resource
        let resource_ref = self
            .reference_manager
            .generate_resource_reference(resource_id)?;

        // Build SoundRecording XML
        let mut resource_xml = String::new();
        resource_xml.push_str("    <SoundRecording>\n");
        resource_xml.push_str(&format!(
            "      <ResourceReference>{}</ResourceReference>\n",
            resource_ref
        ));
        resource_xml.push_str("      <Type>SoundRecording</Type>\n");
        resource_xml.push_str(&format!(
            "      <ResourceId>{}</ResourceId>\n",
            escape_xml(resource_id)
        ));
        resource_xml.push_str(&format!(
            "      <ReferenceTitle>{}</ReferenceTitle>\n",
            escape_xml(title)
        ));
        resource_xml.push_str(&format!(
            "      <DisplayArtist>{}</DisplayArtist>\n",
            escape_xml(artist)
        ));

        if let Some(isrc_value) = isrc {
            resource_xml.push_str(&format!("      <ISRC>{}</ISRC>\n", escape_xml(isrc_value)));
        }

        if let Some(duration_value) = duration {
            resource_xml.push_str(&format!(
                "      <Duration>{}</Duration>\n",
                escape_xml(duration_value)
            ));
        }

        if let Some(file_path_value) = file_path {
            resource_xml.push_str("      <TechnicalResourceDetails>\n");
            resource_xml.push_str(&format!(
                "        <FileName>{}</FileName>\n",
                escape_xml(file_path_value)
            ));
            resource_xml.push_str("        <AudioCodecType>MP3</AudioCodecType>\n");
            resource_xml.push_str("      </TechnicalResourceDetails>\n");
        }

        resource_xml.push_str("    </SoundRecording>\n");

        self.xml_buffer.extend_from_slice(resource_xml.as_bytes());

        self.resources_written += 1;

        // Check for progress callback
        if self.resources_written % self.config.progress_callback_frequency == 0 {
            self.report_progress();
        }

        // Flush if buffer is getting large
        self.flush_if_needed()?;

        Ok(resource_ref)
    }

    /// Finish the resource section and start the release section
    pub fn finish_resources_start_releases(&mut self) -> Result<(), BuildError> {
        if !self.message_started || self.message_finished {
            return Err(BuildError::XmlGeneration(
                "Message not in valid state".to_string(),
            ));
        }

        // End ResourceList and start ReleaseList
        self.xml_buffer.extend_from_slice(b"  </ResourceList>\n");
        self.xml_buffer.extend_from_slice(b"  <ReleaseList>\n");

        self.flush_if_needed()?;
        Ok(())
    }

    /// Write a single release to the stream
    pub fn write_release(
        &mut self,
        release_id: &str,
        title: &str,
        artist: &str,
        label: Option<&str>,
        upc: Option<&str>,
        release_date: Option<&str>,
        genre: Option<&str>,
        resource_references: &[String],
    ) -> Result<String, BuildError> {
        if !self.message_started || self.message_finished {
            return Err(BuildError::XmlGeneration(
                "Message not in valid state for writing releases".to_string(),
            ));
        }

        // Generate stable reference for this release
        let release_ref = self
            .reference_manager
            .generate_release_reference(release_id)?;

        // Build Release XML
        let mut release_xml = String::new();
        release_xml.push_str("    <Release>\n");
        release_xml.push_str(&format!(
            "      <ReleaseReference>{}</ReleaseReference>\n",
            release_ref
        ));
        release_xml.push_str(&format!(
            "      <ReleaseId>{}</ReleaseId>\n",
            escape_xml(release_id)
        ));
        release_xml.push_str("      <ReleaseType>Album</ReleaseType>\n");
        release_xml.push_str(&format!("      <Title>{}</Title>\n", escape_xml(title)));
        release_xml.push_str(&format!(
            "      <DisplayArtist>{}</DisplayArtist>\n",
            escape_xml(artist)
        ));

        if let Some(label_value) = label {
            release_xml.push_str(&format!(
                "      <LabelName>{}</LabelName>\n",
                escape_xml(label_value)
            ));
        }

        if let Some(upc_value) = upc {
            release_xml.push_str(&format!("      <UPC>{}</UPC>\n", escape_xml(upc_value)));
        }

        if let Some(date_value) = release_date {
            release_xml.push_str(&format!(
                "      <ReleaseDate>{}</ReleaseDate>\n",
                escape_xml(date_value)
            ));
        }

        if let Some(genre_value) = genre {
            release_xml.push_str(&format!(
                "      <Genre>{}</Genre>\n",
                escape_xml(genre_value)
            ));
        }

        // Write ResourceGroup linking to resources
        if !resource_references.is_empty() {
            release_xml.push_str("      <ResourceGroup>\n");
            for resource_ref in resource_references {
                release_xml.push_str(&format!(
                    "        <ResourceReference>{}</ResourceReference>\n",
                    resource_ref
                ));
            }
            release_xml.push_str("      </ResourceGroup>\n");
        }

        release_xml.push_str("    </Release>\n");

        self.xml_buffer.extend_from_slice(release_xml.as_bytes());

        self.releases_written += 1;

        // Check for progress callback
        if self.releases_written % self.config.progress_callback_frequency == 0 {
            self.report_progress();
        }

        // Flush if buffer is getting large
        self.flush_if_needed()?;

        Ok(release_ref)
    }

    /// Finish the message and close all tags
    pub fn finish_message(&mut self) -> Result<StreamingStats, BuildError> {
        if !self.message_started || self.message_finished {
            return Err(BuildError::XmlGeneration(
                "Message not in valid state to finish".to_string(),
            ));
        }

        // End ReleaseList and close root element
        self.xml_buffer.extend_from_slice(b"  </ReleaseList>\n");
        self.xml_buffer.extend_from_slice(b"</NewReleaseMessage>\n");

        // Final flush of any remaining content
        if !self.xml_buffer.is_empty() {
            self.buffer_manager
                .write_chunk(&self.xml_buffer)
                .map_err(|e| {
                    BuildError::XmlGeneration(format!("Failed to write final chunk: {}", e))
                })?;
            self.xml_buffer.clear();
        }

        // Final flush
        self.buffer_manager
            .flush_all()
            .map_err(|e| BuildError::XmlGeneration(format!("Failed to flush: {}", e)))?;

        self.message_finished = true;

        Ok(StreamingStats {
            releases_written: self.releases_written,
            resources_written: self.resources_written,
            deals_written: self.deals_written,
            bytes_written: self.buffer_manager.total_bytes_written(),
            warnings: self.warnings.clone(),
            peak_memory_usage: self.buffer_manager.peak_buffer_size(),
        })
    }

    // Private helper methods

    fn write_message_header(&mut self, header: &MessageHeaderRequest) -> Result<(), BuildError> {
        // Generate message ID if not provided
        let default_id = Uuid::new_v4().to_string();
        let message_id = header.message_id.as_deref().unwrap_or(&default_id);

        let mut header_xml = String::new();
        header_xml.push_str("  <MessageHeader>\n");
        header_xml.push_str(&format!(
            "    <MessageId>{}</MessageId>\n",
            escape_xml(message_id)
        ));

        // Write MessageSender
        header_xml.push_str("    <MessageSender>\n");
        if !header.message_sender.party_name.is_empty() {
            header_xml.push_str(&format!(
                "      <PartyName>{}</PartyName>\n",
                escape_xml(&header.message_sender.party_name[0].text)
            ));
        }
        header_xml.push_str("    </MessageSender>\n");

        // Write MessageRecipient
        header_xml.push_str("    <MessageRecipient>\n");
        if !header.message_recipient.party_name.is_empty() {
            header_xml.push_str(&format!(
                "      <PartyName>{}</PartyName>\n",
                escape_xml(&header.message_recipient.party_name[0].text)
            ));
        }
        header_xml.push_str("    </MessageRecipient>\n");

        // Write MessageCreatedDateTime
        let default_time = chrono::Utc::now().to_rfc3339();
        let created_time = header
            .message_created_date_time
            .as_deref()
            .unwrap_or(&default_time);
        header_xml.push_str(&format!(
            "    <MessageCreatedDateTime>{}</MessageCreatedDateTime>\n",
            escape_xml(created_time)
        ));

        header_xml.push_str("  </MessageHeader>\n");

        self.xml_buffer.extend_from_slice(header_xml.as_bytes());
        Ok(())
    }

    fn flush_if_needed(&mut self) -> Result<(), BuildError> {
        // Check if XML buffer is getting large and flush it
        if self.xml_buffer.len() >= self.config.max_buffer_size {
            self.buffer_manager
                .write_chunk(&self.xml_buffer)
                .map_err(|e| {
                    BuildError::XmlGeneration(format!("Failed to flush XML buffer: {}", e))
                })?;
            self.xml_buffer.clear();
        }
        Ok(())
    }

    fn report_progress(&self) {
        if let Some(ref callback) = self.progress_callback {
            let current_memory = self.xml_buffer.len() + self.buffer_manager.current_buffer_size();

            let completion_percent = if let Some(total) = self.estimated_total_items {
                Some(
                    ((self.releases_written + self.resources_written) as f64 / total as f64)
                        * 100.0,
                )
            } else {
                None
            };

            let progress = StreamingProgress {
                releases_written: self.releases_written,
                resources_written: self.resources_written,
                bytes_written: self.buffer_manager.total_bytes_written(),
                current_memory_usage: current_memory,
                estimated_completion_percent: completion_percent,
            };

            callback(progress);
        }
    }
}

/// Result of streaming build operation
///
/// Contains comprehensive statistics and metadata about a completed
/// streaming DDEX XML build operation, including performance metrics
/// and any warnings that were generated during the process.
///
/// # Example
///
/// Illustrative fragment (not compiled): assumes a `streaming_builder` that has
/// already written a complete message.
///
/// ```ignore
/// use ddex_builder::streaming::StreamingResult;
///
/// // After completing a streaming build
/// let result = streaming_builder.finish_message()?;
/// println!("Built {} releases with {} resources",
///          result.releases_written, result.resources_written);
/// println!("Generated {} bytes using {} peak memory",
///          result.bytes_written, result.peak_memory_usage);
///
/// if !result.warnings.is_empty() {
///     println!("Warnings: {:?}", result.warnings);
/// }
/// ```
#[derive(Debug)]
pub struct StreamingResult {
    /// Total number of releases written to the stream
    pub releases_written: usize,
    /// Total number of resources (tracks/recordings) written to the stream
    pub resources_written: usize,
    /// Total number of deals written to the stream
    pub deals_written: usize,
    /// Total bytes written to the output stream
    pub bytes_written: usize,
    /// Any warnings generated during the streaming operation
    pub warnings: Vec<BuildWarning>,
    /// Peak memory usage in bytes during the streaming process
    pub peak_memory_usage: usize,
}

/// Statistics from a completed streaming operation
///
/// Internal statistics structure used during streaming operations.
/// Similar to StreamingResult but used for internal tracking.
///
/// # Example
/// ```
/// use ddex_builder::streaming::StreamingStats;
///
/// // Internal usage - returned by finish_message()
/// let stats = StreamingStats {
///     releases_written: 1000,
///     resources_written: 15000,
///     deals_written: 50,
///     bytes_written: 25 * 1024 * 1024, // 25MB
///     warnings: vec![],
///     peak_memory_usage: 8 * 1024 * 1024, // 8MB peak
/// };
/// ```
#[derive(Debug, Clone)]
pub struct StreamingStats {
    /// Number of releases successfully written
    pub releases_written: usize,
    /// Number of resources successfully written
    pub resources_written: usize,
    /// Number of deals successfully written
    pub deals_written: usize,
    /// Total bytes written to the output
    pub bytes_written: usize,
    /// List of warnings generated during streaming
    pub warnings: Vec<BuildWarning>,
    /// Peak memory usage observed during streaming
    pub peak_memory_usage: usize,
}

/// Errors that can occur during streaming operations
///
/// Comprehensive error types for streaming DDEX XML generation,
/// including state management errors, I/O issues, and XML formatting problems.
///
/// # Example
///
/// Illustrative fragment (not compiled): assumes an existing `streaming_builder`.
///
/// ```ignore
/// use ddex_builder::streaming::{StreamingBuilder, StreamingError};
///
/// match streaming_builder.write_resource(/*...*/) {
///     Ok(resource_ref) => println!("Resource written: {}", resource_ref),
///     Err(StreamingError::InvalidState { message }) => {
///         eprintln!("Invalid state: {}", message);
///     }
///     Err(StreamingError::IoError(io_err)) => {
///         eprintln!("I/O error: {}", io_err);
///     }
///     Err(other_err) => eprintln!("Other error: {}", other_err),
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum StreamingError {
    /// Invalid state transition (e.g., writing resources after finishing message)
    #[error("Invalid state: {message}")]
    InvalidState {
        /// Description of the invalid state and what operation was attempted
        message: String,
    },

    /// I/O error during streaming operations (writing to output, flushing buffers)
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// XML writing or formatting error
    #[error("XML writing error: {0}")]
    XmlError(#[from] quick_xml::Error),

    /// General build error from the underlying builder system
    #[error("Build error: {0}")]
    BuildError(#[from] BuildError),
}

impl From<StreamingError> for BuildError {
    fn from(err: StreamingError) -> Self {
        match err {
            StreamingError::InvalidState { message } => BuildError::XmlGeneration(message),
            StreamingError::IoError(e) => BuildError::XmlGeneration(format!("I/O error: {}", e)),
            StreamingError::XmlError(e) => BuildError::XmlGeneration(format!("XML error: {}", e)),
            StreamingError::BuildError(e) => e,
        }
    }
}

/// Escape XML special characters
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
