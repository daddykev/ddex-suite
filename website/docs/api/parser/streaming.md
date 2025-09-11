# Streaming API

The streaming API enables memory-efficient processing of large DDEX XML files by parsing content incrementally rather than loading everything into memory.

## Core Streaming Classes

### StreamingParser

Main class for streaming DDEX content:

```typescript
class StreamingParser {
  constructor(options?: StreamingOptions);
  parseStream(source: ReadableStream | NodeJS.ReadableStream): AsyncIterableIterator<StreamedRelease>;
  getProgress(): StreamProgress;
  pause(): void;
  resume(): void;
  abort(): void;
}
```

### StreamingOptions

Configuration for streaming operations:

```typescript
interface StreamingOptions {
  chunkSize?: number;        // Size of chunks to process (default: 8192 bytes)
  maxMemory?: number;        // Maximum memory usage (default: 64MB)
  bufferSize?: number;       // Internal buffer size (default: 16384 bytes)
  timeout?: number;          // Timeout per chunk (default: 5000ms)
  skipValidation?: boolean;  // Skip XML schema validation for speed
  preserveWhitespace?: boolean; // Preserve whitespace in text content
  encoding?: string;         // Input encoding (default: 'utf-8')
}
```

## Stream Processing

### Streaming Parse Results

```typescript
interface StreamedRelease {
  releaseId: string;
  title: string;
  releaseDate?: string;
  mainArtist?: string;
  tracks: StreamedTrack[];
  metadata: ReleaseMetadata;
  xmlFragment?: string;      // Raw XML if preserveRaw: true
}

interface StreamedTrack {
  resourceId: string;
  title: string;
  isrc?: string;
  duration?: string;
  sequenceNumber?: number;
  metadata: TrackMetadata;
}
```

### Progress Tracking

```typescript
interface StreamProgress {
  bytesProcessed: number;
  bytesTotal?: number;
  releasesProcessed: number;
  tracksProcessed: number;
  elapsedTimeMs: number;
  estimatedRemainingMs?: number;
  throughputBytesPerSec: number;
  memoryUsage: MemoryUsage;
}

interface MemoryUsage {
  heapUsed: number;
  heapTotal: number;
  external: number;
  rss: number;
}
```

## Usage Examples

### Basic Streaming

```typescript
import { StreamingParser } from 'ddex-parser';
import { createReadStream } from 'fs';

const parser = new StreamingParser({
  chunkSize: 16384,      // 16KB chunks
  maxMemory: 128 * 1024 * 1024, // 128MB limit
  skipValidation: false
});

const fileStream = createReadStream('large-catalog.xml');
const releaseStream = parser.parseStream(fileStream);

// Process releases as they're parsed
for await (const release of releaseStream) {
  console.log(`Processing: ${release.title} (${release.tracks.length} tracks)`);
  
  // Process individual release
  await processRelease(release);
  
  // Check progress periodically
  const progress = parser.getProgress();
  if (progress.releasesProcessed % 100 === 0) {
    const completion = progress.bytesTotal 
      ? (progress.bytesProcessed / progress.bytesTotal * 100).toFixed(1)
      : 'unknown';
    console.log(`Progress: ${progress.releasesProcessed} releases, ${completion}% complete`);
  }
}

console.log('Streaming parse complete!');
```

### Memory-Constrained Streaming

```typescript
const constrainedParser = new StreamingParser({
  maxMemory: 32 * 1024 * 1024,  // 32MB limit
  chunkSize: 4096,              // Smaller chunks
  bufferSize: 8192              // Smaller buffer
});

const stream = createReadStream('massive-catalog.xml');
const releases = constrainedParser.parseStream(stream);

let processedCount = 0;
const batchSize = 50;

for await (const release of releases) {
  await processRelease(release);
  processedCount++;
  
  // Force garbage collection periodically
  if (processedCount % batchSize === 0) {
    if (global.gc) {
      global.gc();
    }
    
    const memory = parser.getProgress().memoryUsage;
    console.log(`Memory usage: ${Math.round(memory.heapUsed / 1024 / 1024)}MB`);
    
    // Pause if memory usage is too high
    if (memory.heapUsed > 30 * 1024 * 1024) { // 30MB threshold
      console.log('Pausing due to high memory usage...');
      parser.pause();
      
      // Wait for memory to be freed
      await new Promise(resolve => setTimeout(resolve, 1000));
      parser.resume();
    }
  }
}
```

### Real-time Progress Monitoring

```typescript
const parser = new StreamingParser({
  chunkSize: 8192,
  maxMemory: 100 * 1024 * 1024
});

// Set up progress monitoring
const progressInterval = setInterval(() => {
  const progress = parser.getProgress();
  
  const memoryMB = Math.round(progress.memoryUsage.heapUsed / 1024 / 1024);
  const throughputMBps = (progress.throughputBytesPerSec / 1024 / 1024).toFixed(2);
  const eta = progress.estimatedRemainingMs 
    ? new Date(Date.now() + progress.estimatedRemainingMs).toLocaleTimeString()
    : 'unknown';
  
  console.log(`📊 Progress: ${progress.releasesProcessed} releases | ${memoryMB}MB | ${throughputMBps} MB/s | ETA: ${eta}`);
}, 5000); // Update every 5 seconds

const fileStream = createReadStream('catalog.xml');
const releases = parser.parseStream(fileStream);

try {
  for await (const release of releases) {
    await processRelease(release);
  }
} finally {
  clearInterval(progressInterval);
  console.log('✅ Streaming complete!');
}
```

## Advanced Streaming Features

### Selective Processing

Process only specific types of content:

```typescript
interface SelectiveStreamingOptions extends StreamingOptions {
  filter?: {
    releaseTypes?: string[];      // Only process specific release types
    territories?: string[];       // Only process specific territories  
    dateRange?: {                // Only process releases in date range
      start: string;
      end: string;
    };
    artistFilter?: string[];     // Only process specific artists
    labelFilter?: string[];      // Only process specific labels
  };
}

const selectiveParser = new StreamingParser({
  chunkSize: 8192,
  filter: {
    releaseTypes: ['Album', 'EP'],
    territories: ['US', 'CA'],
    dateRange: {
      start: '2024-01-01',
      end: '2024-12-31'
    }
  }
} as SelectiveStreamingOptions);

// Only albums and EPs for US/CA released in 2024 will be yielded
for await (const release of selectiveParser.parseStream(stream)) {
  // Process filtered release
  console.log(`${release.title} - matches filter criteria`);
}
```

### Parallel Processing

Process multiple streams concurrently:

```typescript
async function processMultipleFiles(filePaths: string[]): Promise<void> {
  const concurrency = 3; // Process 3 files at once
  const semaphore = new Array(concurrency).fill(null);
  
  const processFile = async (filePath: string): Promise<void> => {
    const parser = new StreamingParser({
      chunkSize: 8192,
      maxMemory: 50 * 1024 * 1024 // 50MB per parser
    });
    
    const stream = createReadStream(filePath);
    const releases = parser.parseStream(stream);
    
    console.log(`📁 Starting: ${filePath}`);
    let count = 0;
    
    for await (const release of releases) {
      await processRelease(release);
      count++;
    }
    
    console.log(`✅ Completed: ${filePath} (${count} releases)`);
  };
  
  // Process files with concurrency limit
  await Promise.all(
    filePaths.map((filePath, index) =>
      semaphore[index % concurrency] = processFile(filePath)
    )
  );
}

// Usage
await processMultipleFiles([
  'catalog-2023.xml',
  'catalog-2024.xml', 
  'catalog-updates.xml'
]);
```

### Stream Transformation

Transform releases during streaming:

```typescript
import { Transform } from 'stream';

class DDEXTransformStream extends Transform {
  constructor(private transformer: (release: StreamedRelease) => StreamedRelease) {
    super({ objectMode: true });
  }
  
  _transform(release: StreamedRelease, encoding: string, callback: Function) {
    try {
      const transformed = this.transformer(release);
      this.push(transformed);
      callback();
    } catch (error) {
      callback(error);
    }
  }
}

// Usage
const parser = new StreamingParser();
const fileStream = createReadStream('catalog.xml');
const releaseStream = parser.parseStream(fileStream);

const transformer = new DDEXTransformStream((release) => {
  // Normalize artist names
  release.mainArtist = release.mainArtist?.trim().toUpperCase();
  
  // Add computed fields
  release.metadata.totalDurationMs = release.tracks.reduce((total, track) => {
    const duration = parseDuration(track.duration || 'PT0S');
    return total + duration * 1000;
  }, 0);
  
  return release;
});

// Chain streams: File -> Parser -> Transform -> Output
fileStream
  .pipe(releaseStream)
  .pipe(transformer)
  .on('data', (release) => {
    console.log(`Processed: ${release.title} (${release.metadata.totalDurationMs}ms)`);
  })
  .on('end', () => {
    console.log('Stream processing complete!');
  });
```

## Error Handling in Streams

### Resilient Streaming

Handle errors gracefully during streaming:

```typescript
const resilientParser = new StreamingParser({
  chunkSize: 8192,
  continueOnError: true,  // Continue processing after errors
  maxErrors: 10           // Stop after 10 errors
});

const stream = createReadStream('catalog.xml');
const releases = resilientParser.parseStream(stream);

let errorCount = 0;
const errors: Error[] = [];

try {
  for await (const release of releases) {
    try {
      await processRelease(release);
    } catch (error) {
      errorCount++;
      errors.push(error);
      
      console.warn(`⚠️  Error processing release ${release.releaseId}: ${error.message}`);
      
      // Continue with next release
      if (errorCount < 10) {
        continue;
      } else {
        console.error('❌ Too many errors, stopping');
        break;
      }
    }
  }
} catch (streamError) {
  console.error('💥 Fatal streaming error:', streamError.message);
} finally {
  console.log(`📊 Summary: ${errorCount} errors encountered`);
  if (errors.length > 0) {
    console.log('Errors:', errors.map(e => e.message));
  }
}
```

## Performance Optimization

### Memory Management

```typescript
const optimizedParser = new StreamingParser({
  chunkSize: 16384,                    // Larger chunks for better throughput
  maxMemory: 100 * 1024 * 1024,      // 100MB limit
  bufferSize: 32768,                   // Larger buffer
  gcThreshold: 0.8,                    // Trigger GC at 80% memory usage
  poolingEnabled: true                 // Pool objects to reduce GC pressure
});

// Monitor memory usage
const memoryMonitor = setInterval(() => {
  const progress = optimizedParser.getProgress();
  const memoryPercent = (progress.memoryUsage.heapUsed / (100 * 1024 * 1024) * 100).toFixed(1);
  
  if (parseFloat(memoryPercent) > 80) {
    console.warn(`⚠️  High memory usage: ${memoryPercent}%`);
  }
}, 1000);

// Process with monitoring
try {
  for await (const release of optimizedParser.parseStream(stream)) {
    await processRelease(release);
  }
} finally {
  clearInterval(memoryMonitor);
}
```

## See Also

- [Parser API](./index.md) - Main parser documentation
- [Memory Management](./memory-management) - Memory optimization strategies  
- [Performance Guide](../../guides/performance-tuning) - Performance optimization
- [Large Files Guide](../../guides/streaming-large-files) - Working with large DDEX files