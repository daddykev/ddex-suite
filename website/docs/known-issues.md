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
- **`SecurityConfig::relaxed()` makes `parse` fail.** It switches to the fast streaming
  path, which is not implemented: since 0.5.1 it returns an error (before, a message with
  no releases).
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
- **Python `ParseOptions` has no effect**: its fields are accepted for forward compatibility
  but do not change the result.

## Builder

- **Not byte-for-byte deterministic yet.** Values the request leaves out (message ID,
  references, creation time) are generated from random UUIDs or the clock, the stable-hash
  ID option puts the current date into the message ID, and the sequential option the
  current Unix time. Update messages always carry the current time, and the streaming
  writer generates a random message ID and the current time when they are missing.
- **Presets do not change the output.** The preset APIs (`applyPreset`, `apply_preset`,
  partner presets) have no effect on `build` in 0.5.x.
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
- Investigate attribute-value normalization: the parser keeps a literal tab, line feed or
  carriage return in an attribute value as written, where XML 1.0 (§3.3.3) normalizes each
  to a space, so its values may differ from the lxml oracle's.

Later, on the way to **1.0.0**: a built-in bridge from parsed messages to build requests (true
Parse → Modify → Build), a semantic diff, and TypeScript API reference pages. (From 0.5.1,
release artifacts are built in GitHub Actions with signed build-provenance attestations and SBOMs, verifiable with `gh attestation verify <file> --repo daddykev/ddex-suite`, provided the published files are the ones release-build produced.)
