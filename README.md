# DDEX Suite

![Rust](https://img.shields.io/badge/Rust-1.82%2B-orange?logo=rust)
![Node.js](https://img.shields.io/badge/Node.js-18%2B-green?logo=node.js)
![Python](https://img.shields.io/badge/Python-3.10%20%E2%80%93%203.14-blue?logo=python)
![TypeScript](https://img.shields.io/badge/TypeScript-types%20included-blue?logo=typescript)
![Platform](https://img.shields.io/badge/Platform-Linux%20x64%20%7C%20arm64%20%7C%20musl%20x64%20%C2%B7%20macOS%20arm64%20%7C%20x64%20%C2%B7%20Windows%20x64-lightgrey)
[![npm ddex-builder](https://img.shields.io/npm/v/ddex-builder?label=npm%20ddex-builder)](https://www.npmjs.com/package/ddex-builder)
[![npm ddex-parser](https://img.shields.io/npm/v/ddex-parser?label=npm%20ddex-parser)](https://www.npmjs.com/package/ddex-parser)
[![PyPI ddex-builder](https://img.shields.io/pypi/v/ddex-builder?label=PyPI%20ddex-builder)](https://pypi.org/project/ddex-builder/)
[![PyPI ddex-parser](https://img.shields.io/pypi/v/ddex-parser?label=PyPI%20ddex-parser)](https://pypi.org/project/ddex-parser/)
[![crates.io ddex-core](https://img.shields.io/crates/v/ddex-core?label=crates.io%20ddex-core)](https://crates.io/crates/ddex-core)
[![crates.io ddex-parser](https://img.shields.io/crates/v/ddex-parser?label=crates.io%20ddex-parser)](https://crates.io/crates/ddex-parser)
[![crates.io ddex-builder](https://img.shields.io/crates/v/ddex-builder?label=crates.io%20ddex-builder)](https://crates.io/crates/ddex-builder)

> DDEX ERN XML parser and builder with native bindings for TypeScript/JavaScript and Python. Built on a single Rust core for consistent behavior across all three languages.

DDEX Suite brings together tools for music industry data exchange, combining the `ddex-parser` library for reading DDEX messages with the `ddex-builder` library for canonical XML generation, both on one shared Rust core:

- **ddex-parser** reads ERN 3.8.1, 3.8.2, 4.2 and 4.3 into a developer-friendly flat model
  and a graph that follows the message structure.
- **ddex-builder** writes ERN 4.3, 4.2, 3.8.2 and 3.8.1 from a typed build request. Output
  is canonicalized with DB-C14N/1.0 (fixed namespace prefixes, sorted attributes,
  consistent formatting, elements in XSD order). Byte-for-byte determinism is not complete
  yet: values the request leaves out are generated (see
  [Canonical output and round trips](#-canonical-output-and-round-trips)). Full
  determinism is planned for 0.6.0.
- **ddex-core** holds the shared data models.

> **0.5.0 has breaking changes**: text is kept exactly as written, `parse` no longer
> switches to streaming above 10 MB, the Node parse options throw, the Python packages
> need their native extension, and the build request is the core's. Read the
> [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md)
> before upgrading.

## 🎯 Why DDEX Suite?

Working with DDEX XML shouldn't feel like archaeology. The suite transforms complex DDEX messages into clean, strongly-typed data structures that are as easy to work with as JSON.

### Core Value Proposition
- **Single Rust Core**: One implementation to rule them all - consistent behavior across JavaScript, Python, and Rust
- **Dual Model Architecture**: Choose between faithful graph representation (Rust) or developer-friendly flattened view (Rust, Node.js, Python)
- **Hardened Parsing**: DTD and external-entity (XXE) rejection, entity expansion, depth and size limits, no network fetches. Pre-1.0: open bugs are listed in [KNOWN_ISSUES.md](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md)
- **Canonical Output**: Schema-valid ERN 4.3, 4.2 and 3.8.2, canonicalized with DB-C14N/1.0; fully deterministic output (no generated message IDs, references or timestamps) is planned for 0.6.0

## 👨🏻‍💻 Developer Statement

I'm building **DDEX Suite** as a rigorous, end-to-end learning project to deepen my Rust skills while unifying my JavaScript and Python experience into a production-grade toolkit for music metadata. It's built on a **single Rust core** that powers both a security-hardened DDEX XML parser library (`ddex-parser`) and a schema-driven builder library (`ddex-builder`). That core is exposed through **napi-rs** for Node/TypeScript, **PyO3** for Python and **wasm-bindgen** for WebAssembly, with a consistent API across languages and **pandas DataFrame** import and export for Python users. The project is deliberately "industry-shaped": it reads ERN 3.8.1, 3.8.2, 4.2 and 4.3 into a faithful **graph** model (in Rust) and a developer-friendly **flattened** view (in every language), and it writes ERN 3.8.2, 4.2 and 4.3 that validates against the official DDEX XSDs. Its guiding rule is to **never invent data**: absent values stay empty, and a build that lacks a value the standard requires fails with an error naming the field. Output is canonicalized by a custom specification, **DB-C14N/1.0**; making it fully byte-for-byte reproducible, with **stable, content-addressable IDs** as the default, is the focus of 0.6.0.

Beyond the core implementation, this is a showcase of **software craftsmanship and platform thinking**. Correctness is checked rather than assumed. CI covers Rust, Node.js and Python 3.10–3.14, and every prebuilt binary for Linux, macOS and Windows is install-tested before release. The builder runs **preflight validation** on every build, and the parser ships a CLI for working with messages from the command line. Paired with my validator work (**DDEX Workbench**, which handles XSD validation and business rules), DDEX Suite covers both ends of a **Parse → Build** pipeline. Next come a built-in bridge from parsed messages to build requests for true **Parse → Modify → Build**, a semantic diff, partner presets that shape the output, and further supply-chain hardening (SBOMs and Sigstore-signed artifacts; `cargo-deny` already runs in CI). The goal is to show how interoperable components can be designed to be correct, safe and easy to adopt in real-world systems.

## 🚧 Development Status

**Latest Release**: Suite v0.5.0 🎉  
**Next Release**: Suite v0.6.0  
**Goal**: Suite v1.0.0

### 📦 Available Packages

All packages are released together at the same version.

| Package | npm | PyPI | crates.io | Version |
|---------|-----|------|-----------|---------|
| **ddex-core** | - | - | ✅ [Published](https://crates.io/crates/ddex-core) | v0.5.0 |
| **ddex-parser** | ✅ [Published](https://www.npmjs.com/package/ddex-parser) | ✅ [Published](https://pypi.org/project/ddex-parser/) | ✅ [Published](https://crates.io/crates/ddex-parser) | v0.5.0 |
| **ddex-builder** | ✅ [Published](https://www.npmjs.com/package/ddex-builder) | ✅ [Published](https://pypi.org/project/ddex-builder/) | ✅ [Published](https://crates.io/crates/ddex-builder) | v0.5.0 |

### Progress Overview

✅ **Parser completeness** - Release dates, P and C lines, genres, parental warnings, labels, `MessageControlType`, deal end dates and the 3.8.2 territory details are read in ERN 3.8.2, 4.2 and 4.3; absent values stay empty  
✅ **Text kept as written** - Values follow their XSD `whiteSpace` facet  
✅ **One parse path** - `parse()` uses the complete parser for every input size; streaming is opt-in and experimental  
✅ **Schema-valid builder output** - ERN 4.3, 4.2 and 3.8.2, validated against the official XSDs in CI; the WASM builder runs the core builder  
✅ **Bindings that fail loudly** - Python raises `ImportError` without its native extension; Node parse options that had no effect throw  
✅ **Packages for every common platform** - abi3 wheels for Python 3.10+ and npm binaries for macOS, Linux and Windows  
✅ **Measured performance** - Criterion bench results in [PERFORMANCE.md](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/PERFORMANCE.md)

🔄 **Next: v0.6.0** - Builder determinism, presets that change the output, a conformance suite  
🎯 **Goal: v1.0.0** - Official stable release

For detailed development progress and technical implementation details, see [blueprint.md](https://github.com/daddykev/ddex-suite/blob/main/blueprint.md).

## 📦 Installation

```bash
cargo add ddex-parser ddex-builder
npm install ddex-parser ddex-builder
pip install ddex-parser ddex-builder
```

Prebuilt binaries:

- **npm**: macOS arm64 and x64, Linux x64 (glibc), Windows x64. Node.js 18 or newer.
- **PyPI**: abi3 wheels for Python 3.10 or newer on Linux x86_64 and aarch64 (glibc
  2.28+), Linux x86_64 (musl), macOS x86_64 and arm64, and Windows x64. Other platforms
  build from the source distribution, which needs a Rust toolchain.
- **Rust**: 1.82 or newer.

The WASM builds (`packages/*/bindings/wasm`) are not published; build them with
`wasm-pack`.

## 💻 Examples

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

## 🎭 Dual Model Architecture

The parser provides two complementary views of the same message. In Rust, `parse` returns
both (`ParsedERNMessage { graph, flat, extensions }`); the Node.js and Python bindings
expose the flattened view.

### Graph Model (Faithful)
Follows the DDEX structure with its references - the basis for compliance work and round trips
(`ddex_core::models::graph::ERNMessage`, abridged):
```rust
pub struct ERNMessage {
    pub message_header: MessageHeader,
    pub parties: Vec<Party>,              // PartyList (4.x)
    pub resources: Vec<Resource>,         // Sound recordings, videos, images
    pub releases: Vec<Release>,           // Releases with their ResourceGroup trees
    pub track_releases: Vec<TrackRelease>, // ERN 4.x TrackReleases
    pub deals: Vec<Deal>,                 // Commercial terms
    pub version: ERNVersion,
    pub attributes: Option<AttributeMap>, // Root attributes (AvsVersionId, ...)
    pub extensions: Option<Extensions>,
    // ...
}
```

### Flattened Model (Developer-Friendly)
Denormalized and resolved for easy consumption - tracks are merged with their resources
and artists, labels and dates are resolved. The Node.js shape (`ParsedMessage.releases`,
from `index.d.ts`, abridged):
```typescript
interface JsRelease {
  releaseId: string
  title?: string
  displayArtist?: string
  releaseType: string
  labelName?: string
  icpn?: string
  genres: JsGenre[]               // Every genre with its territories
  pLine?: JsCopyright
  cLine?: JsCopyright
  releaseDates: JsDateInfo[]      // As written: YYYY, YYYY-MM or YYYY-MM-DD
  tracks: JsTrack[]               // Fully resolved: ISRC, title, duration, artists, disc, position
}
```
The Rust equivalent is `ddex_core::models::flat::ParsedRelease`. There is no built-in
conversion from a parsed message to a build request yet.

## 📐 Canonical output and round trips

- **DB-C14N/1.0** ([specification](https://github.com/daddykev/ddex-suite/blob/main/docs/DB-C14N-SPEC.md)):
  builder output uses fixed namespace prefixes, sorted attributes and consistent
  formatting; the generators write elements in XSD order.
- **Not yet deterministic**: with the default options, a message ID, release and resource
  references and `MessageCreatedDateTime` that the request leaves out are generated
  (random UUIDs, the current time), and the stable-hash ID strategy puts today's date
  into the `MessageId` (the Rust-only sequential strategy uses the current Unix time).
  Update messages always carry the current time, and the streaming writer generates a
  random `MessageId` and the current time when they are missing. Two builds of the same
  request can therefore differ. Full determinism is planned for 0.6.0; the complete list is
  in [KNOWN_ISSUES.md](KNOWN_ISSUES.md).
- **Round trips on the official samples**: each of the 19 official DDEX ERN 4.2 and 4.3
  samples in `examples/` is parsed, rebuilt in its own version, validated against the
  official XSD and re-parsed in the test suite. Release title, ICPN, type and display
  artist, per-track ISRC, title, duration, disc and position, the deal count and the deals'
  release references match the original. Data the build request cannot carry yet (contributors, P and C lines,
  images, ...) is not compared.

## What to expect

- **Parsing**: absent values stay empty, nothing is invented, and text keeps its
  whitespace following the XSD. Release dates, P and C lines, genres, parental warnings,
  labels, track groups and deal dates are read in every ERN version that has them.
- **Building**: nothing is filled in. A value the target version requires but the request
  lacks fails the build with an error naming the field. ERN 4.3, 4.2 and 3.8.2 output is
  validated against the official XSDs in CI. ERN 3.8.1 output is the 3.8.2 document in
  the 3.8.1 namespace and is not checked against an official XSD (none is available).
- **Streaming**: `parse` reads the whole document. The Rust streaming APIs are
  experimental and opt-in, and read a subset of the fields. The parallel streaming
  iterator does not currently scale with the thread count (planned for 0.6.0).
- **Presets**: the builder's preset APIs do not change the output in 0.5.0.

Open bugs are listed in
[KNOWN_ISSUES.md](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md).

## 🏗️ Architecture

Built as a monorepo with shared core components:

```
┌───────────────────────────────────────┐
│            DDEX Suite                 │
├─────────────────┬─────────────────────┤
│   DDEX Parser   │   DDEX Builder      │
│  Read & Parse   │  Generate & Build   │
├─────────────────┴─────────────────────┤
│        Shared Core (ddex-core)        │
│  Models │ Errors │ FFI │ Namespaces   │
├───────────────────────────────────────┤
│         Language Bindings             │
│   napi-rs │ PyO3 │ WASM │ CLI*        │
└───────────────────────────────────────┘
```

\* The parser and builder crates each have a CLI binary. The builder CLI's `build` command is a known stub: it
writes a placeholder document, and its `--validate` never fails
([KNOWN_ISSUES.md](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md)).
Use the library or a binding to build.

## 🔒 Security

The libraries defend against XML-level threats:

- DTDs and external entities (XXE) are rejected
- Entity expansion limits (billion laughs protection)
- Nesting depth and input size limits
- No network fetches
- Metadata text that looks like SQL, HTML or script is accepted, passed through
  unchanged and XML-escaped on output

Neither package validates against the XSD or the AVS business rules at runtime; use
[DDEX Workbench](https://ddex-workbench.org) for that.

Supply chain:

- `Cargo.lock` is committed
- CI runs `cargo-deny` (advisories, banned crates, licenses, sources) with the policy in
  [`deny.toml`](https://github.com/daddykev/ddex-suite/blob/main/deny.toml)
- The release workflow installs every wheel and npm package it builds and smoke-tests it
  with an official sample; publishing is a manual step

Report vulnerabilities as described in
[SECURITY.md](https://github.com/daddykev/ddex-suite/blob/main/SECURITY.md).

## 📊 Performance

Criterion benches on an Apple M1 Pro, release profile
([details and commands](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/PERFORMANCE.md)):

| Operation | Measured |
|---|---:|
| Parse an official ERN 4.3 sample (10–330 KB) | 0.5–14 ms (about 20–24 MiB/s) |
| Parse `1 Audio.xml` (75 KB, a typical album) | 3.4 ms |
| Parse a ~10 MB ERN 4.3 catalog | 441 ms (22.7 MiB/s) |
| Build ERN 4.3, 1 / 12 / 100 tracks | 2.7 / 11.4 / 80 ms |

## 🗂️ Repository layout

- `packages/core`: `ddex-core`
- `packages/ddex-parser`, `packages/ddex-builder`: the crates, with their bindings in
  `bindings/node`, `bindings/python` and `bindings/wasm`
- `website`: the documentation site, [ddex-suite.org](https://ddex-suite.org)
- `examples`: the official DDEX ERN 4.2 and 4.3 sample messages

## 📚 Links

- Documentation: [ddex-suite.org](https://ddex-suite.org), [docs.rs/ddex-core](https://docs.rs/ddex-core), [docs.rs/ddex-parser](https://docs.rs/ddex-parser), [docs.rs/ddex-builder](https://docs.rs/ddex-builder)
- [Blueprint](https://github.com/daddykev/ddex-suite/blob/main/blueprint.md)
- [Changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md)
- [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md)
- [Known issues](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md)
- [Report an issue](https://github.com/daddykev/ddex-suite/issues)

## 🤝 Contributing

External contributions are not accepted yet; issue reports and feedback are welcome.

## 📜 License

MIT, see [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE).

## 🙏 Acknowledgments

DDEX Suite is designed to complement [DDEX Workbench](https://ddex-workbench.org) by providing structural parsing and canonical generation while Workbench handles XSD validation and business rules.

---

**Repository**: https://github.com/daddykev/ddex-suite  
**Status**: v0.5.0 released; v0.6.0 in development  
**Parser**: v0.5.0 on [npm](https://www.npmjs.com/package/ddex-parser), [PyPI](https://pypi.org/project/ddex-parser/) and [crates.io](https://crates.io/crates/ddex-parser)  
**Builder**: v0.5.0 on [npm](https://www.npmjs.com/package/ddex-builder), [PyPI](https://pypi.org/project/ddex-builder/) and [crates.io](https://crates.io/crates/ddex-builder)  
**Core**: v0.5.0 on [crates.io](https://crates.io/crates/ddex-core)  
**Suite Goal**: v1.0.0  
**Last Updated**: November 22, 2026
