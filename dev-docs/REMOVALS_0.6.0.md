# Removals for 0.6.0: fake, no-op and unreachable ddex-builder APIs (internal)

Inventory for 0.6.0 PR 3. Covers every public item of `ddex-builder` (the `lib.rs` `Builder`
facade and its option types, `builder.rs`, every `src/` module, the CLI) and the builder
bindings (Node, Python, WASM), on `main` after PR 2 (`d89fdff`). This document is the
inventory. The full removal touches about 100 files, so the work is split (see
[Proposed split](#proposed-split)).

**Status:** PR 3c is done: the dead modules, the entity classifier and the error
sanitizer, the fakes and no-ops in streaming, linker, security, preflight and
`canonical::rules`, and the `DB_C14N` config argument. Those rows are marked "removed in 3c";
rows that proved wrong are marked "row was wrong, kept". The streaming-reference and
`id_generator` rows are left for PR 6.

PR 3b is done too: the guarantees, the determinism verifier, verification, the
fidelity module and the other fidelity / canonicalization / round-trip fakes, and the twelve
unread `DeterminismConfig` fields; those rows are marked "removed in 3b". The `DB_C14N`
config argument (NO-OP) is left for 3c.

PR 3a is done. It removed the [Decided removals](#decided-removals) and the
no-op CLI flags `--config`, `build --profile`, `build --strict`, `schema --profile` and
`schema --with-docs`; those rows are marked "removed in 3a". The rest is for 3b–3d. Line
numbers are as of `d89fdff`, before 3a.

Method: I read the code and grepped every field, method and type for its readers and callers
across `src/`, `tests/`, `benches/`, `examples/`, `fuzz/`, the three binding crates and their
JS/Python wrappers, the READMEs and `website/`. I spot-checked the main claims by grep: the
`DeterminismConfig` field reads, the `BuildVerifier` round-trip check and the streaming
`deals_written` counter. I did not build or run anything.

Line numbers are as of `d89fdff`. Paths are relative to `packages/ddex-builder/` unless they
start with `packages/`, `website/` or a top-level file.

## Classes

- **FAKE**: returns "not implemented", or returns results that are not computed from real
  inputs (hard-coded values, "not verified", placeholder scores, success that was not checked).
- **NO-OP**: configuration or state that is stored but never read by a build or by any other
  real computation.
- **DEAD**: not compiled, or not reachable from any public entry point (private and unused,
  never constructed, or unreachable in practice).
- **REAL**: does what its docs say, from real inputs. Some REAL items are buggy; the bugs are
  listed under [Found along the way](#found-along-the-way-not-removals) and are not removals.
- **UNSURE**: cannot tell from reading. The list is in [UNSURE](#unsure).

"Affected" names who loses something if the item goes: Rust users of the published crate
(**Rust**), the **Node**, **Python** or **WASM** binding, the **CLI**, and the tests, benches,
examples or docs that use the item.

## Counts

Counted per row of the [Inventory](#inventory) tables, the bindings table excluded. A row is
one item, or a group of items with the same class and evidence. Where a row has a mixed
class (e.g. "DEAD + FAKE", or "REAL (build) / FAKE (verification)"), it is counted under the
first class named.

| class | rows |
|---|---|
| FAKE | 56 |
| NO-OP | 33 |
| DEAD | 48 |
| REAL | 82 |
| UNSURE | 1 |
| **total** | **220** |

Not counted:
- 9 rows marked "decided", "duplicate" or "follows the module". They go with their
  module or with a decided removal.
- The [Decided removals](#decided-removals): the update-message generator together with the
  diff module, and the presets.

The [UNSURE](#unsure) list has 4 entries. Only one of them (CLI `DdexVersionArg`) is an
inventory row; the others are noted next to REAL rows or are binding-only.

## Entry points

- `DDEXBuilder::build` (`builder.rs:721`) reads only `request`, `options.preflight_level`,
  `options.id_strategy`, `options.stable_hash_config` and `options.determinism`. Of
  `DeterminismConfig` it reads 4 of 16 fields: `canon_mode`, `indent_char`, `indent_width`
  and `emit_reproducibility_banner`. It never touches `fidelity`, `verification`,
  `round_trip`, `guarantees`, `linker`, `caching`, `memory_optimization`,
  `parallel_processing`, `optimized_strings`, `namespace_minimizer`, `versions`, `schema`,
  `security` or `api_security`.
- The CLI, Node, Python and WASM always pass `BuildOptions::default()`.
- `DDEXBuilder` holds `_inner: Builder` (`builder.rs:709`), which is built and never read.
- The **`lib.rs` `Builder`** never builds with its own configuration:
  - `build_with_fidelity` (`lib.rs:775`) and `build_internal` (`:968`) both use
    `BuildOptions::default()`.
  - So `Builder.config`, which is set by `with_config` and by `apply_preset`, is never applied.
- **Bindings:**
  - Node and Python call `DDEXBuilder::build`, `round_trip::verify_round_trip` and the presets.
  - Node also calls `streaming::StreamingBuilder`.
  - WASM calls `DDEXBuilder::build`, and its `diff_viewer` uses the diff module.
  - None of the bindings uses `Builder`, `fidelity`, `verification`, `versions`, `schema` or
    `guarantees`.

## Decided removals

These are decided (PR 3 prompt), so no classification is needed. The notes say what each one
actually does.

### Update messages and diff (removed in 3a)

| item | path | note |
|---|---|---|
| `DDEXBuilder::create_update`, `create_update_with_config`, `apply_update`, `create_update_from_request`, `validate_update`, `serialize_update` and the private serializers (`serialize_update_message_to_xml`, `_header`, `_metadata`, `_list`, `_resource_updates`, `_release_updates`, `_deal_updates`, `escape_xml`) | `builder.rs:1165-1584` | `parse_xml_to_ast` (`:1143`) wraps the whole document in one text node, so every update is a single `/#text` change. `apply_update` is not implemented. `entity_id` is random (D11); timestamps are now (D9, D10) |
| `DDEXBuilder::diff_xml`, `diff_xml_with_config`, `diff_request_with_xml`, private `parse_xml_to_ast` | `builder.rs:1101-1163` | stub parser (`TODO: Implement proper XML parsing`) |
| `messages/` (`mod.rs`, `update_release.rs`; `test_data.rs` is not compiled): `UpdateGenerator`, `UpdateReleaseMessage`, `UpdateConfig`, `UpdateMetadata`, `UpdateOperation`, `UpdateAction`, `ValidationStatus` and the rest | `src/messages/*` | `messages/mod.rs` holds nothing else |
| `diff/` (`mod.rs`, `types.rs`, `formatter.rs`, `diff_tests.rs`, `test_data.rs`): `DiffEngine`, `DiffConfig`, `VersionCompatibility`, `ChangeSet`, `ChangeType`, `DiffPath`, `ImpactLevel`, `SemanticChange`, `DiffFormatter` and the rest | `src/diff/*` | D15, D16 |
| root re-exports | `lib.rs:11,19,37-39,44-46` | — |
| WASM `DdexDiffViewer`, `DiffViewerConfig` | `bindings/wasm/src/diff_viewer.rs` (whole file), `lib.rs:9` | its only use of the diff module; its README paragraph goes too |
| CLI `diff` | `bin/ddex-builder.rs:57-58,145-168` (`DiffCommand`), `:469-474` (`DiffFormat`), `:542`, `:709-714` | already returns "not implemented" |
| test (e) `update_message_is_byte_identical` and helper `update_message_xml` | `tests/it/determinism_0_6.rs:253-288` | ignored known bug (D10, D11) |
| test `apply_update_is_not_implemented` | `tests/it/honest_results.rs:264-270` | — |

### Presets (removed in 3a)

| item | path | note |
|---|---|---|
| `presets/` (`mod.rs`, `generic.rs`, `youtube.rs`; `custom_template.rs` is not compiled): `PartnerPreset`, `PresetConfig`, `all_presets`, the 7 presets and the rest | `src/presets/*` | no preset changes any output (`PartnerPreset.determinism` goes to `Builder.config`, which is never applied) |
| **kept, moved in 3a**: `DdexVersion` (with `Display`) and `MessageProfile` | `presets/mod.rs:109-152` | used by kept code (`versions`, `schema`, the CLI, `Builder::with_version`). Now `versions::DdexVersion` (still `ddex_builder::DdexVersion` at the root) and `schema::MessageProfile`. If 3d removes `versions` / `schema`, they move again |
| `Builder::apply_preset`, `preset`, `available_presets`, `get_preset`, `is_preset_locked`; fields `presets`, `locked_preset`; `load_default_presets` | `lib.rs:399-400,525-608,750-752` | — |
| root re-export `PartnerPreset` | `lib.rs:49` | — |
| `SchemaGenerator.preset` field and `with_preset` | `schema/mod.rs:168-170,348-360` | the field is `#[allow(dead_code)]`, never read |
| doctests on `Builder` and `Builder::new` that use presets | `lib.rs:323-356,424-430,543-556` | rustdoc |
| CLI `preset list/show/apply`, `--preset` on `build`, `validate` and `watch`, `PresetChoice`, `preset_to_string`, `get_available_presets` (a hard-coded list), `get_preset_details` (hard-coded "spotify_audio_43" with a placeholder config), `PresetInfo` / `PresetDetails` / `PresetSummary` / `PresetConfig` | `bin/ddex-builder.rs:65-66,93-95,183-185,254-324,348-350,444-460,497-509,546,608-645,912-1024,1120-1130,1161-1255,1358-1390` | `validate` and `watch` themselves stay (PR 4); only their `--preset` flag goes |
| Node `getAvailablePresets`, `getPresetInfo`, `applyPreset`, `getPresetValidationRules`; `PresetInfo`, `ValidationRule`; `core_preset` | `bindings/node/src/lib.rs:64-82,214-219,434-483`; `index.d.ts:51-64,168-171`; `js/index.ts:18-19,39-40,68-69,73-86,184-190,297-368,710-711` (including the `BuildOptions.preset` / `StreamingBuildOptions.preset` options, which call `applyPreset`) | test: `test-honest-results.js:94-102` |
| Python `get_available_presets`, `get_preset_info`, `apply_preset`, `get_preset_validation_rules`; `PresetInfo`, `ValidationRulePy`; `core_preset` | `bindings/python/src/lib.rs:237-319,709-758,1364-1371,1379-1380`; `python/ddex_builder/__init__.py:17,20,44,47` | tests: `test_honest_results.py:73-83`, `test_import.py:28`. There are no `.pyi` stubs |
| tests | `tests/it/presets.rs` (whole file), `tests/it/main.rs:24`, `lib.rs:1113-1132` (unit tests) | — |
| example / fuzz | `examples/spotify_album_example.rs:10,57-75,470`, `examples/README.md`; `fuzz/fuzz_targets/fuzz_builder_api.rs:12-36` | `fuzz/` is not built by CI and is already broken |

Dependencies: no ddex-builder dependency becomes unused because of the decided removals. In
`bindings/wasm/Cargo.toml`, `serde`, `html-escape` and `indexmap` become unused once
`diff_viewer.rs` goes.

## Inventory

### `lib.rs`: `Builder` facade and its option types

| item | class | evidence | affected |
|---|---|---|---|
| `Builder::new`, `Default` | REAL | constructor | Rust, CLI (`bin:490,548,876`), tests |
| `Builder.config`, `Builder::with_config`, `config()` | NO-OP (**removed in 3b**) | stored (`:398,449`) and returned (`:611`); `build_with_fidelity` / `build_internal` use `BuildOptions::default()` (`:775,969`) | Rust |
| `Builder.version_manager` | REAL (container) (**removed in 3d**) | read by `detect_version`, `is_version_compatible` | — |
| `with_version`, `target_version` | REAL (CLI only) | no `Builder` method reads it; the CLI reads it at `bin:1028` to set `request.version` | CLI `build --ddex-version`, `preset apply --version-override` |
| `detect_version` | REAL (heuristic, often wrong) (**reclassified FAKE** (see `versions`) and **removed in 3d**) | wraps `VersionManager::detect_version` (see `versions`) | tests: `honest_results.rs:220-227`; fuzz |
| `convert_version` | FAKE (**removed in 3d**) | wraps `VersionConverter::convert` (see `versions`); always `Ok` | CLI `convert`; doctest `lib.rs:358-385`; fuzz |
| `is_version_compatible` | REAL (hard-coded table) (**reclassified FAKE** and **removed in 3d**) | `VersionManager::is_conversion_supported` | doctest; fuzz |
| `supported_versions` | REAL | `versions::utils::supported_versions` (omits 3.8.1) | — |
| `FidelityOptions.enable_perfect_fidelity` | NO-OP (**removed in 3b**) | read only by the getter `is_perfect_fidelity_enabled` (`:698`) | Rust |
| `FidelityOptions.preserve_comments`, `preserve_processing_instructions`, `preserve_namespace_prefixes`, `enable_deterministic_ordering` | NO-OP (**removed in 3b**) | read only to echo into `db_c14n_config` (`:950-955`) | Rust |
| `FidelityOptions.preserve_extensions`, `custom_canonicalization_rules`, `verification_config` | NO-OP (**removed in 3b**) | never read (`Builder` uses its own `self.verification_config`) | Rust |
| `FidelityOptions.preserve_attribute_order` | NO-OP for builds (**removed in 3b**) | read only by `verification.rs:351` (gates a regex heuristic) | Rust |
| `FidelityOptions.canonicalization` | REAL (`None`, `DbC14N`) | read by `canonicalize_with` (`:1084`) and the guard in `build_with_fidelity` (`:765`) | tests: `honest_results.rs:120-168` |
| `FidelityOptions.collect_statistics`, `enable_verification` | REAL (`enable_verification` **removed in 3b**; `collect_statistics` kept) | `:786`, `:840` | tests: `honest_results.rs:142-143` |
| `CanonicalizationAlgorithm::None`, `DbC14N` | REAL | `canonicalize_with` | tests |
| `CanonicalizationAlgorithm::C14N`, `C14N11`, `Custom` | FAKE (**removed in 3b**) | always `Err(not_implemented)` (`:1093-1103`) | tests: `honest_results.rs:126-127,168` |
| `CustomCanonicalizationRules` (all fields), its `UnorderedMap` alias and the `HashMap` import | NO-OP (**removed in 3b**) | never read (PR 2 allows H4, `:66-78,165`) | Rust |
| `with_canonicalization`, `with_db_c14n` | REAL (setters) | set a field that is read | — |
| `with_c14n`, `with_c14n11`, `with_custom_canonicalization` | FAKE (**removed in 3b**) | select algorithms that always error | — |
| `with_perfect_fidelity`, `for_round_trip`, `with_fidelity_options`, `set_fidelity_options`, `enable_perfect_fidelity`, `disable_perfect_fidelity`, `fidelity_options`, `is_perfect_fidelity_enabled`, `canonicalization_algorithm` | NO-OP (apart from the real fields they set) (`with_perfect_fidelity`, `for_round_trip`, `enable_perfect_fidelity`, `disable_perfect_fidelity`, `is_perfect_fidelity_enabled` **removed in 3b**; the others kept) | they mostly set NO-OP fields; their only real effect is `enable_verification` / `collect_statistics` / `canonicalization` | tests: `honest_results.rs:114,147,167` (`with_fidelity_options`) |
| `with_verification`, `with_statistics` | REAL (setters) (`with_verification` **removed in 3b**; `with_statistics` kept) | set `enable_verification` / `collect_statistics` | — |
| `VerificationConfig` (lib copy) | duplicate (**removed in 3b**) | field-for-field copy of `verification::VerificationConfig`, converted at `:788-801,855-866` | goes with `verification` |
| `VerificationResult`, `VerificationIssue`, `VerificationSeverity` (lib copies) | duplicate (**removed in 3b**) | converted from the `verification` types (`:807-833,872-896`) | goes with `verification` |
| `Builder::build_with_fidelity` | REAL (build + statistics) / FAKE (verification branch) (verification branch **removed in 3b**; build + statistics kept) | builds with `DDEXBuilder::build`; its `verification_result` comes from `BuildVerifier::verify` (FAKE, see `verification`); `canonicalization_applied` / `db_c14n_version` are constants but accurate | tests: `honest_results.rs:140-172` |
| `FidelityBuildResult` | REAL (data) | — | — |
| `BuildStatistics` (lib): `build_time`, `verification_time`, `output_size_bytes`, `element_count`, `attribute_count`, `namespace_count`, `comment_count`, `processing_instruction_count` | REAL (`verification_time` **removed in 3b**; the others kept) | `count_xml` (`:1058`) | tests |
| `BuildStatistics` (lib): `canonicalization_time`, `validation_time`, `peak_memory_bytes`, `input_size_bytes` | FAKE (**removed in 3b**) | always 0 | — |
| `Builder::verify_build` | FAKE (**removed in 3b**) | `BuildVerifier::verify` | — |
| `Builder::test_round_trip_fidelity` | FAKE (**removed in 3b**) | always errors (`:899-909`) | tests: `honest_results.rs:85-88` |
| `Builder::test_round_trip_fidelity_with`, lib `RoundTripResult` | REAL | `RoundTripTester::test_round_trip_with` | tests: `honest_results.rs:96-99` |
| `Builder::canonicalize` | REAL (`None`, `DbC14N`) | `canonicalize_with`; the other algorithms are the FAKE variants above | tests: `honest_results.rs:107-135` |
| `Builder::db_c14n_config`, `DbC14NConfig` | FAKE (**removed in 3b**) | echoes options that are never applied | — |
| `NamespaceHandling` | FAKE / DEAD (**removed in 3b**) | only built by the echo; `Normalize` is never constructed | — |
| `DB_C14N_VERSION`, `version_info` | REAL | `builder.rs:803` | tests |
| `fidelity_engine_info` | REAL (**removed in 3b** (decided: "fidelity engine" claim)) | static, accurate text | tests: `honest_results.rs:179` |
| `pub(crate) build_internal` | REAL (**removed in 3b** (no caller left)) | used by `DeterminismVerifier` | — |
| root re-exports of `fidelity`, `round_trip::FidelityAnalysis`, `verification` | follow their items (**removed in 3b**) | `lib.rs:59-62` | — |

### `builder.rs`

| item | class | evidence | affected |
|---|---|---|---|
| `BuildRequest` and the request types, `ReleaseIdRequest::is_empty`, `DealTerms`, `parse_target_version` | REAL | build input | all |
| `DDEXBuilder::new`, `build`, `BuildOptions.preflight_level`, `id_strategy`, `stable_hash_config`, `determinism` | REAL | `:721-826` | all |
| `IdStrategy`, the ID generation and the `MessageId` / `MessageCreatedDateTime` fallbacks | REAL (non-deterministic) | **out of scope: PR 6** | — |
| `BuildResult.xml`, `warnings`, `statistics`, `canonical_hash`, `reproducibility_banner` | REAL | `:812-825` | all |
| `BuildResult.errors` | FAKE | always `Vec::new()` (`:815`); errors are returned as `Err` | Rust |
| `BuildStatistics` (builder) | REAL | `:816-822` | — |
| `DDEXBuilder._inner: Builder` | NO-OP | private; never read (`:709,716`) | — (before 3a it also loaded every preset each time `DDEXBuilder::new` ran) |
| private `preflight` (legacy) | DEAD (**removed in 3c**) | `#[allow(dead_code)]` (`:1070`); no callers | — |
| private `parse_duration_to_seconds` | REAL | used at `:1002` | — |
| update / diff methods | decided | see [Decided removals](#decided-removals) | — |

### `determinism.rs`

`DeterminismConfig` has 16 fields; `build` reads 4. The 12 others have no reader anywhere
outside their own `Default` impl. That was checked with `grep -w` over `src/` and the three
binding crates; the only other hits are a local variable in `canonical/rules.rs:168`, the
`unicode_normalization` crate, and the CLI's own `--verify-determinism` flag.

| item | class | evidence | affected |
|---|---|---|---|
| `DeterminismConfig.canon_mode`, `CanonMode::DbC14n`, `CanonMode::Pretty` | REAL | `builder.rs:789` (DbC14n vs. anything else) | tests: `ern4_xsd.rs`, `ern38_xsd.rs`, `ern382_output.rs`, `golden_files.rs`, `basic.rs` |
| `CanonMode::Compact` | NO-OP (**removed in 3b**) | same output as `Pretty`; nothing compacts | — |
| `.indent_char` / `IndentChar`, `.indent_width` | REAL (only when `canon_mode` ≠ `DbC14n`) | `xml_writer.rs:133,137`; DB-C14N re-indents with a fixed two spaces | tests: `basic.rs:12` |
| `.emit_reproducibility_banner` | REAL | `builder.rs:799` | — |
| `.sort_strategy` / `SortStrategy`, `.custom_sort_order`, `.namespace_strategy` / `NamespaceStrategy`, `.locked_prefixes` (+ private `default_namespace_prefixes`), `.output_mode` / `OutputMode`, `.line_ending` / `LineEnding`, `.unicode_normalization` / `UnicodeNormalization`, `.xml_character_policy` / `XmlCharacterPolicy`, `.quote_style` / `QuoteStyle`, `.time_zone_policy` / `TimeZonePolicy`, `.date_time_format` / `DateTimeFormat`, `.verify_determinism` | NO-OP (**removed in 3b**) | only set in `Default` (`:193-208`) and in the module doc (`:60-75`) | Rust; tests: `golden_files.rs:102-103` (sets `sort_strategy`, `output_mode`), `basic.rs:15` (asserts `locked_prefixes` is non-empty) |
| `DeterminismVerifier::verify`, `quick_check`, `verify_legacy`, `with_context_chars`, `with_outputs_retained` | REAL (**removed in 3b** (decided)) | builds N times through `Builder::build_internal` and compares the bytes and hashes (`:481-501`) | benches: `determinism.rs`; tests: `honest_results.rs:234-238` |
| `DeterminismVerifier::verify_with_threading_stress` | REAL (**removed in 3b** (decided)) | N threads build, and their hashes are compared with a sequential run (`:578-625`) | tests: `honest_results.rs:238` |
| `DeterminismVerifier.config` / the `new(config)` argument | NO-OP (**removed in 3b**) | passed to `Builder::with_config`, which no build reads (`:481,592`) | CLI, benches, tests |
| `DeterminismVerifier::verify_with_hashmap_stress` | FAKE (**removed in 3b**) | calls `self.verify`; there is no stress (`:569-575`) | via `thorough_check` |
| `DeterminismVerifier::thorough_check` | FAKE (**removed in 3b**) | its "multiple stress tests" are `verify` twice (`:735-756`) | CLI `guarantees --thorough` (result discarded, `bin:662`); benches: `determinism.rs:59-60` |
| `DeterminismStats.overhead_percentage` | FAKE (**removed in 3b**) | `(total − min) / min` over N builds comes out at about (N−1)×100%; it is not an overhead (`:521-525`) | — |
| `DeterminismResult`, `DeterminismDifference`, `HashDifference`, `LengthDifference`, `DifferenceContext`, other `DeterminismStats` fields | REAL (data) (**removed in 3b** (with the verifier)) | filled from real builds | — |

### `guarantees.rs` (removed in 3b)

`generate_guarantee_report` is the CLI `guarantees` subcommand. Ten of its twelve
guarantees are not checked as named.

| item | class | evidence | affected |
|---|---|---|---|
| `DeterministicCollections`, `StableSorting` (`validate_code_analysis_guarantee`) | FAKE | hard-coded `passed: false`, "Not verified" (`:240-256`) | CLI; tests: `honest_results.rs:243-247` |
| `FixedTimestamps`, `UnicodeNormalization`, `StableHashing`, `CanonicalOrdering`, `LockedNamespacePrefixes`, `CanonicalXmlOutput` (`validate_runtime_guarantee`) | FAKE | all six are the same `verifier.verify(request, 3)` call (`:278-285`, "itself is not checked separately") | CLI |
| `PlatformIndependence` (`validate_cross_platform_guarantee`) | FAKE | sets `LC_ALL` (`:410-416`), which nothing reads, then runs `verify` twice; it also mutates the process environment from a library | CLI |
| `MemoryIndependence` (`validate_stress_test_guarantee`) | FAKE | allocates 50 MB, builds, then sleeps 100 ms to "Allow GC" (`:455-466`) | CLI |
| `ThreadSafety` (`validate_concurrency_guarantee`) | REAL | 4 threads × `verify(2)`, then the hashes are compared (`:318-372`) | CLI |
| `validate_no_hashmap_usage`, `validate_deterministic_sorting` | FAKE | return `not_implemented` (`:512-527`) | tests: `honest_results.rs:241-242` |
| `DeterminismGuaranteeValidator.config` / the `new(config)` argument | NO-OP | goes to `DeterminismVerifier`, whose config is never applied | CLI, tests |
| `generate_guarantee_report`, `GuaranteeReport` (`failed_guarantees`, `critical_failures`, `summary`, `success_rate`) | FAKE (aggregate) | it aggregates the results above, so `overall_pass` / `success_rate` summarise checks that mostly did not run | CLI; root re-export `lib.rs:41` |
| `DeterminismGuarantee::description` | FAKE (text) | static claims that are false today (e.g. FixedTimestamps, while `MessageCreatedDateTime` is the current time) | CLI |
| `priority`, `validator`, `GuaranteeValidator`, `GuaranteePriority`, `GuaranteeValidationResult` | REAL (data) | — | CLI |
| `BuildError::DeterminismGuaranteeViolated` (`error.rs:60`) | follows the module | constructed only here | — |

### `fidelity.rs` (removed in 3b)

| item | class | evidence | affected |
|---|---|---|---|
| `FidelityConfig` and its whole tree (`PreservationLevel`, `CommentPreservationConfig`, `ExtensionPreservationConfig`, `UnknownExtensionHandling`, `ExtensionValidationConfig`, `AttributePreservationConfig`, `AttributeNormalizationConfig`, `AttributeValidationConfig`, `AttributeValidationRule`, `NamespacePreservationConfig`, `NamespaceMinimizationStrategy`, `NamespaceMinimizationRules`, `DefaultNamespaceHandling`, `WhitespacePreservationConfig`, `LineEndingStyle`, `IndentationStyle`) | NO-OP | nothing in `src/`, the tests or the bindings reads any of it; `FidelityConfig` is not even a field of `FidelityOptions` | Rust; root re-export `lib.rs:60`; `examples/perfect-fidelity/` at the repo root (not a workspace member, never built) |
| `FidelityStatistics`, `FidelityResult`, `FidelityIssue`, `FidelitySeverity` | DEAD | never constructed (only `Default`) | root re-export `lib.rs:60` |

### `verification.rs` (removed in 3b)

| item | class | evidence | affected |
|---|---|---|---|
| `BuildVerifier::verify` (overall `success`) | FAKE | the round-trip check is on by default and always fails, so the default `success` is always false; the schema check always fails; the "determinism" check never builds | `Builder::verify_build`, `build_with_fidelity` |
| round-trip check (`verify_round_trip`, `:158`) | FAKE | always `Err(not_implemented)` | — |
| canonicalization check (`:171`) | REAL (near-tautological) | canonicalizes the same input 3× and compares, which tests that `DB_C14N` is pure, not that the XML is canonical | — |
| schema check (`:264`) | FAKE | checks well-formedness, then always sets `success = false` ("XSD not implemented") | — |
| determinism check (`verify_determinism`, `:309`) | FAKE | documented as "multiple builds identical", but it never builds: it runs 4 regex warnings and an alphabetical-attributes regex | — |
| `VerificationConfig.enable_*` (4 flags) | REAL | gates (`:50,70,94,114`) | — |
| `VerificationConfig.determinism_test_iterations`, `verification_timeout` | NO-OP | read only by unit tests (`:570,577`); copied at `lib.rs:799-800,864-865` | — |
| `VerificationResult`, `VerificationIssue`, `VerificationSeverity` | REAL (data) | — | — |
| `VerificationReport` | DEAD | never constructed | — |
| `VerificationStatistics` | DEAD | only `Default`, never filled | root re-export `lib.rs:62` |

### `round_trip.rs`, `round_trip/parsed.rs`

| item | class | evidence | affected |
|---|---|---|---|
| `verify_round_trip`, `RoundTripVerification` | REAL | parse → `request_from_parsed` → build ×2 → parse → `compare_flat` (`:67-108`) | Node, Python (`testRoundTripFidelity`, `buildWithFidelity`), tests |
| `parsed::request_from_parsed`, `compare_flat`, `iso_duration`, `release_ids`, `release_type_name`, `commercial_model_name`, `use_type_name` | REAL | used by `verify_round_trip` | tests: `ern4_support.rs`, `ern4_samples.rs`, `ern38_round_trip.rs`, `round_trip.rs` |
| `pub(crate) not_implemented` | REAL (helper) (**removed in 3b** (no caller left)) | used by FAKE items in `guarantees`, `messages`, `lib.rs`, `verification`; it goes once they do | — |
| `RoundTripTester::new`, `test_round_trip_with`, `RoundTripResult` | REAL | `verify_round_trip` plus a C14N comparison (`:141-168`) | `Builder::test_round_trip_fidelity_with`; tests |
| `RoundTripTester.fidelity_options` / `fidelity_options()` | NO-OP (**removed in 3b** (`RoundTripTester::new()` takes no argument)) | the options are never applied | — |
| `RoundTripTester::test_round_trip` | FAKE (**removed in 3b**) | always `Err(not_implemented)` (`:130`) | `Builder::test_round_trip_fidelity`; tests: `honest_results.rs:83`, unit test `:289-295` |
| `RoundTripTester::analyze_fidelity` | FAKE (**removed in 3b**) | always `Err` (`:171`) | tests: `honest_results.rs:84` |
| `FidelityAnalysis`, `ElementAnalysis`, `AttributeAnalysis`, `CommentAnalysis`, `ExtensionAnalysis`, `NamespaceAnalysis` (+ `UnorderedMap`, PR 2 allow H7) | DEAD (**removed in 3b**) | never constructed; only the return type of `analyze_fidelity` | root re-export `lib.rs:61` |

### `canonical/*`

| item | class | evidence | affected |
|---|---|---|---|
| `DB_C14N::new`, `canonicalize` | REAL | `builder.rs:790` | all builds; tests; benches |
| `DB_C14N.config` (the `new(config)` argument) | NO-OP (**removed in 3c** (`new()` / `with_version(version)` take no config)) | `#[allow(dead_code)]` field (`mod.rs:103-104`), never read | every `DB_C14N::new(DeterminismConfig::default())` call (builder, lib, round_trip, about 6 test files, `benches/canonicalization.rs`) |
| `DB_C14N::with_version` | REAL | the version is used only when the XML has no ERN namespace (`:126-137`) | `canonical/tests.rs` |
| `DB_C14N::canonical_hash` | REAL | SHA-256 of the given bytes | `builder.rs:792`; tests: `golden_files.rs` |
| private `sort_child_elements`, the code after its early return | DEAD (**removed in 3c** (the whole function: it never reordered anything)) | the early return (`mod.rs:405-411`) covers every version key in `ElementOrder` | — |
| `#[cfg(test)] pub create_test_canonicalizer`, `create_test_canonicalizer_with_version` | DEAD (**row was wrong, kept**: `tests.rs`'s `integration_tests` module uses them; updated to the new `DB_C14N::new()`) | shadowed by local helpers in `canonical/tests.rs:10-18` | — |
| `rules::XML_DECLARATION`, `CanonicalNamespaceManager::{new, canonicalize_namespaces}`, `NamespacePrefixLock::{new, get_version_prefixes, deduplicate_prefixes}` | REAL | build path (`mod.rs:353,372,446`) | — |
| `NamespacePrefixLock.registry` | DEAD (**removed in 3c**) | `#[allow(dead_code)]` (`rules.rs:21-22`) | — |
| `NamespacePrefixLock::get_locked_prefix` | DEAD (**row was wrong, kept**: REAL; it is `pub` in the `pub` module `canonical::rules` and works) | unit tests only | — |
| `ElementOrder` (`new`, `get_element_order`, `get_version_orders`), `get_canonical_element_order`, the `CanonicalNamespaceManager.element_order` field | DEAD (**reclassified FAKE** (its tables do not follow the XSD sequences) and **removed in 3c**, with `get_canonical_element_order` and the manager's `element_order` field) | reached only from the dead tail of `sort_child_elements` | `rules.rs` unit tests |
| `AVSNamespaceHandler` (`is_avs_namespace`, `get_avs_prefix`), `requires_avs_handling` | DEAD (**row was wrong, kept**: REAL; `pub` and correct, only unused) | no caller | `rules.rs` unit tests |
| free functions `get_namespace_prefixes`, `get_element_order` | DEAD (`get_element_order` **removed in 3c** (with `ElementOrder`); `get_namespace_prefixes` **row was wrong, kept** (REAL, `pub`)) | no callers ("backward compatibility") | — |

### `generator/*`, `ast.rs`, `xml_escape.rs`

| item | class | evidence | affected |
|---|---|---|---|
| `ASTGenerator::{new, generate}`, `XmlWriter::{new, write}`, the ERN 3.8 / ERN 4 generators | REAL | build path | all |
| `generator::optimized_xml_writer` (`OptimizedXmlWriter`, `vectorized::write_elements_parallel`) | DEAD (**removed in 3c**) | no users outside its unit tests; its escaping differs from `xml_escape`, so its output would differ from `XmlWriter`'s | — |
| `ast::{AST, Element, Node}` and their builders | REAL | generators, writer; WASM `diff_viewer` (decided removal) | — |
| `Node::Comment`, `SimpleComment`, `add_comment`, `add_simple_comment`, `with_comment`, `with_namespace` | REAL (no build-path producer) | the writer emits them; no generator creates them | doctests |
| `xml_escape::*` (`pub(crate)`) | REAL | build path | — |

### `namespace_minimizer.rs` (removed in 3c)

| item | class | evidence | affected |
|---|---|---|---|
| `NamespaceMinimizer::{new, minimize, hoist_namespaces}`, `MinimizationResult`, `NamespaceUsage`, `NamespaceAnalysis` (+ 6 PR 2 allows, H8, D22) | DEAD (**removed in 3c**) | no user in `src/`, tests, benches, examples, bindings or the CLI; only `lib.rs:20` declares it | Rust |
| `with_conflict_resolution` / the `conflict_resolution` field | NO-OP (**removed in 3c**) | set (`:65,71`), never read | — |
| `AdvancedNamespaceMinimizer`, `OptimizationStrategy` | DEAD (**removed in 3c**) | no users; `Conservative` is `Minimal` plus a warning string (`:427-431`) | — |

### Files not compiled

| item | class | evidence | affected |
|---|---|---|---|
| `src/attribute_normalizer.rs` (whole file; H2) | DEAD (**removed in 3c**) | no `mod attribute_normalizer` in `lib.rs` | none |
| `src/extension_restoration.rs` (whole file) | DEAD (**removed in 3c**) | no `mod extension_restoration` in `lib.rs` | none (`docs/integration_test_extensions.md`, `dev-docs/TEST_TRIAGE.md` mention it) |
| `src/presets/custom_template.rs`, `src/messages/test_data.rs` | DEAD | not declared | go with the decided removals |

### `caching.rs` (removed in 3c)

Nothing on the build path uses it. The only external user is `tests/it/honest_results.rs:252-263`.

| item | class | evidence | affected |
|---|---|---|---|
| `CacheManager::fast_hash`, `stats` (+ PR 2 allow D23) | REAL (no build-path user) (**removed in 3c** (with the module)) | — | tests: `honest_results.rs:254-262` |
| `CacheManager::get_schema`, `clear_all`, `prune_expired`; `GlobalCache`, `SchemaCache`, `HashCache` | DEAD (**removed in 3c**) | no users; there are no schemas to cache | Rust |
| `CacheStats` `validation_*` / `template_*` fields, `overall_hit_rate`, `summary` | FAKE (**removed in 3c**) | the validation and template counters are never incremented, so they average in as 0 | — |
| `ValidationCache` | DEAD (**removed in 3c**) | no `CacheManager` accessor | — |
| `ValidationCache::hit_rate` | FAKE (**removed in 3c**) | `entries / total_accesses` is not a hit rate (`:328-342`) | — |
| `CompiledSchema`, `ValidationRule`, `RuleType`, `ElementConstraint`, `caching::ValidationResult`, `TemplateCache`, `TemplateKey`, `CompiledTemplate`, `TemplatePart` | DEAD (**removed in 3c**) | nothing produces or consumes them (there is no schema or template compiler) | — |

### `memory_optimization.rs`, `optimized_strings.rs`, `parallel_processing.rs`

None is on the build path, and no binding uses them. Their only users outside the three files
are each other, `benches/performance.rs` and `tests/it/honest_results.rs`.

| item | class | evidence | affected |
|---|---|---|---|
| `memory_optimization::Arena::{alloc, allocated_bytes, capacity_bytes, reset, clear}` | FAKE (**removed in 3c**) | `Box::new(value)` plus zero-filling a byte `Vec` to "simulate allocation" (`:31-57`) | via `BuildMemoryManager` |
| `BuildMemoryManager` (+ `memory_usage`, `reset_for_next_build`, `full_reset`) | NO-OP (**removed in 3c**) | its only consumer ignores it (`_memory_manager`, `parallel_processing.rs:99,117`) | benches: `performance.rs:121,134` |
| `MemoryStats::total_bytes` | FAKE (**removed in 3c**) | hard-coded 64 B / 8192 B per pooled object | — |
| `ObjectPool`, `PooledObject`, `LazyField` | DEAD (**removed in 3c**) | no users | — |
| `CompactAST` (`from_ast`, `intern_string`, `get_string`, `add_element`, `memory_footprint`), `CompactAttributes`, `CompactNodeRef`, `CompactElement` | DEAD (**removed in 3c**) | write-only (private fields, no getters for the tree); nothing reads a `CompactAST` back | tests: `honest_results.rs:312-330` |
| `NodeType` | DEAD (**removed in 3c**) | never used | — |
| `optimized_strings::{BuildContext, BuildStats, MemoryUsage, StringInterner, OptimizedString, FastString, BufferPool, buffer_sizes}` | DEAD (**removed in 3c**) | used only by `caching`, `optimized_xml_writer`, `parallel_processing` (DEAD / FAKE) and the bench | benches: `performance.rs:122` |
| `BufferPool::memory_usage` | FAKE (**removed in 3c**) | `current_size` is never decremented (`:285-311`) | — |
| `intern_atom`, `CowString`, `OptimizedLocalizedString` | DEAD (**removed in 3c**) | no users | — |
| `ParallelProcessor::process_build_parallel`, `ParallelBuildResult` | FAKE (**removed in 3c**) | writes no AST and no XML ("Simulate track processing work", `:188-189`); returns names and timings | benches: `performance.rs:119-145` |
| `ParallelBuildResult::meets_performance_target` | FAKE (**removed in 3c**) | hard-coded 5 / 10 / 50 ms targets applied to that non-build | — |
| `ParallelProcessor::{new, validate_items_parallel, generate_xml_sections_parallel}`, `ParallelConfig`, `WorkloadAnalyzer`, `WorkloadAnalysis`, `ProcessedElement`, `parallel_processing::ValidationResult` | DEAD (**removed in 3c**) | no users besides the bench; `generate_xml_sections_parallel` writes unescaped text | benches |
| `ThreadMetrics` | DEAD (**removed in 3c**) | never constructed | — |
| `BuildError::Parallel` (`error.rs:73`) | follows the module (**removed in 3c**) | constructed only here | — |

### `versions/*` (removed in 3d, except `DdexVersion` and `utils::supported_versions`)

No binding, bench, example, website doc or README uses it. Its users are the CLI `convert`
subcommand, `Builder`, `tests/it/honest_results.rs`, `tests/it/undefined_entities.rs` and
`fuzz/` (already broken).

| item | class | evidence | affected |
|---|---|---|---|
| `VersionConverter::convert`, `ConverterResult` | FAKE (**removed in 3d**) | changes only the ERN namespace URI and renames one element (`TechnicalSoundRecordingDetails` ↔ `TechnicalDetails`, 3.8.2 ↔ 4.2). It drops `CommercialModelType` (4.2 → 3.8.2; it exists in 3.8.2) and `VideoResource` (4.3 → 4.2; no such element). It never fixes `MessageSchemaVersionId` / `AvsVersionId` / `schemaLocation`, does no 3.8 → 4.x restructuring, and skips self-closing elements. It returns `Success` for any well-formed input; same-version input is returned unchanged as `Success`. No path produces target-schema-valid output | CLI `convert`; `Builder::convert_version`; tests: `honest_results.rs:186-212`, `undefined_entities.rs:69-80`; fuzz |
| `ConverterReport.from_version`, `to_version`, `warnings`, `elements_converted`, `elements_dropped` | REAL (counts) (**removed in 3d**) | `converter.rs:457-460` | CLI `--report` |
| `ConverterReport.elements_added` | FAKE (**removed in 3d**) | always 0 (`:722-723`) | tests: `honest_results.rs:212` |
| `ConversionWarningType` (variants other than `ElementRenamed` / `ElementDropped`) | DEAD (**removed in 3d**) | never built | — |
| `VersionConverter::new`, `get_supported_conversions`, `can_convert`, `ConverterWarning` | REAL (tables) (**removed in 3d**) | — | — |
| private `NamespaceMapping.schema_version_*`, `_field_migrations`, `_validation_changes`, `ElementMapping::Split` / `Merge` | DEAD (**removed in 3d**) | `#[allow(dead_code)]` (`converter.rs:81-160`) | — |
| `ConversionOptions.preserve_comments` | REAL (**removed in 3d**) | `converter.rs:637` | CLI |
| `ConversionOptions.allow_lossy`, `detailed_reports`, `preserve_unknown`, `add_metadata`, `validation_level`, `custom_mappings`; `ValidationLevel` | NO-OP (**removed in 3d**) | only set, never read; CLI `--allow-lossy` maps to `preserve_unknown`, so it does nothing (`bin:550-551`) | CLI |
| `VersionManager::new`, `detect_version`, `VersionDetection.detected_version`, `clues`, `DetectionClue` | REAL (heuristic) (**reclassified FAKE** (it scores invented element lists and misidentifies real messages) and **removed in 3d**) | the namespace scoring is real but often wrong: it scores invented `new_elements` (a port of the scoring got 38 of 84 repo ERN files wrong) | `Builder::detect_version`; tests |
| `VersionDetection.confidence` | FAKE (**removed in 3d**) | invented weights (`mod.rs:470-532`) | — |
| `VersionDetection.ambiguities` | FAKE (**removed in 3d**) | always empty (`// TODO`, `:538`) | — |
| `ClueType::SchemaLocation`, `StructuralPattern`, `NamespacePrefix` | DEAD (**removed in 3d**) | never built | — |
| `VersionManager::is_conversion_supported` | REAL (hard-coded table) (**reclassified FAKE** (compatibility of a converter that does not convert) and **removed in 3d**) | 6 pairs | `Builder::is_version_compatible` |
| `VersionManager::get_version_spec`, `get_conversion_path`, `get_feature_compatibility`, `get_recommended_strategy`; `CompatibilityMatrix`, `ConversionPath`, `ConversionDifficulty`, `FeatureSupport`, `SupportLevel`, `ConversionStrategy`, `RiskLevel` | FAKE (**removed in 3d**) | hard-coded prose, invented features ("EnhancedMetadata New in 4.3"), "High-fidelity conversion" / `RiskLevel::Low` claims (`mod.rs:551-765`) | Rust |
| `VersionManager._default_options` | NO-OP (**removed in 3d**) | never read | — |
| `VersionSpec.namespace`, `message_schema_version_id`, `new_elements` | REAL / FAKE data (**removed in 3d**) | read by `detect_version`; `new_elements` lists elements that are in no ERN XSD | — |
| `VersionSpec.version`, `schema_location`, `supported_message_types`, `element_mappings`, `required_elements`, `deprecated_elements`, `namespace_prefixes` | NO-OP (**removed in 3d**) | never read | — |
| `versions::ConversionResult` (struct, `converted_at`, D19), `ConversionReport` (mod.rs copy), `ElementConversion`, `ConversionType`, `ConversionWarning` (mod.rs copy), `ConversionError`, `versions::ImpactLevel`, `ConversionMetadata` (`fidelity_percentage`) | DEAD (**removed in 3d**) | never constructed (`mod.rs:101-241`) | Rust |
| `utils::supported_versions`, `is_legacy_version`, `is_modern_version` | REAL (`supported_versions` kept; `is_legacy_version` / `is_modern_version` **reclassified FAKE** (arbitrary: 4.2 is neither) and **removed in 3d**) | — | — |
| `utils::get_version_release_date`, `get_version_description` | FAKE (**removed in 3d**) | unsourced dates and prose | — |
| `ern_382` / `ern_42` / `ern_43` `get_version_spec` | FAKE (data) (**removed in 3d**) | invented elements (`LegacyTechnicalDetails`, `BasicPrice`, `SimpleTerritory`, `OldStyleGenre`, `TechnicalResourceDetails`); read only by `VersionManager` | — |
| `Ern382Constraints` / `Ern42Constraints` / `Ern43Constraints`, `get_namespace_mappings*`, `get_xml_template*` (4.x templates carry `MessageSchemaVersionId`), `builders::*` (schema-invalid ASTs), `validation::validate_*` (hard-coded short lists), `validate_ern_*_message`, `ern_382::get_validation_rules` / `ValidationRule` / `ValidationRuleType`, and the root re-exports `builders`, `get_version_spec`, `validation` | DEAD + FAKE (**removed in 3d**) | no callers anywhere; the content is invented | Rust |

### `schema/*` (removed in 3d, except `SchemaValidator`, `JsonSchema` and `MessageProfile`)

No binding, bench, example, website doc or README uses it. Its users are the CLI `schema`
subcommand and `tests/it/honest_results.rs:272-300` (`SchemaValidator`).

| item | class | evidence | affected |
|---|---|---|---|
| `SchemaGenerator::generate_build_request_schema`, `generate_complete_schema` | FAKE (**removed in 3d**) | hand-written JSON, not derived from `BuildRequest`. It lists 3 of `BuildRequest`'s 9 fields under `additionalProperties: false` (no `version`, a required serde field), so every real request violates it. `$ref`s dangle in one, and the root accepts any object in the other. The output does not vary by version or profile | CLI `schema`; doctests `schema/mod.rs:40-95`; unit tests |
| `SchemaGenerator::generate_flat_release_schema` | FAKE (**removed in 3d**) | no matching type (`artist`, `upc` do not match core `ParsedRelease`) | — |
| `generate_typescript_types`, `generate_python_types` | REAL (a transform of a FAKE input; timestamp D17) (**removed in 3d** (they exist only for the generator)) | `types.rs` | CLI |
| `SchemaGenerator::new`, `with_config`, `SchemaDraft`, `SchemaGenerationResult`, `SchemaWarning` | REAL (mechanics) (**removed in 3d**) | — | CLI |
| `SchemaConfig.draft_version`, `include_examples` | REAL (**removed in 3d** (generator-only)) | read | — |
| `SchemaConfig.include_descriptions`, `strict_validation`, `include_deprecated` | NO-OP (**removed in 3d**) | never read; the CLI's `--with-docs` builds a config that is then discarded (`_schema_config`, `bin:608`) | CLI |
| `SchemaConfig.version_conditionals` | NO-OP (**removed in 3d**) | `apply_version_conditionals` has empty match arms (`generators.rs:848-871`) | — |
| `SchemaMetadata.complexity_score` | FAKE (**removed in 3d**) | invented weights (`mod.rs:597-632`) | — |
| `SchemaMetadata.property_count`, `required_count`, `generated_at` | REAL (**removed in 3d** (generator-only)) | — | — |
| `schema::SchemaCommand` (lib) + `execute` | DEAD (**removed in 3d**) | the CLI has its own clap `SchemaCommand`; root re-export `lib.rs:50` | unit test |
| `types::generate_openapi_spec` | FAKE + DEAD (**removed in 3d**) | a fictional REST API (`https://api.ddex-builder.example.com`); no callers | — |
| `generators::*` (`pub(crate)`) | FAKE (**removed in 3d**) | hand-written schemas with invented patterns (`^DEAL[0-9]+$`, `^R[0-9]+$`) | — |
| `SchemaValidator::{new, with_config, validate}`, `schema::ValidationConfig` (all fields read), `ValidationResult` (+ `is_valid`, `error_messages`, `warning_messages`, `summary`), `ValidationError`, `ValidationWarning`, `ValidationMetadata.properties_validated` / `validation_time` | REAL (partial) | a real JSON Schema subset. Numbers are not checked, `"integer"` is reported as a mismatch, and the ISRC/UPC checks are keyed on path names | tests: `honest_results.rs:290` |
| `SchemaValidator::validate_with_paths` | FAKE (**removed in 3d**) | identical to `validate`; the docs claim more detail (`validation.rs:158-161`) | — |
| `ValidationMetadata.schema_complexity`, `data_complexity` | FAKE (**removed in 3d**) | invented scores (`:813-858`) | — |
| `JsonSchema` | REAL (data) | `SchemaValidator`'s input | tests |

### `streaming/*`

| item | class | evidence | affected |
|---|---|---|---|
| `StreamingBuilder::{new, new_with_config, set_progress_callback, set_estimated_total, into_inner, start_message, write_resource, finish_resources_start_releases, write_release, finish_message}`, `StreamingProgress`, `ProgressCallback` | REAL (known issues: ERN 3.8-style tree only, D12/D13/D14) | — | Node `StreamingDdexBuilder`; tests: `target_version.rs`, `determinism_0_6.rs`; `examples/streaming_catalog_example.rs` |
| `StreamingConfig.max_buffer_size`, `progress_callback_frequency` | REAL | `:137,545`, `:287,398` | Node |
| `StreamingConfig.deterministic`, `determinism_config`, `validate_during_stream` | NO-OP (**removed in 3c** (also the Node `StreamingConfig.deterministic` / `validateDuringStream`)) | never read: it always uses `StreamingReferenceManager::new()` (`:141`) and no validation runs | Node `startMessage` (`bindings/node/src/lib.rs:676-681`) |
| `StreamingStats.releases_written`, `resources_written`, `bytes_written` | REAL | — | Node `finishMessage` |
| `StreamingStats.deals_written` | FAKE (**removed in 3c** (also Node `dealsWritten`)) | always 0: there is no `write_deal`, and the field is never incremented | Node `StreamingStats` |
| `StreamingStats.warnings` | FAKE (**removed in 3c** (also Node `warnings`)) | always empty: nothing pushes to `self.warnings` | Node |
| `StreamingResult` | DEAD (**removed in 3c**) | never constructed; duplicates `StreamingStats` | — |
| `StreamingError` and `From<StreamingError> for BuildError` | DEAD (**removed in 3c**) | no variant is ever constructed | — |
| `BufferConfig.max_buffer_size`, `buffer_count`; `BufferManager` and all its methods; `BufferStats` | REAL | — | — |
| `BufferConfig.enable_compression` | NO-OP (**removed in 3c**) | both branches of `write_buffer` write the buffer unchanged (TODO) | — |
| `ReferenceConfig` (all fields), `StreamingReferenceManager::generate_*_reference`, `store_*_metadata`, `get_stats`, `ReferenceStats`, `get_resource_reference`, `get_release_reference` | REAL | — | — |
| `StreamingReferenceManager.hash_generator` | DEAD (left for PR 6 (streaming references)) | `#[allow(dead_code)]`; SHA-256 is used directly | — |
| `StreamingReferenceManager::validate_references`, `ReferenceValidationResult.errors` / `is_valid` | FAKE (errors) (left for PR 6 (streaming references)) | every insert and eviction updates both maps, so `errors` is always empty and `is_valid` always true (`:399-416`); the warnings are real | — |
| `ReferenceValidation` | DEAD (left for PR 6 (streaming references)) | never constructed | — |

### `preflight.rs`, `id_generator.rs`, `error.rs`

| item | class | evidence | affected |
|---|---|---|---|
| `PreflightValidator::{new, validate}`, `PreflightLevel`, `ValidationResult`, `ValidationError`, `ValidationWarning`, `ValidationInfo`; `ValidationConfig.level`, `profile`, `validate_identifiers`, `check_required_fields`, `validate_references` | REAL | `builder.rs:738-770` | all |
| `ValidationConfig.validate_checksums` | NO-OP (**removed in 3c**) | never read (the UPC check digit is always checked under `validate_identifiers`) | `builder.rs:743`; tests: `golden_files.rs:186` |
| `ValidationConfig.validate_dates` | NO-OP (**removed in 3c**) | never read (there is no date validation) | `builder.rs:745`; tests: `golden_files.rs:188` |
| private statics `ISWC_PATTERN`, `ISNI_PATTERN` | DEAD (**removed in 3c**) | unused | — |
| `StableHashGenerator`, `StableHashConfig`, `HashAlgorithm` | REAL | `IdStrategy::StableHash`; **out of scope (PR 6)** | — |
| `HashRecipe`, `NormalizeOptions`, `UnicodeForm`, `CaseNormalization` | DEAD (left for PR 6 (ID generation)) | pub, but no public API takes or returns them | — |
| `BuildError` variants except the next two | REAL | constructed in `src/` | all |
| `BuildError::DeterminismFailed` | DEAD (**removed in 3b**) | never constructed | `DETERMINISM.md:391` |
| `error::BuildWarning` (exported at the root as `ddex_builder::BuildWarning`) | REAL (confusing duplicate) | used by `streaming`; `DDEXBuilder::build` returns the different `builder::BuildWarning` | — |

### `linker/*`

Not used by the build path, the CLI or the bindings; used only by `tests/it/linker.rs`,
`linker_xml_integration.rs` and `xml_generation.rs`.

| item | class | evidence | affected |
|---|---|---|---|
| `ReferenceLinker`, `ReferenceGenerator`, `RelationshipManager` (all methods), `EntityType`, `ReferenceStyle`, `ReleaseResourceReference`, `LinkingReport` counts, `LinkerConfig.reference_style` | REAL | standalone | tests |
| `LinkerConfig.auto_link`, `validate_on_build`, `strict` | NO-OP (**removed in 3c** (and the stored `config` field)) | the `config` field is `#[allow(dead_code)]` (`mod.rs:31`); only `reference_style` is read, at construction | tests: `linker.rs:228-230` |
| `LinkingReport.validation_passed` | FAKE (**removed in 3c**) | `true` even when validation fails (`auto_linker.rs:156-163`) | tests: `linker.rs:158`, `linker_xml_integration.rs:82`, `xml_generation.rs:82` |
| `AutoLinker::with_config`, `AutoLinkerConfig` (incl. the NO-OP `generate_missing_refs`) | DEAD (`generate_missing_refs` **removed in 3c**; **row partly wrong**: `with_config` can be called with `Default::default()`, and the other fields are read, so the rest stays) | private module, not re-exported (`mod.rs:11`) | — |
| `LinkingStats`, `ResourceLink` | DEAD (**removed in 3c**) | never used | — |
| `LinkingError::{UnknownRelease, DuplicateReference, CircularReference, InvalidEntityType, ValidationFailed}` | DEAD (**removed in 3c**) | never constructed | — |
| `RelationshipStatistics` | REAL (not nameable) | returned by `get_statistics`, not re-exported | — |

### `security.rs`, `security/*`, `api_security.rs`

Opt-in helpers: no build, CLI or binding uses them. Their users are `tests/it/security/**`,
`security_fixes.rs`, `security_integration.rs`, `entity_classification.rs`,
`error_sanitizer.rs`, `benches/security.rs` and `examples/error_sanitization_demo.rs`.
Most of it is REAL; many REAL items are buggy (see
[Found along the way](#found-along-the-way-not-removals)).

| item | class | evidence | affected |
|---|---|---|---|
| `SecurityConfig` fields other than the next row, `SecureXmlReader`, `InputValidator` (all methods), `RateLimiter`, `OutputSanitizer::{new, sanitize_xml_output, create_secure_log_message}`, `SecureTempFile`, `utils::*` | REAL | — | tests, benches |
| `SecurityConfig.max_attributes_per_element` (+ const `:54`) | NO-OP (**removed in 3c**) | never read; set only in `Default` (`:141`) | tests: `security/mod.rs:106,123` |
| `OutputSanitizer.config` | NO-OP (field **removed in 3c** (the `new(config)` argument stays)) | `#[allow(dead_code)]` (`:625`); the `SecurityConfig` passed to `new` is never read (the argument can stay) | — |
| private `DANGEROUS_PATH_REGEX` | DEAD (**removed in 3c**) | `#[allow(dead_code)]` (`:90`) | — |
| `ApiSecurityManager::{new, validate_request, sanitize_response, validate_ffi_input, create_secure_error_response}`, `BatchOperationMonitor`, `BatchStats`, `FfiValidator`, `FfiDataType` | REAL | — | unit tests |
| `ApiSecurityManager::get_wasm_security_headers` | FAKE (**removed in 3c**) | returns a hard-coded header map and reads no config; the WASM binding does not use it (`api_security.rs:72-103`) | unit test `:454-470` |
| `ApiSecurityConfig` (all fields) | NO-OP (**removed in 3c**) | no field is read and no function takes it | root re-export `lib.rs:56` |
| `RateLimitInfo`, `ContentType`, private `OperationRecord.resource_usage` | DEAD (**removed in 3c**) | never constructed / referenced (`:201,212`; `resource_usage` is always 1, `:136,171`) | — |
| `EntityClassifier` (all methods), `ClassifierConfig` (all fields read), `Entity`, `EntityClass`, `ValidationResult`, `EntityMetrics`, `create_*_entity`, `AttackType::{ExternalEntity, NetworkRequest, EntityBomb}` | REAL (heuristic) (**removed in 3c** (decided: unused by any build, wrong answers in some configurations), with `InputValidator::validate_entities` / `classify_entity` / `get_entity_metrics` and the classifier-only `SecurityConfig` fields) | — | tests, bench |
| private `load_ddex_whitelist` (what makes `EntityClass::SafeDdex` for DDEX element names) | FAKE (**removed in 3c**) | hard-coded element names "would typically be loaded from DDEX schema files"; DDEX defines no XML entities. It runs before the external-reference check, so `classify_entity("ddex", "SYSTEM 'file:///etc/passwd'")` is `SafeDdex` (`entity_classifier.rs:275,285,527-583`) | tests: `entity_classification.rs:44-66`, unit test `:681-689` |
| `AttackType::{ExponentialExpansion, RecursiveEntity, FileAccess, ParameterEntity}`, `ClassificationResult` | DEAD (**removed in 3c**) | never constructed / referenced | — |
| `ErrorSanitizer` (all methods), `SanitizerConfig` (all fields read), `SanitizedError`, `RedactionRule`, `ErrorMode`, `ErrorContext`, `SanitizerStatistics`, the `sanitize_*` free functions, `init_global_sanitizer`, `with_global_sanitizer` | REAL (**removed in 3c** (decided)) | — | tests, example, bench |
| `SecureError` trait, `ErrorLevel` | DEAD (**removed in 3c**) | no implementor outside a unit test; nothing takes a `SecureError` | unit test `error_sanitizer.rs:698-753`; re-exports `security.rs:26-27` |
| `PathValidator` (all methods), `PathValidationConfig` (all fields read), `ValidatedPath` | REAL (buggy) | — | tests |

### CLI (`bin/ddex-builder.rs`)

| item | class | evidence | affected |
|---|---|---|---|
| `build` | REAL | `DDEXBuilder::build` | CLI |
| `build --verify-determinism`, `--determinism-iterations` | REAL | builds N times and compares the bytes (`:1138-1224`) | CLI |
| `build --validate` | REAL (redundant) | runs the build once more; preflight runs anyway | CLI |
| `build --profile`, `build --strict` | NO-OP (**removed in 3a**) | `cmd.profile` is never read; `validate_input_data` ignores `_strict` (`:1016`) | CLI |
| `build --preset` | decided | — | — |
| `convert` | FAKE (**removed in 3d**) | `VersionConverter::convert`; `--allow-lossy` is NO-OP (above) | CLI |
| `diff` | decided (FAKE: "not implemented") | — | — |
| `validate`, `watch`, `server` | FAKE ("not implemented") | **PR 4**; only their `--preset` flag went (3a). `validate`'s own `--profile` / `--strict` stay with the command | CLI |
| `schema` | FAKE (**removed in 3d**) | `generate_complete_schema`; `--profile` is ignored (hard-coded `MessageProfile::AudioAlbum`, `:614`) and `--with-docs` is NO-OP (both flags **removed in 3a**) | CLI |
| `guarantees` | FAKE (**removed in 3b**) | `generate_guarantee_report`; `--thorough` discards its result (`:661-662`) | CLI |
| `preset list/show/apply` | decided | — | — |
| `completions` | REAL | clap | CLI |
| global `--config` (`load_config`, `ConfigFile`) | NO-OP (**removed in 3a**) | `load_config` returns an empty default (`:473-481`, `TODO`) | CLI |
| `DdexVersionArg::V41`, `V44` | UNSURE | `4.4` maps silently to `Ern43` (`:399`) and `4.1` to `Ern41`, which the builder cannot target | CLI |

### Bindings beyond the decided removals

PR 3 removes only binding methods that wrap removed Rust APIs. Binding-only stubs are PR 4.
For reference:

| item | class | evidence | PR |
|---|---|---|---|
| Node / Python `testRoundTripFidelity` / `test_round_trip_fidelity` | REAL | `round_trip::verify_round_trip` (kept) | stays |
| Node / Python `buildWithFidelity` / `build_with_fidelity` | REAL build + REAL round trip; NO-OP options | they call `DDEXBuilder::build` and `verify_round_trip`, not the `lib.rs` `Builder`. Of the 14 `FidelityOptions` fields only `canonicalization`, `collect_statistics` and `enable_verification` are read; `applied_fidelity` returns constants; `fidelity_score` is always null | PR 4 (binding-only) |
| Node `StreamingConfig.deterministic`, `validateDuringStream` | NO-OP | they forward the Rust NO-OP fields | PR 4, or with the Rust fields |
| Node / Python `Release.metadata` / `Resource.metadata` | UNSURE | not checked in this pass | PR 4 |

## UNSURE

1. **CLI `DdexVersionArg::V41` / `V44`**: `4.4` silently becomes 4.3 and `4.1` cannot be
   built. Removing the two values is a CLI change, not an API removal, and depends on what
   the CLI should accept. Kept.
2. **`StreamingStats.peak_memory_usage`**: computed, but only from queued `BufferManager`
   chunks. Full `xml_buffer` flushes take `write_directly` and skip the peak update, so it
   under-reports. Whether that counts as FAKE depends on what it is meant to measure. Kept.
3. **Node / Python `Release.metadata` / `Resource.metadata`**: binding-only; I did not trace
   whether any build path reads them. PR 4.
4. **`guarantees::ThreadSafety`**: it is a real check, but it is one row of a report whose
   other ten guarantees are FAKE. `DeterminismVerifier::verify_with_threading_stress` does
   the same check. **Decided and removed in 3b**: the module, ThreadSafety and the verifier
   went (the `determinism_0_6` fresh-thread tests cover threading in CI).

## What a full removal touches

About 100 files, so PR 3 stops at this inventory. Rough count:

- **Deleted** (about 31 files):
  - `diff/*` (5), `messages/*` (3), `presets/*` (4)
  - `fidelity.rs`, `verification.rs`, `guarantees.rs`, `namespace_minimizer.rs`
  - `caching.rs`, `memory_optimization.rs`, `optimized_strings.rs`, `parallel_processing.rs`
  - `generator/optimized_xml_writer.rs`, `attribute_normalizer.rs`, `extension_restoration.rs`
  - `versions/*` (5, except whatever holds `DdexVersion`)
  - `schema/generators.rs`, `schema/types.rs`
  - `tests/it/presets.rs`
  - `bindings/wasm/src/diff_viewer.rs`
- **Edited `src/`** (about 24):
  - `lib.rs`, `builder.rs`, `error.rs`, `determinism.rs`
  - `canonical/{mod,rules,tests}.rs`, `generator/mod.rs`, `round_trip.rs`
  - `streaming/{mod,buffer_manager,reference_manager}.rs`
  - `security.rs`, `api_security.rs`, `security/{entity_classifier,error_sanitizer}.rs`
  - `preflight.rs`, `id_generator.rs`, `linker/{mod,auto_linker,types}.rs`
  - `schema/{mod,validation}.rs`, `bin/ddex-builder.rs`
- **Tests** (about 14):
  - `main.rs`, `honest_results.rs`, `determinism_0_6.rs`, `golden_files.rs`, `basic.rs`
  - `canonical_debug.rs`, `undefined_entities.rs`, `entity_classification.rs`
  - `security/mod.rs`, `linker.rs`, `linker_xml_integration.rs`, `xml_generation.rs`
  - plus the `DB_C14N::new` callers, if its NO-OP argument goes
- **Benches, examples, fuzz** (about 8):
  - `benches/{performance,determinism,canonicalization}.rs`, `Cargo.toml`
  - `examples/spotify_album_example.rs`, `examples/README.md`
  - `fuzz/fuzz_targets/{fuzz_builder_api,fuzz_version_conversion}.rs`
- **Bindings** (about 13):
  - Node: `src/lib.rs`, `index.d.ts`, `js/index.ts`, `test-honest-results.js`, `README.md`
  - Python: `src/lib.rs`, `__init__.py`, `test_honest_results.py`, `test_import.py`, `README.md`
  - WASM: `src/lib.rs`, `Cargo.toml`, `README.md`
- **Records** (about 7):
  - `CHANGELOG.md`, `KNOWN_ISSUES.md`, `dev-docs/DETERMINISM_AUDIT_0.6.0.md`, this file
  - `.gitignore`, `packages/ddex-builder/README.md`, `Cargo.lock`

The website needs no change: no runnable block there uses a removed API, only prose does
(`intro.md:54`, `known-issues.md`, `python-dataframes.md:244`), and that is out of scope.

**PR 2 allows:** 65 in all (counted in the source). The full removal deletes 31 of them
and leaves 34; PR 3a alone deletes 6 (diff, update messages and `serialize_update_header`).
The 31 by group:

- diff: 2
- update messages: 3
- `builder.rs` `serialize_update_header`: 1
- `fidelity.rs`: 4
- `guarantees.rs`: 2
- `lib.rs` (H4): 3
- `namespace_minimizer.rs`: 6
- `round_trip.rs` (H7): 1
- `schema/mod.rs`: 5
- `verification.rs`: 3
- `caching.rs`: 1

**Dependencies:**
- ddex-builder: none becomes unused through the decided removals. With the full removal,
  `rayon` and `num_cpus` go once `parallel_processing` and `guarantees` are gone; the CLI
  still uses `num_cpus` for `server --workers`, which is PR 4. `string_cache` and
  `smartstring` go once `optimized_strings` is gone. Check each with cargo-deny / unused-deps.
- WASM: `serde`, `html-escape` and `indexmap` go with `diff_viewer.rs`.
- Already unused today, independent of these removals: `log`, `dhat` (no code is gated on
  `dhat-heap`), and the dev-dependencies `similar`, `pretty_assertions`, `rand` and
  `tokio-test`.

## Proposed split

Each part is under about 60 files and can land on its own.

1. **PR 3a: the decided removals (done).** About 45 files:
   - update messages + diff, and presets (Rust, CLI `diff` / `preset` / `--preset`, the
     Node / Python preset methods, the WASM diff viewer);
   - the move of `DdexVersion` / `MessageProfile`;
   - test (e);
   - the CHANGELOG / KNOWN_ISSUES / audit records.

   Mechanical, with no open questions.
2. **PR 3b: fidelity, verification and determinism fakes (done).** About 35 files:
   - `fidelity.rs`, `verification.rs`, `guarantees.rs` and the CLI `guarantees` command;
   - the FAKE / NO-OP `Builder` fidelity methods and options;
   - the 12 NO-OP `DeterminismConfig` fields and their enums, plus `CanonMode::Compact`;
   - `thorough_check` / `verify_with_hashmap_stress`;
   - `RoundTripTester::test_round_trip` / `analyze_fidelity` and `FidelityAnalysis`;
   - the `DB_C14N` config argument.

   Needs your answer on UNSURE 4.
3. **PR 3c: dead and fake internals (done).** About 25 files:
   - `attribute_normalizer.rs`, `extension_restoration.rs`, `namespace_minimizer.rs`;
   - `caching.rs`, `memory_optimization.rs`, `optimized_strings.rs`, `parallel_processing.rs`,
     `optimized_xml_writer.rs` and `benches/performance.rs`'s parallel bench;
   - the DEAD / FAKE / NO-OP items in `canonical/rules.rs`, `streaming/*`, `linker/*`,
     `security*`, `preflight.rs`, `id_generator.rs` and `error.rs`.

   The removal of `rayon` / `string_cache` / `smartstring` / `log` / `dhat` lands here.
4. **PR 3d: `versions` and `schema` (done).** About 20 files:
   - CLI `convert` and `schema`, `Builder::convert_version` / `detect_version` /
     `is_version_compatible`, and the `versions` / `schema` generator code;
   - keep `SchemaValidator` and `DdexVersion`.

   This removes two CLI commands that run today (with made-up results), so it is the one most
   worth a separate look.

## Closing summary (after 3d)

PR 3 is complete: 3a #21, 3b #22, 3c #23, 3d (this PR).

**Inventory rows marked removed, per PR** (a row is one item or a group of items):

| PR | rows | what |
|---|---:|---|
| 3a | 22 | the decided removals (update messages, diff, presets: 19 of the 20 decided rows; `DdexVersion` / `MessageProfile` were moved, not removed) and 3 no-op CLI flag rows |
| 3b | 39 | guarantees, the determinism verifier, verification, fidelity, the fake round-trip and canonicalization APIs, the 12 unread `DeterminismConfig` fields |
| 3c | 57 | dead modules, the entity classifier and error sanitizer, the streaming / linker / security / preflight / `canonical::rules` no-ops and fakes, the `DB_C14N` config argument |
| 3d | 42 | version conversion and detection, schema generation, the CLI `convert` and `schema` commands |
| **total** | **160** | |

A few rows were only partly removed, or turned out to be wrong and were kept. Each is
marked in its table.

**Left for later PRs:**
- PR 4: the binding-only stubs.
- PR 5: renames, the `Builder` / `DDEXBuilder` question, `BuildResult.errors`,
  `DDEXBuilder._inner`, `MessageProfile` (unused since 3d).
- PR 6: ID generation, streaming references and `id_generator`'s unreachable types.
- PR 7: preflight Warn mode, the UPC checksum, `StreamingStats.peak_memory_usage`.
- PR 8: the quick-xml types in the public API.

**What is left of the `lib.rs` `Builder`** (for the PR 5 decision):
- Fields: `target_version` and `fidelity_options`.
- Methods:
  - `new`, `with_fidelity_options`, `fidelity_options`, `set_fidelity_options`;
  - `with_canonicalization`, `with_db_c14n`, `canonicalization_algorithm`, `canonicalize`
    (DB-C14N/1.0 or nothing);
  - `with_statistics`, `build_with_fidelity` (`DDEXBuilder::build` plus counted statistics);
  - `with_version` / `target_version` (read only by the CLI `build --ddex-version`) and
    `supported_versions`;
  - `test_round_trip_fidelity_with` (a real round trip with the caller's parse function).
- Nothing in it changes what `DDEXBuilder::build` writes. `DDEXBuilder` still holds an unused
  `Builder` (`_inner`).

## Found along the way (not removals)

Bugs in REAL code, noted while reading. None of them is pinned by a test yet, and none was
run.

- **`preflight`:**
  - `validate_upc_checksum` weights digits from the left, which is right only for 13-digit
    codes. It rejects valid 12-digit UPC-A codes (`036000291452`).
  - In `Warn` mode `DDEXBuilder::build` drops preflight *errors* silently; only warnings
    become build warnings (`builder.rs:752-770`).
- **`entity_classifier`:**
  - The fake DDEX whitelist short-circuits external-entity detection (above).
  - `is_safe_entity` ignores `system_id`.
- **`error_sanitizer`:**
  - `SanitizedError`'s `Display` panics when `generate_correlation_ids` is false
    (`&correlation_id[0..8]` on `"none"`, `:226`).
  - The truncation at `:420` slices bytes and can panic on multibyte text.
- **`api_security`:** `create_secure_error_response` puts `request_id` into JSON unescaped
  (`:121`).
- **`security`:** `validate_url`'s `"::1"` check never matches (`host_str()` gives `"[::1]"`).
  Its `172.` check blocks all of `172.x`, and `169.254.x` is not blocked.
- **`streaming`:** `progress_callback_frequency = 0` panics (`% 0`).
- **`fuzz/`:** already does not compile (`DdexVersion::Ern383`, a stale `BuildRequest`
  literal); it is its own workspace and not in CI.
