# Test triage (test-cleanup branch)

Baseline: `main` @ 57ea322, after `cargo clean`, debug profile, Apple Silicon.
Status columns come from running every test binary on its own, because one
non-compiling target aborts a plain `cargo test --workspace`.

Categories:

- **KEEP**: tests real behaviour against the current API.
- **KEEP-IGNORED**: fails because of a real bug. Marked `#[ignore = "known bug: …"]`
  and listed in `KNOWN_ISSUES.md`.
- **REWRITE**: worth keeping, but it tested the wrong thing or used invalid input.
- **DELETE**: milestone/report tests, duplicates, no-assertion tests, dead APIs,
  placeholder assertions.
- **DEFER**: exercises only streaming implementations that may be removed. Gated
  behind the `legacy-streaming-tests` feature (off by default).
- **PERF**: since 0.5.0, timings are benchmarks that report numbers
  (`cargo bench`, recorded in `dev-docs/PERFORMANCE.md`). The `perf-tests` feature keeps
  only a few sanity limits (about 10× the recorded timings, release mode only):
  `cargo test --release --features perf-tests perf::`.

DELETE approval key: **P** = pre-approved category (milestone, report generator,
duplicate, no-assertion), **A** = approved by Kevin during triage (dead APIs,
low-value tests).

## Baseline numbers

| | |
|---|---|
| Rust integration test files compiled | 54 (core 1, parser 34, builder 19) |
| Rust test files never compiled (orphans) | 18 (builder `tests/{security,integration,determinism,stress}/`, 3 orphan src test files) |
| Test binaries | 60 (incl. 8 binding cdylibs with 0 tests and 4 bin targets) + 3 doctest runs |
| Integration targets that do not compile | 4: `type_conversion_test`, `playground_regression_test`, `model_consistency`, `v0_4_0_final_tests` |
| Rust tests | 624 passed / 56 failed / 4 ignored (+18 tests in non-compiling targets) |
| Doctests | ddex-builder 28 passed / 22 failed; core and parser have none |
| target/ after the full test build | 3.7 GB (17.8 GB before `cargo clean`) |
| Wall-clock time, sum of binaries | ~390 s (parser lib ~97 s and the parser bin's duplicate copy ~97 s, `validate_performance_achievement` ~103 s, `v0_4_0_production` ~65 s) |

## ddex-core

| File / module | Tests | Status | Category | Reason |
|---|---|---|---|---|
| tests/attribute_preservation_tests.rs | 12 | pass | KEEP | Real attribute round-trip checks |
| src inline (attributes, extensions, comprehensive_tests, streaming_types, validation, namespace, common::identifier) | 57 | pass | KEEP | Sensible unit tests |
| src/models/versions/tests.rs | 2 | not compiled (not in module tree) | DELETE (A) | Orphaned test file |

## ddex-parser: integration tests

| File | Tests | Status | Category | Reason |
|---|---|---|---|---|
| community_prs_regression.rs | 8 | pass | KEEP | Issue #1 / PRs #2–#4 regressions |
| ern4_tracks.rs | 16 | pass | KEEP | ERN 4.x track extraction; adds an official-sample track-count test (oracle numbers) |
| depth_limit_test.rs | 4 | pass | KEEP | Depth-limit enforcement |
| test_depth_fix.rs | 1 | fail (fixture lacks the required 4.3 `ReleaseType`) | REWRITE | Siblings-are-not-depth regression; merged into the depth_limit module with a valid fixture |
| error_contract_test.rs | 4 | pass | KEEP | Errors on invalid/empty/malformed input; FFI error serialization |
| malformed_xml_test.rs | 13 | 4 pass / 9 fail | REWRITE + KEEP-IGNORED | Fixtures qualify local elements (`<ern:PartyId>`), which ERN's unqualified schemas do not allow. Fixtures rewritten. The tests that expect `MismatchedTags`/`UnexpectedClosingTag`/`InvalidAttribute`/`MalformedXml` get a generic `XmlError`, because `xml_validator` is not on the parse path → KEEP-IGNORED (error-type mismatch) |
| utf8_handling_test.rs | 10 | 1 pass / 9 fail | REWRITE | Same invalid prefixed fixtures plus invented elements (`<Artist>`, `<Title>`). Rewritten as valid 4.3 messages that still exercise UTF-8 |
| vendor_quirks.rs | 3 | 0 pass / 2 fail / 1 ign | REWRITE + DELETE (A) | The two quirk tests also omit required MessageSender/Recipient; the fixtures now omit only the quirk. `test_all_vendor_quirks` reads a fixture dir that doesn't exist → deleted |
| version_detection.rs | 4 | pass | KEEP | Namespace-based version detection |
| version_parsing.rs | 3 | 1 pass / 2 fail | REWRITE | Fixtures from the gitignored test-suite/ lack the required `ReleaseType` (4.2/4.3). Fixtures moved to tests/fixtures/ and made valid |
| model_consistency.rs | 3 | does not compile | REWRITE + DELETE (A) | graph/flat consistency and serde round trip kept (folded into version_parsing). `test_ffi_error_conversion`: `From<ParseError> for FFIError` doesn't exist → deleted |
| integration_test.rs | 1 | fail | DELETE (P) | Duplicate of version_parsing |
| version_migration.rs | 2 | fail | DELETE (P) | Duplicate of version_parsing (no migration is performed) |
| version_works.rs | 2 | 1 pass / 1 fail | DELETE (P) | Duplicate of version_detection and vendor_quirks |
| version_summary.rs | 2 | pass | DELETE (P) | Milestone printout, no assertions |
| phase2_complete.rs | 2 | fail | DELETE (P) | Milestone test; duplicate of version_parsing |
| test_phase_2_1.rs | 3 | pass | KEEP (renamed parse_options) + DELETE (P) | ParseOptions presets kept; `test_to_build_request_exists` is `assert!(true)` → deleted |
| playground_regression_test.rs | 5 | does not compile | REWRITE | Playground samples are real regressions; ported to `flat.*` / `Option` sender names |
| type_conversion_test.rs | 9 | does not compile | DELETE (A) | Builds struct literals and asserts the fields it just set; ErrorLocation API gone |
| parser_builder_integration.rs | 3 | 2 pass / 1 fail | REWRITE + DELETE (A) | Its "round trip" used a hand-written quick-xml matcher and a string-template builder. Rewritten in ddex-builder's tests as real parse → build → re-parse. Large-file/selective tests exercised only test helpers → deleted |
| comprehensive_test.rs | 1 | fail | DEFER | Only exercises `streaming::comprehensive` |
| streaming_integration_test.rs | 2 | pass | DEFER | Only exercises `streaming::minimal` |
| test_fast_streaming_connection.rs | 3 | fail | KEEP-IGNORED + PERF | `parse_fast_streaming` (reachable via `SecurityConfig::relaxed()`/`enable_fast_streaming`) returns no releases. basic/explicit → known bug; vs-normal timing → PERF (and known bug) |
| fast_streaming_test.rs | 1 | pass | DELETE (P) | Timing printout, no assertions |
| streaming_performance_tests.rs | 7 | pass | DELETE (P) | Timing printouts, no assertions |
| run_performance_benchmarks.rs | 1 | pass | DELETE (P) | Benchmark printout, no assertions |
| simple_performance_validation.rs | 4 | 3 pass / 1 fail | DELETE (P) | Writes report/certificate files |
| validate_performance_achievement.rs | 5 | 4 pass / 1 fail | DELETE (P) | Writes PERFORMANCE_CERTIFICATE/REPORT; v0.4.0 milestone |
| final_v0_4_0_validation.rs | 1 | pass | DELETE (P) | Writes RELEASE_READY_CERTIFICATE; milestone, no assertions |
| v0_4_0_final_tests.rs | 1 | does not compile | DELETE (P) | Milestone suite |
| v0_4_0_production.rs | 5 | 4 pass / 1 fail | PERF + DELETE (P) | Throughput/memory targets → perf module. The SIMD "correctness" tests only assert `is_ok()` on the fast-streaming path and print claims they don't check → deleted (milestone) |
| improved_failing_tests.rs + test_categories.rs | 2 (+helpers) | 5 pass / 1 fail | DELETE (P) | v0.4.0 "failing tests" scaffolding that tests its own category system |
| working_test_improvements.rs | 5 | pass | DELETE (P) | Same milestone scaffolding |
| tests/integration/npm-package.test.ts | – | not run by anything | DELETE (P) | Duplicate of the Node binding smoke tests; no runner |

## ddex-parser: inline unit tests (src/)

`src/main.rs` re-declares the library modules with `mod`, so the bin target compiles
and runs every inline test a second time (~97 s). Bin-target tests disabled
(`test = false`): **DELETE (P), duplicate**.

| Module | Tests | Status | Category | Reason |
|---|---|---|---|---|
| lib.rs tests, api_integration_test (public `DDEXParser` streaming API) | 4 | pass | KEEP | Public API |
| parser::{attribute_extractor, extension_capture, multi_release_parser, namespace_detector, selective_parser, tests, xml_validator, xpath_selector} | 45 | pass | KEEP | Unit tests of live code |
| utf8_utils | 3 | pass | KEEP | |
| streaming::fast_streaming_parser | 4 | pass | KEEP (3) + PERF (`test_performance_target`) | Used by `parse_fast_streaming` |
| streaming::{aligned_comprehensive, comprehensive, comprehensive_tests, consistency_test, debug_test, fast_zero_copy, fixed_comprehensive, iterator, minimal, verification, working_impl, zero_copy_parser} | 34 | 2 fail (comprehensive, fixed_comprehensive technical details) | DEFER | Legacy streaming implementations |
| streaming::parallel_parser | 4 | pass (flaky) | DEFER (2) + PERF (`test_parallel_speedup_measurement`, `test_thread_scaling`) | |
| streaming::{perf_analysis, zero_copy_benchmark, parallel_benchmark} | 10 | 1–2 fail (flaky) | PERF | Incl. `test_specific_parallel_speedup` |
| streaming/simple_test.rs | 2 | not compiled (not in module tree) | DELETE (A) | Orphaned test file |
| streaming/accumulator.rs tests | 4 | not compiled (module not declared) | out of scope | Dead library source, not a test file. Noted for the v0.5.0 streaming cleanup |

## ddex-builder: integration tests

| File | Tests | Status | Category | Reason |
|---|---|---|---|---|
| golden_files.rs | 5 | 4 pass / 1 fail | KEEP + KEEP-IGNORED | `test_audio_album_golden`: the stable-hash MessageId mixes in `Utc::now()`'s date (src/builder.rs ~L683), so the snapshot goes stale every day |
| determinism_tests.rs | 16 | pass | KEEP | |
| cross_platform_determinism.rs | 9 | pass | KEEP | |
| linker_test.rs, linker_xml_integration_test.rs, xml_generation_test.rs | 12 | pass | KEEP | |
| presets_test.rs | 16 | pass | KEEP | |
| target_version.rs | 3 | pass | KEEP | |
| entity_classification_test.rs | 13 | pass | KEEP | |
| error_sanitizer_test.rs | 4 | pass | KEEP | |
| security_integration_test.rs | 10 | pass | KEEP | |
| security_fixes_integration.rs | 10 | 4 pass / 6 fail | KEEP + KEEP-IGNORED + PERF | The path validator lets `settings.json`, empty paths, Unicode homographs and malformed paths through, and rejects valid paths under concurrency (×5) → known bug; overhead benchmark → PERF |
| canonical_debug.rs | 1 | pass | KEEP | Small canonicalization smoke test (has an assertion) |
| basic_test.rs | 3 | pass | KEEP (1) + DELETE | `test_builder_creation` is `assert!(true)` (P); `test_no_hashmap_usage` tests IndexMap itself (A) |
| comprehensive_test_suite.rs | 6 | 4 pass / 2 ign | DELETE (P) | Describes itself as placeholder tests; each sample duplicates a dedicated suite |
| performance_regression.rs | 10 | 5 pass / 4 fail / 1 ign | PERF | ms targets |
| tests/security/*.rs | 83 | never compiled (not included anywhere) | REWRITE | Revived: compiled against the current API; real failures → KEEP-IGNORED |
| tests/integration/*.rs | 42 | never compiled; hundreds of errors | DELETE (A) | Written against removed APIs |
| tests/determinism/mod.rs | 5 | never compiled; submodules missing | DELETE (A) | |
| tests/stress/mod.rs | 5 | never compiled | DELETE (A) | |
| src/messages/update_tests.rs | 18 | not compiled (not in module tree) | DELETE (A) | Orphaned test file |

## ddex-builder: inline unit tests and doctests

| Module | Tests | Status | Category | Reason |
|---|---|---|---|---|
| lib inline (31 modules) | 175 | pass | KEEP | |
| src/{attribute_normalizer, extension_restoration, presets/custom_template}.rs tests | 16 | not compiled (modules not declared) | out of scope | Dead library source |
| doctests | 50 | 28 pass / 22 fail | REWRITE | Fixed against the current API, or marked `ignore`/`no_run` with a reason when purely illustrative |

## Node bindings (baseline: debug build, Node 22)

| File | Status | Category | Reason |
|---|---|---|---|
| ddex-parser/bindings/node test-community-prs.js, test-ern4-tracks.js (`npm test`) | pass | KEEP | Real `node:assert` checks |
| ddex-parser/bindings/node test-manual.js (`npm test`) | pass, but would also pass if bad XML were accepted (prints ❌, exits 0) | REWRITE | Now asserts the rejection |
| ddex-parser/bindings/node `__tests__/{basic,parser}.test.ts` (jest) | not runnable: no jest; targets the unpublished src/parser.ts + test-suite/ | DELETE (A) | |
| ddex-parser/bindings/node test-fixed-parser, test-native-vs-published, test_complete_json, test_complex_data, test_data_exposure, test_data_structure, test_error_handling, test_error_propagation, test_indexmap_conversion, test_playground_xml, test_real_parser, test_simple_complete, test_streaming (.js) | not run by anything | DELETE (P) | Debug scripts: console output only, no assertions, never exit non-zero |
| ddex-parser/bindings/node test-build.js, test-linux-binary.js | manual | out of scope | Load/smoke tools for release checks (exit 1 on failure); not tests |
| ddex-builder/bindings/node test.js (`npm test`) | pass | KEEP (weak) | Prints "Invalid XML validation: { isValid: true }" without failing; that's the known `validateStructure` bug, deferred to v0.6.0 |
| ddex-builder/bindings/node test-exports.js, test-types.ts, test_complete_fix.js | not run | DELETE (P) | No assertions |
| ddex-builder/bindings/node test-build.js, test-linux-binary.js, integration_test_node.js, streaming_test.js | manual | out of scope | Smoke tools (exit 1 on failure) |

## Python bindings (baseline: venv, arm64 CPython 3.12.4, `maturin develop`)

| File | Tests | Status | Category | Reason |
|---|---|---|---|---|
| ddex-parser tests/test_full.py | – | collection ERROR (module-level script; aborts the whole run) | DELETE (P) | No test functions or assertions; prints only |
| ddex-parser tests/test_parser.py | 11 | 2 pass / 9 fail | REWRITE | SAMPLE_XML omits the required MessageSender/Recipient. `test_parse_result` references an undefined `ParseResult`. `detect_version` returns `V4_3` but the test expects `4.3` (checked after the rewrite) |
| ddex-parser tests/test_integration.py | 3 | pass | KEEP | |
| ddex-parser tests/validate_package.py, test-local.sh | – | not collected | out of scope | Packaging scripts |
| ddex-builder test_builder.py | 7 | pass | KEEP | |
| ddex-builder test_import.py | 4 | pass, but each test `return`s a bool, so failures can't fail | REWRITE | Use asserts |

Also seen: the Python parser reports `__version__ == "0.4.3"` while the package is
0.4.5. Version-consistency is v0.5.0 packaging work and not touched here.

## Outcomes after the fix-up commits

Things the triage couldn't predict, found while making the KEEP/REWRITE tests pass:

- **New KEEP-IGNORED (parser):** `StreamingParser::stream_releases` yields nothing
  for a full document (`utf8_handling::streaming_parser_handles_utf8`); `]]>` in
  text is accepted (`malformed_xml::test_cdata_terminator_in_text_is_rejected`).
- **REWRITE adjusted:** `version_parsing::graph_and_flat_models_agree` now compares
  `flat.version` with the enum spelling (`V4_3`), which the Node binding contract
  already asserts (`test-community-prs.js`).
- **Round trip (builder):** both new tests are KEEP-IGNORED. The 4.3 output is a
  3.8-style tree that the parser rejects (no ReleaseType) or reads without titles.
- **Revived security suite:** 83 tests compile. One no-assertion 93 s stress test
  was deleted (P); of the remaining 82, 59 pass and 23 are ignored (12 clear bugs,
  11 marked "policy decision needed"). One test bug fixed (case-sensitive keyword check).
- **Python:** after the sample fix, the remaining failures are binding bugs
  (`message_id` is a method; `detect_version` returns `V4_3`), now
  `xfail(strict=True)`.

## builder-ern4: schema-valid ERN 4.2/4.3 builder output

- **Round trip (builder):** both tests are KEEP again and run for 4.2 and 4.3
  (`it::round_trip`): the output validates against the official XSDs, the parser
  reads it back, and build → parse → build is byte-identical.
- **Un-ignored:** `security::builder_security_tests::test_illegal_xml_chars_never_reach_builder_output`
  (the builder now rejects XML-illegal characters, naming the field; the test asserts
  that).
- **Script-escaping assertions fixed:** `test_output_validation` and
  `test_output_sanitization_pipeline` build script-looking metadata through the real
  builder and assert it comes out as escaped text, never a raw element. Both stay
  KEEP-IGNORED for their other assertions (OutputSanitizer bugs). New non-ignored
  `script_metadata_is_escaped_in_builder_output` covers the escaping.
- **New:** `ern4_xsd` (xmllint against the official XSDs; fails, not skips, without
  xmllint unless `DDEX_SKIP_XSD=1`), `ern4_samples` (19 official samples: parse → build
  → XSD → re-parse), `ern4_escaping` (proptest), `ern382_output` (3.8.2 snapshots),
  parser `ern4_build_inputs`; Node `smoke-ern43.js` (`npm run test:smoke`), Python
  `test_smoke_ern43.py`.
- **Rewritten expectations:** canonicalizer unit tests that pinned whitespace collapsing
  and the old (schema-invalid) 4.3 reorder now pin exact text and 4.x document order.
  Tests that build 4.3 for other purposes (determinism, linker, security helpers) get
  explicit 4.x test data (`ern4_support::ern4_test_data`); their dangling deal
  references, non-pattern references and 100-release message were fixed in the data.

## builder-ern382: schema-valid ERN 3.8.2 builder output with deals

- **Rewritten:** `ern382_output` snapshots now pin the schema-valid 3.8.2 document
  (and are XSD-checked by `ern38_xsd`); the request gained `use_types` on its deal
  (3.8.2 `Usage/UseType` is required). `target_version` gives its 3.8.1 request party
  IDs (required by the 3.8 XSD). The canonicalizer unit test that pinned the 3.8.2
  `MessageHeader` reorder now pins document order (the old order contradicted the XSD).
- **New:** `ern38_xsd` (xmllint against the official 3.8.2 XSD, canonical and pretty;
  3.8.1 = 3.8.2 document in the 381 namespace; named errors for missing values;
  escaping and illegal characters), `ern38_round_trip` (YouTube 3.8.2 samples,
  repository 3.8.x fixtures and builder-made messages: parse → build → XSD → re-parse;
  official 4.x samples rebuilt as 3.8.2). Expected build failures are pinned with the
  error they must name (parser gaps, no 3.8.x video, fixtures without required data).
  Node `smoke-ern382.js` (part of `npm run test:smoke`), Python `test_smoke_ern382.py`.
- **Unchanged:** the 4.x tests (`ern4_xsd`, `ern4_samples`, `ern4_escaping`,
  `round_trip`); `ern4_support` gained the 3.8.2 schema in its tooling check and a few
  helpers made public for the 3.8 round trip.

## parser-completeness: 0.5.0 parser fields, whitespace, large files

- **New (ddex-parser `tests/it`):** `whitespace` (4.3 and 3.8.2: text exact per the XSD
  whiteSpace facet, references and typed values collapsed, CRLF normalized, `&#13;` kept,
  empty vs absent), `large_file` (an 11+ MB 4.3 message through `parse()` yields the new
  fields; `Auto` resolves to `Dom`), `ern38_details` (3.8.2 details blocks, ReleaseId
  ProprietaryId order, ResourceGroup, track artists), `serialization` (the shape the WASM
  binding returns, null / [] for absent values). Fixtures `ern43_details.xml` and
  `ern382_details.xml` (XSD-valid) carry every new field.
- **New (ddex-builder `tests/it`):** `whitespace_round_trip` (build -> parse keeps
  whitespace in titles for 4.2 / 4.3 / 3.8.2; an empty title is the builder's
  required-field error, see `KNOWN_ISSUES.md`).
- **Changed:** `ern4_escaping`'s property test no longer trims its generated text (the
  parser keeps surrounding whitespace). `ern38_round_trip`: the builder's ProprietaryId
  multi-track album now round-trips and `ern382_details.xml` joins the sources; the
  YouTube audio samples stay pinned, now on the builder's missing release ISRC (the
  parser reads it); the 3.8.2 rebuild of the official DjMix samples is pinned on the
  builder's empty-MessageId check (the parser no longer invents `MSG_<version>`).
- **Bindings:** Node `test-parser-fields.js` (in `npm test`), Python
  `tests/test_parser_fields.py` (deals() / message_control_type() and the new columns);
  the builder bindings' `smoke-ern382.js` / `test_smoke_ern382.py` check deals and labels
  through the parser bindings.
- **Scripts (lxml, not part of `cargo test`):** `compare_ern4_oracle.py` gained a second
  table for the new fields (all 19 official 4.x samples match on every column);
  `ern38_population.py` checks that each new field is populated exactly where the 3.8.x
  XML has it (repository fixtures and the YouTube 3.8.2 samples: no misses, nothing
  invented).
- **perf-tests (0.5.0):** the wall-clock pass/fail tests became criterion benches
  (`ddex-parser` `throughput`, `ddex-builder` `performance` / `security`); the
  `perf-tests` feature keeps five sanity limits (`perf::`). The builder benches used an
  incomplete ERN 4.3 request, so they timed a validation error; they now build for real
  and unwrap the result. See `dev-docs/PERFORMANCE.md`.

The full list of known bugs lives in `KNOWN_ISSUES.md`.
