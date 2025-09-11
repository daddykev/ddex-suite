# Memory Issues

Troubleshoot and resolve memory-related problems in DDEX Suite.

## Common Memory Issues

### Out of Memory Errors

**Symptoms**: Application crashes with "Out of memory" errors

**Causes**:
- Processing very large XML files
- Memory leaks in processing loops
- Insufficient system memory
- Multiple concurrent operations

**Solutions**:
```typescript
// Use streaming parser for large files
const parser = new DDEXParser({
  streaming: true,
  maxMemoryUsage: 512 * 1024 * 1024, // 512MB limit
  chunkSize: 64 * 1024 // 64KB chunks
});

// Process file in chunks
for await (const chunk of parser.parseStream(xmlContent)) {
  processChunk(chunk);
  // Memory is freed after each chunk
}
```

### Memory Leaks

**Symptoms**: Gradually increasing memory usage over time

**Solutions**:
```python
# Proper cleanup in loops
parser = DDEXParser()

for xml_file in large_file_list:
    try:
        with open(xml_file, 'r') as f:
            result = parser.parse(f.read())
            process_result(result)
    finally:
        # Explicit cleanup
        del result
        gc.collect()  # Force garbage collection periodically
```

### High Memory Usage

**Symptoms**: Unexpectedly high memory consumption

**Debugging**:
```typescript
// Monitor memory usage
function monitorMemory(operation: string) {
  const before = process.memoryUsage();
  
  return {
    end: () => {
      const after = process.memoryUsage();
      console.log(`${operation} Memory Usage:`);
      console.log(`  Heap: ${(after.heapUsed - before.heapUsed) / 1024 / 1024:.1f}MB`);
      console.log(`  RSS: ${(after.rss - before.rss) / 1024 / 1024:.1f}MB`);
    }
  };
}

// Usage
const monitor = monitorMemory('DDEX Parse');
const result = await parser.parse(xmlContent);
monitor.end();
```

## Memory Optimization Strategies

### 1. Streaming Processing
```python
# Process large files without loading into memory
def process_large_ddex_file(file_path):
    parser = DDEXParser()
    
    with open(file_path, 'rb') as f:
        for chunk in parser.parse_streaming(f, chunk_size=1024*1024):
            yield process_chunk(chunk)
            # Each chunk is processed and released
```

### 2. Memory Pooling
```typescript
// Reuse objects to reduce allocations
class DDEXProcessor {
  private objectPool = {
    releases: [],
    tracks: [],
    buffers: []
  };
  
  getRelease() {
    return this.objectPool.releases.pop() || { tracks: [] };
  }
  
  returnRelease(release: any) {
    // Clear and return to pool
    release.tracks.length = 0;
    Object.keys(release).forEach(key => {
      if (key !== 'tracks') delete release[key];
    });
    this.objectPool.releases.push(release);
  }
}
```

### 3. Garbage Collection Tuning
```typescript
// Force garbage collection at appropriate times
class MemoryManagedProcessor {
  private processedCount = 0;
  
  async processFile(xmlContent: string) {
    const result = await this.parser.parse(xmlContent);
    
    this.processedCount++;
    
    // Trigger GC every 100 files
    if (this.processedCount % 100 === 0 && global.gc) {
      global.gc();
    }
    
    return result;
  }
}
```

## Memory Monitoring

### Real-time Monitoring
```python
import psutil
import threading
import time

class MemoryMonitor:
    def __init__(self, threshold_mb=1000):
        self.threshold_mb = threshold_mb
        self.monitoring = False
        
    def start_monitoring(self):
        self.monitoring = True
        threading.Thread(target=self._monitor_loop, daemon=True).start()
        
    def _monitor_loop(self):
        while self.monitoring:
            memory_mb = psutil.Process().memory_info().rss / 1024 / 1024
            
            if memory_mb > self.threshold_mb:
                print(f"WARNING: High memory usage: {memory_mb:.1f}MB")
                
            time.sleep(5)  # Check every 5 seconds

# Usage
monitor = MemoryMonitor(threshold_mb=500)
monitor.start_monitoring()
```

### Memory Profiling
```typescript
// Profile memory usage patterns
class MemoryProfiler {
  private snapshots: Array<{
    timestamp: number;
    heapUsed: number;
    operation: string;
  }> = [];

  takeSnapshot(operation: string) {
    const usage = process.memoryUsage();
    this.snapshots.push({
      timestamp: Date.now(),
      heapUsed: usage.heapUsed,
      operation
    });
  }

  getMemoryTrend(): 'increasing' | 'stable' | 'decreasing' {
    if (this.snapshots.length < 2) return 'stable';
    
    const recent = this.snapshots.slice(-10);
    const first = recent[0].heapUsed;
    const last = recent[recent.length - 1].heapUsed;
    
    const change = (last - first) / first;
    
    if (change > 0.1) return 'increasing';
    if (change < -0.1) return 'decreasing';
    return 'stable';
  }
}
```

## Memory Configuration

### Node.js Memory Settings
```bash
# Increase heap size for large operations
node --max-old-space-size=4096 your-ddex-processor.js

# Enable garbage collection logging
node --trace-gc your-ddex-processor.js
```

### Python Memory Settings
```python
# Set memory limits
import resource

# Limit memory usage to 2GB
resource.setrlimit(resource.RLIMIT_RSS, (2*1024*1024*1024, 2*1024*1024*1024))

# Configure garbage collection
import gc
gc.set_threshold(700, 10, 10)  # More frequent GC
```

## Emergency Recovery

### Memory Pressure Response
```typescript
// Respond to memory pressure
function handleMemoryPressure() {
  console.warn('Memory pressure detected, taking corrective action...');
  
  // Clear caches
  cache.clear();
  
  // Force garbage collection
  if (global.gc) {
    global.gc();
  }
  
  // Reduce buffer sizes
  parser.setBufferSize(parser.getBufferSize() / 2);
  
  // Pause processing temporarily
  setTimeout(() => {
    console.log('Resuming processing after memory cleanup');
    resumeProcessing();
  }, 5000);
}

// Monitor memory and respond to pressure
setInterval(() => {
  const usage = process.memoryUsage();
  const usagePercent = usage.heapUsed / usage.heapTotal;
  
  if (usagePercent > 0.9) {
    handleMemoryPressure();
  }
}, 10000);
```

## Best Practices

1. **Use Streaming**: Always use streaming for files >10MB
2. **Monitor Usage**: Implement memory monitoring in production
3. **Set Limits**: Configure appropriate memory limits
4. **Clean Up**: Explicitly clean up resources in loops
5. **Pool Objects**: Reuse objects when processing many files
6. **Force GC**: Trigger garbage collection at appropriate intervals
7. **Profile Regularly**: Use profiling tools to identify memory issues
8. **Plan Capacity**: Size systems based on expected memory usage