# Memory Management

Advanced memory management techniques for processing large DDEX files efficiently.

## Overview

Effective memory management enables:
- Processing files larger than available RAM
- Stable performance under memory pressure
- Efficient resource utilization
- Prevention of memory leaks
- Handling concurrent processing workloads

## Memory-Bounded Parsing

### Streaming Parser Configuration

```typescript
import { DDEXParser, StreamingConfig } from 'ddex-parser';

const streamingConfig: StreamingConfig = {
  // Memory limits
  maxMemoryUsage: 512 * 1024 * 1024, // 512MB
  chunkSize: 64 * 1024,              // 64KB chunks
  
  // Processing limits
  maxElementDepth: 100,
  maxElementCount: 1000000,
  
  // Buffer management
  bufferSize: 1024 * 1024,          // 1MB buffer
  flushThreshold: 0.8,              // Flush at 80% capacity
  
  // Error handling
  continueOnError: false,
  maxErrors: 10
};

export class MemoryEfficientParser {
  private parser: DDEXParser;
  private memoryMonitor: MemoryMonitor;

  constructor() {
    this.parser = new DDEXParser({ streaming: streamingConfig });
    this.memoryMonitor = new MemoryMonitor();
  }

  async parseWithMemoryControl(xmlContent: string): Promise<any> {
    const memorySnapshot = this.memoryMonitor.takeSnapshot();
    
    try {
      // Check available memory before starting
      const available = this.getAvailableMemory();
      if (available < streamingConfig.maxMemoryUsage) {
        throw new Error('Insufficient memory available');
      }

      // Start memory monitoring
      this.memoryMonitor.startMonitoring();

      const result = await this.parser.parseStream(xmlContent, {
        onChunk: (chunk, metadata) => this.processChunk(chunk, metadata),
        onMemoryPressure: () => this.handleMemoryPressure(),
        onProgress: (progress) => this.reportProgress(progress)
      });

      return result;

    } finally {
      this.memoryMonitor.stopMonitoring();
      this.logMemoryUsage(memorySnapshot);
    }
  }

  private processChunk(chunk: any, metadata: ChunkMetadata): void {
    // Process chunk incrementally
    const currentMemory = process.memoryUsage();
    
    if (currentMemory.heapUsed > streamingConfig.maxMemoryUsage * 0.9) {
      // Trigger garbage collection when approaching limit
      if (global.gc) {
        global.gc();
      }
      
      // Check if we're still over threshold
      const afterGC = process.memoryUsage();
      if (afterGC.heapUsed > streamingConfig.maxMemoryUsage * 0.85) {
        throw new Error('Memory limit exceeded during processing');
      }
    }
  }

  private handleMemoryPressure(): void {
    console.warn('Memory pressure detected, optimizing...');
    
    // Force garbage collection
    if (global.gc) {
      global.gc();
    }
    
    // Reduce buffer sizes temporarily
    this.parser.setBufferSize(streamingConfig.bufferSize * 0.5);
    
    // Flush any pending data
    this.parser.flush();
  }

  private getAvailableMemory(): number {
    const totalMemory = require('os').totalmem();
    const freeMemory = require('os').freemem();
    const processMemory = process.memoryUsage().rss;
    
    return Math.min(freeMemory, totalMemory - processMemory);
  }
}
```

### Python Memory-Efficient Processing

```python
import gc
import resource
import threading
from typing import Iterator, Optional, Dict, Any
from dataclasses import dataclass
from ddex_parser import DDEXParser, StreamingParser

@dataclass 
class MemoryConfig:
    max_memory_mb: int = 512
    chunk_size_kb: int = 64
    gc_threshold: float = 0.8  # Trigger GC at 80% memory usage
    buffer_size_kb: int = 1024

class MemoryEfficientDDEXProcessor:
    def __init__(self, config: MemoryConfig = None):
        self.config = config or MemoryConfig()
        self.parser = StreamingParser()
        self.memory_monitor = MemoryMonitor()
        
        # Set resource limits
        max_memory_bytes = self.config.max_memory_mb * 1024 * 1024
        resource.setrlimit(resource.RLIMIT_RSS, (max_memory_bytes, max_memory_bytes))
    
    def process_large_file(self, file_path: str) -> Iterator[Dict[str, Any]]:
        """Process large DDEX file in chunks without loading entire file into memory"""
        
        self.memory_monitor.start_monitoring()
        
        try:
            with open(file_path, 'rb') as f:
                for chunk_data in self.parser.parse_streaming(
                    f, 
                    chunk_size=self.config.chunk_size_kb * 1024
                ):
                    # Check memory usage
                    self._check_memory_pressure()
                    
                    # Yield processed chunk
                    yield self._process_chunk(chunk_data)
                    
                    # Manual garbage collection at intervals
                    if self.memory_monitor.should_collect_garbage():
                        self._force_garbage_collection()
        
        finally:
            self.memory_monitor.stop_monitoring()
    
    def process_multiple_files(self, file_paths: list, max_concurrent: int = 2):
        """Process multiple files with controlled concurrency"""
        
        import concurrent.futures
        from queue import Queue
        
        # Limit concurrent processing to control memory usage
        with concurrent.futures.ThreadPoolExecutor(max_workers=max_concurrent) as executor:
            # Use a queue to limit memory usage from pending futures
            active_futures = Queue(maxsize=max_concurrent * 2)
            
            def process_file_safe(file_path: str):
                """Process file with memory monitoring"""
                try:
                    return list(self.process_large_file(file_path))
                except MemoryError:
                    print(f"Memory error processing {file_path}, retrying with smaller chunks")
                    # Retry with smaller chunk size
                    old_chunk_size = self.config.chunk_size_kb
                    self.config.chunk_size_kb = old_chunk_size // 2
                    
                    try:
                        result = list(self.process_large_file(file_path))
                        return result
                    finally:
                        self.config.chunk_size_kb = old_chunk_size
            
            # Submit jobs with memory pressure checks
            futures = []
            for file_path in file_paths:
                # Wait if memory usage is too high
                while self.memory_monitor.get_memory_usage_percent() > 85:
                    time.sleep(1)
                    self._force_garbage_collection()
                
                future = executor.submit(process_file_safe, file_path)
                futures.append((file_path, future))
            
            # Collect results
            for file_path, future in futures:
                try:
                    result = future.result()
                    yield file_path, result
                except Exception as e:
                    print(f"Error processing {file_path}: {e}")
                    yield file_path, None
    
    def _process_chunk(self, chunk_data: Dict[str, Any]) -> Dict[str, Any]:
        """Process individual chunk with memory optimization"""
        
        # Convert to more memory-efficient format if needed
        if 'releases' in chunk_data:
            for release in chunk_data['releases']:
                # Remove unused fields to save memory
                release.pop('raw_xml', None)
                release.pop('debug_info', None)
                
                # Optimize track data
                if 'tracks' in release:
                    for track in release['tracks']:
                        # Convert to more compact format
                        track['duration'] = track.pop('duration_ms', 0) // 1000  # Store in seconds
        
        return chunk_data
    
    def _check_memory_pressure(self):
        """Check for memory pressure and take action"""
        
        memory_percent = self.memory_monitor.get_memory_usage_percent()
        
        if memory_percent > self.config.gc_threshold * 100:
            self._force_garbage_collection()
            
            # Check again after GC
            memory_percent = self.memory_monitor.get_memory_usage_percent()
            
            if memory_percent > 90:
                raise MemoryError(f"Memory usage too high: {memory_percent:.1f}%")
    
    def _force_garbage_collection(self):
        """Force garbage collection and log results"""
        
        before = self.memory_monitor.get_memory_usage_mb()
        
        # Full garbage collection
        gc.collect()
        
        after = self.memory_monitor.get_memory_usage_mb()
        freed = before - after
        
        if freed > 10:  # Only log if significant memory freed
            print(f"GC freed {freed:.1f}MB (before: {before:.1f}MB, after: {after:.1f}MB)")

class MemoryMonitor:
    def __init__(self):
        self.monitoring = False
        self.gc_count = 0
        self.start_memory = None
    
    def start_monitoring(self):
        """Start memory monitoring"""
        import psutil
        
        self.monitoring = True
        self.start_memory = psutil.Process().memory_info().rss
        self.gc_count = 0
    
    def stop_monitoring(self):
        """Stop monitoring and return summary"""
        if not self.monitoring:
            return None
            
        self.monitoring = False
        
        import psutil
        end_memory = psutil.Process().memory_info().rss
        
        return {
            'start_memory_mb': self.start_memory / 1024 / 1024,
            'end_memory_mb': end_memory / 1024 / 1024,
            'memory_delta_mb': (end_memory - self.start_memory) / 1024 / 1024,
            'gc_count': self.gc_count
        }
    
    def get_memory_usage_mb(self) -> float:
        """Get current memory usage in MB"""
        import psutil
        return psutil.Process().memory_info().rss / 1024 / 1024
    
    def get_memory_usage_percent(self) -> float:
        """Get memory usage as percentage of system memory"""
        import psutil
        return psutil.virtual_memory().percent
    
    def should_collect_garbage(self) -> bool:
        """Determine if garbage collection should be triggered"""
        # Collect every 100MB of processing or when memory usage is high
        current_memory = self.get_memory_usage_mb()
        
        if self.start_memory is None:
            return False
            
        memory_delta = current_memory - (self.start_memory / 1024 / 1024)
        
        return (memory_delta > 100 or  # Every 100MB increase
                self.get_memory_usage_percent() > 80)  # Or when system memory > 80%

# Usage example
config = MemoryConfig(
    max_memory_mb=1024,  # 1GB limit
    chunk_size_kb=32,    # 32KB chunks for very large files
    gc_threshold=0.7     # GC at 70% usage
)

processor = MemoryEfficientDDEXProcessor(config)

# Process single large file
for chunk_result in processor.process_large_file('very_large_release.xml'):
    print(f"Processed chunk with {len(chunk_result.get('releases', []))} releases")

# Process multiple files with memory control
files = ['file1.xml', 'file2.xml', 'file3.xml']
for file_path, result in processor.process_multiple_files(files, max_concurrent=2):
    if result:
        print(f"Successfully processed {file_path}")
    else:
        print(f"Failed to process {file_path}")
```

## Memory Pooling and Reuse

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct MemoryPool<T> {
    objects: Arc<Mutex<Vec<T>>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T> MemoryPool<T> {
    pub fn new<F>(factory: F, max_size: usize) -> Self 
    where 
        F: Fn() -> T + Send + Sync + 'static
    {
        Self {
            objects: Arc::new(Mutex::new(Vec::new())),
            factory: Box::new(factory),
            max_size,
        }
    }

    pub fn acquire(&self) -> PooledObject<T> {
        let mut objects = self.objects.lock().unwrap();
        
        let object = if let Some(obj) = objects.pop() {
            obj
        } else {
            (self.factory)()
        };
        
        PooledObject {
            object: Some(object),
            pool: self.objects.clone(),
            max_size: self.max_size,
        }
    }
}

pub struct PooledObject<T> {
    object: Option<T>,
    pool: Arc<Mutex<Vec<T>>>,
    max_size: usize,
}

impl<T> PooledObject<T> {
    pub fn get(&self) -> &T {
        self.object.as_ref().unwrap()
    }
    
    pub fn get_mut(&mut self) -> &mut T {
        self.object.as_mut().unwrap()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            let mut objects = self.pool.lock().unwrap();
            if objects.len() < self.max_size {
                objects.push(object);
            }
            // Otherwise let object drop naturally
        }
    }
}

// Buffer pooling for DDEX processing
pub struct DDEXBufferPool {
    string_pool: MemoryPool<String>,
    vec_pool: MemoryPool<Vec<u8>>,
    hashmap_pool: MemoryPool<HashMap<String, String>>,
}

impl DDEXBufferPool {
    pub fn new() -> Self {
        Self {
            string_pool: MemoryPool::new(
                || String::with_capacity(4096),
                100
            ),
            vec_pool: MemoryPool::new(
                || Vec::with_capacity(8192), 
                50
            ),
            hashmap_pool: MemoryPool::new(
                || HashMap::with_capacity(64),
                25
            ),
        }
    }

    pub fn get_string_buffer(&self) -> PooledObject<String> {
        let mut buffer = self.string_pool.acquire();
        buffer.get_mut().clear();  // Reset for reuse
        buffer
    }

    pub fn get_byte_buffer(&self) -> PooledObject<Vec<u8>> {
        let mut buffer = self.vec_pool.acquire();
        buffer.get_mut().clear();  // Reset for reuse
        buffer
    }

    pub fn get_metadata_map(&self) -> PooledObject<HashMap<String, String>> {
        let mut map = self.hashmap_pool.acquire();
        map.get_mut().clear();  // Reset for reuse
        map
    }
}

// Memory-efficient DDEX parser using pools
pub struct PooledDDEXParser {
    buffer_pool: DDEXBufferPool,
    parser_config: ParserConfig,
}

impl PooledDDEXParser {
    pub fn new() -> Self {
        Self {
            buffer_pool: DDEXBufferPool::new(),
            parser_config: ParserConfig::default(),
        }
    }

    pub fn parse_with_pooling(&self, xml_data: &str) -> Result<DDEXData, ParseError> {
        // Use pooled buffers for processing
        let mut work_buffer = self.buffer_pool.get_string_buffer();
        let mut byte_buffer = self.buffer_pool.get_byte_buffer();
        let mut metadata_map = self.buffer_pool.get_metadata_map();

        // Process XML using pooled resources
        work_buffer.get_mut().push_str(xml_data);
        
        // Parsing logic using pooled buffers...
        let result = self.parse_internal(
            work_buffer.get(),
            byte_buffer.get_mut(),
            metadata_map.get_mut()
        );

        // Buffers automatically returned to pool when dropped
        result
    }

    fn parse_internal(
        &self,
        xml_data: &str,
        work_buffer: &mut Vec<u8>,
        metadata: &mut HashMap<String, String>
    ) -> Result<DDEXData, ParseError> {
        // Implementation using provided buffers
        // This avoids allocating new buffers for each parse operation
        Ok(DDEXData::default())
    }
}
```

## Memory Leak Detection

```typescript
export class MemoryLeakDetector {
  private snapshots: MemorySnapshot[] = [];
  private intervalId?: NodeJS.Timeout;
  private leakThreshold = 50 * 1024 * 1024; // 50MB growth threshold

  startDetection(intervalMs: number = 30000): void {
    this.intervalId = setInterval(() => {
      this.takeSnapshot();
      this.analyzeSnapshots();
    }, intervalMs);
  }

  stopDetection(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = undefined;
    }
  }

  private takeSnapshot(): void {
    const memUsage = process.memoryUsage();
    
    const snapshot: MemorySnapshot = {
      timestamp: Date.now(),
      heapUsed: memUsage.heapUsed,
      heapTotal: memUsage.heapTotal,
      external: memUsage.external,
      rss: memUsage.rss
    };

    this.snapshots.push(snapshot);

    // Keep only recent snapshots (last hour)
    const oneHourAgo = Date.now() - 60 * 60 * 1000;
    this.snapshots = this.snapshots.filter(s => s.timestamp > oneHourAgo);
  }

  private analyzeSnapshots(): void {
    if (this.snapshots.length < 3) return;

    const recent = this.snapshots.slice(-3);
    const growth = recent[2].heapUsed - recent[0].heapUsed;

    if (growth > this.leakThreshold) {
      this.reportPotentialLeak(growth);
    }
  }

  private reportPotentialLeak(growth: number): void {
    const growthMB = growth / 1024 / 1024;
    
    console.warn(`🚨 Potential memory leak detected!`);
    console.warn(`Memory growth: ${growthMB.toFixed(2)}MB over recent samples`);
    
    // Trigger heap dump for analysis
    this.createHeapDump();
    
    // Force garbage collection to see if growth persists
    if (global.gc) {
      global.gc();
      
      setTimeout(() => {
        const afterGC = process.memoryUsage().heapUsed;
        const reduction = this.snapshots[this.snapshots.length - 1].heapUsed - afterGC;
        const reductionMB = reduction / 1024 / 1024;
        
        console.log(`Memory after GC: ${reductionMB.toFixed(2)}MB freed`);
        
        if (reduction < growth * 0.5) {
          console.error(`⚠️  Potential memory leak confirmed - GC only freed ${reductionMB.toFixed(2)}MB`);
          this.alertOnLeak(growthMB);
        }
      }, 1000);
    }
  }

  private createHeapDump(): void {
    try {
      const v8 = require('v8');
      const fs = require('fs');
      const path = require('path');
      
      const filename = `heap-${Date.now()}.heapsnapshot`;
      const filepath = path.join(process.cwd(), 'logs', filename);
      
      const heapSnapshot = v8.getHeapSnapshot();
      const writeStream = fs.createWriteStream(filepath);
      
      heapSnapshot.pipe(writeStream);
      
      console.log(`Heap dump saved to: ${filepath}`);
      
    } catch (error) {
      console.error('Failed to create heap dump:', error);
    }
  }

  private alertOnLeak(growthMB: number): void {
    // Send alert to monitoring system
    const alert = {
      type: 'memory_leak',
      severity: 'high',
      message: `Memory leak detected: ${growthMB.toFixed(2)}MB growth`,
      timestamp: new Date().toISOString(),
      process: process.pid,
      version: process.version
    };

    // In production, send to alerting system
    console.error('MEMORY_LEAK_ALERT:', JSON.stringify(alert, null, 2));
  }

  getMemoryTrend(): MemoryTrend {
    if (this.snapshots.length < 2) {
      return { trend: 'unknown', rate: 0 };
    }

    const first = this.snapshots[0];
    const last = this.snapshots[this.snapshots.length - 1];
    const timeDiff = (last.timestamp - first.timestamp) / 1000; // seconds
    const memoryDiff = last.heapUsed - first.heapUsed;
    
    const rate = memoryDiff / timeDiff; // bytes per second

    let trend: 'growing' | 'stable' | 'decreasing';
    if (Math.abs(rate) < 1000) { // Less than 1KB/s change
      trend = 'stable';
    } else if (rate > 0) {
      trend = 'growing';
    } else {
      trend = 'decreasing';
    }

    return { trend, rate };
  }
}

interface MemorySnapshot {
  timestamp: number;
  heapUsed: number;
  heapTotal: number;
  external: number;
  rss: number;
}

interface MemoryTrend {
  trend: 'growing' | 'stable' | 'decreasing' | 'unknown';
  rate: number; // bytes per second
}

// Usage with DDEX processing
export class MemoryManagedDDEXService {
  private parser: DDEXParser;
  private builder: DDEXBuilder;
  private leakDetector: MemoryLeakDetector;

  constructor() {
    this.parser = new DDEXParser();
    this.builder = new DDEXBuilder();
    this.leakDetector = new MemoryLeakDetector();
    
    // Start leak detection
    this.leakDetector.startDetection(60000); // Check every minute
  }

  async processWithMemoryTracking(xmlContent: string): Promise<any> {
    const startMemory = process.memoryUsage().heapUsed;
    
    try {
      const result = await this.parser.parse(xmlContent);
      
      // Check for unexpected memory growth
      const endMemory = process.memoryUsage().heapUsed;
      const growth = endMemory - startMemory;
      const growthMB = growth / 1024 / 1024;
      
      if (growthMB > 100) { // More than 100MB growth for single operation
        console.warn(`Large memory growth during processing: ${growthMB.toFixed(2)}MB`);
      }
      
      return result;
      
    } finally {
      // Force cleanup
      if (global.gc && Math.random() < 0.1) { // 10% chance to trigger GC
        global.gc();
      }
    }
  }

  getMemoryStatus(): any {
    const trend = this.leakDetector.getMemoryTrend();
    const current = process.memoryUsage();
    
    return {
      current: {
        heapUsedMB: current.heapUsed / 1024 / 1024,
        heapTotalMB: current.heapTotal / 1024 / 1024,
        rssMB: current.rss / 1024 / 1024
      },
      trend,
      timestamp: new Date().toISOString()
    };
  }

  shutdown(): void {
    this.leakDetector.stopDetection();
  }
}
```

## Best Practices

1. **Streaming Processing**: Use streaming for large files to avoid loading entire content into memory
2. **Memory Limits**: Set and enforce memory limits for processing operations
3. **Garbage Collection**: Monitor and optimize garbage collection patterns
4. **Object Pooling**: Reuse objects to reduce allocation overhead
5. **Memory Monitoring**: Continuously monitor memory usage in production
6. **Leak Detection**: Implement automatic memory leak detection
7. **Resource Cleanup**: Ensure proper cleanup of resources after processing
8. **Chunked Processing**: Break large operations into smaller chunks
9. **Memory Profiling**: Regular profiling to identify optimization opportunities
10. **Capacity Planning**: Plan memory requirements based on workload analysis