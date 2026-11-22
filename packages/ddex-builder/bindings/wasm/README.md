# ddex-builder-wasm

The DDEX Suite builder for browsers and other WebAssembly hosts: the core Rust
`ddex-builder` compiled to WebAssembly. It takes the same build request as the Rust core
and the Node binding, applies the same validation, and writes the same deterministic,
schema-valid XML (ERN 4.3, 4.2 and 3.8.2).

## Build

```bash
npm run build        # wasm-pack build --target web    → pkg/
npm run build:node   # wasm-pack build --target nodejs → pkg-node/
npm test             # builds pkg-node and runs test-node.cjs (XSD-checks the output)
```

## Usage

```javascript
import init, { WasmDdexBuilder } from './pkg/ddex_builder_wasm.js';

await init();
const builder = new WasmDdexBuilder();

const xml = builder.build({
  header: {
    message_sender: { party_name: [{ text: 'Example Label' }], party_id: 'PADPIDA0000000001' },
    message_recipient: { party_name: [{ text: 'Example DSP' }], party_id: 'PADPIDA0000000002' },
  },
  version: '4.3',
  avs_version_id: '12',
  language_and_script_code: 'en',
  releases: [{
    release_id: 'A1-ABCDE-0000000001-M',
    title: [{ text: 'My Single' }],
    artist: 'Example Artist',
    label: 'Example Label',
    release_type: 'Single',
    genre: 'Pop',
    parental_warning_type: 'NotExplicit',
    tracks: [{
      track_id: 'T1',
      isrc: 'USRT12400001',
      title: 'My Track',
      duration: 'PT3M42S',
      artist: 'Example Artist',
      resource_type: 'MusicalWorkSoundRecording',
      parental_warning_type: 'NotExplicit',
    }],
  }],
  deals: [],
});
```

The request type (`BuildRequest`, snake_case field names) is in the generated
`ddex_builder_wasm.d.ts`; it is the same file as the Node binding's `build-request.d.ts`.
`header` and `version` are required. Nothing is filled in: when the target ERN version
requires a value that the request does not carry, `build` throws an error naming the
field, e.g. `ERN 4.3 requires Release/Genre/GenreText (releases[0].genre)`.

## Changes in 0.5.0

Before 0.5.0 this package had its own string-concatenated generator whose "ERN 4.3"
output was not schema-valid, and `batchBuild` returned placeholder XML. In 0.5.0:

- `build(request)` calls the core builder (synchronous; it no longer returns a Promise).
- Removed: `batchBuild` (call `build` once per request), the `Release` / `Resource`
  classes with `addRelease` / `addResource`, `buildWithFidelity`,
  `testRoundTripFidelity`, `canonicalizeXml`, `validate`, `getStats`, `reset`, the preset
  methods and `validateStructure`. They returned fixed or invented values.
- `DdexDiffViewer` is unchanged. It does not parse XML yet: it compares the two
  documents as single text nodes (see KNOWN_ISSUES.md).

## Size

The module holds the whole core builder: about 2.4 MB (about 800 KB gzipped), built
with `wasm-pack --release` and no `wasm-opt` pass.

## License

MIT
