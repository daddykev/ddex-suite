---
sidebar_position: 1
title: Introduction
description: What DDEX Suite does, what it does not do, and its status at 0.5.1.
---

# Introduction

DDEX Suite reads and writes [DDEX](https://ddex.net) ERN (Electronic Release Notification)
XML, the format labels and distributors use to send release metadata to music services.
It is one Rust core with bindings for Node.js and Python:

- **ddex-parser** reads ERN 3.8.1, 3.8.2, 4.2 and 4.3 messages into a flat,
  developer-friendly model (Rust, Node.js, Python) and a graph that follows the message
  structure (Rust only).
- **ddex-builder** writes ERN 4.3, 4.2, 3.8.2 and 3.8.1 messages from a build request.
  ERN 4.3, 4.2 and 3.8.2 output is validated against the official DDEX XSDs in CI.
- **ddex-core** holds the data models the two share.

The packages are published as `ddex-parser` and `ddex-builder` on
[npm](https://www.npmjs.com/package/ddex-parser),
[PyPI](https://pypi.org/project/ddex-parser/) and [crates.io](https://crates.io/crates/ddex-parser),
plus `ddex-core` on crates.io. All of them are released together at the same version.

## What it does

- **Parses without inventing data.** A value that is absent from the XML is `None` /
  `undefined` / empty, never a default. Text is kept as written, following each value's
  XSD `whiteSpace` facet.
- **Reads the fields people need.** Release and track titles, artists, identifiers,
  release dates (full or partial, as written), P and C lines, genres, parental warnings,
  labels, the track layout (disc, position, nested resource groups), ERN 4.x track
  releases and deals. See the [parse output reference](./parse-output.mdx).
- **Builds schema-valid XML.** Nothing is filled in for you: when the target ERN version
  requires a value the request does not carry, the build fails with an error naming the
  field. See the [build request reference](./build-request.mdx).
- **Hardens XML parsing.** DTDs and external entities are rejected, no entity is expanded,
  nesting depth is limited, and nothing is fetched from the network. See the
  [security model](./security.md).
- **Exports to pandas.** The Python parser returns DataFrames and the Python builder
  builds from one. See [Python and DataFrames](./python-dataframes.md).

## What it does not do

- **No runtime XSD or AVS validation.** Neither package checks a message against the DDEX
  schema or the allowed-value sets and business rules at runtime. Use
  [DDEX Workbench](https://ddex-workbench.org) for that.
- **No parse-to-build bridge.** There is no built-in conversion from a parsed message to a
  build request yet; you map the fields you need (the
  [building quick start](./quick-start-building.mdx#parse-modify-build) shows how).
- **Not byte-for-byte deterministic yet.** A message ID, references and
  `MessageCreatedDateTime` that the request leaves out are generated, so two builds of the
  same request can differ. Full determinism is planned for 0.6.0.
- **No streaming in Node.js or Python.** `parse` reads the whole document. The Rust
  streaming APIs are experimental and read a subset of the fields.

## Status

The current release is **0.5.1**, a pre-1.0 patch release (security fixes and fixes of
results that were not computed; see the
[changelog](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md)). APIs can still
change between minor versions, and 0.5.0 had breaking changes (see
[Migrating to 0.5.0](./migration.md)).
Open bugs are listed in [Known issues and roadmap](./known-issues.md).

DDEX Suite is designed to complement [DDEX Workbench](https://ddex-workbench.org): the suite
parses and builds messages, and Workbench validates them against the XSDs and business
rules.

## Next steps

- [Install](./installation.md) the packages for your language.
- [Parse](./quick-start-parsing.mdx) your first message.
- [Build](./quick-start-building.mdx) an ERN 4.3 message.
- Try the [Playground](./playground.md) without installing anything.
