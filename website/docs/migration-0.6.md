---
sidebar_position: 11
title: Migrating to 0.6.0
description: The breaking changes in 0.6.0 and what to change when you upgrade from 0.5.x, for Rust, Node.js, Python, WASM and the CLI.
---

# Migrating to 0.6.0

0.6.0 removes the APIs that returned no real results (invented values, placeholder documents, constant scores) or whose options
had no effect, folds the builder into one type with one options type, and makes the
builder's inputs explicit. Each section below is for one audience; the complete list, with
who is affected by each change, is the Breaking section of the
[changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md).

A removed API that returned no real results has **no replacement**: code that called it got
nothing it could rely on. Where a real alternative exists, it is named.

## Rust: ddex-builder

### `MessageId` and `MessageCreatedDateTime` are required

`MessageHeaderRequest::message_id` and `message_created_date_time` are `String`. A request
without them used to get a random UUID and the current time; now an empty value fails the
build, naming the field (`ERN 4.3 requires MessageHeader/MessageId (header.message_id)`).
The builder reads no clock and no random source on any build path.

```text
// 0.5.x
MessageHeaderRequest { message_id: None, message_created_date_time: None, .. }
```

Supply both values (`message_created_date_time` is an `xs:dateTime`):

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
            message_control_type: None,
            message_created_date_time: "2024-01-01T00:00:00Z".to_string(),
        },
        version: "4.3".to_string(),
        releases: vec![ReleaseRequest {
            release_id: Some("A1-ABCDE-0000000001-M".to_string()),
            title: text("Example Single"),
            // Absent values are None; Some("") fails the build
            release_date: None,
            artist: Some("Example Artist".to_string()),
            label: Some("Example Records".to_string()),
            upc: Some("123456789012".to_string()),
            release_type: Some("Single".into()),
            genre: Some("Pop".to_string()),
            parental_warning_type: Some("NotExplicit".to_string()),
            tracks: vec![TrackRequest {
                track_id: "TRK001".to_string(),
                isrc: Some("ZZ0000000001".to_string()),
                title: "Example Track".to_string(),
                duration: "PT3M0S".to_string(),
                artist: Some("Example Artist".to_string()),
                resource_type: Some("MusicalWorkSoundRecording".into()),
                parental_warning_type: Some("NotExplicit".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        }],
        deals: vec![],
        extensions: None,
        profile: None,
        language_and_script_code: Some("en".to_string()),
        avs_version_id: Some("12".to_string()),
    };

    let result = DDEXBuilder::new().build(request, BuildOptions::default())?;
    assert!(result.xml.contains("<MessageId>MSG-001</MessageId>"));
    // References the request leaves out are numbered in request order
    assert!(result.xml.contains("<ReleaseReference>R1</ReleaseReference>"));
    Ok(())
}
```

References the request leaves out are numbered in request order, one counter per kind per
message: `R1`, `R2`, ... for releases, `A1`, `A2`, ... for resources and `D1`, `D2`, ... for
deals (they were `R<uuid>` / `A<uuid>`); a number the request already uses is skipped.
`IdStrategy`, `BuildOptions::id_strategy`, `stable_hash_config` and the `id_generator`
module were removed: supply references, or rely on the sequential ones. The streaming
writer's references are now 128 bits, and its default resource prefix is `A`.

### `Option` instead of `""` for absent values

These request fields are `Option<String>`: `ReleaseRequest::release_id` and `artist`,
`TrackRequest::isrc` and `artist`, and `DealTerms::commercial_model_type`. `None` writes
nothing; `Some("")` fails the build ("is empty; leave it out (None / null) when there is no
value"). An empty release title text also fails now.

```text
// 0.5.x                         // 0.6.0
release_id: String::new(),       release_id: None,
isrc: "ZZ0000000001".into(),     isrc: Some("ZZ0000000001".into()),
```

### `Builder` is folded into `DDEXBuilder`, with one `BuildOptions`

| 0.5.x | 0.6.0 |
|---|---|
| `ddex_builder::Builder` | `ddex_builder::DDEXBuilder` (a unit struct, `Copy` and `Default`) |
| `Builder::build_with_fidelity(&request)` | `DDEXBuilder::build_with_statistics(request, options)` |
| `Builder::test_round_trip_fidelity_with(xml, parse)` | `DDEXBuilder::check_round_trip(xml, parse)` |
| `Builder::canonicalize(xml)` | `DDEXBuilder::canonicalize(xml, &options)` |
| `Builder::supported_versions()` | `DDEXBuilder::supported_versions()` |
| `Builder::with_version` / `target_version` | `BuildRequest::version` (the target version was never applied) |
| `FidelityOptions` | `BuildOptions` (`canonicalization`, `collect_statistics`) |
| `FidelityBuildResult` | `StatisticsBuildResult` (`build`, `statistics`) |
| `BuildOptions::determinism: Option<DeterminismConfig>` | `BuildOptions::output: OutputConfig` |
| `ddex_builder::determinism` / `DeterminismConfig` | `ddex_builder::output` / `OutputConfig` |
| `emit_reproducibility_banner` | `emit_generator_banner` |
| `BuildResult::reproducibility_banner` | `BuildResult::generator_banner` |
| `CanonMode::Compact` | `CanonMode::Pretty` (they wrote the same output) |

```text
// 0.5.x
let mut builder = Builder::new();
builder.with_canonicalization(CanonicalizationAlgorithm::DbC14N);
let result = builder.build_with_fidelity(&request)?;
let options = BuildOptions { determinism: Some(config), ..Default::default() };
```

```rust
use ddex_builder::{BuildOptions, BuildRequest, CanonMode, DDEXBuilder, OutputConfig, PreflightLevel};
use ddex_parser::DDEXParser;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = std::fs::read_to_string("release.xml")?;
    let parsed = DDEXParser::new().parse(Cursor::new(xml.as_bytes()))?;
    let request = BuildRequest::from_parsed(&parsed).request;
    let builder = DDEXBuilder::new();

    // build_with_fidelity → build_with_statistics; determinism → output
    let options = BuildOptions {
        output: OutputConfig { emit_generator_banner: true, ..Default::default() },
        collect_statistics: true,
        ..Default::default()
    };
    let result = builder.build_with_statistics(request.clone(), options)?;
    let statistics = result.statistics.expect("collect_statistics is set");
    println!("{} elements, banner {:?}", statistics.element_count, result.build.generator_banner);

    // Compact → Pretty: the generator's output without DB-C14N
    let pretty = BuildOptions {
        output: OutputConfig { canon_mode: CanonMode::Pretty, ..Default::default() },
        ..Default::default()
    };
    builder.build(request.clone(), pretty)?;

    // canonicalize takes the options (canonicalization: DbC14N by default, or None)
    let canonical = builder.canonicalize(&result.build.xml, &BuildOptions::default())?;
    println!("{} bytes canonical", canonical.len());

    // test_round_trip_fidelity_with → check_round_trip, with the parser's parse function
    let round_trip = builder.check_round_trip(&xml, |x| {
        DDEXParser::new()
            .parse(Cursor::new(x.as_bytes()))
            .map_err(|e| e.to_string())
    })?;
    println!("round trip: {} difference(s)", round_trip.differences.len());

    // New in 0.6.0: the preflight checks without building
    let preflight = builder.preflight(&request, PreflightLevel::Strict)?;
    println!("preflight: {} error(s)", preflight.errors.len());
    Ok(())
}
```

With the default `PreflightLevel::Warn`, preflight errors (for example `INVALID_UPC`) are now
returned in `BuildResult::warnings` instead of being discarded; `Strict` fails the build.
The UPC / EAN check digit is the GS1 mod-10 check, which accepts valid 12- and 14-digit codes
it used to reject (and rejects wrong ones it used to accept).

### Parse, modify, build

New in 0.6.0, not a replacement: `BuildRequest::from_parsed(&parsed)` returns
`FromParsed { request, unmapped }`, a build request for the same ERN version and the dotted
path of every parsed value the request cannot hold. See the
[building quick start](./quick-start-building.mdx#parse-modify-build).
`round_trip::request_from_parsed` uses it.

### Removed, with no replacement

Each of these returned no real results, or had settings that no build read:

- **Update messages**: `create_update*`, `apply_update`, `validate_update`,
  `serialize_update` and the `messages` module. Send a full `NewReleaseMessage` instead.
- **Semantic diff**: `diff_xml*`, `diff_request_with_xml` and the `diff` module.
- **Presets**: the `presets` module and the `Builder` preset methods. Set the fields in the
  request. `DdexVersion` is now `versions::DdexVersion` (still `ddex_builder::DdexVersion`);
  `MessageProfile` is `schema::MessageProfile` (also `ddex_builder::MessageProfile`).
- **Determinism guarantees and verifier**: the `guarantees` module, `DeterminismVerifier`,
  `BuildError::DeterminismGuaranteeViolated` / `DeterminismFailed`. To compare builds, build
  twice and compare the bytes.
- **Verification and fidelity**: the `verification` and `fidelity` modules, the
  perfect-fidelity methods, `RoundTripTester::test_round_trip` / `analyze_fidelity`
  (`RoundTripTester::new()` takes no argument), and the `FidelityOptions` fields other than
  `canonicalization` and `collect_statistics`. For a real round trip use `check_round_trip`
  or `round_trip::verify_round_trip`; for XSD and business-rule validation use
  [DDEX Workbench](https://ddex-workbench.org).
- **Canonicalization algorithms**: `C14N`, `C14N11` and `Custom` (each returned an error).
  `CanonicalizationAlgorithm` is `DbC14N` (the default) or `None`.
- **Version conversion and detection**: `VersionConverter`, `VersionManager`,
  `convert_version` and the spec tables. Build the target version from a request; the parser
  reads a message's version.
- **Schema generation**: `SchemaGenerator` and the JSON Schema / TypeScript / Python
  generators. Use the documented request types.
- **Unused modules and security items**: `caching`, `memory_optimization`,
  `optimized_strings`, `parallel_processing`, `namespace_minimizer`, the entity classifier,
  the error sanitizer, and the `SecurityConfig` / `ApiSecurityConfig` fields nothing read.
- **Fields nothing read**: the removed `DeterminismConfig` fields (now `OutputConfig`), the
  `StreamingConfig` and `LinkerConfig` no-ops, `LinkingReport::validation_passed` (read
  `warnings`), `preflight::ValidationConfig::validate_checksums` / `validate_dates`, and the
  `DeterminismConfig` argument of `DB_C14N::new` / `with_version`. Drop them;
  `..Default::default()` keeps working.
- **quick-xml in the API**: `From<quick_xml::Error> for BuildError` and
  `SecureXmlReader::read_event` / `into_inner`. `SecureXmlReader::read_next(&mut buf)` returns
  `Ok(false)` at the end of the document.

## Rust: ddex-parser

```text
// 0.5.x
let config = SecurityConfig { max_entity_expansions: 1000, max_file_size: .., ..SecurityConfig::default() };
match parser.sanity_check(reader) { Err(_) => /* unknown version */, Ok(r) => .. }
```

```rust
use ddex_parser::parser::security::SecurityConfig;
use ddex_parser::DDEXParser;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = std::fs::read("release.xml")?;

    // SecurityConfig keeps only max_element_depth
    let mut parser = DDEXParser::with_config(SecurityConfig { max_element_depth: 100 });

    // An unrecognised version is Ok with version None (it was an Err)
    let check = parser.sanity_check(Cursor::new(xml.as_slice()))?;
    match check.version {
        Some(version) => println!("ERN {}", version.as_str()),
        None => println!("not an ERN message: {:?}", check.errors),
    }

    let message = parser.parse(Cursor::new(xml.as_slice()))?;
    println!("{} release(s)", message.flat.releases.len());
    Ok(())
}
```

- **`SecurityConfig`**: `relaxed()`, `enable_fast_streaming`, `max_entity_expansions` and the
  fields nothing read (`disable_dtd`, `max_file_size`, `parse_timeout`, `allow_network`, ...)
  were removed. Use `SecurityConfig::strict()` (the default). A reference to an undefined
  entity is an error on every reader, so nothing is expanded; a document with more than 100
  references (`&amp;`, character references) no longer fails the default parse.
- **Every reader rejects a DOCTYPE**: `ParseMode::Stream`, `stream*`, the selective, xpath,
  multi-release and extension-capture readers and the zero-copy and parallel scanners now
  fail with `ParseError::SecurityViolation` like `parse`. `GraphBuilder::build_from_xml` and
  `detect_version` also reject undefined entities; the `build_from_xml_with_*` variants were
  removed (use `build_from_xml`).
- **`ParseOptions`**: `auto_threshold`, `include_raw`, `allow_blocking`,
  `include_raw_extensions`, `include_comments`, `preserve_unknown_elements`,
  `with_extensions()`, `for_round_trip()` and `ParseMode::Auto` were removed. Use
  `ParseMode::Dom` (the default).
- **`sanity_check`**: `SanityCheckResult::version` is `Option<ERNVersion>`, and `warnings` was
  removed (nothing added one).
- **Statistics**: `WorkingStreamingStats::max_depth_reached` was removed; `current_depth`,
  `current_memory_bytes` and `max_memory_used_bytes` are `Option` (treat `None` as "not
  measured"); `ParseProgress::resources_parsed` and `estimated_total_bytes` were removed.
- **Removed, with no replacement**: `FastStreamingParser` and its types (use `parse`, or
  `DDEXParser::stream*` for raw elements), `stream_resources` / `stream_parties`,
  `VersionAdapter`, `ParallelBenchmark`, `WorkingStreamingParser::feed_chunk`, XPath index
  filters (`[n]`, now an error), the never-constructed error variants, the `bench_report`
  module and the `bench`, `wasm` and `simd` features. `MultiReleaseParser::new()` and
  `with_security_config(config)` take no version.
- **quick-xml in the API**: the `From` impls from quick-xml errors into `ParseError` were
  removed. Match on the `ParseError` variants; the messages are unchanged.

## Node.js

**Node.js 22 or newer** is required (`engines.node` is `>=22` in both packages and their
platform packages). On Node.js 18 or 20, stay on 0.5.x.

```text
// 0.5.x
parser.parseSync(xml, { includeRawExtensions: false });
builder.buildWithFidelity(request, { enablePerfectFidelity: true });
builder.testRoundTripFidelity(xml, options).then((r) => r.determinismVerified);
builder.applyPreset('spotify_album');
```

```js
const fs = require('node:fs');
const { DdexParser } = require('ddex-parser');
const { DdexBuilder, requestFromXml } = require('ddex-builder');

const xml = fs.readFileSync('release.xml', 'utf8');
const parser = new DdexParser();

// parse / parseSync take only the XML
const message = parser.parseSync(xml);
console.log(`${message.releases.length} release(s)`);

async function main() {
  // version is null when no ERN version is recognised (it was "Unknown")
  const check = await parser.sanityCheck(xml);
  console.log(check.version);

  // New in 0.6.0: parse and map to a build request
  const { request, unmapped } = requestFromXml(xml);
  console.log(`${unmapped.length} values the request cannot hold`);

  // buildWithFidelity → buildWithStatistics, FidelityOptions → BuildOptions
  const builder = new DdexBuilder();
  const result = await builder.buildWithStatistics(request, { collectStatistics: true });
  console.log(`${result.statistics.elementCount} elements`);

  // testRoundTripFidelity → checkRoundTrip; determinismVerified → buildsIdentical
  const roundTrip = await builder.checkRoundTrip(xml);
  console.log(roundTrip.buildsIdentical, roundTrip.issues.length);
}

main();
```

- **Parser**: `ParseOptions` and the second argument of `parse` / `parseSync` were removed
  (every field threw since 0.5.0). `sanityCheck` returns `version: null` for an unrecognised
  version, and its `warnings` was removed.
- **Builder renames**: `buildWithFidelity` → `buildWithStatistics`,
  `testRoundTripFidelity(xml, options)` → `checkRoundTrip(xml)` (the options were never
  applied), `FidelityOptions` → `BuildOptions` (`canonicalization`, `collectStatistics`,
  `enableVerification`), `determinismVerified` → `buildsIdentical`.
- **Builder removals**: the preset methods (`getAvailablePresets`, `getPresetInfo`,
  `applyPreset`, `getPresetValidationRules`) and their types; the `FidelityOptions` fields
  nothing read (`enablePerfectFidelity`, `preserveComments`, ..., `enableDeterministicOrdering`);
  `FidelityInfo.perfectFidelityEnabled`; the `BuildStatistics` fields that were always null
  (`memoryUsedBytes`, `extensionCount`, `canonicalizationTimeMs`); `StreamingConfig.deterministic`
  / `validateDuringStream` and `StreamingStats.dealsWritten` / `warnings`. In the TypeScript
  wrapper, `build` takes no `BuildOptions` (they were never passed on): set the version in the
  request.
- **Build requests**: `message_id` and `message_created_date_time` are required, absent values
  are a missing key or `null` (an empty string fails the build), and generated references are
  `R1`, `A1`, `D1`, ... (see [Rust: ddex-builder](#rust-ddex-builder)).
- `validate()` returns the preflight errors (`CODE: message`) as well as the warnings.

## Python

**Python 3.11 or newer** is required: the wheels are `cp311-abi3` and `requires-python` is
`>=3.11`. On Python 3.10, pip keeps resolving 0.5.1, the latest release that allows it.

```text
# 0.5.x
DDEXParser().parse(xml, ParseOptions(include_raw_extensions=True))
DdexBuilder().build_with_fidelity(request, fidelity_options=FidelityOptions(enable_perfect_fidelity=True))
DdexBuilder().test_round_trip_fidelity(xml).determinism_verified
```

```python
from ddex_builder import BuildOptions, DdexBuilder, request_from_xml
from ddex_parser import DDEXParser

with open("release.xml", encoding="utf-8") as f:
    xml = f.read()

# parse takes only the XML; ParseOptions and ParseResult are gone
parser = DDEXParser()
message = parser.parse(xml)

# version is None when no ERN version is recognised
print(parser.sanity_check(xml)["version"])

# New in 0.6.0: parse and map to a build request
bridge = request_from_xml(xml)
print(len(bridge["unmapped"]), "values the request cannot hold")

# build_with_fidelity -> build_with_statistics; fidelity_options= -> options=
builder = DdexBuilder()
result = builder.build_with_statistics(
    options=BuildOptions(collect_statistics=True), request=bridge["request"]
)
print(result.statistics.element_count, "elements")

# test_round_trip_fidelity -> check_round_trip; determinism_verified -> builds_identical
round_trip = builder.check_round_trip(xml)
print(round_trip.builds_identical, len(round_trip.issues))
```

- **Parser**: `ParseOptions`, the `options` argument of `parse` / `parse_async` / `stream`, the
  keyword arguments of `ddex_parser.parse`, `ParseResult` and the native
  `DDEXParser.from_dataframe` (it raised `NotImplementedError`) were removed. To build from a
  DataFrame, use `ddex_builder.DdexBuilder().from_dataframe(df, schema, request)`.
  `sanity_check` has no `warnings`, and a read failure raises `ValueError`.
- **Builder renames**: as in Node.js, in snake_case (`build_with_statistics`,
  `check_round_trip`, `BuildOptions`, `builds_identical`); the keyword argument
  `fidelity_options=` is now `options=`.
- **Builder removals**: the preset methods and the `PresetInfo` / `ValidationRule` classes;
  the `FidelityOptions` arguments nothing read (passing one is a `TypeError`); the
  `BuildStatistics` fields that were always `None` (`memory_used_bytes`, `extension_count`,
  `canonicalization_time_ms`, also gone from its constructor).
- **Build requests**: as in Node.js. `from_dataframe` no longer requires a `release_id`
  column; a missing, `None` or NaN cell writes no GRid.

## WASM

The WASM builds are not published; this affects code built from
`packages/*/bindings/wasm`.

- **Parser**: `parse_stream` (it rejected with "not implemented") and the ignored `options`
  argument of `parse` were removed. Read the stream into a string and call `parse(xml)`.
- **Builder**: `DdexDiffViewer` and `DiffViewerConfig` were removed, with no replacement. The
  build request changes are the same as in Node.js.

## CLI

| 0.5.x | 0.6.0 |
|---|---|
| `ddex-builder validate`, `watch`, `server` | Removed (each exited with "not implemented"). `build` runs preflight validation (`--validate`) |
| `ddex-builder diff`, `convert`, `schema`, `guarantees`, `preset ...` | Removed, no replacement |
| `--preset`, global `--config`, `build --profile`, `build --strict`, `schema --profile`, `schema --with-docs` | Removed (they did not change the output) |
| `--verify-determinism` | `--check-repeatable` (prints "N builds were byte-identical" or where they differed) |
| `--determinism-iterations` | `--repeat-count` |
| Message ID and creation time filled in | Required: in the request file, or `--message-id` and `--message-created-date-time` |
