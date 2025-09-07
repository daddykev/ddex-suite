# DDEX Parser

![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange?logo=rust)
![Node.js](https://img.shields.io/badge/Node.js-18%20|%2020%20|%2022-green?logo=node.js)
![Python](https://img.shields.io/badge/Python-3.8%20|%203.9%20|%203.10%20|%203.11%20|%203.12-blue?logo=python)
![TypeScript](https://img.shields.io/badge/TypeScript-5.0%2B-blue?logo=typescript)
![Platform](https://img.shields.io/badge/Platform-Linux%20|%20macOS%20|%20Windows-lightgrey)

High-performance DDEX XML parser with native bindings for TypeScript/JavaScript and Python. Built on a single Rust core for consistent behavior across all platforms.

**"One parser, every language, structural parsing excellence"**

## 🎯 Why DDEX Parser?

Working with DDEX XML shouldn't feel like archaeology. This parser transforms complex DDEX messages into clean, strongly-typed data structures that are as easy to work with as JSON.

### Core Value Proposition
- **Single Rust Core**: One implementation to rule them all - consistent behavior across JavaScript, Python, and Rust
- **Dual Model Architecture**: Choose between faithful graph representation or developer-friendly flattened view
- **Production Ready**: Built-in XXE protection, memory-bounded streaming, and comprehensive security hardening
- **Blazing Fast**: Parse typical releases in <50ms, stream gigabyte files with <100MB memory

## 👨🏻‍💻 Developer Statement

I'm building **DDEX Parser** as a rigorous, end-to-end learning project to deepen my Rust skills while unifying my JavaScript and Python experience into a single, production-grade library. The intent is to ship a **single Rust core** that parses complex DDEX XML into clean, strongly-typed models, then expose it through **napi-rs** for Node/TypeScript and **PyO3** for Python—demonstrating cross-language API design, FFI ergonomics, and disciplined performance engineering. The project is deliberately "industry-shaped": it targets real music-metadata workloads, implements a dual **graph + flattened** data model for developer UX, and bakes in XML security (XXE/billion-laughs protections), reference resolution, streaming for large files, and reproducible benchmarks across platforms.

Beyond parsing, this is a showcase of **software craftsmanship and platform thinking**: consistent APIs across ecosystems, prebuilt binaries and wheels for painless install, a hardened CI/CD pipeline, supply-chain safety (SBOM, cargo-deny, signing), and a comprehensive test suite spanning fuzzing, performance, and compatibility. Paired with my validator work (DDEX Workbench), DDEX Parser rounds out a credible processing pipeline—**validator for correctness, parser for structure and transformation**—illustrating how I design interoperable components that are fast, safe, and easy to adopt in real systems.

## 🚧 Development Status

**Current Phase**: Phase 3 - JavaScript/TypeScript Bindings (Week 11-14)  
**Target Release**: v1.0.0 in Q4 2025

### Progress Tracker

#### ✅ Phase 0: Project Setup (COMPLETED)

  - [x] Repository structure and Cargo workspace
  - [x] CI/CD pipelines for all platforms
  - [x] Security framework (cargo-deny, SBOM)
  - [x] Test corpus generation
  - [x] Blueprint and architecture finalized

#### ✅ Phase 1: Rust Core Foundation (COMPLETED)

**Week 3-4: Core Parser Infrastructure** ✅ **COMPLETED**

  - [x] Rust workspace with feature flags
  - [x] quick-xml integration for secure parsing
  - [x] Version/namespace detection (ERN 3.8.2, 4.2, 4.3)
  - [x] Security configurations (XXE protection, limits)
  - [x] Error types with XPath-like location tracking
  - [x] Security test suite (billion laughs, deep nesting)

**Week 5-6: Dual Model Implementation** ✅ **COMPLETED**

  - [x] Graph model structs (faithful DDEX representation)
  - [x] Flattened model (developer-friendly)
  - [x] Bidirectional transformation
  - [x] Reference resolution with integrity
  - [x] Multi-language support (LocalizedString)
  - [x] Identifier normalization

#### ✅ Phase 2: Advanced Parsing Features (COMPLETED)

**Week 7-8: Streaming & Performance** ✅ **COMPLETED**

  - [x] SAX-based streaming parser implemented
  - [x] Progress callbacks with backpressure added
  - [x] Auto mode selection (DOM vs. stream) built
  - [x] Performance targets met and benchmarks published

**Week 9-10: ERN Version Support** ✅ **COMPLETED**

  - [x] Add ERN 4.2 model variations
  - [x] Add ERN 3.8.2 model variations
  - [x] Handle complex DealTerms across versions
  - [x] Test with vendor-quirk corpus
  - [x] Document version differences
  - [x] Build compatibility matrix
  - [x] Add migration helpers

#### ✅ Pre-Phase 3: Foundation Improvements (COMPLETED)

**Critical improvements based on feedback analysis** ✅ **COMPLETED**

Before starting the language bindings, we implemented foundational improvements to ensure robust cross-language support:

**TypeScript Generation & Type Safety**
  - [x] Integrated ts-rs for automatic TypeScript definition generation
  - [x] Eliminated manual type synchronization between Rust and TypeScript
  - [x] Configured type generation with proper feature flags
  - [x] Ensured all models can be automatically exported to TypeScript

**FFI Error Contract**
  - [x] Created FFI-friendly error representation (`FFIError`)
  - [x] Added structured error categories and severity levels
  - [x] Implemented JSON-serializable error format for all bindings
  - [x] Provided helpful hints for error resolution
  - [x] Established clear spec for cross-boundary error handling

**Model Integrity & Testing**
  - [x] Verified graph to flat model transformation consistency
  - [x] Tested round-trip preservation (serialize → deserialize)
  - [x] Validated computed fields generation
  - [x] Ensured dual model approach maintains data integrity

**Documentation & Contracts**
  - [x] Created comprehensive error contract documentation
  - [x] Defined error codes and categories
  - [x] Provided language-specific error handling examples
  - [x] Established clear specifications for FFI boundaries

**Test Results:**
- FFI Error Module: All tests passing ✅
- Model Consistency: 4/4 tests passing ✅
- Error Contract: 3/3 tests passing ✅
- TypeScript Feature: Builds successfully ✅

**Impact:** These improvements provide a rock-solid foundation for language bindings, ensuring automatic type synchronization, consistent error handling across all languages, verified data model integrity, and clear contracts for FFI boundaries.

#### 🔄 Phase 3: JavaScript/TypeScript Bindings (IN PROGRESS)

**Week 11-12: Node.js Native Addon** 🔄 **CURRENT**

  - [ ] Setup napi-rs project structure
  - [ ] Configure prebuildify for multi-platform builds
  - [ ] Expose parse/stream/sanityCheck functions
  - [x] Generate TypeScript definitions from Rust (completed in Pre-Phase 3)
  - [ ] Implement async iterator with backpressure
  - [ ] Add progress callbacks

**Week 13-14: Browser Support (WASM)** ⏳ **UPCOMING**

  - [ ] Setup wasm-bindgen with feature flags
  - [ ] Optimize for <500KB with wasm-opt
  - [ ] Implement Web Streams API support
  - [ ] Add Web Worker support
  - [ ] Create browser-specific examples
  - [ ] Publish to npm as unified package

### Upcoming Phases

  - **Phase 4**: Python Bindings (Week 15-18)
  - **Phase 5**: CLI & Developer Tools (Week 19-20)
  - **Phase 6**: DDEX Workbench Integration (Week 21-22)
  - **Phase 7**: Extended Message Support (Week 23-26)
  - **Phase 8**: v1.0 Release (Week 27-28)

## 🎭 Dual Model Architecture

The parser provides two complementary views of the same data:

### Graph Model (Faithful)
Preserves the exact DDEX structure with references - perfect for validation and compliance:
```typescript
interface ERNMessage {
  messageHeader: MessageHeader;
  parties: Party[];        // All parties with IDs
  resources: Resource[];   // Audio, video, image resources
  releases: Release[];     // Release metadata with references
  deals: Deal[];          // Commercial terms
}
```

### Flattened Model (Developer-Friendly)
Denormalized and resolved for easy consumption - ideal for applications and data pipelines:
```typescript
interface ParsedRelease {
  releaseId: string;
  title: string;
  displayArtist: string;
  tracks: ParsedTrack[];   // Fully resolved with resources merged
  coverArt?: ParsedImage;
  territories: TerritoryInfo[];
  // ... simplified, denormalized fields
}
```

## 🚀 Features (Planned)

- **Performance**: Parse typical releases in <50ms, stream GB files with bounded memory
- **Security**: Built-in XXE protection, entity expansion limits, timeout controls
- **Multi-Version**: Supports ERN 3.8.2, 4.2, and 4.3 with automatic detection
- **Cross-Platform**: Native bindings for Node.js, Python, and browsers (WASM)
- **Streaming**: Handle massive catalogs with backpressure and progress callbacks
- **Type Safety**: Fully typed interfaces in TypeScript and Python (auto-generated from Rust)
- **Reference Resolution**: Automatic resolution with integrity guarantees
- **Error Handling**: Structured, FFI-friendly errors with helpful hints and categories

## 📦 Installation

Packages will be available once v1.0 is released:

```bash
# JavaScript/TypeScript
npm install ddex-parser

# Python
pip install ddex-parser

# Rust
cargo add ddex-parser-core
```

## 💻 Usage Examples (Coming Soon)

### JavaScript/TypeScript
```typescript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();

// Parse with dual model
const result = await parser.parse(xmlContent);
console.log(result.flat.releases[0].title);  // Easy access
console.log(result.graph.parties);            // Full structure

// Stream large files
for await (const release of parser.stream(fileStream)) {
  await processRelease(release);
}

// Handle errors with structured information
try {
  const result = await parser.parse(invalidXml);
} catch (error) {
  console.log(error.code);       // 'XML_PARSE_ERROR'
  console.log(error.category);   // 'XmlParsing'
  console.log(error.hint);        // 'Check XML syntax and ensure it's well-formed'
  console.log(error.location);    // { line: 42, column: 10, path: '/Release' }
}
```

### Python
```python
from ddex_parser import DDEXParser

parser = DDEXParser()

# Parse to structured data
message = parser.parse(xml_content)
print(message.flat.releases[0].title)

# Export to DataFrame
df = parser.to_dataframe(xml_content, schema='flat')

# Handle errors with structured information
try:
    message = parser.parse(invalid_xml)
except DDEXParseError as e:
    print(e.code)        # 'XML_PARSE_ERROR'
    print(e.category)    # 'XmlParsing'
    print(e.hint)        # Helpful suggestion
    print(e.location)    # Error location in XML
```

## 🏗️ Architecture

Built on a single Rust core with native bindings:

```
┌─────────────────────────────────────────┐
│            Applications                 │
├────────┬────────┬───────────────────────┤
│   JS   │ Python │        Rust           │
├────────┴────────┴───────────────────────┤
│         Language Bindings               │
│  napi-rs │ PyO3 │ WASM │ Direct         │
├─────────────────────────────────────────┤
│         Rust Core (ddex-parser-core)    │
│   Parser │ Transform │ Security         │
│   Models │ References │ Streaming       │
│   FFI Contract │ Type Generation        │
└─────────────────────────────────────────┘
```

## 🔒 Security

- XXE (XML External Entity) protection
- Entity expansion limits (billion laughs protection)
- Deep nesting protection
- Size and timeout limits
- Memory-bounded streaming
- No network access by default

## 📊 Performance Targets

| File Size | Parse Time | Memory | Mode |
|-----------|------------|--------|------|
| 10KB | <5ms | <2MB | DOM |
| 100KB | <10ms | <5MB | DOM |
| 1MB | <50ms | <20MB | DOM |
| 10MB | <500ms | <100MB | Auto |
| 100MB | <5s | <50MB | Stream |
| 1GB | <60s | <100MB | Stream |

## 📚 Documentation

- [Blueprint](./blueprint.md) - Detailed architecture and roadmap
- [API Reference](./docs/API.md) - Complete API documentation (coming soon)
- [Security Guide](./governance/SECURITY.md) - Threat model and mitigations (coming soon)
- [Error Handbook](./docs/ERROR_HANDBOOK.md) - Understanding and handling errors (coming soon)

## 🤝 Contributing

This project is in early development and not yet ready for external contributions. We're targeting community involvement starting in 2026 once the core architecture is stable.

Follow the project for updates!

## 📜 License

MIT License - see [LICENSE](./LICENSE) file for details.

## 🙏 Acknowledgments

This parser is designed to complement [DDEX Workbench](https://github.com/ddex/ddex-workbench) by providing structural parsing while Workbench handles XSD validation and business rules.

---

**Repository**: https://github.com/daddykev/ddex-parser  
**Status**: Pre-Phase 3 Complete, Phase 3 In Progress