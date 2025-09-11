# Large File Processing

Handle multi-gigabyte DDEX files efficiently with streaming and memory optimization techniques.

## When to Use Streaming

Use streaming mode for files over 100MB or when memory is limited:

- **Files > 100MB**: Always use streaming
- **Files > 1GB**: Use streaming with additional optimizations  
- **Memory constraints**: Use streaming regardless of file size
- **Batch processing**: Process multiple files without memory accumulation

## Basic Streaming

Enable streaming mode with a simple option:

```typescript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();
const result = await parser.parse(xmlContent, { 
  streaming: true 
});
```

## Advanced Streaming Configuration

Fine-tune streaming for optimal performance:

```typescript
const result = await parser.parse(xmlContent, {
  streaming: true,
  bufferSize: 64 * 1024,           // 64KB chunks
  maxMemoryUsage: 500 * 1024 * 1024, // 500MB limit
  progressCallback: (progress) => {
    console.log(`Progress: ${progress.percentage}%`);
    console.log(`Processed: ${progress.bytesProcessed} bytes`);
  }
});
```

## Stream-Based Input

Process files directly from streams without loading into memory:

```typescript
import { createReadStream } from 'fs';

const fileStream = createReadStream('large-release.xml');
const result = await parser.stream(fileStream, {
  encoding: 'utf-8',
  highWaterMark: 64 * 1024  // 64KB buffer
});
```

## Chunked Processing

Process large files in smaller, manageable chunks:

```typescript
import { DDEXParser } from 'ddex-parser';
import { createReadStream } from 'fs';

class ChunkedProcessor {
  private parser = new DDEXParser();
  private results: any[] = [];

  async processLargeFile(filePath: string) {
    const stream = createReadStream(filePath, { 
      encoding: 'utf-8',
      highWaterMark: 1024 * 1024 // 1MB chunks
    });

    let buffer = '';
    let messageCount = 0;

    for await (const chunk of stream) {
      buffer += chunk;
      
      // Process complete messages
      const messages = this.extractCompleteMessages(buffer);
      for (const message of messages) {
        const result = await this.parser.parse(message, { streaming: true });
        this.results.push(result);
        messageCount++;
        
        if (messageCount % 100 === 0) {
          console.log(`Processed ${messageCount} messages`);
        }
      }
      
      // Keep incomplete message for next iteration
      buffer = this.getIncompleteMessage(buffer);
    }
    
    return this.results;
  }

  private extractCompleteMessages(buffer: string): string[] {
    // Implementation to extract complete DDEX messages
    const messages: string[] = [];
    const messageRegex = /<NewReleaseMessage[\s\S]*?<\/NewReleaseMessage>/g;
    let match;
    
    while ((match = messageRegex.exec(buffer)) !== null) {
      messages.push(match[0]);
    }
    
    return messages;
  }

  private getIncompleteMessage(buffer: string): string {
    // Return remaining incomplete message
    const lastComplete = buffer.lastIndexOf('</NewReleaseMessage>');
    return lastComplete === -1 ? buffer : buffer.substring(lastComplete + 21);
  }
}
```

## Memory-Efficient Iteration

Process resources, releases, and deals one at a time:

```typescript
const parser = new DDEXParser();

// Iterator-based processing
for await (const resource of parser.iterateResources(fileStream)) {
  console.log(`Processing resource: ${resource.title}`);
  
  // Process resource without loading entire file
  await processResource(resource);
  
  // Explicit cleanup to free memory
  resource.cleanup?.();
}
```

## Progress Monitoring

Track processing progress for long-running operations:

```typescript
class ProgressTracker {
  private startTime = Date.now();
  private processedBytes = 0;
  private totalBytes: number;

  constructor(totalBytes: number) {
    this.totalBytes = totalBytes;
  }

  onProgress(bytesProcessed: number) {
    this.processedBytes = bytesProcessed;
    const elapsed = Date.now() - this.startTime;
    const rate = this.processedBytes / elapsed * 1000; // bytes per second
    const eta = (this.totalBytes - this.processedBytes) / rate;
    
    console.log(`Progress: ${this.getPercentage()}%`);
    console.log(`Rate: ${this.formatBytes(rate)}/s`);
    console.log(`ETA: ${this.formatTime(eta)}`);
  }

  private getPercentage(): number {
    return Math.round((this.processedBytes / this.totalBytes) * 100);
  }

  private formatBytes(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB'];
    let i = 0;
    while (bytes >= 1024 && i < units.length - 1) {
      bytes /= 1024;
      i++;
    }
    return `${bytes.toFixed(1)} ${units[i]}`;
  }

  private formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
}

// Usage
const tracker = new ProgressTracker(fileSize);
const result = await parser.parse(xmlContent, {
  streaming: true,
  progressCallback: (progress) => tracker.onProgress(progress.bytesProcessed)
});
```

## Python Large File Processing

```python
from ddex_parser import DDEXParser
import pandas as pd

def process_large_file(file_path: str):
    parser = DDEXParser()
    
    # Streaming mode for large files
    result = parser.parse_file(file_path, streaming=True, chunk_size=1024*1024)
    
    # Process in batches to avoid memory issues
    batch_size = 1000
    for i in range(0, len(result.flat.releases), batch_size):
        batch = result.flat.releases[i:i+batch_size]
        df = pd.DataFrame([r.__dict__ for r in batch])
        
        # Process batch
        process_batch(df)
        
        # Clear memory
        del df, batch

def process_with_progress(file_path: str):
    parser = DDEXParser()
    
    def progress_callback(processed_bytes, total_bytes):
        percentage = (processed_bytes / total_bytes) * 100
        print(f"Progress: {percentage:.1f}%")
    
    result = parser.parse_file(
        file_path,
        streaming=True,
        progress_callback=progress_callback
    )
    
    return result
```

## Memory Optimization Tips

### 1. Process in Batches
```typescript
const batchSize = 100;
for (let i = 0; i < result.flat.releases.length; i += batchSize) {
  const batch = result.flat.releases.slice(i, i + batchSize);
  await processBatch(batch);
  
  // Force garbage collection hint
  if (global.gc) {
    global.gc();
  }
}
```

### 2. Use Selective Parsing
```typescript
// Only parse what you need
const result = await parser.parse(xmlContent, {
  streaming: true,
  includeResources: false,      // Skip resource details
  includeDeals: false,          // Skip deal information
  includeParties: false,        // Skip party information
  flattenOnly: true             // Only generate flat representation
});
```

### 3. Implement Backpressure
```typescript
class BackpressureProcessor {
  private processing = 0;
  private readonly maxConcurrent = 5;

  async processItem(item: any): Promise<void> {
    // Wait if too many concurrent operations
    while (this.processing >= this.maxConcurrent) {
      await this.sleep(10);
    }

    this.processing++;
    try {
      await this.doProcess(item);
    } finally {
      this.processing--;
    }
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
```

## Performance Benchmarks

Expected performance for streaming mode:

| File Size | Processing Time | Memory Usage | 
|-----------|----------------|--------------|
| 100MB     | 5-10 seconds   | ~50MB       |
| 500MB     | 20-30 seconds  | ~100MB      |
| 1GB       | 40-60 seconds  | ~150MB      |
| 5GB       | 3-5 minutes    | ~200MB      |

## Troubleshooting

**Out of Memory Errors**: Reduce batch size and enable streaming mode.

**Slow Processing**: Increase buffer size and use SSD storage.

**Connection Timeouts**: Implement retry logic for network streams.

**Incomplete Processing**: Ensure proper stream error handling and cleanup.

## Next Steps

- [Performance Optimization](./performance) - Advanced performance tuning
- [Error Handling](./error-handling) - Handle errors in streaming scenarios  
- [Memory Optimization](../advanced/memory) - Advanced memory management techniques