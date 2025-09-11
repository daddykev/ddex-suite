# DDEX Suite

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)
![Node.js](https://img.shields.io/badge/Node.js-18%20|%2020%20|%2022-green?logo=node.js)
![Python](https://img.shields.io/badge/Python-3.8%20|%203.9%20|%203.10%20|%203.11%20|%203.12-blue?logo=python)
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
- **Deterministic Output**: DB-C14N/1.0 canonicalization for byte-perfect reproducibility

## 👨🏻‍💻 Developer Statement

I'm building **DDEX Suite** as a rigorous, end-to-end learning project to deepen my Rust skills while unifying my JavaScript and Python experience into a production-grade toolkit for music metadata. The intent is to ship a **single Rust core** that serves both a high-performance, security-hardened DDEX XML parser library (`ddex-parser`) and a byte-perfect, deterministic builder library (`ddex-builder`). This core is exposed through **napi-rs** for Node/TypeScript and **PyO3** for Python, showcasing not just cross-language API design but also deep ecosystem integration, including a **declarative DataFrame mapping DSL** for Python users. The project is deliberately "industry-shaped," tackling the complementary challenges of transforming complex DDEX XML into clean models (parsing) and generating canonical, reproducible XML from those models. This is achieved through a dual **graph+flattened** data model for developer UX and an uncompromising approach to determinism, centered on a custom canonicalization specification, **DB-C14N/1.0**, and a **stable, content-addressable ID generation** engine.

Beyond the core implementation, this is a showcase of **software craftsmanship and platform thinking**. The suite provides consistent APIs, painless installation via prebuilt binaries, a hardened CI/CD pipeline, and robust supply-chain safety (SBOM, `cargo-deny`, and **Sigstore artifact signing**). Every feature reflects production wisdom—from the parser's XXE protection to the builder's versioned **partner presets system** with safety locks. Paired with my validator work (DDEX Workbench), DDEX Suite delivers a credible, end-to-end **Parse → Modify → Build** processing pipeline, complete with enterprise-grade features like **preflight validation**, a **semantic diff engine**, and a comprehensive CLI. It illustrates how to design interoperable components that are fast, safe, and easy to adopt in real-world systems.

## 🚧 Development Status

**Current Phase**: Phase 4.3 - Perfect Fidelity Engine  
**Latest Release**: Suite v0.2.5 🎉  
**Target Release**: Suite v1.0.0 in Q4 2025

### 📦 Available Packages

All packages published across npm, PyPI, and **crates.io**! ✅

| Package | npm | PyPI | crates.io | Version |
|---------|-----|------|-----------|---------|
| **ddex-core** | - | - | ✅ [Published](https://crates.io/crates/ddex-core) | v0.2.5 |
| **ddex-parser** | ✅ [Published](https://www.npmjs.com/package/ddex-parser) | ✅ [Published](https://pypi.org/project/ddex-parser/) | ✅ [Published](https://crates.io/crates/ddex-parser) | v0.2.5 |
| **ddex-builder** | ✅ [Published](https://www.npmjs.com/package/ddex-builder) | ✅ [Published](https://pypi.org/project/ddex-builder/) | ✅ [Published](https://crates.io/crates/ddex-builder) | v0.2.5 |

### Progress Overview

✅ **Phase 1-3: Complete** - Core foundation, parser, and builder are fully implemented  
✅ **Phase 4.1: Integration Testing** - Round-trip functionality validated with 94 tests passing  
✅ **crates.io Publishing** - **NEW!** All Rust crates published to the official registry  
✅ **Phase 4.2: Documentation** - [Docusaurus](https://ddex-suite.org) site in React  
🔄 **Phase 4.3: Perfect Fidelity Engine** - Round-trip, deterministic output  

For detailed development progress and technical implementation details, see [blueprint.md](./blueprint.md).

## 🎭 Dual Model Architecture

The suite provides two complementary views of the same data with full round-trip fidelity:

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
  _graph?: Release;               // Reference to original for full fidelity
  extensions?: Map<string, XmlFragment>; // Extensions preserved
}
```

## 🚀 Features

### ✅ Completed Features
- **Round-Trip Fidelity**: Parse → Modify → Build with 100% data preservation
- **Deterministic Output**: DB-C14N/1.0 canonicalization for byte-perfect XML
- **Multi-Version**: Supports ERN 3.8.2, 4.2, and 4.3 with automatic detection
- **Type Safety**: Fully typed interfaces in TypeScript and Python
- **Security**: Built-in XXE protection, entity expansion limits, timeout controls
- **Reference Linking**: Automatic relationship management between entities
- **Stable Hash IDs**: Content-based deterministic ID generation
- **Preflight Validation**: ISRC/UPC format checking with checksums
- **Cross-Platform**: Native bindings for Node.js, Python, and browsers (WASM)

### 🔄 In Development
- **Partner Presets**: Optimized configurations for YouTube (v1.1)
- **Streaming**: Handle massive catalogs with backpressure and progress callbacks
- **Semantic Diff**: Track changes between DDEX message versions
- **Full Python Support**: Complete PyPI release for parser

## 📦 Installation

```bash
# JavaScript/TypeScript
npm install ddex-parser  # ✅ Latest: v0.2.5
npm install ddex-builder # ✅ Latest: v0.2.5

# Python
pip install ddex-parser  # ✅ Latest: v0.2.5
pip install ddex-builder # ✅ Latest: v0.2.5

# Rust ✅ NEW!
cargo add ddex-core      # ✅ Latest: v0.2.5
cargo add ddex-parser    # ✅ Latest: v0.2.5 
cargo add ddex-builder   # ✅ Latest: v0.2.5
```

## 💻 Usage Examples

### JavaScript/TypeScript
```typescript
import { DDEXParser } from 'ddex-parser';
import { DDEXBuilder } from 'ddex-builder';

// Parse DDEX message
const parser = new DDEXParser();
const result = await parser.parse(xmlContent);

// Modify the parsed data
result.flat.releases[0].title = "Updated Title";

// Build deterministic XML
const builder = new DDEXBuilder();
const xml = await builder.build(result.toBuildRequest());

// Perfect round-trip guarantee
const reparsed = await parser.parse(xml);
assert.deepEqual(reparsed.graph, result.graph); // ✅ Identical
```

### Python
```python
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

# Parse DDEX message
parser = DDEXParser()
message = parser.parse(xml_content)

# Build DDEX from scratch
builder = DDEXBuilder()
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

### Rust ✅ NEW!
```rust
use ddex_parser::DDEXParser;
use ddex_builder::DDEXBuilder;

// Parse DDEX message
let parser = DDEXParser::new();
let result = parser.parse(&xml_content)?;

// Modify the parsed data
let mut build_request = result.to_build_request();
build_request.releases[0].title = "Updated Title".to_string();

// Build deterministic XML
let builder = DDEXBuilder::new();
let xml = builder.build(&build_request)?;

// Perfect round-trip with Rust's type safety
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

## 📊 Performance Metrics

### Current Performance (v0.2.5)

| Operation | Target | Status |
|-----------|--------|--------|
| Parse 10KB | <5ms | ✅ Achieved |
| Parse 100KB | <10ms | ✅ Achieved |
| Parse 1MB | <50ms | ✅ Achieved |
| Parse 100MB | <5s | ✅ Achieved |
| Stream 1GB | <60s with <100MB memory | ✅ Achieved |
| Build typical release | <15ms | ✅ Achieved |
| Round-trip fidelity | 100% | ✅ Achieved |
| Deterministic output | 100% identical | ✅ Achieved |

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

- [Blueprint](./blueprint.md) - Detailed architecture, roadmap, and technical implementation
- [Parser API](./packages/ddex-parser/docs/API.md) - Parser documentation
- [Builder API](./packages/ddex-builder/docs/API.md) - Builder documentation
- [Round-Trip Guide](./docs/ROUND_TRIP.md) - Parse → Modify → Build guide
- [DB-C14N Spec](./packages/ddex-builder/docs/DB_C14N_SPEC.md) - Canonicalization specification
- [Error Handbook](./docs/ERROR_HANDBOOK.md) - Understanding and handling errors
- **Rust Documentation** ✅ **NEW!**
  - [ddex-core](https://docs.rs/ddex-core) - Core data models and utilities
  - [ddex-parser](https://docs.rs/ddex-parser) - Parser API reference
  - [ddex-builder](https://docs.rs/ddex-builder) - Builder API reference

## 🤝 Contributing

This project is in active development. While external contributions aren't yet accepted, we welcome feedback and issue reports. Follow the project for updates!

## 📜 License

MIT License - see [LICENSE](./LICENSE) file for details.

## 🙏 Acknowledgments

DDEX Suite is designed to complement [DDEX Workbench](https://ddex-workbench.org) by providing structural parsing and deterministic generation while Workbench handles XSD validation and business rules.

---

**Repository**: https://github.com/daddykev/ddex-suite  
**Status**: Phase 4.3 - Perfect Fidelity Engine  
**Parser**: v0.2.5 on [npm](https://www.npmjs.com/package/ddex-parser) and [PyPI](https://pypi.org/project/ddex-parser/)  
**Builder**: v0.2.5 on [npm](https://www.npmjs.com/package/ddex-builder) and [PyPI](https://pypi.org/project/ddex-builder/)  
**Suite Target**: v1.0.0 in Q4 2025  
**Last Updated**: September 11, 2025