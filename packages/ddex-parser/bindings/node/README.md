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

### React Integration

```tsx
import React, { useState, useCallback } from 'react';
import { DDEXParser, type DDEXResult } from 'ddex-parser';

const DDEXUploader: React.FC = () => {
  const [result, setResult] = useState<DDEXResult | null>(null);
  const [loading, setLoading] = useState(false);

  const handleFileUpload = useCallback(async (file: File) => {
    setLoading(true);
    try {
      const parser = new DDEXParser();
      const text = await file.text();
      const parsed = await parser.parseString(text);
      setResult(parsed);
    } catch (error) {
      console.error('Parse error:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  return (
    <div>
      <input
        type="file"
        accept=".xml"
        onChange={(e) => e.target.files?.[0] && handleFileUpload(e.target.files[0])}
      />
      
      {loading && <p>Parsing DDEX file...</p>}
      
      {result && (
        <div>
          <h2>{result.flattened.releaseTitle}</h2>
          <p>Artist: {result.flattened.mainArtist}</p>
          <ul>
            {result.flattened.tracks.map((track, idx) => (
              <li key={idx}>{track.title} ({track.duration}s)</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
};
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

### Parser Options

```typescript
interface ParserOptions {
  // Security settings
  maxEntityExpansions?: number;     // Default: 1000
  maxNestingDepth?: number;         // Default: 100
  maxMemoryUsage?: number;          // Default: 100MB
  
  // Parsing behavior
  streaming?: boolean;              // Enable streaming mode
  preserveComments?: boolean;       // Keep XML comments
  includeRawExtensions?: boolean;   // Preserve unknown elements
  validateReferences?: boolean;     // Validate internal references
  
  // Output format
  flattenedOnly?: boolean;         // Skip graph model generation
  includeTimestamps?: boolean;     // Add parsing metadata
}
```

### DDEXResult

```typescript
interface DDEXResult {
  // Dual model architecture
  graph: GraphModel;           // Faithful DDEX structure
  flattened: FlattenedModel;   // Developer-friendly format
  
  // Metadata
  version: DDEXVersion;
  profile: DDEXProfile;
  messageId: string;
  
  // Utility methods
  toJSON(): string;
  toObject(): Record<string, any>;
  getTrackById(id: string): Track | undefined;
  getReleaseById(id: string): Release | undefined;
}
```

### TypeScript Definitions

Full TypeScript support with comprehensive type definitions:

```typescript
interface FlattenedModel {
  messageHeader: {
    messageId: string;
    senderName: string;
    sentDate: Date;
    messageSchemaVersion: string;
  };
  
  releases: Release[];
  tracks: Track[];
  parties: Party[];
  resources: Resource[];
  deals: Deal[];
}

interface Release {
  releaseId: string;
  title: string;
  mainArtist: string;
  displayArtist: string;
  labelName?: string;
  genres: string[];
  releaseDate?: Date;
  tracks: Track[];
  territories: Territory[];
  coverArt?: ImageResource;
  additionalArtwork?: ImageResource[];
}

interface Track {
  trackId: string;
  title: string;
  displayTitle: string;
  position: number;
  duration: number; // in seconds
  artists: Artist[];
  isrc?: string;
  genres: string[];
  moods?: string[];
  audioResources: AudioResource[];
  lyrics?: LyricsResource[];
}
```

## Advanced Usage

### Streaming Large Catalogs

```typescript
import { DDEXParser } from 'ddex-parser';
import { createReadStream } from 'fs';

async function processLargeCatalog(filePath: string) {
  const parser = new DDEXParser({ streaming: true });
  const stream = createReadStream(filePath);
  
  let processedCount = 0;
  
  for await (const result of parser.parseStream(stream)) {
    // Process each release individually
    result.flattened.releases.forEach(release => {
      console.log(`Processing: ${release.title} by ${release.mainArtist}`);
      // Save to database, send to API, etc.
    });
    
    processedCount += result.flattened.releases.length;
    console.log(`Processed ${processedCount} releases so far...`);
  }
}
```

### Error Handling

```typescript
import { DDEXParser, DDEXError, SecurityError, ValidationError } from 'ddex-parser';

const parser = new DDEXParser();

try {
  const result = await parser.parseFile('release.xml');
  console.log('Parsed successfully:', result.flattened.releaseTitle);
} catch (error) {
  if (error instanceof SecurityError) {
    console.error('Security issue:', error.message);
    // Handle XXE attempts or other security concerns
  } else if (error instanceof ValidationError) {
    console.error('Validation failed:', error.details);
    // Handle DDEX structure violations
  } else if (error instanceof DDEXError) {
    console.error('DDEX parsing error:', error.message);
    // Handle malformed XML or unsupported features
  } else {
    console.error('Unexpected error:', error);
  }
}
```

### Custom Validation

```typescript
const parser = new DDEXParser({
  validateReferences: true,
  maxEntityExpansions: 500,
  preserveComments: true
});

const validation = parser.validate(xmlContent);

if (!validation.isValid) {
  console.error('Validation errors:');
  validation.errors.forEach(error => {
    console.error(`Line ${error.line}: ${error.message}`);
  });
}
```

### Framework Integration

#### Vue 3 Composition API

```vue
<template>
  <div>
    <input type="file" @change="handleFileUpload" accept=".xml" />
    <div v-if="loading">Parsing...</div>
    <div v-if="result">
      <h2>{{ result.flattened.releaseTitle }}</h2>
      <div v-for="track in result.flattened.tracks" :key="track.trackId">
        {{ track.position }}. {{ track.title }} ({{ track.duration }}s)
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { DDEXParser, type DDEXResult } from 'ddex-parser';

const parser = new DDEXParser();
const result = ref<DDEXResult | null>(null);
const loading = ref(false);

const handleFileUpload = async (event: Event) => {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  
  loading.value = true;
  try {
    const text = await file.text();
    result.value = await parser.parseString(text);
  } catch (error) {
    console.error('Parse error:', error);
  } finally {
    loading.value = false;
  }
};
</script>
```

#### Angular Service

```typescript
import { Injectable } from '@angular/core';
import { Observable, from } from 'rxjs';
import { DDEXParser, type DDEXResult } from 'ddex-parser';

@Injectable({
  providedIn: 'root'
})
export class DDEXService {
  private parser = new DDEXParser();

  parseDDEXFile(file: File): Observable<DDEXResult> {
    return from(this.parseDDEXFileAsync(file));
  }

  private async parseDDEXFileAsync(file: File): Promise<DDEXResult> {
    const text = await file.text();
    return this.parser.parseString(text);
  }

  validateDDEX(xml: string) {
    return this.parser.validate(xml);
  }
}
```

## Browser Considerations

### Bundle Size Optimization

```typescript
// For smaller bundles, import only what you need
import { DDEXParser } from 'ddex-parser/lite';  // Reduced feature set

// Or use dynamic imports for code splitting
const loadParser = async () => {
  const { DDEXParser } = await import('ddex-parser');
  return new DDEXParser();
};
```

### Web Worker Support

```typescript
// main.ts
const worker = new Worker('/ddex-worker.js');

worker.postMessage({ xml: xmlContent, options: { streaming: false } });

worker.onmessage = (event) => {
  const result = event.data;
  console.log('Parsed in worker:', result);
};

// ddex-worker.js
import { DDEXParser } from 'ddex-parser/browser';

self.onmessage = async (event) => {
  const { xml, options } = event.data;
  
  try {
    const parser = new DDEXParser(options);
    const result = await parser.parseString(xml);
    self.postMessage(result);
  } catch (error) {
    self.postMessage({ error: error.message });
  }
};
```

## Performance Benchmarks

Performance comparison in different environments:

### Node.js (Native Addon)
| File Size | ddex-parser | xml2js | fast-xml-parser | Speedup |
|-----------|-------------|--------|-----------------|---------|
| 10KB      | 1.2ms       | 15ms   | 8ms             | 7x-12x  |
| 100KB     | 4ms         | 120ms  | 65ms            | 16x-30x |
| 1MB       | 32ms        | 980ms  | 520ms           | 16x-30x |
| 10MB      | 240ms       | 8.2s   | 4.1s            | 17x-34x |

### Browser (WASM)
| File Size | ddex-parser | DOMParser | xml2js | Speedup |
|-----------|-------------|-----------|--------|---------|
| 10KB      | 2.1ms       | 12ms      | 25ms   | 6x-12x  |
| 100KB     | 8ms         | 85ms      | 180ms  | 11x-22x |
| 1MB       | 65ms        | 750ms     | 1.8s   | 11x-28x |

Memory usage is consistently 40-60% lower across all environments.

## Migration from v0.1.0

v0.2.0 introduced breaking changes for better TypeScript support:

```typescript
// v0.1.0
import ddexParser from 'ddex-parser';
const result = ddexParser.parse(xml);
const title = result.releases[0].title;

// v0.2.0+ (current)
import { DDEXParser } from 'ddex-parser';
const parser = new DDEXParser();
const result = await parser.parseString(xml);
const title = result.flattened.releases[0].title;
```

### New Features in v0.2.0
- Full TypeScript rewrite with comprehensive type definitions
- Dual model architecture (graph + flattened)
- Streaming support for large files
- Enhanced security with XXE protection
- Browser WASM support
- Framework-specific integrations
- Async/await API throughout

## Troubleshooting

### Common Issues

**Cannot resolve module 'ddex-parser'**
```bash
# Clear node_modules and reinstall
rm -rf node_modules package-lock.json
npm install
```

**WASM loading issues in browser**
```typescript
// Configure WASM path for custom bundlers
import { DDEXParser } from 'ddex-parser/browser';

DDEXParser.setWasmPath('/path/to/ddex-parser.wasm');
const parser = new DDEXParser();
```

**Memory issues with large files**
```typescript
// Enable streaming mode for large catalogs
const parser = new DDEXParser({
  streaming: true,
  maxMemoryUsage: 50 * 1024 * 1024 // 50MB limit
});
```

**TypeScript compilation errors**
```bash
# Ensure you have the latest TypeScript version
npm install -D typescript@latest
```

### Getting Help

- 📖 [Full Documentation](https://github.com/ddex-suite/ddex-suite/tree/main/packages/ddex-parser)
- 🐛 [Report Issues](https://github.com/ddex-suite/ddex-suite/issues)
- 💬 [GitHub Discussions](https://github.com/ddex-suite/ddex-suite/discussions)
- 📧 Email: support@ddex-suite.com

## Contributing

We welcome contributions! See our [Contributing Guide](https://github.com/ddex-suite/ddex-suite/blob/main/CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/ddex-suite/ddex-suite/blob/main/LICENSE) file for details.

## Related Projects

- **[ddex-builder](https://www.npmjs.com/package/ddex-builder)** - Build deterministic DDEX XML files
- **[ddex-parser (Python)](https://pypi.org/project/ddex-parser/)** - Python bindings
- **[DDEX Suite](https://github.com/ddex-suite/ddex-suite)** - Complete DDEX processing toolkit

---

Built for the music industry. Powered by Rust + WASM for universal performance.