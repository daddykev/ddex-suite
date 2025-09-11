# Performance Tuning for Large DDEX Catalogs

Learn how to optimize the DDEX Suite for high-performance processing of large catalogs, streaming operations, and memory-efficient workflows.

## Problem Statement

Processing large DDEX catalogs presents several performance challenges:

- **Memory Constraints**: Large XML files can consume excessive memory during parsing
- **Processing Speed**: Sequential processing doesn't scale for thousands of files
- **I/O Bottlenecks**: Disk and network operations become limiting factors
- **Memory Leaks**: Accumulating data structures lead to out-of-memory errors
- **CPU Utilization**: Single-threaded processing underutilizes modern hardware
- **Scalability**: Performance degrades non-linearly with catalog size

Without proper optimization, processing a 100MB DDEX file might consume 1GB+ of memory and take several minutes, making large-scale operations impractical.

## Solution Approach

The DDEX Suite provides multiple optimization strategies including streaming parsers, memory-efficient builders, parallel processing, and intelligent caching to handle catalogs of any size efficiently.

### Performance Targets

| Catalog Size | Target Processing Time | Memory Usage | Throughput |
|--------------|----------------------|--------------|------------|
| 10MB | &lt;5 seconds | &lt;50MB | 2MB/s |
| 100MB | &lt;30 seconds | &lt;200MB | 3MB/s |
| 1GB | &lt;5 minutes | &lt;500MB | 3.5MB/s |
| 10GB+ | &lt;30 minutes | &lt;1GB | 6MB/s |

## Memory Optimization

### Streaming Parser Configuration

```typescript
import { DDEXParser } from 'ddex-parser';
import { createReadStream } from 'fs';

class MemoryEfficientParser {
  private parser: DDEXParser;

  constructor() {
    this.parser = new DDEXParser();
  }

  async parseHugeFile(filePath: string): Promise<void> {
    // Configure for memory efficiency
    const options = {
      streaming: true,           // Enable streaming mode
      includeRawExtensions: false, // Skip raw XML to save memory
      validateReferences: false, // Skip validation for speed
      includeComments: false     // Skip comments
    };

    console.log(`Starting streaming parse of ${filePath}...`);
    
    // Use streaming to process file in chunks
    const stream = createReadStream(filePath, { 
      encoding: 'utf-8',
      highWaterMark: 64 * 1024 // 64KB chunks
    });

    let processedReleases = 0;
    let processedTracks = 0;

    try {
      for await (const chunk of this.parser.streamParse(stream, options)) {
        // Process each chunk immediately
        await this.processChunk(chunk);
        
        // Track progress
        processedReleases += chunk.releases?.length || 0;
        processedTracks += chunk.soundRecordings?.length || 0;
        
        // Report progress
        if (processedReleases % 100 === 0) {
          console.log(`Processed ${processedReleases} releases, ${processedTracks} tracks`);
          
          // Force garbage collection if available
          if (global.gc) {
            global.gc();
          }
        }
      }
      
      console.log(`Completed: ${processedReleases} releases, ${processedTracks} tracks`);
      
    } catch (error) {
      console.error('Streaming parse failed:', error);
      throw error;
    }
  }

  private async processChunk(chunk: any): Promise<void> {
    // Process chunk data immediately and discard
    if (chunk.releases) {
      for (const release of chunk.releases) {
        await this.processRelease(release);
      }
    }
    
    if (chunk.soundRecordings) {
      for (const track of chunk.soundRecordings) {
        await this.processTrack(track);
      }
    }
    
    // Chunk data can now be garbage collected
  }

  private async processRelease(release: any): Promise<void> {
    // Example: Store to database immediately
    await this.database.insertRelease({
      releaseId: release.releaseId,
      title: release.title,
      artist: release.displayArtist,
      releaseDate: release.releaseDate
    });
  }

  private async processTrack(track: any): Promise<void> {
    // Example: Process track metadata
    await this.processAudioMetadata(track);
  }

  private database = {
    async insertRelease(data: any) {
      // Database insertion logic
    }
  };

  private async processAudioMetadata(track: any) {
    // Audio processing logic
  }
}

// Usage with memory monitoring
async function processWithMemoryMonitoring(filePath: string) {
  const parser = new MemoryEfficientParser();
  
  // Monitor memory usage
  const startMemory = process.memoryUsage();
  console.log(`Initial memory: ${Math.round(startMemory.heapUsed / 1024 / 1024)}MB`);
  
  const memoryMonitor = setInterval(() => {
    const usage = process.memoryUsage();
    console.log(`Memory: ${Math.round(usage.heapUsed / 1024 / 1024)}MB`);
  }, 5000);
  
  try {
    await parser.parseHugeFile(filePath);
  } finally {
    clearInterval(memoryMonitor);
    
    const endMemory = process.memoryUsage();
    console.log(`Final memory: ${Math.round(endMemory.heapUsed / 1024 / 1024)}MB`);
    console.log(`Memory increase: ${Math.round((endMemory.heapUsed - startMemory.heapUsed) / 1024 / 1024)}MB`);
  }
}
```

### Streaming Builder for Large Outputs

```typescript
import { StreamingDdexBuilder } from 'ddex-builder';

class HighThroughputBuilder {
  private builder: StreamingDdexBuilder;

  constructor() {
    // Configure for high throughput
    const config = {
      maxBufferSize: 50 * 1024 * 1024,  // 50MB buffer
      deterministic: true,               // Maintain deterministic output
      validateDuringStream: false,       // Validate at end for speed
      progressCallbackFrequency: 1000    // Progress every 1000 items
    };

    this.builder = new StreamingDdexBuilder(config);
    
    // Set up progress monitoring
    this.builder.setProgressCallback(this.onProgress.bind(this));
  }

  async buildLargeCatalog(catalogData: CatalogData): Promise<string> {
    const startTime = Date.now();
    
    // Set estimated total for accurate progress
    const totalItems = catalogData.releases.length + catalogData.tracks.length;
    this.builder.setEstimatedTotal(totalItems);

    console.log(`Building catalog with ${catalogData.releases.length} releases, ${catalogData.tracks.length} tracks...`);

    // Start message
    this.builder.startMessage({
      messageId: `CATALOG_${Date.now()}`,
      messageSenderName: catalogData.senderName,
      messageRecipientName: catalogData.recipientName,
      messageCreatedDateTime: new Date().toISOString()
    }, '4.3');

    // Stream resources (tracks) first
    console.log('Writing resources...');
    const resourceRefs: string[] = [];
    
    for (const track of catalogData.tracks) {
      const ref = this.builder.writeResource(
        track.resourceId,
        track.title,
        track.artist,
        track.isrc,
        track.duration,
        track.filePath
      );
      resourceRefs.push(ref);
      
      // Periodic memory cleanup
      if (resourceRefs.length % 1000 === 0) {
        if (global.gc) global.gc();
      }
    }

    // Transition to releases
    this.builder.finishResourcesStartReleases();

    // Stream releases
    console.log('Writing releases...');
    const releaseResourceMap = this.createReleaseResourceMap(catalogData);
    
    for (const release of catalogData.releases) {
      const releaseResourceRefs = releaseResourceMap.get(release.releaseId) || [];
      
      this.builder.writeRelease(
        release.releaseId,
        release.title,
        release.artist,
        release.label,
        release.upc,
        release.releaseDate,
        release.genre,
        releaseResourceRefs
      );
    }

    // Finalize message
    console.log('Finalizing message...');
    const stats = this.builder.finishMessage();
    
    const buildTime = Date.now() - startTime;
    console.log(`Build completed in ${buildTime}ms`);
    console.log(`Releases: ${stats.releasesWritten}, Resources: ${stats.resourcesWritten}`);
    console.log(`Output size: ${stats.bytesWritten} bytes`);
    console.log(`Peak memory: ${Math.round(stats.peakMemoryUsage / 1024 / 1024)}MB`);

    return this.builder.getXml();
  }

  private onProgress(progress: any): void {
    const percent = progress.estimatedCompletionPercent || 0;
    const memoryMB = Math.round(progress.currentMemoryUsage / 1024 / 1024);
    
    console.log(`Progress: ${percent.toFixed(1)}% | Memory: ${memoryMB}MB | Items: ${progress.releasesWritten + progress.resourcesWritten}`);
  }

  private createReleaseResourceMap(catalogData: CatalogData): Map<string, string[]> {
    const map = new Map<string, string[]>();
    
    // Group tracks by release
    for (const track of catalogData.tracks) {
      if (!map.has(track.releaseId)) {
        map.set(track.releaseId, []);
      }
      map.get(track.releaseId)!.push(track.resourceId);
    }
    
    return map;
  }
}

interface CatalogData {
  senderName: string;
  recipientName: string;
  releases: Release[];
  tracks: Track[];
}

interface Release {
  releaseId: string;
  title: string;
  artist: string;
  label: string;
  upc: string;
  releaseDate: string;
  genre: string;
}

interface Track {
  resourceId: string;
  releaseId: string;
  title: string;
  artist: string;
  isrc: string;
  duration: string;
  filePath?: string;
}
```

## Parallel Processing

### Concurrent File Processing

```typescript
import { Worker, isMainThread, parentPort, workerData } from 'worker_threads';
import { cpus } from 'os';

class ParallelDDEXProcessor {
  private maxWorkers: number;
  private activeWorkers: number = 0;

  constructor(maxWorkers?: number) {
    this.maxWorkers = maxWorkers || Math.min(cpus().length, 8);
  }

  async processBatch(filePaths: string[]): Promise<ProcessingResult[]> {
    const results: ProcessingResult[] = [];
    const chunks = this.chunkArray(filePaths, this.maxWorkers);
    
    console.log(`Processing ${filePaths.length} files using ${this.maxWorkers} workers...`);

    const workerPromises = chunks.map((chunk, index) => 
      this.processChunk(chunk, index)
    );

    const chunkResults = await Promise.all(workerPromises);
    
    // Flatten results
    for (const chunkResult of chunkResults) {
      results.push(...chunkResult);
    }

    return results;
  }

  private async processChunk(filePaths: string[], workerId: number): Promise<ProcessingResult[]> {
    return new Promise((resolve, reject) => {
      const worker = new Worker(__filename, {
        workerData: { filePaths, workerId }
      });

      this.activeWorkers++;
      console.log(`Worker ${workerId} started (${this.activeWorkers} active)`);

      worker.on('message', (results: ProcessingResult[]) => {
        this.activeWorkers--;
        console.log(`Worker ${workerId} completed (${this.activeWorkers} active)`);
        resolve(results);
      });

      worker.on('error', (error) => {
        this.activeWorkers--;
        console.error(`Worker ${workerId} error:`, error);
        reject(error);
      });

      worker.on('exit', (code) => {
        if (code !== 0) {
          this.activeWorkers--;
          reject(new Error(`Worker ${workerId} exited with code ${code}`));
        }
      });
    });
  }

  private chunkArray<T>(array: T[], chunkSize: number): T[][] {
    const chunks: T[][] = [];
    for (let i = 0; i < array.length; i += chunkSize) {
      chunks.push(array.slice(i, i + chunkSize));
    }
    return chunks;
  }
}

// Worker thread code
if (!isMainThread) {
  const { filePaths, workerId } = workerData;
  
  (async () => {
    const { DDEXParser } = await import('ddex-parser');
    const parser = new DDEXParser();
    const results: ProcessingResult[] = [];

    for (const filePath of filePaths) {
      try {
        const startTime = Date.now();
        
        const xmlContent = require('fs').readFileSync(filePath, 'utf-8');
        const parseResult = await parser.parse(xmlContent, {
          streaming: true,
          includeRawExtensions: false
        });

        const processingTime = Date.now() - startTime;
        
        results.push({
          filePath,
          success: true,
          processingTime,
          releaseCount: parseResult.flat.releases.length,
          trackCount: parseResult.flat.soundRecordings.length,
          fileSize: xmlContent.length
        });

        console.log(`Worker ${workerId}: Processed ${require('path').basename(filePath)} (${processingTime}ms)`);

      } catch (error) {
        results.push({
          filePath,
          success: false,
          error: error.message,
          processingTime: 0,
          releaseCount: 0,
          trackCount: 0,
          fileSize: 0
        });

        console.error(`Worker ${workerId}: Failed ${require('path').basename(filePath)}:`, error.message);
      }
    }

    parentPort?.postMessage(results);
  })();
}

interface ProcessingResult {
  filePath: string;
  success: boolean;
  processingTime: number;
  releaseCount: number;
  trackCount: number;
  fileSize: number;
  error?: string;
}

// Usage example
async function processLargeBatch() {
  const processor = new ParallelDDEXProcessor();
  
  // Get all DDEX files
  const filePaths = [
    '/path/to/catalog1.xml',
    '/path/to/catalog2.xml',
    // ... thousands of files
  ];

  const startTime = Date.now();
  const results = await processor.processBatch(filePaths);
  const totalTime = Date.now() - startTime;

  // Analyze results
  const successful = results.filter(r => r.success);
  const failed = results.filter(r => !r.success);
  const totalReleases = successful.reduce((sum, r) => sum + r.releaseCount, 0);
  const totalTracks = successful.reduce((sum, r) => sum + r.trackCount, 0);
  const avgProcessingTime = successful.reduce((sum, r) => sum + r.processingTime, 0) / successful.length;

  console.log('\n=== Processing Results ===');
  console.log(`Total time: ${totalTime}ms`);
  console.log(`Files processed: ${results.length}`);
  console.log(`Successful: ${successful.length}`);
  console.log(`Failed: ${failed.length}`);
  console.log(`Total releases: ${totalReleases}`);
  console.log(`Total tracks: ${totalTracks}`);
  console.log(`Average processing time: ${avgProcessingTime.toFixed(2)}ms`);
  console.log(`Throughput: ${(results.length / (totalTime / 1000)).toFixed(2)} files/sec`);
}
```

## Python High-Performance Workflows

```python
import asyncio
import aiofiles
import multiprocessing as mp
from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
from typing import List, Dict, Any
import time
import psutil
import pandas as pd
from ddex_parser import DDEXParser
from ddex_builder import DdexBuilder

class HighPerformanceDDEXProcessor:
    def __init__(self, max_workers: int = None):
        self.max_workers = max_workers or mp.cpu_count()
        self.parser = DDEXParser()
        
    async def process_large_catalog_async(self, file_paths: List[str]) -> Dict[str, Any]:
        """Process large catalogs using async I/O and multiprocessing"""
        
        print(f"Processing {len(file_paths)} files with {self.max_workers} workers...")
        
        # Split files into chunks for multiprocessing
        chunk_size = max(1, len(file_paths) // self.max_workers)
        file_chunks = [
            file_paths[i:i + chunk_size] 
            for i in range(0, len(file_paths), chunk_size)
        ]
        
        start_time = time.time()
        initial_memory = psutil.Process().memory_info().rss / 1024 / 1024  # MB
        
        # Process chunks in parallel
        with ProcessPoolExecutor(max_workers=self.max_workers) as executor:
            tasks = [
                self.process_chunk_in_process(chunk, chunk_idx)
                for chunk_idx, chunk in enumerate(file_chunks)
            ]
            
            chunk_results = await asyncio.gather(*tasks)
        
        # Combine results
        all_results = []
        for chunk_result in chunk_results:
            all_results.extend(chunk_result)
        
        processing_time = time.time() - start_time
        final_memory = psutil.Process().memory_info().rss / 1024 / 1024  # MB
        
        return {
            'results': all_results,
            'processing_time': processing_time,
            'memory_used': final_memory - initial_memory,
            'throughput': len(file_paths) / processing_time,
            'success_rate': len([r for r in all_results if r['success']]) / len(all_results) * 100
        }
    
    async def process_chunk_in_process(self, file_paths: List[str], chunk_idx: int) -> List[Dict[str, Any]]:
        """Process a chunk of files in a separate process"""
        loop = asyncio.get_event_loop()
        
        with ProcessPoolExecutor(max_workers=1) as executor:
            future = executor.submit(self._process_chunk_sync, file_paths, chunk_idx)
            return await loop.run_in_executor(None, future.result)
    
    @staticmethod
    def _process_chunk_sync(file_paths: List[str], chunk_idx: int) -> List[Dict[str, Any]]:
        """Synchronous processing function for multiprocessing"""
        parser = DDEXParser()
        results = []
        
        for file_idx, file_path in enumerate(file_paths):
            try:
                start_time = time.time()
                
                # Read file
                with open(file_path, 'r', encoding='utf-8') as f:
                    xml_content = f.read()
                
                # Parse with memory optimization
                result = parser.parse(xml_content)
                
                processing_time = time.time() - start_time
                
                results.append({
                    'file_path': file_path,
                    'chunk_idx': chunk_idx,
                    'file_idx': file_idx,
                    'success': True,
                    'processing_time': processing_time,
                    'release_count': result.release_count,
                    'file_size': len(xml_content),
                    'releases_per_second': result.release_count / processing_time if processing_time > 0 else 0
                })
                
                print(f"Chunk {chunk_idx}, File {file_idx}: {processing_time:.3f}s")
                
            except Exception as e:
                results.append({
                    'file_path': file_path,
                    'chunk_idx': chunk_idx,
                    'file_idx': file_idx,
                    'success': False,
                    'error': str(e),
                    'processing_time': 0,
                    'release_count': 0,
                    'file_size': 0
                })
                
                print(f"Chunk {chunk_idx}, File {file_idx}: ERROR - {str(e)}")
        
        return results
    
    async def stream_process_giant_file(self, file_path: str) -> Dict[str, Any]:
        """Stream process a very large DDEX file"""
        
        print(f"Stream processing {file_path}...")
        
        start_time = time.time()
        processed_releases = 0
        processed_tracks = 0
        
        # Monitor memory usage
        memory_samples = []
        
        async def memory_monitor():
            while True:
                memory_mb = psutil.Process().memory_info().rss / 1024 / 1024
                memory_samples.append(memory_mb)
                await asyncio.sleep(1)
        
        monitor_task = asyncio.create_task(memory_monitor())
        
        try:
            # Stream process file
            async with aiofiles.open(file_path, 'r', encoding='utf-8') as f:
                content = await f.read()
            
            # Use streaming parser
            for chunk in self.parser.stream(content):
                # Process chunk immediately
                chunk_releases = len(chunk.get('releases', []))
                chunk_tracks = len(chunk.get('soundRecordings', []))
                
                processed_releases += chunk_releases
                processed_tracks += chunk_tracks
                
                # Report progress
                if processed_releases % 1000 == 0:
                    elapsed = time.time() - start_time
                    rate = processed_releases / elapsed if elapsed > 0 else 0
                    print(f"Processed {processed_releases} releases ({rate:.1f}/sec)")
                
                # Yield control to allow memory monitoring
                await asyncio.sleep(0)
                
        finally:
            monitor_task.cancel()
        
        processing_time = time.time() - start_time
        peak_memory = max(memory_samples) if memory_samples else 0
        
        return {
            'file_path': file_path,
            'processing_time': processing_time,
            'releases_processed': processed_releases,
            'tracks_processed': processed_tracks,
            'peak_memory_mb': peak_memory,
            'releases_per_second': processed_releases / processing_time,
            'tracks_per_second': processed_tracks / processing_time
        }
    
    def create_performance_dataframe(self, results: List[Dict[str, Any]]) -> pd.DataFrame:
        """Create performance analysis DataFrame"""
        
        df = pd.DataFrame(results)
        
        # Add performance metrics
        df['mb_per_second'] = df['file_size'] / (1024 * 1024) / df['processing_time']
        df['efficiency_score'] = df['release_count'] / df['processing_time'] / (df['file_size'] / 1024 / 1024)
        
        # Performance analysis
        print("\n=== Performance Analysis ===")
        print(f"Total files processed: {len(df)}")
        print(f"Success rate: {df['success'].mean() * 100:.1f}%")
        print(f"Average processing time: {df[df['success']]['processing_time'].mean():.3f}s")
        print(f"Average throughput: {df[df['success']]['mb_per_second'].mean():.2f} MB/s")
        print(f"Average releases per second: {df[df['success']]['releases_per_second'].mean():.1f}")
        
        # Identify performance bottlenecks
        print("\n=== Performance Bottlenecks ===")
        slow_files = df[df['processing_time'] > df['processing_time'].quantile(0.9)]
        if len(slow_files) > 0:
            print(f"Slowest 10% of files (>{slow_files['processing_time'].min():.2f}s):")
            for _, row in slow_files.iterrows():
                print(f"  {row['file_path']}: {row['processing_time']:.2f}s")
        
        return df

class OptimizedDDEXBuilder:
    def __init__(self):
        self.builder = DdexBuilder()
    
    async def build_large_catalog_optimized(self, catalog_data: Dict[str, Any]) -> str:
        """Build large catalog with memory and performance optimization"""
        
        print("Building optimized large catalog...")
        
        # Apply memory-efficient preset
        self.builder.apply_preset('universal')  # Most permissive preset
        
        start_time = time.time()
        initial_memory = psutil.Process().memory_info().rss / 1024 / 1024
        
        # Process releases in batches to manage memory
        batch_size = 1000
        releases = catalog_data['releases']
        
        for i in range(0, len(releases), batch_size):
            batch = releases[i:i + batch_size]
            
            print(f"Processing release batch {i // batch_size + 1}/{(len(releases) + batch_size - 1) // batch_size}")
            
            for release in batch:
                self.builder.add_release(release)
            
            # Add corresponding tracks
            for release in batch:
                tracks = [t for t in catalog_data['tracks'] if t['release_id'] == release['release_id']]
                for track in tracks:
                    self.builder.add_resource(track)
            
            # Force garbage collection every batch
            import gc
            gc.collect()
            
            current_memory = psutil.Process().memory_info().rss / 1024 / 1024
            print(f"Memory usage: {current_memory:.1f}MB")
        
        # Build final XML
        print("Generating final XML...")
        xml = await self.builder.build()
        
        build_time = time.time() - start_time
        final_memory = psutil.Process().memory_info().rss / 1024 / 1024
        
        print(f"Build completed in {build_time:.2f}s")
        print(f"Memory used: {final_memory - initial_memory:.1f}MB")
        print(f"Output size: {len(xml) / 1024 / 1024:.1f}MB")
        
        return xml

# Usage examples
async def main():
    processor = HighPerformanceDDEXProcessor(max_workers=8)
    
    # Example 1: Process large batch of files
    file_paths = [f"catalog_{i:04d}.xml" for i in range(1000)]  # 1000 files
    
    batch_result = await processor.process_large_catalog_async(file_paths)
    
    print(f"\nBatch processing completed:")
    print(f"Processing time: {batch_result['processing_time']:.2f}s")
    print(f"Throughput: {batch_result['throughput']:.2f} files/sec")
    print(f"Success rate: {batch_result['success_rate']:.1f}%")
    print(f"Memory used: {batch_result['memory_used']:.1f}MB")
    
    # Create performance analysis
    df = processor.create_performance_dataframe(batch_result['results'])
    df.to_csv('performance_analysis.csv', index=False)
    
    # Example 2: Process giant single file
    giant_file_result = await processor.stream_process_giant_file('giant_catalog.xml')
    
    print(f"\nGiant file processing completed:")
    print(f"Releases: {giant_file_result['releases_processed']}")
    print(f"Tracks: {giant_file_result['tracks_processed']}")
    print(f"Peak memory: {giant_file_result['peak_memory_mb']:.1f}MB")
    print(f"Rate: {giant_file_result['releases_per_second']:.1f} releases/sec")

if __name__ == "__main__":
    asyncio.run(main())
```

## Caching Strategies

### Intelligent Parsing Cache

```typescript
import crypto from 'crypto';
import { promises as fs } from 'fs';
import path from 'path';

class DDEXParseCache {
  private cacheDir: string;
  private maxCacheSize: number;
  private cacheStats: Map<string, CacheEntry> = new Map();

  constructor(cacheDir: string = './ddex-cache', maxCacheSize: number = 100) {
    this.cacheDir = cacheDir;
    this.maxCacheSize = maxCacheSize;
    this.ensureCacheDir();
  }

  async parseWithCache(xmlContent: string, parser: any, options?: any): Promise<any> {
    // Generate cache key based on content and options
    const cacheKey = this.generateCacheKey(xmlContent, options);
    const cachedResult = await this.getCachedResult(cacheKey);

    if (cachedResult) {
      console.log(`Cache hit for ${cacheKey.substring(0, 8)}...`);
      this.updateCacheStats(cacheKey, true);
      return cachedResult;
    }

    console.log(`Cache miss for ${cacheKey.substring(0, 8)}...`);
    
    // Parse and cache result
    const startTime = Date.now();
    const result = await parser.parse(xmlContent, options);
    const parseTime = Date.now() - startTime;

    await this.cacheResult(cacheKey, result, parseTime);
    this.updateCacheStats(cacheKey, false);

    return result;
  }

  private generateCacheKey(xmlContent: string, options?: any): string {
    const contentHash = crypto.createHash('sha256').update(xmlContent).digest('hex');
    const optionsHash = options ? crypto.createHash('sha256').update(JSON.stringify(options)).digest('hex') : '';
    return `${contentHash}-${optionsHash}`;
  }

  private async getCachedResult(cacheKey: string): Promise<any | null> {
    try {
      const cachePath = path.join(this.cacheDir, `${cacheKey}.json`);
      const cacheData = await fs.readFile(cachePath, 'utf-8');
      const parsed = JSON.parse(cacheData);

      // Check if cache entry is still valid (e.g., not too old)
      const maxAge = 24 * 60 * 60 * 1000; // 24 hours
      if (Date.now() - parsed.timestamp > maxAge) {
        await fs.unlink(cachePath);
        return null;
      }

      return parsed.result;
    } catch (error) {
      return null;
    }
  }

  private async cacheResult(cacheKey: string, result: any, parseTime: number): Promise<void> {
    try {
      const cachePath = path.join(this.cacheDir, `${cacheKey}.json`);
      const cacheData = {
        result,
        timestamp: Date.now(),
        parseTime
      };

      await fs.writeFile(cachePath, JSON.stringify(cacheData));

      // Manage cache size
      await this.manageCacheSize();
    } catch (error) {
      console.warn('Failed to cache result:', error);
    }
  }

  private async manageCacheSize(): Promise<void> {
    const files = await fs.readdir(this.cacheDir);
    const cacheFiles = files.filter(f => f.endsWith('.json'));

    if (cacheFiles.length > this.maxCacheSize) {
      // Remove oldest files
      const fileStats = await Promise.all(
        cacheFiles.map(async file => {
          const filePath = path.join(this.cacheDir, file);
          const stats = await fs.stat(filePath);
          return { file, mtime: stats.mtime };
        })
      );

      fileStats.sort((a, b) => a.mtime.getTime() - b.mtime.getTime());
      
      const filesToRemove = fileStats.slice(0, cacheFiles.length - this.maxCacheSize);
      
      for (const { file } of filesToRemove) {
        await fs.unlink(path.join(this.cacheDir, file));
      }
    }
  }

  private updateCacheStats(cacheKey: string, hit: boolean): void {
    const entry = this.cacheStats.get(cacheKey) || { hits: 0, misses: 0 };
    
    if (hit) {
      entry.hits++;
    } else {
      entry.misses++;
    }
    
    this.cacheStats.set(cacheKey, entry);
  }

  getCacheStatistics(): CacheStatistics {
    const totalHits = Array.from(this.cacheStats.values()).reduce((sum, entry) => sum + entry.hits, 0);
    const totalMisses = Array.from(this.cacheStats.values()).reduce((sum, entry) => sum + entry.misses, 0);
    const total = totalHits + totalMisses;

    return {
      hitRate: total > 0 ? totalHits / total : 0,
      totalHits,
      totalMisses,
      uniqueKeys: this.cacheStats.size
    };
  }

  private async ensureCacheDir(): Promise<void> {
    try {
      await fs.mkdir(this.cacheDir, { recursive: true });
    } catch (error) {
      console.warn('Failed to create cache directory:', error);
    }
  }
}

interface CacheEntry {
  hits: number;
  misses: number;
}

interface CacheStatistics {
  hitRate: number;
  totalHits: number;
  totalMisses: number;
  uniqueKeys: number;
}

// Usage with cache
async function processWithIntelligentCaching(filePaths: string[]) {
  const cache = new DDEXParseCache('./cache', 500);
  const parser = new DDEXParser();
  
  console.log('Processing files with intelligent caching...');
  
  for (const filePath of filePaths) {
    const xmlContent = await fs.readFile(filePath, 'utf-8');
    
    const result = await cache.parseWithCache(xmlContent, parser, {
      includeRawExtensions: false,
      validateReferences: true
    });
    
    console.log(`Processed ${path.basename(filePath)}: ${result.flat.releases.length} releases`);
  }
  
  const stats = cache.getCacheStatistics();
  console.log(`\nCache statistics:`);
  console.log(`Hit rate: ${(stats.hitRate * 100).toFixed(1)}%`);
  console.log(`Total operations: ${stats.totalHits + stats.totalMisses}`);
  console.log(`Unique files: ${stats.uniqueKeys}`);
}
```

## Database Optimization

### Efficient Data Pipeline

```typescript
import { Pool } from 'pg'; // PostgreSQL
import { DDEXParser } from 'ddex-parser';

class DatabaseOptimizedProcessor {
  private db: Pool;
  private parser: DDEXParser;
  private batchSize: number = 1000;

  constructor(dbConfig: any) {
    this.db = new Pool(dbConfig);
    this.parser = new DDEXParser();
  }

  async processToDatabaseBatched(filePaths: string[]): Promise<void> {
    console.log(`Processing ${filePaths.length} files to database...`);

    // Prepare batch insert statements
    await this.prepareBatchStatements();

    let releaseBatch: any[] = [];
    let trackBatch: any[] = [];

    for (const filePath of filePaths) {
      try {
        const xmlContent = await require('fs').promises.readFile(filePath, 'utf-8');
        const result = await this.parser.parse(xmlContent, {
          streaming: true,
          includeRawExtensions: false
        });

        // Add to batches
        for (const release of result.flat.releases) {
          releaseBatch.push({
            releaseId: release.releaseId,
            title: release.title,
            artist: release.displayArtist,
            label: release.labelName,
            releaseDate: release.releaseDate,
            genre: release.genre,
            sourceFile: filePath
          });
        }

        for (const track of result.flat.soundRecordings) {
          trackBatch.push({
            soundRecordingId: track.soundRecordingId,
            isrc: track.isrc,
            title: track.title,
            artist: track.displayArtist,
            duration: track.durationSeconds,
            sourceFile: filePath
          });
        }

        // Flush batches when they reach target size
        if (releaseBatch.length >= this.batchSize) {
          await this.flushReleaseBatch(releaseBatch);
          releaseBatch = [];
        }

        if (trackBatch.length >= this.batchSize) {
          await this.flushTrackBatch(trackBatch);
          trackBatch = [];
        }

      } catch (error) {
        console.error(`Failed to process ${filePath}:`, error);
      }
    }

    // Flush remaining items
    if (releaseBatch.length > 0) {
      await this.flushReleaseBatch(releaseBatch);
    }

    if (trackBatch.length > 0) {
      await this.flushTrackBatch(trackBatch);
    }

    console.log('Database processing completed');
  }

  private async prepareBatchStatements(): Promise<void> {
    // Create tables if they don't exist
    await this.db.query(`
      CREATE TABLE IF NOT EXISTS releases (
        release_id VARCHAR(255) PRIMARY KEY,
        title TEXT,
        artist TEXT,
        label TEXT,
        release_date DATE,
        genre VARCHAR(100),
        source_file TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      )
    `);

    await this.db.query(`
      CREATE TABLE IF NOT EXISTS tracks (
        sound_recording_id VARCHAR(255) PRIMARY KEY,
        isrc VARCHAR(12),
        title TEXT,
        artist TEXT,
        duration INTEGER,
        source_file TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
      )
    `);

    // Create indexes for performance
    await this.db.query(`
      CREATE INDEX IF NOT EXISTS idx_releases_artist ON releases(artist);
      CREATE INDEX IF NOT EXISTS idx_releases_label ON releases(label);
      CREATE INDEX IF NOT EXISTS idx_tracks_isrc ON tracks(isrc);
    `);
  }

  private async flushReleaseBatch(batch: any[]): Promise<void> {
    const client = await this.db.connect();
    
    try {
      await client.query('BEGIN');

      const values = batch.map((_, index) => {
        const base = index * 7;
        return `($${base + 1}, $${base + 2}, $${base + 3}, $${base + 4}, $${base + 5}, $${base + 6}, $${base + 7})`;
      }).join(', ');

      const flatValues = batch.flatMap(release => [
        release.releaseId,
        release.title,
        release.artist,
        release.label,
        release.releaseDate,
        release.genre,
        release.sourceFile
      ]);

      await client.query(`
        INSERT INTO releases (release_id, title, artist, label, release_date, genre, source_file)
        VALUES ${values}
        ON CONFLICT (release_id) DO UPDATE SET
          title = EXCLUDED.title,
          artist = EXCLUDED.artist,
          label = EXCLUDED.label,
          release_date = EXCLUDED.release_date,
          genre = EXCLUDED.genre,
          source_file = EXCLUDED.source_file
      `, flatValues);

      await client.query('COMMIT');
      console.log(`Inserted ${batch.length} releases`);

    } catch (error) {
      await client.query('ROLLBACK');
      throw error;
    } finally {
      client.release();
    }
  }

  private async flushTrackBatch(batch: any[]): Promise<void> {
    const client = await this.db.connect();
    
    try {
      await client.query('BEGIN');

      const values = batch.map((_, index) => {
        const base = index * 6;
        return `($${base + 1}, $${base + 2}, $${base + 3}, $${base + 4}, $${base + 5}, $${base + 6})`;
      }).join(', ');

      const flatValues = batch.flatMap(track => [
        track.soundRecordingId,
        track.isrc,
        track.title,
        track.artist,
        track.duration,
        track.sourceFile
      ]);

      await client.query(`
        INSERT INTO tracks (sound_recording_id, isrc, title, artist, duration, source_file)
        VALUES ${values}
        ON CONFLICT (sound_recording_id) DO UPDATE SET
          isrc = EXCLUDED.isrc,
          title = EXCLUDED.title,
          artist = EXCLUDED.artist,
          duration = EXCLUDED.duration,
          source_file = EXCLUDED.source_file
      `, flatValues);

      await client.query('COMMIT');
      console.log(`Inserted ${batch.length} tracks`);

    } catch (error) {
      await client.query('ROLLBACK');
      throw error;
    } finally {
      client.release();
    }
  }
}
```

## Performance Monitoring

### Real-time Performance Dashboard

```typescript
class PerformanceMonitor {
  private metrics: PerformanceMetrics = {
    filesProcessed: 0,
    totalProcessingTime: 0,
    totalFileSize: 0,
    errors: 0,
    startTime: Date.now(),
    memoryPeak: 0,
    throughputSamples: []
  };

  startMonitoring(): void {
    // Monitor memory usage
    setInterval(() => {
      const usage = process.memoryUsage();
      const currentMB = usage.heapUsed / 1024 / 1024;
      
      if (currentMB > this.metrics.memoryPeak) {
        this.metrics.memoryPeak = currentMB;
      }
    }, 1000);

    // Report status every 10 seconds
    setInterval(() => {
      this.reportStatus();
    }, 10000);
  }

  recordFileProcessed(filePath: string, processingTime: number, fileSize: number, success: boolean): void {
    this.metrics.filesProcessed++;
    this.metrics.totalProcessingTime += processingTime;
    this.metrics.totalFileSize += fileSize;
    
    if (!success) {
      this.metrics.errors++;
    }

    // Calculate throughput
    const throughput = fileSize / processingTime; // bytes/ms
    this.metrics.throughputSamples.push(throughput);
    
    // Keep only recent samples
    if (this.metrics.throughputSamples.length > 100) {
      this.metrics.throughputSamples.shift();
    }
  }

  private reportStatus(): void {
    const elapsed = (Date.now() - this.metrics.startTime) / 1000;
    const avgProcessingTime = this.metrics.totalProcessingTime / this.metrics.filesProcessed;
    const totalSizeMB = this.metrics.totalFileSize / 1024 / 1024;
    const avgThroughput = this.metrics.throughputSamples.reduce((a, b) => a + b, 0) / this.metrics.throughputSamples.length;
    
    console.log('\n=== Performance Status ===');
    console.log(`Uptime: ${elapsed.toFixed(1)}s`);
    console.log(`Files processed: ${this.metrics.filesProcessed}`);
    console.log(`Total size: ${totalSizeMB.toFixed(1)}MB`);
    console.log(`Average processing time: ${avgProcessingTime.toFixed(2)}ms`);
    console.log(`Current throughput: ${(avgThroughput * 1000 / 1024 / 1024).toFixed(2)}MB/s`);
    console.log(`Memory peak: ${this.metrics.memoryPeak.toFixed(1)}MB`);
    console.log(`Error rate: ${(this.metrics.errors / this.metrics.filesProcessed * 100).toFixed(1)}%`);
    console.log(`Files/sec: ${(this.metrics.filesProcessed / elapsed).toFixed(2)}`);
  }

  getMetrics(): PerformanceMetrics {
    return { ...this.metrics };
  }
}

interface PerformanceMetrics {
  filesProcessed: number;
  totalProcessingTime: number;
  totalFileSize: number;
  errors: number;
  startTime: number;
  memoryPeak: number;
  throughputSamples: number[];
}
```

## Common Performance Pitfalls

### Pitfall 1: Loading Entire Files into Memory

```typescript
// WRONG: Loading huge files entirely
const xmlContent = fs.readFileSync('huge-file.xml', 'utf-8'); // 1GB+ in memory
const result = await parser.parse(xmlContent);

// RIGHT: Use streaming
const stream = fs.createReadStream('huge-file.xml');
const result = await parser.streamParse(stream, { streaming: true });
```

### Pitfall 2: Blocking the Event Loop

```typescript
// WRONG: Synchronous processing blocks event loop
for (const file of hugeFileList) {
  const result = parser.parseSync(file); // Blocks!
  processResult(result);
}

// RIGHT: Async processing with batching
const semaphore = new Semaphore(5);
for (const file of hugeFileList) {
  await semaphore.acquire();
  parser.parse(file).then(result => {
    processResult(result);
    semaphore.release();
  });
}
```

### Pitfall 3: Memory Accumulation

```typescript
// WRONG: Accumulating results in memory
const allResults = [];
for (const file of files) {
  const result = await parser.parse(file);
  allResults.push(result); // Memory grows indefinitely
}

// RIGHT: Process and discard
for (const file of files) {
  const result = await parser.parse(file);
  await processAndStore(result); // Process immediately
  // Result can be garbage collected
}
```

## Links to API Documentation

- [Streaming Parser API](../api/parser/typescript) - Streaming methods and configuration
- [Builder Performance](../api/builder/typescript) - High-performance building
- [Memory Management](../api/parser/types) - Memory optimization options
- [Error Handling](./error-handling) - Performance-aware error handling
- [Streaming Large Files](./streaming-large-files) - Specialized streaming techniques

This guide provides comprehensive optimization strategies for processing DDEX catalogs at scale while maintaining memory efficiency and maximizing throughput.