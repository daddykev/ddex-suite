# ddex-builder (Python)

DDEX ERN XML builder for Python, a native extension over the Rust
[`ddex-builder`](https://crates.io/crates/ddex-builder) crate. Part of
[DDEX Suite](https://github.com/daddykev/ddex-suite). It writes ERN 4.3, 4.2, 3.8.2 and
3.8.1 from a build request (a dict) or a pandas DataFrame, canonicalized with
DB-C14N/1.0 (fixed namespace prefixes, sorted attributes, consistent formatting,
elements in XSD order).

Byte-for-byte determinism is not complete yet: a message ID, release and resource
references and `message_created_date_time` that the request leaves out are generated
(random UUIDs, the current time), so two builds of the same request can differ. Full
determinism is planned for 0.6.0.

> **0.5.0 has breaking changes.** `build(request)` and `from_dataframe(df, schema,
> request)` run the core builder (nothing is filled in), and importing without the native
> extension raises `ImportError`. See the
> [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md).

## Installation

```bash
pip install ddex-builder
pip install "ddex-builder[pandas]"   # for from_dataframe()
```

Wheels (abi3, Python 3.10 or newer) are published for Linux x86_64 and aarch64 (glibc
2.28+), Linux x86_64 (musl), macOS x86_64 and arm64, and Windows x64. On other platforms
pip builds from the source distribution, which needs a Rust toolchain. If the native
extension cannot be loaded, `import ddex_builder` raises an `ImportError` that names the
cause.

## Example

```python
from ddex_builder import DdexBuilder


def party(name, party_id):
    return {"party_name": [{"text": name}], "party_id": party_id}


request = {
    "header": {
        "message_id": "MSG-001",
        "message_sender": party("Example Label", "PADPIDA0000000001"),
        "message_recipient": party("Example DSP", "PADPIDA0000000002"),
        "message_created_date_time": "2024-01-01T00:00:00Z",
    },
    "version": "4.3",
    "avs_version_id": "12",
    "language_and_script_code": "en",
    "releases": [
        {
            "release_id": "A1-ABCDE-0000000001-M",
            "title": [{"text": "Example Single"}],
            "artist": "Example Artist",
            "label": "Example Label",
            "release_type": "Single",
            "genre": "Pop",
            "parental_warning_type": "NotExplicit",
            "tracks": [
                {
                    "track_id": "T1",
                    "isrc": "ZZ0000000001",
                    "title": "Example Track",
                    "duration": "PT3M42S",
                    "artist": "Example Artist",
                    "resource_type": "MusicalWorkSoundRecording",
                    "parental_warning_type": "NotExplicit",
                }
            ],
        }
    ],
    "deals": [],
}

xml = DdexBuilder().build(request)
print(xml)
```

The request has the fields of the Rust `BuildRequest`; `ddex_builder.BuildRequest` and
the other `TypedDict`s in `ddex_builder.build_request` describe them for type checkers.
`header` and `version` are required. When the target version's schema requires a value
the request does not carry, `build` raises an error naming the field, for example
`ERN 4.3 requires Release/ReleaseType (releases[0].release_type)`.

### From a DataFrame

`from_dataframe(df, schema, request)` builds one release from the rows (the column names
of ddex-parser's DataFrames); `request` supplies the header, version and the other
message-level values:

```python
import pandas as pd
from ddex_builder import DdexBuilder

message = {
    "header": {
        "message_sender": {"party_name": [{"text": "Example Label"}], "party_id": "PADPIDA0000000001"},
        "message_recipient": {"party_name": [{"text": "Example DSP"}], "party_id": "PADPIDA0000000002"},
    },
    "version": "4.3",
    "avs_version_id": "12",
    "language_and_script_code": "en",
}
df = pd.DataFrame(
    [
        {
            "release_id": "A1-ABCDE-0000000003-M",
            "release_reference": "R0",
            "release_title": "Frame Album",
            "release_artist": "Example Artist",
            "release_label": "Example Records",
            "release_genre": "Pop",
            "release_type": "Album",
            "release_parental_warning_type": "NotExplicit",
            "track_index": i,
            "track_title": f"Track {i + 1}",
            "artist": "Example Artist",
            "isrc": f"ZZ000000010{i}",
            "duration": 201.0,  # seconds
            "resource_type": "MusicalWorkSoundRecording",
            "parental_warning_type": "NotExplicit",
        }
        for i in range(3)
    ]
)
xml = DdexBuilder().from_dataframe(df, schema="tracks", request=message)
print(len(xml), "bytes")
```

## Supported versions

| `version` | Output |
|---|---|
| `"4.3"`, `"4.2"` | Validated against the official XSDs in CI |
| `"3.8.2"` | Validated against the official XSD in CI; video resources are not supported yet |
| `"3.8.1"` | The 3.8.2 document in the 3.8.1 namespace; not checked against an official XSD (none is available) |

## Scope

- Use `build(request)` or `from_dataframe(...)`. The preset methods (`apply_preset`, ...)
  list the builder's presets, which do not change the output in 0.5.x; see the known
  issues.
- `test_round_trip_fidelity(xml)` parses, builds and parses again, and lists in `issues`
  the differences on the fields a build request carries (round trips are not lossless:
  contributors, P/C lines, images and other data are not rebuilt). `fidelity_score` is
  always `None`: no score is computed.
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
