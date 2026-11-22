# ddex-builder

DDEX ERN XML builder in Rust, part of
[DDEX Suite](https://github.com/daddykev/ddex-suite). It writes ERN 4.3, 4.2, 3.8.2 and
3.8.1 messages from a typed build request, canonicalized with DB-C14N/1.0 (fixed
namespace prefixes, sorted attributes, consistent formatting, elements in XSD order).
Node.js and Python bindings use the same builder: `npm install ddex-builder`,
`pip install ddex-builder`.

Byte-for-byte determinism is not complete yet: with the default options, a message ID,
release and resource references and `MessageCreatedDateTime` that the request leaves out
are generated (random UUIDs, the current time), and the stable-hash ID strategy puts
today's date into the `MessageId`, so two builds of the same request can differ. Full
determinism is planned for 0.6.0.

> **0.5.0 has breaking changes.** ERN 3.8.2 output is now schema-valid, the WASM binding
> runs this builder, and the request types gained fields (use `..Default::default()`).
> See the
> [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md).

## Installation

```toml
[dependencies]
ddex-builder = "0.5.0"
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
            message_id: Some("MSG-001".to_string()),
            message_sender: party("PADPIDA0000000001", "Example Label"),
            message_recipient: party("PADPIDA0000000002", "Example DSP"),
            message_control_type: Some("LiveMessage".to_string()),
            message_created_date_time: Some("2024-01-01T00:00:00Z".to_string()),
        },
        version: "4.3".to_string(),
        profile: Some("AudioSingle".to_string()),
        releases: vec![ReleaseRequest {
            release_id: "A1-ABCDE-0000000001-M".to_string(),
            release_reference: Some("R1".to_string()),
            title: text("Example Single"),
            artist: "Example Artist".to_string(),
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
                isrc: "ZZ0000000001".to_string(),
                title: "Example Track".to_string(),
                duration: "PT3M0S".to_string(),
                artist: "Example Artist".to_string(),
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

## Supported versions

| Target | Output |
|---|---|
| ERN 4.3, 4.2 | Validated against the official XSDs in CI |
| ERN 3.8.2 | Validated against the official XSD in CI; video resources are not supported yet |
| ERN 3.8.1 | The 3.8.2 document in the 3.8.1 namespace; not checked against an official XSD (none is available) |

Output is read back by [`ddex-parser`](https://crates.io/crates/ddex-parser). There is
no built-in conversion from a parsed message to a build request yet: map the fields you
need into a `BuildRequest`.

## Scope

- Text is XML-escaped on output. `build` checks the fields the target version requires;
  it does not validate the output against the XSD or the AVS business rules at runtime.
- The preset APIs (`presets`, partner presets) do not change `DDEXBuilder::build` output
  in 0.5.0, and the `ddex-builder` command-line tool writes a placeholder document; see
  the known issues.

## Performance

Criterion benches on an Apple M1 Pro, release profile: ERN 4.3 with 1 track builds in
2.7 ms, 12 tracks in 11.4 ms and 100 tracks in 80 ms, about 0.8 ms per track plus 2 ms
per message ([details](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/PERFORMANCE.md)).

## Links

- Documentation: [docs.rs/ddex-builder](https://docs.rs/ddex-builder) and [ddex-suite.org](https://ddex-suite.org)
- [Changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md)
- [0.5.0 migration notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md)
- [Known issues](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md)
- [Report an issue](https://github.com/daddykev/ddex-suite/issues)

## License

MIT, see [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE).
