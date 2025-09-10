# DDEX Parser

[![Crates.io](https://img.shields.io/crates/v/ddex-parser-core.svg)](https://crates.io/crates/ddex-parser-core)
[![npm version](https://img.shields.io/npm/v/ddex-parser.svg)](https://www.npmjs.com/package/ddex-parser)
[![PyPI version](https://img.shields.io/pypi/v/ddex-parser.svg)](https://pypi.org/project/ddex-parser/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-ddex--suite-blue)](https://github.com/ddex-suite/ddex-suite)

High-performance DDEX XML parser with native bindings for JavaScript, Python, and browser support via WASM. Parse DDEX files up to 15x faster than traditional parsers with built-in security features, comprehensive metadata extraction, and perfect round-trip compatibility with ddex-builder.

Part of the [DDEX Suite](https://github.com/ddex-suite/ddex-suite) - a comprehensive toolkit for working with DDEX metadata in the music industry.

## 🚀 Language Support

Choose your preferred language and get started immediately:

| Language | Package | Installation |
|----------|---------|-------------|
| **JavaScript/TypeScript** | [ddex-parser (npm)](https://www.npmjs.com/package/ddex-parser) | `npm install ddex-parser` |
| **Python** | [ddex-parser (PyPI)](https://pypi.org/project/ddex-parser/) | `pip install ddex-parser` |
| **Rust** | [ddex-parser-core (crates.io)](https://crates.io/crates/ddex-parser-core) | `cargo add ddex-parser-core` |

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
use ddex_parser_core::DDEXParser;

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

### 🌐 Universal Compatibility
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

## API Overview

All language bindings provide consistent APIs with language-specific optimizations:

### Common Operations

| Operation | JavaScript | Python | Rust |
|-----------|------------|--------|------|
| Parse file | `parser.parseFile(path)` | `parser.parse_file(path)` | `parser.parse_file(path)` |
| Parse string | `parser.parseString(xml)` | `parser.parse_string(xml)` | `parser.parse_string(xml)` |
| To DataFrame | `result.toDataFrame()` | `result.to_dataframe()` | `result.to_dataframe()` |
| To JSON | `result.toJSON()` | `result.to_json()` | `result.to_json()` |

### Dual Model Access

```javascript
// Graph model (faithful DDEX structure)
const messageId = result.graph.messageHeader.messageId;
const parties = result.graph.parties;

// Flattened model (developer-friendly)
const releaseTitle = result.flattened.releaseTitle;
const tracks = result.flattened.tracks;
```

## Advanced Features

### Streaming for Large Catalogs

Process massive DDEX files with constant memory usage:

```typescript
// JavaScript/TypeScript
for await (const batch of parser.parseStream(stream)) {
  console.log(`Processing ${batch.releases.length} releases...`);
}
```

```python
# Python
async for batch in parser.parse_async("large_catalog.xml"):
    print(f"Processing {len(batch.releases)} releases...")
```

### DataFrame Integration (Python)

Perfect for data analysis workflows:

```python
import pandas as pd
from ddex_parser import DDEXParser

parser = DDEXParser()
result = parser.parse_file("catalog.xml")

# Extract tracks as DataFrame
tracks_df = result.to_dataframe("tracks")
print(tracks_df.groupby('genre')['duration'].mean())

# Export for further analysis
tracks_df.to_csv("catalog_analysis.csv")
```

### Framework Integration

#### React Component

```tsx
import React, { useState } from 'react';
import { DDEXParser, type DDEXResult } from 'ddex-parser';

const DDEXUploader: React.FC = () => {
  const [result, setResult] = useState<DDEXResult | null>(null);
  
  const handleFileUpload = async (file: File) => {
    const parser = new DDEXParser();
    const text = await file.text();
    const parsed = await parser.parseString(text);
    setResult(parsed);
  };

  return (
    <div>
      <input 
        type="file" 
        accept=".xml" 
        onChange={(e) => e.target.files?.[0] && handleFileUpload(e.target.files[0])} 
      />
      {result && (
        <div>
          <h2>{result.flattened.releaseTitle}</h2>
          <p>Artist: {result.flattened.mainArtist}</p>
          <p>Tracks: {result.flattened.tracks.length}</p>
        </div>
      )}
    </div>
  );
};
```

## Round-Trip Compatibility

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
console.log(reparsed.tracks[0].title); // "New Title"
```

## Performance Benchmarks

Performance comparison across environments:

### Native Performance (Node.js/Python)
| File Size | ddex-parser | Traditional | Speedup | Memory |
|-----------|-------------|-------------|---------|---------|
| 10KB      | 0.8ms       | 12ms        | 15x     | -70%    |
| 100KB     | 3ms         | 45ms        | 15x     | -65%    |
| 1MB       | 28ms        | 420ms       | 15x     | -60%    |
| 10MB      | 180ms       | 2.8s        | 16x     | -55%    |

### Browser Performance (WASM)
| File Size | ddex-parser | DOMParser | xml2js | Bundle Size |
|-----------|-------------|-----------|---------|-------------|
| 10KB      | 2.1ms       | 12ms      | 25ms    | 489KB       |
| 100KB     | 8ms         | 85ms      | 180ms   | (gzipped)   |
| 1MB       | 65ms        | 750ms     | 1.8s    |             |

## Getting Started

### Installation Guides

- **[JavaScript/TypeScript →](./bindings/node/README.md)** - npm package with Node.js and browser support
- **[Python →](./bindings/python/README.md)** - PyPI package with pandas integration
- **[Rust →](./README-rust.md)** - Crates.io package documentation

### Example Projects

- [React DDEX Viewer](./examples/react-viewer) - Upload and visualize DDEX files
- [Python Analytics](./examples/python-analytics) - Catalog analysis with pandas
- [Node.js Processor](./examples/node-processor) - Batch processing pipeline
- [Vue.js Dashboard](./examples/vue-dashboard) - Interactive DDEX dashboard

## Migration Guides

### From v0.1.0 to v0.2.0

The v0.2.0 release introduced significant improvements with some breaking changes:

```javascript
// v0.1.0 (deprecated)
import ddexParser from 'ddex-parser';
const result = ddexParser.parse(xml);

// v0.2.0+ (current)
import { DDEXParser } from 'ddex-parser';
const parser = new DDEXParser();
const result = await parser.parseString(xml);
```

**New in v0.2.0:**
- Dual model architecture (graph + flattened)
- Async/await throughout
- Enhanced security features
- Streaming support
- Better error handling
- Full TypeScript support

### From Other Parsers

Migration helpers for common XML parsers:

```javascript
// From xml2js
const xml2js = require('xml2js');
xml2js.parseString(xml, callback); // Old way

import { DDEXParser } from 'ddex-parser';
const result = await new DDEXParser().parseString(xml); // New way

// From fast-xml-parser
const parser = new XMLParser();
const obj = parser.parse(xml); // Old way

const result = await new DDEXParser().parseString(xml); // New way
```

## Error Handling

Comprehensive error types with detailed information:

```typescript
import { DDEXParser, DDEXError, SecurityError, ValidationError } from 'ddex-parser';

try {
  const result = await parser.parseFile('release.xml');
} catch (error) {
  if (error instanceof SecurityError) {
    // Handle XXE or security violations
    console.error('Security issue:', error.message);
  } else if (error instanceof ValidationError) {
    // Handle DDEX structure violations  
    console.error('Invalid DDEX:', error.details);
  } else if (error instanceof DDEXError) {
    // Handle parsing errors
    console.error('Parse error:', error.message);
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
cargo test -p ddex-parser-core
```

## Contributing

We welcome contributions! Please read our [Contributing Guide](./CONTRIBUTING.md) before submitting PRs.

### Development Setup

```bash
git clone https://github.com/ddex-suite/ddex-suite.git
cd ddex-suite/packages/ddex-parser

# Install dependencies
npm install
pip install -r requirements-dev.txt
cargo build

# Run tests
npm run test:all
```

## Support

- 📖 [Full Documentation](https://ddex-suite.github.io/docs/)
- 🐛 [Report Issues](https://github.com/ddex-suite/ddex-suite/issues)
- 💬 [GitHub Discussions](https://github.com/ddex-suite/ddex-suite/discussions)
- 📧 Email: support@ddex-suite.com

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Related Projects

- **[ddex-builder](https://github.com/ddex-suite/ddex-suite/tree/main/packages/ddex-builder)** - Build deterministic DDEX XML files
- **[DDEX Suite](https://github.com/ddex-suite/ddex-suite)** - Complete DDEX processing toolkit
- **[DDEX Workbench](https://github.com/ddex/ddex-workbench)** - Official DDEX validation tools

---

Built with ❤️ for the music industry. Powered by Rust for maximum performance and safety.