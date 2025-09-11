---
sidebar_position: 3
---

# Rust API Reference

Complete API reference for using DDEX Builder in Rust with deterministic XML generation and full type safety.

## Installation

```toml
[dependencies]
ddex-builder = "0.2.5"
ddex-core = "0.2.5"
tokio = { version = "1.0", features = ["full"] } # For async features
serde_json = "1.0" # For JSON serialization
uuid = { version = "1.0", features = ["v4"] } # For ID generation
```

## Basic Usage

### Simple Build

```rust
use ddex_builder::DDEXBuilder;
use ddex_core::models::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder = DDEXBuilder::new();
    
    let build_request = BuildRequest {
        message_header: MessageHeader {
            message_id: "MSG_001".to_string(),
            message_sender_name: "My Label".to_string(),
            message_recipient_name: "Spotify".to_string(),
            ..Default::default()
        },
        version: ERNVersion::Ern43,
        releases: vec![Release {
            release_id: "REL_001".to_string(),
            title: vec![LocalizedString {
                text: "My Amazing Album".to_string(),
                language_code: Some("en".to_string()),
                ..Default::default()
            }],
            display_artist: "Amazing Artist".to_string(),
            release_type: ReleaseType::Album,
            tracks: vec![Track {
                resource_reference: "TR_001".to_string(),
                title: "Track 1".to_string(),
                isrc: Some("USABC1234567".to_string()),
                duration: Some(std::time::Duration::from_secs(180)),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    
    let xml = builder.build(&build_request)?;
    println!("Generated XML: {} bytes", xml.len());
    
    Ok(())
}
```

### Async Build

```rust
use ddex_builder::DDEXBuilder;
use ddex_core::models::BuildRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder = DDEXBuilder::new();
    let request = BuildRequest { /* ... */ };
    
    let result = builder.build_async(&request).await?;
    
    println!("Generated XML: {} bytes", result.xml.len());
    println!("Warnings: {:?}", result.warnings);
    
    Ok(())
}
```

## Core Types

### DDEXBuilder

The main builder struct with configuration and presets.

```rust
pub struct DDEXBuilder {
    // Internal configuration
}

impl DDEXBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self;
    
    /// Create a builder with custom configuration
    pub fn with_config(config: BuildConfig) -> Self;
    
    /// Build DDEX XML synchronously
    pub fn build(&self, request: &BuildRequest) -> Result<String, BuildError>;
    
    /// Build DDEX XML asynchronously
    pub async fn build_async(&self, request: &BuildRequest) -> Result<BuildResult, BuildError>;
    
    /// Build with detailed result information
    pub fn build_detailed(&self, request: &BuildRequest) -> Result<DetailedBuildResult, BuildError>;
    
    /// Apply a preset configuration
    pub fn apply_preset(&mut self, preset: &str) -> Result<(), BuildError>;
    
    /// Get available presets
    pub fn get_available_presets(&self) -> Vec<String>;
    
    /// Perform preflight validation
    pub fn preflight(&self, request: &BuildRequest) -> PreflightResult;
    
    /// Canonicalize existing XML
    pub fn canonicalize(&self, xml: &str) -> Result<String, BuildError>;
    
    /// Generate diff between original and new
    pub fn diff(&self, original_xml: &str, request: &BuildRequest) -> Result<DiffResult, BuildError>;
}
```

### BuildConfig

Comprehensive configuration for deterministic builds.

```rust
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Determinism configuration
    pub determinism: DeterminismConfig,
    
    /// ID generation strategy
    pub id_strategy: IdStrategy,
    
    /// Validation level
    pub validation_level: ValidationLevel,
    
    /// Preset configuration
    pub preset: Option<String>,
    
    /// Lock preset to prevent changes
    pub preset_locked: bool,
    
    /// Maximum memory usage
    pub memory_limit: Option<usize>,
    
    /// Build timeout
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct DeterminismConfig {
    /// Canonicalization mode
    pub canon_mode: CanonMode,
    
    /// Element sorting strategy
    pub sort_strategy: SortStrategy,
    
    /// Output formatting mode
    pub output_mode: OutputMode,
    
    /// Line ending style
    pub line_ending: LineEnding,
    
    /// Unicode normalization
    pub unicode_normalization: UnicodeNorm,
    
    /// Enable reproducibility verification
    pub verify_determinism: bool,
    
    /// Emit reproducibility banner
    pub emit_banner: bool,
}

#[derive(Debug, Clone)]
pub enum CanonMode {
    /// DDEX Builder Canonicalization v1.0
    DbC14n,
    /// Pretty-printed XML
    Pretty,
    /// Compact XML
    Compact,
}

#[derive(Debug, Clone)]
pub enum IdStrategy {
    /// Random UUIDs
    Uuid,
    /// Time-ordered UUIDs
    UuidV7,
    /// Sequential IDs (A1, A2, etc.)
    Sequential,
    /// Content-based stable hashes
    StableHash { recipe: String },
}
```

### BuildRequest

The input structure for building DDEX XML.

```rust
#[derive(Debug, Clone)]
pub struct BuildRequest {
    /// Message header information
    pub message_header: MessageHeader,
    
    /// DDEX version to generate
    pub version: ERNVersion,
    
    /// DDEX profile (AudioAlbum, AudioSingle, etc.)
    pub profile: Option<ERNProfile>,
    
    /// Message control type
    pub message_control_type: MessageControlType,
    
    /// Releases to include
    pub releases: Vec<Release>,
    
    /// Deals and commercial terms
    pub deals: Vec<Deal>,
    
    /// Parties (labels, publishers, etc.)
    pub parties: Vec<Party>,
    
    /// Resources (tracks, images, videos)
    pub resources: Vec<Resource>,
    
    /// Extensions and custom elements
    pub extensions: HashMap<String, serde_json::Value>,
}

impl BuildRequest {
    /// Create a minimal build request
    pub fn minimal(
        message_id: &str,
        sender: &str,
        recipient: &str,
    ) -> Self;
    
    /// Add a release to the request
    pub fn with_release(mut self, release: Release) -> Self;
    
    /// Add deals for a release
    pub fn with_deals(mut self, deals: Vec<Deal>) -> Self;
    
    /// Validate the request structure
    pub fn validate(&self) -> Result<(), ValidationError>;
}
```

### BuildResult

Detailed results from the build process.

```rust
#[derive(Debug, Clone)]
pub struct BuildResult {
    /// Generated XML content
    pub xml: String,
    
    /// Build warnings (non-fatal issues)
    pub warnings: Vec<BuildWarning>,
    
    /// Build statistics
    pub statistics: BuildStatistics,
    
    /// Canonical hash (if enabled)
    pub canonical_hash: Option<String>,
    
    /// Reproducibility information
    pub reproducibility_info: Option<ReproducibilityInfo>,
    
    /// Generation metadata
    pub metadata: BuildMetadata,
}

#[derive(Debug, Clone)]
pub struct BuildStatistics {
    /// Build duration
    pub build_duration: std::time::Duration,
    
    /// Memory usage peak
    pub peak_memory_bytes: usize,
    
    /// Generated XML size
    pub xml_size_bytes: usize,
    
    /// Number of elements generated
    pub element_count: usize,
    
    /// Number of references linked
    pub reference_count: usize,
}

#[derive(Debug, Clone)]
pub struct BuildWarning {
    /// Warning code
    pub code: String,
    
    /// Human-readable message
    pub message: String,
    
    /// Location in the input data
    pub location: Option<String>,
    
    /// Suggested fix
    pub suggestion: Option<String>,
}
```

## Error Handling

Comprehensive error types with actionable information.

```rust
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("Missing required field: {field} at {location}")]
    MissingRequired {
        field: String,
        location: String,
    },
    
    #[error("Invalid format for {field}: {value}")]
    InvalidFormat {
        field: String,
        value: String,
    },
    
    #[error("Reference not found: {reference} of type {reference_type}")]
    ReferenceNotFound {
        reference: String,
        reference_type: String,
    },
    
    #[error("Circular reference detected: {chain}")]
    CircularReference {
        chain: String,
    },
    
    #[error("Preset not found: {preset}")]
    PresetNotFound {
        preset: String,
    },
    
    #[error("Preset locked: cannot modify {preset}")]
    PresetLocked {
        preset: String,
    },
    
    #[error("Memory limit exceeded: {used} > {limit} bytes")]
    MemoryLimitExceeded {
        used: usize,
        limit: usize,
    },
    
    #[error("Build timeout after {timeout_ms}ms")]
    Timeout {
        timeout_ms: u64,
    },
    
    #[error("XML generation error: {message}")]
    XmlGeneration {
        message: String,
    },
}

impl BuildError {
    /// Get error code for programmatic handling
    pub fn code(&self) -> &str;
    
    /// Get detailed location information
    pub fn location(&self) -> Option<&str>;
    
    /// Get suggestions for fixing the error
    pub fn suggestions(&self) -> Vec<String>;
}
```

## Deterministic Builds

### DB-C14N/1.0 Canonicalization

```rust
use ddex_builder::{DDEXBuilder, BuildConfig, DeterminismConfig, CanonMode};

fn create_canonical_builder() -> DDEXBuilder {
    let determinism = DeterminismConfig {
        canon_mode: CanonMode::DbC14n,
        sort_strategy: SortStrategy::Canonical,
        output_mode: OutputMode::DbC14n,
        line_ending: LineEnding::LF,
        unicode_normalization: UnicodeNorm::NFC,
        verify_determinism: true,
        emit_banner: true,
    };
    
    let config = BuildConfig {
        determinism,
        id_strategy: IdStrategy::StableHash {
            recipe: "v1".to_string(),
        },
        validation_level: ValidationLevel::Strict,
        ..Default::default()
    };
    
    DDEXBuilder::with_config(config)
}

fn build_deterministic(request: &BuildRequest) -> Result<String, BuildError> {
    let builder = create_canonical_builder();
    
    // This will produce identical output every time
    let result = builder.build_detailed(request)?;
    
    if result.reproducibility_info.is_some() {
        println!("✅ Build is reproducible");
        if let Some(hash) = result.canonical_hash {
            println!("Canonical hash: {}", hash);
        }
    }
    
    Ok(result.xml)
}
```

### Stable Hash IDs

Content-based deterministic ID generation.

```rust
use ddex_builder::{DDEXBuilder, IdStrategy, StableHashConfig};

fn create_stable_hash_builder() -> DDEXBuilder {
    let config = BuildConfig {
        id_strategy: IdStrategy::StableHash {
            recipe: "v1".to_string(),
        },
        ..Default::default()
    };
    
    DDEXBuilder::with_config(config)
}

// These will generate the same IDs for the same content
fn demonstrate_stable_ids() -> Result<(), BuildError> {
    let builder = create_stable_hash_builder();
    
    let request = BuildRequest {
        releases: vec![Release {
            title: vec![LocalizedString {
                text: "Same Title".to_string(),
                language_code: Some("en".to_string()),
                ..Default::default()
            }],
            upc: Some("123456789012".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };
    
    let xml1 = builder.build(&request)?;
    let xml2 = builder.build(&request)?;
    
    assert_eq!(xml1, xml2); // Identical output
    
    Ok(())
}
```

## Preset System

Platform-specific configurations with provenance tracking.

```rust
use ddex_builder::{DDEXBuilder, PresetInfo};

fn use_spotify_preset() -> Result<(), BuildError> {
    let mut builder = DDEXBuilder::new();
    
    // Apply Spotify-specific configuration
    builder.apply_preset("spotify_audio_43")?;
    
    // Check preset information
    let preset_info = builder.get_preset_info("spotify_audio_43")?;
    println!("Preset: {} v{}", preset_info.name, preset_info.version);
    println!("Source: {}", preset_info.provenance_url);
    
    // Lock preset to prevent accidental changes
    builder.lock_preset()?;
    
    let request = BuildRequest { /* ... */ };
    let xml = builder.build(&request)?;
    
    Ok(())
}

fn list_available_presets() -> Result<(), BuildError> {
    let builder = DDEXBuilder::new();
    let presets = builder.get_available_presets();
    
    for preset in presets {
        let info = builder.get_preset_info(&preset)?;
        println!("{}: {}", preset, info.description);
        println!("  Source: {}", info.source);
        println!("  Version: {}", info.version);
    }
    
    Ok(())
}
```

## Advanced Features

### Preflight Validation

```rust
use ddex_builder::{DDEXBuilder, PreflightResult, ValidationLevel};

fn validate_before_build(request: &BuildRequest) -> Result<(), BuildError> {
    let builder = DDEXBuilder::new();
    let preflight = builder.preflight(request);
    
    if !preflight.is_valid {
        println!("❌ Preflight validation failed:");
        for error in &preflight.errors {
            println!("  Error: {} at {}", error.message, error.location.as_ref().unwrap_or(&"unknown".to_string()));
            if let Some(suggestion) = &error.suggestion {
                println!("    Suggestion: {}", suggestion);
            }
        }
        return Err(BuildError::PreflightFailed);
    }
    
    println!("✅ Preflight validation passed");
    for warning in &preflight.warnings {
        println!("  Warning: {}", warning.message);
    }
    
    // Show validation statistics
    println!("Validation stats:");
    println!("  Total fields: {}", preflight.statistics.total_fields);
    println!("  Valid fields: {}", preflight.statistics.validated_fields);
    println!("  Missing required: {:?}", preflight.statistics.missing_required_fields);
    
    Ok(())
}
```

### Streaming Builds

For generating large DDEX catalogs efficiently.

```rust
use ddex_builder::{DDEXBuilder, StreamingBuilder, StreamConfig};
use futures::stream::StreamExt;

#[tokio::main]
async fn build_large_catalog() -> Result<(), Box<dyn std::error::Error>> {
    let config = StreamConfig {
        batch_size: 100,
        memory_limit: 500 * 1024 * 1024, // 500MB
    };
    
    let streaming_builder = StreamingBuilder::new(config);
    
    // Create a stream of releases from database/file
    let release_stream = get_releases_stream().await?;
    
    let mut xml_stream = streaming_builder.build_stream(release_stream);
    
    while let Some(xml_chunk) = xml_stream.next().await {
        let chunk = xml_chunk?;
        
        // Write chunk to file or send to destination
        write_chunk_to_file(&chunk).await?;
    }
    
    Ok(())
}

async fn get_releases_stream() -> Result<impl Stream<Item = Release>, Box<dyn std::error::Error>> {
    // Implementation would return a stream of releases
    unimplemented!()
}

async fn write_chunk_to_file(chunk: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Write XML chunk to output file
    unimplemented!()
}
```

### Custom Extension Handling

```rust
use ddex_builder::{DDEXBuilder, ExtensionProcessor};
use serde_json::Value;

struct MyExtensionProcessor;

impl ExtensionProcessor for MyExtensionProcessor {
    fn process_extension(
        &self,
        key: &str,
        value: &Value,
        context: &ExtensionContext,
    ) -> Result<XmlElement, ProcessError> {
        match key {
            "customMetadata" => {
                // Convert JSON to XML element
                Ok(XmlElement {
                    name: "CustomMetadata".to_string(),
                    attributes: HashMap::new(),
                    content: value.to_string(),
                })
            }
            _ => Err(ProcessError::UnknownExtension(key.to_string())),
        }
    }
}

fn build_with_extensions() -> Result<(), BuildError> {
    let mut builder = DDEXBuilder::new();
    builder.set_extension_processor(Box::new(MyExtensionProcessor));
    
    let request = BuildRequest {
        extensions: {
            let mut ext = HashMap::new();
            ext.insert("customMetadata".to_string(), serde_json::json!({
                "label": "Special Edition",
                "priority": "high"
            }));
            ext
        },
        ..Default::default()
    };
    
    let xml = builder.build(&request)?;
    
    Ok(())
}
```

### Batch Processing

```rust
use ddex_builder::{DDEXBuilder, BatchProcessor, BatchConfig};
use rayon::prelude::*;

fn batch_build_releases(releases: Vec<BuildRequest>) -> Result<Vec<String>, BuildError> {
    let config = BatchConfig {
        parallel: true,
        max_workers: num_cpus::get(),
        memory_per_worker: 100 * 1024 * 1024, // 100MB per worker
    };
    
    let processor = BatchProcessor::new(config);
    
    // Process in parallel
    let results: Result<Vec<_>, _> = releases
        .par_iter()
        .map(|request| {
            let builder = DDEXBuilder::new();
            builder.build(request)
        })
        .collect();
    
    results
}

// Alternative: Use the built-in batch processor
fn batch_build_with_processor(requests: Vec<BuildRequest>) -> Result<Vec<String>, BuildError> {
    let builder = DDEXBuilder::new();
    let results = builder.build_batch(&requests)?;
    
    for (i, result) in results.iter().enumerate() {
        if let Some(warnings) = &result.warnings {
            println!("Request {}: {} warnings", i, warnings.len());
        }
    }
    
    Ok(results.into_iter().map(|r| r.xml).collect())
}
```

## Integration Examples

### With ddex-parser (Round-trip)

```rust
use ddex_parser::DDEXParser;
use ddex_builder::DDEXBuilder;

fn round_trip_example(original_xml: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Parse original DDEX
    let parser = DDEXParser::new();
    let mut parsed = parser.parse(original_xml)?;
    
    // Modify the data
    if let Some(release) = parsed.flat.releases.get_mut(0) {
        release.title = "Modified Title".to_string();
        release.tracks.push(Track {
            title: "Bonus Track".to_string(),
            position: release.tracks.len() as u32 + 1,
            isrc: Some("USXYZ2024001".to_string()),
            duration: Some(std::time::Duration::from_secs(240)),
            ..Default::default()
        });
    }
    
    // Build new XML with modifications
    let builder = DDEXBuilder::new();
    let build_request = parsed.to_build_request();
    let new_xml = builder.build(&build_request)?;
    
    Ok(new_xml)
}
```

### With Database Integration

```rust
use ddex_builder::{DDEXBuilder, BuildRequest};
use sqlx::{PgPool, Row};
use serde_json;

async fn build_from_database(
    release_id: &str,
    pool: &PgPool,
) -> Result<String, Box<dyn std::error::Error>> {
    // Fetch release data from database
    let release_row = sqlx::query!(
        "SELECT * FROM releases WHERE id = $1",
        release_id
    )
    .fetch_one(pool)
    .await?;
    
    let tracks_rows = sqlx::query!(
        "SELECT * FROM tracks WHERE release_id = $1 ORDER BY position",
        release_id
    )
    .fetch_all(pool)
    .await?;
    
    // Convert database rows to DDEX models
    let release = Release {
        release_id: release_row.id,
        title: vec![LocalizedString {
            text: release_row.title,
            language_code: Some("en".to_string()),
            ..Default::default()
        }],
        display_artist: release_row.artist,
        upc: release_row.upc,
        tracks: tracks_rows.into_iter().map(|track| Track {
            title: track.title,
            position: track.position as u32,
            isrc: track.isrc,
            duration: track.duration_seconds.map(|d| std::time::Duration::from_secs(d as u64)),
            ..Default::default()
        }).collect(),
        ..Default::default()
    };
    
    let build_request = BuildRequest {
        message_header: MessageHeader {
            message_id: format!("REL_{}", release_id),
            message_sender_name: "My Label".to_string(),
            message_recipient_name: "Platform".to_string(),
            ..Default::default()
        },
        releases: vec![release],
        ..Default::default()
    };
    
    let builder = DDEXBuilder::new();
    let xml = builder.build(&build_request)?;
    
    Ok(xml)
}
```

## CLI Tool

The builder also provides a command-line interface:

```bash
# Install CLI
cargo install ddex-builder

# Build from JSON
ddex-builder build --from-json request.json --out release.xml

# Use preset
ddex-builder build --from-json request.json --preset spotify_audio_43

# Enable deterministic mode
ddex-builder build --from-json request.json --db-c14n --id stable-hash:v1

# Verify determinism
ddex-builder build --from-json request.json --verify-determinism 3

# Canonicalize existing XML
ddex-builder canon input.xml > canonical.xml

# Generate diff
ddex-builder diff --old original.xml --from-json modified.json

# Show preset information
ddex-builder preset-info spotify_audio_43

# Validate build request
ddex-builder preflight request.json
```

## Performance Tips

### Optimization Strategies

```rust
use ddex_builder::{DDEXBuilder, BuildConfig, ValidationLevel};

fn create_optimized_builder() -> DDEXBuilder {
    let config = BuildConfig {
        // Skip validation for trusted input
        validation_level: ValidationLevel::Basic,
        
        // Use fast ID generation for non-production builds
        id_strategy: IdStrategy::Sequential,
        
        // Disable determinism checks for speed
        determinism: DeterminismConfig {
            verify_determinism: false,
            emit_banner: false,
            ..Default::default()
        },
        
        // Reasonable memory limits
        memory_limit: Some(200 * 1024 * 1024), // 200MB
        
        ..Default::default()
    };
    
    DDEXBuilder::with_config(config)
}
```

### Memory Management

```rust
fn build_large_catalog_efficiently(
    releases: Vec<BuildRequest>,
) -> Result<Vec<String>, BuildError> {
    let builder = DDEXBuilder::new();
    let mut results = Vec::new();
    
    // Process in chunks to manage memory
    for chunk in releases.chunks(100) {
        let chunk_results: Result<Vec<_>, _> = chunk
            .iter()
            .map(|request| builder.build(request))
            .collect();
        
        results.extend(chunk_results?);
        
        // Force garbage collection between chunks if needed
        // (This is more relevant for other languages)
    }
    
    Ok(results)
}
```

## Documentation Links

- **Core Types**: [ddex-core documentation](https://docs.rs/ddex-core)
- **Builder**: [ddex-builder documentation](https://docs.rs/ddex-builder)
- **DB-C14N Spec**: [Canonicalization guide](../../builder/canonicalization.md)
- **Examples**: [GitHub repository](https://github.com/daddykev/ddex-suite/tree/main/examples)

## Next Steps

- **[Parser API](../parser/rust.md)** - Learn to parse DDEX XML in Rust
- **[Canonicalization](../../builder/canonicalization.md)** - Understanding deterministic output
- **[Presets Guide](../../builder/presets.md)** - Platform-specific configurations
- **[Examples](../../examples/)** - Complete working examples