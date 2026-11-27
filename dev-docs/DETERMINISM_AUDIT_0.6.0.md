# Determinism audit for 0.6.0 (internal)

Working document for the 0.6.0 "deterministic builder output" work. Covers `ddex-builder`,
`ddex-core` and the builder bindings (Node, Python, WASM) on `main` at 0.5.1 (after
`0ca4f0c`). This is an inventory only: nothing here has been changed yet. The tests that pin
the 0.6.0 target are in `packages/ddex-builder/tests/it/determinism_0_6.rs`. Each test names
its row below. The ones that fail today are `#[ignore = "known bug: …"]` and listed in
`KNOWN_ISSUES.md`.

**Since 0.6.0 PR 3a** the update-message generator, the diff module and the presets are
removed (`dev-docs/REMOVALS_0.6.0.md`). Their rows (D9, D10, D11, D15, D16) and allows are
marked "removed in 3a"; their code and the ignored test `update_message_is_byte_identical`
are gone. The line numbers are left as they were.

**Since 0.6.0 PR 3b** the guarantees, the determinism verifier, verification, the fidelity
module, the fake round-trip APIs, the unimplemented canonicalization algorithms and the
twelve `DeterminismConfig` fields no build read are removed. Rows D18 and H4–H7 and their
allows are marked "removed in 3b". D27's `DeterminismConfig::unicode_normalization` field is
gone too (the NFC normalization it described stays in the canonicalizer and ID generator).

**Since 0.6.0 PR 3c** the dead internals (`caching`, `memory_optimization`,
`optimized_strings`, `parallel_processing`, `namespace_minimizer`,
`generator::optimized_xml_writer`, the uncompiled `attribute_normalizer.rs`), the entity
classifier and the error sanitizer are removed. Rows D21, D22, D23, D26, H2 and H8, part of
H3, and their 16 allows are marked "removed in 3c".

Method: I read `builder.rs`, the ERN 3.8 and ERN 4 generators, the canonicalizer, the ID
generator, the update-message, streaming and diff code, and the three binding crates. I
grepped the in-scope crates for clocks (`Utc::now`, `Local::now`, `SystemTime::now`,
`Instant`), randomness (`Uuid::new_v4` / `v7`, `rand`, `getrandom`, `fastrand`), hash
containers (`HashMap`, `HashSet`, `ahash`, `FxHash`, `DashMap`, `DefaultHasher`), `rayon`,
`read_dir`, `env::var`, hostname, pid, pointer formatting and float formatting. Then I
traced each hit back to `DDEXBuilder::build`, the streaming writer, update messages and the
bindings.

"Reached by default" means a plain `DDEXBuilder::build(request, BuildOptions::default())`
(or the plain call of the API in that row) hits it whenever the request leaves the value
out. None of these sources fires when the caller supplies every value. Every existing
determinism test supplies them, which is why those tests pass.

## Summary

- **27 findings, D1–D27.**
- **16 reach DDEX XML or the IDs written into it (D1–D16).**
  - 13 depend on the clock or randomness: D1–D5, D8–D13, D15, D16.
  - 3 are deterministic but wrong: D6, D7, D14. These are invalid references, colliding
    references, and a 32-bit truncated reference hash.
- **Reached by default** (no option, only an omitted value or a plain call):
  - `DDEXBuilder::build`: D1, D2, D8.
  - Update messages (`create_update*`): D10, D11.
  - Streaming writer: D12, D13, D14.
  - Diff changesets: D16.
- **The other 11 (D17–D27) do not reach build XML.** They are report or schema timestamps,
  security logging, an opt-in error sanitizer, internals off the build path, and
  order-stable code.
- **Bindings (B1–B3).**
  - All three always call `BuildOptions::default()` and expose no ID strategy, so they
    inherit D1/D2/D8.
  - Node `addRelease` releases always leave `release_reference` unset, so D2 hits every one.
    So do Python stored releases, and Python DataFrame rows without a `release_reference`
    column.
- **Hash containers**: no `HashMap`/`HashSet` is iterated into XML on the `build` path. They
  appear in public statistics/config fields and in lookup-only internals (H1–H8).

## Findings

| # | file:line | function | affects | reached by default? | public API involved | proposed fix | breaking? |
|---|---|---|---|---|---|---|---|
| D1 | `ddex-builder/src/builder.rs:859` | `DDEXBuilder::generate_uuid_ids` | `MessageId` = `MSG_<uuid v4>` | **yes** (default `IdStrategy::UUID`, `header.message_id: None`) | `BuildRequest.header.message_id`, `BuildOptions::id_strategy`, `IdStrategy::UUID` | Derive the default from the request (content hash), or make a missing `MessageId` an error | yes (output changes; an error variant would reject requests that build today) |
| D2 | `ddex-builder/src/builder.rs:865`, `:871` | `generate_uuid_ids` | `ReleaseReference` `R<uuid>`, `ResourceReference` `A<uuid>` | **yes** (`release_reference` / `resource_reference: None`) | `ReleaseRequest.release_reference`, `TrackRequest.resource_reference` | Number the references in request order (`R1`, `A1`, …) or hash their content | yes (output changes) |
| D3 | `ddex-builder/src/builder.rs:889-896` | `generate_uuidv7_ids` | same as D1/D2 (falls back to v4; "UUID v7 is not implemented") | only with `IdStrategy::UUIDv7` | `IdStrategy::UUIDv7` | Remove the variant, or alias it to the deterministic default | yes if removed (`IdStrategy` is not `#[non_exhaustive]`) |
| D4 | `ddex-builder/src/builder.rs:905` | `generate_sequential_ids` | `MessageId` = `MSG_<Unix seconds>` | only with `IdStrategy::Sequential` | `IdStrategy::Sequential` | Use a fixed or content-derived message ID (references are already numbered) | behaviour only |
| D5 | `ddex-builder/src/builder.rs:964` | `generate_stable_hash_ids` | `MessageId` hash includes `Utc::now().format("%Y%m%d")`, so it changes daily | only with `IdStrategy::StableHash` | `IdStrategy::StableHash`, `StableHashConfig` | Drop the date from the hash inputs | behaviour only |
| D6 | `ddex-builder/src/builder.rs:976-997`, `id_generator.rs` `hash_blake3` / `hash_sha256` | `generate_stable_hash_ids` | generated release/resource references are `B3:<hex>` / `SHA256:<hex>`. The `:` breaks the ERN reference pattern `X[\d\-_a-zA-Z]+`, so the build fails (`generator/shared.rs:273` `is_reference`). Deterministic, but unusable | only with `StableHash` and an omitted reference | `StableHashGenerator` output format | Emit `R`/`A` plus hex for references (keep the prefixed form for `MessageId` if wanted) | behaviour only |
| D7 | `ddex-builder/src/builder.rs:1003-1012` | `generate_stable_hash_ids` | `DealReference` = `DEAL_<model>_<territories>`. Two deals with the same model and territories get the same reference (e.g. `multi_track_album`'s deals 1 and 3). Deterministic, but colliding | only with `StableHash` and `deal_reference: None` | `DealRequest.deal_reference` | Hash every term and the release references, or number them like the other strategies (`D1`, …) | behaviour only |
| D8 | `ddex-builder/src/generator/ern4.rs:490`, `generator/ern38.rs:509` | `message_header` | `MessageCreatedDateTime` = `Utc::now().to_rfc3339()` | **yes** (`message_created_date_time: None`) | `MessageHeaderRequest.message_created_date_time` | Error naming the field (`message_created_date_time`) | yes (requests that build today fail) |
| D9 (**removed in 3a**) | `ddex-builder/src/builder.rs:1311` | `serialize_update_header` | `MessageCreatedDateTime` of a serialized update = now | only for a hand-built `UpdateReleaseMessage` whose header has no time (`create_update` always sets one, from D10) | `DDEXBuilder::serialize_update` | Error, or require the field | yes (error) |
| D10 (**removed in 3a**) | `ddex-builder/src/messages/update_release.rs:867` (also the test helper at `:1134`) | `UpdateGenerator::create_update_metadata` | `UpdateMetadata.update_created_timestamp` = `Utc::now()`. It is copied into the update header's `MessageCreatedDateTime` (`create_update_header`) and written as `UpdateCreatedDateTime` | **yes** (every `create_update*` call; no way to pass a time) | `DDEXBuilder::create_update*`, `UpdateGenerator::create_update`, `UpdateConfig`, `UpdateMetadata` (pub `DateTime<Utc>` fields) | Take the timestamp from the caller (`UpdateConfig` field or a parameter) | yes (new required input, or a new field in a non-`#[non_exhaustive]` struct) |
| D11 (**removed in 3a**) | `ddex-builder/src/messages/update_release.rs:765` | `UpdateGenerator::extract_entity_id` | `UpdateOperation.entity_id` = `unknown_<8 hex of uuid v4>`. Written as `EntityId`, and used as the key of `resource_updates` / `release_updates` / `deal_updates` | **yes**, for every operation: it only reads `Id=` from `DiffPath::to_string()`, and `DiffPath`'s `Display` never writes `Id=` (`diff/types.rs:195`). In addition, `parse_xml_to_ast` (`:684`) wraps the whole document in one text node, so the diff is a single `/NewReleaseMessage/#text` change | `create_update*`, `UpdateOperation.entity_id` | Derive the ID from the path deterministically (or report the entity as unidentified without randomness). A real XML→AST parse is needed for useful updates anyway | behaviour only (the IDs are meaningless today) |
| D12 | `ddex-builder/src/streaming/mod.rs:451` | `StreamingBuilder::write_message_header` | `MessageId` = uuid v4 | **yes** (`header.message_id: None`) | `StreamingBuilder::start_message`, Node `StreamingDdexBuilder.startMessage` | Error naming the field | yes (error) |
| D13 | `ddex-builder/src/streaming/mod.rs:482` | `write_message_header` | `MessageCreatedDateTime` = now | **yes** (`header.message_created_date_time: None`) | same as D12 | Error naming the field | yes (error) |
| D14 | `ddex-builder/src/streaming/reference_manager.rs:189-196`, `:232-238`, `:271-277` | `StreamingReferenceManager::generate_*_reference` | References are SHA-256 of the caller's ID, truncated to 8 hex digits (32 bits). Deterministic, but a collision fails the stream with `InvalidReference`. The birthday bound puts that above 50% at about 77k items. The default `max_cache_size` is 100 000, and the docs pitch the writer for large catalogs | yes (`ReferenceConfig::default().deterministic = true`) | `ReferenceConfig`, `StreamingBuilder::write_resource` / `write_release` | Use more hash digits, or number the references | yes (references change) |
| D15 (**removed in 3a**) | `ddex-builder/src/diff/formatter.rs:405` | `DiffFormatter::generate_update_message` | `MessageId` = uuid v4 when `message_id: None` | only for that function | `DiffFormatter::generate_update_message` | Require the ID, or derive it | yes if it becomes required |
| D16 (**removed in 3a**) | `ddex-builder/src/diff/types.rs:50` | `ChangeSet::new` | `ChangeSet.timestamp` = now. It is written as `MessageCreatedDateTime` by D15's function (`formatter.rs:415`) and serialized with the changeset | yes, for every diff | `ChangeSet.timestamp` (pub `DateTime<Utc>`), `DiffEngine::diff` | Make the timestamp optional / caller-supplied | yes (field type) |
| D17 | `ddex-builder/src/schema/mod.rs:378`, `:401`, `:467`, `:490`, `:513` | `SchemaGenerator::generate_*`, TypeScript/Python code generators | `generated_at` metadata, and a timestamp line in the generated `.ts` / `.py` source | yes, for schema generation (CLI `schema`); not build output | `JsonSchema` / `SchemaGenerator` results (`generated_at: DateTime<Utc>`) | Omit the timestamp from generated source, or make it optional | yes (field type) if changed |
| D18 (**removed in 3b**) | `ddex-builder/src/guarantees.rs:202`, `:541` | `DeterminismGuaranteeValidator`, `generate_guarantee_report` | report timestamps (not build output) | report only | `GuaranteeReport.timestamp` and the validation result's `timestamp` | None needed (a report time is legitimate) | no |
| D19 | `ddex-builder/src/versions/mod.rs:230` | `ConversionResult` | `converted_at: DateTime<Utc>`. It is never assigned in the crate (the struct is not constructed), so nothing reaches output | no | pub field | Remove or make optional with D16 | yes if removed |
| D20 | `ddex-builder/src/security.rs:726`, `:785`, `:842`; `api_security.rs:115` | secure logging, `SecureTempFile`, request IDs | log timestamps, temp-file names, request IDs (not build output) | not on the build path | `SecureTempFile`, `SecurityConfig` | None needed | no |
| D21 (**removed in 3c**) | `ddex-builder/src/security/error_sanitizer.rs:88`, `:358` | `SanitizedErrorTrait::error_id`, `ErrorSanitizer::sanitize` | random correlation IDs in sanitized error text | only if the caller uses `ErrorSanitizer` (`generate_correlation_ids`). `build`, the CLI and the bindings do not | `security::ErrorSanitizer`, `sanitize_*` | None for 0.6.0. Document that error text then carries a random ID | no |
| D22 (**removed in 3c**) | `ddex-builder/src/namespace_minimizer.rs:261-266` | `NamespaceMinimizer` prefix fallback | prefix `ns<DefaultHasher(uri) % 1000>`. Stable today (SipHash with fixed keys), but std does not guarantee `DefaultHasher` output across Rust releases | not on the build path (`NamespaceMinimizer` is pub but unused by `build`) | `namespace_minimizer::NamespaceMinimizer` | Use `sha2`/`blake3` (already dependencies) | no (only if someone relies on the prefix) |
| D23 (**removed in 3c**) | `ddex-builder/src/caching.rs:784` | hash cache fallback | `DefaultHasher` cache key (internal only) | no | `caching` module | None needed (in-process key) | no |
| D24 | `ddex-builder/src/builder.rs:727` (`web_time::Instant`), `lib.rs:755`, bindings' `Instant` | `build`, `build_with_fidelity`, binding stats | `BuildResult.statistics.generation_time_ms`, binding `getStats` times (not XML) | yes, but not in XML | `BuildStatistics` | None needed. Tests must compare `.xml`, not the whole `BuildResult` | no |
| D25 | `ddex-builder/src/canonical/mod.rs:319-327`, `:423` | `DB_C14N::canonicalize` | attribute order (`BTreeMap`, byte order) and child order (`sort_by`, stable) | yes | — | None (order-stable). `canonical_hash` is SHA-256 of the canonical bytes | no |
| D26 (**removed in 3c**) | `ddex-builder/src/parallel_processing.rs` (`par_iter().collect()`), `generator/optimized_xml_writer.rs:294` | `ParallelProcessor`, `vectorized::write_elements_parallel` | result order (rayon's indexed `collect` keeps input order) | not on the build path | `parallel_processing`, `generator::optimized_xml_writer` | None needed | no |
| D27 | `ddex-builder/src/canonical/mod.rs` (`unicode_normalization: NFC`), `id_generator.rs` `normalize_value` | NFC and case folding | canonical text and stable-hash inputs depend on the Unicode tables of `unicode-normalization` / std | yes (canonical output) | `DeterminismConfig::unicode_normalization` | None for 0.6.0. Note that a dependency or toolchain upgrade can change output for unassigned/new code points | no |

Searched for and **not found** in the in-scope crates' library code: `Local::now`,
`SystemTime::now`, `rand` / `fastrand` / `getrandom` calls (only the WASM crate enables
`getrandom`'s `wasm_js` backend for `uuid`), `ahash`, `FxHash`, `DashMap`, `read_dir`,
hostname, process ID, pointer formatting (`as_ptr` only in a unit test), locale-dependent
formatting, and floats in the XML. Environment variables: only `DDEX_QUIET` (CLI logging,
`bin/ddex-builder.rs:1156`) and a test-only `LC_ALL` read (`guarantees.rs:410`).

### Bindings

| # | file:line | what differs from the Rust defaults | affects | proposed fix | breaking? |
|---|---|---|---|---|---|
| B1 | `bindings/node/src/lib.rs:273`, `:312`, `:394` | Always `BuildOptions::default()`. No ID strategy, determinism or preflight option is exposed | inherits D1/D2/D8 | Follow the Rust default, whatever 0.6.0 makes it | follows D1/D2/D8 |
| B1a | `bindings/node/src/lib.rs:564` (`stored_releases`) | Releases from `addRelease`/`addResource` always get `release_reference: None`, so D2 hits every stored release (`R<uuid>`). Resource references are the caller's `resourceId` | `ReleaseReference` | Same as D2 | follows D2 |
| B1b | `bindings/node/src/lib.rs:717-736` (`StreamingDdexBuilder.startMessage`) | Passes `messageId` / `messageCreatedDateTime` through as `Option` | D12/D13 | Same as D12/D13 | follows D12/D13 |
| B2 | `bindings/python/src/lib.rs:583`, `:618`, `:684`, `:827`, `:1000` | Always `BuildOptions::default()` | inherits D1/D2/D8 | as B1 | as B1 |
| B2a | `bindings/python/src/lib.rs:1096` (stored releases), `:1224` (DataFrame rows: `release_reference` column, else `None`) | Stored releases always, and DataFrame rows without a `release_reference` column, hit D2 | `ReleaseReference` | Same as D2 | follows D2 |
| B2b | `bindings/python/src/lib.rs:1194` (`date_cell`), `duration_cell` | Formatting only: RFC 3339 → date part in the value's own offset; seconds → `PTxMyS` via Rust float formatting (platform-independent) | release date / duration text | None needed | no |
| B3 | `bindings/wasm/src/lib.rs:49` | Always `BuildOptions::default()`. uuid v4 comes from `crypto.getRandomValues` (`getrandom` `wasm_js`) and `Utc::now` from `js_sys::Date` (`chrono` `wasmbind`). Same behaviour as native | inherits D1/D2/D8 | as B1 | as B1 |

The Node and Python bindings keep `HashMap<String, String>` `metadata` / `parameters` fields
on their `Release` / `Resource` / preset-rule classes. They are not used in build output.

## Hash containers and the `disallowed_types` lint

Configuration as found at 0.5.1 (PR 1):

- `packages/ddex-builder/clippy.toml`: `disallowed-types = ["std::collections::HashMap",
  "std::collections::HashSet"]`.
- `packages/ddex-builder/.clippy.toml` (a second, older file from the 0.1.0 release) listed
  the same two paths plus bare `"HashMap"` and `"HashSet"`. When both files are in one
  directory, clippy reads `.clippy.toml` and ignores `clippy.toml`. CI said so ("using config
  file `.clippy.toml`, `clippy.toml` will be ignored") and also warned that the bare names
  "do not refer to a reachable type".
- `packages/ddex-builder/Cargo.toml` `[lints.clippy] disallowed_types = "deny"`.
- CI: the `clippy` job passes `-A clippy::disallowed_types` for the whole workspace. The
  `clippy-builder-determinism` job ran `cargo clippy -p ddex-builder --all-targets` with
  `continue-on-error: true`, so the lint was reported (44 errors) but never enforced.
- ddex-core and ddex-parser have no clippy config. The binding crates under
  `packages/ddex-builder/bindings/` pick up the builder's config, because clippy looks for it
  from the crate's directory upwards. They do not deny the lint.
- There were no `#[allow(clippy::disallowed_types)]` attributes or other exemptions.

**Since 0.6.0 PR 2** the guard is enforced:

- `packages/ddex-builder/clippy.toml` is the only config file; `.clippy.toml` is deleted.
- It disallows these types: `std::collections::HashMap`, `std::collections::HashSet`,
  `std::hash::RandomState` and `std::hash::DefaultHasher`.
- It disallows these clock and RNG methods: `chrono::Utc::now` / `today`,
  `chrono::Local::now` / `today`, `std::time::SystemTime::now` and `uuid::Uuid::new_v4`.
  `uuid::Uuid::now_v7` / `new_v7`, `rand::thread_rng` / `rng` / `random` and
  `getrandom::getrandom` / `fill` are also listed, with `allow-invalid`, because not every
  crate that reads the file has them.
- `Cargo.toml` denies `disallowed_types` and `disallowed_methods`.
- The `clippy-builder-determinism` job has no `continue-on-error`, so any new use without a
  justified allow fails it.
- Every existing use has an `#[allow(..., reason = "...")]` naming its row. They are listed
  in [Allow attributes](#allow-attributes-060-pr-2).

Uses in ddex-builder (library code) and whether each one is safe for output:

| # | where | use | safe? |
|---|---|---|---|
| H1 | `canonical/rules.rs:22`, `:169`, `:187` | `HashSet` of reserved / used namespace prefixes | safe: membership only, never iterated (the declarations are an `IndexMap`) |
| H2 (**removed in 3c**) | `attribute_normalizer.rs:52` | `HashMap` of special handlers | safe: the file is not compiled (`lib.rs` has no `mod attribute_normalizer`), so clippy never sees it and it needs no allow |
| H3 (entity classifier and error sanitizer parts **removed in 3c**; `path_validator` remains) | `security/entity_classifier.rs:220`, `security/error_sanitizer.rs:181-182`, `security/path_validator.rs:56`, `:99`, `:547` | caches, code maps, extension allow-lists, visited set | safe: lookup / membership only. But `PathValidatorConfig.allowed_extensions` is a pub `HashSet` (its `Debug` / serde order varies) |
| H4 (**removed in 3b**) | `lib.rs:149` | `CustomCanonicalizationRules.element_ordering: HashMap` (pub field) | not reached: custom canonicalization returns "not implemented". If it is ever implemented, iteration order matters, so make it an `IndexMap` (breaking: pub field type) |
| H5 (**removed in 3b**) | `fidelity.rs:216`, `:289`, `:358-366` | pub `HashMap` fields in `FidelityConfig` / `FidelityStatistics` | not XML. Their serde/`Debug` order varies. IndexMap is breaking (pub field types) |
| H6 (**removed in 3b**) | `verification.rs:524` | `VerificationStatistics.extensions_verified` (pub) | as H5 |
| H7 (**removed in 3b**) | `round_trip.rs:213`, `:226`, `:254`, `:269` | pub `HashMap` fields in `FidelityAnalysis` statistics | as H5 (and `analyze_fidelity` returns an error in 0.5.x) |
| H8 (**removed in 3c**) | `namespace_minimizer.rs:33`, `:99` | `NamespaceAnalysis.namespace_elements: HashMap` (pub), iterated by the minimizer | not on the build path. If the minimizer is used, its prefix choices may depend on iteration order. Make it an `IndexMap` (breaking: pub field) |

ddex-core, outside the lint: `models/extensions.rs:119` `Extensions.legacy_data: HashMap`
(`#[serde(flatten)]`, so serialized key order varies), `models/graph/message.rs:26`
`legacy_extensions: Option<HashMap<..>>`, and `models/validation.rs:216-218` (validator
lookups). The builder's generators never write `BuildRequest.extensions` or these core
fields, so none of them reaches builder XML. They do reach the parser's JSON output and the
bindings' serialized messages.

## Allow attributes (0.6.0 PR 2)

Every remaining use of a disallowed type or method has an allow on the narrowest item that
contains it (a `use`, a field, a `static` or a fn; never a module or the crate). The reasons
come in two kinds:

- `known bug D<n>: removed in 0.6.0 determinism PR` for uses that reach output.
- `D<n>/H<n>: <why it cannot affect build output>` for the rest.

Test and example code gets `test only:` / `example only:` reasons.

A field-level allow does not cover the impls that serde's derive generates: they are separate
items, so the lint fires there at the field's span. The pub `HashMap` fields of serde-derived
structs (H4–H7: `lib.rs`, `fidelity.rs`, `round_trip.rs`, `verification.rs`) therefore use a
private alias, `type UnorderedMap<K, V> = HashMap<K, V>;`, defined once per file with the
allow. It is the same type, so the fields and the public API are unchanged. The line numbers in the
Findings and Hash containers tables above are as of 0.5.1; this table has the current ones.

| allow (file:line) | lint | row | reason | on |
|---|---|---|---|---|
| `packages/ddex-builder/examples/spotify_album_example.rs:191` | `disallowed_methods` | test/example | example only: stamps the sample request with the current time | `fn create_spotify_album_request() -> BuildRequest {` |
| `packages/ddex-builder/src/api_security.rs:106` | `disallowed_methods` | D20 | D20: error-response timestamp; not build output | `pub fn create_secure_error_response(&self, error: &BuildErro` |
| `packages/ddex-builder/src/builder.rs:851` | `disallowed_methods` | D1, D2 | known bug D1, D2: removed in 0.6.0 determinism PR | `fn generate_uuid_ids(` |
| `packages/ddex-builder/src/builder.rs:903` | `disallowed_methods` | D4 | known bug D4: removed in 0.6.0 determinism PR | `fn generate_sequential_ids(` |
| `packages/ddex-builder/src/builder.rs:943` | `disallowed_methods` | D5 | known bug D5: removed in 0.6.0 determinism PR | `fn generate_stable_hash_ids(` |
| `packages/ddex-builder/src/builder.rs:1282` (**removed in 3a**) | `disallowed_methods` | D9 | known bug D9: removed in 0.6.0 determinism PR | `fn serialize_update_header(` |
| `packages/ddex-builder/src/caching.rs:759` (**removed in 3c**) | `disallowed_types` | D23 | D23: in-process cache key; never written to output | `pub fn fast_hash<T: Hash + std::fmt::Debug>(` |
| `packages/ddex-builder/src/canonical/rules.rs:9` | `disallowed_types` | H1 | H1: membership checks only, never iterated; the declarations are an IndexMap | `use std::collections::HashSet;` |
| `packages/ddex-builder/src/canonical/rules.rs:26` | `disallowed_types` | H1 | H1: membership checks only, never iterated; the declarations are an IndexMap | `reserved_prefixes: HashSet<String>,` |
| `packages/ddex-builder/src/canonical/rules.rs:35` | `disallowed_types` | H1 | H1: membership checks only, never iterated; the declarations are an IndexMap | `pub fn new() -> Self {` |
| `packages/ddex-builder/src/canonical/rules.rs:158` | `disallowed_types` | H1 | H1: membership checks only, never iterated; the declarations are an IndexMap | `pub fn deduplicate_prefixes(` |
| `packages/ddex-builder/src/canonical/rules.rs:203` | `disallowed_types` | H1 | H1: membership checks only, never iterated; the declarations are an IndexMap | `fn generate_unique_prefix(&self, base_prefix: &str, used_pre` |
| `packages/ddex-builder/src/diff/formatter.rs:389` (**removed in 3a**) | `disallowed_methods` | D15 | known bug D15: removed in 0.6.0 determinism PR | `pub fn generate_update_message(` |
| `packages/ddex-builder/src/diff/types.rs:45` (**removed in 3a**) | `disallowed_methods` | D16 | known bug D16: removed in 0.6.0 determinism PR | `pub fn new() -> Self {` |
| `packages/ddex-builder/src/fidelity.rs:9` (**removed in 3b**) | `disallowed_types` | H5 | H5: fidelity config/statistics; not build output | `use std::collections::HashMap;` |
| `packages/ddex-builder/src/fidelity.rs:17` (**removed in 3b**) | `disallowed_types` | H5 | H5: fidelity config/statistics; not build output | `type UnorderedMap<K, V> = HashMap<K, V>;` |
| `packages/ddex-builder/src/fidelity.rs:232` (**removed in 3b**) | `disallowed_types` | H5 | H5: fidelity config/statistics; not build output | `fn default() -> Self {` |
| `packages/ddex-builder/src/fidelity.rs:392` (**removed in 3b**) | `disallowed_types` | H5 | H5: fidelity config/statistics; not build output | `fn default() -> Self {` |
| `packages/ddex-builder/src/generator/ern38.rs:494` | `disallowed_methods` | D8 | known bug D8: removed in 0.6.0 determinism PR | `fn message_header(request: &BuildRequest) -> Element {` |
| `packages/ddex-builder/src/generator/ern4.rs:464` | `disallowed_methods` | D8 | known bug D8: removed in 0.6.0 determinism PR | `fn message_header(request: &BuildRequest) -> Element {` |
| `packages/ddex-builder/src/guarantees.rs:197` (**removed in 3b**) | `disallowed_methods` | D18 | D18: report timestamp; not build output | `pub fn validate_guarantee(` |
| `packages/ddex-builder/src/guarantees.rs:529` (**removed in 3b**) | `disallowed_methods` | D18 | D18: report timestamp; not build output | `pub fn generate_guarantee_report(` |
| `packages/ddex-builder/src/lib.rs:66` (**removed in 3b**) | `disallowed_types` | H4 | H4: custom canonicalization is not implemented (returns an error), so this map is never read | `use std::collections::HashMap;` |
| `packages/ddex-builder/src/lib.rs:74` (**removed in 3b**) | `disallowed_types` | H4 | H4: custom canonicalization is not implemented (returns an error), so this map is never read | `type UnorderedMap<K, V> = HashMap<K, V>;` |
| `packages/ddex-builder/src/lib.rs:165` (**removed in 3b**) | `disallowed_types` | H4 | H4: custom canonicalization is not implemented (returns an error), so this map is never read | `fn default() -> Self {` |
| `packages/ddex-builder/src/messages/update_release.rs:754` (**removed in 3a**) | `disallowed_methods` | D11 | known bug D11: removed in 0.6.0 determinism PR | `fn extract_entity_id(&self, path: &crate::diff::types::DiffP` |
| `packages/ddex-builder/src/messages/update_release.rs:861` (**removed in 3a**) | `disallowed_methods` | D10 | known bug D10: removed in 0.6.0 determinism PR | `fn create_update_metadata(` |
| `packages/ddex-builder/src/messages/update_release.rs:1096` (**removed in 3a**) | `disallowed_methods` | D10 | D10: unit-test fixture timestamp; not build output | `fn test_operation_validation() {` |
| `packages/ddex-builder/src/namespace_minimizer.rs:11` (**removed in 3c**) | `disallowed_types` | H8 | H8: NamespaceMinimizer is not on the build path | `use std::collections::HashMap;` |
| `packages/ddex-builder/src/namespace_minimizer.rs:37` (**removed in 3c**) | `disallowed_types` | H8 | H8: NamespaceMinimizer is not on the build path | `pub namespace_elements: HashMap<String, IndexSet<String>>,` |
| `packages/ddex-builder/src/namespace_minimizer.rs:105` (**removed in 3c**) | `disallowed_types` | H8 | H8: NamespaceMinimizer is not on the build path | `fn analyze_namespace_usage(&self, ast: &AST) -> Result<Names` |
| `packages/ddex-builder/src/namespace_minimizer.rs:144` (**removed in 3c**) | `disallowed_types` | H8 | H8: NamespaceMinimizer is not on the build path | `fn analyze_element_usage(` |
| `packages/ddex-builder/src/namespace_minimizer.rs:265` (**removed in 3c**) | `disallowed_types` | D22 | D22: NamespaceMinimizer is not on the build path | `fn generate_prefix_for_uri(&self, uri: &str) -> String {` |
| `packages/ddex-builder/src/namespace_minimizer.rs:478` (**removed in 3c**) | `disallowed_types` | H8 | H8: NamespaceMinimizer is not on the build path | `fn test_minimal_declarations() {` |
| `packages/ddex-builder/src/round_trip.rs:24` (**removed in 3b**) | `disallowed_types` | H7 | H7: round-trip statistics; not build output | `type UnorderedMap<K, V> = std::collections::HashMap<K, V>;` |
| `packages/ddex-builder/src/schema/mod.rs:369` | `disallowed_methods` | D17 | D17: schema metadata timestamp; not DDEX build output | `pub fn generate_build_request_schema(&self) -> Result<Schema` |
| `packages/ddex-builder/src/schema/mod.rs:396` | `disallowed_methods` | D17 | D17: schema metadata timestamp; not DDEX build output | `pub fn generate_flat_release_schema(&self) -> Result<SchemaG` |
| `packages/ddex-builder/src/schema/mod.rs:423` | `disallowed_methods` | D17 | D17: schema metadata timestamp; not DDEX build output | `pub fn generate_complete_schema(&self) -> Result<SchemaGener` |
| `packages/ddex-builder/src/schema/mod.rs:493` | `disallowed_methods` | D17 | D17: timestamp in generated TypeScript; not DDEX build output | `pub fn generate_typescript_types(&self, schema: &JsonSchema)` |
| `packages/ddex-builder/src/schema/mod.rs:520` | `disallowed_methods` | D17 | D17: timestamp in generated Python; not DDEX build output | `pub fn generate_python_types(&self, schema: &JsonSchema) -> ` |
| `packages/ddex-builder/src/security.rs:720` | `disallowed_methods` | D20 | D20: log line timestamp; not build output | `pub fn create_secure_log_message(` |
| `packages/ddex-builder/src/security.rs:783` | `disallowed_methods` | D20 | D20: random temp-file name; not build output | `pub fn new() -> Result<Self, BuildError> {` |
| `packages/ddex-builder/src/security.rs:849` | `disallowed_methods` | D20 | D20: random secure ID for callers; not build output | `pub fn generate_secure_id() -> String {` |
| `packages/ddex-builder/src/security/entity_classifier.rs:220` (**removed in 3c**) | `disallowed_types` | H3 | H3: lookup-only entity cache; never iterated into output | `entity_cache: HashMap<String, EntityClass>,` |
| `packages/ddex-builder/src/security/entity_classifier.rs:235` (**removed in 3c**) | `disallowed_types` | H3 | H3: lookup-only entity cache; never iterated into output | `pub fn with_config(config: ClassifierConfig) -> Self {` |
| `packages/ddex-builder/src/security/error_sanitizer.rs:10` (**removed in 3c**) | `disallowed_types` | H3 | H3: lookup-only error-code map and correlation store; never iterated into output | `use std::collections::HashMap;` |
| `packages/ddex-builder/src/security/error_sanitizer.rs:91` (**removed in 3c**) | `disallowed_methods` | D21 | D21: opt-in error correlation ID; build() does not use ErrorSanitizer | `fn error_id(&self) -> String {` |
| `packages/ddex-builder/src/security/error_sanitizer.rs:189` (**removed in 3c**) | `disallowed_types` | H3 | H3: lookup-only error-code map and correlation store; never iterated into output | `error_code_map: HashMap<ErrorContext, &'static str>,` |
| `packages/ddex-builder/src/security/error_sanitizer.rs:194` (**removed in 3c**) | `disallowed_types` | H3 | H3: lookup-only error-code map and correlation store; never iterated into output | `correlation_store: HashMap<String, String>,` |
| `packages/ddex-builder/src/security/error_sanitizer.rs:352` (**removed in 3c**) | `disallowed_types` | H3 | H3: lookup-only error-code map and correlation store; never iterated into output | `pub fn with_config(config: SanitizerConfig) -> Self {` |
| `packages/ddex-builder/src/security/error_sanitizer.rs:373` (**removed in 3c**) | `disallowed_methods` | D21 | D21: opt-in error correlation ID; build() does not use ErrorSanitizer | `pub fn sanitize<E>(&mut self, error: E, context: ErrorContex` |
| `packages/ddex-builder/src/security/error_sanitizer.rs:516` (**removed in 3c**) | `disallowed_types` | H3 | H3: lookup-only error-code map and correlation store; never iterated into output | `fn create_error_code_map() -> HashMap<ErrorContext, &'static` |
| `packages/ddex-builder/src/security/path_validator.rs:21` | `disallowed_types` | H3 | H3: membership-only sets for path validation; not on the build path | `use std::collections::HashSet;` |
| `packages/ddex-builder/src/security/path_validator.rs:60` | `disallowed_types` | H3 | H3: membership-only sets for path validation; not on the build path | `static WINDOWS_RESERVED_NAMES: Lazy<HashSet<&str>> = Lazy::n` |
| `packages/ddex-builder/src/security/path_validator.rs:107` | `disallowed_types` | H3 | H3: membership-only sets for path validation; not on the build path | `pub allowed_extensions: HashSet<String>,` |
| `packages/ddex-builder/src/security/path_validator.rs:117` | `disallowed_types` | H3 | H3: membership-only sets for path validation; not on the build path | `fn default() -> Self {` |
| `packages/ddex-builder/src/security/path_validator.rs:534` | `disallowed_types` | H3 | H3: membership-only sets for path validation; not on the build path | `fn validate_symlinks(` |
| `packages/ddex-builder/src/streaming/mod.rs:449` | `disallowed_methods` | D12, D13 | known bug D12, D13: removed in 0.6.0 determinism PR | `fn write_message_header(&mut self, header: &MessageHeaderReq` |
| `packages/ddex-builder/src/verification.rs:8` (**removed in 3b**) | `disallowed_types` | H6 | H6: verification statistics; not build output | `use std::collections::HashMap;` |
| `packages/ddex-builder/src/verification.rs:16` (**removed in 3b**) | `disallowed_types` | H6 | H6: verification statistics; not build output | `type UnorderedMap<K, V> = HashMap<K, V>;` |
| `packages/ddex-builder/src/verification.rs:542` (**removed in 3b**) | `disallowed_types` | H6 | H6: verification statistics; not build output | `fn default() -> Self {` |
| `packages/ddex-builder/tests/it/determinism.rs:27` | `disallowed_methods` | test/example | test only: the request is built once and cloned, so its timestamp is the same in every build | `fn create_test_build_request_base() -> BuildRequest {` |
| `packages/ddex-builder/tests/it/determinism.rs:92` | `disallowed_methods` | test/example | test only: the request is built once and cloned, so its timestamps are the same in every build | `fn create_complex_build_request_base() -> BuildRequest {` |
| `packages/ddex-builder/tests/it/determinism_0_6.rs:69` | `disallowed_methods` | test/example | test only: reads the clock to check the MessageId is not time-derived | `fn assert_not_time_derived(id: &str) {` |
| `packages/ddex-builder/tests/it/determinism_0_6.rs:210` | `disallowed_methods` | test/example | test only: reads the clock to check the MessageId is not time-derived | `fn stable_hash_message_id_is_not_time_derived() {` |

## Facts for later PRs

Reported as found. Nothing has been changed.

**IdStrategy** (`ddex-builder/src/builder.rs:606`, `#[derive(Debug, Clone, Copy, PartialEq,
Eq, Serialize, Deserialize)]`, not `#[non_exhaustive]`):

- `UUID` is **the default** (`BuildOptions::default()`, `builder.rs:593-600`). It gives
  `MSG_<uuid v4>`, `R<uuid simple>` and `A<uuid simple>`; deals get `D<n>`.
- `UUIDv7` is identical to `UUID` (v4 fallback, `TODO: Implement proper UUID v7`).
- `Sequential` gives `MSG_<Unix seconds>`, `R<n>`, `A<release_idx*1000 + track_idx + 1>` and
  `D<n>`. The resource numbering collides once a release has more than 1000 tracks (release
  0 track 1001 and release 1 track 1 are both `A1001`).
- `StableHash` gives a `MessageId` from BLAKE3 (default) or SHA-256 over sender name,
  recipient name, `"MessageHeader"` and today's date. References are `B3:<hex>` /
  `SHA256:<hex>`, which are invalid ERN references (D6). Deals get
  `DEAL_<model>_<territories>` (D7). `StableHashConfig::default()` is `recipe "v1"`,
  `Blake3`, cache on, no salt.
- IDs are generated only for values the request leaves as `None`. Supplied values are never
  changed.

**Build-request fields** (`ddex-builder/src/builder.rs`):

- `MessageHeaderRequest.message_id: Option<String>` (`:120`). `None` is filled in by the ID
  strategy (`DDEXBuilder::generate_ids`, `builder.rs:828`, called after preflight and
  before AST generation). If it is still `None` there (it cannot be), the generators write
  `""` (`ern4.rs:469`, `ern38.rs:498` `unwrap_or_default`). ERN 3.8.2 reports an empty
  `MessageId` as missing (KNOWN_ISSUES).
- `MessageHeaderRequest.message_created_date_time: Option<String>` (`:128`). `None` is
  filled with `Utc::now().to_rfc3339()` inside the generators (`ern4.rs:487-490`,
  `ern38.rs:506-509`), not in `builder.rs`. The value is checked as `xs:dateTime`
  (`shared.rs` `is_xs_date_time`).
- `MessageHeaderRequest.message_sender` / `message_recipient: PartyRequest` (not `Option`):
  `party_name: Vec<LocalizedStringRequest>`, `party_id: Option<String>`,
  `party_reference: Option<String>`. No defaults are applied. The ERN 4 generator writes
  `PartyId` as `party_id.unwrap_or_default()` (`ern4.rs:478`) and the checker reports a
  missing or non-DPID ID. `generate_stable_hash_ids` reads `party_name.first()` with
  `unwrap_or_default()`.
- `MessageHeaderRequest.message_control_type: Option<String>`: written only when `Some`.
- Update messages (**removed in 3a**): `DDEXBuilder::create_update(original_xml, updated_xml,
  original_message_id: &str)`. The header is built by `UpdateGenerator::create_update_header`
  (`update_release.rs:876`): `message_id = Some("UPD-<original_message_id>-<seq:04>")`
  (`update_sequence` is always 1), sender "DDEX Builder Update Engine", recipient "Update
  Recipient", `message_control_type "UpdateMessage"`, and `message_created_date_time =
  Some(update_created_timestamp.to_rfc3339())`, which is now (D10).
  `UpdateOperation.operation_id` is `OP<counter:06>` (per generator, deterministic);
  `entity_id` is D11. `UpdateMetadata.update_created_timestamp: DateTime<Utc>` and
  `original_message_timestamp: Option<DateTime<Utc>>` are pub `chrono` fields.
  `UpdateConfig::default()` excludes `MessageId` and `MessageCreatedDateTime` from diffs.
- Streaming: `StreamingBuilder::start_message(&MessageHeaderRequest, version)`. The same
  `Option` fields are filled in `write_message_header` (D12, D13).

**Third-party types in the public API of the published crates** (by reading and grep; not
tool-checked):

- `chrono`: **yes, widely.**
  - ddex-core: pub `DateTime<Utc>` fields in `models::flat` (`message.rs:43`,
    `deal.rs:23-61`, `release.rs:33-34`), `models::graph` (`header.rs:15`, `message.rs:53`,
    `release.rs:130`, `deal.rs:18-32`), `models::versions::{ern_382, ern_42, ern_43,
    common}` and `models::streaming_types.rs:386`.
  - ddex-parser: re-exposes those models. `bench_report::BenchReport.timestamp` exists only
    with the `bench` feature.
  - ddex-builder: `messages::UpdateMetadata` (`update_release.rs:484`, `:487`) and
    `diff::types::ChangeSet.timestamp` (`:40`) (both removed in 3a), `guarantees` (`:154`, `:554`),
    `versions::ConversionResult.converted_at` (`mod.rs:230`), `versions` `pub fn
    get_version_release_date(..) -> chrono::NaiveDate` (`mod.rs:847`),
    `schema::*.generated_at` (`mod.rs:302`).
- `smartstring`: **yes** (ddex-builder). `optimized_strings::FastString = SmartString<LazyCompact>`
  (pub type alias, `:15`), and the pub enum variant `OptimizedString::Small(FastString)`
  (`:169`). `OptimizedString::Atom(DefaultAtom)` also exposes `string_cache`.
- `serde_yaml`: **no.** It is used only by the CLIs (`ddex-builder/src/bin/ddex-builder.rs`,
  `ddex-parser/src/cli.rs`; `cli` is not a library module).
- `indicatif`: **no.** Only `ddex-parser/src/cli.rs` (CLI).
- `quick-xml`: **yes.**
  - ddex-builder: `impl From<quick_xml::Error> for BuildError` (`error.rs:100`),
    `streaming::StreamingError::XmlError(#[from] quick_xml::Error)` (`streaming/mod.rs:651`),
    `security::SecureXmlReader::read_event(..) -> Result<quick_xml::events::Event<'a>, _>`
    (`security.rs:193`) and `into_inner(self) -> quick_xml::Reader<R>` (`:262`).
  - ddex-parser: `impl From<quick_xml::Error>` and `From<quick_xml::events::attributes::AttrError>`
    for `ParseError` (`error.rs:139`, `:150`), `utf8_utils::handle_text_node(&BytesText, ..)`
    (pub mod, `utf8_utils.rs:31`), and `parser::extension_capture` `process_extension_start_tag(&BytesStart)`
    / `process_extension_text(&BytesText)` (`extension_capture.rs:140`, `:190`).
  - ddex-core: none found.

**Public request/options types in ddex-builder and `#[non_exhaustive]`**: none of the
in-scope crates uses `#[non_exhaustive]` anywhere. The request/options types, all exhaustive:

- `builder.rs`:
  - structs: `BuildRequest`, `MessageHeaderRequest`, `PartyRequest`,
    `LocalizedStringRequest`, `AvsValueRequest`, `ProprietaryIdRequest`, `ReleaseIdRequest`,
    `ResourceGroupRequest`, `ResourceGroupContentItemRequest`, `TrackReleaseRequest`,
    `ReleaseRequest`, `TrackRequest`, `DealRequest`, `DealTerms`, `BuildOptions`,
    `BuildResult`, `BuildWarning`, `BuildStatistics`;
  - enums: `ResourceKind`, `IdStrategy`.
  - All request structs derive `Default` except `BuildOptions`, which has a manual `Default`.
- `determinism.rs`:
  - structs: `DeterminismConfig`;
  - enums: `CanonMode`, `SortStrategy`, `NamespaceStrategy`, `OutputMode`, `LineEnding`,
    `IndentChar`, `UnicodeNormalization`, `XmlCharacterPolicy`, `QuoteStyle`,
    `TimeZonePolicy`, `DateTimeFormat`.
- `id_generator.rs`:
  - structs: `StableHashConfig`, `HashRecipe`, `NormalizeOptions`;
  - enums: `HashAlgorithm`, `UnicodeForm`, `CaseNormalization`.
- `preflight.rs`:
  - structs: `ValidationConfig`;
  - enums: `PreflightLevel`.
- `streaming/mod.rs`, `streaming/reference_manager.rs`:
  - structs: `StreamingConfig`, `ReferenceConfig`.
- `messages/update_release.rs`:
  - structs: `UpdateConfig`;
  - enums: `UpdateAction`, `EntityType`, `ValidationStatus`.
- `lib.rs`:
  - structs: `FidelityOptions`, `CustomCanonicalizationRules`, `VerificationConfig`;
  - enums: `CanonicalizationAlgorithm`.

**Presets**: `presets::all_presets()` has 7: `audio_album`, `audio_single`, `video_single`,
`compilation` (`presets/generic.rs:262-265`) and `youtube_album`, `youtube_video`,
`youtube_single` (`presets/youtube.rs:228-230`). **None changes output today.** Every one
sets `determinism: DeterminismConfig::default()`. `Builder::apply_preset` (`lib.rs:552`)
only stores that config in `Builder.config`, which no build path reads.
`build_with_fidelity` uses `BuildOptions::default()`, and `DDEXBuilder` (`builder.rs`) has
no preset API. The binding `apply_preset` methods only check that the name exists. The
rustdoc on `apply_preset` still lists `universal_basic` and `sony_enhanced`, which do not
exist.

## Other things noticed (not determinism)

- The streaming writer declares `xmlns="http://ddex.net/xml/ern/43"` for every version it
  accepts (only 3.8.x). This is already in KNOWN_ISSUES.
- `serialize_update` and `DiffFormatter::generate_update_message` also hard-coded the
  `ern/43` namespace (removed in 3a).
- StableHash release IDs always hash `"Album"` as the release type
  (`builder.rs:974`), whatever `release_type` says.
