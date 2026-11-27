# ddex-parser (Python)

DDEX ERN XML parser for Python, a native extension over the Rust
[`ddex-parser`](https://crates.io/crates/ddex-parser) crate. Part of
[DDEX Suite](https://github.com/daddykev/ddex-suite). It reads ERN 3.8.1, 3.8.2, 4.2 and
4.3 messages and exports them to pandas DataFrames.

> **0.6.0 has breaking changes.** `ParseOptions`, `ParseResult` and the native
> `DDEXParser.from_dataframe` were removed (`parse(xml)` takes no options), `sanity_check`
> has no `warnings`, and Python 3.11 or newer is required. See the [0.6.0 migration
> guide](https://github.com/daddykev/ddex-suite/blob/main/website/docs/migration-0.6.md).

## Installation

```bash
pip install ddex-parser
pip install "ddex-parser[pandas]"   # for to_dataframe()
```

Wheels (abi3, Python 3.11 or newer) are published for Linux x86_64 and aarch64 (glibc
2.28+), Linux x86_64 (musl), macOS x86_64 and arm64, and Windows x64. On other platforms
pip builds from the source distribution, which needs a Rust toolchain. If the native
extension cannot be loaded, `import ddex_parser` raises an `ImportError` that names the
cause.

## Example

```python
from ddex_parser import DDEXParser

parser = DDEXParser()
with open("release.xml", encoding="utf-8") as f:
    message = parser.parse(f.read())

print(message.message_id(), message.version(), message.message_control_type())
print(message.release_count(), "release(s)")
for deal in message.deals():
    print(deal["deal_id"], deal["releases"], deal["start_date"], deal["end_date"], deal["territories"])

releases = message.to_dataframe("releases")
print(releases[["release_id", "title", "artist", "genre", "label_name", "release_dates"]])

tracks = message.to_dataframe("tracks")
print(tracks[["track_index", "track_title", "isrc", "duration", "disc_number", "group_path"]])
```

- `parse(xml)` accepts `str` or `bytes` and returns a `ParsedERNMessage`:
  `message_id()`, `version()`, `release_count()`, `message_control_type()`, `deals()`
  (one dict per deal) and `to_dataframe(schema)`. These are methods.
- `to_dataframe` schemas: `"flat"` (a message row, then one row per release),
  `"releases"` and `"tracks"`. Columns include release dates as written, P and C lines,
  genres, parental warnings, labels, and each track's position, disc and track numbers
  and enclosing groups. Absent values are `None` or empty lists, never invented.
  Durations are in seconds.
- `parse_async(xml)` is the awaitable version; `detect_version(xml)` returns the enum
  spelling (`"V4_3"`).
- `stream(xml)` parses the whole document and then yields its releases one by one; it
  does not reduce memory use.
- `parse` takes no options (`ParseOptions`, whose fields had no effect, was removed in
  0.6.0).
- `sanity_check(xml)` detects the ERN version (`None` when there is no ERN namespace)
  and checks that the XML is well-formed, listing each problem in `errors`. It is not XSD
  validation, and `parse` can still fail on a document that passes.
- The `ddex-parser` command prints a JSON summary (`ddex-parser parse release.xml`),
  the detected version (`version`), or whether the file parses (`check`; this is not XSD
  validation).

## Security

The parser rejects DTDs and external entities (XXE), expands no entity, limits element
depth, and never fetches from the network. It does not validate against
the DDEX XSD or the AVS business rules.

## Links

- Documentation: [ddex-suite.org](https://ddex-suite.org/docs/quick-start-parsing)
- [Changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md)
- [0.6.0 migration guide](https://github.com/daddykev/ddex-suite/blob/main/website/docs/migration-0.6.md)
- [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md)
- [Known issues](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md)
- [Report an issue](https://github.com/daddykev/ddex-suite/issues)

## License

MIT, see [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE).
