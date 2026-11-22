# DDEX Builder - JavaScript/TypeScript Bindings

[![npm version](https://img.shields.io/npm/v/ddex-builder.svg)](https://www.npmjs.com/package/ddex-builder)
[![Downloads](https://img.shields.io/npm/dm/ddex-builder.svg)](https://www.npmjs.com/package/ddex-builder)
[![Bundle Size](https://img.shields.io/bundlephobia/minzip/ddex-builder)](https://bundlephobia.com/package/ddex-builder)
[![TypeScript](https://img.shields.io/badge/TypeScript-Ready-blue.svg)](https://www.typescriptlang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Generate deterministic, industry-compliant DDEX XML files from JavaScript/TypeScript with consistent reproducibility. Build DDEX messages with full TypeScript support, streaming capabilities, and partner-specific presets for major platforms.

## Installation

```bash
npm install ddex-builder
# or for specific version
npm install ddex-builder@0.5.0
# or
yarn add ddex-builder
```

> **✅ v0.4.4 Compatibility**
> Enhanced round-trip compatibility with ddex-parser v0.4.4's strict validation. Improved error handling for better workflow integration.

## Documentation

📚 **API Documentation**: Complete TypeScript/JavaScript API documentation is available in the `/docs` directory. Open `docs/index.html` in your browser to browse the full API reference with examples, type definitions, and cross-references.

**Key Documentation Features:**
- Complete API reference for all classes and interfaces
- TypeScript type definitions with IntelliSense support
- Comprehensive examples and usage patterns
- Search functionality across all documentation
- Cross-references between related types and methods

Full documentation will be published online with the v1.0.0 release.

## Quick Start

### TypeScript

```typescript
import { DdexBuilder } from 'ddex-builder';

const builder = new DdexBuilder({ validate: true });

const releaseData = {
  messageHeader: {
    senderName: 'My Record Label',
    messageId: 'RELEASE_2024_001',
    sentDate: new Date('2024-01-15T10:30:00Z')
  },
  releases: [{
    releaseId: 'REL001',
    title: 'Amazing Album',
    mainArtist: 'Incredible Artist',
    labelName: 'My Record Label',
    releaseDate: '2024-02-01',
    genres: ['Pop', 'Electronic'],
    tracks: [{
      trackId: 'TRK001',
      title: 'Hit Song',
      position: 1,
      duration: 195,
      isrc: 'US1234567890',
      artists: ['Incredible Artist']
    }]
  }]
};

const xml = await builder.buildFromObject(releaseData, { version: '4.3' });
console.log('Generated DDEX XML:', xml.length, 'bytes');
```

## Features

### 🎯 Deterministic Output
- **100% reproducible** XML generation with stable hash IDs
- DB-C14N/1.0 canonicalization for deterministic consistency
- Stable ordering ensures identical output across Node.js versions
- Content-addressable resource IDs for reliable references

### 🌐 Cross-Platform Compatibility
- **Node.js 16+** with native addon performance
- **Browser support** via optimized WASM bundle (<400KB)
- **TypeScript-first** with comprehensive type definitions
- **ESM and CommonJS** support for maximum compatibility

### 🏭 Industry Presets
- **YouTube Music**: Content ID and monetization settings
- **Generic**: Default preset for broad distributor compatibility

### 🚀 High Performance
- Native Rust core with optimized Node.js bindings
- Streaming generation for large catalogs (>10,000 tracks)
- Memory-efficient processing with configurable limits
- Async/await throughout with proper backpressure handling

### 🔒 Built-in Validation
- Real-time DDEX schema validation with detailed error messages
- Business rule enforcement for industry compliance
- Reference integrity checking across the entire message
- Territory and rights validation with suggestion engine

## Performance Benchmarks

Performance comparison in different environments:

### Node.js (Native Addon)
| Dataset Size | Build Time | Memory Usage | Output Size | Throughput |
|--------------|------------|-------------|-------------|------------|
| Single release (10 tracks) | 3ms | 8MB | 25KB | 333 releases/sec |
| Album catalog (100 releases) | 25ms | 35MB | 2.5MB | 40 releases/sec |
| Label catalog (1000 releases) | 180ms | 120MB | 25MB | 5.6 releases/sec |
| Large catalog (10000 releases) | 1.8s | 300MB | 250MB | 5.6 releases/sec |

### Browser (WASM)
| Dataset Size | Build Time | Memory Usage | Bundle Impact |
|--------------|------------|-------------|---------------|
| Single release | 8ms | 12MB | 394KB (gzipped) |
| Small catalog (50 releases) | 85ms | 25MB | No additional |
| Medium catalog (500 releases) | 650ms | 80MB | No additional |

Memory usage remains constant with streaming mode regardless of dataset size.

## Round-Trip Compatibility

Perfect integration with ddex-parser for complete workflows:

```typescript
import { DdexParser } from 'ddex-parser';
import { DdexBuilder } from 'ddex-builder';

// Parse existing DDEX file
const parser = new DdexParser();
const original = await parser.parseFile('input.xml');

// Modify specific fields
const modified = { ...original.flattened };
modified.releases[0].title = 'Remastered Edition';

// Build new deterministic XML
const builder = new DdexBuilder({ canonical: true });
const newXml = await builder.buildFromFlattened(modified);

// Perfect round-trip fidelity guaranteed
const reparsed = await parser.parseString(newXml);
console.assert(reparsed.releases[0].title === 'Remastered Edition');
```

## Security

v0.4.0 includes comprehensive security enhancements:
- Zero unsafe code, comprehensive input validation
- Supply chain security with cargo-deny and SBOM
- Memory-bounded processing with configurable limits
- Built-in validation prevents malformed output
- Deterministic generation prevents injection attacks

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE) file for details.

## Related Projects

- **[ddex-parser](https://www.npmjs.com/package/ddex-parser)** - Parse DDEX XML files to JavaScript objects
- **[ddex-builder (Python)](https://pypi.org/project/ddex-builder/)** - Python bindings
- **[DDEX Suite](https://ddex-suite.org)** - Complete DDEX processing toolkit

---

Built with ❤️ for the music industry. Powered by Rust + TypeScript for deterministic, type-safe DDEX generation.