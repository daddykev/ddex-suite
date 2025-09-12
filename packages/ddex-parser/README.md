# DDEX Parser

[![Crates.io](https://img.shields.io/crates/v/ddex-parser)](https://crates.io/crates/ddex-parser)
[![npm version](https://img.shields.io/npm/v/ddex-parser.svg)](https://www.npmjs.com/package/ddex-parser)
[![PyPI version](https://img.shields.io/pypi/v/ddex-parser.svg)](https://pypi.org/project/ddex-parser/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-ddex--suite-blue)](https://github.com/daddykev/ddex-suite)

High-performance DDEX XML parser with native bindings for JavaScript, Python, and browser support via WASM. Parse DDEX files up to 15x faster than traditional parsers with built-in security features, comprehensive metadata extraction, and perfect round-trip compatibility with ddex-builder.

Part of the [DDEX Suite](https://github.com/daddykev/ddex-suite) - a comprehensive toolkit for working with DDEX metadata in the music industry.

> **Version 0.3.5** - Security & Stability Release with PyO3 0.24 compatibility and enhanced performance optimizations.

## 🚀 Language Support

Choose your preferred language and get started immediately:

| Language | Package | Installation |
|----------|---------|-------------|
| **JavaScript/TypeScript** | [ddex-parser (npm)](https://www.npmjs.com/package/ddex-parser) | `npm install ddex-parser` |
| **Python** | [ddex-parser (PyPI)](https://pypi.org/project/ddex-parser/) | `pip install ddex-parser` |
| **Rust** | [ddex-parser (crates.io)](https://crates.io/crates/ddex-parser) | `cargo add ddex-parser` |

## Quick Start

### JavaScript/TypeScript

```typescript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();
const result = await parser.parseFile('release.xml');

console.log(`Release: ${result.flattened.releaseTitle}`);
console.log(`Artist: ${result.flattened.mainArtist}`);
console.log(`Tracks: ${result.flattened.tracks.length}`);
```

### Python

```python
from ddex_parser import DDEXParser
import pandas as pd

parser = DDEXParser()
result = parser.parse_file("release.xml")

print(f"Release: {result.release_title}")
print(f"Artist: {result.main_artist}")

# Convert to DataFrame for analysis
tracks_df = result.to_dataframe()
print(tracks_df.head())
```

### Rust

```rust
use ddex_parser::DDEXParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = DDEXParser::new();
    let result = parser.parse_file("release.xml")?;
    
    println!("Release: {}", result.flattened.release_title);
    println!("Artist: {}", result.flattened.main_artist);
    println!("Tracks: {}", result.flattened.tracks.len());
    
    Ok(())
}
```

## Core Features

### 🚀 Blazing Performance
- **Up to 15x faster** than traditional XML parsers
- Native Rust core with optimized language bindings
- Streaming support for large files (>100MB)
- Memory-efficient processing with configurable limits

### 🔒 Security First
- Built-in XXE (XML External Entity) protection
- Entity expansion limits (billion laughs protection)
- Deep nesting protection
- Memory-bounded parsing with timeout controls

### 🎭 Dual Model Architecture
- **Graph Model**: Faithful DDEX structure with references (perfect for compliance)
- **Flattened Model**: Developer-friendly denormalized data (easy to consume)
- Full round-trip fidelity between both representations

### 🌐 Cross-Platform Compatibility
- **Node.js 16+** with native addon performance
- **Browser support** via optimized WASM (<500KB)
- **Python 3.8+** with comprehensive type hints
- **TypeScript-first** with complete type definitions

### 🎵 Music Industry Ready
- Support for all DDEX ERN versions (3.8.2, 4.2, 4.3+)
- Complete metadata extraction (releases, tracks, artists, rights)
- Territory and deal information parsing
- Image and audio resource handling
- Genre, mood, and classification support

## Performance Benchmarks

Performance comparison across environments:

### Native Performance (Node.js/Python)
| File Size | ddex-parser | Traditional | Speedup | Memory |
|-----------|-------------|-------------|---------|----------|
| 10KB      | 0.8ms       | 12ms        | 15x     | -70%     |
| 100KB     | 3ms         | 45ms        | 15x     | -65%     |
| 1MB       | 28ms        | 420ms       | 15x     | -60%     |
| 10MB      | 180ms       | 2.8s        | 16x     | -55%     |

### Browser Performance (WASM)
| File Size | ddex-parser | DOMParser | xml2js | Bundle Size |
|-----------|-------------|-----------|---------|-------------|
| 10KB      | 2.1ms       | 12ms      | 25ms    | 489KB       |
| 100KB     | 8ms         | 85ms      | 180ms   | (gzipped)   |
| 1MB       | 65ms        | 750ms     | 1.8s    |             |

## Security

v0.3.5 includes comprehensive security enhancements:
- XXE (XML External Entity) protection
- Entity expansion limits (billion laughs protection)
- Deep nesting protection
- Memory-bounded streaming
- Supply chain security with cargo-deny and SBOM
- Zero vulnerabilities, forbids unsafe code

## Getting Started

### Installation Guides

- **[JavaScript/TypeScript →](https://github.com/daddykev/ddex-suite/blob/main/packages/ddex-parser/bindings/node/README.md)** - npm package with Node.js and browser support
- **[Python →](https://github.com/daddykev/ddex-suite/blob/main/packages/ddex-parser/bindings/python/README.md)** - PyPI package with pandas integration
- **[Rust →](https://github.com/daddykev/ddex-suite/blob/main/packages/ddex-parser/README.md)** - Crates.io package documentation

### Round-Trip Compatibility

Perfect integration with ddex-builder for complete workflows:

```typescript
import { DDEXParser } from 'ddex-parser';
import { DDEXBuilder } from 'ddex-builder';

// Parse existing DDEX file
const parser = new DDEXParser();
const original = await parser.parseFile('input.xml');

// Modify data
const modified = { ...original.flattened };
modified.tracks[0].title = "New Title";

// Build new DDEX file with deterministic output
const builder = new DDEXBuilder();
const newXML = await builder.buildFromFlattened(modified);

// Verify round-trip integrity
const reparsed = await parser.parseString(newXML);
assert.deepEqual(reparsed.tracks[0].title, "New Title"); // ✅ Perfect fidelity
```

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE) file for details.

## Related Projects

- **[ddex-builder](https://crates.io/crates/ddex-builder)** - Build deterministic DDEX XML files
- **[DDEX Suite](https://ddex-suite.org)** - Complete DDEX processing toolkit
- **[DDEX Workbench](https://ddex-workbench.org)** - Official DDEX validation tools

---

Built with ❤️ for the music industry. Powered by Rust for maximum performance and safety.