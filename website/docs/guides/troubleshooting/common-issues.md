# Common Issues

Solutions to frequently encountered problems when using DDEX Suite.

## Installation Issues

### Node.js Native Binding Errors

**Problem**: Installation fails with "node-gyp" or "prebuild" errors.

```
Error: Cannot find module './build/Release/ddex_parser.node'
gyp ERR! build error
```

**Solutions**:

1. **Ensure compatible Node.js version** (18.0.0 or higher):
   ```bash
   node --version  # Should be >= 18.0.0
   npm install ddex-parser
   ```

2. **Clear npm cache and reinstall**:
   ```bash
   npm cache clean --force
   rm -rf node_modules package-lock.json
   npm install
   ```

3. **For ARM64 Macs (M1/M2)**:
   ```bash
   npm install ddex-parser --target_arch=arm64
   ```

4. **Install build tools if missing**:
   ```bash
   # macOS
   xcode-select --install
   
   # Ubuntu/Debian
   sudo apt-get install build-essential
   
   # Windows
   npm install --global windows-build-tools
   ```

### Python Package Installation Issues

**Problem**: Python bindings fail to install or import.

```python
ImportError: cannot import name 'DDEXParser' from 'ddex_parser'
```

**Solutions**:

1. **Check Python version** (3.8+ required):
   ```bash
   python --version  # Should be >= 3.8
   pip install ddex-parser
   ```

2. **Install with verbose output** to see specific errors:
   ```bash
   pip install --verbose ddex-parser
   ```

3. **For Apple Silicon Macs**:
   ```bash
   pip install ddex-parser --no-binary ddex-parser
   ```

4. **Update pip and setuptools**:
   ```bash
   pip install --upgrade pip setuptools wheel
   pip install ddex-parser
   ```

## Parsing Issues

### XML Parse Errors

**Problem**: "Invalid XML" errors when parsing seemingly valid DDEX files.

**Common Causes & Solutions**:

1. **BOM (Byte Order Mark) issues**:
   ```typescript
   // Remove BOM before parsing
   const cleanXml = xmlContent.replace(/^\uFEFF/, '');
   const result = await parser.parse(cleanXml);
   ```

2. **Encoding issues**:
   ```typescript
   // Ensure UTF-8 encoding
   import { readFileSync } from 'fs';
   const xmlContent = readFileSync('file.xml', 'utf-8');
   
   // Or explicitly handle encoding
   const buffer = readFileSync('file.xml');
   const xmlContent = buffer.toString('utf-8');
   ```

3. **Namespace declaration issues**:
   ```xml
   <!-- Ensure proper namespace -->
   <NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">
   <!-- Not -->
   <NewReleaseMessage>
   ```

### Memory Issues with Large Files

**Problem**: Out of memory errors when parsing large DDEX files.

```
FATAL ERROR: Ineffective mark-compacts near heap limit
```

**Solutions**:

1. **Use streaming mode**:
   ```typescript
   const result = await parser.parse(xmlContent, { 
     streaming: true,
     maxMemoryUsage: 500 * 1024 * 1024 // 500MB limit
   });
   ```

2. **Increase Node.js heap size**:
   ```bash
   node --max-old-space-size=4096 your-script.js
   ```

3. **Process in chunks** (see [Large File Processing](../parser/large-files) guide)

### Missing or Empty Data

**Problem**: Parsed result has empty or missing fields that exist in the XML.

**Debugging Steps**:

1. **Enable raw extensions**:
   ```typescript
   const result = await parser.parse(xmlContent, {
     includeRawExtensions: true
   });
   
   // Check if data is in extensions
   console.log('Extensions:', result.graph.extensions);
   ```

2. **Check validation errors**:
   ```typescript
   const result = await parser.parse(xmlContent, {
     validateReferences: true,
     collectErrors: true
   });
   
   if (result.hasErrors) {
     console.log('Parsing errors:', result.errors);
   }
   ```

3. **Verify XML structure** matches expected DDEX schema:
   ```bash
   # Use xmllint to validate structure
   xmllint --format your-file.xml
   ```

## Building Issues

### Validation Failures

**Problem**: Generated DDEX fails validation with cryptic error messages.

**Common Issues & Fixes**:

1. **Missing required fields**:
   ```typescript
   // Ensure all required fields are present
   const releaseData = {
     messageHeader: {
       messageId: 'MSG_001',           // Required
       messageSenderName: 'Label',     // Required
       messageRecipientName: 'DSP',   // Required
       messageCreatedDateTime: new Date().toISOString() // Required
     },
     releases: [{
       releaseId: 'REL_001',          // Required
       title: 'Album Title',          // Required
       artist: 'Artist Name',         // Required
       releaseType: 'Album',          // Required
       // ... other required fields
     }]
   };
   ```

2. **Invalid date formats**:
   ```typescript
   // Use ISO format for dates
   releaseDate: '2024-03-15',        // Correct: YYYY-MM-DD
   // Not: '03/15/2024' or '15-03-2024'
   
   messageCreatedDateTime: new Date().toISOString(), // Correct: full ISO
   // Not: new Date().toString()
   ```

3. **Invalid territory codes**:
   ```typescript
   // Use ISO 3166 territory codes
   territories: ['US', 'GB', 'DE'],  // Correct
   // Not: ['USA', 'UK', 'Germany']
   ```

### Deterministic Output Issues

**Problem**: Same input data generates different XML output each time.

**Solution**: Ensure stable ordering of arrays and objects:

```typescript
const builder = new DDEXBuilder({
  deterministicOutput: true,
  stableArraySorting: true
});

// Sort arrays consistently
const sortedReleases = releaseData.releases.sort((a, b) => 
  a.releaseId.localeCompare(b.releaseId)
);

const result = await builder.build({
  ...releaseData,
  releases: sortedReleases
});
```

## Performance Issues

### Slow Parsing Performance

**Problem**: Parsing takes much longer than expected benchmarks.

**Optimization Steps**:

1. **Profile the parsing operation**:
   ```typescript
   console.time('parsing');
   const result = await parser.parse(xmlContent);
   console.timeEnd('parsing');
   
   // Check memory usage
   const memUsage = process.memoryUsage();
   console.log('Memory used:', Math.round(memUsage.heapUsed / 1024 / 1024), 'MB');
   ```

2. **Disable unnecessary features**:
   ```typescript
   const result = await parser.parse(xmlContent, {
     includeRawExtensions: false,     // Skip if not needed
     includeComments: false,          // Skip XML comments
     validateReferences: false,       // Skip if data is trusted
     preserveWhitespace: false        // Strip whitespace
   });
   ```

3. **Use appropriate parsing mode**:
   ```typescript
   // For files < 10MB: use default mode
   const result = await parser.parse(xmlContent);
   
   // For files > 10MB: use streaming
   const result = await parser.parse(xmlContent, { streaming: true });
   ```

### Memory Leaks in Long-Running Processes

**Problem**: Memory usage grows over time when processing many files.

**Solutions**:

1. **Explicit cleanup**:
   ```typescript
   for (const file of files) {
     const result = await parser.parse(xmlContent);
     
     // Process result
     await processResult(result);
     
     // Explicit cleanup hint
     result.cleanup?.();
     
     // Force garbage collection periodically
     if (processedFiles % 100 === 0 && global.gc) {
       global.gc();
     }
   }
   ```

2. **Use worker threads** for isolation:
   ```typescript
   import { Worker } from 'worker_threads';
   
   const worker = new Worker('./parse-worker.js');
   worker.postMessage({ xmlContent });
   
   worker.on('message', (result) => {
     // Process result
     worker.terminate(); // Clean up worker
   });
   ```

## DDEX Specification Issues

### Version Compatibility Problems

**Problem**: DDEX file claims to be version X but has elements from version Y.

**Solutions**:

1. **Auto-detect version** instead of trusting headers:
   ```typescript
   const result = await parser.parse(xmlContent, {
     autoDetectVersion: true,
     fallbackVersion: '4.3'  // Use as fallback
   });
   
   console.log('Detected version:', result.version);
   ```

2. **Handle mixed versions** gracefully:
   ```typescript
   const result = await parser.parse(xmlContent, {
     strictVersionValidation: false,
     includeRawExtensions: true  // Capture unknown elements
   });
   ```

### Reference Resolution Failures

**Problem**: Internal references (ResourceReference, PartyReference, etc.) cannot be resolved.

**Common Causes**:

1. **Missing referenced elements**:
   ```xml
   <!-- ReleaseResourceReference points to A1, but no Resource has ResourceReference=A1 -->
   <ReleaseResourceReferenceList>
     <ReleaseResourceReference>A1</ReleaseResourceReference>
   </ReleaseResourceReferenceList>
   ```

2. **Case sensitivity issues**:
   ```xml
   <!-- References are case-sensitive -->
   <ResourceReference>A1</ResourceReference>
   <!-- Later referenced as -->
   <ReleaseResourceReference>a1</ReleaseResourceReference> <!-- Wrong! -->
   ```

**Solutions**:

```typescript
// Disable reference validation if data is incomplete
const result = await parser.parse(xmlContent, {
  validateReferences: false
});

// Or get detailed reference errors
const result = await parser.parse(xmlContent, {
  validateReferences: true,
  collectErrors: true
});

result.errors?.forEach(error => {
  if (error.type === 'REFERENCE_ERROR') {
    console.log(`Reference error: ${error.reference} -> ${error.target}`);
  }
});
```

## Integration Issues

### Database Integration Problems

**Problem**: Converting DDEX data to database format loses information or fails.

**Solutions**:

1. **Use flattened representation** for simpler database storage:
   ```typescript
   const result = await parser.parse(xmlContent);
   
   // Use flat representation for database
   const releases = result.flat.releases;
   const tracks = result.flat.tracks;
   
   // Store in database
   await db.releases.insertMany(releases);
   await db.tracks.insertMany(tracks);
   ```

2. **Handle JSON serialization** of complex objects:
   ```typescript
   // Store graph data as JSON
   const graphJson = JSON.stringify(result.graph);
   await db.ddex_messages.insert({
     id: result.messageId,
     graph_data: graphJson,
     created_at: new Date()
   });
   ```

### API Integration Issues

**Problem**: REST API responses are too large or complex for clients.

**Solutions**:

1. **Paginate large responses**:
   ```typescript
   app.get('/releases', async (req, res) => {
     const { page = 1, limit = 10 } = req.query;
     const offset = (page - 1) * limit;
     
     const releases = result.flat.releases
       .slice(offset, offset + limit);
     
     res.json({
       data: releases,
       pagination: {
         page: parseInt(page),
         limit: parseInt(limit),
         total: result.flat.releases.length
       }
     });
   });
   ```

2. **Provide different detail levels**:
   ```typescript
   app.get('/releases/:id', async (req, res) => {
     const { detail = 'basic' } = req.query;
     const release = findRelease(req.params.id);
     
     if (detail === 'full') {
       res.json(release); // Full DDEX data
     } else {
       res.json({        // Basic data only
         id: release.releaseId,
         title: release.title,
         artist: release.artist,
         releaseDate: release.releaseDate
       });
     }
   });
   ```

## Getting Help

### Debugging Steps

1. **Enable verbose logging**:
   ```typescript
   const parser = new DDEXParser({
     logLevel: 'debug',
     logOutput: './ddex-debug.log'
   });
   ```

2. **Create minimal reproduction** of the issue:
   ```typescript
   // Isolate the problematic XML snippet
   const minimalXml = `<?xml version="1.0" encoding="UTF-8"?>
   <NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">
     <!-- Minimal example that reproduces the issue -->
   </NewReleaseMessage>`;
   ```

3. **Check version compatibility**:
   ```bash
   npm list ddex-parser  # Check installed version
   npm view ddex-parser versions --json  # See available versions
   ```

### Reporting Issues

When reporting issues, include:

1. **Version information**:
   ```bash
   node --version
   npm list ddex-parser
   python --version  # If using Python
   pip show ddex-parser
   ```

2. **Sample data** (anonymized if necessary)

3. **Full error messages** and stack traces

4. **Environment details** (OS, architecture, etc.)

### Community Resources

- **GitHub Issues**: [ddex-suite/issues](https://github.com/daddykev/ddex-suite/issues)
- **Documentation**: [ddex-suite.web.app](https://ddex-suite.web.app)
- **DDEX Standards**: [ddex.net](https://ddex.net) for official specifications

## Next Steps

- [Performance Issues](./performance) - Advanced performance troubleshooting
- [Memory Issues](./memory) - Memory optimization and debugging  
- [Validation Errors](./validation-errors) - Understanding validation failures