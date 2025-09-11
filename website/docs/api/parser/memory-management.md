# Memory Management

The DDEX Parser includes sophisticated memory management to handle large XML files efficiently while preventing out-of-memory errors.

## Memory Management Strategies

### Adaptive Memory Allocation

The parser automatically adjusts memory usage based on input size and available system resources:

```typescript
interface MemoryOptions {
  maxMemoryUsage?: number;      // Maximum memory limit (bytes)
  adaptiveThresholds?: boolean; // Auto-adjust based on file size
  gcTriggerThreshold?: number;  // Trigger GC at percentage (0.0-1.0)
  memoryWarningThreshold?: number; // Warn at percentage (0.0-1.0)
  enablePooling?: boolean;      // Object pooling to reduce allocations
  chunkingStrategy?: 'fixed' | 'adaptive' | 'dynamic';
}
```

### Memory-Aware Parser Configuration

```typescript
import { DDEXParser, MemoryManager } from 'ddex-parser';

// Configure memory management
const parser = new DDEXParser({
  memory: {
    maxMemoryUsage: 256 * 1024 * 1024,  // 256MB limit
    adaptiveThresholds: true,
    gcTriggerThreshold: 0.85,            // GC at 85% usage
    memoryWarningThreshold: 0.75,        // Warn at 75% usage
    enablePooling: true,
    chunkingStrategy: 'adaptive'
  }
});

// Monitor memory events
parser.on('memoryWarning', (usage) => {
  console.warn(`⚠️  Memory usage: ${(usage.percentage * 100).toFixed(1)}%`);
});

parser.on('memoryLimit', (usage) => {
  console.error(`❌ Memory limit reached: ${usage.currentMB}MB / ${usage.limitMB}MB`);
});
```

## Memory Monitoring

### Real-time Memory Tracking

```typescript
interface MemoryUsage {
  heapUsed: number;        // Currently used heap memory
  heapTotal: number;       // Total allocated heap
  external: number;        // External memory (buffers, etc.)
  rss: number;            // Resident Set Size
  arrayBuffers: number;    // ArrayBuffer memory
}

interface MemoryStats {
  current: MemoryUsage;
  peak: MemoryUsage;
  limit: number;
  percentage: number;
  allocationsPerSec: number;
  garbageCollections: number;
}

class MemoryMonitor {
  private stats: MemoryStats;
  private startTime: number;
  
  constructor(private options: MemoryOptions) {
    this.startTime = Date.now();
    this.resetStats();
  }
  
  getCurrentStats(): MemoryStats {
    const memUsage = process.memoryUsage();
    this.updateStats(memUsage);
    return this.stats;
  }
  
  private updateStats(usage: MemoryUsage): void {
    this.stats.current = usage;
    
    // Track peak usage
    if (usage.heapUsed > this.stats.peak.heapUsed) {
      this.stats.peak = { ...usage };
    }
    
    // Calculate percentage of limit
    this.stats.percentage = usage.heapUsed / this.options.maxMemoryUsage!;
    
    // Trigger warnings if needed
    if (this.stats.percentage > (this.options.memoryWarningThreshold || 0.75)) {
      this.emitMemoryWarning();
    }
  }
  
  private emitMemoryWarning(): void {
    // Implementation for memory warnings
  }
}
```

### Usage Example with Monitoring

```typescript
const parser = new DDEXParser({
  memory: {
    maxMemoryUsage: 128 * 1024 * 1024, // 128MB
    adaptiveThresholds: true,
    gcTriggerThreshold: 0.8,
    enablePooling: true
  }
});

// Set up memory monitoring
const memoryMonitor = new MemoryMonitor(parser.memoryOptions);
const monitoringInterval = setInterval(() => {
  const stats = memoryMonitor.getCurrentStats();
  
  console.log(`Memory: ${Math.round(stats.current.heapUsed / 1024 / 1024)}MB ` +
              `(${(stats.percentage * 100).toFixed(1)}% of limit)`);
              
  if (stats.percentage > 0.9) {
    console.warn('⚠️  Memory usage critical - consider streaming mode');
  }
}, 5000);

try {
  const result = await parser.parse(largeXmlContent);
  console.log(`✅ Parsed successfully. Peak memory: ${Math.round(memoryMonitor.getCurrentStats().peak.heapUsed / 1024 / 1024)}MB`);
} catch (error) {
  if (error.code === 'MEMORY_LIMIT_EXCEEDED') {
    console.error('❌ Out of memory - try streaming or increase limit');
  }
} finally {
  clearInterval(monitoringInterval);
}
```

## Memory Optimization Techniques

### Object Pooling

Reduce garbage collection pressure through object reuse:

```typescript
class ObjectPool<T> {
  private pool: T[] = [];
  private createFn: () => T;
  private resetFn: (obj: T) => void;
  
  constructor(createFn: () => T, resetFn: (obj: T) => void, initialSize: number = 10) {
    this.createFn = createFn;
    this.resetFn = resetFn;
    
    // Pre-populate pool
    for (let i = 0; i < initialSize; i++) {
      this.pool.push(createFn());
    }
  }
  
  acquire(): T {
    return this.pool.pop() || this.createFn();
  }
  
  release(obj: T): void {
    this.resetFn(obj);
    this.pool.push(obj);
  }
  
  clear(): void {
    this.pool.length = 0;
  }
}

// Usage in parser
const releasePool = new ObjectPool(
  () => ({ releaseId: '', title: '', tracks: [] }),
  (release) => {
    release.releaseId = '';
    release.title = '';
    release.tracks.length = 0;
  },
  50 // Pre-allocate 50 release objects
);

// Parser uses pooled objects
const release = releasePool.acquire();
// ... populate release data ...
releasePool.release(release); // Return to pool when done
```

### Chunked Processing

Process large files in memory-efficient chunks:

```typescript
interface ChunkingOptions {
  chunkSize: number;           // Size of each chunk in bytes
  overlapSize: number;         // Overlap between chunks for continuity
  maxConcurrentChunks: number; // Max chunks in memory at once
}

class ChunkedProcessor {
  private options: ChunkingOptions;
  private activeChunks = new Map<string, any>();
  
  constructor(options: ChunkingOptions) {
    this.options = options;
  }
  
  async processInChunks(data: Buffer, processor: (chunk: Buffer) => Promise<any>): Promise<void> {
    const totalSize = data.length;
    let offset = 0;
    let chunkId = 0;
    
    while (offset < totalSize) {
      // Wait if too many active chunks
      while (this.activeChunks.size >= this.options.maxConcurrentChunks) {
        await this.waitForChunkCompletion();
      }
      
      const chunkSize = Math.min(this.options.chunkSize, totalSize - offset);
      const chunk = data.slice(offset, offset + chunkSize);
      
      const currentChunkId = `chunk-${chunkId++}`;
      this.activeChunks.set(currentChunkId, true);
      
      // Process chunk asynchronously
      processor(chunk)
        .then(() => {
          this.activeChunks.delete(currentChunkId);
        })
        .catch((error) => {
          console.error(`Error processing chunk ${currentChunkId}:`, error);
          this.activeChunks.delete(currentChunkId);
        });
      
      offset += chunkSize - this.options.overlapSize;
    }
    
    // Wait for all chunks to complete
    while (this.activeChunks.size > 0) {
      await this.waitForChunkCompletion();
    }
  }
  
  private async waitForChunkCompletion(): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, 100));
  }
}
```

### Memory-Efficient Data Structures

Use specialized data structures for large datasets:

```typescript
// Streaming array for large collections
class StreamingArray<T> {
  private items: T[] = [];
  private maxSize: number;
  private flushCallback?: (items: T[]) => Promise<void>;
  
  constructor(maxSize: number, flushCallback?: (items: T[]) => Promise<void>) {
    this.maxSize = maxSize;
    this.flushCallback = flushCallback;
  }
  
  async push(item: T): Promise<void> {
    this.items.push(item);
    
    if (this.items.length >= this.maxSize) {
      await this.flush();
    }
  }
  
  async flush(): Promise<void> {
    if (this.items.length > 0 && this.flushCallback) {
      await this.flushCallback([...this.items]);
      this.items.length = 0; // Clear array
    }
  }
  
  get length(): number {
    return this.items.length;
  }
}

// Usage for processing large release collections
const releases = new StreamingArray<Release>(1000, async (batch) => {
  console.log(`Processing batch of ${batch.length} releases`);
  await processBatch(batch);
  
  // Force garbage collection after processing
  if (global.gc) {
    global.gc();
  }
});

// Parser fills streaming array
for (const release of parsedReleases) {
  await releases.push(release);
}

// Flush remaining items
await releases.flush();
```

## Garbage Collection Management

### Proactive GC Triggering

```typescript
class GCManager {
  private lastGC: number = Date.now();
  private gcThreshold: number;
  private memoryLimit: number;
  
  constructor(gcThreshold: number = 0.85, memoryLimit: number = 256 * 1024 * 1024) {
    this.gcThreshold = gcThreshold;
    this.memoryLimit = memoryLimit;
  }
  
  checkAndTriggerGC(): boolean {
    const memUsage = process.memoryUsage();
    const memoryRatio = memUsage.heapUsed / this.memoryLimit;
    
    if (memoryRatio > this.gcThreshold) {
      this.triggerGC();
      return true;
    }
    
    return false;
  }
  
  private triggerGC(): void {
    const beforeMem = process.memoryUsage().heapUsed;
    
    if (global.gc) {
      global.gc();
      
      const afterMem = process.memoryUsage().heapUsed;
      const freed = beforeMem - afterMem;
      
      console.log(`🗑️  GC freed ${Math.round(freed / 1024 / 1024)}MB`);
      this.lastGC = Date.now();
    } else {
      console.warn('⚠️  Garbage collection not available (use --expose-gc)');
    }
  }
  
  getTimeSinceLastGC(): number {
    return Date.now() - this.lastGC;
  }
}

// Usage in parser
const gcManager = new GCManager(0.8, 200 * 1024 * 1024); // 200MB limit

// Trigger GC periodically during parsing
setInterval(() => {
  if (gcManager.checkAndTriggerGC()) {
    console.log('🗑️  Proactive GC triggered');
  }
}, 10000); // Check every 10 seconds
```

### Memory Leak Detection

```typescript
class MemoryLeakDetector {
  private snapshots: MemoryUsage[] = [];
  private maxSnapshots: number = 10;
  
  takeSnapshot(): void {
    const usage = process.memoryUsage();
    this.snapshots.push(usage);
    
    if (this.snapshots.length > this.maxSnapshots) {
      this.snapshots.shift();
    }
  }
  
  detectLeak(): { isLeak: boolean; trend: number; recommendation: string } {
    if (this.snapshots.length < 3) {
      return { isLeak: false, trend: 0, recommendation: 'Need more data' };
    }
    
    const recent = this.snapshots.slice(-3);
    const trend = (recent[2].heapUsed - recent[0].heapUsed) / recent[0].heapUsed;
    
    const isLeak = trend > 0.1; // 10% increase trend
    
    let recommendation = '';
    if (isLeak) {
      recommendation = 'Possible memory leak detected. Consider using streaming mode or object pooling.';
    } else if (trend > 0.05) {
      recommendation = 'Memory usage trending upward. Monitor closely.';
    } else {
      recommendation = 'Memory usage stable.';
    }
    
    return { isLeak, trend, recommendation };
  }
}

// Usage during long-running operations
const leakDetector = new MemoryLeakDetector();

setInterval(() => {
  leakDetector.takeSnapshot();
  const analysis = leakDetector.detectLeak();
  
  if (analysis.isLeak) {
    console.error(`🚨 ${analysis.recommendation}`);
  } else if (analysis.trend > 0.05) {
    console.warn(`⚠️  ${analysis.recommendation}`);
  }
}, 30000); // Check every 30 seconds
```

## Best Practices

### Memory-Efficient Parsing Patterns

```typescript
// ✅ Good: Stream large files
const parser = new DDEXParser({
  memory: { maxMemoryUsage: 100 * 1024 * 1024 },
  streaming: true
});

for await (const release of parser.parseStream(largeFile)) {
  await processRelease(release);
  // Each release is processed and can be garbage collected
}

// ❌ Bad: Load entire large file into memory
const result = await parser.parse(entireLargeFileContent); // May cause OOM
```

### Resource Cleanup

```typescript
class ResourceManager {
  private resources = new Set<() => void>();
  
  register(cleanup: () => void): void {
    this.resources.add(cleanup);
  }
  
  cleanup(): void {
    for (const cleanup of this.resources) {
      try {
        cleanup();
      } catch (error) {
        console.error('Error during cleanup:', error);
      }
    }
    this.resources.clear();
  }
}

// Usage
const resourceManager = new ResourceManager();

// Register cleanup functions
resourceManager.register(() => objectPool.clear());
resourceManager.register(() => streamingArray.flush());
resourceManager.register(() => clearInterval(monitoringInterval));

// Ensure cleanup happens
process.on('exit', () => resourceManager.cleanup());
process.on('SIGINT', () => {
  resourceManager.cleanup();
  process.exit(0);
});
```

## See Also

- [Streaming API](./streaming) - Stream processing for large files
- [Performance Guide](../../guides/performance-tuning) - Overall performance optimization
- [Parser API](./index.md) - Main parser documentation