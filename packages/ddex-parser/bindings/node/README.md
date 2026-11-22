# DDEX Parser - JavaScript/TypeScript Bindings
## DDEX XML Parser for Node.js

[![npm version](https://img.shields.io/npm/v/ddex-parser.svg)](https://www.npmjs.com/package/ddex-parser)
[![Downloads](https://img.shields.io/npm/dm/ddex-parser.svg)](https://www.npmjs.com/package/ddex-parser)
[![Bundle Size](https://img.shields.io/bundlephobia/minzip/ddex-parser)](https://bundlephobia.com/package/ddex-parser)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Fully functional DDEX XML parser for JavaScript and TypeScript with complete data structure access. Built on Rust with native Node.js bindings and WASM browser support - parse DDEX files with full access to releases, resources, deals, and tracks.

## Installation

```bash
npm install ddex-parser
# or for specific version
npm install ddex-parser@0.5.0
# or
yarn add ddex-parser
```

> **⚠️ v0.4.4 Breaking Changes**
> Enhanced validation now properly fails on missing required fields instead of using placeholder values. Ensure your error handling can catch validation errors for incomplete DDEX files.

## Quick Start

### Node.js (ESM/CommonJS)

```javascript
const { DdexParser } = require('ddex-parser');
const fs = require('fs');

const parser = new DdexParser();
const xmlContent = fs.readFileSync('release.xml', 'utf8');
const result = parser.parseSync(xmlContent);

// Full access to parsed data structures
console.log('Message ID:', result.messageId);
console.log('Total Releases:', result.releases.length);
console.log('Total Resources:', Object.keys(result.resources).length);
console.log('Total Deals:', result.deals.length);

// Access individual releases
result.releases.forEach((release, index) => {
  console.log(`Release ${index + 1}:`, {
    id: release.releaseId,
    title: release.title,
    artist: release.displayArtist,
    trackCount: release.tracks.length
  });
});
```

### Error Handling (v0.4.4+)

```javascript
const { DdexParser } = require('ddex-parser');

const parser = new DdexParser();

try {
  const result = parser.parseSync(xmlContent);
  console.log('Parse successful:', result.messageId);
} catch (error) {
  console.error('Parse failed:', error.message);

  // v0.4.4+ provides detailed field-specific errors
  if (error.message.includes('missing required field')) {
    console.log('Incomplete DDEX file - missing required fields');
  } else if (error.message.includes('XML parsing failed')) {
    console.log('Malformed XML structure');
  }
}
```

### Browser (ES Modules)

```typescript
import { DdexParser } from 'ddex-parser/browser';

const parser = new DdexParser();
const result = await parser.parseString(xmlContent);

// Process the parsed data
result.flattened.tracks.forEach(track => {
  console.log(`${track.position}. ${track.title} - ${track.duration}s`);
});
```

## Features

### 🚀 Blazing Performance
- Rust core: a typical ERN 4.3 album (75 KB) parses in about 3.4 ms (measured, Apple M1 Pro)
- Native Node.js addon with Rust performance
- Optimized WASM for browsers (<500KB gzipped)
- The complete parser for every input size (inputs up to 100 MB; no streaming in Node)

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

## Complete Data Structure

The parser now provides full access to all DDEX data structures:

```typescript
interface ParsedMessage {
  // Message metadata
  messageId: string;
  messageType: string;
  messageDate: string;
  senderName: string;
  senderId: string;
  recipientName: string;
  recipientId: string;
  version: string;
  profile?: string;

  // Legacy counts (for backward compatibility)
  releaseCount: number;
  trackCount: number;
  dealCount: number;
  resourceCount: number;
  totalDurationSeconds: number;

  // COMPLETE DATA STRUCTURES
  releases: Release[];              // Full release objects with tracks
  resources: { [key: string]: Resource }; // Resource dictionary with IDs as keys
  deals: Deal[];                   // Complete deal information
}

interface Release {
  releaseId: string;
  title: string;
  defaultTitle: string;
  subtitle?: string;
  displayArtist: string;
  releaseType: string;
  genre?: string;
  subGenre?: string;
  trackCount: number;
  discCount?: number;
  releaseDate?: string;
  originalReleaseDate?: string;
  labelName?: string;
  tracks: Track[];                 // Full track array
}

interface Track {
  trackId: string;
  title: string;
  artist: string;
  duration?: string;
  position?: number;
  discNumber?: number;
  isrc?: string;
  resourceReference?: string;
}

interface Resource {
  resourceId: string;
  resourceType: string;
  title: string;
  durationSeconds?: number;
  durationString?: string;
  fileFormat?: string;
  bitrate?: number;
  sampleRate?: number;
  fileSize?: string;
}

interface Deal {
  dealId: string;
  releases: string[];
  startDate?: string;
  endDate?: string;
  territories: string[];
  usageRights: string[];
  restrictions: string[];
  commercialModel: string;
}
```

## API Reference

### DdexParser

```typescript
class DdexParser {
  constructor();

  // Synchronous parsing with complete data access
  parseSync(xml: string, options?: ParseOptions): ParsedMessage;

  // Asynchronous parsing with complete data access
  parse(xml: string, options?: ParseOptions): Promise<ParsedMessage>;

  // Utilities
  detectVersion(xml: string): string;
  sanityCheck(xml: string): Promise<SanityCheckResult>;
}
```

`options` is accepted for compatibility but no option is supported: since 0.5.0 every
`ParseOptions` field throws (the streaming options `mode`, `autoThreshold`,
`enableStreaming`, `streamingThreshold`, and the ones that had no effect, such as
`resolveReferences`, `includeRaw`, `collectStatistics` and the fidelity options).
`ParsedMessage.statistics` and `fidelityInfo` were removed, and so was `stream()`.

## Comprehensive Usage Examples

### Working with Releases and Tracks

```javascript
const { DdexParser } = require('ddex-parser');
const parser = new DdexParser();
const result = parser.parseSync(xmlContent);

// Access all releases
console.log(`Found ${result.releases.length} releases`);

result.releases.forEach(release => {
  console.log(`\nRelease: ${release.title}`);
  console.log(`Artist: ${release.displayArtist}`);
  console.log(`Type: ${release.releaseType}`);
  console.log(`Release Date: ${release.releaseDate}`);
  console.log(`Track Count: ${release.tracks.length}`);

  // Access individual tracks
  release.tracks.forEach((track, index) => {
    console.log(`  Track ${index + 1}: ${track.title}`);
    console.log(`    Artist: ${track.artist}`);
    console.log(`    Duration: ${track.duration}`);
    console.log(`    ISRC: ${track.isrc}`);
  });
});
```

### Working with Resources

```javascript
const parser = new DdexParser();
const result = parser.parseSync(xmlContent);

// Access all resources
const resourceIds = Object.keys(result.resources);
console.log(`Found ${resourceIds.length} resources`);

resourceIds.forEach(resourceId => {
  const resource = result.resources[resourceId];
  console.log(`\nResource: ${resource.title}`);
  console.log(`Type: ${resource.resourceType}`);
  console.log(`Duration: ${resource.durationString}`);
  console.log(`File Format: ${resource.fileFormat}`);
  console.log(`Bitrate: ${resource.bitrate}`);
  console.log(`Sample Rate: ${resource.sampleRate}`);
});
```

### Working with Deals

```javascript
const parser = new DdexParser();
const result = parser.parseSync(xmlContent);

// Access all deals
console.log(`Found ${result.deals.length} deals`);

result.deals.forEach(deal => {
  console.log(`\nDeal: ${deal.dealId}`);
  console.log(`Releases: ${deal.releases.join(', ')}`);
  console.log(`Territories: ${deal.territories.join(', ')}`);
  console.log(`Start Date: ${deal.startDate}`);
  console.log(`End Date: ${deal.endDate}`);
  console.log(`Commercial Model: ${deal.commercialModel}`);
  console.log(`Usage Rights: ${deal.usageRights.join(', ')}`);
});
```

### Async Parsing

```javascript
const parser = new DdexParser();

try {
  const result = await parser.parse(xmlContent);
  console.log('Successfully parsed:', {
    messageId: result.messageId,
    releases: result.releases.length,
    resources: Object.keys(result.resources).length,
    deals: result.deals.length
  });
} catch (error) {
  console.error('Parse failed:', error.message);
}
```

### Version Detection and Validation

```javascript
const parser = new DdexParser();

// Detect DDEX version
const version = parser.detectVersion(xmlContent);
console.log('DDEX Version:', version); // e.g., "V4_3"

// Comprehensive validation
const sanityResult = await parser.sanityCheck(xmlContent);
if (sanityResult.isValid) {
  console.log(`Valid DDEX ${sanityResult.version}`);
} else {
  console.log('Validation errors:', sanityResult.errors);
  console.log('Warnings:', sanityResult.warnings);
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

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE) file for details.

## Related Projects

- **[ddex-builder](https://www.npmjs.com/package/ddex-builder)** - Build deterministic DDEX XML files
- **[ddex-parser (Python)](https://pypi.org/project/ddex-parser/)** - Python bindings
- **[DDEX Suite](https://ddex-suite.org)** - Complete DDEX processing toolkit

---

Built for the music industry. Powered by Rust + WASM for universal performance.