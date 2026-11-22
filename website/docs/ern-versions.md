---
sidebar_position: 7
title: ERN version support
description: Which ERN versions the parser reads and the builder writes, the ERN 3.8.1 caveat, and how CI checks them against the official XSDs.
---

# ERN version support

| ERN version | Parse | Build | Builder output checked in CI |
|---|---|---|---|
| **4.3** | ✓ | ✓ | Validated against the official XSD |
| **4.2** | ✓ | ✓ | Validated against the official XSD |
| **3.8.2** | ✓ | ✓ (sound recordings only) | Validated against the official XSD |
| **3.8.1** | ✓ | ✓ (sound recordings only) | Not checked: no official XSD is available |

Other versions (3.7, 3.8.3, 4.0, 4.1, ...) are not supported. The parser detects the version
from the message's namespace and rejects a document without an ERN namespace; the builder
rejects any other `version` value.

## Parsing

The parser reads ERN 3.8.1, 3.8.2, 4.2 and 4.3 into the same flat model, so code written
against the flat model works for every version. Where the versions differ, the
[parse output reference](./parse-output.mdx) says so: for example, ERN 3.8.x takes genres,
labels, P and C lines and dates from the `...DetailsByTerritory` blocks (with their
territories as the scope), and only ERN 4.x has track releases.

ERN 3.8.1 messages are read with the same rules as 3.8.2.

## Building

Set `version` in the build request to `"4.3"`, `"4.2"`, `"3.8.2"` or `"3.8.1"`; there is no
default. The [build request reference](./build-request.mdx#required-fields-by-ern-version)
lists what each version requires. The differences in short:

- **ERN 4.3 and 4.2** write a `PartyList` (artists and labels as parties), `ReleaseList`
  with one release and its `ResourceGroup` tree, optional `TrackRelease`s, and `DealList`.
  4.3 also requires `avs_version_id`.
- **ERN 3.8.2** writes the 3.8 structure: `...DetailsByTerritory` blocks with
  `TerritoryCode` `Worldwide`, and `ReleaseResourceReferenceList`. It writes sound
  recordings only: a track with `resource_kind: "Video"` fails the build. A release or track
  release identified only by an ISRC cannot be built yet.
- **ERN 3.8.1** output is the ERN 3.8.2 document in the 3.8.1 namespace
  (`http://ddex.net/xml/ern/381`).

### The ERN 3.8.1 caveat

DDEX no longer serves the ERN 3.x schemas, and no archived copy of the official 3.8.1 XSD
could be found, so 3.8.1 output is **not validated against an official schema**. The
elements the builder writes have the same content models in an unofficial copy of the 3.8.1
schema, and CI checks that the 3.8.1 output is the 3.8.2 document in the 3.8.1 namespace. If
your recipient accepts ERN 3.8.2, prefer it.

## How CI checks this

The [CI workflow](https://github.com/daddykev/ddex-suite/blob/main/.github/workflows/ci.yml)
runs on every pull request:

- **XSD validation.** The official ERN 4.3, 4.2 and 3.8.2 XSDs and their allowed-value-set
  schemas are downloaded (pinned by SHA-256; the 3.8.2 XSD comes from the Internet Archive's
  capture of its official URL, since DDEX removed it) and builder output is validated with
  `xmllint`. This covers the Rust builder and the WASM builder.
- **Round trip on the official samples.** Each official DDEX ERN 4.2 and 4.3 sample in
  [`examples/`](https://github.com/daddykev/ddex-suite/tree/main/examples) is parsed,
  rebuilt in its own version, validated against the XSD and parsed again. Release title,
  ICPN, type and display artist, each track's ISRC, title, duration, disc and position, the
  deal count and the deals' release references must match the original.
- **ERN 3.8.2 round trip.** YouTube's public ERN 3.8.2 reference messages and the
  official 4.x samples are parsed, rebuilt as ERN 3.8.2 and validated; the cases the builder
  cannot express yet (videos, ISRC-only release IDs) are listed in
  [KNOWN_ISSUES.md](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md).
- **Parser oracle.** The parser's output for the official 4.2 and 4.3 samples is compared,
  track by track, with an independent `lxml` reading of the same files.

The XSDs check structure and types only. Neither CI nor the packages check the AVS business
rules; [DDEX Workbench](https://ddex-workbench.org) validates those.
