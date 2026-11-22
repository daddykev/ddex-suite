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
| `InputValidator::validate_xml_content`: accepts empty input, doesn't enforce `max_xml_depth`, accepts malformed CDATA, accepts an empty DTD although `allow_dtd` is false | `it::security::integration_tests::test_security_edge_cases`, `test_memory_safety_under_attack`, `it::security::malicious_payload_tests::test_cdata_section_abuse`, `it::security::xxe_prevention_tests::test_xxe_edge_cases` |
| `InputValidator::validate_string` misses event-handler XSS vectors such as `onfocus=` | `it::security::malicious_payload_tests::test_script_injection_attempts` |
| `OutputSanitizer::sanitize_xml_output` HTML-escapes the whole document instead of validating it | `it::security::builder_security_tests::test_output_validation`, `it::security::integration_tests::test_output_sanitization_pipeline` |
| Secure log messages still contain sensitive paths (e.g. `passwd`) | `it::security::integration_tests::test_secure_logging` |

### Needs a security-policy decision

The revived `tests/security/` suite also asserts behaviour that may not be wanted.
These tests are ignored with "known bug or wrong expectation (security policy
decision needed)". Decide the policy, then fix the code or the test:

- Flagging SQL-injection-like text (`' OR '1'='1`) in metadata, which legitimately contains quotes.
- Rejecting JSON whose string *values* contain DOCTYPE/XXE text (it is escaped on output).
- Rejecting `<script>` elements in XML input, and mixed-case `<!eNtItY` (not well-formed anyway).
- Rejecting every internal entity declaration, even small ones.
- Schema-checking JSON input types in `validate_json_input`.
- Error-message wording for rejected paths (three tests expect specific keywords).
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
