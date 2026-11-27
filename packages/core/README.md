# ddex-core

Shared DDEX data models for [`ddex-parser`](https://crates.io/crates/ddex-parser) and
[`ddex-builder`](https://crates.io/crates/ddex-builder), part of
[DDEX Suite](https://github.com/daddykev/ddex-suite).

> **0.5.0 has breaking changes.** The flat and graph models gained fields (dates, P/C
> lines, genres, parental warnings, labels, track groups), so struct literals need the new
> fields. See the
> [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md).

You normally use this crate through `ddex-parser`, whose `DDEXParser::parse` returns a
`ddex_core::models::flat::ParsedERNMessage`:

- `models::flat`: the developer-friendly view (`ParsedRelease`, `ParsedTrack`,
  `ParsedResource`, `ParsedDeal`, ...). Absent values are `None` or empty; nothing is
  invented.
- `models::graph`: the structure of the ERN message as written.
- `ERNVersion`: ERN 3.8.1, 3.8.2, 4.2 and 4.3.

Parsed model structs may gain fields in minor releases.

## Installation

```toml
[dependencies]
ddex-core = "0.5.1"
```

## Example

```rust
use ddex_core::models::flat::ParsedRelease;
use ddex_core::ERNVersion;

/// The track titles of a release parsed by ddex-parser.
fn track_titles(release: &ParsedRelease) -> Vec<&str> {
    release.tracks.iter().filter_map(|track| track.title.as_deref()).collect()
}

assert_eq!(ERNVersion::V4_3.as_str(), "4.3");
assert_eq!(ERNVersion::V4_3.namespace(), "http://ddex.net/xml/ern/43");
```

## Features

- `typescript`: derive TypeScript definitions with `ts-rs`.

## Links

- Documentation: [docs.rs/ddex-core](https://docs.rs/ddex-core) and [ddex-suite.org](https://ddex-suite.org)
- [Changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md)
- [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md)
- [Known issues](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md)
- [Report an issue](https://github.com/daddykev/ddex-suite/issues)

## License

MIT, see [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE).
