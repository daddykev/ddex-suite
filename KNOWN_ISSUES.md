# Known issues

Real bugs that the test suite currently pins as ignored. Each one is a test marked
`#[ignore = "known bug: …"]` (Rust) or `pytest.mark.xfail(strict=True, …)` (Python).
Run them with `cargo test -- --ignored` (or `--include-ignored`). When you fix a
bug, remove the ignore/xfail and delete its entry here. A strict xfail starts
failing as soon as the bug is fixed, as a reminder.

Classification and history: `dev-docs/TEST_TRIAGE.md`.

## ddex-parser

| Bug | Test(s) |
|---|---|
| `parse_fast_streaming` (used when `SecurityConfig::relaxed()` / `enable_fast_streaming` is set) returns no releases or resources; the conversion is a TODO in `lib.rs` | `it::fast_streaming::test_fast_streaming_connection_basic`, `…_explicitly` |
| `StreamingParser::stream_releases` skips the message root element (only `stream_deals` descends into it), so it yields nothing for a full document | `it::utf8_handling::streaming_parser_handles_utf8` |
| quick-xml's own end-tag/attribute checks fire first and come back as a generic `XmlError`, pre-empting the typed `MismatchedTags` / `UnexpectedClosingTag` / `InvalidAttribute` variants from `parser::xml_validator` | `it::malformed_xml::test_mismatched_opening_closing_tags`, `test_unexpected_closing_tag`, `test_duplicate_attributes`, `test_complex_malformed_structure` |
| `]]>` in character data is accepted (not well-formed XML) and the surrounding text is mangled | `it::malformed_xml::test_cdata_terminator_in_text_is_rejected` |

## ddex-builder

| Bug | Test(s) |
|---|---|
| ERN 3.8.x targets: Video resources are rejected (`resource_kind is Video`; the 3.8 generator writes `SoundRecording`s only) | `it::ern38_round_trip` (expected failures: 12 YouTube video samples, 9 official 4.x samples with videos) |
| ERN 3.8.x targets: `ReleaseIdRequest` has no ISRC, so a 3.8.2 release (or track release) identified only by `ReleaseId/ISRC` cannot be rebuilt (the parser reads it since 0.5.0) | `it::ern38_round_trip` (expected failures: the 8 YouTube 3.8.2 audio samples `721620118165_*`, `USGCV1172318_*`) |
| ERN 3.8.2 target: an empty `MessageId` (`<MessageId/>`, valid `xs:string`, parsed as `""`) is reported as missing, while the 4.x targets write it | `it::ern38_round_trip::official_4x_samples_rebuilt_as_ern382` (expected failures: Samples42/43 `8 DjMix.xml`) |
| The request model cannot express a present but empty title: `TrackRequest.title` / release title `""` mean "not given" and the build fails with the required-field error (whitespace-only titles are written faithfully) | `it::whitespace_round_trip::empty_title_is_reported_missing_by_the_builder` |
| The stable-hash MessageId includes `Utc::now()`'s date (`builder.rs`, `generate_stable_hash_ids`), so output changes daily and the golden snapshot goes stale | `it::golden_files::test_audio_album_golden` |
| `PathValidator`: allows `settings.json`, empty paths, malformed paths, Unicode homographs and `malware.exe`; rejects valid relative paths, paths within the depth limit and valid paths under concurrency; mishandles a bare `\` | `it::security_fixes::*` (5), `it::security::path_validator_tests::*` (4) |
| `InputValidator::validate_xml_content` is regex-only: accepts empty input, doesn't enforce `max_xml_depth`, accepts malformed CDATA and `]]>` in text, and ignores `allow_dtd` (with `false` it accepts an empty DTD or a mixed-case `<!Entity … System` declaration; with `true` it still rejects every internal entity declaration) | `it::security::integration_tests::test_security_edge_cases`, `test_memory_safety_under_attack`, `it::security::malicious_payload_tests::test_cdata_section_abuse`, `test_xml_injection_in_ddex_fields`, `it::security::xxe_prevention_tests::test_xxe_edge_cases`, `it::security::parameter_entity_tests::test_parameter_entity_evasions`, `it::security::entity_expansion_tests::test_reasonable_entities_allowed` |
| `InputValidator::validate_string` / `validate_json_content` reject legitimate metadata text. A keyword blocklist (`select`, `update`, `delete`, `drop`, `union`, `insert`, `exec`, `script`, `javascript`, `onload`, `onerror`, …) flags SQL- and script-looking text, encoded or not, and Unicode lookalikes such as `＜script＞`; a title like "Delete Me" is rejected too. `validate_string` also rejects entity-like `&name;` text and path-like fragments (`/etc/`, `../`) | `it::security::malicious_payload_tests::test_sql_injection_in_metadata`, `test_script_injection_attempts`, `test_encoding_manipulation`, `test_legitimate_unicode_metadata_accepted`, `test_command_like_metadata_text_accepted`, `test_xml_injection_in_ddex_fields`, `it::security::builder_security_tests::test_build_time_xxe_prevention` |
| `OutputSanitizer::sanitize_xml_output` HTML-escapes (rewrites) every document it accepts instead of validating it; it also accepts truncated XML (`<root><unclosed>`) and misses sensitive elements such as `<ddex:Password>` | `it::security::builder_security_tests::test_output_validation`, `it::security::integration_tests::test_output_sanitization_pipeline` |
| Secure log messages still contain sensitive paths (e.g. `passwd`) | `it::security::integration_tests::test_secure_logging` |

Security policy used by these tests: the libraries defend against XML-level threats
(XXE, entity expansion, DTD handling, depth/size limits, CDATA handling, escaping of
output). SQL-, HTML- or script-looking text in metadata is accepted, passed through
unchanged and XML-escaped on output. Characters that are illegal in XML 1.0 may be
rejected; legitimate Unicode is accepted.

### Needs a decision

These tests are ignored with "known bug or wrong expectation (security policy
decision needed)". The security policy above does not settle them:

- Error-message wording for rejected paths (three tests expect specific keywords; the paths are rejected).
- Warning on a 48-character filename.

## Python bindings

| Bug | Test(s) |
|---|---|
| Native `ParsedERNMessage.message_id` is a method, not the documented `str` attribute | `tests/test_parser.py::TestDDEXParser::test_parse_string`, `test_parse_bytes`, `test_parse_async`, `test_convenience_function` |
| Native `detect_version` returns `"V4_3"`, while the docs use `"4.3"` | `tests/test_parser.py::TestDDEXParser::test_detect_version` |

## Not pinned by a test (seen during the cleanup)

- ddex-parser `ParseMode::Stream` (experimental, opt-in since 0.5.0; `parse()` never selects
  it): `parser/stream.rs` reads a subset of the fields (no territory details, dates, P/C
  lines, genres, labels, track layout, ...) and still trims text.
- ddex-parser ERN 3.8.x release display artists: each `DisplayArtist/PartyName/FullName`
  becomes an artist with the role `MainArtist` regardless of its `ArtistRole` (kept as
  before 0.5.0 so no existing value changed; 3.8.x track artists use the real role).
- ddex-parser: with several `ValidityPeriod`s in one deal, the last start / end date
  given wins; `StartDateTime` / `EndDateTime` are not read; the flat deal has no
  `CommercialModelType` (Node `commercialModel` is always null).
- ddex-parser ERN 3.8.x: only the first `ResourceGroup` of the default
  `ReleaseDetailsByTerritory` block is kept in the graph; resource `ProprietaryId`s
  (`SoundRecordingId`, `VideoId`, `ImageId`) are not read.
- Node parser binding: `parse` / `parseSync` take no working options. Every
  `ParseOptions` field throws since 0.5.0: the streaming ones (`mode`, `autoThreshold`,
  `enableStreaming`, `streamingThreshold`) and the ones that had no effect
  (`resolveReferences`, `includeRaw`, `maxMemory`, `timeoutMs`, `allowBlocking`,
  `chunkSize`, the fidelity options, `collectStatistics`, `validationLevel`, ...).
  `ParsedMessage.statistics` (all zeros) and `fidelityInfo` (an echo of the options) were
  removed.
- ddex-builder output is not deterministic (planned for 0.6.0). Values the request leaves
  out are generated from randomness or the clock:
  - `DDEXBuilder::build`, default `IdStrategy::UUID` (and `UUIDv7`, which falls back to
    it): `MessageId` `MSG_<uuid v4>`, release references `R<uuid>`, resource references
    `A<uuid>` (`builder.rs`, `generate_uuid_ids`).
  - `IdStrategy::Sequential`: `MessageId` is `MSG_` plus the current Unix time
    (`generate_sequential_ids`). `IdStrategy::StableHash`: the `MessageId` hash includes
    today's date (`generate_stable_hash_ids`; the pinned golden-file entry above).
  - `MessageCreatedDateTime` falls back to the current time (`generator/ern4.rs` and
    `generator/ern38.rs`, `message_header`).
  - Update messages (`create_update*`, `serialize_update`): `MessageCreatedDateTime` and
    `UpdateCreatedDateTime` are always the current time (`messages/update_release.rs`,
    `create_update_metadata`), and an entity ID that cannot be read from the diff path
    becomes `unknown_` plus random hex (`extract_entity_id`).
  - Streaming writer (`streaming::StreamingBuilder`): a missing `MessageId` becomes a random
    UUID and a missing `MessageCreatedDateTime` the current time (`write_message_header`).

  Giving `message_id`, `message_created_date_time`, `release_reference` and each track's
  `resource_reference` makes `DDEXBuilder::build` output repeatable; update messages always
  carry the current time.
- ddex-parser `DDEXParser::sanity_check` is a placeholder (`lib.rs`): it returns
  `is_valid: true`, version 4.3 and no errors without reading its input. The Python
  `sanity_check` and Node `sanityCheck` call it (Node first rejects empty input and input
  that does not start with `<`). The Python CLI's `check` command does a full parse instead.
- Python parser binding: no `ParseOptions` field changes the result. `include_raw_extensions`
  and `include_comments` reach the Rust `ParseOptions`, which nothing reads;
  `validate_references` (Rust `resolve_references`) only controls `tracing` warnings for
  unresolved references, which the binding does not surface; `streaming` is not read by
  `rust_parse_options_from_dict`; `timeout` is left out by `ParseOptions.to_dict`. The fields
  are kept (accepted for forward compatibility) so existing code keeps working.
- ERN 3.8.1 builder output is not XSD-checked: no official 3.8.1 schema can be obtained
  (DDEX no longer serves ERN 3.x XSDs, and no archived copy of the 3.8.1 one exists). It
  is the 3.8.2 document in the 381 namespace (`it::ern38_xsd`), and the content models
  of every element written are the same in an unofficial copy of the 3.8.1 schema.
- The ERN 3.8.2 XSD is fetched from the Internet Archive's capture of its official URL
  (DDEX removed the ERN 3.x schemas from service.ddex.net in 2026), pinned by hash.
- ddex-builder streaming writer (`streaming::StreamingBuilder`, Node `StreamingDdexBuilder`): ERN 4.2/4.3 targets are rejected with "not yet supported by the streaming writer" (it only writes the 3.8-style tree; use `DDEXBuilder::build` for 4.x). Its 3.8 output still declares the `ern/43` namespace whatever version is requested, and is the old 3.8-style tree, not the schema-valid 3.8.2 that `DDEXBuilder::build` writes.
- ddex-builder WASM binding: `DdexDiffViewer` compares the two documents as single text nodes (it wraps each document in one text node instead of parsing it, like `DDEXBuilder::diff_xml`), so any difference is one change.
- ddex-parser `FastStreamingParser::new(StreamingConfig)` ignores the config: buffer size, `max_memory` and the `SecurityConfig` limits are not applied on the fast path (`SecurityConfig::relaxed()` / `enable_fast_streaming`).
- ddex-builder `DDEXBuilder::diff_xml` / `diff_xml_with_config` / `diff_request_with_xml` are stubs: `parse_xml_to_ast` wraps the whole document in one text node, so any difference is reported as a single `/#text` modification. The `DiffEngine` itself works on real ASTs.
- ddex-builder CLI: `build` writes a placeholder document (`build_ddex_xml` returns `<xml><!-- DDEX XML would be generated here --></xml>`), and `--validate` never fails (`validate_input_data` returns `Ok(())`). Presets and the target version set on `ddex_builder::Builder` do not affect `DDEXBuilder::build` output. (The no-op `batch` subcommand was removed for the same reason.)
- The Node builder binding's `validateStructure` reports `isValid: true` for invalid XML (`npm test` prints it without failing).
- ERN prefixed local elements (`<ern:PartyId>`, not schema-valid) are handled inconsistently: some are found, `MessageSender/PartyId` is not.
