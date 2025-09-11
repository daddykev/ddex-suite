# Common Patterns

Common patterns and best practices for DDEX Suite usage.

## Data Transformation Patterns

### Parse → Transform → Build Pattern

```typescript
// Classic ETL pattern with DDEX Suite
import { DDEXParser, DDEXBuilder } from 'ddex-suite';

class DDEXTransformer {
  private parser = new DDEXParser();
  private builder = new DDEXBuilder();

  async transformRelease(xmlContent: string, transformFn: (data: any) => any): Promise<string> {
    // Parse
    const parsed = await this.parser.parse(xmlContent);
    
    // Transform
    const transformed = transformFn(parsed.flat);
    
    // Build
    return this.builder.build(transformed);
  }
}

// Usage
const transformer = new DDEXTransformer();

const result = await transformer.transformRelease(xmlContent, (data) => {
  // Add explicit content warnings
  data.releases.forEach(release => {
    release.tracks.forEach(track => {
      if (track.title.includes('explicit')) {
        track.parental_warning_type = 'Explicit';
      }
    });
  });
  
  return data;
});
```

### Batch Processing Pattern

```python
# Process multiple files efficiently
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder
from concurrent.futures import ThreadPoolExecutor
import os

class BatchDDEXProcessor:
    def __init__(self, max_workers=4):
        self.parser = DDEXParser()
        self.builder = DDEXBuilder()
        self.max_workers = max_workers
    
    def process_directory(self, input_dir, output_dir, transform_fn=None):
        """Process all XML files in directory"""
        
        xml_files = [
            os.path.join(input_dir, f) 
            for f in os.listdir(input_dir) 
            if f.endswith('.xml')
        ]
        
        with ThreadPoolExecutor(max_workers=self.max_workers) as executor:
            futures = [
                executor.submit(self._process_file, xml_file, output_dir, transform_fn)
                for xml_file in xml_files
            ]
            
            results = [future.result() for future in futures]
        
        return results
    
    def _process_file(self, xml_file, output_dir, transform_fn):
        """Process single file"""
        try:
            with open(xml_file, 'r') as f:
                content = f.read()
            
            # Parse
            parsed = self.parser.parse(content)
            
            # Transform if function provided
            if transform_fn:
                data = transform_fn(parsed.flat.to_dict())
            else:
                data = parsed.flat.to_dict()
            
            # Build
            new_xml = self.builder.build(data)
            
            # Save
            filename = os.path.basename(xml_file)
            output_path = os.path.join(output_dir, f"processed_{filename}")
            
            with open(output_path, 'w') as f:
                f.write(new_xml)
            
            return {'status': 'success', 'file': xml_file, 'output': output_path}
            
        except Exception as e:
            return {'status': 'error', 'file': xml_file, 'error': str(e)}
```

## Validation Patterns

### Multi-Level Validation Pattern

```typescript
// Comprehensive validation approach
export class ValidationPipeline {
  private validators: Validator[] = [];

  addValidator(validator: Validator): void {
    this.validators.push(validator);
  }

  async validate(data: any): Promise<ValidationResult> {
    const results: ValidationResult[] = [];

    for (const validator of this.validators) {
      const result = await validator.validate(data);
      results.push(result);
      
      // Stop on critical errors
      if (result.hasCriticalErrors) {
        break;
      }
    }

    return this.combineResults(results);
  }

  private combineResults(results: ValidationResult[]): ValidationResult {
    return {
      isValid: results.every(r => r.isValid),
      errors: results.flatMap(r => r.errors),
      warnings: results.flatMap(r => r.warnings),
      hasCriticalErrors: results.some(r => r.hasCriticalErrors)
    };
  }
}

// Usage
const pipeline = new ValidationPipeline();
pipeline.addValidator(new SchemaValidator());
pipeline.addValidator(new BusinessRulesValidator());
pipeline.addValidator(new PartnerSpecificValidator('spotify'));

const validation = await pipeline.validate(releaseData);
```

### Progressive Validation Pattern

```python
# Validate with increasing strictness
class ProgressiveValidator:
    def __init__(self):
        self.validation_levels = [
            ('basic', self._validate_basic),
            ('standard', self._validate_standard),
            ('strict', self._validate_strict),
            ('partner', self._validate_partner)
        ]
    
    def validate_progressively(self, data, stop_on_error=True):
        """Validate with increasing strictness"""
        
        results = {}
        
        for level_name, validation_fn in self.validation_levels:
            try:
                result = validation_fn(data)
                results[level_name] = result
                
                if stop_on_error and not result.get('is_valid', True):
                    break
                    
            except Exception as e:
                results[level_name] = {'is_valid': False, 'error': str(e)}
                
                if stop_on_error:
                    break
        
        return results
    
    def _validate_basic(self, data):
        """Basic validation - required fields only"""
        errors = []
        
        if not data.get('releases'):
            errors.append("No releases found")
        
        for release in data.get('releases', []):
            if not release.get('title'):
                errors.append("Release missing title")
        
        return {'is_valid': len(errors) == 0, 'errors': errors}
    
    def _validate_standard(self, data):
        """Standard validation - business rules"""
        # Implementation for standard validation
        pass
    
    def _validate_strict(self, data):
        """Strict validation - all best practices"""
        # Implementation for strict validation
        pass
    
    def _validate_partner(self, data):
        """Partner-specific validation"""
        # Implementation for partner validation
        pass
```

## Error Handling Patterns

### Resilient Processing Pattern

```typescript
// Handle errors gracefully while processing
export class ResilientDDEXProcessor {
  private parser = new DDEXParser();
  private maxRetries = 3;
  private retryDelay = 1000;

  async processWithRetry(xmlContent: string): Promise<ProcessingResult> {
    let lastError: Error | null = null;

    for (let attempt = 1; attempt <= this.maxRetries; attempt++) {
      try {
        const result = await this.parser.parse(xmlContent);
        
        return {
          success: true,
          data: result,
          attempts: attempt
        };

      } catch (error) {
        lastError = error as Error;
        
        console.warn(`Parsing attempt ${attempt} failed:`, error.message);
        
        if (attempt < this.maxRetries) {
          await this.delay(this.retryDelay * attempt);
        }
      }
    }

    return {
      success: false,
      error: lastError?.message || 'Unknown error',
      attempts: this.maxRetries
    };
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
```

### Circuit Breaker Pattern

```python
# Prevent cascade failures
import time
from enum import Enum

class CircuitState(Enum):
    CLOSED = "closed"      # Normal operation
    OPEN = "open"         # Failure mode
    HALF_OPEN = "half_open"  # Testing recovery

class DDEXCircuitBreaker:
    def __init__(self, failure_threshold=5, timeout=60):
        self.failure_threshold = failure_threshold
        self.timeout = timeout
        self.failure_count = 0
        self.last_failure_time = None
        self.state = CircuitState.CLOSED
    
    def call(self, func, *args, **kwargs):
        """Execute function with circuit breaker protection"""
        
        if self.state == CircuitState.OPEN:
            if self._should_attempt_reset():
                self.state = CircuitState.HALF_OPEN
            else:
                raise Exception("Circuit breaker is OPEN")
        
        try:
            result = func(*args, **kwargs)
            self._on_success()
            return result
            
        except Exception as e:
            self._on_failure()
            raise e
    
    def _should_attempt_reset(self):
        return (time.time() - self.last_failure_time) >= self.timeout
    
    def _on_success(self):
        self.failure_count = 0
        self.state = CircuitState.CLOSED
    
    def _on_failure(self):
        self.failure_count += 1
        self.last_failure_time = time.time()
        
        if self.failure_count >= self.failure_threshold:
            self.state = CircuitState.OPEN

# Usage
from ddex_parser import DDEXParser

circuit_breaker = DDEXCircuitBreaker(failure_threshold=3, timeout=30)
parser = DDEXParser()

try:
    result = circuit_breaker.call(parser.parse, xml_content)
except Exception as e:
    print(f"Circuit breaker prevented call: {e}")
```

## Caching Patterns

### LRU Cache Pattern

```typescript
// Cache frequently accessed parsed data
export class LRUCache<K, V> {
  private cache = new Map<K, V>();
  private maxSize: number;

  constructor(maxSize = 100) {
    this.maxSize = maxSize;
  }

  get(key: K): V | undefined {
    if (this.cache.has(key)) {
      // Move to end (most recently used)
      const value = this.cache.get(key)!;
      this.cache.delete(key);
      this.cache.set(key, value);
      return value;
    }
    return undefined;
  }

  set(key: K, value: V): void {
    if (this.cache.has(key)) {
      this.cache.delete(key);
    } else if (this.cache.size >= this.maxSize) {
      // Remove oldest entry
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
    
    this.cache.set(key, value);
  }

  clear(): void {
    this.cache.clear();
  }
}

// Cached DDEX processor
export class CachedDDEXProcessor {
  private parser = new DDEXParser();
  private cache = new LRUCache<string, any>(50);

  async parseWithCache(xmlContent: string): Promise<any> {
    // Use content hash as cache key
    const hash = this.calculateHash(xmlContent);
    
    // Check cache first
    const cached = this.cache.get(hash);
    if (cached) {
      console.log('Cache hit');
      return cached;
    }

    // Parse and cache result
    const result = await this.parser.parse(xmlContent);
    this.cache.set(hash, result);
    
    return result;
  }

  private calculateHash(content: string): string {
    // Simple hash function for demo
    let hash = 0;
    for (let i = 0; i < content.length; i++) {
      const char = content.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return hash.toString();
  }
}
```

## Factory Patterns

### Parser Factory Pattern

```python
# Create appropriate parser based on context
from abc import ABC, abstractmethod
from ddex_parser import DDEXParser

class DDEXProcessorFactory:
    """Factory for creating DDEX processors"""
    
    @staticmethod
    def create_processor(processor_type, **kwargs):
        """Create processor based on type"""
        
        if processor_type == 'streaming':
            return StreamingDDEXProcessor(**kwargs)
        elif processor_type == 'batch':
            return BatchDDEXProcessor(**kwargs)
        elif processor_type == 'memory_efficient':
            return MemoryEfficientProcessor(**kwargs)
        elif processor_type == 'high_performance':
            return HighPerformanceProcessor(**kwargs)
        else:
            return StandardDDEXProcessor(**kwargs)

class BaseDDEXProcessor(ABC):
    """Base class for DDEX processors"""
    
    def __init__(self):
        self.parser = DDEXParser()
    
    @abstractmethod
    def process(self, data):
        pass

class StreamingDDEXProcessor(BaseDDEXProcessor):
    def __init__(self, chunk_size=1024*1024):
        super().__init__()
        self.chunk_size = chunk_size
        self.parser.set_streaming_mode(True)
        self.parser.set_chunk_size(chunk_size)
    
    def process(self, file_path):
        """Process file in streaming mode"""
        with open(file_path, 'rb') as f:
            for chunk in self.parser.parse_streaming(f):
                yield self._process_chunk(chunk)
    
    def _process_chunk(self, chunk):
        # Process individual chunk
        return chunk

class BatchDDEXProcessor(BaseDDEXProcessor):
    def __init__(self, batch_size=10):
        super().__init__()
        self.batch_size = batch_size
    
    def process(self, file_paths):
        """Process files in batches"""
        for i in range(0, len(file_paths), self.batch_size):
            batch = file_paths[i:i + self.batch_size]
            yield self._process_batch(batch)
    
    def _process_batch(self, file_paths):
        # Process batch of files
        results = []
        for file_path in file_paths:
            with open(file_path, 'r') as f:
                result = self.parser.parse(f.read())
                results.append(result)
        return results

# Usage
processor = DDEXProcessorFactory.create_processor(
    'streaming',
    chunk_size=512*1024
)

for chunk_result in processor.process('large_file.xml'):
    print(f"Processed chunk: {len(chunk_result)} items")
```

## Observer Pattern for Events

```typescript
// Event-driven DDEX processing
interface DDEXEvent {
  type: string;
  data: any;
  timestamp: Date;
}

interface DDEXEventListener {
  onEvent(event: DDEXEvent): void;
}

export class DDEXEventProcessor {
  private listeners: Map<string, DDEXEventListener[]> = new Map();

  subscribe(eventType: string, listener: DDEXEventListener): void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, []);
    }
    this.listeners.get(eventType)!.push(listener);
  }

  unsubscribe(eventType: string, listener: DDEXEventListener): void {
    const listeners = this.listeners.get(eventType);
    if (listeners) {
      const index = listeners.indexOf(listener);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    }
  }

  private emit(event: DDEXEvent): void {
    const listeners = this.listeners.get(event.type) || [];
    listeners.forEach(listener => {
      try {
        listener.onEvent(event);
      } catch (error) {
        console.error(`Event listener error:`, error);
      }
    });
  }

  async processWithEvents(xmlContent: string): Promise<any> {
    this.emit({
      type: 'parsing_started',
      data: { contentLength: xmlContent.length },
      timestamp: new Date()
    });

    try {
      const result = await this.parser.parse(xmlContent);
      
      this.emit({
        type: 'parsing_completed',
        data: { 
          releases: result.flat.releases.length,
          tracks: result.flat.tracks.length 
        },
        timestamp: new Date()
      });

      return result;

    } catch (error) {
      this.emit({
        type: 'parsing_failed',
        data: { error: error.message },
        timestamp: new Date()
      });

      throw error;
    }
  }
}

// Event listeners
class LoggingListener implements DDEXEventListener {
  onEvent(event: DDEXEvent): void {
    console.log(`[${event.timestamp.toISOString()}] ${event.type}:`, event.data);
  }
}

class MetricsListener implements DDEXEventListener {
  onEvent(event: DDEXEvent): void {
    // Send metrics to monitoring system
    this.recordMetric(event.type, event.data);
  }

  private recordMetric(eventType: string, data: any): void {
    // Implementation for metrics recording
  }
}

// Usage
const processor = new DDEXEventProcessor();
processor.subscribe('parsing_started', new LoggingListener());
processor.subscribe('parsing_completed', new MetricsListener());

const result = await processor.processWithEvents(xmlContent);
```

## Best Practices

1. **Use Factory Pattern**: Create processors based on use case requirements
2. **Implement Circuit Breakers**: Prevent cascade failures in distributed systems
3. **Cache Strategically**: Cache parsed results for frequently accessed data
4. **Handle Errors Gracefully**: Implement retry logic with exponential backoff
5. **Use Observer Pattern**: Implement event-driven processing for better monitoring
6. **Batch Operations**: Group related operations for better performance
7. **Validate Progressively**: Start with basic validation and increase strictness
8. **Monitor Everything**: Emit events for all significant processing steps