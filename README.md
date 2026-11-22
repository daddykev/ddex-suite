# DDEX Suite

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)
![Node.js](https://img.shields.io/badge/Node.js-18%20|%2020%20|%2022-green?logo=node.js)
![Python](https://img.shields.io/badge/Python-3.9%20|%203.10%20|%203.11%20|%203.12%20|%203.13-blue?logo=python)
![TypeScript](https://img.shields.io/badge/TypeScript-5.0%2B-blue?logo=typescript)
![Platform](https://img.shields.io/badge/Platform-Linux%20|%20macOS%20|%20Windows-lightgrey)
[![npm ddex-builder](https://img.shields.io/npm/v/ddex-builder?label=npm%20ddex-builder)](https://www.npmjs.com/package/ddex-builder)
[![npm ddex-parser](https://img.shields.io/npm/v/ddex-parser?label=npm%20ddex-parser)](https://www.npmjs.com/package/ddex-parser)
[![PyPI ddex-builder](https://img.shields.io/pypi/v/ddex-builder?label=PyPI%20ddex-builder)](https://pypi.org/project/ddex-builder/)
[![PyPI ddex-parser](https://img.shields.io/pypi/v/ddex-parser?label=PyPI%20ddex-parser)](https://pypi.org/project/ddex-parser/)
[![crates.io ddex-core](https://img.shields.io/crates/v/ddex-core?label=crates.io%20ddex-core)](https://crates.io/crates/ddex-core)
[![crates.io ddex-parser](https://img.shields.io/crates/v/ddex-parser?label=crates.io%20ddex-parser)](https://crates.io/crates/ddex-parser)
[![crates.io ddex-builder](https://img.shields.io/crates/v/ddex-builder?label=crates.io%20ddex-builder)](https://crates.io/crates/ddex-builder)

> High-performance DDEX XML builder and parser with native bindings for TypeScript/JavaScript and Python. Built on a single Rust core for consistent behavior across all platforms.

DDEX Suite brings together powerful tools for music industry data exchange, combining the robust `ddex-parser` library for reading and transforming DDEX messages with the `ddex-builder` library for deterministic XML generation, creating a complete round-trip solution for DDEX processing.

## 🎯 Why DDEX Suite?

Working with DDEX XML shouldn't feel like archaeology. The suite transforms complex DDEX messages into clean, strongly-typed data structures that are as easy to work with as JSON.

### Core Value Proposition
- **Single Rust Core**: One implementation to rule them all - consistent behavior across JavaScript, Python, and Rust
- **Dual Model Architecture**: Choose between faithful graph representation or developer-friendly flattened view  
- **Production Ready**: Built-in XXE protection, memory-bounded streaming, and comprehensive security hardening
- **Deterministic Output**: Consistent, reproducible XML generation with smart normalization

## 👨🏻‍💻 Developer Statement

I'm building **DDEX Suite** as a rigorous, end-to-end learning project to deepen my Rust skills while unifying my JavaScript and Python experience into a production-grade toolkit for music metadata. The intent is to ship a **single Rust core** that serves both a high-performance, security-hardened DDEX XML parser library (`ddex-parser`) and a consistent, deterministic builder library (`ddex-builder`). This core is exposed through **napi-rs** for Node/TypeScript and **PyO3** for Python, showcasing not just cross-language API design but also deep ecosystem integration, including a **declarative DataFrame mapping DSL** for Python users. The project is deliberately "industry-shaped," tackling the complementary challenges of transforming complex DDEX XML into clean models (parsing) and generating canonical, reproducible XML from those models. This is achieved through a dual **graph+flattened** data model for developer UX and an uncompromising approach to determinism, centered on a custom canonicalization specification, **DB-C14N/1.0**, and a **stable, content-addressable ID generation** engine.

Beyond the core implementation, this is a showcase of **software craftsmanship and platform thinking**. The suite provides consistent APIs, painless installation via prebuilt binaries, a hardened CI/CD pipeline, and robust supply-chain safety (SBOM, `cargo-deny`, and **Sigstore artifact signing**). Every feature reflects production wisdom—from the parser's XXE protection to the builder's versioned **partner presets system** with safety locks. Paired with my validator work (DDEX Workbench), DDEX Suite delivers a credible, end-to-end **Parse → Modify → Build** processing pipeline, complete with enterprise-grade features like **preflight validation**, a **semantic diff engine**, and a comprehensive CLI. It illustrates how to design interoperable components that are fast, safe, and easy to adopt in real-world systems.

## 🚧 Development Status

**Latest Release**: Suite v0.5.0  
**Current Development Phase**: 5.1 - Performance & Scale  
**Target Release**: Suite v1.0.0 in Q1 2026

### 📦 Available Packages

All packages published across npm, PyPI, and **crates.io**! ✅

| Package | npm | PyPI | crates.io | Version |
|---------|-----|------|-----------|---------|
| **ddex-core** | - | - | ✅ [Published](https://crates.io/crates/ddex-core) | v0.5.0 |
| **ddex-parser** | ✅ [Published](https://www.npmjs.com/package/ddex-parser) | ✅ [Published](https://pypi.org/project/ddex-parser/) | ✅ [Published](https://crates.io/crates/ddex-parser) | v0.5.0 |
| **ddex-builder** | ✅ [Published](https://www.npmjs.com/package/ddex-builder) | ✅ [Published](https://pypi.org/project/ddex-builder/) | ✅ [Published](https://crates.io/crates/ddex-builder) | v0.5.0 |

### Progress Overview

✅ **Phase 1-3: Complete** - Core foundation, parser, and builder are fully implemented  
✅ **Phase 4.1: Integration Testing** - Round-trip functionality validated with 94 tests passing  
✅ **crates.io Publishing** - **NEW!** All Rust crates published to the official registry  
✅ **Phase 4.2: Documentation** - [Docusaurus](https://ddex-suite.org) site in React  
✅ **Phase 4.3: Smart Normalization Engine** - Round-trip, deterministic output  
✅ **Phase 4.3.5: Core Stabilization** - Stability and performance upgrades  
✅ **Phase 4.4: Streaming Parser** - High-performance XML parser

For detailed development progress and technical implementation details, see [blueprint.md](./blueprint.md).

## 🎭 Dual Model Architecture

The suite provides two complementary views of the same data with full round-trip data integrity:

### Graph Model (Faithful)
Preserves the exact DDEX structure with references and extensions - perfect for compliance and round-trip operations:
```typescript
interface ERNMessage {
  messageHeader: MessageHeader;
  parties: Party[];               // All parties with IDs
  resources: Resource[];          // Audio, video, image resources
  releases: Release[];            // Release metadata with references
  deals: Deal[];                  // Commercial terms
  extensions?: Map<string, XmlFragment>;  // Preserved for round-trip
  toBuildRequest(): BuildRequest; // Convert to builder input
}
```

### Flattened Model (Developer-Friendly)
Denormalized and resolved for easy consumption - ideal for applications while maintaining round-trip capability:
```typescript
interface ParsedRelease {
  releaseId: string;
  title: string;
  displayArtist: string;
  tracks: ParsedTrack[];          // Fully resolved with resources merged
  coverArt?: ParsedImage;
  _graph?: Release;               // Reference to original for full data integrity
  extensions?: Map<string, XmlFragment>; // Extensions preserved
}
```

## 🧹 Smart Normalization & Clean Output

The DDEX Suite provides powerful normalization capabilities that transform inconsistent, messy DDEX files into clean, compliant output.

### Why Normalization Matters

Real-world DDEX files come from many sources with varying quality:
- Different namespace conventions (ern:Title vs Title vs ns2:Title)
- Inconsistent element ordering
- Mixed DDEX versions and dialects
- Redundant whitespace and formatting issues
- Non-standard extensions and attributes

DDEX Builder solves this by:
- **Normalizing** all input to clean DDEX 4.3 structure
- **Standardizing** element and attribute ordering
- **Optimizing** output for compliance and size
- **Preserving** all semantic data and business information

### Build & Normalization Features

```typescript
// Build DDEX
const { DdexBuilder } = require('ddex-builder');
const builder = new DdexBuilder();
builder.applyPreset('audio_album'); // Apply baseline preset

// Configuration options
builder.setConfig({
  canonical: true,           // Consistent, deterministic output
  validate: true,            // Ensure DDEX compliance
  version: '4.3',           // Target DDEX version
  optimize_size: true       // Remove redundant whitespace
});

// Parse messy vendor DDEX → Output clean DDEX
const { DdexParser } = require('ddex-parser');
const parser = new DdexParser();
const parsed = await parser.parse(messyVendorFile);
const cleanDdex = await builder.build(parsed);
// Result: Beautiful, compliant DDEX 4.3
```

### What Gets Normalized

| Input Chaos | Output Order |
|------------|--------------|
| Mixed namespace prefixes | Consistent DDEX namespaces |
| Random element ordering | Specification-compliant order |
| Whitespace soup | Clean, minimal formatting |
| Legacy DDEX versions | Modern DDEX 4.3 |
| Vendor-specific quirks | Standard-compliant structure |

### 🔒 Data Integrity Guarantees

The DDEX Suite ensures your business data is always preserved:

#### **Guarantee 1: Semantic Preservation**
All business-critical data (ISRCs, titles, artists, deals) is preserved with 100% accuracy.

#### **Guarantee 2: Deterministic Output**
Building the same data always produces identical output - perfect for testing and validation.

#### **Guarantee 3: Extension Support**
Partner extensions (YouTube, generic) are preserved and properly namespaced.

#### **Guarantee 4: Round-Trip Data Integrity**
Parse → Modify → Build workflows maintain all your data, with beneficial normalization applied.

## 🚀 Features

### ✅ Parser completeness and honest bindings (v0.5.0)
- Parser reads release dates, P/C lines, genres, parental warnings, labels,
  `MessageControlType` and deal end dates in ERN 3.8.2, 4.2 and 4.3; text is kept
  exactly as written (XSD whitespace rules)
- `parse()` always uses the complete parser (no automatic switch to streaming above 10 MB)
- Builder writes schema-valid ERN 3.8.2; the WASM builder runs the core builder
- Python packages raise `ImportError` with the cause when the native extension cannot load
  (no mock fallback); Node `ParseOptions` that had no effect now throw
- Performance numbers are measured with criterion benches ([dev-docs/PERFORMANCE.md](./dev-docs/PERFORMANCE.md))
- See [CHANGELOG.md](./CHANGELOG.md) for the breaking changes

### ✅ Error Refactoring (v0.4.5)
- Enhanced error handling across all bindings
- Critical bug fixes for production stability
- Memory leak fixes in streaming parser
- Improved cross-platform binary compatibility

### ✅ Stability Improvements (v0.4.4)
- Enhanced error handling for missing required fields
- Improved validation messages with specific field paths
- Removed placeholder data generation
- Fixed validation to reject incomplete DDEX documents

### ✅ Enhanced Parser Performance (v0.4.3)
- **ENHANCED**: Improved parser graph structure optimization
- **IMPROVED**: Enhanced memory management for large DDEX files
- **OPTIMIZED**: Better resource allocation patterns for streaming
- **UPDATED**: Refined error handling and validation routines

### ✅ Linux x64 Node.js Binaries Added (v0.4.2)
- **Native**: Linux x64 GNU binaries for Node.js (Node 18+ compatible)
- Cloud deployment support for Google Cloud, AWS, Azure
- Server-side rendering and cloud function compatibility
- Added `ddex-parser-node.linux-x64-gnu.node` binary
- Added `ddex-builder-node.linux-x64-gnu.node` binary

### ✅ Streaming Parser (v0.4.0)
- Low-level streaming iterators and a memchr-based `FastStreamingParser`
- The throughput, memory and parallel-scaling figures published with v0.4.0 were not
  reproduced by the benches; see [Performance](#-performance) for measured numbers

### ✅ Smart Normalization (v0.4.0)
- **🧹 Clean Output**: Transform messy vendor DDEX into compliant DDEX 4.3
- **📐 Consistent Structure**: Standardized element ordering and namespaces
- **✨ Optimized Size**: Remove redundant whitespace and formatting
- **🔄 Data Preservation**: 100% semantic accuracy maintained
- **🎯 Deterministic**: Same input always produces same output

### ✅ Native Python Bindings (v0.3.0)
- **🐍 Production-Ready Python**: Native PyO3 bindings with full DataFrame integration
- **📊 DataFrame Support**: Three schema options (flat, releases, tracks) for pandas integration
- **🔄 Round-Trip Python**: Complete Parse → DataFrame → Build workflow support
- **🔗 PyPI Available**: Install with `pip install ddex-parser ddex-builder`

### ✅ Core Features
- **🔄 Round-Trip Workflow**: Parse → Modify → Build with 100% data preservation
- **🎭 Dual Model Architecture**: Graph (faithful) and flattened (developer-friendly) views
- **🛡️ Enterprise Security**: XXE protection, entity expansion limits, memory bounds
- **⚡ Fast**: a typical ERN 4.3 album message parses in about 3 ms
- **🌐 Multi-Platform**: Native bindings for Node.js, Python, WASM, and Rust
- **🔗 Reference Linking**: Automatic relationship resolution between entities
- **🆔 Stable Hash IDs**: Content-based deterministic ID generation
- **✨ Multi-Version Support**: ERN 3.8.2, 4.2, and 4.3 with automatic detection

### 🔄 In Development
- **Streaming**: Handle massive catalogs with backpressure and progress callbacks
- **Semantic Diff**: Track changes between DDEX message versions
- **Additional Bindings**: C#/.NET and Go language bindings

## 📦 Installation

```bash
# JavaScript/TypeScript
npm install ddex-parser  # ✅ Latest: v0.5.0
npm install ddex-builder # ✅ Latest: v0.5.0

# Python
pip install ddex-parser  # ✅ Latest: v0.5.0
pip install ddex-builder # ✅ Latest: v0.5.0

# Rust
cargo add ddex-core      # ✅ Latest: v0.5.0
cargo add ddex-parser    # ✅ Latest: v0.5.0
cargo add ddex-builder   # ✅ Latest: v0.5.0
```

### Browser/WASM
```html
<script type="module">
import init, { DdexParser, DdexBuilder } from '@ddex/wasm';
await init();
const parser = new DdexParser();
const builder = new DdexBuilder();
</script>
```

Bundle sizes (v0.4.0):
- Parser: 37KB (gzipped: ~12KB)
- Builder: 420KB (gzipped: ~140KB)

## 💻 Usage Examples

### JavaScript/TypeScript
```typescript
// Parse DDEX
const { DdexParser } = require('ddex-parser');
const parser = new DdexParser();
const result = await parser.parse(xmlContent);

// Modify the parsed data
result.flat.releases[0].title = "Updated Title";

// Build DDEX
const { DdexBuilder } = require('ddex-builder');
const builder = new DdexBuilder();
builder.applyPreset('audio_album'); // optional
const xml = await builder.build(result.toBuildRequest());

// Round-trip with beneficial normalization
const reparsed = await parser.parse(xml);
assert.deepEqual(reparsed.graph, result.graph); // ✅ Identical
```

### Python (v0.4.0 - Native Implementation)
```python
from ddex_parser import DdexParser
from ddex_builder import DdexBuilder
import pandas as pd

# Parse DDEX message with native performance
parser = DdexParser()
message = parser.parse(xml_content)

# Export to DataFrame for analysis (NEW!)
df = message.to_dataframe(schema='releases')  # 'flat', 'releases', or 'tracks'
print(f"Found {len(df)} releases")

# Modify DataFrame data
df.loc[0, 'title'] = 'Updated Album Title'

# Build from DataFrame (Round-trip support)
builder = DdexBuilder()
xml = builder.from_dataframe(df, version='4.3')

# Traditional object-based building also supported
xml = builder.build({
    'header': {
        'message_sender': {'party_name': [{'text': 'My Label'}]},
        'message_recipient': {'party_name': [{'text': 'YouTube'}]}
    },
    'version': '4.3',
    'releases': [{
        'release_id': '1234567890123',
        'title': [{'text': 'Amazing Album'}],
        'display_artist': 'Great Artist',
        'tracks': [
            {'position': 1, 'isrc': 'USXYZ2600001', 'title': 'Track 1', 'duration': 180}
        ]
    }]
})
```

### Rust
```rust
use ddex_parser::DdexParser;
use ddex_builder::DdexBuilder;

// Parse DDEX message
let parser = DdexParser::new();
let result = parser.parse(&xml_content)?;

// Modify the parsed data
let mut build_request = result.to_build_request();
build_request.releases[0].title = "Updated Title".to_string();

// Build deterministic XML
let builder = DdexBuilder::new();
let xml = builder.build(&build_request)?;

// Round-trip with beneficial normalization and type safety
let reparsed = parser.parse(&xml)?;
assert_eq!(reparsed.graph, result.graph); // ✅ Identical
```

## 🏗️ Architecture

Built as a monorepo with shared core components:

```
┌───────────────────────────────────────┐
│            DDEX Suite                 │
├─────────────────┬─────────────────────┤
│   DDEX Parser   │   DDEX Builder      │
│  Read & Parse   │  Generate & Build   │
├─────────────────┴─────────────────────┤
│           Shared Core                 │
│    Models │ Errors │ FFI │ Utils      │
├───────────────────────────────────────┤
│         Language Bindings             │
│   napi-rs │ PyO3 │ WASM │ CLI         │
└───────────────────────────────────────┘
```

## 🔒 Security

- XXE (XML External Entity) protection
- Entity expansion limits (billion laughs protection)
- Deep nesting protection
- Size and timeout limits
- Memory-bounded streaming
- Supply chain security with cargo-deny and SBOM

## 📊 Performance

Measured with the criterion benches on an Apple M1 Pro, release profile
(`cargo bench`; details, commands and caveats in
[dev-docs/PERFORMANCE.md](./dev-docs/PERFORMANCE.md)). Your numbers will differ by machine.

| Operation | Measured |
|---|---:|
| Parse an official ERN 4.3 sample (10–330 KB) | 0.5–14 ms (about 20–24 MiB/s) |
| Parse `1 Audio.xml` (75 KB, typical album) | 3.4 ms |
| Parse a ~10 MB ERN 4.3 catalog | 441 ms (22.7 MiB/s) |
| Build an ERN 4.3 message, 1 track | 2.7 ms |
| Build an ERN 4.3 message, 12 tracks | 11.4 ms |
| Build an ERN 4.3 message, 100 tracks | 80 ms |

- The complete parser runs at about 20–24 MiB/s on real ERN 4.3 documents, whatever
  their size. The builder costs roughly 0.8 ms per track plus about 2 ms per message.
- The low-level streaming iterators (which do not build the release/track model) scan a
  10 MB catalog at 167–860 MiB/s; those numbers are not comparable with `parse`.
- **The parallel streaming iterator does not currently scale with the thread count**
  (1 to 8 threads all take 11.6 ms on the 10 MB catalog). Making it scale is planned for 0.6.0.
- Earlier figures (sub-3 ms parsing, 25–30 / 500–700 / 1,265 MB/s, "6.25× on 8 cores")
  were not reproduced by these benches and have been removed.

### Package Sizes

| Component | Size | Target | Status |
|-----------|------|--------|--------|
| Rust Core | 9.4MB | - | ✅ Development artifact |
| Node.js (npm) | 347KB | <1MB | ✅ Excellent |
| Python wheel | 235KB | <1MB | ✅ Compact |
| WASM bundle | 114KB | <500KB | ✅ 77% under target! |
| **crates.io** ✅ **NEW!** | | | |
| ddex-core | 57.2KiB (34 files) | <10MB | ✅ Compact |
| ddex-parser | 197.9KiB (43 files) | <10MB | ✅ Efficient |
| ddex-builder | 1.1MiB (81 files) | <10MB | ✅ Under limit |

## 📚 Documentation


### 📖 Core Documentation
- [Blueprint](./blueprint.md) - Detailed architecture, roadmap, and technical implementation
- [Parser API](./packages/ddex-parser/docs/API.md) - Parser documentation
- [Builder API](./packages/ddex-builder/docs/API.md) - Builder documentation
- [Round-Trip Guide](./docs/ROUND_TRIP.md) - Parse → Modify → Build guide
- [Error Handbook](./docs/ERROR_HANDBOOK.md) - Understanding and handling errors

### 🦀 Rust Documentation ✅ **NEW!**
- [ddex-core](https://docs.rs/ddex-core) - Core data models and utilities
- [ddex-parser](https://docs.rs/ddex-parser) - Parser API reference
- [ddex-builder](https://docs.rs/ddex-builder) - Builder API reference

### 🔧 Developer Resources
- [Contributing Guide](./CONTRIBUTING.md) - Development setup and guidelines
- [API Changelog](./CHANGELOG.md) - Version history and breaking changes

## 🤝 Contributing

This project is in active development. While external contributions aren't yet accepted, we welcome feedback and issue reports. Follow the project for updates!

## 📜 License

MIT License - see [LICENSE](./LICENSE) file for details.

## 🙏 Acknowledgments

DDEX Suite is designed to complement [DDEX Workbench](https://ddex-workbench.org) by providing structural parsing and deterministic generation while Workbench handles XSD validation and business rules.

---

**Repository**: https://github.com/daddykev/ddex-suite  
**Parser**: v0.5.0 on [npm](https://www.npmjs.com/package/ddex-parser) and [PyPI](https://pypi.org/project/ddex-parser/)
**Builder**: v0.5.0 on [npm](https://www.npmjs.com/package/ddex-builder) and [PyPI](https://pypi.org/project/ddex-builder/)  
**Suite Target**: v1.0.0 in Q1 2026  
**Last Updated**: November 22, 2026