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
| `ParseMode::Stream` fails on any message with a release: it builds releases without a `ReleaseType`, and flattening requires one ("Required DDEX field missing: Release/ReleaseType") | `it::reader_paths::every_reader_accepts_the_official_sample` (pins the current error) |
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
| The stable-hash MessageId includes `Utc::now()`'s date (`builder.rs`, `generate_stable_hash_ids`), so output changes daily and the golden snapshot goes stale | `it::golden_files::test_audio_album_golden`, `it::determinism_0_6::stable_hash_message_id_is_not_time_derived`, `…::id_strategy_stable_hash_is_byte_identical` |
| Not deterministic (0.6.0; rows D1–D3 of `dev-docs/DETERMINISM_AUDIT_0.6.0.md`): `IdStrategy::UUID` (the default) fills an omitted `MessageId` with `MSG_<uuid v4>` and omitted release/resource references with `R<uuid>` / `A<uuid>`; `IdStrategy::UUIDv7` falls back to the same v4 IDs | `it::determinism_0_6::default_options_generated_ids_are_byte_identical`, `…::id_strategy_uuid_is_byte_identical`, `…::id_strategy_uuidv7_is_byte_identical` |
| Not deterministic (0.6.0; D4): `IdStrategy::Sequential` makes the `MessageId` `MSG_` plus the current Unix time | `it::determinism_0_6::id_strategy_sequential_is_byte_identical`, `…::sequential_message_id_is_not_time_derived` |
| `IdStrategy::StableHash` references are `B3:<hex>` / `SHA256:<hex>`, which do not match the ERN reference pattern (`X[\d\-_a-zA-Z]+`), so a build that leaves a resource or release reference to it fails; its deal references (`DEAL_<model>_<territories>`) repeat for deals with the same terms (D6, D7) | `it::determinism_0_6::id_strategy_stable_hash_is_byte_identical` |
| Not deterministic (0.6.0; D8): an omitted `MessageCreatedDateTime` is filled with the current time (`generator/ern4.rs`, `generator/ern38.rs`, `message_header`) instead of failing with an error that names the field | `it::determinism_0_6::omitted_message_created_date_time_is_an_error` |
| Not deterministic (0.6.0; D12, D13): the streaming writer (`streaming::StreamingBuilder`) fills an omitted `MessageId` with a random UUID and an omitted `MessageCreatedDateTime` with the current time (`write_message_header`) | `it::determinism_0_6::streaming_writer_omitted_header_values_are_byte_identical` |
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

## Unresolved dependency advisories

`cargo deny check advisories` runs in CI; these are ignored in `deny.toml`, each with its
reason, until the upgrade lands.

| Advisory | Crate | Status |
|---|---|---|
| RUSTSEC-2026-0543 (unmaintained) | chrono | Direct dependency of every crate; no replacement chosen. |
| RUSTSEC-2026-0249 (unmaintained) | smartstring | ddex-builder; to be replaced. |
| RUSTSEC-2025-0119 (unmaintained) | number_prefix | Via indicatif 0.17 (parser CLI); indicatif 0.18 drops it. |
| RUSTSEC-2024-0436 (unmaintained) | paste | Via rmp / rmp-serde (parser CLI MessagePack output). |
| RUSTSEC-2024-0449 (unmaintained) | unsafe-libyaml | Via serde_yaml (deprecated; CLI YAML support). |
| RUSTSEC-2026-0441 (unmaintained) | pin-utils | Via futures-util (pyo3-async-runtimes). |

## Not pinned by a test (seen during the cleanup)

- ddex-parser `ParseMode::Stream` (experimental, opt-in since 0.5.0; `parse()` never selects
  it): `parser/stream.rs` reads a subset of the fields (no resources or parties, no
  territory details, dates, P/C lines, genres, labels, track layout, ...) and still trims
  text. (`StreamingParser::stream_resources` / `stream_parties`, which were not
  implemented, were removed in 0.6.0.)
- ddex-parser ERN 3.8.x release display artists: each `DisplayArtist/PartyName/FullName`
  becomes an artist with the role `MainArtist` regardless of its `ArtistRole` (kept as
  before 0.5.0 so no existing value changed; 3.8.x track artists use the real role).
- ddex-parser: with several `ValidityPeriod`s in one deal, the last start / end date
  given wins; `StartDateTime` / `EndDateTime` are not read; the flat deal has no
  `CommercialModelType` (Node `commercialModel` is always null).
- ddex-parser ERN 3.8.x: only the first `ResourceGroup` of the default
  `ReleaseDetailsByTerritory` block is kept in the graph; resource `ProprietaryId`s
  (`SoundRecordingId`, `VideoId`, `ImageId`) are not read.
- ddex-builder output is not deterministic (planned for 0.6.0). The cases above (the
  "Not deterministic" rows) are now pinned by `it::determinism_0_6`; the full inventory is
  `dev-docs/DETERMINISM_AUDIT_0.6.0.md`. Giving `message_id`, `message_created_date_time`,
  `release_reference` and each track's `resource_reference` makes `DDEXBuilder::build`
  output repeatable. Not pinned by a test: the streaming writer's references are SHA-256 of the caller's IDs cut to 8 hex digits, so a large
  catalog can fail with a reference collision (D14).
- ERN 3.8.1 builder output is not XSD-checked: no official 3.8.1 schema can be obtained
  (DDEX no longer serves ERN 3.x XSDs, and no archived copy of the 3.8.1 one exists). It
  is the 3.8.2 document in the 381 namespace (`it::ern38_xsd`), and the content models
  of every element written are the same in an unofficial copy of the 3.8.1 schema.
- The ERN 3.8.2 XSD is fetched from the Internet Archive's capture of its official URL
  (DDEX removed the ERN 3.x schemas from service.ddex.net in 2026), pinned by hash.
- ddex-builder streaming writer (`streaming::StreamingBuilder`, Node `StreamingDdexBuilder`): ERN 4.2/4.3 targets are rejected with "not yet supported by the streaming writer" (it only writes the 3.8-style tree; use `DDEXBuilder::build` for 4.x). Its 3.8 output still declares the `ern/43` namespace whatever version is requested, and is the old 3.8-style tree, not the schema-valid 3.8.2 that `DDEXBuilder::build` writes.
- ERN prefixed local elements (`<ern:PartyId>`, not schema-valid) are handled inconsistently: some are found, `MessageSender/PartyId` is not.

## Not implemented in 0.5.x

These APIs returned made-up results before 0.5.1 (success, scores, counts, invented values).
They now fail, or report the value as not measured, instead:

- ddex-builder round trips: `RoundTripTester::test_round_trip_with`,
  `DDEXBuilder::check_round_trip` and `round_trip::verify_round_trip` take the
  parser's parse function and run a real round trip. Node `checkRoundTrip` and Python
  `check_round_trip` run it. Round trips compare only the fields a build request
  carries (contributors, P and C lines, images, ... are not rebuilt), and
  `fidelity_score` / `fidelityScore` is always null. (The fake round-trip, verification,
  fidelity-analysis and determinism-guarantee APIs were removed in 0.6.0.)
- Node / Python builder bindings: `buildWithStatistics` / `build_with_statistics` reject a
  `canonicalization` other than `db_c14n` or `none`. Memory, extension count and
  canonicalization time are not measured (null). (The fidelity options that had no effect
  were removed in 0.6.0.)
- ddex-core `AttributeValidator::add_custom_validator` does not store the closure: only
  the built-in validators `ddex_territory_code`, `ddex_language_code` and
  `ddex_currency_code` can be registered, and a custom rule naming any other validator
  fails validation as unknown.

## Parked

- The parsed models leave some DDEX fields always empty (`Release.party_list`, `Resource.rights_controller`, `Party.party_role` / `Party.contact_details`, `DealTerms.distribution_channel` / `DealTerms.price_information`); these elements are not parsed in 0.6.0.
- Node parser `src/index.ts`, `src/parser.ts`, `src/types.ts`, `src/wasm.d.ts` and `js/index.ts` are unbuilt, unpublished TypeScript wrappers that still declare `ParseOptions` and `parse_stream`; CI and `npm publish` use only `index.js` / `index.d.ts`.
- `docs/python_parser_api.txt` (a pydoc dump) still lists `ParseOptions` and `ParseResult`, and `docs/DB-C14N-SPEC.md` describes a `preserve_comments` option.
- KNOWN_ISSUES.md repeats the end of its "Not implemented in 0.5.x" section (from "They now fail, or report the value as not measured" on) a second time at the end of the file.
