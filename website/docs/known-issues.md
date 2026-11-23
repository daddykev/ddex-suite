---
sidebar_position: 12
title: Known issues and roadmap
description: The open issues that matter most when using 0.5.0, and what is planned for 0.6.0.
---

# Known issues and roadmap

The complete list of open bugs, each with the test that pins it, is
[KNOWN_ISSUES.md](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md). This page
summarizes the ones you are most likely to meet. Report new ones on
[GitHub Issues](https://github.com/daddykev/ddex-suite/issues).

## Parser

- **Streaming is experimental (Rust only).** `parse` reads the whole document and is the
  supported API. `ParseMode::Stream` and the `stream*` iterators read a subset of the fields
  (no territory details, dates, P and C lines, genres, labels or track layout) and still trim
  text. `StreamingParser::stream_releases` yields nothing for a full document.
- **`SecurityConfig::relaxed()` returns no releases.** It switches to the fast streaming
  path, whose conversion to the model is not written yet, and that path ignores its memory
  and security settings.
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
- **`sanity_check` is a placeholder** (Rust, Node.js `sanityCheck`, Python): it reports any
  input as valid ERN 4.3. Use `parse` to check that a message can be read.
- **Python `ParseOptions` has no effect**: its fields are accepted for forward compatibility
  but do not change the result.

## Builder

- **Not byte-for-byte deterministic yet.** Values the request leaves out (message ID,
  references, creation time) are generated, and the stable-hash ID option puts the current
  date into the message ID.
- **Presets do not change the output.** The preset APIs (`applyPreset`, `apply_preset`,
  partner presets) have no effect on `build` in 0.5.0.
- **ERN 3.8.x targets**: video resources are rejected, and a release identified only by an
  ISRC cannot be built. An empty `MessageId` is reported as missing for 3.8.2.
- **Empty titles**: the request cannot express a present but empty title; `""` means "not
  given" and fails the required-field check.
- **ERN 3.8.1 output is not validated** against an official XSD (none is available; see
  [ERN version support](./ern-versions.md#the-ern-381-caveat)).
- **The `ddex-builder` command-line tool** writes a placeholder document and its
  `--validate` never fails. Use the library or a binding to build.
- **Other APIs to avoid**: the streaming writer (`StreamingBuilder`, Node.js
  `StreamingDdexBuilder`) writes only the old 3.8-style tree and rejects 4.x; the Node.js
  `validateStructure` reports invalid XML as valid; `DDEXBuilder::diff_xml` and the WASM
  `DdexDiffViewer` report any difference as one change. The security helpers
  (`PathValidator`, `InputValidator`, `OutputSanitizer`) have the issues listed in the
  [security model](./security.md#known-security-issues).

## Packaging

- No npm binaries for Linux arm64 or Linux musl.
- The WASM builds are not published.

## Roadmap

**0.6.0** (next):

- Builder determinism: byte-for-byte reproducible output, with stable, content-addressed IDs
  as the default.
- Presets that change the output.
- A conformance suite.
- A parallel streaming iterator that scales with the thread count.
- The builder's determinism lint (no `HashMap` / `HashSet` in output paths) enforced in CI.

Later, on the way to **1.0.0**: a built-in bridge from parsed messages to build requests (true
Parse → Modify → Build), a semantic diff, TypeScript API reference pages, and supply-chain
hardening (`cargo-deny` in CI, SBOMs and signed artifacts).
