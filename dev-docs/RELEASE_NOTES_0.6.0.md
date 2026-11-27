# DDEX Suite 0.6.0 release notes

Date: 2026-11-27. Every package is released at 0.6.0: crates (`ddex-core`, `ddex-parser`,
`ddex-builder`), npm (`ddex-parser`, `ddex-builder` and their platform packages) and PyPI
(`ddex-parser`, `ddex-builder`). Full list: `CHANGELOG.md` [0.6.0].

0.6.0 is a breaking release. Its aim is an honest API that can stay stable: the APIs that
returned no real results (invented values, placeholder documents, constant scores) or whose
options had no effect are removed, the builder is one type with one options type, its
inputs are explicit, and a parsed message can be turned into a build request. Read
"Upgrading" before you update; the
[migration guide](https://github.com/daddykev/ddex-suite/blob/main/website/docs/migration-0.6.md)
has the details for each audience.

## Highlights

- **Only real results.** Removed, because they returned no real results or no build read
  them: update messages, the semantic diff, presets, the determinism guarantees and
  verifier, verification, the fidelity APIs, the C14N / C14N 1.1 / custom canonicalization
  algorithms, version conversion and detection, schema generation, unused builder modules,
  the entity classifier and error sanitizer, `FastStreamingParser` and other parser stubs,
  and the options and fields nothing read. The inventory is `dev-docs/REMOVALS_0.6.0.md`.
- **One builder.** `Builder` is folded into `DDEXBuilder`, which has `build`,
  `build_with_statistics`, `check_round_trip`, `canonicalize`, `supported_versions` and the
  new `preflight`. `BuildOptions` is the one options type (`output: OutputConfig` replaces
  `determinism`). The Node.js and Python bindings use the same names
  (`buildWithStatistics` / `build_with_statistics`, `checkRoundTrip` / `check_round_trip`,
  `BuildOptions`).
- **Deterministic output.** `MessageId` and `MessageCreatedDateTime` are required: the
  builder no longer fills them in with a random UUID and the current time. The builder
  reads no clock or random source and uses no unordered iteration on the build path
  (enforced by clippy lints). The same request and options produce byte-identical output
  with the same crate version; repeated builds are tested to be byte-identical. References
  the request leaves out are numbered in request order (`R1`, `A1`, `D1`, ...).
- **Parse → modify → build** in Rust, Node.js and Python: `BuildRequest::from_parsed`,
  `requestFromXml(xml)` and `request_from_xml(xml)` map a parsed message to a build request
  for the same ERN version, invent nothing, and list in `unmapped` what the request cannot
  hold. `tests/it/bridge_round_trip.rs` round-trips the ERN 4.x and 3.8.x samples through it.
- **Absent means absent.** Optional request values (`release_id`, release and track
  `artist`, `isrc`, `commercial_model_type`) are `Option` / `null`; an empty string fails
  the build instead of meaning "absent".
- **Preflight.** In the default `Warn` mode, preflight errors are returned as warnings
  instead of being discarded; the Node / Python `validate()` returns them. The UPC / EAN
  check digit is the GS1 mod-10 check (valid 12- and 14-digit codes were rejected).
- **Parser.** Every reader rejects a DOCTYPE. Attribute values are normalized per XML 1.0
  §3.3.3 on every reader. A document with more than 100 references (`&amp;`, character
  references) no longer fails the default parse.
- **Platforms.** Node.js 22 or newer and Python 3.11 or newer (`cp311-abi3` wheels).
- **Dependencies.** ddex-builder drops `rayon`, `string_cache`, `smartstring`, `log`,
  `dhat`, `blake3` and `num_cpus`; ddex-parser drops `thiserror` and its direct `ts-rs`.

## Upgrading

The [migration guide](https://github.com/daddykev/ddex-suite/blob/main/website/docs/migration-0.6.md)
has before / after code for each audience; the Breaking section of `CHANGELOG.md` says who
is affected by each change. In short:

- **Rust builder:** replace `Builder::new()` with `DDEXBuilder::new()` and pass the settings
  in `BuildOptions` (`determinism: Some(config)` becomes `output: config`); set the target
  version in `BuildRequest::version`. Supply `message_id` and `message_created_date_time`.
  Wrap optional request values in `Some(...)` and replace `""` with `None`. Renamed:
  `emit_reproducibility_banner` → `emit_generator_banner`,
  `BuildResult::reproducibility_banner` → `generator_banner`.
- **Rust parser:** drop the removed `SecurityConfig` fields (it keeps `max_element_depth`)
  and `ParseOptions` fields; use `ParseMode::Dom` instead of `Auto`. `sanity_check` returns
  `Ok` with `version: None` for an unrecognised version. quick-xml types are no longer part
  of the API.
- **Node.js:** `parse` / `parseSync` take only the XML; `sanityCheck` returns `version: null`
  for an unrecognised version. Builder: `buildWithFidelity` → `buildWithStatistics`,
  `testRoundTripFidelity` → `checkRoundTrip`, `FidelityOptions` → `BuildOptions`,
  `determinismVerified` → `buildsIdentical`; the preset methods are gone.
- **Python:** `parse` takes only the XML (`ParseOptions` and `ParseResult` are gone).
  Builder: the same renames in snake_case, and `fidelity_options=` is `options=`.
- **WASM:** the parser's `parse_stream` and `options` argument and the builder's
  `DdexDiffViewer` were removed.
- **CLI:** `validate`, `watch`, `server`, `diff`, `convert`, `schema`, `guarantees` and
  `preset` were removed; `--verify-determinism` is `--check-repeatable` and
  `--determinism-iterations` is `--repeat-count`; `--message-id` and
  `--message-created-date-time` supply the header values a request file lacks.
- **Platforms:** on Python 3.10 pip keeps resolving 0.5.1; on Node.js 18 or 20, stay on
  0.5.x.

## Performance

Re-measured for 0.6.0 on a GitHub-hosted runner (Intel Xeon Platinum 8573C, 4 vCPUs) with
the Bench workflow, `cargo bench` (release profile), commit `410315f`,
[run 45985971656](https://github.com/daddykev/ddex-suite/actions/runs/45985971656):

| Operation | Time |
|---|---:|
| Parse `1 Audio.xml` (75 KB) | 2.5 ms |
| Parse official ERN 4.3 samples (10–330 KB) | 0.41–11 ms (about 22–31 MiB/s) |
| Parse a ~10 MB ERN 4.3 catalog | 323 ms |
| Build ERN 4.3, 1 / 12 / 100 tracks | 0.73 / 3.4 / 24 ms |

The 0.5.1 numbers were measured on a runner with another CPU (Intel Xeon 6975P-C) and are
not comparable. The parallel streaming iterator does not scale with the thread count (1, 2
and 4 threads take 20.8–21.1 ms on the 10 MB catalog). Full results:
`dev-docs/PERFORMANCE.md`.

## Known issues

`KNOWN_ISSUES.md` lists the bugs pinned by ignored tests and the ones without a test. The
most visible ones:
- The Rust streaming APIs are experimental: `ParseMode::Stream` fails on any message with a
  release, and `StreamingParser::stream_releases` yields nothing for a full document.
- ERN 3.8.x targets reject video resources and cannot rebuild a release identified only by
  an ISRC; an empty `MessageId` (as in the official `8 DjMix.xml` samples) is reported as
  missing.
- Round trips are not lossless: data a build request cannot carry (contributors, P and C
  lines, images, ...) is not rebuilt; `unmapped` lists it.
- The streaming writer (`StreamingBuilder`, Node.js `StreamingDdexBuilder`) rejects ERN 4.2
  and 4.3.
- The builder's security helpers (`PathValidator`, `InputValidator`, `OutputSanitizer`)
  have the issues listed there; `build` does not depend on them.
- The Python `detect_version` returns `"V4_3"`.

### Parked

Smaller findings recorded for later (`KNOWN_ISSUES.md`, "Parked"):
- Some parsed model fields are always empty (`Release.party_list`,
  `Resource.rights_controller`, `Party.party_role` / `contact_details`, `DealTerms`
  distribution channel and price information).
- `ProprietaryIdRequest.value` is not checked: an empty value writes an empty
  `ProprietaryId`, which the schema rejects.
- The canonicalizer rewrites the prefix of a DDEX namespace declaration but not the element
  names that use it, so input with other prefixes comes out not namespace-well-formed (the
  builder's own output is not affected).
- Only the DOM path normalizes literal CRLF / CR in text to LF, and several secondary
  readers keep attribute references unresolved.
- The Node.js / Python `validate()` reports the missing header of stored releases before
  any preflight error.
- The WASM builder has no parse-to-build bridge.
- Housekeeping: `#[non_exhaustive]` before 1.0, golden-file output tests, a stale Node
  streaming test script, a module that is dead code outside its tests, and unused
  TypeScript devDependencies in the Node parser package.

## Verification

- CI on the release branch:
  - Rust: fmt, clippy (with the builder's determinism lints), workspace tests, XSD
    validation of builder output, cargo-deny.
  - Bindings: Node.js 22, Python 3.11–3.14, the README and website code examples.
  - The ERN 4 oracle comparison.
- The **Release build** workflow builds every published artifact, install-tests each one on
  a matching runner, writes the SBOMs and attests them. `PUBLISH.md` says how to verify the
  downloaded files with `gh attestation verify` and publish those exact files.
