# DDEX Parser

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-ddex--suite-blue)](https://github.com/daddykev/ddex-suite)

High-performance DDEX XML parser built in Rust with comprehensive security protections and command-line interface. Parse ERN 3.8.2, 4.2, and 4.3 files with built-in validation, security hardening against XML attacks, and deterministic JSON output.

Part of the [DDEX Suite](https://github.com/daddykev/ddex-suite) - a comprehensive toolkit for working with DDEX metadata in the music industry.

> **v0.5.0** - Complete parsing of dates, P/C lines, genres, labels and deal end dates in ERN 3.8.2 / 4.2 / 4.3; text kept exactly as written; `parse()` no longer switches to streaming above 10 MB. See the CHANGELOG for breaking changes.
>
> **v0.4.4 Released** - Enhanced validation with strict error handling! Parser now properly fails on missing required fields instead of using placeholders.
>
> **v0.4.3 Released** - Performance optimizations and enhanced stability.
>
> **v0.4.2 Released** - Linux x64 Node.js binaries added for cloud deployment compatibility!

## 🛡️ Security-First Design

**Fixed Critical Vulnerabilities:**
- ✅ **XML Bomb Protection** - Guards against billion laughs and entity expansion attacks
- ✅ **Deep Nesting Protection** - Prevents stack overflow from malicious XML
- ✅ **Input Validation** - Rejects malformed XML with clear error messages
- ✅ **Memory Bounds** - Configurable limits for large file processing

## 🚀 Current Implementation Status

### ✅ **Fully Working**
- **Command Line Interface** - Complete CLI with parse, validate, batch operations
- **Rust API** - Full programmatic access via `DDEXParser` struct
- **ERN Support** - ERN 3.8.2, 4.2, and 4.3 parsing and validation
- **Security Hardened** - Protection against XML bombs, deep nesting, malformed input
- **JSON Output** - Clean, deterministic JSON serialization

### ✅ **Language Bindings (Production Ready)**
- **Node.js/TypeScript Bindings** - Complete DDEX data structure access
- **Python Bindings** - PyO3-based Python integration with pandas support
- **WebAssembly** - Browser-compatible WASM module
- **Full Node.js bindings with TypeScript support**

## Quick Start

### Command Line Interface (Ready Now)

```bash
# Install from source
git clone https://github.com/daddykev/ddex-suite
cd ddex-suite/packages/ddex-parser
cargo build --release

# Parse DDEX file to JSON
./target/release/ddex-parser parse release.xml --output release.json

# Validate DDEX file
./target/release/ddex-parser validate release.xml

# Batch process multiple files
./target/release/ddex-parser batch "*.xml" --output-dir results/
```

### Rust Library (Ready Now)

```rust
use ddex_parser::DDEXParser;
use std::fs::File;
use std::io::BufReader;

// Create parser with secure defaults
let parser = DDEXParser::new();

// Parse DDEX file
let file = File::open("release.xml")?;
let reader = BufReader::new(file);
let parsed = parser.parse(reader)?;

// Access flattened data
println!("Release: {}", parsed.releases[0].release_title[0].text);
println!("Tracks: {}", parsed.releases[0].track_count);
```

## Core Features

### 🚀 Performance (measured)

Criterion benches on an Apple M1 Pro, release profile (`cargo bench -p ddex-parser`;
source: `dev-docs/PERFORMANCE.md` in the repository):

| Input | `DDEXParser::parse` |
|---|---:|
| Official ERN 4.3 samples, 10–330 KB | 0.5–14 ms (about 20–24 MiB/s) |
| `1 Audio.xml`, 75 KB (typical album) | 3.4 ms |
| ~10 MB ERN 4.3 catalog | 441 ms (22.7 MiB/s) |

- The low-level streaming iterators (no release/track model) scan the 10 MB catalog at
  167–860 MiB/s. The parallel streaming iterator does not currently scale with the
  thread count (planned for 0.6.0).
- `FastStreamingParser` (the `SecurityConfig::relaxed()` fast path) scans at about
  3 GiB/s but returns no releases or resources (see KNOWN_ISSUES.md).
- Debug builds are 50–100× slower: measure with `cargo bench` or `--release`.
- The v0.4.0 throughput figures (25–30 / 500–700 / 1,265 MB/s) were not reproduced and
  have been removed.

### 🔒 Security First
- Built-in XXE (XML External Entity) protection
- Entity expansion limits (billion laughs protection)
- Deep nesting protection
- Memory-bounded parsing with timeout controls

### 🎭 Dual Model Architecture
- **Graph Model**: Faithful DDEX structure with references (perfect for compliance)
- **Flattened Model**: Developer-friendly denormalized data (easy to consume)
- Full round-trip data integrity between both representations

### 🧹 Parser + Builder Workflow
DDEX Parser extracts data faithfully, while **ddex-builder** provides smart normalization:
- **Parser role**: Preserves exact input structure and semantics
- **Builder role**: Transforms data into clean, compliant DDEX 4.3
- **Combined workflow**: Parse messy vendor DDEX → Modify data → Generate clean output
- **Data integrity**: All business data (ISRCs, titles, deals) preserved through round-trip

```typescript
// Parser preserves input exactly as received
const messyVendorDdex = await parser.parse(vendorFile);
// Builder normalizes output to clean DDEX 4.3
const cleanDdex = await builder.build(messyVendorDdex, { normalize: true });
```

### 🌐 Cross-Platform Compatibility
- **Node.js 16+** with native addon performance and complete data access
- **Browser support** via optimized WASM (<500KB)
- **Python 3.9+** with comprehensive type hints
- **TypeScript-first** with complete type definitions
- **Complete DDEX data structure access** across all language bindings

### 🎵 Music Industry Ready
- Support for all DDEX ERN versions (3.8.2, 4.2, 4.3+)
- Complete metadata extraction (releases, tracks, artists, rights)
- Territory and deal information parsing
- Image and audio resource handling
- Genre, mood, and classification support

## Security

v0.4.0 includes comprehensive security enhancements:
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

### Node.js/JavaScript Example (v0.4.2+)

```javascript
const { DdexParser } = require('ddex-parser');
const parser = new DdexParser();

const result = parser.parseSync(xmlContent);

// Full access to parsed data
console.log('Message ID:', result.messageId);
console.log('Releases:', result.releases);
console.log('Resources:', result.resources);
console.log('Deals:', result.deals);

// Access individual release data
result.releases.forEach(release => {
  console.log('Release:', release.title);
  console.log('Artist:', release.displayArtist);
  console.log('Tracks:', release.tracks.length);
});
```

### Round-Trip Compatibility

Seamless integration with ddex-builder for complete workflows with smart normalization:

```typescript
import { DDEXParser } from 'ddex-parser';
import { DDEXBuilder } from 'ddex-builder';

// Parse existing DDEX file
const parser = new DDEXParser();
const original = await parser.parseFile('input.xml');

// Modify data
const modified = { ...original.flattened };
modified.tracks[0].title = "New Title";

// Build new DDEX file with smart normalization
const builder = new DDEXBuilder();
const newXML = await builder.buildFromFlattened(modified);

// Verify round-trip integrity (with beneficial normalization)
const reparsed = await parser.parseString(newXML);
assert.deepEqual(reparsed.tracks[0].title, "New Title"); // ✅ Data integrity preserved
```

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE) file for details.

## Related Projects

- **[ddex-builder](https://crates.io/crates/ddex-builder)** - Build deterministic DDEX XML files
- **[DDEX Suite](https://ddex-suite.org)** - Complete DDEX processing toolkit
- **[DDEX Workbench](https://ddex-workbench.org)** - Official DDEX validation tools

---

Built with ❤️ for the music industry. Powered by Rust for maximum performance and safety.