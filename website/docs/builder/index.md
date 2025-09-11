---
sidebar_position: 4
---

# DDEX Builder

The DDEX Builder generates deterministic, compliant DDEX XML from structured data. It ensures perfect round-trip fidelity and produces byte-identical output for the same input data.

## Key Features

- **Deterministic**: Same input always produces identical XML output
- **Compliant**: Validates against DDEX schemas before generation
- **Fast**: Build typical releases in &lt;15ms
- **Canonical**: Uses DB-C14N/1.0 canonicalization for reproducible output
- **Configurable**: Support for partner-specific requirements

## Basic Usage

### Node.js / TypeScript

```typescript
import { DDEXBuilder } from 'ddex-builder';

const builder = new DDEXBuilder();

// Build from parsed data
const xml = await builder.build(parseResult.toBuildRequest());

// Or build from scratch
const xml = await builder.build({
  version: '4.3',
  messageHeader: {
    messageId: 'MSG_001',
    sentOnBehalfOf: 'SENDER_PARTY',
    messageCreatedDateTime: new Date().toISOString(),
  },
  releases: [{
    title: 'My Great Album',
    artist: 'Amazing Artist',
    releaseDate: '2024-01-01',
    territories: ['US', 'CA', 'GB'],
  }],
  soundRecordings: [{
    title: 'Track 1',
    artist: 'Amazing Artist',
    duration: 'PT3M45S',
  }],
});
```

### Python

```python
from ddex_builder import DDEXBuilder

builder = DDEXBuilder()

# Build from parsed data
xml = builder.build(parse_result.to_build_request())

# Or build from DataFrame
xml = builder.from_dataframe(df, version='4.3')
```

## Configuration Options

### Builder Settings

```typescript
const builder = new DDEXBuilder({
  // Output format
  canonicalization: 'db-c14n-1.0', // 'db-c14n-1.0' | 'none'
  
  // Validation level
  validation: 'strict', // 'strict' | 'permissive' | 'none'
  
  // Partner presets
  preset: 'spotify', // 'spotify' | 'youtube' | 'apple' | 'generic'
  
  // Pretty printing
  prettyPrint: false,
  
  // Encoding
  encoding: 'UTF-8',
});
```

### Partner Presets

Different partners have specific requirements. Use presets for compliance:

```typescript
// Spotify preset
const spotifyBuilder = new DDEXBuilder({ preset: 'spotify' });

// YouTube preset  
const youtubeBuilder = new DDEXBuilder({ preset: 'youtube' });

// Apple Music preset
const appleBuilder = new DDEXBuilder({ preset: 'apple' });
```

## Building from Scratch

Create DDEX messages programmatically:

```typescript
const buildRequest = {
  version: '4.3',
  messageHeader: {
    messageId: `MSG_${Date.now()}`,
    sentOnBehalfOf: 'YOUR_PARTY_ID',
    messageCreatedDateTime: new Date().toISOString(),
    messageControlType: 'LiveMessage',
  },
  releases: [{
    releaseId: 'REL_001',
    title: 'My Album',
    artist: 'Artist Name',
    releaseDate: '2024-01-01',
    releaseType: 'Album',
    territories: ['Worldwide'],
    genres: ['Rock', 'Alternative'],
  }],
  soundRecordings: [{
    soundRecordingId: 'SR_001',
    title: 'Track Title',
    artist: 'Artist Name',
    duration: 'PT3M45S',
    languageOfPerformance: 'en',
  }],
  deals: [{
    dealId: 'DEAL_001',
    territories: ['US', 'CA'],
    useTypes: ['Stream', 'PermanentDownload'],
    commercialModelType: 'Subscription',
    dealStartDate: '2024-01-01',
  }],
};

const xml = await builder.build(buildRequest);
```

## Validation and Preflight

The builder validates data before generation:

```typescript
try {
  // Validate without building
  const validation = await builder.validate(buildRequest);
  
  if (validation.isValid) {
    const xml = await builder.build(buildRequest);
  } else {
    console.log('Validation errors:', validation.errors);
  }
} catch (error) {
  if (error instanceof DDEXBuildError) {
    console.log('Build error:', error.message);
    console.log('Field:', error.field);
    console.log('Value:', error.value);
  }
}
```

### Common Validation Errors

- **`RequiredFieldError`**: Missing required DDEX field
- **`InvalidValueError`**: Field value doesn't match DDEX constraints
- **`ReferenceError`**: Invalid reference between DDEX elements
- **`TerritoryError`**: Invalid territory codes

## Deterministic Output

The builder ensures reproducible output:

```typescript
// These will generate identical XML
const xml1 = await builder.build(data);
const xml2 = await builder.build(data);

console.log(xml1 === xml2); // true

// Content-based IDs for determinism
const buildRequest = {
  // ... your data
  generateDeterministicIds: true, // default: true
};
```

## Streaming for Large Files

For large datasets, use the streaming builder:

```typescript
import { DDEXStreamBuilder } from 'ddex-builder';

const streamBuilder = new DDEXStreamBuilder();

// Start the document
await streamBuilder.start({
  version: '4.3',
  messageHeader: { /* ... */ },
});

// Add releases incrementally
for (const release of largeReleaseList) {
  await streamBuilder.addRelease(release);
}

// Add sound recordings
for (const recording of largeRecordingList) {
  await streamBuilder.addSoundRecording(recording);
}

// Finalize and get XML
const xml = await streamBuilder.finish();
```

## DataFrame Support (Python)

Build DDEX from pandas DataFrames:

```python
import pandas as pd
from ddex_builder import DDEXBuilder

# Create DataFrames
releases_df = pd.DataFrame([{
    'title': 'My Album',
    'artist': 'Artist Name',
    'release_date': '2024-01-01',
    'territories': ['US', 'CA'],
}])

recordings_df = pd.DataFrame([{
    'title': 'Track 1',
    'artist': 'Artist Name', 
    'duration': 'PT3M45S',
}])

# Build from DataFrames
builder = DDEXBuilder()
xml = builder.from_dataframes({
    'releases': releases_df,
    'sound_recordings': recordings_df,
}, version='4.3')
```

## CLI Usage

The builder provides command-line tools:

```bash
# Build from JSON
ddex-builder build data.json --output output.xml

# Validate build request
ddex-builder validate data.json

# Use partner preset
ddex-builder build data.json --preset spotify

# Pretty print output
ddex-builder build data.json --pretty --output formatted.xml
```

## Performance Benchmarks

| Operation | Typical Release | Large Release | Streaming |
|-----------|----------------|---------------|-----------|
| Build | &lt;15ms | &lt;100ms | &lt;1s |
| Validate | &lt;5ms | &lt;50ms | &lt;500ms |
| Memory | &lt;10MB | &lt;50MB | &lt;20MB |

## Advanced Features

### Custom Templates

Create reusable templates:

```typescript
const template = builder.createTemplate({
  version: '4.3',
  messageHeader: {
    sentOnBehalfOf: 'YOUR_PARTY_ID',
    messageControlType: 'LiveMessage',
  },
  defaults: {
    territories: ['Worldwide'],
    commercialModelType: 'Subscription',
  },
});

// Use template
const xml = await template.build({
  releases: [{ /* ... */ }],
  soundRecordings: [{ /* ... */ }],
});
```

### Batch Processing

Process multiple releases efficiently:

```typescript
const batchBuilder = new DDEXBatchBuilder();

const results = await batchBuilder.buildMany([
  buildRequest1,
  buildRequest2,
  buildRequest3,
], {
  preset: 'spotify',
  parallel: true,
});
```

## Next Steps

- **[Parser Documentation](../parser/)** - Learn to parse DDEX XML
- **[API Reference](../api/)** - Complete builder API
- **[Examples](../examples/)** - Real-world builder examples
- **[Guides](../guides/)** - How-to guides for common tasks