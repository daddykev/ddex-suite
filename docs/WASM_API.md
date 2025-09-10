# WASM API Documentation

## DDEX Builder WASM API

The DDEX Builder WebAssembly module provides the same powerful DDEX generation capabilities as the Node.js and Python versions, optimized for browser environments.

### Installation

```bash
npm install ddex-builder
```

The WASM module is automatically included and will be used when running in a browser environment.

### Basic Usage

```javascript
import { DDEXBuilder } from 'ddex-builder';

// Initialize builder (automatically uses WASM in browser)
const builder = new DDEXBuilder();

// Build DDEX XML
const request = {
    header: {
        message_sender: {party_name: [{text: 'Web Music App'}]},
        message_recipient: {party_name: [{text: 'Streaming Service'}]}
    },
    version: '4.3',
    releases: [{
        release_id: '1234567890123',
        title: [{text: 'Web-Generated Album'}],
        display_artist: 'Browser Artist',
        tracks: [{
            position: 1,
            isrc: 'USWEB2400001',
            title: 'Client-Side Song',
            duration: 180
        }]
    }]
};

const result = await builder.build(request);
console.log(`Generated ${result.xml.length} bytes of DDEX XML`);
```

### Bundle Information

- **Bundle Size**: 114KB (compressed)
- **Compression**: 77% under 500KB target
- **Support**: All modern browsers with WASM support

### API Surface

The WASM API provides the same interface as the Node.js version:

#### DDEXBuilder Class

```typescript
class DDEXBuilder {
    constructor();
    build(request: BuildRequest): Promise<BuildResult>;
    validateStructure(request: BuildRequest): Promise<ValidationResult>;
}
```

#### StreamingDdexBuilder Class

```typescript
class StreamingDdexBuilder {
    constructor();
    addRelease(release: ReleaseRequest): Promise<void>;
    finalize(): Promise<BuildResult>;
}
```

#### Batch Functions

```typescript
function batchBuild(requests: BuildRequest[]): Promise<BuildResult[]>;
```

### Browser Compatibility

- Chrome 69+
- Firefox 60+
- Safari 13+
- Edge 79+

### Performance

- Initialization: ~5ms
- Small release (1 track): ~8ms
- Medium album (10 tracks): ~25ms
- Large catalog (100 releases): ~800ms

### Memory Usage

- Base memory: ~2MB
- Per release: ~50KB
- Streaming mode: Constant memory usage

### Example Applications

#### Music Web App Integration

```html
<!DOCTYPE html>
<html>
<head>
    <title>DDEX Generator</title>
</head>
<body>
    <div id="app">
        <h1>DDEX XML Generator</h1>
        <form id="releaseForm">
            <input name="title" placeholder="Album Title" required>
            <input name="artist" placeholder="Artist Name" required>
            <input name="upc" placeholder="UPC Code" required>
            <button type="submit">Generate DDEX</button>
        </form>
        <pre id="output"></pre>
    </div>

    <script type="module">
        import { DDEXBuilder } from 'ddex-builder';

        const builder = new DDEXBuilder();
        const form = document.getElementById('releaseForm');
        const output = document.getElementById('output');

        form.addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(e.target);
            
            try {
                const request = {
                    header: {
                        message_sender: {party_name: [{text: 'Web App'}]},
                        message_recipient: {party_name: [{text: 'Distribution'}]}
                    },
                    version: '4.3',
                    releases: [{
                        release_id: formData.get('upc'),
                        title: [{text: formData.get('title')}],
                        display_artist: formData.get('artist'),
                        tracks: [{
                            position: 1,
                            isrc: 'USWEB2400001',
                            title: 'Generated Track',
                            duration: 180
                        }]
                    }]
                };

                const result = await builder.build(request);
                output.textContent = result.xml;
                
                // Download the XML
                const blob = new Blob([result.xml], {type: 'application/xml'});
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = `${formData.get('title')}.xml`;
                a.click();
                
            } catch (error) {
                output.textContent = `Error: ${error.message}`;
            }
        });
    </script>
</body>
</html>
```

#### Streaming Catalog Processor

```javascript
import { StreamingDdexBuilder } from 'ddex-builder';

class CatalogProcessor {
    constructor() {
        this.builder = new StreamingDdexBuilder();
    }
    
    async processCatalogFile(file) {
        const reader = new FileReader();
        return new Promise((resolve, reject) => {
            reader.onload = async (e) => {
                try {
                    const data = JSON.parse(e.target.result);
                    
                    for (const release of data.releases) {
                        await this.builder.addRelease({
                            release_id: release.id,
                            title: [{text: release.title}],
                            display_artist: release.artist,
                            tracks: release.tracks.map((track, i) => ({
                                position: i + 1,
                                isrc: track.isrc,
                                title: track.title,
                                duration: track.duration
                            }))
                        });
                    }
                    
                    const result = await this.builder.finalize();
                    resolve(result);
                } catch (error) {
                    reject(error);
                }
            };
            reader.readAsText(file);
        });
    }
}

// Usage
const processor = new CatalogProcessor();
document.getElementById('fileInput').addEventListener('change', async (e) => {
    const file = e.target.files[0];
    if (file) {
        try {
            const result = await processor.processCatalogFile(file);
            console.log(`Generated DDEX XML: ${result.xml.length} bytes`);
        } catch (error) {
            console.error('Processing failed:', error);
        }
    }
});
```

### Error Handling

```javascript
try {
    const result = await builder.build(request);
    console.log('Success:', result.xml.length, 'bytes');
} catch (error) {
    if (error.name === 'ValidationError') {
        console.error('Validation failed:', error.details);
    } else if (error.name === 'BuildError') {
        console.error('Build failed:', error.message);
    } else {
        console.error('Unexpected error:', error);
    }
}
```

### Advanced Features

#### Custom Validation

```javascript
const result = await builder.validateStructure({
    releases: [{
        release_id: '1234567890123',
        title: [{text: 'Test Album'}],
        tracks: []  // This will trigger validation warnings
    }]
});

if (!result.isValid) {
    console.log('Validation errors:', result.errors);
}
```

#### Batch Processing

```javascript
import { batchBuild } from 'ddex-builder';

const requests = [
    { /* request 1 */ },
    { /* request 2 */ },
    { /* request 3 */ }
];

const results = await batchBuild(requests);
results.forEach((result, i) => {
    console.log(`Request ${i + 1}: ${result.xml.length} bytes`);
});
```

### Limitations

- No file system access (browser security)
- Memory constraints in mobile browsers
- No persistent storage (unless using IndexedDB)
- Single-threaded execution

### Best Practices

1. **Memory Management**: Use streaming builder for large catalogs
2. **Error Handling**: Always wrap calls in try-catch blocks  
3. **Progress Feedback**: Show loading indicators for large operations
4. **File Downloads**: Use Blob API for generated XML files
5. **Validation**: Validate inputs before processing

### TypeScript Support

Full TypeScript definitions are included:

```typescript
import { DDEXBuilder, BuildRequest, BuildResult } from 'ddex-builder';

const builder: DDEXBuilder = new DDEXBuilder();
const request: BuildRequest = { /* typed request */ };
const result: BuildResult = await builder.build(request);
```