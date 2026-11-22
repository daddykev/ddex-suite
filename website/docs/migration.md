---
sidebar_position: 11
title: Migrating to 0.5.0
description: The breaking changes in 0.5.0 and what to change when you upgrade from 0.4.x, per language.
---

# Migrating to 0.5.0

0.5.0 contains breaking changes. This page follows the migration guide in the
[0.5.0 release notes](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/RELEASE_NOTES_0.5.0.md);
the full list of changes is in the
[changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md).

## All languages

- **Text is no longer trimmed.** Values keep their whitespace, following each value's XSD
  `whiteSpace` facet. If you compared titles or names after the parser trimmed them, trim
  them yourself.
- **Large inputs return the complete model.** `parse` used to switch to a streaming path
  above 10 MB, which returned fewer fields. It now always uses the complete parser; large
  inputs take about as long per MB as small ones (about 20–24 MiB/s).
- **New fields.** Release dates, P and C lines, genres, parental warnings, labels,
  `MessageControlType`, deal end dates, the track layout and ERN 3.8.2 territory details are
  read in ERN 3.8.2, 4.2 and 4.3 ([parse output reference](./parse-output.mdx)). Absent values
  stay empty.
- **Builder.** The build request is the core's `BuildRequest` in every language: `header`
  and `version` are required, nothing is filled in, and a value the target version requires
  fails the build with an error naming the field. ERN 3.8.2 output is now schema-valid.

## Node.js

### `ddex-parser`

- **Remove every parse option.** Each `ParseOptions` field now throws, for example
  `parseSync(xml, { collectStatistics: true })`:
  - the streaming options `mode`, `autoThreshold`, `enableStreaming`, `streamingThreshold`;
  - the options that had no effect: `resolveReferences`, `includeRaw`, `maxMemory`,
    `timeoutMs`, `allowBlocking`, `chunkSize`, `fidelityLevel`, `preserve*`,
    `canonicalization`, `collectStatistics`, `validationLevel`, `extensionValidation`,
    `enableChecksums`, `memoryLimit`, `enableDetailedErrors`.

  `parse(xml)` and `parseSync(xml)` without options behave as before.
- `ParsedMessage.statistics` and `fidelityInfo` were removed. They held zeros and an echo of
  the options.
- `DdexParser.stream()` was removed: it returned hard-coded test data. Use `parse` /
  `parseSync`.
- `ParsedMessage.resources` values use camelCase keys: `resources.A1.resource_id` is now
  `resources.A1.resourceId`.
- `displayArtist`, `artist`, `senderName` and `recipientName` are optional.

### `ddex-builder`

- `build(request)` takes the core build request (snake_case fields; see
  `build-request.d.ts` and the [build request reference](./build-request.mdx)). `header`
  and `version` are required.
- `batchBuild` was removed.

## Python

- **Python 3.10 or newer** is required (0.4.5 allowed 3.8). The wheels are tagged
  `cp310-abi3`.
- **The native extension is required.** `import ddex_parser` / `import ddex_builder` raise
  `ImportError` if it cannot load; the message names the cause and the fix (reinstall a wheel
  for your platform, or run `maturin develop` in a checkout). The mock classes and fake
  results are gone, and `ddex_builder.is_rust_available()` always returns `True`.
- `__version__` reports the installed package version.
- The flat DataFrame's `sender` column holds the sender's name.
- The DataFrames gained columns ([Python and DataFrames](./python-dataframes.md)).
- `DdexBuilder.build(request)` and `from_dataframe(df, schema, request)` run the core
  builder; nothing is filled in.

## WASM builder

- `build(request)` takes the core `BuildRequest` and is synchronous.
- These members returned placeholder values and were removed: `batchBuild`, the
  `Release` / `Resource` classes, and the preset and fidelity methods.

## Rust

- The graph and flat models have new fields. Struct literals that construct these types need
  the new fields (most new fields are `Option` or `Vec`).
- The builder's request structs gained fields; use `..Default::default()`.
- `ParseMode::Auto` no longer selects `Stream`.
