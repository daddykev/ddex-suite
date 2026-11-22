# DDEX Suite

[![crates.io ddex-parser](https://img.shields.io/crates/v/ddex-parser?label=crates.io%20ddex-parser)](https://crates.io/crates/ddex-parser)
[![crates.io ddex-builder](https://img.shields.io/crates/v/ddex-builder?label=crates.io%20ddex-builder)](https://crates.io/crates/ddex-builder)
[![npm ddex-parser](https://img.shields.io/npm/v/ddex-parser?label=npm%20ddex-parser)](https://www.npmjs.com/package/ddex-parser)
[![npm ddex-builder](https://img.shields.io/npm/v/ddex-builder?label=npm%20ddex-builder)](https://www.npmjs.com/package/ddex-builder)
[![PyPI ddex-parser](https://img.shields.io/pypi/v/ddex-parser?label=PyPI%20ddex-parser)](https://pypi.org/project/ddex-parser/)
[![PyPI ddex-builder](https://img.shields.io/pypi/v/ddex-builder?label=PyPI%20ddex-builder)](https://pypi.org/project/ddex-builder/)

DDEX ERN XML parsing and deterministic building from one Rust core, with native bindings
for Node.js and Python.

- **ddex-parser** reads ERN 3.8.1, 3.8.2, 4.2 and 4.3 into a developer-friendly flat model
  and a graph that follows the message structure.
- **ddex-builder** writes ERN 4.3, 4.2, 3.8.2 and 3.8.1 from a typed build request,
  canonicalized with DB-C14N/1.0 so the same request always produces the same bytes.
- **ddex-core** holds the shared data models.

> **0.5.0 has breaking changes**: text is kept exactly as written, `parse` no longer
> switches to streaming above 10 MB, the Node parse options throw, the Python packages
> need their native extension, and the build request is the core's. Read the
> [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md)
> before upgrading.

## Packages

| Package | Rust | Node.js | Python |
|---|---|---|---|
| Parser | [`ddex-parser`](https://crates.io/crates/ddex-parser) | [`ddex-parser`](https://www.npmjs.com/package/ddex-parser) | [`ddex-parser`](https://pypi.org/project/ddex-parser/) |
| Builder | [`ddex-builder`](https://crates.io/crates/ddex-builder) | [`ddex-builder`](https://www.npmjs.com/package/ddex-builder) | [`ddex-builder`](https://pypi.org/project/ddex-builder/) |
| Models | [`ddex-core`](https://crates.io/crates/ddex-core) | | |

All packages are released together at the same version (0.5.0).

```bash
cargo add ddex-parser ddex-builder
npm install ddex-parser ddex-builder
pip install ddex-parser ddex-builder
```

Prebuilt binaries:

- **npm**: macOS arm64 and x64, Linux x64 (glibc), Windows x64. Node.js 18 or newer.
- **PyPI**: abi3 wheels for Python 3.9 or newer on Linux x86_64 and aarch64 (glibc
  2.28+), Linux x86_64 (musl), macOS x86_64 and arm64, and Windows x64. Other platforms
  build from the source distribution, which needs a Rust toolchain.
- **Rust**: 1.82 or newer.

The WASM builds (`packages/*/bindings/wasm`) are not published; build them with
`wasm-pack`.

## Examples

Parse with Node.js (`release.xml` is any ERN message):

```js
const fs = require('node:fs');
const { DdexParser } = require('ddex-parser');

const message = new DdexParser().parseSync(fs.readFileSync('release.xml', 'utf8'));
for (const release of message.releases) {
  console.log(release.releaseId, release.title, release.releaseDates.map((d) => d.date));
  for (const track of release.tracks) {
    console.log(`  ${track.position}. ${track.title} [${track.isrc}]`);
  }
}
```

Build with Python:

```python
from ddex_builder import DdexBuilder


def party(name, party_id):
    return {"party_name": [{"text": name}], "party_id": party_id}


xml = DdexBuilder().build({
    "header": {
        "message_sender": party("Example Label", "PADPIDA0000000001"),
        "message_recipient": party("Example DSP", "PADPIDA0000000002"),
    },
    "version": "4.3",
    "avs_version_id": "12",
    "language_and_script_code": "en",
    "releases": [{
        "release_id": "A1-ABCDE-0000000001-M",
        "title": [{"text": "Example Single"}],
        "artist": "Example Artist",
        "label": "Example Label",
        "release_type": "Single",
        "genre": "Pop",
        "parental_warning_type": "NotExplicit",
        "tracks": [{
            "track_id": "T1",
            "isrc": "ZZ0000000001",
            "title": "Example Track",
            "duration": "PT3M42S",
            "artist": "Example Artist",
            "resource_type": "MusicalWorkSoundRecording",
            "parental_warning_type": "NotExplicit",
        }],
    }],
    "deals": [],
})
print(xml)
```

Each package README has the full example for its language:
[Rust parser](https://github.com/daddykev/ddex-suite/blob/main/packages/ddex-parser/README.md),
[Rust builder](https://github.com/daddykev/ddex-suite/blob/main/packages/ddex-builder/README.md),
[Node parser](https://github.com/daddykev/ddex-suite/blob/main/packages/ddex-parser/bindings/node/README.md),
[Node builder](https://github.com/daddykev/ddex-suite/blob/main/packages/ddex-builder/bindings/node/README.md),
[Python parser](https://github.com/daddykev/ddex-suite/blob/main/packages/ddex-parser/bindings/python/README.md),
[Python builder](https://github.com/daddykev/ddex-suite/blob/main/packages/ddex-builder/bindings/python/README.md).

## What to expect

- **Parsing**: absent values stay empty, nothing is invented, and text keeps its
  whitespace following the XSD. Release dates, P and C lines, genres, parental warnings,
  labels, track groups and deal dates are read in every ERN version that has them.
- **Building**: nothing is filled in. A value the target version requires but the request
  lacks fails the build with an error naming the field. ERN 4.3, 4.2 and 3.8.2 output is
  validated against the official XSDs in CI. ERN 3.8.1 output is the 3.8.2 document in
  the 3.8.1 namespace and is not checked against an official XSD (none is available).
  There is no built-in conversion from a parsed message to a build request yet.
- **Security**: the parser rejects DTDs and external entities (XXE), limits entity
  expansion, depth and size, and never fetches from the network. Neither package
  validates against the XSD or the AVS business rules at runtime.
- **Streaming**: `parse` reads the whole document. The Rust streaming APIs are
  experimental and opt-in, and read a subset of the fields. The parallel streaming
  iterator does not currently scale with the thread count (planned for 0.6.0).
- **Presets**: the builder's preset APIs do not change the output in 0.5.0.

Open bugs are listed in
[KNOWN_ISSUES.md](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md).

## Performance

Criterion benches on an Apple M1 Pro, release profile
([details and commands](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/PERFORMANCE.md)):

| Operation | Measured |
|---|---:|
| Parse an official ERN 4.3 sample (10–330 KB) | 0.5–14 ms (about 20–24 MiB/s) |
| Parse `1 Audio.xml` (75 KB, a typical album) | 3.4 ms |
| Parse a ~10 MB ERN 4.3 catalog | 441 ms (22.7 MiB/s) |
| Build ERN 4.3, 1 / 12 / 100 tracks | 2.7 / 11.4 / 80 ms |

## Repository layout

- `packages/core`: `ddex-core`
- `packages/ddex-parser`, `packages/ddex-builder`: the crates, with their bindings in
  `bindings/node`, `bindings/python` and `bindings/wasm`
- `website`: the documentation site, [ddex-suite.org](https://ddex-suite.org)
- `examples`: the official DDEX ERN 4.2 and 4.3 sample messages

## Links

- Documentation: [ddex-suite.org](https://ddex-suite.org), [docs.rs/ddex-parser](https://docs.rs/ddex-parser), [docs.rs/ddex-builder](https://docs.rs/ddex-builder)
- [Changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md)
- [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md)
- [Known issues](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md)
- [Report an issue](https://github.com/daddykev/ddex-suite/issues)

DDEX Suite complements [DDEX Workbench](https://ddex-workbench.org), which handles XSD
validation and business rules. External contributions are not accepted yet; issue
reports and feedback are welcome.

## License

MIT, see [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE).
