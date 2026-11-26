# Security policy

## Reporting a vulnerability

Please do not open a public GitHub issue for a security problem. Report it privately on
GitHub: **[Report a vulnerability](https://github.com/daddykev/ddex-suite/security/advisories/new)**
(the repository's **Security** tab, then "Report a vulnerability"). Only you and the
maintainer can see the report and the discussion that follows.

Include a description, steps to reproduce (a minimal XML message or build request helps
most), the affected package and version, and the impact you see. Fixes are released as
patch versions and described in [CHANGELOG.md](CHANGELOG.md).

## Supported versions

DDEX Suite is pre-1.0. Security fixes go into the latest release line, currently 0.5.x
(crates `ddex-core`, `ddex-parser`, `ddex-builder`; npm and PyPI packages `ddex-parser`,
`ddex-builder`). Older lines are not patched.

## What the libraries defend against

DDEX messages come from partners, so the parser treats its input as untrusted and defends
against XML-level threats. With the default configuration (`SecurityConfig::strict()`;
the Node.js and Python bindings always use it):

- DTDs are rejected, and with them internal entity declarations and external entities
  (XXE). Nothing is ever fetched from files or the network.
- Entity expansion, element depth, attribute and text sizes and input size are limited.
- quick-xml is 0.41 or newer, which fixes the quadratic duplicate-attribute check
  (RUSTSEC-2026-0194); a regression test parses a start tag with 50,000 attributes.

Metadata that looks like SQL, HTML or script is legitimate text: the parser returns it
unchanged and the builder writes it XML-escaped. Escaping it for a database, web page or
shell is up to the caller. Neither package validates against the DDEX XSD or the AVS rules
at runtime.

The details, with the exact limits, are in the
[security model](https://ddex-suite.org/docs/security) page of the documentation.

## Known security issues

Some helper APIs of `ddex-builder` that `build` does not use have open bugs:
`PathValidator`, `InputValidator`, `OutputSanitizer` and the secure log messages. Each is
pinned by an ignored test and listed in [KNOWN_ISSUES.md](KNOWN_ISSUES.md), with the
dependency advisories that are still open (unmaintained crates).

## How the code and its dependencies are checked

What runs today (`.github/workflows/ci.yml` and `release-build.yml`):

- `Cargo.lock` is committed.
- `cargo-deny` runs in CI on the whole workspace (including the Node, Python and WASM
  binding crates): RustSec advisories (vulnerabilities, unmaintained and unsound crates),
  banned crates, the license allow-list and allowed sources, with the policy in
  [`deny.toml`](deny.toml). An advisory may be ignored only with a written reason and a
  KNOWN_ISSUES.md entry.
- The test suite includes XML attack tests (XXE, entity expansion, deep nesting, malformed
  XML, the attribute flood above).
- `ddex-builder` is `#![forbid(unsafe_code)]`.
- Every wheel and npm package built for a release is installed on its platform and
  smoke-tested with an official sample before the (manual) publish.
- From 0.5.1, release artifacts are built in GitHub Actions with signed build-provenance
  attestations and SBOMs, verifiable with
  `gh attestation verify <file> --repo daddykev/ddex-suite`, provided the published files
  are the ones release-build produced. The `.crate`, npm `.tgz`, wheel and sdist files each
  have a SLSA build-provenance attestation and a CycloneDX SBOM attestation; PUBLISH.md
  publishes the downloaded, verified files. Publishing is done by hand, not from CI, so
  there is no npm `--provenance` and no PyPI trusted-publishing attestation.

Not in place yet: fuzzing in CI (fuzz targets exist in `packages/ddex-builder/fuzz` and run
only by hand with `cargo fuzz`).
