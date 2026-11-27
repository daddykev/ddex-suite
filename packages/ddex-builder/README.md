# ddex-builder

DDEX ERN XML builder in Rust, part of
[DDEX Suite](https://github.com/daddykev/ddex-suite). It writes ERN 4.3, 4.2, 3.8.2 and
3.8.1 messages from a typed build request, canonicalized with DB-C14N/1.0 (fixed
namespace prefixes, sorted attributes, consistent formatting, elements in XSD order).
Node.js and Python bindings use the same builder: `npm install ddex-builder`,
`pip install ddex-builder`.

The builder reads no clock or random source and uses no unordered iteration on the build
path (enforced by clippy lints). The same request and options produce byte-identical output
with the same crate version; repeated builds are tested to be byte-identical.
`MessageId` and `MessageCreatedDateTime` are required inputs. References the request leaves
out are numbered in request order (`R1`, `A1`, `D1`, ...).

> **0.5.0 has breaking changes.** ERN 3.8.2 output is now schema-valid, the WASM binding
> runs this builder, and the request types gained fields (use `..Default::default()`).
> See the
> [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md).

## Installation

```toml
[dependencies]
ddex-builder = "0.5.1"
```

## Example

```rust
use ddex_builder::builder::{
    LocalizedStringRequest, MessageHeaderRequest, PartyRequest, ReleaseRequest, TrackRequest,
};
use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};

fn main() -> Result<(), ddex_builder::BuildError> {
    let text = |s: &str| vec![LocalizedStringRequest { text: s.to_string(), language_code: None }];
    let party = |id: &str, name: &str| PartyRequest {
        party_name: text(name),
        party_id: Some(id.to_string()),
        party_reference: None,
    };

    let request = BuildRequest {
        header: MessageHeaderRequest {
            message_id: "MSG-001".to_string(),
            message_sender: party("PADPIDA0000000001", "Example Label"),
            message_recipient: party("PADPIDA0000000002", "Example DSP"),
            message_control_type: Some("LiveMessage".to_string()),
            message_created_date_time: "2024-01-01T00:00:00Z".to_string(),
        },
        version: "4.3".to_string(),
        profile: Some("AudioSingle".to_string()),
        releases: vec![ReleaseRequest {
            release_id: Some("A1-ABCDE-0000000001-M".to_string()),
            release_reference: Some("R1".to_string()),
            title: text("Example Single"),
            artist: Some("Example Artist".to_string()),
            label: Some("Example Records".to_string()),
            release_date: None,
            upc: Some("123456789012".to_string()),
            // Required by ERN 4.x (the build fails, naming each one, if they are missing)
            release_type: Some("Single".into()),
            genre: Some("Pop".to_string()),
            parental_warning_type: Some("NotExplicit".to_string()),
            tracks: vec![TrackRequest {
                track_id: "TRK001".to_string(),
                resource_reference: Some("A1".to_string()),
                isrc: Some("ZZ0000000001".to_string()),
                title: "Example Track".to_string(),
                duration: "PT3M0S".to_string(),
                artist: Some("Example Artist".to_string()),
                resource_type: Some("MusicalWorkSoundRecording".into()),
                parental_warning_type: Some("NotExplicit".to_string()),
                ..Default::default()
            }],
            resource_references: None,
            ..Default::default()
        }],
        deals: vec![],
        extensions: None,
        language_and_script_code: Some("en".to_string()),
        avs_version_id: Some("12".to_string()),
    };

    // Canonical (DB-C14N/1.0) ERN 4.3. The message ID, creation time and references are
    // given, so none of them is generated.
    let result = DDEXBuilder::new().build(request, BuildOptions::default())?;
    assert!(result.xml.contains("<MessageId>MSG-001</MessageId>"));
    println!("{}", result.xml);
    Ok(())
}
```

Nothing is filled in for you: when the target version's schema requires a value the
request does not carry, `build` fails with an error naming the field (for example
`ERN 4.3 requires Release/ReleaseType (releases[0].release_type)`).

Build requests are constructed with `..Default::default()`; new optional fields may be
added in minor releases. A value the schema makes optional is an `Option`: `None` leaves
the element out, and an empty string (`Some("")`) fails the build.

## Supported versions

| Target | Output |
|---|---|
| ERN 4.3, 4.2 | Validated against the official XSDs in CI |
| ERN 3.8.2 | Validated against the official XSD in CI; video resources are not supported yet |
| ERN 3.8.1 | The 3.8.2 document in the 3.8.1 namespace; not checked against an official XSD (none is available) |

Output is read back by [`ddex-parser`](https://crates.io/crates/ddex-parser).

## Parse, modify, build

`BuildRequest::from_parsed` maps a parsed message to a build request for the same ERN
version. It invents nothing: absent values stay `None`. `unmapped` lists the dotted path of
every non-empty parsed value the request cannot hold (P and C lines, contributors, images,
...; the full list is in the rustdoc).

```rust
use ddex_builder::{BuildOptions, BuildRequest, DDEXBuilder};
use ddex_parser::DDEXParser;
use std::io::Cursor;

let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/Samples43/1 Audio.xml");
let xml = std::fs::read_to_string(path)?;

// Parse, map to a build request, modify, build
let parsed = DDEXParser::new().parse(Cursor::new(xml.as_bytes()))?;
let mut bridge = BuildRequest::from_parsed(&parsed);
bridge.request.releases[0].title[0].text = "Yume no Lullaby (Remastered)".to_string();
let built = DDEXBuilder::new().build(bridge.request, BuildOptions::default())?;

// Read it back
let rebuilt = DDEXParser::new().parse(Cursor::new(built.xml.as_bytes()))?;
assert_eq!(
    rebuilt.flat.releases[0].default_title.as_deref(),
    Some("Yume no Lullaby (Remastered)")
);
// What the request could not carry, e.g. the sample's MessageThreadId
assert!(bridge.unmapped.contains(&"header.message_thread_id".to_string()));
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Scope

- Text is XML-escaped on output. `build` checks the fields the target version requires;
  it does not validate the output against the XSD or the AVS business rules at runtime.
- The `ddex-builder` command-line tool's `build` runs `DDEXBuilder::build`; its
  `validate`, `watch` and `server` are not implemented. See the known issues. (The preset
  APIs, update messages and the diff were removed in 0.6.0; see the changelog.)

## Performance

Criterion benches on a GitHub-hosted runner (Intel Xeon, 4 vCPUs), release profile,
0.5.1: ERN 4.3 with 1 track builds in 1.7 ms, 12 tracks in 8.3 ms and 100 tracks in 58 ms,
about 0.56 ms per track plus 1–1.5 ms per message ([details](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/PERFORMANCE.md)).

## Links

- Documentation: [docs.rs/ddex-builder](https://docs.rs/ddex-builder) and [ddex-suite.org](https://ddex-suite.org)
- [Changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md)
- [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md)
- [Known issues](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md)
- [Report an issue](https://github.com/daddykev/ddex-suite/issues)

## License

MIT, see [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE).
