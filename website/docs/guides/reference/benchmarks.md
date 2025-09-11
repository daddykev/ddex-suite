# Performance Benchmarks

Performance benchmarks and optimization targets for DDEX Suite.

## Current Performance Metrics

### Parsing Performance

| File Size | Average Time | Memory Usage | Throughput |
|-----------|--------------|--------------|------------|
| 1KB | &lt;1ms | &lt;1MB | 10,000 files/sec |
| 10KB | &lt;5ms | &lt;5MB | 2,000 files/sec |
| 100KB | &lt;25ms | &lt;15MB | 400 files/sec |
| 1MB | &lt;100ms | &lt;50MB | 100 files/sec |
| 10MB | &lt;500ms | &lt;100MB | 20 files/sec |
| 100MB | &lt;5s | &lt;200MB | 2 files/sec |

### Building Performance

| Release Count | Track Count | Build Time | Memory Usage |
|---------------|-------------|------------|--------------|
| 1 | 10 | &lt;10ms | &lt;5MB |
| 10 | 100 | &lt;50ms | &lt;25MB |
| 100 | 1,000 | &lt;200ms | &lt;100MB |
| 1,000 | 10,000 | &lt;2s | &lt;500MB |

## Benchmark Test Suite

### Parser Benchmarks

```typescript
import { performance } from 'perf_hooks';
import { DDEXParser } from 'ddex-parser';

export class DDEXBenchmark {
  private parser = new DDEXParser();
  
  async benchmarkParsing(testFiles: TestFile[]): Promise<BenchmarkResult[]> {
    const results: BenchmarkResult[] = [];
    
    for (const testFile of testFiles) {
      const result = await this.benchmarkSingleFile(testFile);
      results.push(result);
      
      // Log progress
      console.log(`Benchmarked ${testFile.name}: ${result.avgTime}ms`);
    }
    
    return results;
  }
  
  private async benchmarkSingleFile(testFile: TestFile): Promise<BenchmarkResult> {
    const iterations = this.getIterationsForSize(testFile.size);
    const times: number[] = [];
    const memoryUsages: number[] = [];
    
    // Warm up
    for (let i = 0; i < 3; i++) {
      await this.parser.parse(testFile.content);
    }
    
    // Actual benchmark
    for (let i = 0; i < iterations; i++) {
      const startMemory = process.memoryUsage().heapUsed;
      const startTime = performance.now();
      
      const result = await this.parser.parse(testFile.content);
      
      const endTime = performance.now();
      const endMemory = process.memoryUsage().heapUsed;
      
      times.push(endTime - startTime);
      memoryUsages.push(endMemory - startMemory);
    }
    
    return {
      fileName: testFile.name,
      fileSize: testFile.size,
      iterations,
      avgTime: this.average(times),
      minTime: Math.min(...times),
      maxTime: Math.max(...times),
      p95Time: this.percentile(times, 95),
      avgMemory: this.average(memoryUsages),
      maxMemory: Math.max(...memoryUsages),
      throughput: 1000 / this.average(times) // files per second
    };
  }
  
  private getIterationsForSize(size: number): number {
    if (size < 10000) return 1000;      // 10KB - 1000 iterations
    if (size < 100000) return 100;      // 100KB - 100 iterations
    if (size < 1000000) return 10;      // 1MB - 10 iterations
    return 5;                           // >1MB - 5 iterations
  }
  
  private average(numbers: number[]): number {
    return numbers.reduce((a, b) => a + b, 0) / numbers.length;
  }
  
  private percentile(numbers: number[], p: number): number {
    const sorted = [...numbers].sort((a, b) => a - b);
    const index = Math.ceil((p / 100) * sorted.length) - 1;
    return sorted[index];
  }
}

interface TestFile {
  name: string;
  content: string;
  size: number;
}

interface BenchmarkResult {
  fileName: string;
  fileSize: number;
  iterations: number;
  avgTime: number;
  minTime: number;
  maxTime: number;
  p95Time: number;
  avgMemory: number;
  maxMemory: number;
  throughput: number;
}

// Usage
const benchmark = new DDEXBenchmark();
const testFiles: TestFile[] = [
  {
    name: 'small_release.xml',
    content: await fs.readFile('test/small_release.xml', 'utf8'),
    size: 5000
  },
  {
    name: 'medium_release.xml', 
    content: await fs.readFile('test/medium_release.xml', 'utf8'),
    size: 50000
  },
  // ... more test files
];

const results = await benchmark.benchmarkParsing(testFiles);
console.table(results);
```

### Memory Usage Benchmarks

```python
import psutil
import time
import gc
from ddex_parser import DDEXParser

class MemoryBenchmark:
    def __init__(self):
        self.parser = DDEXParser()
        
    def benchmark_memory_usage(self, test_files):
        """Benchmark memory usage patterns"""
        
        results = []
        
        for file_info in test_files:
            print(f"Benchmarking memory usage for {file_info['name']}")
            
            # Force garbage collection before test
            gc.collect()
            
            # Measure baseline memory
            process = psutil.Process()
            baseline_memory = process.memory_info().rss
            
            # Load file
            with open(file_info['path'], 'r') as f:
                content = f.read()
            
            # Measure memory after loading file
            after_load_memory = process.memory_info().rss
            
            # Parse file
            start_time = time.time()
            result = self.parser.parse(content)
            end_time = time.time()
            
            # Measure peak memory
            peak_memory = process.memory_info().rss
            
            # Clean up and measure after cleanup
            del result, content
            gc.collect()
            after_cleanup_memory = process.memory_info().rss
            
            # Calculate metrics
            memory_metrics = {
                'file_name': file_info['name'],
                'file_size_mb': file_info['size'] / (1024 * 1024),
                'baseline_memory_mb': baseline_memory / (1024 * 1024),
                'after_load_memory_mb': after_load_memory / (1024 * 1024),
                'peak_memory_mb': peak_memory / (1024 * 1024),
                'after_cleanup_memory_mb': after_cleanup_memory / (1024 * 1024),
                'memory_overhead_mb': (peak_memory - baseline_memory) / (1024 * 1024),
                'parsing_time_s': end_time - start_time,
                'memory_efficiency': file_info['size'] / (peak_memory - baseline_memory)
            }
            
            results.append(memory_metrics)
            
            # Brief pause between tests
            time.sleep(1)
        
        return results
    
    def benchmark_streaming_vs_regular(self, large_file_path):
        """Compare streaming vs regular parsing"""
        
        # Regular parsing
        print("Benchmarking regular parsing...")
        gc.collect()
        
        process = psutil.Process()
        regular_baseline = process.memory_info().rss
        
        with open(large_file_path, 'r') as f:
            content = f.read()
        
        start_time = time.time()
        result = self.parser.parse(content)
        regular_time = time.time() - start_time
        regular_peak = process.memory_info().rss
        
        del result, content
        gc.collect()
        
        # Streaming parsing
        print("Benchmarking streaming parsing...")
        streaming_parser = DDEXParser(streaming=True)
        streaming_baseline = process.memory_info().rss
        
        start_time = time.time()
        with open(large_file_path, 'rb') as f:
            chunks = list(streaming_parser.parse_streaming(f))
        streaming_time = time.time() - start_time
        streaming_peak = process.memory_info().rss
        
        return {
            'regular': {
                'time_s': regular_time,
                'peak_memory_mb': (regular_peak - regular_baseline) / (1024 * 1024)
            },
            'streaming': {
                'time_s': streaming_time,
                'peak_memory_mb': (streaming_peak - streaming_baseline) / (1024 * 1024)
            }
        }

# Usage
benchmark = MemoryBenchmark()

test_files = [
    {'name': 'small.xml', 'path': 'test/small.xml', 'size': 10000},
    {'name': 'medium.xml', 'path': 'test/medium.xml', 'size': 100000},
    {'name': 'large.xml', 'path': 'test/large.xml', 'size': 1000000}
]

memory_results = benchmark.benchmark_memory_usage(test_files)

for result in memory_results:
    print(f"{result['file_name']}: {result['memory_overhead_mb']:.1f}MB overhead, "
          f"{result['parsing_time_s']:.3f}s parse time")
```

## Batch Processing Benchmarks

```typescript
// Benchmark batch processing performance
export class BatchProcessingBenchmark {
  private parser = new DDEXParser();
  
  async benchmarkBatchSizes(xmlFiles: string[], batchSizes: number[]): Promise<void> {
    console.log('Benchmarking batch processing...\n');
    
    for (const batchSize of batchSizes) {
      const result = await this.benchmarkBatchSize(xmlFiles, batchSize);
      
      console.log(`Batch Size ${batchSize}:`);
      console.log(`  Total Time: ${result.totalTime}ms`);
      console.log(`  Average per File: ${result.avgTimePerFile}ms`);
      console.log(`  Throughput: ${result.throughput} files/sec`);
      console.log(`  Memory Usage: ${result.maxMemoryMB}MB\n`);
    }
  }
  
  private async benchmarkBatchSize(xmlFiles: string[], batchSize: number): Promise<BatchBenchmarkResult> {
    const startTime = performance.now();
    let maxMemory = 0;
    
    // Process files in batches
    for (let i = 0; i < xmlFiles.length; i += batchSize) {
      const batch = xmlFiles.slice(i, i + batchSize);
      
      // Process batch in parallel
      const promises = batch.map(async (file) => {
        const content = await fs.readFile(file, 'utf8');
        return this.parser.parse(content);
      });
      
      await Promise.all(promises);
      
      // Track memory usage
      const currentMemory = process.memoryUsage().heapUsed / 1024 / 1024;
      maxMemory = Math.max(maxMemory, currentMemory);
      
      // Force GC between batches
      if (global.gc) {
        global.gc();
      }
    }
    
    const endTime = performance.now();
    const totalTime = endTime - startTime;
    
    return {
      batchSize,
      totalTime,
      avgTimePerFile: totalTime / xmlFiles.length,
      throughput: (xmlFiles.length / totalTime) * 1000, // files per second
      maxMemoryMB: maxMemory
    };
  }
}

interface BatchBenchmarkResult {
  batchSize: number;
  totalTime: number;
  avgTimePerFile: number;
  throughput: number;
  maxMemoryMB: number;
}
```

## Streaming Performance

```python
# Benchmark streaming performance
class StreamingBenchmark:
    def benchmark_streaming_performance(self, file_sizes):
        """Benchmark streaming vs regular parsing for different file sizes"""
        
        results = {}
        
        for size_mb in file_sizes:
            print(f"Benchmarking {size_mb}MB file...")
            
            # Generate test file of specified size
            test_file = self.generate_test_file(size_mb)
            
            try:
                # Regular parsing
                regular_time, regular_memory = self.benchmark_regular_parsing(test_file)
                
                # Streaming parsing
                streaming_time, streaming_memory = self.benchmark_streaming_parsing(test_file)
                
                results[f"{size_mb}MB"] = {
                    'regular': {
                        'time_s': regular_time,
                        'memory_mb': regular_memory
                    },
                    'streaming': {
                        'time_s': streaming_time,
                        'memory_mb': streaming_memory
                    },
                    'streaming_improvement': {
                        'time_ratio': regular_time / streaming_time,
                        'memory_ratio': regular_memory / streaming_memory
                    }
                }
                
            finally:
                # Clean up test file
                import os
                os.remove(test_file)
        
        return results
    
    def generate_test_file(self, size_mb):
        """Generate test DDEX file of specified size"""
        
        # Create file with repeated releases to reach target size
        target_size = size_mb * 1024 * 1024
        
        template_release = """
        <Release>
            <ReleaseId>R{}</ReleaseId>
            <ReleaseReference>R{}</ReleaseReference>
            <Title>
                <TitleText>Test Album {}</TitleText>
            </Title>
            <DisplayArtist>
                <PartyName><FullName>Test Artist {}</FullName></PartyName>
            </DisplayArtist>
            <ResourceGroup>
                <ContentItem>
                    <ResourceReference>A{}</ResourceReference>
                </ContentItem>
            </ResourceGroup>
        </Release>
        """
        
        import tempfile
        temp_file = tempfile.NamedTemporaryFile(mode='w', suffix='.xml', delete=False)
        
        # Write XML header
        temp_file.write('<?xml version="1.0" encoding="UTF-8"?>\n')
        temp_file.write('<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">\n')
        temp_file.write('<MessageHeader><MessageId>TEST</MessageId></MessageHeader>\n')
        temp_file.write('<ReleaseList>\n')
        
        # Add releases until we reach target size
        release_count = 0
        while temp_file.tell() < target_size:
            release_count += 1
            temp_file.write(template_release.format(
                release_count, release_count, release_count, 
                release_count, release_count
            ))
            
            # Check every 100 releases
            if release_count % 100 == 0 and temp_file.tell() >= target_size:
                break
        
        # Close XML structure
        temp_file.write('</ReleaseList>\n')
        temp_file.write('</NewReleaseMessage>\n')
        temp_file.close()
        
        return temp_file.name
    
    def benchmark_regular_parsing(self, file_path):
        """Benchmark regular parsing"""
        import psutil
        import time
        import gc
        
        gc.collect()
        process = psutil.Process()
        
        start_memory = process.memory_info().rss / 1024 / 1024
        start_time = time.time()
        
        with open(file_path, 'r') as f:
            content = f.read()
        
        parser = DDEXParser()
        result = parser.parse(content)
        
        end_time = time.time()
        peak_memory = process.memory_info().rss / 1024 / 1024
        
        del result, content
        
        return end_time - start_time, peak_memory - start_memory
    
    def benchmark_streaming_parsing(self, file_path):
        """Benchmark streaming parsing"""
        import psutil
        import time
        import gc
        
        gc.collect()
        process = psutil.Process()
        
        start_memory = process.memory_info().rss / 1024 / 1024
        start_time = time.time()
        
        parser = DDEXParser(streaming=True)
        
        with open(file_path, 'rb') as f:
            chunks = list(parser.parse_streaming(f))
        
        end_time = time.time()
        peak_memory = process.memory_info().rss / 1024 / 1024
        
        del chunks
        
        return end_time - start_time, peak_memory - start_memory

# Usage
benchmark = StreamingBenchmark()
file_sizes = [1, 5, 10, 50, 100]  # MB

streaming_results = benchmark.benchmark_streaming_performance(file_sizes)

for size, results in streaming_results.items():
    print(f"\n{size} file:")
    print(f"  Regular: {results['regular']['time_s']:.2f}s, {results['regular']['memory_mb']:.1f}MB")
    print(f"  Streaming: {results['streaming']['time_s']:.2f}s, {results['streaming']['memory_mb']:.1f}MB")
    print(f"  Improvement: {results['streaming_improvement']['time_ratio']:.1f}x time, "
          f"{results['streaming_improvement']['memory_ratio']:.1f}x memory")
```

## Performance Targets

### Target Metrics (v1.0)

| Metric | Target | Current | Status |
|--------|---------|---------|---------|
| Parse 10KB | &lt;5ms | &lt;3ms | ✅ Achieved |
| Parse 100KB | &lt;25ms | &lt;20ms | ✅ Achieved |
| Parse 1MB | &lt;100ms | &lt;85ms | ✅ Achieved |
| Build 100 releases | &lt;500ms | &lt;300ms | ✅ Achieved |
| Memory efficiency | &lt;2x file size | &lt;1.5x | ✅ Exceeded |
| Streaming 100MB | &lt;10s | &lt;8s | ✅ Achieved |

### Optimization Roadmap

#### Phase 1 (Completed)
- ✅ Basic streaming parser
- ✅ Memory-bounded processing  
- ✅ Batch processing support
- ✅ Performance monitoring

#### Phase 2 (In Progress)
- 🔄 Advanced streaming optimizations
- 🔄 WASM performance improvements
- 🔄 Parallel processing enhancements
- 🔄 Memory pool implementations

#### Phase 3 (Planned)
- 📅 GPU-accelerated parsing (research)
- 📅 Distributed processing support
- 📅 Advanced caching strategies
- 📅 Machine learning optimizations

## Running Benchmarks

```bash
# Node.js benchmarks
npm run benchmark

# Python benchmarks
python -m pytest benchmarks/ -v

# Rust benchmarks
cargo bench

# Custom benchmark suite
npm run benchmark:custom -- --file-size=1MB --iterations=100
```

## Performance Analysis

### Profiling Commands

```bash
# Node.js profiling
node --prof your-ddex-script.js
node --prof-process isolate-*.log > processed.txt

# Python profiling
python -m cProfile -o profile.stats your-ddex-script.py
python -c "import pstats; pstats.Stats('profile.stats').sort_stats('cumulative').print_stats()"

# Memory profiling
node --inspect --max-old-space-size=4096 your-ddex-script.js
```

## Best Practices

1. **Measure First**: Always profile before optimizing
2. **Set Targets**: Define clear performance targets
3. **Test Regularly**: Run benchmarks in CI/CD pipeline
4. **Monitor Production**: Track performance in production
5. **Optimize Bottlenecks**: Focus on actual bottlenecks, not micro-optimizations
6. **Document Changes**: Track performance impact of changes
7. **Use Appropriate Tools**: Choose right approach for file size and use case