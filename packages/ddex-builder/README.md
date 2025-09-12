# DDEX Builder

[![Crates.io](https://img.shields.io/crates/v/ddex-builder)](https://crates.io/crates/ddex-builder)
[![npm version](https://img.shields.io/npm/v/ddex-builder.svg)](https://www.npmjs.com/package/ddex-builder)
[![PyPI version](https://img.shields.io/pypi/v/ddex-builder.svg)](https://pypi.org/project/ddex-builder/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-ddex--suite-blue)](https://github.com/daddykev/ddex-suite)

Generate deterministic, industry-compliant DDEX XML files with byte-perfect reproducibility. Build DDEX messages from structured data with comprehensive validation, partner-specific presets, and perfect round-trip compatibility with ddex-parser.

Part of the [DDEX Suite](https://github.com/daddykev/ddex-suite) - a comprehensive toolkit for working with DDEX metadata in the music industry.

## 🚀 Language Support

Choose your preferred language and get started immediately:

| Language | Package | Installation |
|----------|---------|-------------|
| **JavaScript/TypeScript** | [ddex-builder (npm)](https://www.npmjs.com/package/ddex-builder) | `npm install ddex-builder` |
| **Python** | [ddex-builder (PyPI)](https://pypi.org/project/ddex-builder/) | `pip install ddex-builder` |
| **WebAssembly** | [Browser/WASM](./bindings/wasm/) | CDN or local build |
| **Rust** | [ddex-builder (crates.io)](https://crates.io/crates/ddex-builder) | `cargo add ddex-builder` |

## Quick Start

### JavaScript/TypeScript

```typescript
import { DDEXBuilder } from 'ddex-builder';

const builder = new DDEXBuilder({ validate: true, preset: 'youtube' });

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

builder = DDEXBuilder(validate=True, preset='youtube')

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

### WebAssembly (Browser)

```javascript
// Load from CDN or local build
import init, { DdexBuilder } from './pkg/ddex_builder_wasm.js';

async function generateDDEX() {
  // Initialize WASM module
  await init();
  const builder = new DdexBuilder();
  
  const releaseData = {
    release_id: 'RELEASE_2024_001',
    title: 'Amazing Album',
    artist: 'Incredible Artist',
    release_date: '2024-01-01',
    tracks: [{
      title: 'Hit Song',
      duration: 195,
      isrc: 'US1234567890'
    }]
  };
  
  const xml = builder.build_release_simple(JSON.stringify(releaseData));
  console.log('Generated deterministic DDEX XML:', xml.length, 'bytes');
  return xml;
}
```

### Rust

```rust
use ddex_builder::DDEXBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder = DDEXBuilder::new()
        .with_validation(true)
        .with_preset("youtube");
    
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
- **YouTube Music**: Content ID and monetization standards
- **Generic**: Default preset for broad distributor compatibility

### 🌐 Cross-Platform Compatibility
- **Node.js 16+** with native addon performance  
- **Python 3.8+** with comprehensive type hints
- **Browser support** via optimized WASM (114KB gzipped)
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

## Performance Benchmarks

Performance comparison across environments and languages:

### Build Performance
| Dataset Size | Node.js | Python | Rust | Browser (WASM) |
|--------------|---------|---------|----- |----------------|
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

## Security

v0.3.5 includes comprehensive security enhancements:
- Zero vulnerabilities, forbids unsafe code
- Supply chain security with cargo-deny and SBOM
- Memory-bounded processing with configurable limits
- Built-in validation prevents malformed output
- Deterministic generation prevents injection attacks

## Getting Started

### Installation Guides

- **[JavaScript/TypeScript →](./bindings/node/README.md)** - npm package with Node.js and browser support
- **[Python →](./bindings/python/README.md)** - PyPI package with pandas integration
- **[WebAssembly →](./bindings/wasm/README.md)** - Browser-ready WASM bundle with examples
- **[Rust →](../../README.md#rust)** - Crates.io package documentation

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

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE) file for details.

## Related Projects

- **[ddex-parser](https://crates.io/crates/ddex-parser)** - Parse DDEX XML files to structured data
- **[DDEX Suite](https://ddex-suite.org)** - Complete DDEX processing toolkit
- **[DDEX Workbench](https://ddex-workbench.org)** - Official DDEX validation tools

---

Built with ❤️ for the music industry. Engineered for deterministic, industry-grade DDEX generation.