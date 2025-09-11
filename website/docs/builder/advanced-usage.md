# Advanced Usage

This guide covers advanced DDEX Builder features including streaming, custom validation, batch processing, and performance optimization.

## Streaming Large Documents

For building large DDEX documents, use streaming to manage memory efficiently:

### TypeScript Streaming

```typescript
import { DDEXBuilder } from 'ddex-builder';

const builder = new DDEXBuilder({
  streaming: true,
  bufferSize: 8192  // 8KB buffer
});

// Stream large dataset
const largeRelease = {
  releaseId: 'R123456789',
  resources: {
    soundRecordings: generateLargeTrackList(10000) // 10k tracks
  }
};

const stream = await builder.buildStream(largeRelease);
const output = fs.createWriteStream('large-release.xml');
stream.pipe(output);
```

### Python Streaming

```python
from ddex_builder import DDEXBuilder
import io

builder = DDEXBuilder(streaming=True)

# Generate large release data
large_data = generate_bulk_release_data(track_count=5000)

# Stream to file-like object  
output = io.BytesIO()
builder.build_stream(large_data, output)

# Write to file
with open('bulk-release.xml', 'wb') as f:
    f.write(output.getvalue())
```

## Custom Validation Rules

Extend the default validation with custom business rules:

### TypeScript Custom Validation

```typescript
import { DDEXBuilder, ValidationRule, ValidationError } from 'ddex-builder';

// Custom validation rule
const customISRCRule: ValidationRule = {
  name: 'customISRCFormat',
  validate: (data) => {
    const errors: ValidationError[] = [];
    
    data.resources?.soundRecordings?.forEach((recording, index) => {
      const isrc = recording.soundRecordingId?.isrc;
      if (isrc && !isrc.match(/^[A-Z]{2}[A-Z0-9]{3}\d{7}$/)) {
        errors.push({
          path: `resources.soundRecordings[${index}].soundRecordingId.isrc`,
          message: 'ISRC format invalid: must be CC-XXX-YY-NNNNN',
          code: 'INVALID_ISRC_FORMAT'
        });
      }
    });
    
    return errors;
  }
};

// Use with builder
const builder = new DDEXBuilder({
  validation: {
    customRules: [customISRCRule],
    strictMode: true
  }
});
```

### Python Custom Validation

```python
from ddex_builder import DDEXBuilder, ValidationRule
import re

def validate_label_codes(data):
    """Custom validation for label codes"""
    errors = []
    
    if 'parties' in data:
        for i, party in enumerate(data['parties']):
            if party.get('roles') and 'RecordLabel' in [r.get('role') for r in party['roles']]:
                # Check for valid label code
                identifiers = party.get('identifiers', [])
                has_valid_code = any(
                    id.get('namespace') == 'DDEX:LabelCode' and 
                    re.match(r'^LC\d{5}$', id.get('value', ''))
                    for id in identifiers
                )
                
                if not has_valid_code:
                    errors.append({
                        'path': f'parties[{i}].identifiers',
                        'message': 'Record label must have valid LC code (LC#####)',
                        'code': 'MISSING_LABEL_CODE'
                    })
    
    return errors

# Register custom rule
custom_rule = ValidationRule('labelCodes', validate_label_codes)
builder = DDEXBuilder(custom_rules=[custom_rule])
```

## Partner Presets and Customization

Use and customize presets for specific DSP requirements:

### Preset Usage

```typescript
import { DDEXBuilder, Presets } from 'ddex-builder';

// Use Spotify preset
const builder = new DDEXBuilder({
  preset: Presets.SPOTIFY,
  version: '4.3'
});

// Customize existing preset
const customSpotifyPreset = {
  ...Presets.SPOTIFY,
  validation: {
    ...Presets.SPOTIFY.validation,
    requirePreviewClips: true,
    maxTrackCount: 500
  },
  technicalRequirements: {
    ...Presets.SPOTIFY.technicalRequirements,
    audioFormats: ['FLAC', 'MP3'],
    minBitrate: 320000
  }
};

const customBuilder = new DDEXBuilder({
  preset: customSpotifyPreset
});
```

### Creating Custom Presets

```typescript
const customDSPPreset = {
  name: 'CustomDSP',
  version: '4.3',
  validation: {
    requireArtworkMinDimensions: { width: 1400, height: 1400 },
    requirePreviewClips: false,
    maxReleaseCount: 100,
    strictTerritoryValidation: true
  },
  technicalRequirements: {
    audioFormats: ['FLAC', 'AAC'],
    minSamplingRate: 44100,
    requiredChannelConfig: ['Stereo', 'Mono']
  },
  outputOptions: {
    canonicalization: 'C14N-1.0',
    encoding: 'UTF-8',
    indentation: 2
  }
};
```

## Batch Processing

Process multiple releases efficiently:

### TypeScript Batch Processing

```typescript
import { DDEXBuilder, BatchProcessor } from 'ddex-builder';

const processor = new BatchProcessor({
  concurrency: 4,          // Process 4 releases at once
  timeout: 30000,          // 30 second timeout per release  
  retries: 2,              // Retry failed builds
  progressCallback: (progress) => {
    console.log(`Progress: ${progress.completed}/${progress.total}`);
  }
});

// Process multiple releases
const releases = [
  { releaseId: 'R001', /* ... */ },
  { releaseId: 'R002', /* ... */ },
  { releaseId: 'R003', /* ... */ }
];

const results = await processor.processBatch(releases, {
  outputDir: './output',
  filenameTemplate: '{releaseId}_{version}.xml'
});

// Handle results
results.forEach((result) => {
  if (result.success) {
    console.log(`✅ ${result.releaseId}: ${result.outputPath}`);
  } else {
    console.error(`❌ ${result.releaseId}: ${result.error.message}`);
  }
});
```

### Python Batch Processing

```python
from ddex_builder import DDEXBuilder, BatchProcessor
import asyncio

async def process_releases():
    processor = BatchProcessor(
        concurrency=4,
        timeout=30,
        max_retries=2
    )
    
    # Load release data
    releases = load_release_data_from_db()
    
    # Process batch
    results = await processor.process_batch(
        releases,
        output_dir='./output',
        filename_template='{release_id}_{timestamp}.xml'
    )
    
    # Summary
    successful = sum(1 for r in results if r.success)
    print(f"Processed {successful}/{len(results)} releases successfully")
    
    return results

# Run batch processing
results = asyncio.run(process_releases())
```

## Performance Optimization

### Memory Management

```typescript
import { DDEXBuilder, MemoryManager } from 'ddex-builder';

const builder = new DDEXBuilder({
  memoryManagement: {
    maxMemoryUsage: 512 * 1024 * 1024,  // 512MB limit
    garbageCollectionThreshold: 0.8,     // GC at 80% usage
    chunkedProcessing: true,             // Process in chunks
    chunkSize: 1000                      // 1000 items per chunk
  }
});

// Monitor memory usage
builder.on('memoryWarning', (usage) => {
  console.warn(`High memory usage: ${usage.percentage}%`);
});
```

### Caching Strategies

```typescript
import { DDEXBuilder, CacheManager } from 'ddex-builder';

const cache = new CacheManager({
  strategy: 'LRU',           // Least Recently Used
  maxSize: 100,              // Cache 100 items
  ttl: 3600000              // 1 hour TTL
});

const builder = new DDEXBuilder({
  cache: cache,
  cacheableOperations: [
    'validation',
    'canonicalization', 
    'templateProcessing'
  ]
});

// Pre-warm cache with common data
await cache.preload([
  'territories',
  'party-roles',
  'use-types'
]);
```

## Error Recovery

Handle and recover from build errors:

### Graceful Error Handling

```typescript
import { DDEXBuilder, ErrorRecovery } from 'ddex-builder';

const builder = new DDEXBuilder({
  errorRecovery: {
    strategy: 'partial',        // 'strict', 'partial', or 'lenient'
    skipInvalidItems: true,     // Skip bad items, continue processing
    reportSkipped: true,        // Report what was skipped
    fallbackValues: {           // Default values for missing data
      releaseDate: new Date().toISOString().split('T')[0],
      partyRole: 'Unknown'
    }
  }
});

try {
  const result = await builder.build(releaseData);
  
  if (result.warnings.length > 0) {
    console.warn('Build completed with warnings:');
    result.warnings.forEach(warning => {
      console.warn(`- ${warning.message} at ${warning.path}`);
    });
  }
  
  return result.xml;
  
} catch (error) {
  if (error.recoverable) {
    // Attempt partial recovery
    const partialResult = await builder.buildPartial(releaseData, {
      includeErrorReport: true
    });
    
    console.log(`Partial build completed: ${partialResult.itemsProcessed}/${partialResult.totalItems}`);
    return partialResult;
  }
  
  throw error; // Unrecoverable error
}
```

## Plugin System

Extend builder functionality with plugins:

### Custom Plugin Development

```typescript
import { DDEXBuilder, Plugin } from 'ddex-builder';

const analyticsPlugin: Plugin = {
  name: 'analytics',
  version: '1.0.0',
  
  hooks: {
    beforeBuild: async (data) => {
      // Track build attempts
      analytics.track('ddex_build_started', {
        releaseId: data.releaseId,
        trackCount: data.resources?.soundRecordings?.length || 0
      });
    },
    
    afterBuild: async (result) => {
      // Track successful builds
      analytics.track('ddex_build_completed', {
        releaseId: result.releaseId,
        xmlSize: result.xml.length,
        buildTime: result.metrics.buildTime
      });
    },
    
    onError: async (error, data) => {
      // Track build failures
      analytics.track('ddex_build_failed', {
        releaseId: data.releaseId,
        errorCode: error.code,
        errorMessage: error.message
      });
    }
  }
};

// Use plugin
const builder = new DDEXBuilder({
  plugins: [analyticsPlugin]
});
```

## CLI Integration

Use builder programmatically with CLI features:

```typescript
import { DDEXBuilder, CLI } from 'ddex-builder';

// Programmatic CLI usage
const cli = new CLI({
  builder: new DDEXBuilder({
    preset: Presets.UNIVERSAL,
    validation: { strictMode: false }
  }),
  
  defaultOptions: {
    outputFormat: 'pretty',
    validateOnly: false,
    verbose: true
  }
});

// Run CLI commands programmatically
const result = await cli.run([
  'build',
  'input.json', 
  'output.xml',
  '--preset', 'spotify',
  '--validate-only'
]);

console.log(result.success ? 'Build successful' : 'Build failed');
```

## See Also

- [API Reference](./api-reference) - Complete API documentation
- [TypeScript Guide](../api/builder/typescript) - TypeScript-specific features
- [Python Guide](../api/builder/python) - Python-specific features
- [Performance Guide](../guides/performance-tuning) - Performance optimization