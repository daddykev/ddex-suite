# Performance Issues

Diagnose and resolve common performance issues in DDEX Suite.

## Common Performance Issues

### Slow Parsing Performance

**Symptoms**: XML parsing takes longer than expected

**Causes & Solutions**:
- **Large file size**: Use streaming parser for files >10MB
- **Complex XML structure**: Enable streaming mode
- **Memory pressure**: Increase available memory or use memory-bounded parsing

```typescript
// For large files, use streaming
const parser = new DDEXParser({ 
  streaming: true,
  maxMemoryUsage: 512 * 1024 * 1024 // 512MB limit
});
```

### Memory Usage Issues

**Symptoms**: High memory consumption during processing

**Solutions**:
- Use streaming parser for large files
- Process files in batches
- Enable garbage collection
- Set memory limits

```python
# Memory-efficient processing
from ddex_parser import DDEXParser

parser = DDEXParser()
parser.set_memory_limit(500 * 1024 * 1024)  # 500MB limit

# Process in chunks
for chunk in parser.parse_streaming(large_xml_file):
    process_chunk(chunk)
    # Memory is released after each chunk
```

### Build Performance Issues

**Symptoms**: DDEX XML generation is slow

**Solutions**:
- Use deterministic mode only when needed
- Batch similar operations
- Pre-validate data structure
- Use appropriate buffer sizes

## Performance Monitoring

```python
import time
import psutil
from contextlib import contextmanager

@contextmanager
def performance_monitor(operation_name):
    """Monitor performance of DDEX operations"""
    
    start_time = time.time()
    start_memory = psutil.Process().memory_info().rss
    
    print(f"Starting {operation_name}...")
    
    try:
        yield
    finally:
        end_time = time.time()
        end_memory = psutil.Process().memory_info().rss
        
        duration = end_time - start_time
        memory_used = (end_memory - start_memory) / 1024 / 1024  # MB
        
        print(f"{operation_name} completed:")
        print(f"  Duration: {duration:.2f} seconds")
        print(f"  Memory used: {memory_used:.1f} MB")

# Usage
with performance_monitor("DDEX Parsing"):
    result = parser.parse(xml_content)
```

## Optimization Strategies

### 1. Batch Processing
```typescript
// Process multiple files efficiently
const results = await parser.parseBatch(xmlFiles, {
  concurrency: 4,
  memoryLimit: 1024 * 1024 * 1024
});
```

### 2. Streaming for Large Files
```python
# Stream large files to avoid memory issues
for chunk in parser.parse_streaming(large_file_path):
    process_data_chunk(chunk)
```

### 3. Caching
```typescript
// Cache parsed results
const cache = new Map();

function parseWithCache(xmlContent: string) {
  const hash = calculateHash(xmlContent);
  if (cache.has(hash)) {
    return cache.get(hash);
  }
  
  const result = parser.parse(xmlContent);
  cache.set(hash, result);
  return result;
}
```

## Best Practices

1. **Profile First**: Use profiling tools to identify bottlenecks
2. **Stream Large Files**: Use streaming for files >10MB
3. **Batch Operations**: Process multiple files in batches
4. **Monitor Memory**: Keep memory usage under control
5. **Use Caching**: Cache frequently accessed data
6. **Optimize Data Structures**: Use efficient data representations
7. **Parallel Processing**: Use worker threads for CPU-intensive tasks