---
sidebar_position: 13
title: Playground
description: What the online playground does, and its limits.
---

# Playground

The [Playground](/playground) runs the parser and the builder without installing anything.

- **Parser mode**: paste an ERN 3.8.x, 4.2 or 4.3 message and get the parser's output as
  JSON: the Node.js `ParsedMessage` described in the
  [parse output reference](./parse-output.mdx). Two ERN 4.3 samples (a single and a
  two-track album) are built in.
- **Builder mode**: edit a build request (JSON) and get the ERN XML. It starts from a
  complete **ERN 4.3** build request template; change `version` to build another
  version. The request format is the [build request](./build-request.mdx).

The footer shows the ddex-parser and ddex-builder versions the server runs.

## How it works

The page sends your input to an API on Firebase Cloud Functions, which runs the published
`ddex-parser` and `ddex-builder` npm packages (`POST /api/parse`, `POST /api/build`,
`GET /api/health`). Nothing runs in your browser, and the input is not stored: the server
logs only the size of a failed request, not its content.

## Limits

- **5 MB per request.** Larger input is refused before it is sent. Install the packages to
  process larger files.
- **Short error messages.** A failed parse reports only that the XML could not be parsed
  as a DDEX ERN message. A failed build reports the first missing field (for example
  `ERN 4.3 requires Release/ReleaseType (releases[0].release_type)`); the library itself
  lists every one.
- **The flat model only.** The playground shows the Node.js output, so the Rust graph model
  and the Python DataFrames are not available there.
- **Fair use.** The API runs on a small number of instances with a 30-second timeout.
