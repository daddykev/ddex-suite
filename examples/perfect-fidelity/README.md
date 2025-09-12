# Perfect Fidelity Engine Examples

This directory contains comprehensive examples demonstrating the Perfect Fidelity Engine capabilities of the DDEX Suite. These examples showcase how to achieve 100% round-trip fidelity with DDEX XML files while preserving all original features including comments, extensions, formatting, and metadata.

## 🎯 Overview

The Perfect Fidelity Engine ensures that DDEX XML files can be:
- **Parsed** with complete preservation of all elements and attributes
- **Modified** with surgical precision while maintaining structural integrity  
- **Built** with deterministic, byte-perfect output
- **Verified** through comprehensive round-trip testing

## 📁 Examples Included

### 1. 🔄 Round-trip Example (`round_trip_example.rs`)

**Demonstrates**: Complete round-trip capability with all preservation features

**Features showcased**:
- Perfect Fidelity configuration setup
- XML comments preservation in original positions
- Processing instructions preservation
- Partner extensions (Spotify, Apple, YouTube, Amazon)
- Custom label extensions with complex metadata
- Original attribute ordering preservation
- Namespace prefix preservation
- Multiple canonicalization algorithms (C14N, C14N11, DB-C14N)
- Build verification and statistics collection
- Performance benchmarking

**Key highlights**:
```rust
// Perfect Fidelity configuration
let fidelity_options = FidelityOptions {
    enable_perfect_fidelity: true,
    preserve_comments: true,
    preserve_processing_instructions: true,
    preserve_extensions: true,
    preserve_attribute_order: true,
    preserve_namespace_prefixes: true,
    canonicalization: CanonicalizationAlgorithm::DbC14N,
    enable_verification: true,
    collect_statistics: true,
    // ... other options
};
```

**Run example**:
```bash
cargo run --example round_trip_example --release
```

### 2. 📐 Canonicalization Example (`canonicalization_example.rs`)

**Demonstrates**: Various canonicalization algorithms and their effects

**Features showcased**:
- XML Canonicalization (C14N) 1.0 and 1.1 standards
- DDEX-specific DB-C14N/1.0 algorithm
- Custom canonicalization rules and configurations
- Deterministic output verification
- Canonicalization consistency testing across multiple runs
- Performance comparison between algorithms
- Namespace minimization and optimization
- Attribute ordering normalization

**Key highlights**:
```rust
// Custom canonicalization rules
let mut custom_rules = CustomCanonicalizationRules::default();
custom_rules.preserve_whitespace = true;
custom_rules.sort_attributes = true;
custom_rules.minimize_namespaces = true;

// DDEX-specific attribute ordering
custom_rules.attribute_ordering = vec![
    "MessageSchemaVersionId".to_string(),
    "BusinessTransactionId".to_string(),
    "LanguageAndScriptCode".to_string(),
    // ... more DDEX-specific ordering
];
```

**Run example**:
```bash
cargo run --example canonicalization_example --release
```

### 3. 🔌 Extension Handling Example (`extension_handling_example.rs`)

**Demonstrates**: Comprehensive partner and custom extension handling

**Features showcased**:
- **Spotify** extensions: Track IDs, metadata, audio features, playlists
- **Apple Music** extensions: ADAM IDs, spatial audio, lossless formats
- **YouTube Music** extensions: Video IDs, channel data, content metadata
- **Amazon Music** extensions: ASINs, Prime eligibility, HD formats
- **Custom label** extensions: Internal workflows, production metadata
- **Mixed partner** scenarios with potential conflicts
- **Unknown extension** handling strategies (preserve, drop, validate, generalize)
- Extension validation and verification
- Extension conflict resolution
- Extension fidelity analysis and performance

**Key highlights**:
```rust
// Extension preservation configuration
let extension_config = ExtensionPreservationConfig {
    enabled: true,
    known_extensions: vec![
        "http://spotify.com/ddex".to_string(),
        "http://apple.com/ddex".to_string(),
        "http://youtube.com/ddex".to_string(),
        "http://amazon.com/ddex".to_string(),
    ],
    unknown_extension_handling: UnknownExtensionHandling::Preserve,
    preserve_extension_attributes: true,
    // ... validation configuration
};
```

**Run example**:
```bash
cargo run --example extension_handling_example --release
```

### 4. 🌊 Large File Streaming Example (`large_file_streaming_example.rs`)

**Demonstrates**: Efficient processing of large DDEX files with memory optimization

**Features showcased**:
- **Memory-bounded streaming** for 100MB+ files
- **Incremental processing** with partial results
- **Performance optimization** strategies for different scenarios
- **Memory usage monitoring** and optimization
- **Stress testing** with extremely large files (10K+ releases)
- **Streaming configuration** options for different use cases
- **Timeout handling** for processing constraints
- **Memory efficiency** analysis and recommendations

**Key highlights**:
```rust
// Memory-optimized fidelity options for large files
let fidelity_options = FidelityOptions {
    enable_perfect_fidelity: true,
    preserve_comments: false,      // Disabled for performance
    preserve_extensions: true,      // Keep essential features
    collect_statistics: false,     // Reduce memory overhead
    enable_verification: false,    // Skip for streaming
    canonicalization: CanonicalizationAlgorithm::DbC14N,
    // ... other streaming optimizations
};
```

**Run example**:
```bash
cargo run --example large_file_streaming_example --release
```

## 🚀 Quick Start Guide

### Prerequisites

1. **Rust**: Install Rust 1.70+ from [rustup.rs](https://rustup.rs/)
2. **Dependencies**: All required dependencies are included in `Cargo.toml`

### Running Examples

```bash
# Navigate to the project root
cd /path/to/ddex-suite

# Run all examples
cargo run --example round_trip_example --release
cargo run --example canonicalization_example --release
cargo run --example extension_handling_example --release
cargo run --example large_file_streaming_example --release

# Run with specific logging (optional)
RUST_LOG=debug cargo run --example round_trip_example --release
```

### Understanding Output

Each example provides detailed console output showing:

- ✅ **Success indicators** for successful operations
- ❌ **Failure indicators** with detailed error messages
- ⏱️ **Performance metrics** (processing time, throughput)
- 📊 **Statistics** (file sizes, element counts, memory usage)
- 💡 **Recommendations** for optimization
- 🔍 **Analysis results** with fidelity scores

## 📊 Performance Targets

The Perfect Fidelity Engine is designed to meet these performance targets:

| File Size | Parse Time | Build Time | Memory Usage | Throughput |
|-----------|------------|------------|--------------|------------|
| 0-10KB    | <5ms      | <10ms      | <10MB       | >2 MB/s    |
| 10-100KB  | <20ms     | <50ms      | <50MB       | >5 MB/s    |
| 100KB-1MB | <100ms    | <200ms     | <200MB      | >10 MB/s   |
| 1-10MB    | <500ms    | <1s        | <500MB      | >20 MB/s   |
| 10-100MB  | <5s       | <10s       | <1GB        | >20 MB/s   |

## 🎛️ Configuration Options

### Perfect Fidelity Options

```rust
pub struct FidelityOptions {
    // Core fidelity features
    pub enable_perfect_fidelity: bool,           // Master switch
    pub preserve_comments: bool,                 // XML comments
    pub preserve_processing_instructions: bool,  // PIs
    pub preserve_extensions: bool,               // Partner extensions
    pub preserve_attribute_order: bool,          // Attribute ordering
    pub preserve_namespace_prefixes: bool,       // Namespace prefixes
    
    // Canonicalization
    pub canonicalization: CanonicalizationAlgorithm,
    pub custom_canonicalization_rules: Option<CustomCanonicalizationRules>,
    
    // Performance and verification
    pub collect_statistics: bool,                // Performance stats
    pub enable_verification: bool,               // Build verification
    pub enable_deterministic_ordering: bool,    // Deterministic output
}
```

### Canonicalization Algorithms

- **`None`**: No canonicalization (preserves exact formatting)
- **`C14N`**: W3C XML Canonicalization 1.0
- **`C14N11`**: W3C XML Canonicalization 1.1  
- **`DbC14N`**: DDEX-specific DB-C14N/1.0 algorithm (recommended)
- **`Custom`**: User-defined canonicalization rules

### Extension Handling Strategies

- **`Preserve`**: Keep all extensions as-is (default)
- **`Drop`**: Remove unknown extensions
- **`ValidateAndPreserve`**: Validate against schema first
- **`Generalize`**: Convert to generic extension format

## 🧪 Testing Your Own Files

### Using the Examples with Custom Files

```rust
// In any example, replace the generated content with your file
let your_xml = std::fs::read_to_string("path/to/your/ddex/file.xml")?;

// Test round-trip fidelity
let builder = Builder::with_perfect_fidelity();
let result = builder.test_round_trip_fidelity(&your_xml)?;

if result.success {
    println!("✅ Your file maintains perfect fidelity!");
} else {
    println!("⚠️  Fidelity issues found:");
    for diff in result.differences {
        println!("  - {}", diff);
    }
}
```

### Custom Extension Testing

```rust
// Test your custom extensions
let extension_config = ExtensionPreservationConfig {
    known_extensions: vec![
        "http://your-label.com/ddex".to_string(),
        "http://your-service.com/ddex".to_string(),
    ],
    unknown_extension_handling: UnknownExtensionHandling::Preserve,
    // ... other settings
};
```

## 📚 Additional Resources

### Related Documentation

- **[DDEX Standards](https://kb.ddex.net/)**: Official DDEX documentation
- **[XML Canonicalization](https://www.w3.org/TR/xml-c14n/)**: W3C C14N specification
- **[DB-C14N Specification](docs/DB-C14N-1.0-spec.md)**: DDEX-specific canonicalization
- **[Perfect Fidelity Guide](docs/perfect-fidelity-guide.md)**: Comprehensive usage guide

### Performance Optimization Guides

1. **Memory Optimization**: 
   - Disable statistics collection for large files
   - Skip verification for trusted sources  
   - Use streaming mode for files >100MB

2. **Speed Optimization**:
   - Disable comment preservation if not needed
   - Skip canonicalization for internal processing
   - Use performance-optimized fidelity options

3. **Fidelity Optimization**:
   - Enable all preservation features
   - Use DB-C14N canonicalization
   - Enable verification for critical workflows

### Troubleshooting Common Issues

#### Performance Issues

```rust
// For large files, use streaming optimized settings
let fidelity_options = FidelityOptions {
    enable_perfect_fidelity: true,
    preserve_comments: false,        // Disable for performance
    collect_statistics: false,       // Reduce memory usage
    enable_verification: false,      // Skip verification
    canonicalization: CanonicalizationAlgorithm::DbC14N,
    ..Default::default()
};
```

#### Memory Issues

```rust
// For memory-constrained environments
let fidelity_options = FidelityOptions {
    enable_perfect_fidelity: false,  // Trade fidelity for memory
    preserve_extensions: true,       // Keep only essential features
    canonicalization: CanonicalizationAlgorithm::None,
    collect_statistics: false,
    ..Default::default()
};
```

#### Extension Issues

```rust
// For files with unknown extensions
let extension_config = ExtensionPreservationConfig {
    unknown_extension_handling: UnknownExtensionHandling::Preserve,
    validate_uris: false,           // Skip URI validation if causing issues
    max_extension_count: 1000,      // Increase limits if needed
    ..Default::default()
};
```

## 🤝 Contributing

These examples are designed to be educational and extensible. Feel free to:

- **Add new test cases** for specific scenarios
- **Extend examples** with additional features
- **Create specialized examples** for your use cases
- **Report issues** or suggestions for improvements

## 📄 License

These examples are part of the DDEX Suite and are licensed under the MIT License. See the main project LICENSE file for details.

---

**Happy DDEX processing with perfect fidelity! 🎵✨**