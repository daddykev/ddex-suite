# DDEX Builder - Node.js Bindings

![npm version](https://img.shields.io/npm/v/ddex-builder)
![license](https://img.shields.io/npm/l/ddex-builder)
![build status](https://img.shields.io/github/actions/workflow/status/daddykev/ddex-suite/ci.yml)

High-performance DDEX XML builder for Node.js with deterministic output and DB-C14N/1.0 canonicalization. Part of the [DDEX Suite](https://github.com/daddykev/ddex-suite) toolkit.

## Features

- ⚡ **High Performance**: Built in Rust with native Node.js bindings
- 🔒 **Deterministic Output**: Consistent XML generation with stable ordering
- 📋 **DB-C14N/1.0 Canonicalization**: Standards-compliant XML canonicalization
- 🌊 **Streaming Support**: Memory-efficient processing for large datasets
- ✅ **Built-in Validation**: Real-time validation with detailed error reporting
- 🎯 **Partner Presets**: Pre-configured settings for major platforms
- 📦 **TypeScript Support**: Full type definitions included
- 🔧 **Cross-Platform**: Supports Windows, macOS, and Linux

## Installation

```bash
npm install ddex-builder
```

### Requirements

- Node.js ≥ 14.0.0
- No additional dependencies required (native binaries included)

## Quick Start

### Basic Usage

```typescript
import { DdexBuilder } from 'ddex-builder';

const builder = new DdexBuilder();

// Add a release
const release = {
  releaseId: 'R001',
  releaseType: 'Album',
  title: 'My Album',
  artist: 'Artist Name',
  label: 'Record Label',
  catalogNumber: 'CAT001',
  upc: '123456789012',
  releaseDate: '2024-01-01',
  genre: 'Electronic',
  trackIds: ['T001', 'T002']
};

builder.addRelease(release);

// Add resources (tracks)
const track = {
  resourceId: 'T001',
  resourceType: 'SoundRecording',
  title: 'Track 1',
  artist: 'Artist Name',
  isrc: 'USRC17607839',
  duration: 'PT3M30S',
  trackNumber: 1
};

builder.addResource(track);

// Build DDEX XML
const xml = await builder.build();
console.log(xml);
```

### Streaming for Large Datasets

```typescript
import { StreamingDdexBuilder, MessageHeader } from 'ddex-builder';

const config = {
  maxBufferSize: 1024 * 1024, // 1MB buffer
  deterministic: true,
  validateDuringStream: true
};

const streamBuilder = new StreamingDdexBuilder(config);

// Set up progress tracking
streamBuilder.setProgressCallback((progress) => {
  console.log(`Progress: ${progress.estimatedCompletionPercent}%`);
});

// Start message
const header: MessageHeader = {
  messageSenderName: 'Your Company',
  messageRecipientName: 'Recipient',
  messageCreatedDateTime: new Date().toISOString()
};

streamBuilder.startMessage(header, '4.2');

// Add resources in streaming fashion
const resourceXml = streamBuilder.writeResource(
  'T001', 
  'Track Title',
  'Artist Name',
  'USRC17607839',
  'PT3M30S'
);

streamBuilder.finishResourcesStartReleases();

// Add releases
const releaseXml = streamBuilder.writeRelease(
  'R001',
  'Album Title', 
  'Artist Name',
  'Label Name',
  '123456789012',
  '2024-01-01',
  'Pop',
  ['T001']
);

const stats = streamBuilder.finishMessage();
const finalXml = streamBuilder.getXml();
```

## API Reference

### DdexBuilder Class

The main builder class for creating DDEX XML documents.

#### Constructor

```typescript
new DdexBuilder()
```

#### Methods

##### `addRelease(release: Release): void`

Add a release to the DDEX document.

```typescript
interface Release {
  releaseId: string;
  releaseType: string;
  title: string;
  artist: string;
  label?: string;
  catalogNumber?: string;
  upc?: string;
  releaseDate?: string;
  genre?: string;
  parentalWarning?: boolean;
  trackIds: string[];
  metadata?: Record<string, string>;
}
```

##### `addResource(resource: Resource): void`

Add a resource (track/sound recording) to the DDEX document.

```typescript
interface Resource {
  resourceId: string;
  resourceType: string;
  title: string;
  artist: string;
  isrc?: string;
  duration?: string;
  trackNumber?: number;
  volumeNumber?: number;
  metadata?: Record<string, string>;
}
```

##### `build(): Promise<string>`

Build the complete DDEX XML document.

Returns a Promise that resolves to the XML string.

##### `validate(): Promise<ValidationResult>`

Validate the current document structure.

```typescript
interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
}
```

##### `getStats(): BuilderStats`

Get statistics about the current document.

```typescript
interface BuilderStats {
  releasesCount: number;
  resourcesCount: number;
  totalBuildTimeMs: number;
  lastBuildSizeBytes: number;
  validationErrors: number;
  validationWarnings: number;
}
```

##### `reset(): void`

Reset the builder to empty state.

##### `getAvailablePresets(): string[]`

Get list of available partner presets.

##### `getPresetInfo(presetName: string): PresetInfo`

Get detailed information about a specific preset.

##### `applyPreset(presetName: string): void`

Apply a partner preset configuration.

### StreamingDdexBuilder Class

For memory-efficient processing of large datasets.

#### Constructor

```typescript
new StreamingDdexBuilder(config?: StreamingConfig)
```

```typescript
interface StreamingConfig {
  maxBufferSize: number;
  deterministic: boolean;
  validateDuringStream: boolean;
  progressCallbackFrequency: number;
}
```

#### Key Methods

##### `startMessage(header: MessageHeader, version: string): void`

Initialize the DDEX message with header information.

##### `writeResource(...): string`

Write a resource to the stream and return its XML.

##### `writeRelease(...): string`

Write a release to the stream and return its XML.

##### `finishMessage(): StreamingStats`

Complete the message and return final statistics.

### Utility Functions

#### `batchBuild(requests: string[]): Promise<string[]>`

Process multiple build requests in parallel.

#### `validateStructure(xml: string): Promise<ValidationResult>`

Validate XML structure without building.

## Partner Presets

The library includes presets for major music platforms:

```typescript
const builder = new DdexBuilder();

// Get available presets
const presets = builder.getAvailablePresets();
console.log(presets); // ['spotify', 'apple', 'youtube', 'generic']

// Apply Spotify preset
builder.applyPreset('spotify');

// Get preset requirements
const spotifyRules = builder.getPresetValidationRules('spotify');
```

## Error Handling

```typescript
try {
  const xml = await builder.build();
} catch (error) {
  if (error.name === 'ValidationError') {
    console.error('Validation failed:', error.details);
  } else if (error.name === 'BuildError') {
    console.error('Build failed:', error.message);
  } else {
    console.error('Unexpected error:', error);
  }
}
```

## Performance Guidelines

### Memory Usage

- Use `StreamingDdexBuilder` for datasets with >1000 releases
- Set appropriate `maxBufferSize` based on available memory
- Call `reset()` between different documents to free memory

### Build Performance

- Batch related operations when possible
- Use `batchBuild()` for multiple independent documents
- Enable validation only when needed (`validateDuringStream: false`)

### Typical Performance

- Single release build: ~5ms
- 100-release build: ~50ms
- 1000-release streaming build: ~500ms
- Memory usage: ~50KB base + ~1KB per release/resource

## TypeScript Support

Full TypeScript definitions are included:

```typescript
import { 
  DdexBuilder, 
  StreamingDdexBuilder,
  Release,
  Resource,
  ValidationResult,
  BuilderStats
} from 'ddex-builder';

// All interfaces are fully typed
const release: Release = {
  releaseId: 'R001',
  releaseType: 'Album',
  title: 'Album Title',
  artist: 'Artist Name',
  trackIds: []
};
```

## Examples

### Complete Album Example

```typescript
import { DdexBuilder } from 'ddex-builder';

async function buildAlbum() {
  const builder = new DdexBuilder();
  
  // Album release
  builder.addRelease({
    releaseId: 'ALB001',
    releaseType: 'Album',
    title: 'Greatest Hits',
    artist: 'The Band',
    label: 'Music Records',
    catalogNumber: 'MR2024001',
    upc: '123456789012',
    releaseDate: '2024-03-15',
    genre: 'Rock',
    trackIds: ['TRK001', 'TRK002', 'TRK003']
  });
  
  // Album tracks
  const tracks = [
    { id: 'TRK001', title: 'Hit Song 1', isrc: 'USRC17607001', duration: 'PT3M45S' },
    { id: 'TRK002', title: 'Hit Song 2', isrc: 'USRC17607002', duration: 'PT4M12S' },
    { id: 'TRK003', title: 'Hit Song 3', isrc: 'USRC17607003', duration: 'PT3M30S' }
  ];
  
  tracks.forEach((track, index) => {
    builder.addResource({
      resourceId: track.id,
      resourceType: 'SoundRecording',
      title: track.title,
      artist: 'The Band',
      isrc: track.isrc,
      duration: track.duration,
      trackNumber: index + 1,
      volumeNumber: 1
    });
  });
  
  // Validate before building
  const validation = await builder.validate();
  if (!validation.isValid) {
    console.error('Validation errors:', validation.errors);
    return;
  }
  
  // Build final XML
  const xml = await builder.build();
  return xml;
}
```

### Batch Processing Example

```typescript
import { batchBuild } from 'ddex-builder';

async function processBatch() {
  const requests = [
    JSON.stringify({
      releases: [{ releaseId: 'R001', title: 'Album 1', artist: 'Artist 1' }]
    }),
    JSON.stringify({
      releases: [{ releaseId: 'R002', title: 'Album 2', artist: 'Artist 2' }]
    })
  ];
  
  const results = await batchBuild(requests);
  results.forEach((xml, index) => {
    console.log(`Document ${index + 1} size:`, xml.length);
  });
}
```

## Troubleshooting

### Common Issues

**"Module not found" errors**: Ensure you're using Node.js ≥ 14.0.0 and the correct import syntax for your module system.

**Build failures**: Check validation results first - most build failures are due to missing required fields.

**Memory issues**: Use `StreamingDdexBuilder` for large datasets and call `reset()` between documents.

**Performance issues**: Disable validation during streaming if not needed, and use appropriate buffer sizes.

### Debug Mode

```typescript
const builder = new DdexBuilder();

// Enable debug logging (if available)
process.env.DDEX_BUILDER_DEBUG = '1';

const stats = builder.getStats();
console.log('Debug stats:', stats);
```

## Related Projects

- [ddex-parser](https://www.npmjs.com/package/ddex-parser) - Parse existing DDEX XML files
- [DDEX Suite](https://github.com/daddykev/ddex-suite) - Complete DDEX processing toolkit

## License

MIT License - see [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE) for details.

## Contributing

Contributions welcome! Please see the [main repository](https://github.com/daddykev/ddex-suite) for contribution guidelines.

## Support

- 📖 [Documentation](https://github.com/daddykev/ddex-suite/tree/main/docs)
- 🐛 [Issue Tracker](https://github.com/daddykev/ddex-suite/issues)
- 💬 [Discussions](https://github.com/daddykev/ddex-suite/discussions)