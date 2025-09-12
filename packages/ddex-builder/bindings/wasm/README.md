# @ddex/builder-wasm

High-performance DDEX builder for browsers (420KB).

[![npm version](https://img.shields.io/npm/v/@ddex/builder-wasm)](https://www.npmjs.com/package/@ddex/builder-wasm)
[![Security Status](https://img.shields.io/badge/security-no%20vulnerabilities-green)](https://github.com/ddex/ddex-suite)

## Installation
```bash
npm install @ddex/builder-wasm@0.3.5
```

## Features
- Complete XML generation capabilities
- Deterministic output
- TypeScript definitions included
- Works in Web Workers
- WASM: 420KB (gzipped: ~140KB)

## Usage
```javascript
import init, { DdexBuilder, Release } from '@ddex/builder-wasm';

await init();
const builder = new DdexBuilder();

const release = new Release('REL001', 'Album', 'My Album', 'Artist Name');
release.label = 'My Label';
release.upc = '123456789012';
builder.addRelease(release);

const xml = await builder.build();
```

## Browser Support
- Chrome 57+, Firefox 52+, Safari 11+, Edge 16+
- Full support in modern browsers
- Web Worker compatible

## License
Apache-2.0