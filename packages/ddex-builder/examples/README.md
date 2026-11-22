# DDEX Builder Examples

This directory contains comprehensive, real-world examples demonstrating how to use DDEX Builder for various music distribution scenarios. Each example is extensively commented and includes error handling patterns suitable for production use.

## 📚 Available Examples

### 🎵 [Spotify Album Example](spotify_album_example.rs)
**Multi-track album release built with `DDEXBuilder`**

```bash
cargo run --example spotify_album_example
```

### 📡 [Streaming Catalog Example](streaming_catalog_example.rs)
**Large-catalog delivery with `ddex_builder::streaming::StreamingBuilder`**

Writes thousands of resources and releases straight to a file without holding
the whole message in memory, with a progress callback and final statistics:

```bash
cargo run --example streaming_catalog_example -- 5000   # number of releases
```

### 🔒 [Error Sanitization Demo](error_sanitization_demo.rs)
**How build errors are sanitized for production vs development**

```bash
cargo run --example error_sanitization_demo
```

## 🚀 Running Examples

### Prerequisites

Make sure you have DDEX Builder installed:

```bash
cd /path/to/ddex-suite/packages/ddex-builder
cargo build --release
```

### Running Individual Examples

```bash
# Run a specific example
cargo run --example spotify_album_example

# Run with release optimizations (faster)
cargo run --release --example spotify_album_example

# Run all examples
for example in spotify_album_example streaming_catalog_example error_sanitization_demo; do
    echo "Running $example..."
    cargo run --example "$example"
done
```

### Example Output

Each example generates XML files and detailed console output:

```text
🎵 DDEX Builder - Generic Album Example
Creating a complete album release for broad distribution...

✅ Applied Generic Audio 4.3 preset
   • ERN 4.3 schema validation enabled
   • Standard field requirements active
   • High-quality audio validation enabled

📀 Album Information:
   📀 Album: 'Digital Horizons'
   🎤 Artist: The Wavelength Collective
   🏷️  Label: Indie Digital Records
   🎵 Tracks: 8
   📅 Release Date: 2024-03-15
   🌍 Territory: Worldwide

🔨 Building DDEX XML...
✅ Successfully built DDEX release
   📄 XML size: 45 KB
   ⏱️  Generation time: 12ms

🔍 Validating DDEX compliance...
✅ All standard compliance checks passed

💾 Saved to: generic_album_example.xml

🎯 DDEX Compliance Summary:
  📋 DDEX Version: ERN 4.3 ✅
  🎵 Message Profile: Audio Album ✅
  🌍 Territory: Worldwide ✅
  💿 Audio Format: FLAC ✅
  🎶 Track Count: 8 ✅
  🏷️  ISRC Codes: Present ✅
  ⏱️  Durations: Present ✅
  🎚️  Audio Quality: Specified ✅
  📡 Streaming Rights: Enabled ✅
  💳 Subscription Model: Enabled ✅

🎉 Album is ready for distribution!
```

## 📖 Common Patterns

### Error Handling

All examples demonstrate robust error handling:

```rust
// Pattern 1: Graceful preset application
if let Err(e) = builder.preset("spotify_audio_43") {
    eprintln!("❌ Failed to apply preset: {}", e);
    eprintln!("💡 Check that the preset exists and is valid");
    return Err(e.into());
}

// Pattern 2: Build with detailed error context
let result = match builder.build_internal(&request) {
    Ok(result) => {
        println!("✅ Build successful: {} KB", result.xml.len() / 1024);
        result
    },
    Err(e) => {
        eprintln!("❌ Build failed: {}", e);
        eprintln!("💡 Verify all required fields are present");
        return Err(e.into());
    }
};

// Pattern 3: Validation with user-friendly messages
if let Err(e) = validate_compliance(&result.xml) {
    eprintln!("❌ Validation failed: {}", e);
    eprintln!("💡 Review platform-specific requirements");
    return Err(e);
}
```

### Performance Monitoring

Examples show how to track performance:

```rust
let start = std::time::Instant::now();
let result = builder.build_internal(&request)?;
let duration = start.elapsed();

println!("⏱️  Build time: {}ms", duration.as_millis());
println!("📊 Throughput: {:.1} KB/s", 
    (result.xml.len() as f64) / duration.as_secs_f64() / 1024.0);
```

### Validation Patterns

Comprehensive validation approaches:

```rust
// Validate schema compliance
builder.validate_schema(&result.xml)?;

// Validate platform-specific requirements  
validate_spotify_compliance(&result.xml)?;

// Validate business rules
validate_release_metadata(&album_data)?;

// Validate audio quality requirements
validate_audio_specifications(&audio_specs)?;
```

## 🔧 Customization

### Creating Your Own Examples

1. **Copy an existing example** as a starting point
2. **Modify the metadata** for your use case
3. **Update the preset** for your target platform
4. **Add custom validation** for your requirements
5. **Test thoroughly** with real metadata

### Example Template

```rust
//! # Your Custom Example
//! 
//! Brief description of what this example demonstrates.

use ddex_builder::{Builder, BuildRequest};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🎵 Your Custom Example");
    
    // Step 1: Initialize builder
    let mut builder = Builder::new();
    builder.preset("your_preset")?;
    
    // Step 2: Create request
    let request = create_your_request();
    
    // Step 3: Build and validate
    let result = builder.build_internal(&request)?;
    validate_your_requirements(&result.xml)?;
    
    // Step 4: Save and report
    std::fs::write("your_output.xml", &result.xml)?;
    println!("✅ Success!");
    
    Ok(())
}

fn create_your_request() -> BuildRequest {
    // Your custom request logic here
    todo!("Implement your request creation")
}

fn validate_your_requirements(xml: &str) -> Result<(), Box<dyn Error>> {
    // Your custom validation logic here
    todo!("Implement your validation")
}
```

## 🐛 Troubleshooting

### Common Issues

**Preset not found**
```
❌ Failed to apply preset: Preset 'custom_preset' not found
💡 Check available presets with builder.available_presets()
```

**Missing required fields**
```
❌ Build failed: Missing required field 'ISRC'
💡 Verify all required fields are present in your request
```

**Validation failure**
```
❌ Validation failed: Invalid ISRC format
💡 ISRC must follow pattern: [A-Z]{2}[A-Z0-9]{3}[0-9]{7}
```

### Debug Mode

Run examples with debug output:

```bash
RUST_LOG=debug cargo run --example spotify_album_example
```

### File Inspection

Examine generated XML files:

```bash
# Pretty-print XML
xmllint --format spotify_album_example.xml

# Validate against schema
xmllint --schema ern43.xsd spotify_album_example.xml

# Compare two releases
diff -u release1.xml release2.xml
```

## 🤝 Contributing

We welcome contributions to improve these examples:

1. **Add new platform examples** (YouTube Music, etc.)
2. **Improve error messages** and user guidance
3. **Add more real-world scenarios** (classical, podcasts, etc.)
4. **Enhance validation** with additional checks
5. **Update documentation** based on user feedback

### Contribution Guidelines

- Follow the existing example structure
- Include comprehensive comments
- Add realistic test data
- Provide clear error messages
- Test with multiple scenarios

## 📧 Support

If you have questions about these examples:

- **Documentation**: [User Guide](../docs/user-guide.md)
- **API Reference**: [docs.rs/ddex-builder](https://docs.rs/ddex-builder)
- **Issues**: [GitHub Issues](https://github.com/daddykev/ddex-suite/issues)
- **Discussions**: [GitHub Discussions](https://github.com/daddykev/ddex-suite/discussions)

---

**Happy DDEX Building! 🎵**

*These examples demonstrate the power and flexibility of DDEX Builder for real-world music distribution scenarios.*