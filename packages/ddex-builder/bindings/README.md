# DDEX Builder Language Bindings

This directory contains language bindings for the DDEX Builder, providing native integration with Node.js, Python, and WebAssembly environments.

## Overview

The DDEX Builder bindings offer consistent APIs across different platforms while respecting each language's conventions:

- **Node.js** - Native performance using napi-rs
- **Python** - Full integration with pandas DataFrames using PyO3  
- **WASM** - Browser-compatible with <500KB bundle size

All bindings support the same core functionality:
- Building deterministic DDEX XML with DB-C14N/1.0 canonicalization
- Adding releases and resources with metadata
- Validation and statistics reporting
- Batch processing capabilities

## Quick Start

### Node.js Binding

```bash
cd node/
npm install
npm test
```

```javascript
const { DdexBuilder } = require('ddex-builder');

const builder = new DdexBuilder();

// Add a release
const release = {
  releaseId: 'REL001',
  releaseType: 'Album',
  title: 'My Album',
  artist: 'My Artist',
  label: 'My Label',
  upc: '123456789012'
};
await builder.addRelease(release);

// Add a resource
const resource = {
  resourceId: 'RES001', 
  resourceType: 'SoundRecording',
  title: 'My Track',
  artist: 'My Artist',
  isrc: 'USRC17607839',
  duration: 'PT3M45S'
};
await builder.addResource(resource);

// Build DDEX XML
const xml = await builder.build();
console.log(xml);

// Get statistics
const stats = await builder.getStats();
console.log(`Generated ${stats.xmlSizeBytes} bytes in ${stats.generationTimeMs}ms`);
```

### Python Binding

```bash
cd python/
pip install maturin
maturin develop
python test_builder.py
```

```python
import ddex_builder
import pandas as pd

# Create builder
builder = ddex_builder.DdexBuilder()

# Add release from dictionary
release_data = {
    'release_id': 'REL001',
    'release_type': 'Album', 
    'title': 'My Album',
    'artist': 'My Artist',
    'label': 'My Label',
    'upc': '123456789012'
}
builder.add_release(**release_data)

# Add resource from DataFrame
df = pd.DataFrame([{
    'resource_id': 'RES001',
    'resource_type': 'SoundRecording',
    'title': 'Track 1',
    'artist': 'My Artist',
    'isrc': 'USRC17607839',
    'duration': 'PT3M45S'
}])
builder.from_dataframe(df, profile='AudioAlbum')

# Build XML
xml = builder.build()
print(f"Generated XML: {len(xml)} characters")

# Validate
result = builder.validate()
if result.is_valid:
    print("✅ Validation passed")
else:
    print("❌ Validation failed:", result.errors)
```

### WASM Binding

```bash
cd wasm/
wasm-pack build --target web --out-dir pkg
python3 -m http.server 8080
# Open http://localhost:8080/test.html
```

```javascript
import init, { WasmDdexBuilder, Release, Resource } from './pkg/ddex_builder_wasm.js';

async function main() {
    // Initialize WASM
    await init();
    
    // Create builder
    const builder = new WasmDdexBuilder();
    
    // Add release
    const release = new Release('REL001', 'Album', 'My Album', 'My Artist');
    release.label = 'My Label';
    release.upc = '123456789012';
    builder.addRelease(release);
    
    // Add resource
    const resource = new Resource('RES001', 'SoundRecording', 'My Track', 'My Artist');
    resource.isrc = 'USRC17607839';
    resource.duration = 'PT3M45S';
    builder.addResource(resource);
    
    // Build XML
    const xml = await builder.build();
    console.log('Generated XML:', xml);
    
    // Get statistics
    const stats = builder.getStats();
    console.log(`Bundle: ${stats.last_build_size_bytes} bytes, Build time: ${stats.total_build_time_ms}ms`);
}

main();
```

## API Reference

### Core Methods

All bindings provide these core methods with language-appropriate naming conventions:

| Method | Node.js | Python | WASM | Description |
|--------|---------|--------|------|-------------|
| Constructor | `new DdexBuilder()` | `DdexBuilder()` | `new WasmDdexBuilder()` | Create new builder |
| Add Release | `addRelease(release)` | `add_release(**kwargs)` | `addRelease(release)` | Add release metadata |
| Add Resource | `addResource(resource)` | `add_resource(**kwargs)` | `addResource(resource)` | Add resource/track |
| Build | `build()` | `build()` | `build()` | Generate DDEX XML |
| Validate | `validate()` | `validate()` | `validate()` | Validate current data |
| Statistics | `getStats()` | `get_stats()` | `getStats()` | Get build statistics |
| Reset | `reset()` | `reset()` | `reset()` | Clear all data |

### Data Structures

#### Release Object
```javascript
// Node.js/WASM
{
  releaseId: string,
  releaseType: string, // 'Album', 'Single', etc.
  title: string,
  artist: string,
  label?: string,
  catalogNumber?: string,
  upc?: string,
  releaseDate?: string, // ISO 8601
  genre?: string,
  parentalWarning?: boolean
}
```

```python
# Python (snake_case)
{
  'release_id': str,
  'release_type': str, 
  'title': str,
  'artist': str,
  'label': str | None,
  'catalog_number': str | None,
  'upc': str | None,
  'release_date': str | None,  # ISO 8601
  'genre': str | None,
  'parental_warning': bool | None
}
```

#### Resource Object
```javascript
// Node.js/WASM
{
  resourceId: string,
  resourceType: string, // 'SoundRecording', 'Video', etc.
  title: string,
  artist: string,
  isrc?: string,
  duration?: string, // ISO 8601 duration
  trackNumber?: number,
  volumeNumber?: number
}
```

```python
# Python (snake_case)
{
  'resource_id': str,
  'resource_type': str,
  'title': str, 
  'artist': str,
  'isrc': str | None,
  'duration': str | None,  # ISO 8601 duration
  'track_number': int | None,
  'volume_number': int | None
}
```

#### Statistics Object
```javascript
// Node.js/WASM
{
  releasesCount: number,
  resourcesCount: number,
  totalBuildTimeMs: number,
  lastBuildSizeBytes: number,
  validationErrors: number,
  validationWarnings: number
}
```

```python
# Python (snake_case)
{
  'releases_count': int,
  'resources_count': int,
  'total_build_time_ms': float,
  'last_build_size_bytes': int,
  'validation_errors': int,
  'validation_warnings': int
}
```

## Advanced Usage

### Batch Processing

#### Node.js
```javascript
// Batch build multiple releases
const requests = [
  { releaseId: 'REL001', title: 'Album 1', artist: 'Artist 1' },
  { releaseId: 'REL002', title: 'Album 2', artist: 'Artist 2' },
];

const results = await batchBuild(requests);
console.log(`Generated ${results.length} XML documents`);
```

#### Python
```python
# Process DataFrame with multiple releases
df = pd.DataFrame([
    {'release_id': 'REL001', 'title': 'Album 1', 'artist': 'Artist 1'},
    {'release_id': 'REL002', 'title': 'Album 2', 'artist': 'Artist 2'}
])

builder.from_dataframe(df, profile='AudioAlbum')
xml = builder.build()
```

#### WASM
```javascript
// Batch processing in browser
import { batchBuild } from './pkg/ddex_builder_wasm.js';

const requests = [/* ... */];
const results = await batchBuild(requests);
```

### Validation

```javascript
// Node.js
const result = await builder.validate();
if (!result.isValid) {
  result.errors.forEach(error => {
    console.error(`Error ${error.code}: ${error.message} at ${error.location}`);
  });
}
```

```python
# Python
result = builder.validate()
if not result.is_valid:
    for error in result.errors:
        print(f"Error {error.code}: {error.message} at {error.location}")
```

### XML Structure Validation

```javascript
// Validate XML structure (all bindings)
const xml = "<?xml version='1.0'?><test>content</test>";
const result = validateStructure(xml);
console.log('Valid XML:', result.isValid);
```

## Performance Guidelines

### Bundle Sizes
- **Node.js**: ~2MB native module
- **Python**: ~3MB wheel package  
- **WASM**: 116KB (optimized for web)

### Build Performance
- Typical release: <15ms build time
- 100 tracks: <50ms build time
- Memory usage: <10MB for large catalogs

### Platform Support

| Platform | Node.js | Python | WASM |
|----------|---------|---------|------|
| Windows | ✅ | ✅ | ✅ |
| macOS | ✅ | ✅ | ✅ |  
| Linux | ✅ | ✅ | ✅ |
| Browser | ❌ | ❌ | ✅ |

### Version Requirements
- **Node.js**: 16.x, 18.x, 20.x
- **Python**: 3.8+
- **Browsers**: Modern browsers with WASM support

## Integration Examples

### Express.js Server
```javascript
const express = require('express');
const { DdexBuilder } = require('ddex-builder');

const app = express();

app.post('/build-ddex', async (req, res) => {
  const builder = new DdexBuilder();
  await builder.addRelease(req.body.release);
  
  const xml = await builder.build();
  res.set('Content-Type', 'application/xml');
  res.send(xml);
});
```

### Django View
```python
from django.http import HttpResponse
import ddex_builder
import json

def build_ddex(request):
    data = json.loads(request.body)
    
    builder = ddex_builder.DdexBuilder()
    builder.add_release(**data['release'])
    
    xml = builder.build()
    return HttpResponse(xml, content_type='application/xml')
```

### React Component
```jsx
import { useEffect, useState } from 'react';
import init, { WasmDdexBuilder, Release } from './ddex_builder_wasm';

function DdexGenerator() {
  const [builder, setBuilder] = useState(null);
  
  useEffect(() => {
    init().then(() => {
      setBuilder(new WasmDdexBuilder());
    });
  }, []);
  
  const generateXML = async () => {
    if (!builder) return;
    
    const release = new Release('REL001', 'Album', 'Test', 'Artist');
    builder.addRelease(release);
    
    const xml = await builder.build();
    console.log('Generated:', xml);
  };
  
  return <button onClick={generateXML}>Generate DDEX</button>;
}
```

## Troubleshooting

### Common Issues

1. **Node.js Module Not Found**
   ```bash
   # Rebuild native module
   npm rebuild
   npm run build
   ```

2. **Python Import Error**
   ```bash
   # Reinstall with maturin
   pip uninstall ddex-builder
   maturin develop --release
   ```

3. **WASM Bundle Size Too Large**
   - Check `wasm-opt = false` in Cargo.toml
   - Use `opt-level = "s"` for size optimization
   - Disable unused features

4. **CORS Issues with WASM**
   ```javascript
   // Serve from same origin or configure CORS
   python3 -m http.server 8080
   ```

### Debug Mode

Enable debug logging in all bindings:

```javascript
// Node.js
process.env.DDEX_DEBUG = '1';
```

```python
# Python
import os
os.environ['DDEX_DEBUG'] = '1'
```

```javascript
// WASM (check browser console)
console.log('WASM debug enabled');
```

## Contributing

See the main repository's contribution guidelines. When adding binding features:

1. Implement in all three bindings consistently
2. Add appropriate tests for each platform
3. Update this README with new usage examples
4. Ensure bundle size targets are maintained

## License

MIT License - see main repository for details.