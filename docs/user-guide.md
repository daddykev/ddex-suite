# DDEX Builder User Guide

Welcome to DDEX Builder, the fastest and most secure way to generate deterministic DDEX XML. This guide will help you get started and master all features.

## Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [Basic Usage](#basic-usage)
- [Partner Presets](#partner-presets)
- [Security Features](#security-features)
- [Version Management](#version-management)
- [Advanced Features](#advanced-features)
- [Performance Tuning](#performance-tuning)
- [Troubleshooting](#troubleshooting)
- [Migration Guide](#migration-guide)

## Quick Start

Get up and running with DDEX Builder in under 5 minutes:

### 1. Installation

```bash
# Rust/Cargo
cargo add ddex-builder

# Node.js
npm install ddex-builder

# Python
pip install ddex-builder
```

### 2. Basic Example

```rust
use ddex_builder::{Builder, DdexVersion};
use ddex_builder::builder::{BuildRequest, OutputFormat};

fn main() -> Result<(), ddex_builder::BuildError> {
    // Create builder with Spotify preset
    let mut builder = Builder::new();
    builder.preset("spotify_audio_43")?;
    
    // Build DDEX XML from minimal input
    let request = BuildRequest {
        source_xml: r#"
            <SoundRecording>
                <SoundRecordingId>
                    <ISRC>USRC17607839</ISRC>
                </SoundRecordingId>
                <ReferenceTitle>
                    <TitleText>My Amazing Song</TitleText>
                </ReferenceTitle>
                <Duration>PT3M45S</Duration>
            </SoundRecording>
        "#.to_string(),
        output_format: OutputFormat::Xml,
        preset: Some("spotify_audio_43".to_string()),
        validate_schema: true,
    };
    
    let result = builder.build_internal(&request)?;
    println!("Generated DDEX XML:\n{}", result.xml);
    Ok(())
}
```

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
ddex-builder = "1.0.0"

# Optional features
ddex-builder = { version = "1.0.0", features = ["async", "strict"] }
```

Available features:
- `async` - Async/await support with Tokio
- `strict` - Strict validation mode
- `ffi` - Foreign Function Interface support
- `wasm` - WebAssembly support

### Node.js

```bash
npm install ddex-builder
# or
yarn add ddex-builder
```

```javascript
const { DDEXBuilder } = require('ddex-builder');

// Create builder
const builder = new DDEXBuilder();
await builder.applyPreset('spotify_audio_43');

// Build XML
const result = await builder.build({
    sourceXml: '<SoundRecording>...</SoundRecording>',
    outputFormat: 'xml',
    validateSchema: true
});

console.log(result.xml);
```

### Python

```bash
pip install ddex-builder
```

```python
from ddex_builder import Builder, BuildRequest, OutputFormat

# Create builder
builder = Builder()
builder.preset('spotify_audio_43')

# Build XML
request = BuildRequest(
    source_xml='<SoundRecording>...</SoundRecording>',
    output_format=OutputFormat.XML,
    validate_schema=True
)

result = builder.build_internal(request)
print(result.xml)
```

### WebAssembly

```bash
npm install ddex-builder-wasm
```

```javascript
import init, { DDEXBuilder } from 'ddex-builder-wasm';

async function buildDDEX() {
    await init();
    
    const builder = new DDEXBuilder();
    await builder.applyPreset('spotify_audio_43');
    
    const result = await builder.build({
        sourceXml: '<SoundRecording>...</SoundRecording>',
        outputFormat: 'xml'
    });
    
    return result.xml;
}
```

## Basic Usage

### Creating a Builder

```rust
use ddex_builder::Builder;

// Default configuration
let builder = Builder::new();

// With custom configuration  
let mut builder = Builder::new();
builder.preset("universal_basic")?;
```

### Building XML

```rust
use ddex_builder::builder::{BuildRequest, OutputFormat};

let request = BuildRequest {
    source_xml: your_xml_content,
    output_format: OutputFormat::Xml,
    preset: Some("spotify_audio_43".to_string()),
    validate_schema: true,
};

let result = builder.build_internal(&request)?;

// Access results
println!("XML: {}", result.xml);
println!("Generated {} releases", result.stats.releases);
println!("Build time: {}ms", result.stats.generation_time_ms);
```

### Error Handling

```rust
use ddex_builder::BuildError;

match builder.build_internal(&request) {
    Ok(result) => {
        println!("Success: {}", result.xml);
    }
    Err(BuildError::InvalidFormat { field, message }) => {
        eprintln!("Invalid format in field '{}': {}", field, message);
    }
    Err(BuildError::Security(msg)) => {
        eprintln!("Security error: {}", msg);
    }
    Err(BuildError::Validation(errors)) => {
        eprintln!("Validation errors:");
        for error in errors {
            eprintln!("  - {}", error.message);
        }
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Partner Presets

DDEX Builder includes optimized presets for major music platforms and distributors.

### Available Presets

| Preset | Platform | DDEX Version | Content Type | Description |
|--------|----------|--------------|--------------|-------------|
| `spotify_audio_43` | Spotify | ERN 4.3 | Audio | Spotify audio releases |
| `youtube_video_43` | YouTube | ERN 4.3 | Video | YouTube video content |
| `apple_music_43` | Apple Music | ERN 4.3 | Audio | Apple Music releases |
| `universal_basic` | Universal Music | ERN 4.3 | Mixed | Universal Music basic |
| `sony_enhanced` | Sony Music | ERN 4.3 | Mixed | Sony Music enhanced |
| `warner_standard` | Warner Music | ERN 4.3 | Mixed | Warner Music standard |
| `independent_basic` | Independent | ERN 4.2 | Mixed | Independent labels |
| `classical_enhanced` | Classical | ERN 4.3 | Audio | Classical music |

### Using Presets

```rust
use ddex_builder::Builder;

let mut builder = Builder::new();

// Apply preset (unlocked)
builder.preset("spotify_audio_43")?;

// Apply and lock preset
builder.apply_preset("spotify_audio_43", true)?;

// Check available presets
let presets = builder.available_presets();
println!("Available: {:?}", presets);

// Get preset details
if let Some(preset) = builder.get_preset("spotify_audio_43") {
    println!("Description: {}", preset.description);
    println!("Version: {}", preset.version);
}
```

### Spotify Audio Preset

Optimized for Spotify's audio delivery requirements:

```rust
builder.preset("spotify_audio_43")?;

// Configured with:
// - ERN 4.3 compliance
// - Audio-focused validation
// - Spotify territory mappings
// - Required metadata fields
// - Optimized for streaming
```

**Required Fields:**
- ISRC (mandatory)
- Title (min 1 character)
- Artist name
- Duration (ISO 8601 format)
- Audio quality metadata

**Territory Support:**
- Worldwide territory codes
- Regional restrictions support
- Spotify-specific territory mappings

### YouTube Video Preset

Optimized for YouTube's video content requirements:

```rust
builder.preset("youtube_video_43")?;

// Configured with:
// - Video resource support
// - YouTube Content ID integration
// - Enhanced metadata requirements
// - Rights management features
```

**Additional Features:**
- Video resource references
- Content ID fingerprinting support
- Monetization metadata
- Regional availability settings

### Apple Music Preset

Optimized for Apple Music distribution:

```rust
builder.preset("apple_music_43")?;

// Configured with:
// - Apple Music metadata standards
// - iTunes Store compatibility
// - Enhanced audio quality requirements
// - Apple-specific identifiers
```

### Custom Presets

Create custom presets for your organization:

```rust
use ddex_builder::presets::{PartnerPreset, PresetDefaults};
use ddex_builder::DeterminismConfig;

let custom_preset = PartnerPreset {
    name: "my_label_custom".to_string(),
    description: "Custom preset for My Label".to_string(),
    version: "1.0.0".to_string(),
    determinism: DeterminismConfig::default(),
    defaults: PresetDefaults {
        version: DdexVersion::Ern43,
        // ... other settings
    },
    // ... other configuration
};
```

## Security Features

DDEX Builder includes comprehensive security measures to protect against attacks and ensure safe processing.

### Input Validation

```rust
use ddex_builder::{InputValidator, SecurityConfig};

// Create validator with custom config
let security_config = SecurityConfig {
    max_xml_size: 5_000_000,        // 5MB limit
    max_json_depth: 16,             // Prevent deep nesting
    rate_limiting_enabled: true,
    max_requests_per_minute: 50,
    validate_urls: true,
    block_private_ips: true,
    ..Default::default()
};

let validator = InputValidator::new(security_config);

// Validate XML content
validator.validate_xml_content(&xml_input)?;

// Validate file paths
validator.validate_path(&file_path)?;

// Validate URLs
validator.validate_url(&url)?;
```

### Rate Limiting

```rust
use ddex_builder::RateLimiter;

let mut limiter = RateLimiter::new(security_config);

// Check rate limit before processing
limiter.check_rate_limit("client_id")?;

// Process request...
```

### API Security

```rust
use ddex_builder::ApiSecurityManager;

let mut api_security = ApiSecurityManager::new(security_config);

// Validate API request
api_security.validate_request("build", "client_id", payload_size)?;

// Sanitize response
let safe_output = api_security.sanitize_response(&xml_output)?;

// Get security headers for WASM
let headers = api_security.get_wasm_security_headers();
```

### Security Best Practices

1. **Always validate inputs** before processing
2. **Use rate limiting** in production environments
3. **Enable strict validation** with the `strict` feature
4. **Monitor security events** in your logs
5. **Update regularly** to get security patches

```rust
// Recommended production configuration
let security_config = SecurityConfig {
    max_xml_size: 10_000_000,       // 10MB max
    max_json_depth: 32,             // Reasonable depth
    rate_limiting_enabled: true,
    max_requests_per_minute: 100,
    validate_urls: true,
    block_private_ips: true,
    ..Default::default()
};
```

## Version Management

DDEX Builder supports multiple DDEX versions with automatic conversion capabilities.

### Supported Versions

| Version | Support Level | Notes |
|---------|---------------|-------|
| ERN 3.8.2 | ✅ Full | Legacy support, stable |
| ERN 4.2 | ✅ Full | Enhanced features |
| ERN 4.3 | ✅ Full | Latest standard, recommended |

### Version Detection

```rust
use ddex_builder::{Builder, DdexVersion};

let builder = Builder::new();

// Detect version from XML
let version = builder.detect_version(&xml_content)?;
println!("Detected version: {:?}", version);
```

### Version Conversion

```rust
use ddex_builder::versions::ConversionOptions;

let builder = Builder::new();

// Check compatibility
let compatible = builder.is_version_compatible(
    DdexVersion::Ern382,
    DdexVersion::Ern43
);

if compatible {
    let options = Some(ConversionOptions {
        preserve_extensions: true,
        update_namespaces: true,
        validate_after_conversion: true,
        ..Default::default()
    });
    
    let result = builder.convert_version(
        &xml_content,
        DdexVersion::Ern382,
        DdexVersion::Ern43,
        options
    )?;
    
    println!("Converted XML: {}", result.converted_xml);
    println!("Conversion notes: {:?}", result.conversion_notes);
}
```

### Conversion Options

```rust
use ddex_builder::versions::ConversionOptions;

let options = ConversionOptions {
    // Preserve proprietary extensions
    preserve_extensions: true,
    
    // Update namespace URIs
    update_namespaces: true,
    
    // Validate after conversion
    validate_after_conversion: true,
    
    // Handle deprecated elements
    handle_deprecated_elements: true,
    
    // Convert date formats
    convert_date_formats: true,
    
    // Update schema locations
    update_schema_locations: true,
    
    // Preserve comments and processing instructions
    preserve_comments: false,
};
```

### Migration Strategies

#### Gradual Migration (ERN 3.8.2 → 4.3)

```rust
// Step 1: Convert to ERN 4.2 first
let intermediate = builder.convert_version(
    &xml_382,
    DdexVersion::Ern382,
    DdexVersion::Ern42,
    Some(options.clone())
)?;

// Step 2: Convert to ERN 4.3
let final_result = builder.convert_version(
    &intermediate.converted_xml,
    DdexVersion::Ern42,
    DdexVersion::Ern43,
    Some(options)
)?;
```

#### Direct Migration

```rust
// Direct conversion (recommended for most cases)
let result = builder.convert_version(
    &xml_382,
    DdexVersion::Ern382,
    DdexVersion::Ern43,
    Some(options)
)?;
```

## Advanced Features

### Deterministic Output

DDEX Builder guarantees byte-perfect reproducible output using DB-C14N/1.0.

```rust
use ddex_builder::DeterminismConfig;

let config = DeterminismConfig {
    verify_determinism: Some(5), // Verify with 5 iterations
    ..Default::default()
};

let mut builder = Builder::with_config(config);
```

### Custom ID Generation

```rust
use ddex_builder::{StableHashGenerator, HashAlgorithm};

let id_generator = StableHashGenerator::new(
    HashAlgorithm::Blake3,  // or HashAlgorithm::Sha256
    "my_seed_value".to_string()
);

let stable_id = id_generator.generate_id("content_key");
```

### Streaming Processing

For large files, enable streaming mode:

```rust
use ddex_builder::streaming::StreamingBuilder;

let mut streaming_builder = StreamingBuilder::new(builder);

// Process in chunks
let result = streaming_builder.build_streaming(
    &large_xml_content,
    1024 * 1024  // 1MB chunks
)?;
```

### Parallel Processing

Enable parallel processing for multiple releases:

```rust
use ddex_builder::parallel_processing::ParallelBuilder;

let parallel_builder = ParallelBuilder::new(builder, 4); // 4 threads

let results = parallel_builder.build_batch(requests)?;
```

### Memory Optimization

```rust
use ddex_builder::memory_optimization::MemoryManager;

let memory_manager = MemoryManager::new(
    50 * 1024 * 1024  // 50MB limit
);

let optimized_builder = memory_manager.optimize_builder(builder);
```

## Performance Tuning

### General Guidelines

1. **Use appropriate presets** for your content type
2. **Enable parallel processing** for batch operations
3. **Use streaming mode** for large files
4. **Configure memory limits** appropriately
5. **Cache builders** when possible

### Configuration Examples

#### High-Throughput Configuration

```rust
let config = SecurityConfig {
    max_xml_size: 50_000_000,      // 50MB
    max_requests_per_minute: 1000,  // High rate limit
    rate_limiting_enabled: true,
    ..Default::default()
};
```

#### Memory-Constrained Environment

```rust
let config = SecurityConfig {
    max_xml_size: 1_000_000,       // 1MB limit
    max_json_depth: 16,            // Shallow nesting
    ..Default::default()
};
```

#### Batch Processing

```rust
use ddex_builder::parallel_processing::ParallelBuilder;

let parallel_builder = ParallelBuilder::new(
    builder,
    num_cpus::get()  // Use all available cores
);

// Process multiple requests in parallel
let results = parallel_builder.build_batch(requests)?;
```

### Performance Benchmarks

| Operation | Size | Performance | Memory |
|-----------|------|-------------|---------|
| Parse 10KB | 10KB | <5ms | 2MB |
| Parse 100KB | 100KB | <10ms | 5MB |
| Parse 1MB | 1MB | <50ms | 15MB |
| Build release | ~50KB | <15ms | 8MB |
| Batch (100 releases) | 5MB | <200ms | 50MB |

## Troubleshooting

### Common Issues

#### Build Errors

**Issue**: `BuildError::InvalidFormat`
```
Solution: Check your XML structure and ensure all required fields are present.
```

**Issue**: `BuildError::Security`
```
Solution: Review your input for potential security issues (XXE, path traversal, etc.).
```

**Issue**: `BuildError::Validation`
```
Solution: Check validation errors and ensure your data meets DDEX requirements.
```

#### Performance Issues

**Issue**: Slow build times
```
Solution: 
- Use appropriate presets
- Enable parallel processing for batch operations
- Consider streaming mode for large files
```

**Issue**: High memory usage
```
Solution:
- Set memory limits in MemoryManager
- Use streaming mode
- Process in smaller batches
```

#### Security Issues

**Issue**: Rate limiting triggered
```
Solution:
- Implement exponential backoff
- Distribute load across multiple clients
- Increase rate limits if appropriate
```

### Debug Mode

Enable debug logging for troubleshooting:

```rust
use tracing_subscriber;

// Enable debug logging
tracing_subscriber::fmt::init();

// Your DDEX Builder code here
```

### Validation Issues

Check validation results:

```rust
match result {
    Err(BuildError::Validation(errors)) => {
        for error in errors {
            println!("Field: {}", error.code);
            println!("Message: {}", error.message);
            if let Some(location) = error.location {
                println!("Location: {}", location);
            }
        }
    }
    _ => {}
}
```

## Migration Guide

### From Other DDEX Libraries

#### From xml-ddex

```rust
// Old (xml-ddex)
let ddex = XMLDdex::new();
let result = ddex.build(data);

// New (ddex-builder)
let mut builder = Builder::new();
builder.preset("universal_basic")?;
let result = builder.build_internal(&request)?;
```

#### From custom XML generators

```rust
// Replace custom XML generation
let xml_output = generate_custom_ddex(data);

// With DDEX Builder
let request = BuildRequest {
    source_xml: your_data_as_xml,
    output_format: OutputFormat::Xml,
    preset: Some("appropriate_preset".to_string()),
    validate_schema: true,
};

let result = builder.build_internal(&request)?;
```

### Breaking Changes in v1.0

1. **Preset names updated** - Check available_presets() for current names
2. **Security enabled by default** - Configure SecurityConfig if needed
3. **Stricter validation** - Some previously accepted XML may now be rejected
4. **API changes** - Some method signatures updated for better type safety

### Migration Checklist

- [ ] Update preset names to new format
- [ ] Configure security settings for your environment
- [ ] Test with strict validation enabled
- [ ] Update error handling for new error types
- [ ] Review performance configuration
- [ ] Test version conversion if using multiple DDEX versions

---

## Next Steps

- 📖 Read the [Developer Guide](developer-guide.md) for advanced usage
- 🔧 Check out [Examples](../examples/) for more code samples
- 🏗️ Explore the [API Documentation](https://docs.rs/ddex-builder)
- 🛡️ Review [Security Policy](../SECURITY.md) for production deployments
- 🚀 See [Performance Guide](performance-guide.md) for optimization tips

Need help? Check our [GitHub Issues](https://github.com/daddykev/ddex-suite/issues) or [Discord Community](https://discord.gg/ddex-builder).