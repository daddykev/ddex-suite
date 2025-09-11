# Performance Optimization

Optimize DDEX Parser performance for your specific use case and requirements.

## Performance Benchmarks

Expected performance baselines for different scenarios:

| File Size | Standard Mode | Streaming Mode | Memory Usage |
|-----------|---------------|----------------|--------------|
| 10KB      | &lt;5ms       | 8-12ms         | &lt;1MB      |
| 100KB     | 20-40ms       | 30-50ms        | 2-5MB         |
| 1MB       | 100-200ms     | 150-250ms      | 10-20MB       |
| 10MB      | 1-2s          | 800ms-1.5s     | 50-80MB       |
| 100MB     | 8-15s         | 5-10s          | 100-150MB     |

## Choosing the Right Mode

### Standard Mode (Default)
Best for files under 10MB with sufficient memory:

```typescript
const result = await parser.parse(xmlContent);
// Fastest for small to medium files
```

### Streaming Mode  
Best for large files or memory-constrained environments:

```typescript
const result = await parser.parse(xmlContent, { 
  streaming: true 
});
// Lower memory usage, consistent performance
```

## Parser Configuration Optimization

### High-Performance Configuration

```typescript
const parser = new DDEXParser({
  // Pre-allocate buffers for better performance
  bufferSize: 128 * 1024,        // 128KB buffer
  
  // Disable unnecessary features
  validateOnParse: false,        // Skip validation during parsing
  preserveWhitespace: false,     // Strip unnecessary whitespace
  
  // Enable performance optimizations
  useNativeParser: true,         // Use native XML parser if available
  enableSIMD: true,             // Enable SIMD optimizations
  
  // Memory management
  maxMemoryUsage: 1024 * 1024 * 1024, // 1GB limit
  garbageCollectionHint: true    // Hint GC when safe
});
```

### Memory-Optimized Configuration

```typescript
const parser = new DDEXParser({
  // Minimize memory footprint
  streaming: true,
  bufferSize: 32 * 1024,         // Smaller 32KB buffer
  maxConcurrentParsers: 2,       // Limit concurrent operations
  
  // Skip optional data to save memory
  includeRawExtensions: false,
  includeComments: false,
  preserveOriginalOrder: false,
  
  // Aggressive cleanup
  autoCleanup: true,
  gcFrequency: 100              // Force GC every 100 operations
});
```

## Selective Parsing

Parse only the data you need to improve performance:

```typescript
// Parse only releases, skip other sections
const result = await parser.parse(xmlContent, {
  sections: ['releases'],        // Only parse releases
  includeResources: false,       // Skip sound recordings/images
  includeDeals: false,          // Skip deal information  
  includeParties: false,        // Skip party information
  flattenOnly: true             // Skip graph representation
});

console.log(`Parsed ${result.flat.releases.length} releases`);
// Much faster when you only need release metadata
```

## Concurrent Processing

Process multiple files efficiently with controlled concurrency:

```typescript
class ConcurrentDDEXProcessor {
  private parser = new DDEXParser();
  private semaphore: Semaphore;
  
  constructor(maxConcurrency: number = 5) {
    this.semaphore = new Semaphore(maxConcurrency);
  }
  
  async processFiles(filePaths: string[]): Promise<any[]> {
    const promises = filePaths.map(filePath => 
      this.semaphore.acquire(() => this.processFile(filePath))
    );
    
    return Promise.all(promises);
  }
  
  private async processFile(filePath: string): Promise<any> {
    const startTime = performance.now();
    
    try {
      const xmlContent = await fs.readFile(filePath, 'utf-8');
      const result = await this.parser.parse(xmlContent, {
        streaming: filePath.endsWith('.large.xml'),
        bufferSize: 64 * 1024
      });
      
      const duration = performance.now() - startTime;
      console.log(`Processed ${filePath} in ${duration.toFixed(2)}ms`);
      
      return result;
    } catch (error) {
      console.error(`Failed to process ${filePath}:`, error.message);
      throw error;
    }
  }
}

class Semaphore {
  private tokens: number;
  private waitingQueue: Array<() => void> = [];
  
  constructor(tokens: number) {
    this.tokens = tokens;
  }
  
  async acquire<T>(task: () => Promise<T>): Promise<T> {
    return new Promise((resolve, reject) => {
      this.waitingQueue.push(async () => {
        try {
          const result = await task();
          this.release();
          resolve(result);
        } catch (error) {
          this.release();
          reject(error);
        }
      });
      
      this.dispatch();
    });
  }
  
  private dispatch(): void {
    if (this.tokens > 0 && this.waitingQueue.length > 0) {
      this.tokens--;
      const next = this.waitingQueue.shift()!;
      next();
    }
  }
  
  private release(): void {
    this.tokens++;
    this.dispatch();
  }
}
```

## Caching Strategies

Implement intelligent caching to avoid re-parsing:

```typescript
import { createHash } from 'crypto';

class CachedDDEXParser {
  private parser = new DDEXParser();
  private cache = new Map<string, any>();
  private maxCacheSize = 100;
  
  async parse(xmlContent: string, options: any = {}): Promise<any> {
    // Generate cache key based on content and options
    const cacheKey = this.generateCacheKey(xmlContent, options);
    
    // Check cache first
    if (this.cache.has(cacheKey)) {
      console.log('Cache hit');
      return this.cache.get(cacheKey);
    }
    
    // Parse and cache result
    const result = await this.parser.parse(xmlContent, options);
    
    // Manage cache size
    if (this.cache.size >= this.maxCacheSize) {
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
    
    this.cache.set(cacheKey, result);
    console.log('Cached new result');
    
    return result;
  }
  
  private generateCacheKey(xmlContent: string, options: any): string {
    const hash = createHash('sha256');
    hash.update(xmlContent);
    hash.update(JSON.stringify(options));
    return hash.digest('hex').substring(0, 16);
  }
  
  clearCache(): void {
    this.cache.clear();
  }
  
  getCacheStats(): { size: number; maxSize: number } {
    return {
      size: this.cache.size,
      maxSize: this.maxCacheSize
    };
  }
}
```

## Batch Processing Optimization

Optimize batch processing workflows:

```typescript
class OptimizedBatchProcessor {
  private parser = new DDEXParser();
  private processed = 0;
  private startTime = Date.now();
  
  async processBatch(xmlFiles: string[], batchSize: number = 10): Promise<any[]> {
    const results: any[] = [];
    
    for (let i = 0; i < xmlFiles.length; i += batchSize) {
      const batch = xmlFiles.slice(i, i + batchSize);
      
      // Process batch concurrently
      const batchPromises = batch.map(async (filePath) => {
        const content = await this.readFileOptimized(filePath);
        const result = await this.parser.parse(content, {
          streaming: content.length > 1024 * 1024, // Auto-detect streaming need
          bufferSize: 64 * 1024
        });
        
        this.processed++;
        this.logProgress(xmlFiles.length);
        
        return { filePath, result };
      });
      
      const batchResults = await Promise.all(batchPromises);
      results.push(...batchResults);
      
      // Cleanup between batches to prevent memory leaks
      if (global.gc && i % 50 === 0) {
        global.gc();
      }
    }
    
    return results;
  }
  
  private async readFileOptimized(filePath: string): Promise<string> {
    // Use streams for large files
    const stats = await fs.stat(filePath);
    
    if (stats.size > 10 * 1024 * 1024) { // 10MB
      return this.readFileStreaming(filePath);
    }
    
    return fs.readFile(filePath, 'utf-8');
  }
  
  private async readFileStreaming(filePath: string): Promise<string> {
    const stream = createReadStream(filePath, { encoding: 'utf-8' });
    const chunks: string[] = [];
    
    for await (const chunk of stream) {
      chunks.push(chunk);
    }
    
    return chunks.join('');
  }
  
  private logProgress(total: number): void {
    if (this.processed % 10 === 0) {
      const elapsed = Date.now() - this.startTime;
      const rate = this.processed / elapsed * 1000;
      const eta = (total - this.processed) / rate;
      
      console.log(`Progress: ${this.processed}/${total} (${rate.toFixed(1)}/s, ETA: ${eta.toFixed(0)}s)`);
    }
  }
}
```

## Memory Profiling

Monitor memory usage during parsing operations:

```typescript
class MemoryProfiledParser {
  private parser = new DDEXParser();
  
  async parseWithProfiling(xmlContent: string): Promise<any> {
    const startMemory = process.memoryUsage();
    console.log('Memory before parsing:', this.formatMemory(startMemory));
    
    const result = await this.parser.parse(xmlContent);
    
    const endMemory = process.memoryUsage();
    console.log('Memory after parsing:', this.formatMemory(endMemory));
    
    const memoryDelta = {
      heapUsed: endMemory.heapUsed - startMemory.heapUsed,
      heapTotal: endMemory.heapTotal - startMemory.heapTotal,
      external: endMemory.external - startMemory.external
    };
    
    console.log('Memory delta:', this.formatMemory(memoryDelta));
    
    return result;
  }
  
  private formatMemory(memory: NodeJS.MemoryUsage | any): string {
    const format = (bytes: number) => `${(bytes / 1024 / 1024).toFixed(2)} MB`;
    
    return Object.entries(memory)
      .map(([key, value]) => `${key}: ${format(value as number)}`)
      .join(', ');
  }
}
```

## Python Performance Optimization

```python
import time
import psutil
from concurrent.futures import ThreadPoolExecutor, ProcessPoolExecutor
from ddex_parser import DDEXParser

class OptimizedPythonParser:
    def __init__(self, max_workers=4):
        self.parser = DDEXParser()
        self.max_workers = max_workers
        
    def parse_batch_threaded(self, xml_files):
        """Use threading for I/O-bound parsing tasks"""
        with ThreadPoolExecutor(max_workers=self.max_workers) as executor:
            futures = [executor.submit(self.parse_file, file) for file in xml_files]
            results = [future.result() for future in futures]
        return results
    
    def parse_batch_multiprocess(self, xml_files):
        """Use multiprocessing for CPU-bound parsing tasks"""
        with ProcessPoolExecutor(max_workers=self.max_workers) as executor:
            results = list(executor.map(self.parse_file, xml_files))
        return results
    
    def parse_file(self, file_path):
        start_time = time.time()
        
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Choose parsing strategy based on file size
        file_size = len(content)
        if file_size > 1024 * 1024:  # 1MB
            result = self.parser.parse(content, streaming=True)
        else:
            result = self.parser.parse(content)
        
        duration = time.time() - start_time
        print(f"Parsed {file_path} in {duration:.3f}s")
        
        return result
    
    def monitor_memory(self):
        """Monitor memory usage during parsing"""
        process = psutil.Process()
        memory_info = process.memory_info()
        
        print(f"RSS: {memory_info.rss / 1024 / 1024:.2f} MB")
        print(f"VMS: {memory_info.vms / 1024 / 1024:.2f} MB")
```

## Performance Testing

Create comprehensive performance tests:

```typescript
describe('DDEX Parser Performance', () => {
  let parser: DDEXParser;
  
  beforeEach(() => {
    parser = new DDEXParser();
  });
  
  test('should parse 10KB file under 5ms', async () => {
    const xmlContent = generateTestXml(10 * 1024); // 10KB
    
    const startTime = performance.now();
    await parser.parse(xmlContent);
    const duration = performance.now() - startTime;
    
    expect(duration).toBeLessThan(5);
  });
  
  test('should handle 100 concurrent small files', async () => {
    const files = Array.from({ length: 100 }, () => generateTestXml(1024));
    
    const startTime = performance.now();
    const results = await Promise.all(
      files.map(content => parser.parse(content))
    );
    const duration = performance.now() - startTime;
    
    expect(results).toHaveLength(100);
    expect(duration).toBeLessThan(1000); // Under 1 second
  });
  
  test('memory usage should be consistent', async () => {
    const initialMemory = process.memoryUsage().heapUsed;
    
    // Parse 100 files
    for (let i = 0; i < 100; i++) {
      const content = generateTestXml(10 * 1024);
      await parser.parse(content);
    }
    
    // Force garbage collection
    if (global.gc) global.gc();
    
    const finalMemory = process.memoryUsage().heapUsed;
    const memoryIncrease = finalMemory - initialMemory;
    
    // Should not leak more than 10MB
    expect(memoryIncrease).toBeLessThan(10 * 1024 * 1024);
  });
});
```

## Production Monitoring

Set up performance monitoring in production:

```typescript
class PerformanceMonitor {
  private metrics = {
    totalParses: 0,
    totalDuration: 0,
    totalBytes: 0,
    errors: 0
  };
  
  async monitorParse(xmlContent: string, options: any = {}): Promise<any> {
    const startTime = performance.now();
    const startMemory = process.memoryUsage();
    
    try {
      const result = await parser.parse(xmlContent, options);
      
      const duration = performance.now() - startTime;
      const endMemory = process.memoryUsage();
      
      // Record metrics
      this.metrics.totalParses++;
      this.metrics.totalDuration += duration;
      this.metrics.totalBytes += xmlContent.length;
      
      // Log performance data
      console.log(`Parse completed in ${duration.toFixed(2)}ms`);
      console.log(`Memory usage: ${(endMemory.heapUsed / 1024 / 1024).toFixed(2)}MB`);
      
      return result;
      
    } catch (error) {
      this.metrics.errors++;
      throw error;
    }
  }
  
  getStats() {
    const avgDuration = this.metrics.totalDuration / this.metrics.totalParses;
    const avgThroughput = this.metrics.totalBytes / this.metrics.totalDuration * 1000;
    
    return {
      totalParses: this.metrics.totalParses,
      averageDuration: avgDuration.toFixed(2),
      throughput: `${(avgThroughput / 1024 / 1024).toFixed(2)} MB/s`,
      errorRate: (this.metrics.errors / this.metrics.totalParses * 100).toFixed(2)
    };
  }
}
```

## Next Steps

- [Large File Processing](./large-files) - Optimize for very large files
- [Error Handling](./error-handling) - Handle performance-related errors  
- [Memory Optimization](../advanced/memory) - Advanced memory management
- [DataFrame Integration](./dataframes) - Optimize analytics workflows