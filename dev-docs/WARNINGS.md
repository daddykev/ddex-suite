# rustc warnings: baseline and classification (zero-warnings branch)

Check command (run from the workspace root):

```bash
cargo build --workspace --all-targets --features perf-tests,legacy-streaming-tests 2>&1 | grep -c "^warning"
```

The count includes cargo's per-target summary lines (`warning: ... generated N warnings`),
so it is a little higher than the number of distinct diagnostics. ddex-builder has no
`legacy-streaming-tests` feature; cargo applies each feature to the packages that define it.

## Prerequisite: the check build did not compile on `main`

On `main` (0361280) the check command stopped with errors before reaching most targets,
so there was no complete baseline. Fixed first (commit "build: make `cargo build
--workspace --all-targets` compile"):

| Target | Problem | Fix |
|---|---|---|
| ddex-parser bench `fast_streaming_benchmark` | E0505: iterator moved while `stats()` borrowed | clone the stats; declare the bench `harness = false` |
| ddex-builder benches `building`, `performance`, `determinism` | written against removed request types/fields | rewritten on the current API via `benches/common/mod.rs` |
| ddex-builder bench `comprehensive_benchmarks` | fake: placeholder "builds" that never call the builder | deleted |
| ddex-builder bench `profiling` | duplicate of the 12-track `performance` case; broken pprof wiring | deleted, with the `pprof` dev-dependency |
| `ddex-builder-python`, `ddex-parser-python` cdylibs | fail to link from the workspace root on macOS (the `-undefined dynamic_lookup` flags live in per-package `.cargo/config.toml`, which cargo only reads inside that directory) | `build.rs` calling `pyo3_build_config::add_extension_module_link_args()` |

## Counts

| Stage | Check-command count | Distinct diagnostics |
|---|---|---|
| Baseline (after the compile fixes above) | **395** | 377 |
| After Step 2 (CLI imports the library crate) | **105** | 88 |

### Baseline, distinct diagnostics per crate and lint

| Crate | dead_code | unused_imports | unused_variables | unused_mut | unused_assignments |
|---|---|---|---|---|---|
| ddex-parser | 296 | 14 | 6 | 11 | 1 |
| ddex-builder | 10 | 8 | 29 | – | – |
| ddex-builder-node | 1 | – | – | – | – |
| ddex-parser-python | 1 | – | – | – | – |

By target: ddex-parser bin 295, ddex-parser lib 28, ddex-builder `it` tests 38,
ddex-builder examples 13, the two binding cdylibs 1 each, ddex-builder bin 1.

### Step 2: false positives from the parser CLI

`packages/ddex-parser/src/main.rs` declared `mod error; mod parser; mod streaming; mod
transform; mod utf8_utils;`, compiling a second private copy of the library into the
binary. `cli.rs` already uses everything through `ddex_parser::`, so the whole copy was
unused, and rustc reported live library code (`transform/ern4.rs`, `Ern4Index`,
`TrackWalker`, `GraphBuilder`, `Flattener`, `flatten_releases_ern4`, `VersionDetector`,
`ParseOptions`, `parse_streaming`, …) as dead: about 290 warnings. main.rs now declares only
`mod cli;`. The ddex-builder CLI already imported `ddex_builder`. 395 → 105.

## Classification of the 88 remaining diagnostics

Categories: DEAD-PRIVATE (delete), DEAD-PUBLIC (public but unused; pre-approved or asks
first), BUG (an unused value means a feature silently does nothing), FAKE-TEST,
EXAMPLE, TRIVIAL.

### ddex-parser: streaming implementations

How the public API reaches `src/streaming/`:

| Module | Reached by | Verdict |
|---|---|---|
| `working_impl` | `DDEXParser::stream`, `stream_with_version_detection`; element/stats types used by `fast_zero_copy`, `parallel_parser` | **KEEP** |
| `fast_streaming_parser` | `DDEXParser::parse` / `parse_with_options` when `SecurityConfig::enable_fast_streaming` (`SecurityConfig::relaxed()`) | **KEEP** |
| `parallel_parser` | `DDEXParser::stream_parallel*` | **KEEP** |
| `fast_zero_copy` | `parallel_parser` (`FastZeroCopyParser`), `DDEXParser::stream_zero_copy*` (feature `zero-copy`) | **KEEP** |
| `zero_copy_parser` | nothing (re-export only) | DEAD-PUBLIC, pre-approved → delete |
| `fixed_comprehensive` | nothing | DEAD-PUBLIC, pre-approved → delete |
| `minimal` | nothing (legacy tests only) | DEAD-PUBLIC, pre-approved → delete |
| `parser` (`StreamingDDEXParser`) | only `iterator` | DEAD-PUBLIC, pre-approved → delete |
| `iterator` (`DDEXStreamIterator`) | nothing | DEAD-PUBLIC, pre-approved → delete |
| `state` (`ParserState`, `ParsingContext`) | only `parser` | DEAD-PUBLIC, pre-approved → delete |
| `verification` | nothing (uses `comprehensive`) | DEAD-PUBLIC, pre-approved → delete |
| `comprehensive` | only `verification` and legacy tests | DEAD-PUBLIC, **not pre-approved: ask** |
| `aligned_comprehensive` | nothing | DEAD-PUBLIC, **not pre-approved: ask** |
| `element` (`ParsedElement`, `HeaderBuilder`) | only `parser` (deleted) and the `ParsedElement` re-export | DEAD-PUBLIC once `parser` is gone (no warning today), **ask** |
| `accumulator.rs`, `parser.rs.tmp` | not in the module tree (never compiled) | orphan files → delete |

Large-file mode (`parser::mode` → `parser::stream::parse_streaming`) does not use
`src/streaming/`. The bindings call only `parse`, `parse_with_options`, `detect_version`
and `sanity_check`.

The `legacy-streaming-tests` feature also gates tests for the **kept** implementations
(`working_impl`, `fast_zero_copy`, `parallel_parser` and the `comprehensive_tests`,
`debug_test`, `consistency_test` modules, which exercise `working_impl`/`parallel_parser`).
They all pass. When the feature is removed they become ordinary tests, or stay behind
`perf-tests` where they assert throughput. Tests of the deleted modules are deleted
with them (2 of them were failing: `comprehensive::test_comprehensive_streaming_parser`,
`fixed_comprehensive::test_resource_with_technical_details`).

| Location | Lint | Category | Action |
|---|---|---|---|
| streaming/zero_copy_parser.rs:25, 72, 699, 725, 893 | dead_code | DEAD-PUBLIC (pre-approved) | delete module |
| streaming/fixed_comprehensive.rs:27 | dead_code | DEAD-PUBLIC (pre-approved) | delete module |
| streaming/minimal.rs:129 | dead_code | DEAD-PUBLIC (pre-approved) | delete module |
| streaming/parser.rs:136 (9 handler methods) | dead_code | DEAD-PUBLIC (pre-approved) | delete module |
| streaming/comprehensive.rs:9, 33, 303 | unused_imports, dead_code | DEAD-PUBLIC (ask) | pending approval |
| streaming/aligned_comprehensive.rs:12, 37, 405, 590 | unused_imports, dead_code, unused_mut | DEAD-PUBLIC (ask) | pending approval |
| streaming/parallel_parser.rs:84 `find_element_boundaries`, `find_pattern` | dead_code | DEAD-PRIVATE (private; used only by their own legacy test) | delete with that test |
| streaming/parallel_parser.rs:185 field `reader` | dead_code | TRIVIAL | the reader is drained into `buffer` in the constructor and never touched again; replace with `PhantomData<R>` (public type unchanged) |
| streaming/parallel_parser.rs:505, fast_zero_copy.rs:430 | unused_mut (legacy tests) | TRIVIAL | remove `mut` |
| streaming/fast_streaming_parser.rs:14 field `config` | dead_code | **BUG**: `FastStreamingParser::new(StreamingConfig)` ignores the whole config (buffer size, `max_memory`, security limits) | ask |
| streaming/fast_streaming_parser.rs:479 `elapsed` | unused_variables (test) | TRIVIAL | only printed under `performance-debug`; gate it the same way |
| streaming/perf_analysis.rs:4, 126, 267, 286; parallel_benchmark.rs:206, 235, 419; zero_copy_benchmark.rs:9 | unused_imports/mut/variables (perf-tests modules) | TRIVIAL | remove |

### ddex-parser: other library code (all live)

| Location | Lint | Category | Action |
|---|---|---|---|
| parser/stream.rs:364 `location` | unused_variables | TRIVIAL | remove the binding; the EOF error message is unchanged |
| parser/xml_validator.rs:158 `depth` | unused_variables | TRIVIAL | `_` pattern (the depth is recomputed from the stack) |
| transform/graph.rs:329 `in_message_header = false` before `break` | unused_assignments | TRIVIAL | remove the dead store |
| api_integration_test.rs:6 `ERNVersion` | unused_imports | TRIVIAL | remove |
| cli.rs:1042 `count_elements` | dead_code | DEAD-PRIVATE | delete (`count_elements_flat` is the one in use) |
| tests/it/fast_streaming.rs:143 `elapsed` | unused_variables | TRIVIAL | remove |
| tests/it/version_detection.rs:7, 15, 24, 33 | unused_mut | TRIVIAL | remove `mut` |

### Bindings

| Location | Lint | Category | Action |
|---|---|---|---|
| ddex-builder/bindings/node/src/lib.rs:689 `generate_placeholder_xml` | dead_code | DEAD-PRIVATE (not `#[napi]`, no callers) | delete |
| ddex-parser/bindings/python/src/lib.rs:671 `StreamIterator::new` | dead_code | DEAD-PRIVATE (not in `#[pymethods]`, no callers; `from_parsed_result` is used) | delete |

### ddex-builder CLI

| Location | Lint | Category | Action |
|---|---|---|---|
| src/bin/ddex-builder.rs:1703 `BatchTask.{output_file, preset, version, validate}` | dead_code | **BUG**: `ddex-builder batch` is a no-op. `process_batch_task` is a TODO that returns `Ok(())`, so every task is reported as successful and nothing is written | ask (recommendation: wire it up) |

### ddex-builder examples

| Example | Warnings | Category | Action |
|---|---|---|---|
| `youtube_video_example` | 3 unused fns | EXAMPLE (fake) | **delete**. The builder has no video-resource support, so the example cannot produce the `VideoResource`/`VideoCodec`/… it checks for, and it exits with an error at runtime |
| `diff_comparison_example` | unused config fields, unused variant | EXAMPLE (fake) | **delete**. It defines its own mock `DiffEngine` and never calls the library. A rewrite on the real API is not possible: `DDEXBuilder::diff_xml` is a stub (`parse_xml_to_ast` wraps the whole document in a single text node), so it reports any change as one `/#text` modification. Recorded in KNOWN_ISSUES.md |
| `streaming_catalog_example` | 8 (unused builder, fields, imports) | EXAMPLE (fake) | **rewrite**. It formatted XML strings by hand and never used the library. Now uses `ddex_builder::streaming::StreamingBuilder` (resources → releases, progress callback, stats) |
| examples/README.md | – | – | lists only examples that exist (it also referenced a nonexistent `generic_album_example`) |

### ddex-builder security tests (FAKE-TEST)

`tests/it/security/*.rs`: 31 diagnostics, mostly `let result = validator...` with no
assertion, plus unused `builder`/`config` variables and unused imports.

| File | Locations |
|---|---|
| builder_security_tests.rs | 74, 89, 111, 280, 296 |
| integration_tests.rs | 8, 12, 14, 16 (imports), 476 |
| malicious_payload_tests.rs | 38, 56, 71, 208, 253, 298, 313, 336, 350, 363, 389–409, 459, 475, 529, 549, 556 |
| path_validator_tests.rs | 7 (import) |
| xxe_prevention_tests.rs | 657 |

Each one gets a real assertion under the security policy, or is deleted if it has no
meaningful expectation. A correct assertion that fails becomes `#[ignore = "known bug:
…"]` plus a KNOWN_ISSUES.md row.

Security policy: the libraries defend against XML-level threats (XXE, entity expansion,
DTD handling, depth/size limits, CDATA handling, correct output escaping). They do not
flag or alter SQL/HTML/JS-looking text in metadata; such text passes through unchanged
and correctly XML-escaped. Tests asserting the opposite are inverted. The per-test outcomes
are in the commit that fixes them and in the final report.

## DEAD-PUBLIC items that produce no warning (not touched)

You named these as DEAD-PUBLIC candidates. They were reported only by the duplicate copy
in the parser CLI. As public library items they produce no warning, so the zero-warnings
goal does not require removing them. Listed for a separate decision:

| Item | Used by |
|---|---|
| `transform::version_adapter::{VersionAdapter, VersionMigrator}` | `parser::version_aware` uses `VersionAdapter`; `VersionMigrator` has no users |
| `transform::resolve::ReferenceResolver` | `parser::dom::resolve_references` (itself unused by the parse path) |
| `utf8_utils::{validate_utf8, validate_utf8_string}` | no users |
