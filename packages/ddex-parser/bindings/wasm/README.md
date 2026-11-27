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
const message = parser.parse(xmlString); // the parsed message (serde form of ParsedERNMessage)
```

`parse` takes only the XML (its ignored `options` argument and the unimplemented
`parse_stream` were removed in 0.6.0). For Node.js use the native
[`ddex-parser`](https://www.npmjs.com/package/ddex-parser) package.

[Changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md),
[known issues](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md).

MIT license.
