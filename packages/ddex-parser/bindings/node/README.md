# DDEX Parser

[![npm version](https://img.shields.io/npm/v/ddex-parser.svg)](https://www.npmjs.com/package/ddex-parser)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-ddex--suite-blue)](https://github.com/daddykev/ddex-suite)

High-performance DDEX XML parser with native Node.js bindings and WASM support for browsers. Parse and transform DDEX messages (ERN 3.8.2, 4.2, 4.3) with blazing speed and perfect compliance.

Part of the [DDEX Suite](https://github.com/daddykev/ddex-suite) - a comprehensive toolkit for working with DDEX metadata in the music industry.

## 🚧 v0.1.0 - Foundation Release

This is the initial npm release establishing the package structure and TypeScript API. The full parser implementation with native Rust performance and WASM browser support is actively in development.

**Current Status:**
- ✅ Package structure and TypeScript definitions
- ✅ API design finalized
- 🚧 Native Rust bindings (coming in v0.2.0)
- 🚧 WASM browser support (coming in v0.2.0)
- 🚧 Streaming parser (coming in v0.3.0)

## ✨ Features (Roadmap)

### Available Now (v0.1.0)
- 📦 **Package Structure**: Clean npm package with TypeScript support
- 🎯 **API Design**: Future-proof API that won't break when implementation lands
- 📊 **Dual Model Architecture**: Graph model for compliance, flattened model for ease of use

### Coming Soon
- 🚀 **Blazing Fast** (v0.2.0): Parse typical releases in <50ms with native Rust
- 🌐 **Universal** (v0.2.0): Native Node.js addon + optimized WASM (<500KB) for browsers
- 🔄 **Streaming Support** (v0.3.0): Handle gigabyte catalogs with bounded memory
- 🛡️ **Security** (v0.2.0): Built-in XXE protection, entity expansion limits
- 🔗 **Perfect Round-Trip** (v1.0.0): Parse → Modify → Build with [`ddex-builder`](https://github.com/daddykev/ddex-suite)

## 📦 Installation

```bash
npm install ddex-parser
```

## 🚀 Quick Start

```javascript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();

// API is ready - implementation coming in v0.2.0
const result = await parser.parse(xmlContent);

// Will return mock data in v0.1.0
// Full parsing in v0.2.0
console.log(result.flat.releases);
console.log(result.graph);
```

## 🎭 Dual Model Architecture

The parser will provide two complementary views of DDEX data:

### Graph Model (Faithful Representation)
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
Denormalized and resolved for easy consumption - ideal for applications:

```typescript
interface ParsedRelease {
  releaseId: string;
  title: string;
  displayArtist: string;
  duration: number;
  tracks: ParsedTrack[];   // Fully resolved with resources merged
  coverArt?: ParsedImage;
  territories: TerritoryInfo[];
}
```

## 💻 Usage Examples (v0.2.0+)

These examples show the API that will be fully functional in v0.2.0:

### Basic Parsing
```javascript
const result = await parser.parse(xmlContent, {
  includeRawExtensions: true,  // Preserve unknown XML elements
  includeComments: true,       // Preserve XML comments
  validateReferences: true      // Validate all references
});

// Access both models
const { graph, flat } = result;
```

### Streaming Large Files (v0.3.0)
```javascript
import { createReadStream } from 'fs';

const stream = createReadStream('huge-catalog.xml');

await parser.stream(stream, {
  onRelease: (release) => {
    console.log(`Processing: ${release.title}`);
  },
  onProgress: (progress) => {
    console.log(`Progress: ${progress.percentage}%`);
  }
});
```

## 📊 Performance Targets

When fully implemented (v0.2.0+), the parser will achieve:

| File Size | Parse Time | Memory | Mode |
|-----------|------------|--------|------|
| 10KB | <5ms | 2MB | DOM |
| 100KB | <10ms | 5MB | DOM |
| 1MB | <50ms | 20MB | DOM |
| 100MB | <5s | 50MB | Stream |
| 1GB | <60s | 100MB | Stream |

## 🛣️ Development Roadmap

### Phase 2.2: JavaScript/TypeScript Bindings (Current)
- ✅ v0.1.0 - Package structure, API design, TypeScript definitions
- 🚧 v0.2.0 - Native Rust bindings via napi-rs
- 🚧 v0.2.0 - WASM browser support <500KB

### Phase 2.3: Python Package
- 📅 v0.3.0 - Streaming parser implementation
- 📅 v0.4.0 - Python package via PyO3

### Phase 3: DDEX Builder
- 📅 v1.0.0 - Complete suite with [`ddex-builder`](https://github.com/daddykev/ddex-suite)
- 📅 v1.0.0 - Perfect round-trip: Parse → Modify → Build

## 👨‍💻 About This Project

DDEX Suite is being built as a rigorous, production-grade toolkit for music industry metadata processing. It combines a single Rust core with native bindings for JavaScript and Python, showcasing cross-language API design and deep ecosystem integration.

The project tackles the complementary challenges of:
- **Parser**: Transform complex DDEX XML into clean, strongly-typed models
- **Builder**: Generate deterministic, byte-perfect DDEX XML (coming soon)

Built with a focus on:
- 🔒 Security hardening (XXE protection, memory bounds)
- ⚡ Performance optimization (native Rust, WASM)
- 🎯 Developer experience (dual models, TypeScript)
- 🔄 Perfect round-trip fidelity

## 📄 License

MIT © Kevin Marques Moo

## 🙏 Acknowledgments

This parser is designed to complement the official [DDEX Workbench](https://github.com/ddex/ddex-workbench) by providing structural parsing while Workbench handles XSD validation.

Special thanks to the DDEX community for their standards documentation and to everyone who provides feedback during this early development phase.

---

**Version**: 0.1.0
**Status**: Early Alpha - Not Production Ready  
**Repository**: https://github.com/daddykev/ddex-suite  
**NPM**: https://www.npmjs.com/package/ddex-parser  
**Author**: Kevin Marques Moo

*Thank you for trying this early release! Your feedback helps shape the future of DDEX Suite.*