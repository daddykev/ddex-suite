# DDEX Suite

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)
![Node.js](https://img.shields.io/badge/Node.js-18%20|%2020%20|%2022-green?logo=node.js)
![Python](https://img.shields.io/badge/Python-3.8%20|%203.9%20|%203.10%20|%203.11%20|%203.12-blue?logo=python)
![TypeScript](https://img.shields.io/badge/TypeScript-5.0%2B-blue?logo=typescript)
![Platform](https://img.shields.io/badge/Platform-Linux%20|%20macOS%20|%20Windows-lightgrey)

> High-performance DDEX XML builder and parser with native bindings for TypeScript/JavaScript and Python. Built on a single Rust core for consistent behavior across all platforms.

DDEX Suite brings together powerful tools for music industry data exchange, combining the robust `ddex-parser` library for reading and transforming DDEX messages with the `ddex-builder` library for deterministic XML generation, creating a complete round-trip solution for DDEX processing.

## 🎯 Why DDEX Suite?

Working with DDEX XML shouldn't feel like archaeology. The suite transforms complex DDEX messages into clean, strongly-typed data structures that are as easy to work with as JSON.

### Core Value Proposition
- **Single Rust Core**: One implementation to rule them all - consistent behavior across JavaScript, Python, and Rust
- **Dual Model Architecture**: Choose between faithful graph representation or developer-friendly flattened view
- **Production Ready**: Built-in XXE protection, memory-bounded streaming, and comprehensive security hardening

## 👨🏻‍💻 Developer Statement

I'm building **DDEX Suite** as a rigorous, end-to-end learning project to deepen my Rust skills while unifying my JavaScript and Python experience into a production-grade toolkit for music metadata. The intent is to ship a **single Rust core** that serves both a high-performance, security-hardened DDEX XML parser library (`ddex-parser`) and a byte-perfect, deterministic builder library (`ddex-builder`). This core is exposed through **napi-rs** for Node/TypeScript and **PyO3** for Python, showcasing not just cross-language API design but also deep ecosystem integration, including a **declarative DataFrame mapping DSL** for Python users. The project is deliberately "industry-shaped," tackling the complementary challenges of transforming complex DDEX XML into clean models (parsing) and generating canonical, reproducible XML from those models. This is achieved through a dual **graph+flattened** data model for developer UX and an uncompromising approach to determinism, centered on a custom canonicalization specification, **DB-C14N/1.0**, and a **stable, content-addressable ID generation** engine.

Beyond the core implementation, this is a showcase of **software craftsmanship and platform thinking**. The suite provides consistent APIs, painless installation via prebuilt binaries, a hardened CI/CD pipeline, and robust supply-chain safety (SBOM, `cargo-deny`, and **Sigstore artifact signing**). Every feature reflects production wisdom—from the parser's XXE protection to the builder's versioned **partner presets system** with safety locks. Paired with my validator work (DDEX Workbench), DDEX Suite delivers a credible, end-to-end **Parse → Modify → Build** processing pipeline, complete with enterprise-grade features like **preflight validation**, a **semantic diff engine**, and a comprehensive CLI. It illustrates how to design interoperable components that are fast, safe, and easy to adopt in real-world systems.

## 🚧 Development Status

**Current Phase**: Phase 2 - Complete DDEX Parser v1.0 
**Target Release**: v1.0.0 in Q1 2026

### Progress Tracker

#### ✅ Phase 1: Foundation Refactor (COMPLETED)

**Monorepo Setup** ✅ **COMPLETED**

  - [x] Create ddex-suite repository
  - [x] Setup root Cargo.toml workspace
  - [x] Setup root package.json for npm workspaces
  - [x] Create packages/ directory structure
  - [x] Configure unified CI/CD pipelines
  - [x] Setup cross-package testing infrastructure
  - [x] Create migration scripts

**Migration & Core Extraction** ✅ **COMPLETED**

  - [x] Run migration script to move all files
  - [x] Extract models to packages/core/src/models/
  - [x] Extract errors to packages/core/src/error.rs
  - [x] Extract FFI types to packages/core/src/ffi.rs
  - [x] Update all import paths in packages/ddex-parser
  - [x] Add extension support to models
  - [x] Implement toBuildRequest() method
  - [x] Verify all 8 version tests pass
  - [x] Verify all 20 Node.js tests pass

#### 🔄 Phase 2: Complete DDEX Parser v1.0 (IN PROGRESS)

**2.1 Enhanced Parser Features** ✅ **COMPLETED**

  - [x] Add includeRawExtensions option
  - [x] Add includeComments option
  - [x] Implement extension preservation
  - [x] Add _graph reference to flattened models
  - [x] Complete toBuildRequest() implementation
  - [x] Test round-trip fidelity
  - [x] Add 10+ round-trip tests (basic tests complete, comprehensive tests pending)

**2.2 JavaScript/TypeScript Bindings** 🔄 **CURRENT**

  - [ ] Complete WASM browser build (<500KB)
  - [ ] Optimize with wasm-opt
  - [ ] Unify npm package (native + WASM)
  - [ ] Update package name to @ddex-suite/parser
  - [ ] Add streaming examples
  - [ ] Test in all major browsers
  - [ ] Publish to npm

**2.3-2.4 Python Bindings & CLI** ⏳ **UPCOMING**

  - [ ] Complete PyO3/maturin setup
  - [ ] Configure cibuildwheel for all platforms
  - [ ] Implement Python API
  - [ ] Add DataFrame integration
  - [ ] Build comprehensive CLI with clap
  - [ ] Tag parser v1.0.0

### Upcoming Phases

  - **Phase 3**: DDEX Builder Development (Week 5-14)
  - **Phase 4**: Suite Integration & Launch (Week 15-20)

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
  tracks: ParsedTrack[];         // Fully resolved with resources merged
  coverArt?: ParsedImage;
  _graph?: Release;              // Reference to original for full fidelity
  extensions?: Map<string, XmlFragment>; // Extensions preserved
}
```

## 🚀 Features (Planned)

- **Round-Trip Fidelity**: Parse → Modify → Build with 100% data preservation
- **Deterministic Output**: DB-C14N/1.0 stable hash algorithm for identical XML
- **Multi-Version**: Supports ERN 3.8.2, 4.2, and 4.3 with automatic detection
- **Partner Presets**: Optimized configurations for Spotify, Apple Music, Amazon
- **Cross-Platform**: Native bindings for Node.js, Python, and browsers (WASM)
- **Streaming**: Handle massive catalogs with backpressure and progress callbacks
- **Type Safety**: Fully typed interfaces in TypeScript and Python (auto-generated from Rust)
- **Security**: Built-in XXE protection, entity expansion limits, timeout controls

## 📦 Installation

Packages will be available once each component reaches v1.0:

```bash
# JavaScript/TypeScript
npm install @ddex-suite/parser  # Available soon
npm install @ddex-suite/builder # Coming Q1 2026

# Python
pip install ddex-parser  # Available soon
pip install ddex-builder # Coming Q1 2026

# Rust
cargo add ddex-parser-core
cargo add ddex-builder-core
```

## 💻 Usage Examples (Coming Soon)

### JavaScript/TypeScript
```typescript
import { DDEXParser, DDEXBuilder } from '@ddex-suite/parser';

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

# Parse to structured data
parser = DDEXParser()
message = parser.parse(xml_content)

# Export to DataFrame for analysis
df = parser.to_dataframe(xml_content)

# Build from DataFrame
builder = DDEXBuilder()
xml = builder.from_dataframe(df, version='4.3')
```

## 🏗️ Architecture

Built as a monorepo with shared core components:

```
┌─────────────────────────────────────────┐
│            DDEX Suite                   │
├─────────────────┬───────────────────────┤
│   DDEX Parser   │   DDEX Builder        │
│  Read & Parse   │  Generate & Build     │
├─────────────────┴───────────────────────┤
│           Shared Core                   │
│   Models │ Errors │ FFI │ Utils        │
├─────────────────────────────────────────┤
│         Language Bindings               │
│  napi-rs │ PyO3 │ WASM │ CLI           │
└─────────────────────────────────────────┘
```

## 🔒 Security

- XXE (XML External Entity) protection
- Entity expansion limits (billion laughs protection)
- Deep nesting protection
- Size and timeout limits
- Memory-bounded streaming
- Supply chain security with cargo-deny and SBOM

## 📊 Performance Targets

| Operation | Target | Status |
|-----------|--------|--------|
| Parse 10KB | <5ms | ✅ Achieved |
| Parse 100KB | <10ms | ✅ Achieved |
| Parse 1MB | <50ms | ✅ Achieved |
| Parse 100MB | <5s | ✅ Achieved |
| Stream 1GB | <60s with <100MB memory | ✅ Achieved |
| Build typical release | <15ms | 🔄 In Development |
| Round-trip fidelity | 100% | 🔄 In Development |
| Deterministic output | 100% identical | 🔄 In Development |

## 📚 Documentation

- [Blueprint](./blueprint.md) - Detailed architecture and roadmap
- [Parser API](./packages/ddex-parser/docs/API.md) - Parser documentation
- [Builder API](./packages/ddex-builder/docs/API.md) - Builder documentation (coming soon)
- [Round-Trip Guide](./docs/ROUND_TRIP.md) - Parse → Modify → Build guide (coming soon)
- [Error Handbook](./docs/ERROR_HANDBOOK.md) - Understanding and handling errors

## 🤝 Contributing

This project is in early development and not yet ready for external contributions. We're targeting community involvement starting in 2026 once the core architecture is stable.

Follow the project for updates!

## 📜 License

MIT License - see [LICENSE](./LICENSE) file for details.

## 🙏 Acknowledgments

DDEX Suite is designed to complement [DDEX Workbench](https://github.com/ddex/ddex-workbench) by providing structural parsing and deterministic generation while Workbench handles XSD validation and business rules.

---

**Repository**: https://github.com/daddykev/ddex-suite  
**Status**: Phase 1 Complete, Phase 2 In Progress