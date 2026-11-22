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
| `parse_fast_streaming` (used when `SecurityConfig::relaxed()` / `enable_fast_streaming` is set) returns no releases or resources; the conversion is a TODO in `lib.rs` | `it::fast_streaming::test_fast_streaming_connection_basic`, `…_explicitly`, `…_vs_normal_performance` (perf) |
| `StreamingParser::stream_releases` skips the message root element (only `stream_deals` descends into it), so it yields nothing for a full document | `it::utf8_handling::streaming_parser_handles_utf8` |
| quick-xml's own end-tag/attribute checks fire first and come back as a generic `XmlError`, pre-empting the typed `MismatchedTags` / `UnexpectedClosingTag` / `InvalidAttribute` variants from `parser::xml_validator` | `it::malformed_xml::test_mismatched_opening_closing_tags`, `test_unexpected_closing_tag`, `test_duplicate_attributes`, `test_complex_malformed_structure` |
| `]]>` in character data is accepted (not well-formed XML) and the surrounding text is mangled | `it::malformed_xml::test_cdata_terminator_in_text_is_rejected` |

## ddex-builder

| Bug | Test(s) |
|---|---|
| For ERN 4.3 the builder writes a 3.8-style tree: no `Release/ReleaseType`, `ReferenceTitle` instead of `DisplayTitleText`, no `ResourceGroup`, `MessageCreatedDateTime` before `MessageSender`. ddex-parser rejects the output or reads it without titles | `it::round_trip::parser_reads_back_what_the_builder_wrote`, `rebuilding_from_parsed_output_is_byte_identical` |
| The stable-hash MessageId includes `Utc::now()`'s date (`builder.rs`, `generate_stable_hash_ids`), so output changes daily and the golden snapshot goes stale | `it::golden_files::test_audio_album_golden` |
| `PathValidator`: allows `settings.json`, empty paths, malformed paths, Unicode homographs and `malware.exe`; rejects valid relative paths, paths within the depth limit and valid paths under concurrency; mishandles a bare `\` | `it::security_fixes::*` (5), `it::security::path_validator_tests::*` (4) |
| `InputValidator::validate_xml_content` is regex-only: accepts empty input, doesn't enforce `max_xml_depth`, accepts malformed CDATA and `]]>` in text, and ignores `allow_dtd` (with `false` it accepts an empty DTD or a mixed-case `<!Entity … System` declaration; with `true` it still rejects every internal entity declaration) | `it::security::integration_tests::test_security_edge_cases`, `test_memory_safety_under_attack`, `it::security::malicious_payload_tests::test_cdata_section_abuse`, `test_xml_injection_in_ddex_fields`, `it::security::xxe_prevention_tests::test_xxe_edge_cases`, `it::security::parameter_entity_tests::test_parameter_entity_evasions`, `it::security::entity_expansion_tests::test_reasonable_entities_allowed` |
| `InputValidator::validate_string` / `validate_json_content` reject legitimate metadata text. A keyword blocklist (`select`, `update`, `delete`, `drop`, `union`, `insert`, `exec`, `script`, `javascript`, `onload`, `onerror`, …) flags SQL- and script-looking text, encoded or not, and Unicode lookalikes such as `＜script＞`; a title like "Delete Me" is rejected too. `validate_string` also rejects entity-like `&name;` text and path-like fragments (`/etc/`, `../`) | `it::security::malicious_payload_tests::test_sql_injection_in_metadata`, `test_script_injection_attempts`, `test_encoding_manipulation`, `test_legitimate_unicode_metadata_accepted`, `test_command_like_metadata_text_accepted`, `test_xml_injection_in_ddex_fields`, `it::security::builder_security_tests::test_build_time_xxe_prevention` |
| `DDEXBuilder` writes XML-illegal C0 control characters (U+0000–U+0008, U+000B, U+000C, U+000E–U+001F) from metadata into the output verbatim, so the output is not well-formed; text escaping only handles `&`, `<`, `>` | `it::security::builder_security_tests::test_illegal_xml_chars_never_reach_builder_output` |
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
| Native `detect_version` returns `"V4_3"`, while the pure-Python fallback and the docs use `"4.3"` | `tests/test_parser.py::TestDDEXParser::test_detect_version` |

## Not pinned by a test (seen during the cleanup)

- The Node builder binding's `validateStructure` reports `isValid: true` for invalid XML (`npm test` prints it without failing).
- The Python parser reports `__version__ == "0.4.3"` while the package is 0.4.5.
- ERN prefixed local elements (`<ern:PartyId>`, not schema-valid) are handled inconsistently: some are found, `MessageSender/PartyId` is not.
