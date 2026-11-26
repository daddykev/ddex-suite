# ddex-builder (Node.js)

DDEX ERN XML builder for Node.js, a native addon over the Rust
[`ddex-builder`](https://crates.io/crates/ddex-builder) crate. Part of
[DDEX Suite](https://github.com/daddykev/ddex-suite). It writes ERN 4.3, 4.2, 3.8.2 and
3.8.1 from a typed build request, canonicalized with DB-C14N/1.0 (fixed namespace
prefixes, sorted attributes, consistent formatting, elements in XSD order). TypeScript
types are included (`index.d.ts`, `build-request.d.ts`).

Byte-for-byte determinism is not complete yet: a message ID, release and resource
references and `message_created_date_time` that the request leaves out are generated
(random UUIDs, the current time), so two builds of the same request can differ. Full
determinism is planned for 0.6.0.

> **0.5.0 has breaking changes.** `build(request)` takes the core build request (`header`
> and `version` are required, nothing is filled in), ERN 3.8.2 output is schema-valid,
> and `batchBuild` was removed. See the
> [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md).

## Installation

```bash
npm install ddex-builder
```

Prebuilt binaries are installed for macOS (arm64, x64), Linux x64 (glibc) and Windows
x64, through the platform packages `ddex-builder-<platform>`. Node.js 18 or newer.

## Example

```js
const { DdexBuilder } = require('ddex-builder');

const party = (name, id) => ({ party_name: [{ text: name }], party_id: id });

const request = {
  header: {
    message_id: 'MSG-001',
    message_sender: party('Example Label', 'PADPIDA0000000001'),
    message_recipient: party('Example DSP', 'PADPIDA0000000002'),
    message_created_date_time: '2024-01-01T00:00:00Z',
  },
  version: '4.3',
  avs_version_id: '12',
  language_and_script_code: 'en',
  releases: [{
    release_id: 'A1-ABCDE-0000000001-M',
    title: [{ text: 'Example Single' }],
    artist: 'Example Artist',
    label: 'Example Label',
    release_type: 'Single',
    genre: 'Pop',
    parental_warning_type: 'NotExplicit',
    tracks: [{
      track_id: 'T1',
      isrc: 'ZZ0000000001',
      title: 'Example Track',
      duration: 'PT3M42S',
      artist: 'Example Artist',
      resource_type: 'MusicalWorkSoundRecording',
      parental_warning_type: 'NotExplicit',
    }],
  }],
  deals: [],
};

new DdexBuilder().build(request).then((xml) => {
  console.log(xml);
});
```

The request mirrors the Rust `BuildRequest` (snake_case field names); see
`build-request.d.ts`. When the target version's schema requires a value the request does
not carry, `build` rejects with an error naming the field, for example
`ERN 4.3 requires Release/ReleaseType (releases[0].release_type)`.

The output reads back with [`ddex-parser`](https://www.npmjs.com/package/ddex-parser):

```js
const { DdexBuilder } = require('ddex-builder');
const { DdexParser } = require('ddex-parser');

const party = (name, id) => ({ party_name: [{ text: name }], party_id: id });
const track = (n) => ({
  track_id: `T${n}`,
  isrc: `ZZ000000000${n}`,
  title: `Track ${n}`,
  duration: 'PT3M0S',
  artist: 'Example Artist',
  resource_type: 'MusicalWorkSoundRecording',
  parental_warning_type: 'NotExplicit',
});

new DdexBuilder()
  .build({
    header: {
      message_sender: party('Example Label', 'PADPIDA0000000001'),
      message_recipient: party('Example DSP', 'PADPIDA0000000002'),
    },
    version: '4.3',
    avs_version_id: '12',
    language_and_script_code: 'en',
    releases: [{
      release_id: 'A1-ABCDE-0000000002-M',
      title: [{ text: 'Example Album' }],
      artist: 'Example Artist',
      label: 'Example Label',
      release_type: 'Album',
      genre: 'Pop',
      parental_warning_type: 'NotExplicit',
      tracks: [track(1), track(2)],
    }],
    deals: [],
  })
  .then((xml) => {
    const message = new DdexParser().parseSync(xml);
    console.log(message.releases[0].tracks.map((t) => t.title)); // [ 'Track 1', 'Track 2' ]
  });
```

## Supported versions

| `version` | Output |
|---|---|
| `"4.3"`, `"4.2"` | Validated against the official XSDs in CI |
| `"3.8.2"` | Validated against the official XSD in CI; video resources are not supported yet |
| `"3.8.1"` | The 3.8.2 document in the 3.8.1 namespace; not checked against an official XSD (none is available) |

## Scope

- Use `build(request)`. The preset methods (`applyPreset`, `getAvailablePresets`, ...)
  list the builder's presets, which do not change its output in 0.5.x, and
  `StreamingDdexBuilder` only writes the old 3.8-style tree; see the known issues.
- `testRoundTripFidelity(xml)` parses, builds and parses again, and lists in `issues` the
  differences on the fields a build request carries (round trips are not lossless:
  contributors, P/C lines, images and other data are not rebuilt). `fidelityScore` is
  always null: no score is computed.
- Text is XML-escaped on output. `build` checks the fields the target version requires;
  it does not validate the output against the XSD or the AVS business rules at runtime.

## Links

- Documentation: [ddex-suite.org](https://ddex-suite.org/docs/builder)
- [Changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md)
- [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md)
- [Known issues](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md)
- [Report an issue](https://github.com/daddykev/ddex-suite/issues)

## License

MIT, see [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE).
