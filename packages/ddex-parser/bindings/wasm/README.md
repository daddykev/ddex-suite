# @ddex/parser-wasm

Ultra-lightweight DDEX parser for browsers (37KB).

## Installation
```bash
npm install @ddex/parser-wasm@0.3.5
```

## Usage
```javascript
import init, { DDEXParser } from '@ddex/parser-wasm';

await init();
const parser = new DDEXParser();
const result = parser.parse(xmlString);
```

## Bundle Size
- WASM: 37KB (gzipped: ~12KB)
- Works in all modern browsers
- TypeScript definitions included

## License
Apache-2.0