# ddex-parser-wasm

The DDEX Suite parser compiled to WebAssembly for browsers. It is **not published to
npm**; build it from this directory with [wasm-pack](https://rustwasm.github.io/wasm-pack/):

```bash
wasm-pack build --target web --release   # → pkg/
```

```text
import init, { DDEXParser } from './pkg/ddex_parser_wasm.js';

await init();
const parser = new DDEXParser();
const message = parser.parse(xmlString, undefined); // the parsed message (serde form of ParsedERNMessage)
```

The `options` argument of `parse` is ignored. For Node.js use the native
[`ddex-parser`](https://www.npmjs.com/package/ddex-parser) package.

[Changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md),
[known issues](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md).

MIT license.
