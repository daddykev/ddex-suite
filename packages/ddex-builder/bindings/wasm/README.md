# DDEX Builder WebAssembly (WASM) Bindings

[![npm version](https://badge.fury.io/js/ddex-builder-wasm.svg)](https://badge.fury.io/js/ddex-builder-wasm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

High-performance DDEX XML building for the browser and Node.js environments using WebAssembly. This package provides deterministic DDEX XML generation with near-native performance in JavaScript environments.

## Features

- ✅ **Browser-Ready**: Runs directly in modern browsers with WebAssembly support
- ✅ **High Performance**: Near-native speed for large DDEX XML generation (10MB in <100ms)
- ✅ **Small Bundle**: Optimized WASM bundle at ~114KB (77% under 500KB target)
- ✅ **Memory Efficient**: Streaming operations with controlled memory usage
- ✅ **Deterministic**: Identical XML output across builds and environments
- ✅ **Standards Compliant**: Full ERN 3.8.2, 4.2, and 4.3 support
- ✅ **TypeScript**: Full TypeScript definitions included

## Installation

### Prerequisites

First, install `wasm-pack` for building:

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Or via npm
npm install -g wasm-pack

# Or via cargo
cargo install wasm-pack
```

### NPM Installation

```bash
npm install ddex-builder-wasm
```

### CDN Installation

```html
<!-- ES modules -->
<script type="module">
  import init, { DdexBuilder } from 'https://unpkg.com/ddex-builder-wasm@latest/ddex_builder_wasm.js';
</script>

<!-- UMD -->
<script src="https://unpkg.com/ddex-builder-wasm@latest/ddex_builder_wasm.js"></script>
```

## Quick Start

### Basic Browser Usage

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>DDEX Builder WASM Example</title>
</head>
<body>
    <script type="module">
        import init, { DdexBuilder, Release } from './pkg/ddex_builder_wasm.js';
        
        async function run() {
            // Initialize the WASM module
            await init();
            
            // Create a builder instance
            const builder = new DdexBuilder();
            
            // Create a release
            const release = new Release();
            release.release_id = "REL001";
            release.title = "My Album";
            release.artist = "My Artist";
            
            // Build DDEX XML
            const xml = builder.build_release(release);
            console.log(xml);
        }
        
        run();
    </script>
</body>
</html>
```

### Node.js Usage

```javascript
const { DdexBuilder, Release } = require('ddex-builder-wasm');

async function buildDdex() {
    const builder = new DdexBuilder();
    
    const release = new Release();
    release.release_id = "REL001";
    release.title = "My Album";
    
    const xml = builder.build_release(release);
    return xml;
}

buildDdex().then(xml => {
    console.log('Generated DDEX XML:', xml);
});
```

## Build Commands

### Web Target (Recommended for Browsers)

```bash
# Standard web build
wasm-pack build --target web --out-dir pkg

# Optimized production build
wasm-pack build --target web --out-dir pkg --release

# Development build with debug info
wasm-pack build --target web --out-dir pkg --dev --debug
```

### Node.js Target

```bash
# Node.js CommonJS modules
wasm-pack build --target nodejs --out-dir pkg-node

# Node.js with ES6 modules
wasm-pack build --target web --out-dir pkg-esm
```

### Bundler Target (Webpack, Rollup, etc.)

```bash
# For bundlers that can handle ES6 modules
wasm-pack build --target bundler --out-dir pkg-bundler

# With specific features
wasm-pack build --target bundler --out-dir pkg-bundler --release -- --features "deterministic"
```

### No-Modules Target (UMD)

```bash
# UMD build for legacy browsers
wasm-pack build --target no-modules --out-dir pkg-umd
```

## Integration Examples

### React Integration

```jsx
// hooks/useDdexBuilder.js
import { useEffect, useState } from 'react';
import init, { DdexBuilder } from 'ddex-builder-wasm';

export function useDdexBuilder() {
    const [builder, setBuilder] = useState(null);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState(null);
    
    useEffect(() => {
        let isMounted = true;
        
        async function initWasm() {
            try {
                await init();
                if (isMounted) {
                    setBuilder(new DdexBuilder());
                    setIsLoading(false);
                }
            } catch (err) {
                if (isMounted) {
                    setError(err);
                    setIsLoading(false);
                }
            }
        }
        
        initWasm();
        
        return () => {
            isMounted = false;
        };
    }, []);
    
    return { builder, isLoading, error };
}

// components/DdexGenerator.jsx
import React, { useState } from 'react';
import { useDdexBuilder } from '../hooks/useDdexBuilder';

export function DdexGenerator() {
    const { builder, isLoading, error } = useDdexBuilder();
    const [xml, setXml] = useState('');
    const [releaseData, setReleaseData] = useState({
        release_id: '',
        title: '',
        artist: ''
    });
    
    const handleGenerate = async () => {
        if (!builder) return;
        
        try {
            const release = {
                release_id: releaseData.release_id,
                title: releaseData.title,
                artist: releaseData.artist
            };
            
            const generatedXml = builder.build_release(release);
            setXml(generatedXml);
        } catch (err) {
            console.error('Failed to generate DDEX XML:', err);
        }
    };
    
    if (isLoading) return <div>Loading WASM module...</div>;
    if (error) return <div>Error loading WASM: {error.message}</div>;
    
    return (
        <div>
            <h2>DDEX XML Generator</h2>
            <input
                type="text"
                placeholder="Release ID"
                value={releaseData.release_id}
                onChange={(e) => setReleaseData(prev => ({
                    ...prev,
                    release_id: e.target.value
                }))}
            />
            <input
                type="text"
                placeholder="Title"
                value={releaseData.title}
                onChange={(e) => setReleaseData(prev => ({
                    ...prev,
                    title: e.target.value
                }))}
            />
            <input
                type="text"
                placeholder="Artist"
                value={releaseData.artist}
                onChange={(e) => setReleaseData(prev => ({
                    ...prev,
                    artist: e.target.value
                }))}
            />
            <button onClick={handleGenerate}>Generate DDEX XML</button>
            {xml && (
                <pre style={{ background: '#f5f5f5', padding: '10px' }}>
                    {xml}
                </pre>
            )}
        </div>
    );
}
```

### Vue.js Integration

```vue
<!-- components/DdexBuilder.vue -->
<template>
  <div class="ddex-builder">
    <div v-if="loading">Loading WASM module...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <div v-else>
      <h2>DDEX Builder</h2>
      <form @submit.prevent="generateXml">
        <input
          v-model="release.release_id"
          placeholder="Release ID"
          required
        />
        <input
          v-model="release.title"
          placeholder="Title"
          required
        />
        <input
          v-model="release.artist"
          placeholder="Artist"
          required
        />
        <button type="submit">Generate XML</button>
      </form>
      
      <div v-if="xml" class="xml-output">
        <h3>Generated XML:</h3>
        <pre>{{ xml }}</pre>
      </div>
    </div>
  </div>
</template>

<script>
import init, { DdexBuilder } from 'ddex-builder-wasm';

export default {
  name: 'DdexBuilder',
  data() {
    return {
      builder: null,
      loading: true,
      error: null,
      xml: '',
      release: {
        release_id: '',
        title: '',
        artist: ''
      }
    };
  },
  
  async mounted() {
    try {
      await init();
      this.builder = new DdexBuilder();
      this.loading = false;
    } catch (err) {
      this.error = err.message;
      this.loading = false;
    }
  },
  
  methods: {
    generateXml() {
      if (!this.builder) return;
      
      try {
        this.xml = this.builder.build_release(this.release);
      } catch (err) {
        this.error = err.message;
      }
    }
  }
};
</script>

<style scoped>
.ddex-builder {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

.error {
  color: red;
  padding: 10px;
  background: #ffe6e6;
  border-radius: 4px;
}

input {
  display: block;
  width: 100%;
  margin: 10px 0;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.xml-output {
  margin-top: 20px;
}

pre {
  background: #f5f5f5;
  padding: 15px;
  border-radius: 4px;
  overflow: auto;
  max-height: 400px;
}
</style>
```

### Vanilla JavaScript (ES6 Modules)

```javascript
// ddex-builder.js
class DdexBuilderWrapper {
    constructor() {
        this.builder = null;
        this.isInitialized = false;
        this.initPromise = null;
    }
    
    async init() {
        if (this.initPromise) {
            return this.initPromise;
        }
        
        this.initPromise = this._initWasm();
        return this.initPromise;
    }
    
    async _initWasm() {
        try {
            const wasmModule = await import('./pkg/ddex_builder_wasm.js');
            await wasmModule.default(); // Initialize WASM
            this.builder = new wasmModule.DdexBuilder();
            this.isInitialized = true;
            console.log('DDEX Builder WASM initialized successfully');
        } catch (error) {
            console.error('Failed to initialize DDEX Builder WASM:', error);
            throw error;
        }
    }
    
    async buildRelease(releaseData) {
        if (!this.isInitialized) {
            await this.init();
        }
        
        if (!this.builder) {
            throw new Error('DDEX Builder not initialized');
        }
        
        return this.builder.build_release(releaseData);
    }
    
    async validateXml(xmlString) {
        if (!this.isInitialized) {
            await this.init();
        }
        
        return this.builder.validate_xml(xmlString);
    }
}

// Usage
const ddexBuilder = new DdexBuilderWrapper();

document.addEventListener('DOMContentLoaded', async () => {
    try {
        await ddexBuilder.init();
        
        document.getElementById('generate-btn').addEventListener('click', async () => {
            const releaseData = {
                release_id: document.getElementById('release-id').value,
                title: document.getElementById('title').value,
                artist: document.getElementById('artist').value
            };
            
            try {
                const xml = await ddexBuilder.buildRelease(releaseData);
                document.getElementById('output').textContent = xml;
            } catch (error) {
                console.error('Generation failed:', error);
                alert('Failed to generate XML: ' + error.message);
            }
        });
    } catch (error) {
        console.error('Initialization failed:', error);
        alert('Failed to load DDEX Builder: ' + error.message);
    }
});
```

## Webpack Configuration

```javascript
// webpack.config.js
const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

module.exports = {
    entry: './index.js',
    mode: 'development',
    devtool: 'inline-source-map',
    
    module: {
        rules: [
            {
                test: /\.wasm$/,
                type: 'webassembly/experimental'
            }
        ]
    },
    
    resolve: {
        extensions: ['.js', '.wasm']
    },
    
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'bundle.js',
    },
    
    plugins: [
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, './ddex-builder-wasm'),
            outDir: path.resolve(__dirname, './pkg'),
            extraArgs: '--target web'
        })
    ],
    
    experiments: {
        asyncWebAssembly: true
    }
};
```

## Performance Considerations

### Bundle Size Optimization

The WASM bundle is optimized for size:

- **Current size**: ~114KB compressed
- **Target**: <500KB (achieved 77% under target)
- **Optimization flags**: `opt-level = "s"`, `lto = true`, `codegen-units = 1`

### Memory Management

```javascript
// Good: Reuse builder instances
const builder = new DdexBuilder();
for (const release of releases) {
    const xml = builder.build_release(release);
    // Process xml...
}

// Avoid: Creating new builders for each operation
releases.forEach(release => {
    const builder = new DdexBuilder(); // Memory inefficient
    const xml = builder.build_release(release);
});
```

### Streaming for Large Datasets

```javascript
// For large datasets, use streaming approach
async function processLargeDataset(releases) {
    const builder = new DdexBuilder();
    const results = [];
    
    // Process in chunks to avoid memory issues
    const chunkSize = 100;
    for (let i = 0; i < releases.length; i += chunkSize) {
        const chunk = releases.slice(i, i + chunkSize);
        
        const xmls = chunk.map(release => {
            try {
                return builder.build_release(release);
            } catch (error) {
                console.error(`Failed to build release ${release.release_id}:`, error);
                return null;
            }
        });
        
        results.push(...xmls.filter(Boolean));
        
        // Allow other tasks to run
        await new Promise(resolve => setTimeout(resolve, 0));
    }
    
    return results;
}
```

### Performance Benchmarks

| Operation | Size | Time | Memory |
|-----------|------|------|--------|
| Single release | 10KB | <1ms | <1MB |
| Album (10 tracks) | 100KB | <5ms | <2MB |
| Large catalog (1000 releases) | 10MB | <100ms | <10MB |
| Batch processing | 100MB | <1s | <50MB |

## Browser Compatibility

### Modern Browsers (Full Support)

| Browser | Version | WASM Support | ES6 Modules | Performance |
|---------|---------|--------------|-------------|-------------|
| Chrome | 61+ | ✅ | ✅ | Excellent |
| Firefox | 60+ | ✅ | ✅ | Excellent |
| Safari | 11+ | ✅ | ✅ | Good |
| Edge | 79+ | ✅ | ✅ | Excellent |

### Legacy Browser Support

For older browsers, use the UMD build:

```html
<!-- Polyfills for older browsers -->
<script src="https://unpkg.com/core-js-bundle@3/minified.js"></script>
<script src="https://unpkg.com/regenerator-runtime/runtime.js"></script>

<!-- UMD build -->
<script src="./pkg-umd/ddex_builder_wasm.js"></script>
<script>
    // Access via global variable
    wasm.default().then(() => {
        const builder = new wasm.DdexBuilder();
        // Use builder...
    });
</script>
```

### Feature Detection

```javascript
function checkWasmSupport() {
    if (typeof WebAssembly === 'object') {
        if (typeof WebAssembly.instantiate === 'function') {
            return 'full';
        }
        return 'partial';
    }
    return 'none';
}

const wasmSupport = checkWasmSupport();
if (wasmSupport === 'none') {
    console.error('WebAssembly not supported');
    // Fallback to pure JS implementation
} else if (wasmSupport === 'partial') {
    console.warn('Limited WebAssembly support');
    // Use polyfills
}
```

## Limitations

### Browser Environment

1. **File System Access**: Limited to browser sandbox - no direct file I/O
2. **Memory Limits**: Subject to browser memory constraints (usually 2-4GB)
3. **Threading**: Single-threaded execution (Web Workers can be used for parallelism)
4. **Network**: CORS restrictions apply for remote XML fetching

### Performance Constraints

1. **Initialization Time**: ~10-50ms WASM module loading time
2. **Bundle Size**: 114KB initial download (cached after first load)
3. **Memory Usage**: ~1MB base + ~10KB per release object
4. **DOM Manipulation**: Not included - use browser APIs for DOM updates

### DDEX Limitations

1. **Validation**: Client-side validation only - server validation recommended
2. **Schema Versions**: ERN 3.8.2, 4.2, 4.3 supported
3. **Extensions**: Limited to standard DDEX elements
4. **Binary Resources**: Referenced by URL only (no embedding)

## Error Handling

```javascript
import init, { DdexBuilder } from 'ddex-builder-wasm';

async function safeInitialization() {
    try {
        await init();
        return new DdexBuilder();
    } catch (error) {
        if (error.message.includes('WebAssembly')) {
            throw new Error('WebAssembly not supported in this browser');
        } else if (error.message.includes('network')) {
            throw new Error('Failed to download WASM module');
        } else {
            throw new Error(`Initialization failed: ${error.message}`);
        }
    }
}

function handleBuildError(error) {
    if (error.message.includes('validation')) {
        return 'Invalid input data provided';
    } else if (error.message.includes('memory')) {
        return 'Insufficient memory for operation';
    } else if (error.message.includes('timeout')) {
        return 'Operation timed out';
    }
    return `Build failed: ${error.message}`;
}

// Usage with error handling
async function buildWithErrorHandling(releaseData) {
    try {
        const builder = await safeInitialization();
        return builder.build_release(releaseData);
    } catch (error) {
        const friendlyMessage = handleBuildError(error);
        console.error(friendlyMessage, error);
        throw new Error(friendlyMessage);
    }
}
```

## Security Model

WASM provides a secure sandboxed environment:

1. **Memory Isolation**: WASM has its own linear memory space
2. **No Direct DOM Access**: Must use JavaScript bindings
3. **No Network Access**: Cannot make HTTP requests directly
4. **No File System**: Cannot access local files
5. **CORS Compliance**: Subject to same-origin policy

See [Security Documentation](#security) for detailed security considerations.

## Debugging

### Browser DevTools

```javascript
// Enable debug logging
import init, { DdexBuilder, set_panic_hook } from 'ddex-builder-wasm';

async function initWithDebug() {
    await init();
    set_panic_hook(); // Better error messages
    
    const builder = new DdexBuilder();
    builder.set_debug(true); // Enable debug output
    return builder;
}
```

### Console Logging

```javascript
// The WASM module includes console logging
// Check browser console for debug information
console.log('WASM module loaded');
```

### Performance Profiling

```javascript
// Measure performance
console.time('XML Generation');
const xml = builder.build_release(release);
console.timeEnd('XML Generation');

// Memory usage
console.log('Memory usage:', performance.memory);
```

## API Reference

### DdexBuilder Class

```typescript
class DdexBuilder {
    constructor();
    
    // Core building methods
    build_release(release: Release): string;
    build_batch(releases: Release[]): string[];
    
    // Validation methods
    validate_release(release: Release): ValidationResult;
    validate_xml(xml: string): ValidationResult;
    
    // Configuration
    set_version(version: string): void;
    set_options(options: BuildOptions): void;
    
    // Utilities
    get_supported_versions(): string[];
    get_stats(): BuildStats;
}
```

### Release Interface

```typescript
interface Release {
    release_id: string;
    title: string;
    artist?: string;
    label?: string;
    release_date?: string;
    genre?: string;
    resources?: Resource[];
    metadata?: Record<string, any>;
}
```

For complete API documentation, see the TypeScript definitions in `pkg/ddex_builder_wasm.d.ts`.

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/new-feature`
3. Make changes and test: `wasm-pack test --headless --firefox`
4. Build for all targets: `npm run build:all`
5. Submit a pull request

## License

MIT License. See [LICENSE](../../../../LICENSE) for details.

## Support

- 📖 [Documentation](https://ddex-suite.web.app)
- 🐛 [Issues](https://github.com/daddykev/ddex-suite/issues)
- 💬 [Discussions](https://github.com/daddykev/ddex-suite/discussions)
- 📧 [Email Support](mailto:support@ddex-suite.dev)