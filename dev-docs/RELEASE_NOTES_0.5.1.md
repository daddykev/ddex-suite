# DDEX Suite 0.5.1 release notes

Date: 2026-11-26. Every package is released at 0.5.1: crates (`ddex-core`, `ddex-parser`,
`ddex-builder`), npm (`ddex-parser`, `ddex-builder` and their platform packages) and PyPI
(`ddex-parser`, `ddex-builder`). Full list: `CHANGELOG.md` [0.5.1].

0.5.1 is a patch release: security updates, and APIs that returned results they did not
compute now compute them or say they cannot. Read "Upgrading" if you use the quick-xml
types the parser and builder expose, the builder's fidelity APIs, or the APIs listed there
that now return an error.

## Highlights

- **Security updates.**
  - quick-xml 0.41: fixes RUSTSEC-2026-0194, a quadratic-time denial of service when a
    start tag has many attributes (the parser checks them for duplicates on untrusted
    input), and RUSTSEC-2026-0195. A regression test parses an official sample with
    50,000 extra root attributes.
  - pyo3 0.29 in the Python bindings: fixes RUSTSEC-2026-0176 and RUSTSEC-2026-0177. The
    Python API is unchanged; building the Python bindings from source needs Rust 1.83.
- **Undefined entities are errors.** A reference such as `&foo;` (neither one of the five
  predefined entities nor a character reference; DTDs are not read) used to be dropped
  silently by several readers. Every reader now fails with an error that names the entity
  and gives its line and column: `undefined entity '&foo;' at line 73, column 29`.
  `&amp;`, `&#233;` and `&#xE9;` decode as before.
- **Results that are computed, or an explicit "not implemented".** Every public API that
  reported success, a score, a count or a value it did not compute was fixed:
  - `sanity_check` (Rust, Python, Node) detects the version and checks well-formedness;
    it used to report every input as valid ERN 4.3.
  - The builder's `validateStructure` / `validate_structure` and the parser CLI's
    `validate --xml-only` read the whole document (they read only its first event).
  - The Python `ddex-parser` command's `parse` works (it failed on every file) and `check`
    parses (it did no checking).
  - Round trips are real: Node `testRoundTripFidelity` (a constant 0.98) and Python
    `test_round_trip_fidelity` (a size-ratio score) parse, build and parse again, and list
    the differences; there is no score (`fidelityScore` / `fidelity_score` is null).
  - The Node / Python fidelity statistics are counted from the XML or null, `validate`
    runs the preflight validation, and the preset methods list the builder's presets.
  - Parse options: Node `ParseOptions` fields throw since 0.5.0; the Python `ParseOptions`
    are documented as having no effect.
  - Rust: `Builder::canonicalize` runs DB-C14N, the version converter writes well-formed
    XML and counts elements, cache statistics count, `total_duration` is computed, and the
    streaming iterators report the document instead of invented defaults.
  - What cannot be computed yet returns "not implemented in 0.5.x" (see "Upgrading").
- **SBOMs and build provenance.** Release artifacts are built in GitHub Actions with
  signed build-provenance attestations and SBOMs, verifiable with
  `gh attestation verify <file> --repo daddykev/ddex-suite`, provided the published files
  are the ones release-build produced. Each `.crate`, npm tarball, wheel and sdist has a
  SLSA provenance attestation and a CycloneDX SBOM attestation. Publishing stays manual:
  there is no npm `--provenance` and no PyPI trusted-publishing attestation.
- **Security contact.** Report vulnerabilities privately by email to
  `security@ddex-suite.org` (SECURITY.md), not in public issues.
- **Dependency cleanup.** Unused dependencies were removed from the crates (ddex-parser:
  ahash, bytes, crossbeam-channel, csv, futures, parking_lot, tokio, toml, uuid;
  ddex-builder: futures, is-terminal, mime, smallstr, tokio, xmltree) and from the binding
  crates. `cargo-deny` (advisories, bans, licenses, sources) runs in CI.

## Upgrading

### Rust: quick-xml 0.41 in the public API
Public items that name quick-xml types now use quick-xml 0.41: the `From<quick_xml::Error>`
and `From<AttrError>` conversions into `ParseError` / `BuildError`,
`StreamingError::XmlError`, `SecureXmlReader::{read_event, into_inner}`,
`utf8_utils::handle_text_node`, `AttributeExtractor::extract_attributes`,
`ExtensionCaptureContext::process_extension_*` and `XmlValidator::validate_event`. **Who is
affected:** only code that passes its own quick-xml values to these, or converts
quick-xml 0.36 errors with `?` into `ParseError` / `BuildError`; it must use quick-xml
0.41. Code that only parses and builds is not affected: results are unchanged.

### Undefined entities
Input with `&name;` for an undefined entity now fails to parse
(`ParseError::MalformedXml`, `position` the byte offset of the `&`). `parse()` already
failed on it before, with a message without line or column; the streaming iterators,
`ParseMode::Stream`, the selective and XPath readers, and the builder's version converter
did not.

### Fields that are now null
Node / Python builder: `VerificationResult.fidelityScore` / `fidelity_score` and
`BuildStatistics.memoryUsedBytes`, `extensionCount`, `canonicalizationTimeMs` (and their
Python names) are nullable and always null. Read `roundTripSuccess` and `issues` instead of
the score.

### APIs that now return an error
They returned made-up results; each one now fails with "not implemented in 0.5.x":
- Rust builder: `RoundTripTester::test_round_trip` and `Builder::test_round_trip_fidelity`
  (use `test_round_trip_with` / `test_round_trip_fidelity_with` /
  `round_trip::verify_round_trip` with the parser's parse function),
  `RoundTripTester::analyze_fidelity`, `UpdateGenerator::apply_update` /
  `DDEXBuilder::apply_update`, C14N / C14N 1.1 / custom canonicalization,
  `guarantees::validate_no_hashmap_usage` / `validate_deterministic_sorting`, and
  `DeterminismVerifier::verify` with fewer than 2 iterations.
- Rust parser: `parse` with `SecurityConfig::relaxed()` (the fast streaming path),
  `VersionAdapter`, `ParallelBenchmark::measure_parallel_speedup`, and
  `StreamingParser::stream_resources` / `stream_parties`.
- Python parser: the native `DDEXParser.from_dataframe` (use the builder's
  `from_dataframe`). WASM parser: `parse_stream`.
- Builder CLI: `diff`, `validate`, `watch` and `server`. `build` now builds the request
  (it wrote a placeholder document).

`BuildVerifier::verify` / `Builder::verify_build` now report round-trip and XSD
verification as not done (`false`, with the reason as an issue).

## Performance

Re-measured for 0.5.1 on a GitHub-hosted runner (Intel Xeon 6975P-C, 4 vCPUs) with the new
Bench workflow, `cargo bench` (release profile):

| Operation | Time |
|---|---:|
| Parse `1 Audio.xml` (75 KB) | 1.7 ms |
| Parse official ERN 4.3 samples (10–330 KB) | 0.27–7.4 ms |
| Parse a ~10 MB ERN 4.3 catalog | 210 ms |
| Build ERN 4.3, 1 / 12 / 100 tracks | 1.7 / 8.2 / 57 ms |

The 0.5.0 numbers were measured on an Apple M1 Pro and are not comparable. The parallel
streaming iterator still does not scale with the thread count (planned for 0.6.0). Full
results: `dev-docs/PERFORMANCE.md`.

## Known issues

`KNOWN_ISSUES.md` lists the bugs pinned by ignored tests, the ones without a test, and
the APIs that are not implemented in 0.5.x. The most visible ones:
- Builder output is not byte-for-byte deterministic yet, and presets do not change it
  (both planned for 0.6.0).
- Round trips are not lossless: data a build request cannot carry (contributors, P and C
  lines, images, ...) is not rebuilt.
- The Python `detect_version` returns `"V4_3"`.
- No npm binaries for Linux musl or arm64.

## Verification

- CI on the release branch:
  - Rust: fmt, clippy, workspace tests (`-D warnings`), cargo doc, cargo package,
    cargo-deny, MSRV 1.82, XSD validation.
  - Bindings: Node, Python 3.10–3.14, the README and website code examples.
  - The ERN 4 oracle.
- The **Release build** workflow builds every published artifact, install-tests each one on
  a matching runner, writes the SBOMs and attests them. `PUBLISH.md` says how to verify the
  downloaded files with `gh attestation verify` and publish those exact files.
