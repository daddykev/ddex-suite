# Streaming Large Files Guide

Comprehensive guide to handling gigabyte-sized DDEX files efficiently using streaming techniques, memory management, and performance optimization.

## Problem Statement

Modern music catalogs can contain enormous DDEX files that present significant challenges:

- **File Sizes**: DDEX files can exceed 1GB with thousands of releases and resources
- **Memory Constraints**: Loading entire files into memory causes out-of-memory errors
- **Processing Time**: Traditional parsing becomes prohibitively slow for large files
- **Network Bandwidth**: Downloading and transferring large files is expensive
- **Real-time Requirements**: Streaming platforms need near real-time processing
- **Parallel Processing**: Large files need to be processed in chunks simultaneously

Without proper streaming techniques, large file processing fails or becomes impractically slow, limiting the ability to handle enterprise-scale music catalogs.

## Solution Approach

The DDEX Suite provides comprehensive streaming capabilities:

1. **Streaming Parser**: Memory-bounded XML parsing with configurable buffers
2. **Chunk Processing**: Break large files into manageable processing units
3. **Parallel Streams**: Process multiple chunks concurrently
4. **Memory Management**: Automatic cleanup and garbage collection
5. **Progress Tracking**: Real-time monitoring of processing status
6. **Error Recovery**: Fault-tolerant processing with partial recovery

## Basic Streaming Concepts

### Understanding Stream Processing

```typescript
import { StreamingDDEXParser, StreamConfig } from 'ddex-parser';

interface StreamConfig {
  chunkSize: number;          // Bytes per chunk (default: 64KB)
  maxMemory: string;          // Maximum memory usage (e.g., '500MB')
  parallelChunks: number;     // Concurrent processing threads
  enableProgress: boolean;    // Progress reporting
  bufferSize: number;         // Internal buffer size
}

async function basicStreamingExample() {
  const parser = new StreamingDDEXParser({
    chunkSize: 64 * 1024,      // 64KB chunks
    maxMemory: '500MB',        // 500MB memory limit
    parallelChunks: 4,         // 4 concurrent workers
    enableProgress: true       // Show progress
  });
  
  // Stream from file
  const fileStream = fs.createReadStream('large-catalog.xml');
  
  // Process in streaming fashion
  const result = await parser.parseStream(fileStream, {
    onProgress: (progress) => {
      console.log(`Processing: ${progress.percentComplete}%`);
      console.log(`Memory usage: ${progress.memoryUsage}MB`);
    },
    onChunkProcessed: (chunk) => {
      console.log(`Processed chunk: ${chunk.releases.length} releases`);
    }
  });
  
  console.log(`Total releases processed: ${result.totalReleases}`);
  return result;
}
```

### Memory-Bounded Processing

```typescript
class MemoryBoundedProcessor {
  private memoryLimit: number;
  private currentMemoryUsage: number = 0;
  private processedChunks: any[] = [];
  
  constructor(memoryLimitMB: number) {
    this.memoryLimit = memoryLimitMB * 1024 * 1024; // Convert to bytes
  }
  
  async processLargeFile(filePath: string): Promise<ProcessingResult> {
    const fileSize = fs.statSync(filePath).size;
    const estimatedChunkSize = this.calculateOptimalChunkSize(fileSize);
    
    console.log(`File size: ${fileSize} bytes`);
    console.log(`Using chunk size: ${estimatedChunkSize} bytes`);
    
    const parser = new StreamingDDEXParser({
      chunkSize: estimatedChunkSize,
      maxMemory: `${this.memoryLimit / 1024 / 1024}MB`,
      enableGarbageCollection: true
    });
    
    const fileStream = fs.createReadStream(filePath, {
      highWaterMark: estimatedChunkSize
    });
    
    const result = await parser.parseStream(fileStream, {
      onChunkProcessed: async (chunk) => {
        await this.processChunkWithMemoryManagement(chunk);
      },
      onMemoryPressure: async (usage) => {
        await this.handleMemoryPressure(usage);
      }
    });
    
    return {
      totalProcessed: result.totalReleases,
      memoryPeakUsage: this.currentMemoryUsage,
      chunksProcessed: this.processedChunks.length
    };
  }
  
  private calculateOptimalChunkSize(fileSize: number): number {
    // Calculate chunk size based on file size and memory limit
    const maxChunks = 100; // Reasonable number of chunks
    const minChunkSize = 32 * 1024; // 32KB minimum
    const maxChunkSize = this.memoryLimit / 4; // Don't use more than 1/4 of memory per chunk
    
    let chunkSize = Math.min(fileSize / maxChunks, maxChunkSize);
    chunkSize = Math.max(chunkSize, minChunkSize);
    
    return Math.floor(chunkSize);
  }
  
  private async processChunkWithMemoryManagement(chunk: any): Promise<void> {
    // Estimate memory usage of chunk
    const chunkMemory = this.estimateMemoryUsage(chunk);
    
    // Check if we need to clear memory
    if (this.currentMemoryUsage + chunkMemory > this.memoryLimit) {
      await this.clearOldChunks();
    }
    
    // Process the chunk
    const processedChunk = await this.processChunk(chunk);
    
    // Update memory tracking
    this.currentMemoryUsage += chunkMemory;
    this.processedChunks.push({
      data: processedChunk,
      memoryUsage: chunkMemory,
      timestamp: Date.now()
    });
  }
  
  private async handleMemoryPressure(usage: MemoryUsage): Promise<void> {
    console.warn(`Memory pressure detected: ${usage.usedMB}MB / ${usage.limitMB}MB`);
    
    // Force garbage collection
    if (global.gc) {
      global.gc();
    }
    
    // Clear oldest chunks
    await this.clearOldChunks();
    
    // If still under pressure, reduce chunk size
    if (usage.usedMB > usage.limitMB * 0.9) {
      throw new Error('Out of memory - consider reducing chunk size or increasing memory limit');
    }
  }
  
  private async clearOldChunks(): Promise<void> {
    // Keep only the most recent chunks
    const keepCount = Math.floor(this.processedChunks.length / 2);
    const removedChunks = this.processedChunks.splice(0, this.processedChunks.length - keepCount);
    
    // Update memory usage
    const freedMemory = removedChunks.reduce((sum, chunk) => sum + chunk.memoryUsage, 0);
    this.currentMemoryUsage -= freedMemory;
    
    console.log(`Cleared ${removedChunks.length} chunks, freed ${freedMemory / 1024 / 1024}MB`);
  }
  
  private estimateMemoryUsage(chunk: any): number {
    // Rough estimation of memory usage
    const jsonSize = JSON.stringify(chunk).length;
    return jsonSize * 2; // Factor for object overhead
  }
  
  private async processChunk(chunk: any): Promise<any> {
    // Implement your chunk processing logic here
    return {
      releases: chunk.releases?.map((release: any) => ({
        id: release.id,
        title: release.title,
        processed: true
      })) || [],
      metadata: {
        chunkId: chunk.id,
        processedAt: new Date().toISOString()
      }
    };
  }
}
```

## Advanced Streaming Patterns

### Parallel Stream Processing

```typescript
import { Worker, isMainThread, parentPort, workerData } from 'worker_threads';
import { pipeline } from 'stream/promises';

class ParallelStreamProcessor {
  private workerPool: Worker[] = [];
  private maxWorkers: number;
  private activeJobs = new Map<number, Promise<any>>();
  
  constructor(maxWorkers: number = 4) {
    this.maxWorkers = maxWorkers;
    this.initializeWorkerPool();
  }
  
  private initializeWorkerPool() {
    for (let i = 0; i < this.maxWorkers; i++) {
      const worker = new Worker(__filename, {
        workerData: { workerId: i }
      });
      
      worker.on('error', (error) => {
        console.error(`Worker ${i} error:`, error);
        this.replaceWorker(i);
      });
      
      this.workerPool.push(worker);
    }
  }
  
  async processLargeFileParallel(filePath: string): Promise<ProcessingResult> {
    const fileSize = fs.statSync(filePath).size;
    const chunkSize = 1024 * 1024; // 1MB chunks
    const chunks = Math.ceil(fileSize / chunkSize);
    
    console.log(`Processing ${chunks} chunks in parallel with ${this.maxWorkers} workers`);
    
    const results: ChunkResult[] = [];
    const promises: Promise<ChunkResult>[] = [];
    
    for (let i = 0; i < chunks; i++) {
      const start = i * chunkSize;
      const end = Math.min(start + chunkSize, fileSize);
      
      const promise = this.processChunkInWorker(filePath, start, end, i);
      promises.push(promise);
      
      // Limit concurrent processing
      if (promises.length >= this.maxWorkers) {
        const completed = await Promise.race(promises);
        results.push(completed);
        promises.splice(promises.findIndex(p => p === Promise.resolve(completed)), 1);
      }
    }
    
    // Wait for remaining chunks
    const remainingResults = await Promise.all(promises);
    results.push(...remainingResults);
    
    return this.combineResults(results);
  }
  
  private async processChunkInWorker(
    filePath: string,
    start: number,
    end: number,
    chunkId: number
  ): Promise<ChunkResult> {
    const availableWorker = await this.getAvailableWorker();
    
    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        reject(new Error(`Worker timeout for chunk ${chunkId}`));
      }, 60000); // 60 second timeout
      
      availableWorker.postMessage({
        type: 'processChunk',
        filePath,
        start,
        end,
        chunkId
      });
      
      const messageHandler = (result: any) => {
        clearTimeout(timeoutId);
        availableWorker.off('message', messageHandler);
        
        if (result.error) {
          reject(new Error(result.error));
        } else {
          resolve(result);
        }
      };
      
      availableWorker.on('message', messageHandler);
    });
  }
  
  private async getAvailableWorker(): Promise<Worker> {
    // Simple round-robin worker selection
    // In production, you might want more sophisticated load balancing
    const workerId = this.activeJobs.size % this.maxWorkers;
    return this.workerPool[workerId];
  }
  
  private combineResults(results: ChunkResult[]): ProcessingResult {
    // Sort results by chunk ID to maintain order
    results.sort((a, b) => a.chunkId - b.chunkId);
    
    const totalReleases = results.reduce((sum, result) => sum + result.releaseCount, 0);
    const totalErrors = results.reduce((sum, result) => sum + result.errorCount, 0);
    
    return {
      totalReleases,
      totalErrors,
      processingTimeMs: Math.max(...results.map(r => r.processingTimeMs)),
      chunks: results.length,
      throughputMBps: results.reduce((sum, result) => sum + result.throughputMBps, 0) / results.length
    };
  }
  
  async shutdown() {
    await Promise.all(this.workerPool.map(worker => worker.terminate()));
  }
}

// Worker thread code
if (!isMainThread) {
  parentPort?.on('message', async (message) => {
    const { type, filePath, start, end, chunkId } = message;
    
    if (type === 'processChunk') {
      try {
        const startTime = Date.now();
        
        // Read chunk from file
        const buffer = Buffer.alloc(end - start);
        const fd = fs.openSync(filePath, 'r');
        fs.readSync(fd, buffer, 0, end - start, start);
        fs.closeSync(fd);
        
        // Convert buffer to string and parse
        const xmlChunk = buffer.toString('utf8');
        const parser = new StreamingDDEXParser();
        const result = await parser.parseXmlChunk(xmlChunk, chunkId);
        
        const processingTime = Date.now() - startTime;
        const chunkSizeMB = (end - start) / 1024 / 1024;
        const throughputMBps = chunkSizeMB / (processingTime / 1000);
        
        parentPort?.postMessage({
          chunkId,
          releaseCount: result.releases.length,
          errorCount: result.errors.length,
          processingTimeMs: processingTime,
          throughputMBps,
          memoryUsage: process.memoryUsage().heapUsed / 1024 / 1024
        });
        
      } catch (error) {
        parentPort?.postMessage({
          chunkId,
          error: error.message,
          releaseCount: 0,
          errorCount: 1,
          processingTimeMs: 0,
          throughputMBps: 0
        });
      }
    }
  });
}
```

### Python Streaming Implementation

```python
import asyncio
import aiofiles
from typing import AsyncIterator, Callable, Dict, Any, Optional
from dataclasses import dataclass
from ddex_parser import StreamingDDEXParser
import psutil
import gc

@dataclass
class StreamConfig:
    chunk_size_mb: int = 64
    max_memory_mb: int = 500
    parallel_workers: int = 4
    enable_progress: bool = True
    buffer_size_kb: int = 256

class MemoryMonitor:
    def __init__(self, max_memory_mb: int):
        self.max_memory_mb = max_memory_mb
        self.process = psutil.Process()
    
    def get_memory_usage(self) -> Dict[str, float]:
        memory_info = self.process.memory_info()
        return {
            'rss_mb': memory_info.rss / 1024 / 1024,
            'vms_mb': memory_info.vms / 1024 / 1024,
            'percent': self.process.memory_percent()
        }
    
    def is_memory_pressure(self) -> bool:
        usage = self.get_memory_usage()
        return usage['rss_mb'] > self.max_memory_mb * 0.85
    
    def force_cleanup(self):
        gc.collect()
        # Additional cleanup strategies can be added here

class AsyncStreamingProcessor:
    def __init__(self, config: StreamConfig):
        self.config = config
        self.memory_monitor = MemoryMonitor(config.max_memory_mb)
        self.parser = StreamingDDEXParser()
        self.processed_chunks = 0
        self.total_releases = 0
        
    async def process_large_file(
        self,
        file_path: str,
        chunk_processor: Optional[Callable] = None
    ) -> Dict[str, Any]:
        """Process large DDEX file using async streaming"""
        
        file_size = await self._get_file_size(file_path)
        chunk_size = self.config.chunk_size_mb * 1024 * 1024
        total_chunks = (file_size + chunk_size - 1) // chunk_size
        
        print(f"Processing {file_size / 1024 / 1024:.1f}MB file in {total_chunks} chunks")
        
        results = []
        semaphore = asyncio.Semaphore(self.config.parallel_workers)
        
        async def process_chunk_worker(chunk_id: int, start: int, end: int):
            async with semaphore:
                return await self._process_chunk(
                    file_path, chunk_id, start, end, chunk_processor
                )
        
        # Create tasks for all chunks
        tasks = []
        for i in range(total_chunks):
            start = i * chunk_size
            end = min(start + chunk_size, file_size)
            task = asyncio.create_task(process_chunk_worker(i, start, end))
            tasks.append(task)
        
        # Process chunks and handle memory pressure
        completed_tasks = []
        while tasks or completed_tasks:
            # Wait for some tasks to complete
            if len(completed_tasks) >= self.config.parallel_workers or not tasks:
                if completed_tasks:
                    done, pending = await asyncio.wait(
                        completed_tasks,
                        return_when=asyncio.FIRST_COMPLETED
                    )
                    
                    for task in done:
                        result = await task
                        results.append(result)
                        self._update_progress(len(results), total_chunks)
                    
                    completed_tasks = list(pending)
            
            # Add more tasks if available and memory allows
            if tasks and len(completed_tasks) < self.config.parallel_workers:
                if not self.memory_monitor.is_memory_pressure():
                    task = tasks.pop(0)
                    completed_tasks.append(task)
                else:
                    print("Memory pressure detected, waiting...")
                    await asyncio.sleep(1)
                    self.memory_monitor.force_cleanup()
        
        return self._combine_results(results)
    
    async def _process_chunk(
        self,
        file_path: str,
        chunk_id: int,
        start: int,
        end: int,
        processor: Optional[Callable] = None
    ) -> Dict[str, Any]:
        """Process a single chunk of the file"""
        
        try:
            start_time = asyncio.get_event_loop().time()
            
            # Read chunk asynchronously
            chunk_data = await self._read_chunk_async(file_path, start, end)
            
            # Parse chunk
            parse_result = await self.parser.parse_chunk(chunk_data, chunk_id)
            
            # Apply custom processor if provided
            if processor:
                parse_result = await processor(parse_result)
            
            processing_time = asyncio.get_event_loop().time() - start_time
            chunk_size_mb = (end - start) / 1024 / 1024
            throughput = chunk_size_mb / processing_time if processing_time > 0 else 0
            
            self.processed_chunks += 1
            self.total_releases += len(parse_result.get('releases', []))
            
            return {
                'chunk_id': chunk_id,
                'releases': parse_result.get('releases', []),
                'errors': parse_result.get('errors', []),
                'processing_time': processing_time,
                'throughput_mbps': throughput,
                'memory_usage': self.memory_monitor.get_memory_usage()
            }
            
        except Exception as e:
            print(f"Error processing chunk {chunk_id}: {e}")
            return {
                'chunk_id': chunk_id,
                'releases': [],
                'errors': [str(e)],
                'processing_time': 0,
                'throughput_mbps': 0,
                'memory_usage': self.memory_monitor.get_memory_usage()
            }
    
    async def _read_chunk_async(self, file_path: str, start: int, end: int) -> str:
        """Read file chunk asynchronously"""
        
        async with aiofiles.open(file_path, 'rb') as file:
            await file.seek(start)
            chunk_bytes = await file.read(end - start)
            return chunk_bytes.decode('utf-8', errors='ignore')
    
    async def _get_file_size(self, file_path: str) -> int:
        """Get file size asynchronously"""
        import aiofiles.os
        stat = await aiofiles.os.stat(file_path)
        return stat.st_size
    
    def _update_progress(self, completed: int, total: int):
        """Update progress display"""
        if self.config.enable_progress:
            percent = (completed / total) * 100
            memory_usage = self.memory_monitor.get_memory_usage()
            print(f"Progress: {completed}/{total} ({percent:.1f}%) - "
                  f"Memory: {memory_usage['rss_mb']:.1f}MB - "
                  f"Releases: {self.total_releases}")
    
    def _combine_results(self, results: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Combine results from all chunks"""
        
        # Sort by chunk_id to maintain order
        results.sort(key=lambda x: x['chunk_id'])
        
        total_releases = sum(len(r['releases']) for r in results)
        total_errors = sum(len(r['errors']) for r in results)
        total_time = max(r['processing_time'] for r in results)
        avg_throughput = sum(r['throughput_mbps'] for r in results) / len(results)
        
        return {
            'total_releases': total_releases,
            'total_errors': total_errors,
            'total_chunks': len(results),
            'processing_time': total_time,
            'average_throughput_mbps': avg_throughput,
            'peak_memory_mb': max(r['memory_usage']['rss_mb'] for r in results),
            'all_releases': [release for r in results for release in r['releases']]
        }

# Usage example
async def process_large_catalog_example():
    """Example usage of streaming processor"""
    
    config = StreamConfig(
        chunk_size_mb=32,      # 32MB chunks
        max_memory_mb=512,     # 512MB memory limit
        parallel_workers=6,    # 6 concurrent workers
        enable_progress=True
    )
    
    processor = AsyncStreamingProcessor(config)
    
    # Custom chunk processor
    async def custom_processor(chunk_result):
        # Example: filter releases by genre
        filtered_releases = [
            release for release in chunk_result.get('releases', [])
            if release.get('genre', '').lower() in ['pop', 'rock', 'electronic']
        ]
        
        return {
            'releases': filtered_releases,
            'errors': chunk_result.get('errors', [])
        }
    
    # Process the large file
    results = await processor.process_large_file(
        'very-large-catalog.xml',
        chunk_processor=custom_processor
    )
    
    print(f"Processing complete:")
    print(f"  Total releases: {results['total_releases']}")
    print(f"  Total errors: {results['total_errors']}")
    print(f"  Processing time: {results['processing_time']:.2f}s")
    print(f"  Average throughput: {results['average_throughput_mbps']:.2f} MB/s")
    print(f"  Peak memory usage: {results['peak_memory_mb']:.1f}MB")
    
    return results

# Run the example
if __name__ == "__main__":
    results = asyncio.run(process_large_catalog_example())
```

## Production-Ready Streaming Pipeline

### Enterprise Streaming Architecture

```typescript
import { EventEmitter } from 'events';
import { pipeline, Transform, Writable } from 'stream';
import { promisify } from 'util';

interface PipelineConfig {
  inputPath: string;
  outputPath: string;
  transformers: StreamTransformer[];
  maxConcurrency: number;
  checkpointInterval: number;
  retryConfig: RetryConfig;
}

interface RetryConfig {
  maxRetries: number;
  retryDelayMs: number;
  exponentialBackoff: boolean;
}

class StreamingPipeline extends EventEmitter {
  private config: PipelineConfig;
  private checkpointManager: CheckpointManager;
  private metrics: PipelineMetrics;
  
  constructor(config: PipelineConfig) {
    super();
    this.config = config;
    this.checkpointManager = new CheckpointManager();
    this.metrics = new PipelineMetrics();
  }
  
  async execute(): Promise<PipelineResult> {
    const pipelineAsync = promisify(pipeline);
    
    try {
      // Create input stream
      const inputStream = fs.createReadStream(this.config.inputPath, {
        highWaterMark: 1024 * 1024 // 1MB buffer
      });
      
      // Create transform streams
      const transformStreams = this.config.transformers.map(
        transformer => this.createTransformStream(transformer)
      );
      
      // Create output stream
      const outputStream = fs.createWriteStream(this.config.outputPath);
      
      // Add monitoring
      const monitoringStream = this.createMonitoringStream();
      
      // Create checkpoint stream
      const checkpointStream = this.createCheckpointStream();
      
      // Build pipeline
      const streams = [
        inputStream,
        ...transformStreams,
        checkpointStream,
        monitoringStream,
        outputStream
      ];
      
      // Execute pipeline
      await pipelineAsync(...streams);
      
      return this.metrics.getResults();
      
    } catch (error) {
      this.emit('error', error);
      throw error;
    }
  }
  
  private createTransformStream(transformer: StreamTransformer): Transform {
    return new Transform({
      objectMode: true,
      transform: async (chunk, encoding, callback) => {
        try {
          const startTime = Date.now();
          const transformed = await transformer.transform(chunk);
          
          this.metrics.recordTransformation(
            transformer.name,
            Date.now() - startTime
          );
          
          callback(null, transformed);
        } catch (error) {
          if (await this.shouldRetry(error, transformer)) {
            // Retry logic
            setTimeout(() => {
              this.createTransformStream(transformer).transform(chunk, encoding, callback);
            }, this.config.retryConfig.retryDelayMs);
          } else {
            callback(error);
          }
        }
      }
    });
  }
  
  private createMonitoringStream(): Transform {
    let processedCount = 0;
    
    return new Transform({
      objectMode: true,
      transform: (chunk, encoding, callback) => {
        processedCount++;
        
        if (processedCount % 1000 === 0) {
          this.emit('progress', {
            processed: processedCount,
            memoryUsage: process.memoryUsage(),
            timestamp: Date.now()
          });
        }
        
        callback(null, chunk);
      }
    });
  }
  
  private createCheckpointStream(): Transform {
    let chunkCount = 0;
    
    return new Transform({
      objectMode: true,
      transform: async (chunk, encoding, callback) => {
        chunkCount++;
        
        if (chunkCount % this.config.checkpointInterval === 0) {
          await this.checkpointManager.saveCheckpoint({
            chunkCount,
            position: chunkCount * chunk.length,
            timestamp: Date.now()
          });
        }
        
        callback(null, chunk);
      }
    });
  }
  
  private async shouldRetry(error: Error, transformer: StreamTransformer): Promise<boolean> {
    // Implement retry logic based on error type and configuration
    return transformer.retryCount < this.config.retryConfig.maxRetries;
  }
}

class CheckpointManager {
  private checkpoints = new Map<string, Checkpoint>();
  
  async saveCheckpoint(checkpoint: Checkpoint): Promise<void> {
    this.checkpoints.set(checkpoint.timestamp.toString(), checkpoint);
    
    // Persist to disk for recovery
    await fs.promises.writeFile(
      'pipeline-checkpoint.json',
      JSON.stringify(Array.from(this.checkpoints.entries()))
    );
  }
  
  async loadLastCheckpoint(): Promise<Checkpoint | null> {
    try {
      const data = await fs.promises.readFile('pipeline-checkpoint.json', 'utf8');
      const checkpoints = new Map(JSON.parse(data));
      
      if (checkpoints.size === 0) return null;
      
      // Get the latest checkpoint
      const latestKey = Math.max(...Array.from(checkpoints.keys()).map(Number));
      return checkpoints.get(latestKey.toString());
    } catch {
      return null;
    }
  }
}

class PipelineMetrics {
  private startTime = Date.now();
  private transformationTimes = new Map<string, number[]>();
  private errorCounts = new Map<string, number>();
  
  recordTransformation(transformerName: string, duration: number): void {
    if (!this.transformationTimes.has(transformerName)) {
      this.transformationTimes.set(transformerName, []);
    }
    this.transformationTimes.get(transformerName)!.push(duration);
  }
  
  recordError(transformerName: string): void {
    const current = this.errorCounts.get(transformerName) || 0;
    this.errorCounts.set(transformerName, current + 1);
  }
  
  getResults(): PipelineResult {
    const totalTime = Date.now() - this.startTime;
    
    const transformerStats = new Map<string, TransformerStats>();
    for (const [name, times] of this.transformationTimes) {
      transformerStats.set(name, {
        totalCalls: times.length,
        averageTime: times.reduce((a, b) => a + b, 0) / times.length,
        minTime: Math.min(...times),
        maxTime: Math.max(...times),
        errors: this.errorCounts.get(name) || 0
      });
    }
    
    return {
      totalProcessingTime: totalTime,
      transformerStats,
      totalErrors: Array.from(this.errorCounts.values()).reduce((a, b) => a + b, 0)
    };
  }
}
```

## Performance Optimization Techniques

### Memory Pool Management

```typescript
class MemoryPool<T> {
  private pool: T[] = [];
  private factory: () => T;
  private reset: (item: T) => void;
  private maxSize: number;
  
  constructor(
    factory: () => T,
    reset: (item: T) => void,
    maxSize: number = 100
  ) {
    this.factory = factory;
    this.reset = reset;
    this.maxSize = maxSize;
    
    // Pre-populate pool
    for (let i = 0; i < Math.min(10, maxSize); i++) {
      this.pool.push(factory());
    }
  }
  
  acquire(): T {
    if (this.pool.length > 0) {
      return this.pool.pop()!;
    }
    return this.factory();
  }
  
  release(item: T): void {
    if (this.pool.length < this.maxSize) {
      this.reset(item);
      this.pool.push(item);
    }
    // Otherwise let it be garbage collected
  }
  
  drain(): void {
    this.pool.length = 0;
  }
}

// Usage in streaming processor
class OptimizedStreamProcessor {
  private bufferPool: MemoryPool<Buffer>;
  private objectPool: MemoryPool<ProcessingContext>;
  
  constructor() {
    this.bufferPool = new MemoryPool(
      () => Buffer.alloc(64 * 1024), // 64KB buffers
      (buffer) => buffer.fill(0),    // Reset buffer
      50                             // Max 50 buffers in pool
    );
    
    this.objectPool = new MemoryPool(
      () => ({ releases: [], errors: [], metadata: {} }),
      (obj) => {
        obj.releases.length = 0;
        obj.errors.length = 0;
        obj.metadata = {};
      },
      100
    );
  }
  
  async processChunk(chunkData: string): Promise<any> {
    const context = this.objectPool.acquire();
    const buffer = this.bufferPool.acquire();
    
    try {
      // Use pooled objects for processing
      const result = await this.parseWithPooledResources(chunkData, context, buffer);
      return result;
    } finally {
      // Return objects to pool
      this.objectPool.release(context);
      this.bufferPool.release(buffer);
    }
  }
  
  private async parseWithPooledResources(
    data: string,
    context: ProcessingContext,
    buffer: Buffer
  ): Promise<any> {
    // Implement parsing using pooled resources
    // This reduces garbage collection pressure
    return context;
  }
  
  shutdown(): void {
    this.bufferPool.drain();
    this.objectPool.drain();
  }
}
```

### Adaptive Chunk Sizing

```python
import time
import statistics
from typing import List, Tuple

class AdaptiveChunkSizer:
    """Dynamically adjust chunk sizes based on performance metrics"""
    
    def __init__(self, initial_chunk_size: int = 1024 * 1024):  # 1MB
        self.current_chunk_size = initial_chunk_size
        self.min_chunk_size = 256 * 1024  # 256KB
        self.max_chunk_size = 10 * 1024 * 1024  # 10MB
        self.performance_history: List[Tuple[int, float, float]] = []  # (size, time, throughput)
        self.adjustment_threshold = 5  # Number of samples before adjustment
        
    def get_next_chunk_size(self) -> int:
        """Get the next optimal chunk size"""
        
        if len(self.performance_history) >= self.adjustment_threshold:
            self._adjust_chunk_size()
        
        return self.current_chunk_size
    
    def record_performance(self, chunk_size: int, processing_time: float, data_size: int):
        """Record performance metrics for a processed chunk"""
        
        throughput = data_size / processing_time if processing_time > 0 else 0
        self.performance_history.append((chunk_size, processing_time, throughput))
        
        # Keep only recent history
        if len(self.performance_history) > 20:
            self.performance_history = self.performance_history[-20:]
    
    def _adjust_chunk_size(self):
        """Adjust chunk size based on performance history"""
        
        recent_samples = self.performance_history[-self.adjustment_threshold:]
        current_throughput = statistics.mean([sample[2] for sample in recent_samples])
        
        # Try to find better performance with different chunk sizes
        if len(self.performance_history) >= 10:
            older_samples = self.performance_history[-10:-self.adjustment_threshold]
            older_throughput = statistics.mean([sample[2] for sample in older_samples])
            
            improvement_ratio = current_throughput / older_throughput if older_throughput > 0 else 1
            
            if improvement_ratio < 0.95:  # Performance degraded
                # Revert to previous size or try smaller chunks
                self.current_chunk_size = max(
                    int(self.current_chunk_size * 0.8),
                    self.min_chunk_size
                )
            elif improvement_ratio > 1.1:  # Performance improved
                # Try larger chunks
                self.current_chunk_size = min(
                    int(self.current_chunk_size * 1.2),
                    self.max_chunk_size
                )
        
        print(f"Adjusted chunk size to {self.current_chunk_size / 1024 / 1024:.1f}MB")

class IntelligentStreamProcessor:
    """Stream processor with adaptive optimization"""
    
    def __init__(self):
        self.chunk_sizer = AdaptiveChunkSizer()
        self.parser = StreamingDDEXParser()
        
    async def process_with_adaptation(self, file_path: str) -> Dict[str, Any]:
        """Process file with adaptive chunk sizing"""
        
        file_size = await self._get_file_size(file_path)
        processed_bytes = 0
        results = []
        
        while processed_bytes < file_size:
            chunk_size = self.chunk_sizer.get_next_chunk_size()
            start_pos = processed_bytes
            end_pos = min(start_pos + chunk_size, file_size)
            
            # Process chunk and measure performance
            start_time = time.time()
            chunk_result = await self._process_chunk(file_path, start_pos, end_pos)
            processing_time = time.time() - start_time
            
            # Record performance for adaptation
            actual_chunk_size = end_pos - start_pos
            self.chunk_sizer.record_performance(
                actual_chunk_size,
                processing_time,
                actual_chunk_size
            )
            
            results.append(chunk_result)
            processed_bytes = end_pos
            
            # Progress reporting
            progress = (processed_bytes / file_size) * 100
            throughput = (actual_chunk_size / 1024 / 1024) / processing_time
            print(f"Progress: {progress:.1f}% - Throughput: {throughput:.2f} MB/s")
        
        return self._combine_results(results)
```

## Common Pitfalls and Solutions

### 1. XML Boundary Issues

**Pitfall**: Chunk boundaries split XML elements

```typescript
// DON'T - Raw chunk splitting
const chunk = fileContent.substring(start, end); // May split XML elements

// DO - Find safe chunk boundaries
function findSafeChunkBoundary(content: string, idealEnd: number): number {
  // Look for complete XML element boundary
  let safeEnd = idealEnd;
  
  // Find the next closing tag
  while (safeEnd < content.length && content[safeEnd] !== '>') {
    safeEnd++;
  }
  
  // Ensure we're not in the middle of a CDATA section
  const cdataStart = content.lastIndexOf('<![CDATA[', safeEnd);
  const cdataEnd = content.indexOf(']]>', cdataStart);
  
  if (cdataStart !== -1 && cdataEnd > safeEnd) {
    safeEnd = cdataEnd + 3;
  }
  
  return safeEnd + 1;
}
```

### 2. Memory Leaks in Streaming

**Pitfall**: Not properly cleaning up stream resources

```python
# DON'T - Potential memory leaks
async def bad_streaming():
    parser = StreamingDDEXParser()
    while True:
        chunk = await read_chunk()
        result = await parser.parse(chunk)  # Accumulates in memory
        # No cleanup

# DO - Proper resource management
async def good_streaming():
    parser = StreamingDDEXParser()
    try:
        async with parser.create_session() as session:
            while True:
                chunk = await read_chunk()
                result = await session.parse_chunk(chunk)
                await process_result(result)
                session.clear_cache()  # Regular cleanup
    finally:
        await parser.cleanup()
```

### 3. Backpressure Handling

**Pitfall**: Not handling slow consumers

```typescript
// DON'T - Unbounded queuing
const queue: any[] = [];
stream.on('data', (chunk) => {
  queue.push(chunk); // Queue grows indefinitely
});

// DO - Implement backpressure
const queue = new BoundedQueue(maxSize);
stream.on('data', (chunk) => {
  if (!queue.tryEnqueue(chunk)) {
    stream.pause(); // Apply backpressure
    queue.onSpace(() => stream.resume());
  }
});
```

## Performance Considerations

1. **Chunk Size Optimization**: Balance between memory usage and I/O efficiency
2. **Parallel Processing**: Use appropriate worker count based on CPU and I/O characteristics
3. **Memory Monitoring**: Implement real-time memory usage tracking
4. **Error Recovery**: Design for partial failures and resumption
5. **Progress Tracking**: Provide meaningful progress indicators for long-running operations

## Links to API Documentation

- [Streaming Parser API](/api/parser/streaming)
- [Memory Management](/api/parser/memory-management)
- [Python Async Streaming](/api/parser/python#async-streaming)
- [Performance Monitoring](/api/monitoring/performance)
- [Error Recovery](/api/error-handling/streaming)

This comprehensive guide enables efficient processing of gigabyte-sized DDEX files using advanced streaming techniques, ensuring scalability and reliability for enterprise music catalog operations.