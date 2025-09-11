# Batch Processing

Process multiple DDEX documents efficiently using the builder's batch processing capabilities.

## Overview

Batch processing allows you to:
- Process hundreds of releases simultaneously
- Optimize memory usage for large datasets
- Implement parallel processing workflows
- Handle bulk operations efficiently

## Basic Batch Processing

### JavaScript/TypeScript

```typescript
import { DDEXBuilder } from 'ddex-builder';

const builder = new DDEXBuilder();
const releases = [
  { title: "Album 1", /* ... */ },
  { title: "Album 2", /* ... */ },
  // ... more releases
];

// Process all releases in a single operation
const batchResult = await builder.buildBatch(releases, {
  version: '4.3',
  batchSize: 50,        // Process 50 at a time
  parallel: true,       // Use parallel processing
  validateEach: true    // Validate each document
});

console.log(`Processed ${batchResult.success.length} releases`);
console.log(`Failed: ${batchResult.errors.length}`);
```

### Python

```python
from ddex_builder import DDEXBuilder
import pandas as pd

builder = DDEXBuilder()

# Process DataFrame with multiple releases
df = pd.read_csv('releases.csv')
results = builder.from_dataframe_batch(
    df, 
    group_by='release_id',
    batch_size=100,
    parallel=True
)

print(f"Processed {len(results.success)} releases")
for error in results.errors:
    print(f"Error in {error.id}: {error.message}")
```

### Rust

```rust
use ddex_builder::{DDEXBuilder, BuildRequest};

let builder = DDEXBuilder::new();
let requests: Vec<BuildRequest> = vec![/* ... */];

let results = builder.build_batch(
    requests,
    BatchOptions {
        batch_size: 50,
        parallel: true,
        fail_fast: false,
    }
)?;

println!("Processed {} documents", results.len());
```

## Streaming Batch Processing

For very large datasets, use streaming to minimize memory usage:

```typescript
import { DDEXBuilder, StreamingBatchProcessor } from 'ddex-builder';

const processor = new StreamingBatchProcessor({
  batchSize: 25,
  maxConcurrency: 4,
  onProgress: (processed, total) => {
    console.log(`Progress: ${processed}/${total}`);
  },
  onError: (error, item) => {
    console.error(`Failed to process ${item.id}:`, error);
  }
});

// Process from stream
await processor.processStream(inputStream, outputStream);
```

## Database Integration

Process releases directly from database queries:

```python
import asyncio
from ddex_builder import DDEXBuilder
from sqlalchemy import create_engine

async def process_from_database():
    builder = DDEXBuilder()
    engine = create_engine('postgresql://...')
    
    # Process in chunks to manage memory
    chunk_size = 100
    offset = 0
    
    while True:
        query = f"""
        SELECT * FROM releases 
        ORDER BY created_at 
        LIMIT {chunk_size} OFFSET {offset}
        """
        
        df = pd.read_sql(query, engine)
        if df.empty:
            break
            
        results = builder.from_dataframe_batch(df)
        
        # Save results
        for i, xml in enumerate(results.success):
            filename = f"release_{offset + i}.xml"
            with open(f"output/{filename}", 'w') as f:
                f.write(xml)
        
        offset += chunk_size
```

## Error Handling

Implement robust error handling for batch operations:

```typescript
const results = await builder.buildBatch(data, {
  continueOnError: true,  // Don't stop on individual failures
  maxRetries: 3,          // Retry failed items
  retryDelay: 1000,       // Wait between retries
});

// Process results
for (const success of results.success) {
  await saveToFile(success.id, success.xml);
}

// Handle errors
for (const error of results.errors) {
  logger.error(`Failed to process ${error.id}:`, error.message);
  
  // Optionally retry with different settings
  if (error.retryable) {
    await retryLater(error.data);
  }
}
```

## Performance Optimization

### Memory Management

```rust
use ddex_builder::BatchOptions;

let options = BatchOptions {
    batch_size: 50,           // Balance memory vs parallelism
    memory_limit: 1024 * 1024 * 100, // 100MB limit
    gc_frequency: 10,         // Clean up every 10 batches
    ..Default::default()
};

let results = builder.build_batch_with_options(requests, options)?;
```

### Parallel Processing

```python
from ddex_builder import DDEXBuilder
import concurrent.futures

def build_release(release_data):
    builder = DDEXBuilder()  # Thread-local instance
    return builder.build(release_data)

# Process with thread pool
with concurrent.futures.ThreadPoolExecutor(max_workers=8) as executor:
    futures = [executor.submit(build_release, release) 
              for release in releases]
    
    results = [future.result() for future in futures]
```

## Monitoring and Metrics

Track batch processing performance:

```typescript
const processor = new BatchProcessor({
  onBatchComplete: (batchStats) => {
    console.log({
      batchNumber: batchStats.batchNumber,
      itemsProcessed: batchStats.itemsProcessed,
      processingTime: batchStats.processingTimeMs,
      throughput: batchStats.itemsPerSecond,
      memoryUsage: batchStats.memoryUsageMB
    });
  },
  
  onComplete: (totalStats) => {
    console.log({
      totalItems: totalStats.totalItems,
      totalTime: totalStats.totalTimeMs,
      averageThroughput: totalStats.averageItemsPerSecond,
      peakMemory: totalStats.peakMemoryMB,
      errorRate: totalStats.errorRate
    });
  }
});
```

## Best Practices

1. **Choose appropriate batch sizes**: Start with 25-50 items per batch
2. **Monitor memory usage**: Implement memory limits to prevent OOM
3. **Handle errors gracefully**: Use `continueOnError` for resilient processing
4. **Use streaming for large datasets**: Avoid loading everything into memory
5. **Implement progress tracking**: Provide feedback for long-running operations
6. **Test with representative data**: Validate performance with real-world datasets
7. **Consider database connection pooling**: For database-driven workflows

## Common Patterns

### Producer-Consumer with Queue

```python
import asyncio
from asyncio import Queue

async def producer(queue: Queue, data_source):
    async for batch in data_source:
        await queue.put(batch)
    await queue.put(None)  # Signal completion

async def consumer(queue: Queue, builder: DDEXBuilder):
    while True:
        batch = await queue.get()
        if batch is None:
            break
        
        results = builder.build_batch(batch)
        await process_results(results)

# Run producer and consumer concurrently
queue = Queue(maxsize=5)
await asyncio.gather(
    producer(queue, data_source),
    consumer(queue, builder)
)
```