# DDEX Builder

[![Crates.io](https://img.shields.io/crates/v/ddex-builder-core.svg)](https://crates.io/crates/ddex-builder-core)
[![npm version](https://img.shields.io/npm/v/ddex-builder.svg)](https://www.npmjs.com/package/ddex-builder)
[![PyPI version](https://img.shields.io/pypi/v/ddex-builder.svg)](https://pypi.org/project/ddex-builder/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-ddex--suite-blue)](https://github.com/ddex-suite/ddex-suite)

Generate deterministic, industry-compliant DDEX XML files with byte-perfect reproducibility. Build DDEX messages from structured data with comprehensive validation, partner-specific presets, and perfect round-trip compatibility with ddex-parser.

Part of the [DDEX Suite](https://github.com/ddex-suite/ddex-suite) - a comprehensive toolkit for working with DDEX metadata in the music industry.

## 🚀 Language Support

Choose your preferred language and get started immediately:

| Language | Package | Installation |
|----------|---------|-------------|
| **JavaScript/TypeScript** | [ddex-builder (npm)](https://www.npmjs.com/package/ddex-builder) | `npm install ddex-builder` |
| **Python** | [ddex-builder (PyPI)](https://pypi.org/project/ddex-builder/) | `pip install ddex-builder` |
| **Rust** | [ddex-builder-core (crates.io)](https://crates.io/crates/ddex-builder-core) | `cargo add ddex-builder-core` |

## Quick Start

### JavaScript/TypeScript

```typescript
import { DDEXBuilder } from 'ddex-builder';

const builder = new DDEXBuilder({ validate: true, preset: 'universal' });

const releaseData = {
  messageHeader: {
    senderName: 'My Record Label',
    messageId: 'RELEASE_2024_001'
  },
  releases: [{
    title: 'Amazing Album',
    mainArtist: 'Incredible Artist',
    tracks: [{
      title: 'Hit Song',
      duration: 195,
      isrc: 'US1234567890'
    }]
  }]
};

const xml = await builder.buildFromObject(releaseData, { version: '4.3' });
console.log('Generated deterministic DDEX XML:', xml.length, 'bytes');
```

### Python

```python
from ddex_builder import DDEXBuilder
import pandas as pd

builder = DDEXBuilder(validate=True, preset='universal')

release_data = {
    'message_header': {
        'sender_name': 'My Record Label',
        'message_id': 'RELEASE_2024_001'
    },
    'releases': [{
        'title': 'Amazing Album',
        'main_artist': 'Incredible Artist',
        'tracks': [{
            'title': 'Hit Song',
            'duration': 195,
            'isrc': 'US1234567890'
        }]
    }]
}

xml = builder.build_from_dict(release_data, version='4.3')
print(f'Generated deterministic DDEX XML: {len(xml)} bytes')
```

### Rust

```rust
use ddex_builder_core::DDEXBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder = DDEXBuilder::new()
        .with_validation(true)
        .with_preset("universal");
    
    let release_data = serde_json::json!({
        "message_header": {
            "sender_name": "My Record Label",
            "message_id": "RELEASE_2024_001"
        },
        "releases": [{
            "title": "Amazing Album",
            "main_artist": "Incredible Artist",
            "tracks": [{
                "title": "Hit Song",
                "duration": 195,
                "isrc": "US1234567890"
            }]
        }]
    });
    
    let xml = builder.build_from_json(&release_data, "4.3")?;
    println!("Generated deterministic DDEX XML: {} bytes", xml.len());
    
    Ok(())
}
```

## Core Features

### 🎯 Deterministic Output
- **100% reproducible** XML generation with stable hash IDs
- **DB-C14N/1.0** canonicalization for byte-perfect consistency  
- **Content-addressable** resource IDs for reliable references
- **Stable ordering** ensures identical output across all platforms

### 🏭 Industry Presets
- **Spotify**: Streaming platform requirements and content flags
- **Apple Music**: iTunes Store compliance and specifications
- **YouTube Music**: Content ID and monetization standards
- **Amazon Music**: Prime Music and Unlimited requirements
- **Universal**: Generic preset for broad distributor compatibility

### 🌐 Universal Compatibility
- **Node.js 16+** with native addon performance  
- **Python 3.8+** with comprehensive type hints
- **Browser support** via optimized WASM (<400KB)
- **Rust native** for maximum performance and safety

### 🔒 Comprehensive Validation
- **Real-time DDEX schema validation** with detailed error messages
- **Business rule enforcement** for industry compliance
- **Reference integrity checking** across the entire message
- **Territory and rights validation** with suggestion engine

### 🚀 High Performance
- **Native Rust core** with optimized language bindings
- **Streaming generation** for large catalogs (>100,000 tracks)
- **Memory-efficient processing** with configurable limits
- **Parallel resource processing** for maximum throughput

## API Overview

All language bindings provide consistent APIs with language-specific optimizations:

### Common Operations

| Operation | JavaScript | Python | Rust |
|-----------|------------|--------|------|
| Build from object | `builder.buildFromObject(data)` | `builder.build_from_dict(data)` | `builder.build_from_json(data)` |
| Build from JSON | `builder.buildFromJSON(json)` | `builder.build_from_json(json)` | `builder.build_from_json_str(json)` |
| Validate | `builder.validate(data)` | `builder.validate(data)` | `builder.validate(data)` |
| Apply preset | `builder.applyPreset(name)` | `builder.apply_preset(name)` | `builder.with_preset(name)` |

### Deterministic Generation

All language bindings guarantee identical output:

```javascript
// JavaScript
const xml1 = await builder.buildFromObject(data, { version: '4.3' });
const xml2 = await builder.buildFromObject(data, { version: '4.3' });
console.assert(xml1 === xml2); // ✅ Byte-perfect reproducibility
```

```python
# Python
xml1 = builder.build_from_dict(data, version='4.3')
xml2 = builder.build_from_dict(data, version='4.3')  
assert xml1 == xml2  # ✅ Byte-perfect reproducibility
```

```rust
// Rust
let xml1 = builder.build_from_json(&data, "4.3")?;
let xml2 = builder.build_from_json(&data, "4.3")?;
assert_eq!(xml1, xml2); // ✅ Byte-perfect reproducibility
```

## Advanced Features

### Streaming for Large Catalogs

Process massive datasets with constant memory usage:

```typescript
// JavaScript/TypeScript - Stream from Node.js readable
import { createReadStream } from 'fs';

const fileStream = createReadStream('huge-catalog.json');
const xml = await builder.buildFromStream(fileStream, {
  version: '4.3',
  batchSize: 1000,
  progressCallback: (progress) => {
    console.log(`Progress: ${progress.percentage}% (${progress.itemsProcessed} items)`);
  }
});
```

```python
# Python - Stream from CSV with pandas
import pandas as pd

def build_streaming_catalog(csv_file: str) -> str:
    builder = DDEXBuilder(streaming=True)
    
    for chunk in pd.read_csv(csv_file, chunksize=1000):
        builder.process_chunk(chunk)
    
    return builder.finalize()

xml = build_streaming_catalog('massive_catalog.csv')
```

### Industry Preset System

Pre-configured settings for major platforms:

```typescript
// Apply Spotify preset for streaming optimization
const spotifyBuilder = new DDEXBuilder({ preset: 'spotify' });

// Automatically applies:
// - Explicit content flagging requirements
// - Territory-specific streaming rights  
// - Preferred genre normalization
// - Audio quality specifications
// - Spotify-specific metadata fields
```

```python
# Apply Apple Music preset for iTunes Store compliance
builder = DDEXBuilder(preset='apple_music')

# Automatically applies:
# - iTunes Store compliance rules
# - Mastered for iTunes requirements
# - Region-specific pricing tiers
# - Album artwork specifications
```

### Round-Trip Compatibility

Perfect integration with ddex-parser for complete workflows:

```typescript
import { DDEXParser } from 'ddex-parser';
import { DDEXBuilder } from 'ddex-builder';

// Parse existing DDEX file
const parser = new DDEXParser();
const original = await parser.parseFile('input.xml');

// Modify specific fields
const modified = { ...original.flattened };
modified.releases[0].title = 'Remastered Edition';

// Build new deterministic XML
const builder = new DDEXBuilder({ canonical: true });
const newXml = await builder.buildFromFlattened(modified);

// Perfect round-trip fidelity guaranteed
const reparsed = await parser.parseString(newXml);
console.assert(reparsed.releases[0].title === 'Remastered Edition');
```

## Performance Benchmarks

Performance comparison across environments and languages:

### Build Performance
| Dataset Size | Node.js | Python | Rust | Browser (WASM) |
|--------------|---------|---------|------|----------------|
| Single release (10 tracks) | 3ms | 5ms | 0.8ms | 8ms |
| Album catalog (100 releases) | 25ms | 40ms | 12ms | 85ms |
| Label catalog (1000 releases) | 180ms | 280ms | 95ms | 650ms |
| Large catalog (10000 releases) | 1.8s | 2.8s | 950ms | 6.5s |

### Memory Usage
| Dataset Size | Traditional XML | ddex-builder | Improvement |
|--------------|-----------------|--------------|-------------|
| 1000 releases | 450MB | 120MB | 73% less |
| 10000 releases | 4.2GB | 300MB | 93% less |
| 100000 releases | >16GB | 500MB* | >97% less |

*With streaming mode enabled

## Getting Started

### Installation Guides

- **[JavaScript/TypeScript →](./bindings/node/README.md)** - npm package with Node.js and browser support
- **[Python →](./bindings/python/README.md)** - PyPI package with pandas integration  
- **[Rust →](./README-rust.md)** - Crates.io package documentation

### Example Projects

- [Express.js API](./examples/express-api) - REST API for DDEX generation
- [Python Analytics](./examples/python-analytics) - Catalog generation from analytics data
- [React Generator](./examples/react-generator) - Interactive DDEX builder
- [Rust CLI](./examples/rust-cli) - Command-line DDEX builder

## Industry Presets Reference

### Spotify Preset
```yaml
name: "spotify"
description: "Optimized for Spotify streaming platform"
requirements:
  - explicit_content_flag: required
  - territory_restrictions: streaming_only
  - genre_normalization: spotify_taxonomy
  - audio_quality: preferred_formats
validation_rules:
  - isrc: required
  - duration: max_10_minutes
  - territories: ["WorldWide", "US", "EU"]
```

### Apple Music Preset
```yaml
name: "apple_music"
description: "iTunes Store compliance and specifications"
requirements:
  - mastered_for_itunes: preferred
  - pricing_tiers: region_specific
  - artwork: itunes_specifications
validation_rules:
  - upc: required_for_albums
  - isrc: required_for_tracks
  - explicit_flag: required
```

### YouTube Music Preset
```yaml
name: "youtube_music"
description: "Content ID and monetization requirements"
requirements:
  - content_id: enabled
  - monetization: standard_policies
  - territory_handling: youtube_specific
validation_rules:
  - track_references: required
  - usage_rights: monetization_compatible
```

### Custom Preset Creation

```typescript
import { DDEXBuilder, type CustomPreset } from 'ddex-builder';

const myLabelPreset: CustomPreset = {
  name: 'my_label',
  defaultTerritories: ['US', 'CA', 'GB'],
  requiredFields: {
    release: ['title', 'mainArtist', 'labelName', 'releaseDate'],
    track: ['title', 'duration', 'isrc', 'position']
  },
  validationRules: {
    maxTrackDuration: 600, // 10 minutes
    requireExplicitFlag: true,
    genreNormalization: ['Pop', 'Rock', 'Electronic', 'Hip-Hop']
  },
  businessRules: {
    enforceISRC: true,
    validateTerritoryRights: true,
    requireUPCForAlbums: true
  }
};

const builder = new DDEXBuilder({ preset: myLabelPreset });
```

## Migration Guides

### From v0.1.0 to v0.2.0

The v0.2.0 release introduced significant improvements:

```javascript
// v0.1.0 (deprecated)
import buildDdex from 'ddex-builder';
const xml = buildDdex(data, { version: '4.3' });

// v0.2.0+ (current)
import { DDEXBuilder } from 'ddex-builder';
const builder = new DDEXBuilder();
const xml = await builder.buildFromObject(data, { version: '4.3' });
```

**New in v0.2.0:**
- Deterministic output with DB-C14N/1.0
- Industry preset system
- Enhanced validation engine
- Streaming support for large datasets
- Round-trip compatibility with ddex-parser
- Full TypeScript support across all bindings

### From Other DDEX Tools

Migration helpers for common scenarios:

```python
# From manual XML generation
template = """<?xml version="1.0"?>
<ernm:NewReleaseMessage>
  <!-- Manual template approach -->
</ernm:NewReleaseMessage>"""

# To ddex-builder
from ddex_builder import DDEXBuilder
builder = DDEXBuilder(validate=True)
xml = builder.build_from_dict(structured_data, version='4.3')
```

## Error Handling

Comprehensive error types with detailed information:

```typescript
import { DDEXBuilder, ValidationError, BuilderError } from 'ddex-builder';

try {
  const xml = await builder.buildFromObject(data, { version: '4.3' });
} catch (error) {
  if (error instanceof ValidationError) {
    console.error('Validation failed:', error.details);
    error.fieldErrors.forEach(fieldError => {
      console.error(`${fieldError.field}: ${fieldError.message}`);
      if (fieldError.suggestions) {
        console.log('Suggestions:', fieldError.suggestions.join(', '));
      }
    });
  } else if (error instanceof BuilderError) {
    console.error('Build failed:', error.message);
  }
}
```

## Testing

Run comprehensive tests across all language bindings:

```bash
# Test all languages
npm test

# Test specific binding
cd bindings/node && npm test
cd bindings/python && python -m pytest
cargo test -p ddex-builder-core

# Run determinism tests
npm run test:determinism

# Run round-trip tests  
npm run test:round-trip
```

## Contributing

We welcome contributions! Please read our [Contributing Guide](./CONTRIBUTING.md) before submitting PRs.

### Development Setup

```bash
git clone https://github.com/ddex-suite/ddex-suite.git
cd ddex-suite/packages/ddex-builder

# Install dependencies
npm install
pip install -r requirements-dev.txt
cargo build

# Run tests
npm run test:all

# Test deterministic output
npm run test:determinism
```

## Roadmap

### Current Status (v0.2.0)
- ✅ Deterministic XML generation with DB-C14N/1.0
- ✅ Industry preset system (Spotify, Apple, YouTube, Amazon)
- ✅ Comprehensive validation engine
- ✅ JavaScript/TypeScript bindings (Node.js + Browser)
- ✅ Python bindings with pandas integration
- ✅ Round-trip compatibility with ddex-parser

### Upcoming Features

#### v0.3.0 (Q1 2025)
- **Partner API Integration**: Direct integration with distributor APIs
- **Batch Processing Engine**: Parallel processing of multiple releases  
- **Advanced Streaming**: Support for incremental updates
- **Schema Evolution**: Support for future DDEX versions

#### v0.4.0 (Q2 2025)  
- **AI-Powered Validation**: Smart error detection and correction
- **Performance Dashboard**: Real-time build metrics and optimization
- **Template System**: Pre-built templates for common release types
- **Multi-format Export**: Support for other metadata formats

#### v1.0.0 (Q3 2025)
- **Production Grade**: Enterprise-ready stability and performance
- **Complete Documentation**: Comprehensive guides and tutorials
- **Certification**: Official DDEX compliance testing
- **Community Ecosystem**: Plugin system and community presets

## Support

- 📖 [Full Documentation](https://ddex-suite.github.io/docs/)
- 🐛 [Report Issues](https://github.com/ddex-suite/ddex-suite/issues)
- 💬 [GitHub Discussions](https://github.com/ddex-suite/ddex-suite/discussions)
- 📧 Email: support@ddex-suite.com

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Related Projects

- **[ddex-parser](https://github.com/ddex-suite/ddex-suite/tree/main/packages/ddex-parser)** - Parse DDEX XML files to structured data
- **[DDEX Suite](https://github.com/ddex-suite/ddex-suite)** - Complete DDEX processing toolkit
- **[DDEX Workbench](https://github.com/ddex/ddex-workbench)** - Official DDEX validation tools

---

Built with ❤️ for the music industry. Engineered for deterministic, industry-grade DDEX generation.