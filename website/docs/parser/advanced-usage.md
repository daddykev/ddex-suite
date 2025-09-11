---
sidebar_position: 5
---

# Advanced Usage

Master advanced DDEX Parser features for production applications, including streaming for large files, performance optimization, and integration patterns.

## Streaming Large Files

When processing large DDEX catalogs (>10MB), streaming prevents memory issues and provides better performance.

### Basic Streaming

#### JavaScript / TypeScript

```typescript
import { DDEXParser } from 'ddex-parser';

async function processLargeCatalog(filePath: string) {
  const parser = new DDEXParser({ 
    streaming: true,
    bufferSize: 8192,
    maxMemoryMB: 100
  });
  
  let totalReleases = 0;
  let totalTracks = 0;
  
  for await (const batch of parser.streamFile(filePath)) {
    // Process each batch
    console.log(`Processing batch: ${batch.releases.length} releases`);
    
    // Your processing logic here
    await processBatch(batch);
    
    totalReleases += batch.releases.length;
    totalTracks += batch.soundRecordings.length;
    
    // Optional: Add delay to prevent overwhelming downstream systems
    await new Promise(resolve => setTimeout(resolve, 10));
  }
  
  console.log(`Processed ${totalReleases} releases, ${totalTracks} tracks`);
}

async function processBatch(batch: any) {
  // Insert into database, send to API, etc.
  for (const release of batch.releases) {
    await database.insertRelease(release);
  }
}
```

#### Python

```python
from ddex_parser import DDEXParser, ParseOptions
import asyncio

async def process_large_catalog(file_path: str):
    parser = DDEXParser()
    options = ParseOptions(
        streaming=True,
        max_memory_mb=100,
        timeout_seconds=300  # 5 minutes
    )
    
    total_releases = 0
    total_tracks = 0
    
    # Stream processing
    for batch in parser.stream(open(file_path, 'r').read(), options):
        releases = batch.get('releases', [])
        tracks = batch.get('sound_recordings', [])
        
        print(f"Processing batch: {len(releases)} releases")
        
        # Process batch
        await process_batch(releases, tracks)
        
        total_releases += len(releases)
        total_tracks += len(tracks)
        
        # Optional: Rate limiting
        await asyncio.sleep(0.01)
    
    print(f"Processed {total_releases} releases, {total_tracks} tracks")

async def process_batch(releases, tracks):
    # Your processing logic
    for release in releases:
        await database.insert_release(release)
```

### Advanced Streaming Patterns

#### Parallel Batch Processing

```typescript
import { DDEXParser } from 'ddex-parser';
import { Worker, isMainThread, parentPort, workerData } from 'worker_threads';

// Main thread: Stream and distribute batches
async function parallelStreamProcessor(filePath: string, numWorkers: number = 4) {
  if (!isMainThread) return;
  
  const parser = new DDEXParser({ streaming: true });
  const workers: Worker[] = [];
  const workQueue: any[] = [];
  
  // Create workers
  for (let i = 0; i < numWorkers; i++) {
    const worker = new Worker(__filename, {
      workerData: { workerId: i }
    });
    workers.push(worker);
  }
  
  // Distribute work
  let workerIndex = 0;
  for await (const batch of parser.streamFile(filePath)) {
    const worker = workers[workerIndex % workers.length];
    worker.postMessage({ type: 'process', batch });
    workerIndex++;
  }
  
  // Signal completion
  workers.forEach(worker => {
    worker.postMessage({ type: 'complete' });
    worker.terminate();
  });
}

// Worker thread: Process individual batches
if (!isMainThread) {
  const { workerId } = workerData;
  
  parentPort?.on('message', async ({ type, batch }) => {
    if (type === 'process') {
      console.log(`Worker ${workerId} processing ${batch.releases.length} releases`);
      await processBatchInWorker(batch);
      parentPort?.postMessage({ type: 'completed', workerId });
    } else if (type === 'complete') {
      process.exit(0);
    }
  });
}

async function processBatchInWorker(batch: any) {
  // CPU-intensive processing here
  for (const release of batch.releases) {
    // Transform, validate, enrich data
    const enrichedRelease = await enrichReleaseData(release);
    await sendToDatabase(enrichedRelease);
  }
}
```

#### Stream with Backpressure Control

```typescript
import { DDEXParser } from 'ddex-parser';
import { pipeline, Transform } from 'stream';

async function streamWithBackpressure(filePath: string) {
  const parser = new DDEXParser({ streaming: true });
  
  let inFlight = 0;
  const maxConcurrent = 10;
  
  for await (const batch of parser.streamFile(filePath)) {
    // Wait if too many concurrent operations
    while (inFlight >= maxConcurrent) {
      await new Promise(resolve => setTimeout(resolve, 10));
    }
    
    inFlight++;
    processBatchAsync(batch)
      .finally(() => inFlight--);
  }
  
  // Wait for all operations to complete
  while (inFlight > 0) {
    await new Promise(resolve => setTimeout(resolve, 10));
  }
}

async function processBatchAsync(batch: any) {
  try {
    // Async processing
    await Promise.all(
      batch.releases.map(release => processRelease(release))
    );
  } catch (error) {
    console.error('Batch processing error:', error);
  }
}
```

## Graph vs Flattened Models

Understanding when to use each data representation:

### Graph Model Use Cases

Use the graph model when you need:
- **Compliance**: Exact DDEX structure preservation
- **Round-trip fidelity**: Parse → modify → build workflows  
- **Reference resolution**: Working with DDEX references and IDs
- **Extension access**: Custom DDEX extensions and namespaces

```typescript
// Graph model: Faithful DDEX structure
const result = await parser.parseFile('release.xml');

// Access original structure
const messageHeader = result.graph.messageHeader;
const parties = result.graph.partyList.party;
const releases = result.graph.releaseList.release;

// Resolve party references
const labelParty = parties.find(p => 
  p.partyReference === releases[0].releaseDetailsByTerritory[0].labelName.partyReference
);

// Access extensions
if (result.graph.extensions) {
  const customData = result.graph.extensions['custom:namespace'];
}
```

### Flattened Model Use Cases

Use the flattened model when you need:
- **Rapid development**: Quick access to common fields
- **Analytics**: Data analysis and reporting
- **API responses**: Clean JSON for web applications
- **Database insertion**: Direct mapping to relational schemas

```typescript
// Flattened model: Developer-friendly
const result = await parser.parseFile('release.xml');

// Direct access to denormalized data
const releases = result.flat.releases;
releases.forEach(release => {
  console.log(`${release.title} by ${release.displayArtist}`);
  console.log(`Label: ${release.label}`);
  console.log(`Territories: ${release.territories.join(', ')}`);
  
  // Perfect for database insertion
  database.insertRelease({
    title: release.title,
    artist: release.displayArtist,
    label: release.label,
    release_date: release.releaseDate,
    territories: release.territories
  });
});
```

### Model Conversion and Switching

```typescript
// Parse once, use both models
const result = await parser.parseFile('release.xml');

// Use graph model for compliance checks
function validateCompliance(graph: GraphModel): string[] {
  const errors = [];
  
  if (!graph.messageHeader.messageId) {
    errors.push('Missing required MessageId');
  }
  
  if (graph.releaseList.release.length === 0) {
    errors.push('No releases found');
  }
  
  return errors;
}

// Use flattened model for API response
function createApiResponse(flat: FlattenedModel) {
  return {
    releases: flat.releases.map(r => ({
      title: r.title,
      artist: r.displayArtist,
      tracks: flat.soundRecordings.filter(t => 
        r.soundRecordingReferences.includes(t.reference)
      ).length
    }))
  };
}

const complianceErrors = validateCompliance(result.graph);
const apiData = createApiResponse(result.flat);
```

## Extension Preservation

DDEX files often contain custom extensions. The parser preserves these for round-trip compatibility.

### Handling Custom Extensions

```typescript
// Parse with extension preservation
const parser = new DDEXParser({
  includeRawExtensions: true,
  validation: 'permissive' // Allow non-standard elements
});

const result = await parser.parseFile('extended-release.xml');

// Access preserved extensions
if (result.graph.extensions) {
  // Custom namespace data
  const spotifyData = result.graph.extensions['spotify:metadata'];
  const appleData = result.graph.extensions['itunes:info'];
  
  console.log('Spotify playlist ID:', spotifyData?.playlistId);
  console.log('Apple Music ID:', appleData?.adamId);
}

// Extensions are also available in individual elements
result.graph.releaseList.release.forEach(release => {
  if (release.extensions) {
    const customReleaseData = release.extensions['custom:releaseInfo'];
    console.log('Custom release data:', customReleaseData);
  }
});
```

### Round-Trip with Extensions

```typescript
import { DDEXParser } from 'ddex-parser';
import { DDEXBuilder } from 'ddex-builder';

async function roundTripWithExtensions(originalXml: string) {
  // Parse with full extension preservation
  const parser = new DDEXParser({
    includeRawExtensions: true,
    includeComments: true,
    validation: 'permissive'
  });
  
  const parsed = await parser.parseString(originalXml);
  
  // Modify data while preserving extensions
  parsed.flat.releases[0].title = "Updated Title";
  
  // Build back to XML with extensions intact
  const builder = new DDEXBuilder({
    preserveExtensions: true,
    deterministic: true
  });
  
  const newXml = await builder.build(parsed.toBuildRequest());
  
  // Verify round-trip fidelity
  const reparsed = await parser.parseString(newXml);
  
  console.log('Original extensions preserved:', 
    JSON.stringify(reparsed.graph.extensions) === JSON.stringify(parsed.graph.extensions)
  );
  
  return newXml;
}
```

## Performance Optimization

### Memory Management

```typescript
// Configure memory limits
const parser = new DDEXParser({
  maxMemoryMB: 256,           // Hard memory limit
  streaming: true,            // Enable streaming for large files
  bufferSize: 16384,         // Larger buffer for better I/O
  parallelProcessing: false   // Disable if memory-constrained
});

// Monitor memory usage
async function parseWithMemoryMonitoring(filePath: string) {
  const initialMemory = process.memoryUsage();
  
  try {
    const result = await parser.parseFile(filePath);
    
    const finalMemory = process.memoryUsage();
    const usedMB = (finalMemory.heapUsed - initialMemory.heapUsed) / 1024 / 1024;
    
    console.log(`Memory used: ${usedMB.toFixed(2)} MB`);
    console.log(`Peak memory: ${(finalMemory.heapTotal / 1024 / 1024).toFixed(2)} MB`);
    
    return result;
  } catch (error) {
    if (error.message.includes('memory')) {
      console.error('Out of memory. Try streaming mode or increase memory limit.');
    }
    throw error;
  }
}
```

### CPU Optimization

```typescript
// CPU-optimized settings
const fastParser = new DDEXParser({
  validation: 'none',         // Skip validation for trusted sources
  includeComments: false,     // Skip comment preservation
  includeRawExtensions: false, // Skip extension preservation  
  parallelProcessing: true,   // Use multiple cores
  caching: true              // Cache parsed schemas
});

// Batch processing optimization
async function optimizedBatchProcessing(files: string[]) {
  // Pre-warm parser cache
  const warmupXml = generateMinimalDDEX();
  await fastParser.parseString(warmupXml);
  
  // Process files in optimal batch sizes
  const batchSize = Math.min(10, Math.ceil(files.length / 4));
  
  for (let i = 0; i < files.length; i += batchSize) {
    const batch = files.slice(i, i + batchSize);
    
    await Promise.all(
      batch.map(async file => {
        const start = performance.now();
        const result = await fastParser.parseFile(file);
        const duration = performance.now() - start;
        
        console.log(`${file}: ${duration.toFixed(2)}ms`);
        return result;
      })
    );
  }
}
```

### Caching Strategies

```typescript
import { DDEXParser } from 'ddex-parser';
import { createHash } from 'crypto';

class CachingParser {
  private parser: DDEXParser;
  private cache = new Map<string, any>();
  private maxCacheSize = 100;
  
  constructor() {
    this.parser = new DDEXParser({ caching: true });
  }
  
  async parseWithCache(xml: string): Promise<any> {
    // Generate cache key
    const hash = createHash('sha256').update(xml).digest('hex');
    
    // Check cache
    if (this.cache.has(hash)) {
      console.log('Cache hit');
      return this.cache.get(hash);
    }
    
    // Parse and cache
    const result = await this.parser.parseString(xml);
    
    // Manage cache size
    if (this.cache.size >= this.maxCacheSize) {
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
    
    this.cache.set(hash, result);
    console.log('Parsed and cached');
    
    return result;
  }
  
  clearCache() {
    this.cache.clear();
  }
}
```

## Production Deployment Patterns

### Microservice Architecture

```typescript
// DDEX Parser Service
import express from 'express';
import { DDEXParser } from 'ddex-parser';
import multer from 'multer';

const app = express();
const parser = new DDEXParser({
  validation: 'strict',
  maxMemoryMB: 512,
  timeoutSeconds: 30
});

// Configure file upload
const upload = multer({ 
  limits: { fileSize: 100 * 1024 * 1024 }, // 100MB
  fileFilter: (req, file, cb) => {
    cb(null, file.mimetype === 'text/xml' || file.originalname.endsWith('.xml'));
  }
});

// Parse endpoint
app.post('/parse', upload.single('ddex'), async (req, res) => {
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No DDEX file uploaded' });
    }
    
    const xmlContent = req.file.buffer.toString('utf-8');
    const result = await parser.parseString(xmlContent);
    
    res.json({
      success: true,
      data: {
        messageId: result.messageId,
        version: result.version,
        releases: result.flat.releases.length,
        tracks: result.flat.soundRecordings.length
      },
      metadata: {
        fileSize: req.file.size,
        processedAt: new Date().toISOString()
      }
    });
  } catch (error) {
    console.error('Parse error:', error);
    res.status(400).json({
      success: false,
      error: error.message,
      type: error.constructor.name
    });
  }
});

// Streaming endpoint for large files
app.post('/parse/stream', upload.single('ddex'), async (req, res) => {
  try {
    const xmlContent = req.file.buffer.toString('utf-8');
    
    res.writeHead(200, {
      'Content-Type': 'application/json',
      'Transfer-Encoding': 'chunked'
    });
    
    for await (const batch of parser.stream(xmlContent)) {
      const chunk = JSON.stringify({ 
        type: 'batch', 
        data: batch 
      }) + '\n';
      res.write(chunk);
    }
    
    res.end(JSON.stringify({ type: 'complete' }) + '\n');
  } catch (error) {
    res.end(JSON.stringify({ type: 'error', error: error.message }) + '\n');
  }
});

app.listen(3000, () => {
  console.log('DDEX Parser service running on port 3000');
});
```

### Event-Driven Processing

```python
import asyncio
import aioredis
from ddex_parser import DDEXParser
import json

class DDEXProcessor:
    def __init__(self, redis_url: str):
        self.parser = DDEXParser()
        self.redis = None
    
    async def start(self):
        """Start the event processor."""
        self.redis = await aioredis.from_url(redis_url)
        
        # Listen for DDEX processing jobs
        while True:
            try:
                # Block for up to 1 second for new jobs
                result = await self.redis.brpop(['ddex:queue'], timeout=1)
                
                if result:
                    queue_name, job_data = result
                    job = json.loads(job_data)
                    await self.process_job(job)
                    
            except Exception as e:
                print(f"Error processing job: {e}")
                await asyncio.sleep(1)
    
    async def process_job(self, job: dict):
        """Process a DDEX parsing job."""
        job_id = job['id']
        xml_content = job['xml']
        
        try:
            # Update job status
            await self.redis.hset(f"ddex:job:{job_id}", "status", "processing")
            
            # Parse DDEX
            result = await self.parser.parse_async(xml_content)
            
            # Store results
            result_data = {
                'message_id': result.message_id,
                'version': result.version,
                'releases': len(result.releases),
                'processed_at': datetime.utcnow().isoformat()
            }
            
            await self.redis.hset(f"ddex:job:{job_id}", mapping={
                "status": "completed",
                "result": json.dumps(result_data)
            })
            
            # Emit completion event
            await self.redis.lpush('ddex:completed', json.dumps({
                'job_id': job_id,
                'result': result_data
            }))
            
            print(f"✅ Completed job {job_id}")
            
        except Exception as e:
            await self.redis.hset(f"ddex:job:{job_id}", mapping={
                "status": "failed",
                "error": str(e)
            })
            
            print(f"❌ Failed job {job_id}: {e}")

# Start processor
if __name__ == "__main__":
    processor = DDEXProcessor("redis://localhost:6379")
    asyncio.run(processor.start())
```

### Database Integration

```typescript
import { DDEXParser } from 'ddex-parser';
import { Pool } from 'pg';

class DDEXDatabaseIntegrator {
  private parser: DDEXParser;
  private db: Pool;
  
  constructor(dbConfig: any) {
    this.parser = new DDEXParser({
      validation: 'strict',
      streaming: true
    });
    this.db = new Pool(dbConfig);
  }
  
  async processFile(filePath: string): Promise<void> {
    const client = await this.db.connect();
    
    try {
      await client.query('BEGIN');
      
      for await (const batch of this.parser.streamFile(filePath)) {
        await this.processBatch(client, batch);
      }
      
      await client.query('COMMIT');
      console.log('✅ File processed successfully');
      
    } catch (error) {
      await client.query('ROLLBACK');
      console.error('❌ Processing failed:', error);
      throw error;
    } finally {
      client.release();
    }
  }
  
  private async processBatch(client: any, batch: any): Promise<void> {
    // Insert releases
    for (const release of batch.releases) {
      await client.query(`
        INSERT INTO releases (title, artist, label, release_date, territories, genres)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (title, artist) DO UPDATE SET
          release_date = EXCLUDED.release_date,
          territories = EXCLUDED.territories
      `, [
        release.title,
        release.displayArtist,
        release.label,
        release.releaseDate,
        JSON.stringify(release.territories),
        JSON.stringify(release.genres)
      ]);
    }
    
    // Insert tracks
    for (const track of batch.soundRecordings) {
      await client.query(`
        INSERT INTO tracks (title, artist, isrc, duration, territories)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (isrc) DO UPDATE SET
          title = EXCLUDED.title,
          artist = EXCLUDED.artist
      `, [
        track.title,
        track.displayArtist,
        track.isrc,
        track.duration,
        JSON.stringify(track.territories)
      ]);
    }
    
    // Insert deals
    for (const deal of batch.deals) {
      await client.query(`
        INSERT INTO deals (commercial_model, territories, use_types, valid_from, valid_until)
        VALUES ($1, $2, $3, $4, $5)
      `, [
        deal.commercialModelType,
        JSON.stringify(deal.territories),
        JSON.stringify(deal.useTypes),
        deal.validityPeriod.startDate,
        deal.validityPeriod.endDate
      ]);
    }
  }
}

// Usage
const integrator = new DDEXDatabaseIntegrator({
  host: 'localhost',
  port: 5432,
  database: 'music_catalog',
  user: 'ddex_user',
  password: 'password'
});

await integrator.processFile('large-catalog.xml');
```

## Error Recovery and Resilience

### Retry Strategies

```typescript
import { DDEXParser, DDEXError, SecurityError } from 'ddex-parser';

class ResilientParser {
  private parser: DDEXParser;
  private maxRetries: number = 3;
  private retryDelay: number = 1000;
  
  constructor() {
    this.parser = new DDEXParser({
      validation: 'strict',
      timeoutSeconds: 30
    });
  }
  
  async parseWithRetry(xml: string): Promise<any> {
    let lastError: Error;
    
    for (let attempt = 1; attempt <= this.maxRetries; attempt++) {
      try {
        return await this.parser.parseString(xml);
      } catch (error) {
        lastError = error;
        
        // Don't retry security errors or validation errors
        if (error instanceof SecurityError || error instanceof ValidationError) {
          throw error;
        }
        
        if (attempt < this.maxRetries) {
          console.log(`Parse attempt ${attempt} failed, retrying in ${this.retryDelay}ms...`);
          await this.delay(this.retryDelay * attempt); // Exponential backoff
        }
      }
    }
    
    throw new Error(`Failed to parse after ${this.maxRetries} attempts: ${lastError.message}`);
  }
  
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// Circuit breaker pattern
class CircuitBreakerParser {
  private parser: DDEXParser;
  private failureCount = 0;
  private failureThreshold = 5;
  private resetTimeout = 60000; // 1 minute
  private state: 'closed' | 'open' | 'half-open' = 'closed';
  private nextAttempt = 0;
  
  constructor() {
    this.parser = new DDEXParser();
  }
  
  async parse(xml: string): Promise<any> {
    if (this.state === 'open') {
      if (Date.now() < this.nextAttempt) {
        throw new Error('Circuit breaker is OPEN');
      } else {
        this.state = 'half-open';
      }
    }
    
    try {
      const result = await this.parser.parseString(xml);
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }
  
  private onSuccess() {
    this.failureCount = 0;
    this.state = 'closed';
  }
  
  private onFailure() {
    this.failureCount++;
    
    if (this.failureCount >= this.failureThreshold) {
      this.state = 'open';
      this.nextAttempt = Date.now() + this.resetTimeout;
    }
  }
}
```

These advanced patterns help you build robust, production-ready applications with the DDEX Parser. For integration with the DDEX Builder for complete round-trip workflows, see the [Builder Documentation](../builder/).