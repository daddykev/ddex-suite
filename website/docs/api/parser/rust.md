---
sidebar_position: 3
---

# Rust API Reference

Complete API reference for using DDEX Parser in Rust with full type safety and zero-cost abstractions.

## Installation

```toml
[dependencies]
ddex-parser = "0.2.5"
ddex-core = "0.2.5"
tokio = { version = "1.0", features = ["full"] } # For async features
```

## Basic Usage

### Synchronous Parsing

```rust
use ddex_parser::DDEXParser;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = DDEXParser::new();
    let xml_content = fs::read_to_string("release.xml")?;
    
    let result = parser.parse(&xml_content)?;
    
    // Access parsed data
    println!("Release title: {}", result.flat.releases[0].title);
    println!("Artist: {}", result.flat.sound_recordings[0].display_artist);
    println!("Track count: {}", result.flat.sound_recordings.len());
    
    Ok(())
}
```

### Asynchronous Parsing

```rust
use ddex_parser::DDEXParser;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = DDEXParser::new();
    let xml_content = fs::read_to_string("release.xml").await?;
    
    let result = parser.parse_async(&xml_content).await?;
    
    // Process results...
    Ok(())
}
```

## Core Types

### DDEXParser

The main parser struct with configuration options.

```rust
pub struct DDEXParser {
    // Configuration fields
}

impl DDEXParser {
    /// Create a new parser with default configuration
    pub fn new() -> Self;
    
    /// Create a parser with custom configuration
    pub fn with_config(config: ParseConfig) -> Self;
    
    /// Parse DDEX XML synchronously
    pub fn parse(&self, xml: &str) -> Result<ParsedMessage, ParseError>;
    
    /// Parse DDEX XML asynchronously
    pub async fn parse_async(&self, xml: &str) -> Result<ParsedMessage, ParseError>;
    
    /// Parse from a file path
    pub fn parse_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<ParsedMessage, ParseError>;
    
    /// Detect DDEX version without full parsing
    pub fn detect_version(&self, xml: &str) -> Result<ERNVersion, ParseError>;
    
    /// Perform sanity check on DDEX XML
    pub fn sanity_check(&self, xml: &str) -> SanityCheckResult;
}
```

### ParseConfig

Configuration for the parser behavior.

```rust
#[derive(Debug, Clone)]
pub struct ParseConfig {
    /// Include raw extension elements
    pub include_extensions: bool,
    
    /// Include XML comments
    pub include_comments: bool,
    
    /// Enable streaming mode for large files
    pub streaming_mode: bool,
    
    /// Memory limit in bytes
    pub memory_limit: Option<usize>,
    
    /// Parse timeout in milliseconds
    pub timeout_ms: Option<u64>,
    
    /// Validation level
    pub validation_level: ValidationLevel,
}

impl Default for ParseConfig {
    fn default() -> Self {
        Self {
            include_extensions: true,
            include_comments: false,
            streaming_mode: false,
            memory_limit: None,
            timeout_ms: Some(30_000),
            validation_level: ValidationLevel::Basic,
        }
    }
}
```

### ParsedMessage

The main result type containing both graph and flattened representations.

```rust
#[derive(Debug, Clone)]
pub struct ParsedMessage {
    /// Graph representation (faithful to DDEX structure)
    pub graph: GraphMessage,
    
    /// Flattened representation (developer-friendly)
    pub flat: FlattenedMessage,
    
    /// Original XML for round-trip fidelity
    pub original_xml: Option<String>,
    
    /// Extensions and passthrough data
    pub extensions: HashMap<String, XmlFragment>,
}

impl ParsedMessage {
    /// Convert to a build request for ddex-builder
    pub fn to_build_request(&self) -> BuildRequest;
    
    /// Get all releases in flattened form
    pub fn releases(&self) -> &[ParsedRelease];
    
    /// Get all sound recordings
    pub fn sound_recordings(&self) -> &[ParsedSoundRecording];
    
    /// Get all deals
    pub fn deals(&self) -> &[ParsedDeal];
    
    /// Get message metadata
    pub fn message_header(&self) -> &MessageHeader;
}
```

### Error Types

Comprehensive error handling with detailed information.

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("XML parsing error: {0}")]
    XmlError(String),
    
    #[error("Invalid DDEX structure: {0}")]
    InvalidStructure(String),
    
    #[error("Unsupported DDEX version: {version}")]
    UnsupportedVersion { version: String },
    
    #[error("File not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Memory limit exceeded: {limit} bytes")]
    MemoryLimitExceeded { limit: usize },
    
    #[error("Parse timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    
    #[error("Security violation: {reason}")]
    SecurityViolation { reason: String },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl ParseError {
    /// Get the error location in the XML
    pub fn location(&self) -> Option<ErrorLocation>;
    
    /// Get suggestions for fixing the error
    pub fn suggestions(&self) -> Vec<String>;
}
```

## Data Models

### Core Models from ddex-core

```rust
use ddex_core::models::*;

// Message structure
pub struct ERNMessage {
    pub message_header: MessageHeader,
    pub parties: Vec<Party>,
    pub resources: Vec<Resource>,
    pub releases: Vec<Release>,
    pub deals: Vec<Deal>,
    pub extensions: HashMap<String, XmlFragment>,
}

// Release information
pub struct Release {
    pub release_reference: String,
    pub release_id: Vec<Identifier>,
    pub release_title: Vec<LocalizedString>,
    pub release_type: Option<ReleaseType>,
    pub genre: Vec<Genre>,
    pub display_artist: Vec<Artist>,
    pub release_date: Vec<ReleaseEvent>,
    pub territory_code: Vec<String>,
    pub extensions: HashMap<String, XmlFragment>,
}

// Resource (tracks, videos, images)
pub struct Resource {
    pub resource_reference: String,
    pub resource_type: ResourceType,
    pub resource_id: Vec<Identifier>,
    pub technical_details: Vec<TechnicalDetails>,
    pub rights_controller: Vec<String>,
    pub p_line: Vec<Copyright>,
    pub c_line: Vec<Copyright>,
}
```

### Flattened Models

Developer-friendly representations with direct access to common fields.

```rust
// Flattened release with resolved references
pub struct ParsedRelease {
    pub release_id: String,
    pub title: String,
    pub display_artist: String,
    pub artists: Vec<Artist>,
    pub release_type: ReleaseType,
    pub release_date: Option<chrono::NaiveDate>,
    pub genre: Vec<String>,
    pub territories: Vec<String>,
    
    // Identifiers
    pub upc: Option<String>,
    pub ean: Option<String>,
    pub catalog_number: Option<String>,
    pub grid: Option<String>,
    pub proprietary_ids: Vec<ProprietaryId>,
    
    // Tracks with resolved references
    pub tracks: Vec<ParsedTrack>,
    
    // Images with resolved references  
    pub images: Vec<ParsedImage>,
    
    // Round-trip data
    pub graph_reference: String,
    pub extensions: HashMap<String, XmlFragment>,
}

pub struct ParsedTrack {
    pub track_id: String,
    pub position: u32,
    pub title: String,
    pub display_artist: String,
    pub isrc: Option<String>,
    pub duration: Option<std::time::Duration>,
    pub is_explicit: Option<bool>,
    pub language: Option<String>,
    pub genre: Vec<String>,
}
```

## Streaming API

For processing large DDEX files efficiently.

```rust
use ddex_parser::{DDEXParser, StreamConfig};
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = DDEXParser::new();
    let config = StreamConfig {
        chunk_size: 100, // Process 100 releases at a time
        buffer_size: 1024 * 1024, // 1MB buffer
    };
    
    let mut stream = parser.stream_file("large-catalog.xml", config).await?;
    
    while let Some(batch) = stream.next().await {
        let releases = batch?;
        
        // Process batch of releases
        for release in releases {
            process_release(&release).await?;
        }
    }
    
    Ok(())
}

async fn process_release(release: &ParsedRelease) -> Result<(), Box<dyn std::error::Error>> {
    // Process individual release
    println!("Processing: {}", release.title);
    Ok(())
}
```

## Advanced Features

### Custom Extension Handling

```rust
use ddex_parser::{DDEXParser, ExtensionHandler};

struct MyExtensionHandler;

impl ExtensionHandler for MyExtensionHandler {
    fn handle_unknown_element(
        &self,
        element_name: &str,
        content: &str,
        attributes: &HashMap<String, String>,
    ) -> Option<serde_json::Value> {
        match element_name {
            "CustomMetadata" => {
                // Custom processing for your extensions
                Some(serde_json::json!({
                    "type": "custom",
                    "content": content,
                    "attributes": attributes
                }))
            }
            _ => None,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = DDEXParser::with_extension_handler(Box::new(MyExtensionHandler));
    let result = parser.parse(&xml_content)?;
    
    // Access custom extensions
    for (key, value) in &result.extensions {
        println!("Extension {}: {:?}", key, value);
    }
    
    Ok(())
}
```

### Performance Optimization

```rust
use ddex_parser::{DDEXParser, ParseConfig, ValidationLevel};

fn create_optimized_parser() -> DDEXParser {
    let config = ParseConfig {
        // Disable extensions for faster parsing
        include_extensions: false,
        include_comments: false,
        
        // Skip validation for trusted sources
        validation_level: ValidationLevel::None,
        
        // Enable streaming for large files
        streaming_mode: true,
        
        // Set reasonable memory limits
        memory_limit: Some(500 * 1024 * 1024), // 500MB
        timeout_ms: Some(60_000), // 60 seconds
    };
    
    DDEXParser::with_config(config)
}
```

### Error Handling

```rust
use ddex_parser::{DDEXParser, ParseError};

fn robust_parse(xml_content: &str) -> Result<ParsedMessage, String> {
    let parser = DDEXParser::new();
    
    match parser.parse(xml_content) {
        Ok(result) => Ok(result),
        Err(ParseError::XmlError(msg)) => {
            eprintln!("XML parsing failed: {}", msg);
            Err(format!("Invalid XML: {}", msg))
        }
        Err(ParseError::UnsupportedVersion { version }) => {
            eprintln!("Unsupported DDEX version: {}", version);
            Err(format!("Please upgrade to support version {}", version))
        }
        Err(ParseError::MemoryLimitExceeded { limit }) => {
            eprintln!("File too large: exceeded {} bytes", limit);
            Err("File too large for processing".to_string())
        }
        Err(ParseError::Timeout { timeout_ms }) => {
            eprintln!("Parse timeout after {}ms", timeout_ms);
            Err("Processing timeout - file may be corrupted".to_string())
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            Err(format!("Parse failed: {}", e))
        }
    }
}
```

## Integration Examples

### With Serde JSON

```rust
use ddex_parser::DDEXParser;
use serde_json;

fn parse_to_json(xml_content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parser = DDEXParser::new();
    let result = parser.parse(xml_content)?;
    
    // Convert to JSON for external systems
    let json = serde_json::to_string_pretty(&result.flat)?;
    Ok(json)
}
```

### With Database Storage

```rust
use ddex_parser::DDEXParser;
use sqlx::{PgPool, Row};

async fn store_in_database(
    xml_content: &str, 
    pool: &PgPool
) -> Result<(), Box<dyn std::error::Error>> {
    let parser = DDEXParser::new();
    let result = parser.parse(xml_content)?;
    
    for release in &result.flat.releases {
        sqlx::query!(
            "INSERT INTO releases (release_id, title, artist, release_date) VALUES ($1, $2, $3, $4)",
            release.release_id,
            release.title,
            release.display_artist,
            release.release_date
        )
        .execute(pool)
        .await?;
    }
    
    Ok(())
}
```

## CLI Tool

The parser also provides a command-line interface:

```bash
# Install CLI
cargo install ddex-parser

# Parse a file to JSON
ddex-parser parse release.xml > output.json

# Detect DDEX version
ddex-parser detect-version release.xml

# Validate DDEX file
ddex-parser validate release.xml

# Stream large files
ddex-parser stream large-catalog.xml --format jsonl

# Extract specific fields
ddex-parser extract release.xml --fields title,artist,isrc
```

## Documentation Links

- **Core Types**: [ddex-core documentation](https://docs.rs/ddex-core)
- **Parser**: [ddex-parser documentation](https://docs.rs/ddex-parser)
- **Examples**: [GitHub repository](https://github.com/daddykev/ddex-suite/tree/main/examples)
- **Changelog**: [Release notes](https://github.com/daddykev/ddex-suite/releases)

## Next Steps

- **[Builder API](../builder/rust.md)** - Learn to build DDEX XML from Rust
- **[Examples](../../examples/)** - See complete working examples
- **[Error Handling](../../guides/error-handling)** - Best practices for error handling