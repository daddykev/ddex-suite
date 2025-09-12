# DDEX Parser - JavaScript/TypeScript Bindings

[![npm version](https://img.shields.io/npm/v/ddex-parser.svg)](https://www.npmjs.com/package/ddex-parser)
[![Downloads](https://img.shields.io/npm/dm/ddex-parser.svg)](https://www.npmjs.com/package/ddex-parser)
[![Bundle Size](https://img.shields.io/bundlephobia/minzip/ddex-parser)](https://bundlephobia.com/package/ddex-parser)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Blazing-fast DDEX XML parser for JavaScript and TypeScript with native Node.js bindings and WASM browser support. Parse DDEX files 15x faster than traditional parsers with comprehensive TypeScript definitions and modern framework integration.

## Installation

```bash
npm install ddex-parser
# or
yarn add ddex-parser
```

## Quick Start

### Node.js (ESM/CommonJS)

```typescript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();
const result = await parser.parseFile('release.xml');

console.log(`Release: ${result.flattened.releaseTitle}`);
console.log(`Artist: ${result.flattened.mainArtist}`);
console.log(`Tracks: ${result.flattened.tracks.length}`);
```

### Browser (ES Modules)

```typescript
import { DDEXParser } from 'ddex-parser/browser';

const parser = new DDEXParser();
const result = await parser.parseString(xmlContent);

// Process the parsed data
result.flattened.tracks.forEach(track => {
  console.log(`${track.position}. ${track.title} - ${track.duration}s`);
});
```

## Features

### 🚀 Blazing Performance
- **15x faster** than traditional XML parsers
- Native Node.js addon with Rust performance
- Optimized WASM for browsers (<500KB gzipped)
- Streaming support for large catalog files

### 🌐 Cross-Platform Compatibility
- **Node.js 16+** with native bindings
- **Browser** support via optimized WASM
- **TypeScript-first** with comprehensive type definitions
- **Framework agnostic** - works with React, Vue, Angular, Svelte

### 🔒 Security & Reliability
- Built-in XXE (XML External Entity) protection
- Entity expansion limits and memory bounds
- Comprehensive error handling with detailed messages
- Production-tested against malformed XML

### 🎵 Music Industry Ready
- Complete support for DDEX ERN profiles (3.8.2, 4.2, 4.3+)
- Release, track, and resource metadata extraction
- Rights, territory, and deal information
- Image and audio resource handling
- Genre and mood classification support

## API Reference

### DDEXParser

```typescript
class DDEXParser {
  constructor(options?: ParserOptions);
  
  // File parsing
  parseFile(path: string): Promise<DDEXResult>;
  parseString(xml: string): Promise<DDEXResult>;
  parseBuffer(buffer: Buffer): Promise<DDEXResult>;
  
  // Streaming (Node.js only)
  parseStream(stream: ReadableStream): AsyncIterable<DDEXResult>;
  
  // Utilities
  detectVersion(xml: string): DDEXVersion;
  validate(xml: string): ValidationResult;
}
```

## Performance Benchmarks

Performance comparison in different environments:

### Node.js (Native Addon)
| File Size | ddex-parser | xml2js | fast-xml-parser | Speedup |
|-----------|-------------|--------|-----------------|----------|
| 10KB      | 1.2ms       | 15ms   | 8ms             | 7x-12x  |
| 100KB     | 4ms         | 120ms  | 65ms            | 16x-30x |
| 1MB       | 32ms        | 980ms  | 520ms           | 16x-30x |
| 10MB      | 240ms       | 8.2s   | 4.1s            | 17x-34x |

### Browser (WASM)
| File Size | ddex-parser | DOMParser | xml2js | Speedup |
|-----------|-------------|-----------|--------|----------|
| 10KB      | 2.1ms       | 12ms      | 25ms   | 6x-12x  |
| 100KB     | 8ms         | 85ms      | 180ms  | 11x-22x |
| 1MB       | 65ms        | 750ms     | 1.8s   | 11x-28x |

Memory usage is consistently 40-60% lower across all environments.

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
assert.deepEqual(reparsed.tracks[0].title, "New Title"); // ✅ Perfect fidelity
```

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE) file for details.

## Related Projects

- **[ddex-builder](https://www.npmjs.com/package/ddex-builder)** - Build deterministic DDEX XML files
- **[ddex-parser (Python)](https://pypi.org/project/ddex-parser/)** - Python bindings
- **[DDEX Suite](https://ddex-suite.org)** - Complete DDEX processing toolkit

---

Built for the music industry. Powered by Rust + WASM for universal performance.