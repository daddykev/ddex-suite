---
sidebar_position: 12
title: Known issues
description: The open issues that matter most on main (0.6.0, unreleased), what 0.6.0 delivers, and the parked items.
---

# Known issues

The complete list of open bugs, each with the test that pins it, is
[KNOWN_ISSUES.md](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md). This page
summarizes the ones you are most likely to meet. Report new ones on
[GitHub Issues](https://github.com/daddykev/ddex-suite/issues).

## Parser

- **Streaming is experimental (Rust only).** `parse` reads the whole document and is the
  supported API. `ParseMode::Stream` and the `stream*` iterators read a subset of the fields
  (no territory details, dates, P and C lines, genres, labels or track layout) and still trim
  text. `StreamingParser::stream_releases` yields nothing for a full document.
- **The parallel streaming iterator does not scale** with the thread count.
- **Deals**: with several `ValidityPeriod`s in one deal the last dates win;
  `StartDateTime` / `EndDateTime` are not read; the flat deal has no commercial model type.
- **ERN 3.8.x**: release display artists all get the role `MainArtist`; only the first
  `ResourceGroup` of the default details block is kept in the graph; resource
  `ProprietaryId`s are not read; track `disc_number` is not read.
- **Versions are enum names**: `version` (flat model, Node.js, Python) and the Python
  `detect_version` return `"V4_3"`, not `"4.3"`.
- **Malformed XML**: `]]>` in text is accepted, and mismatched tags come back as a generic
  XML error.

## Builder

- **ERN 3.8.x targets**: video resources are rejected, and a release identified only by an
  ISRC cannot be built.
- **An empty `MessageId`** (`<MessageId/>`, as in the official `8 DjMix.xml` samples) is
  reported as missing by every target, so those samples cannot be rebuilt as they are.
- **Empty titles**: the request cannot express a present but empty title; `""` means "not
  given" and fails the required-field check.
- **ERN 3.8.1 output is not validated** against an official XSD (none is available; see
  [ERN version support](./ern-versions.md#the-ern-381-caveat)).
- **Other APIs to avoid**: the streaming writer (`StreamingBuilder`, Node.js
  `StreamingDdexBuilder`) writes only the old 3.8-style tree and rejects 4.x. The security
  helpers
  (`PathValidator`, `InputValidator`, `OutputSanitizer`) have the issues listed in the
  [security model](./security.md#known-security-issues).

## Packaging

- No npm binaries for Linux arm64 or Linux musl.
- The WASM builds are not published.

## What 0.6.0 delivers

0.6.0 is not released yet; this is what main has. The breaking changes and what to change
are in [Migrating to 0.6.0](./migration-0.6.md).

- **Only real results.** APIs that returned invented, placeholder or constant results, and
  options that nothing read, were removed: update messages, the semantic diff, presets,
  the fidelity, verification and determinism-guarantee APIs, version conversion and
  detection, schema generation, the fast streaming parser and other stubs, and the builder
  CLI's `validate`, `watch` and `server`.
- **One builder API.** `DDEXBuilder` with one options type, `BuildOptions`; the bindings use
  the same names (`buildWithStatistics` / `build_with_statistics`, `checkRoundTrip` /
  `check_round_trip`).
- **Deterministic inputs.** `MessageId` and `MessageCreatedDateTime` are required (the
  builder no longer fills them in from a clock or a random source). The builder reads no
  clock or random source and uses no unordered iteration on the build path (enforced by
  clippy lints). The same request and options produce byte-identical output with the same
  crate version; repeated builds are tested to be byte-identical. References the request
  leaves out are numbered in request order (`R1`, `A1`, `D1`, ...).
- **Absent means absent.** Optional request values are `None` / `null`; an empty string
  fails the build instead of meaning "absent".
- **Parse → modify → build.** `BuildRequest::from_parsed` (Rust), `requestFromXml` (Node.js)
  and `request_from_xml` (Python) map a parsed message to a build request and list what it
  cannot hold.
- **Preflight.** `DDEXBuilder::preflight` runs the checks without building; in the default
  `Warn` mode preflight errors are returned as warnings instead of being discarded; the UPC
  / EAN check digit is the GS1 mod-10 check.
- **Parser.** Every reader rejects a DOCTYPE; attribute values are normalized per XML 1.0
  on every reader; a document with more than 100 references no longer fails the default
  parse.
- **Platforms.** Node.js 22 or newer and Python 3.11 or newer.

Not in 0.6.0: stable, content-addressed IDs (generated references are sequential instead),
a conformance suite, and a parallel streaming iterator that scales with the thread count.

## Parked

Smaller findings recorded for later, as listed in
[KNOWN_ISSUES.md](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md#parked):

- The parsed models leave some DDEX fields always empty (`Release.party_list`, `Resource.rights_controller`, `Party.party_role` / `Party.contact_details`, `DealTerms.distribution_channel` / `DealTerms.price_information`); these elements are not parsed in 0.6.0.
- Consider #[non_exhaustive] on options/results/errors/enums before 1.0.
- `ProprietaryIdRequest.value` is not checked: an empty value is written as an empty `ProprietaryId` (`packages/ddex-builder/src/generator/`), which the schema's non-empty string type rejects.
- Golden-file output tests.
- `packages/ddex-builder/bindings/node/streaming_test.js` (not run by `npm test`) passes snake_case header keys and `ern/43`, which `StreamingDdexBuilder.startMessage` does not accept.
- Node/Python validate() checks stored releases, which have no header, so it reports the missing header before any preflight error; document or rework.
- `parser::attribute_extractor::AttributeExtractor::extract_attributes` is only called by its unit tests (crate-private since 0.6.0 because it takes a quick-xml type), so the module is dead code outside tests.
- Text line endings (XML 1.0 §2.11): only the DOM path (`transform/ern4.rs`) turns a literal CRLF / lone CR in character data into LF; the other readers (streaming, selective, xpath, multi-release, Stream mode) keep it as written.
- The xpath, selective, multi-release, extension-capture, version-detection, working-streaming and zero-copy readers keep attribute values with their references unresolved (`&amp;` stays as written; the XML 1.0 whitespace normalization is applied); only the DOM path and the namespace detector resolve them.
- The WASM builder binding has no parse-to-build bridge (`requestFromXml`): it does not include ddex-parser.
- ddex-builder canonicalizer (`src/canonical/mod.rs`, `rules.rs`): the locked-prefix table rewrites the prefix of a DDEX namespace declaration but not the element and attribute names that use it, so input with other prefixes (e.g. `ernm:`) comes out not namespace-well-formed.
- `packages/ddex-parser/bindings/node/package.json` keeps the `typescript` and `@types/node` devDependencies, which only the removed `build:ts` (tsc) used.
