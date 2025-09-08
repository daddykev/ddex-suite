# DDEX Parser

[![npm version](https://img.shields.io/npm/v/ddex-parser.svg)](https://www.npmjs.com/package/ddex-parser)
[![npm downloads](https://img.shields.io/npm/dm/ddex-parser.svg)](https://www.npmjs.com/package/ddex-parser)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/daddykev/ddex-suite/workflows/CI/badge.svg)](https://github.com/daddykev/ddex-suite/actions)

High-performance DDEX XML parser with native Node.js bindings and WASM support for browsers. Parse and transform DDEX messages (ERN 3.8.2, 4.2, 4.3) with perfect compliance and blazing speed.

Part of the [DDEX Suite](https://github.com/daddykev/ddex-suite) - comprehensive tools for working with DDEX metadata.

## ✨ Features

- 🚀 **Blazing Fast**: Parse typical releases in <50ms, stream gigabyte catalogs with bounded memory
- 🔒 **Secure by Default**: Built-in XXE protection, entity expansion limits, timeout controls
- 🎯 **Multi-Version Support**: ERN 3.8.2, 4.2, and 4.3 with automatic detection
- 🌐 **Universal**: Works in Node.js (native Rust addon) and browsers (optimized WASM <500KB)
- 📊 **Dual Model Architecture**: Graph model for compliance, flattened model for ease of use
- 🔄 **Streaming Support**: Handle massive catalogs with backpressure and progress callbacks
- 📝 **Full TypeScript Support**: Complete type definitions auto-generated from Rust
- 🛡️ **Error Recovery**: Detailed error messages with location tracking and helpful hints
- 🔗 **Perfect Round-Trip**: Preserves extensions and comments for lossless parsing

## 📦 Installation

```bash
npm install ddex-parser
```

Or with yarn:
```bash
yarn add ddex-parser
```

Or with pnpm:
```bash
pnpm add ddex-parser
```

## 🚀 Quick Start

```javascript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();

// Parse DDEX XML
const xmlContent = fs.readFileSync('release.xml', 'utf-8');
const result = await parser.parse(xmlContent);

// Access the easy-to-use flattened model
console.log(result.flat.releases[0].title);
console.log(result.flat.releases[0].artists);
console.log(result.flat.releases[0].tracks);

// Or work with the faithful graph model
console.log(result.graph.releaseList);
console.log(result.graph.resourceList);
console.log(result.graph.dealList);
```

## 🎭 Dual Model Architecture

The parser provides two complementary views of the same data:

### Graph Model (Faithful Representation)
Preserves the exact DDEX structure with references - perfect for validation, compliance, and round-trip operations:

```typescript
interface ERNMessage {
  messageHeader: MessageHeader;
  parties: Party[];        // All parties with IDs
  resources: Resource[];   // Audio, video, image resources  
  releases: Release[];     // Release metadata with references
  deals: Deal[];          // Commercial terms
  
  // Extensions preserved for round-trip
  extensions?: Map<string, XmlFragment>;
  comments?: Comment[];
}
```

### Flattened Model (Developer-Friendly)
Denormalized and resolved for easy consumption - ideal for applications, data pipelines, and quick access:

```typescript
interface ParsedRelease {
  releaseId: string;
  title: string;
  displayArtist: string;
  duration: number;
  tracks: ParsedTrack[];   // Fully resolved with resources merged
  coverArt?: ParsedImage;
  territories: TerritoryInfo[];
  deals: ParsedDeal[];     // Simplified deal terms
  // ... all references resolved
}
```

## 💻 Usage Examples

### Basic Parsing

```javascript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();

// Parse with options
const result = await parser.parse(xmlContent, {
  includeRawExtensions: true,  // Preserve unknown XML elements
  includeComments: true,       // Preserve XML comments
  validateReferences: true      // Validate all references
});

// Access both models
const { graph, flat } = result;

// Work with flattened data (easy mode)
for (const release of flat.releases) {
  console.log(`${release.title} by ${release.displayArtist}`);
  console.log(`${release.tracks.length} tracks, ${release.duration}ms total`);
  
  for (const track of release.tracks) {
    console.log(`  - ${track.title} (${track.isrc})`);
  }
}
```

### Streaming Large Files

Handle massive catalogs without loading everything into memory:

```javascript
import { DDEXParser } from 'ddex-parser';
import { createReadStream } from 'fs';

const parser = new DDEXParser();
const stream = createReadStream('huge-catalog.xml');

// Stream with progress tracking
await parser.stream(stream, {
  onRelease: (release) => {
    console.log(`Processing: ${release.title}`);
    // Process each release as it's parsed
    return processRelease(release);
  },
  
  onProgress: (progress) => {
    console.log(`Progress: ${progress.percentage}% (${progress.releases} releases)`);
  },
  
  onError: (error, canContinue) => {
    console.error(`Error: ${error.message}`);
    return canContinue; // Continue parsing if possible
  }
});
```

### Browser Usage

The parser automatically uses WASM in browser environments:

```html
<script type="module">
import { DDEXParser } from 'https://unpkg.com/ddex-parser/dist/browser.js';

const parser = new DDEXParser();

// Parse files from file input
document.getElementById('file-input').addEventListener('change', async (e) => {
  const file = e.target.files[0];
  const text = await file.text();
  
  try {
    const result = await parser.parse(text);
    console.log('Parsed releases:', result.flat.releases);
  } catch (error) {
    console.error('Parse error:', error.message);
  }
});

// Or stream large files
async function streamFile(file) {
  const stream = file.stream();
  await parser.stream(stream, {
    onRelease: (release) => {
      addReleaseToUI(release);
    }
  });
}
</script>
```

### Error Handling

Get detailed, actionable error information:

```javascript
try {
  const result = await parser.parse(xmlContent);
} catch (error) {
  // Structured error information
  console.error(`Error Code: ${error.code}`);           // 'INVALID_REFERENCE'
  console.error(`Category: ${error.category}`);         // 'ReferenceValidation'  
  console.error(`Message: ${error.message}`);           // Human-readable description
  console.error(`Location: Line ${error.location.line}, Column ${error.location.column}`);
  console.error(`XPath: ${error.location.path}`);       // '/ReleaseList/Release[2]'
  console.error(`Hint: ${error.hint}`);                 // Helpful suggestion for fixing
  
  // Error context
  if (error.context) {
    console.error(`Release ID: ${error.context.releaseId}`);
    console.error(`Resource Ref: ${error.context.resourceReference}`);
  }
}
```

### Round-Trip Preservation

Parse, modify, and rebuild while preserving all original data:

```javascript
import { DDEXParser } from 'ddex-parser';
import { DDEXBuilder } from 'ddex-builder'; // Coming soon in v1.0

const parser = new DDEXParser();
const builder = new DDEXBuilder();

// Parse with full preservation
const result = await parser.parse(xmlContent, {
  includeRawExtensions: true,
  includeComments: true
});

// Modify the data
result.flat.releases[0].title = 'Updated Title';

// Rebuild with perfect fidelity
const newXml = await builder.build(result.flat, {
  preserveExtensions: result.graph.extensions,
  preserveComments: result.graph.comments,
  version: result.graph.version
});
```

## 🔒 Security

Built with security as a top priority:

- **XXE Protection**: XML External Entity attacks prevented by default
- **Billion Laughs Protection**: Entity expansion limits prevent DoS attacks
- **Size Limits**: Configurable limits on file size and parsing depth
- **Timeout Controls**: Prevent hanging on malformed input
- **Memory Bounds**: Streaming mode ensures bounded memory usage
- **No Network Access**: Parser never makes network requests

## 📊 Performance

Optimized for real-world DDEX files:

| File Size | Parse Time | Memory Usage | Mode |
|-----------|------------|--------------|------|
| 10KB | <5ms | 2MB | DOM |
| 100KB | <10ms | 5MB | DOM |
| 1MB | <50ms | 20MB | DOM |
| 10MB | <500ms | 50MB | Auto |
| 100MB | <5s | 50MB | Stream |
| 1GB | <60s | 100MB | Stream |

## 🌐 Platform Support

### Node.js
- Native Rust addon for maximum performance
- Supports Node.js 18.0.0 and higher
- Pre-built binaries for common platforms
- Automatic fallback to WASM if native fails

### Browsers
- Optimized WASM build (<500KB)
- Works in all modern browsers
- Web Streams API support
- Web Worker compatible

### Platforms
- Windows (x64, ARM64)
- macOS (x64, Apple Silicon)
- Linux (x64, ARM64, musl)

## 📚 API Reference

### Parser Options

```typescript
interface ParserOptions {
  // Preservation options
  includeRawExtensions?: boolean;  // Keep unknown XML elements (default: false)
  includeComments?: boolean;       // Keep XML comments (default: false)
  
  // Validation options  
  validateReferences?: boolean;    // Validate all ID references (default: true)
  strictMode?: boolean;           // Strict ERN compliance (default: false)
  
  // Performance options
  streaming?: boolean;            // Use streaming parser (default: auto)
  maxFileSize?: number;          // Max file size in bytes (default: 1GB)
  timeout?: number;              // Parse timeout in ms (default: 30000)
  
  // Version handling
  autoDetectVersion?: boolean;   // Auto-detect ERN version (default: true)
  assumeVersion?: ERNVersion;    // Force specific version
}
```

### Stream Options

```typescript
interface StreamOptions extends ParserOptions {
  // Callbacks
  onRelease?: (release: ParsedRelease) => void | Promise<void>;
  onProgress?: (progress: ProgressInfo) => void;
  onError?: (error: DDEXError, canContinue: boolean) => boolean;
  
  // Backpressure control
  highWaterMark?: number;         // Buffer size (default: 16)
  parallelism?: number;          // Concurrent processing (default: 1)
}
```

## 🛠️ Development

This parser is part of the [DDEX Suite](https://github.com/daddykev/ddex-suite) monorepo.

```bash
# Clone the monorepo
git clone https://github.com/daddykev/ddex-suite.git
cd ddex-suite

# Install dependencies
npm install

# Build the parser
cd packages/ddex-parser
npm run build

# Run tests
npm test

# Run benchmarks
npm run bench
```

## 🤝 Related Packages

- [ddex-builder](https://www.npmjs.com/package/ddex-builder) - Generate DDEX XML with DB-C14N/1.0 compliance (coming soon)
- [DDEX Workbench](https://github.com/ddex/ddex-workbench) - Official DDEX validation service

## 📈 Roadmap

- ✅ **v0.1.0** - Initial release with ERN 4.3 support
- ✅ **v0.2.0** - Multi-version support (ERN 3.8.2, 4.2)
- ✅ **v0.3.0** - Streaming parser
- ✅ **v0.4.0** - Round-trip preservation
- 🚧 **v0.5.0** - WASM browser support (in progress)
- 📅 **v1.0.0** - Production ready (Q4 2025)

## 📄 License

MIT © Kevin Marques Moo

## 🙏 Acknowledgments

This parser complements the official [DDEX Workbench](https://github.com/ddex/ddex-workbench) by providing structural parsing while Workbench handles XSD validation and business rules.

Special thanks to the DDEX community for their feedback and real-world test files.

## 🐛 Found an Issue?

Please report issues on our [GitHub repository](https://github.com/daddykev/ddex-suite/issues).

---

**Repository**: https://github.com/daddykev/ddex-suite  
**Documentation**: https://github.com/daddykev/ddex-suite/tree/main/packages/ddex-parser  
**NPM Package**: https://www.npmjs.com/package/ddex-parser