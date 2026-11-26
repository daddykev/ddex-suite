# Security policy

## Reporting a vulnerability

Please do not open a public GitHub issue for a security problem. Email
[security@ddex-suite.org](mailto:security@ddex-suite.org) with a description, steps to
reproduce (a minimal XML message or build request helps most), the affected package and
version, and the impact you see. Fixes are released as patch versions and described in
[CHANGELOG.md](CHANGELOG.md).

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

Not in place yet: SBOMs, build provenance or signed artifacts, and fuzzing in CI (fuzz
targets exist in `packages/ddex-builder/fuzz` and run only by hand with `cargo fuzz`).
SBOMs and signed artifacts are on the roadmap before 1.0.
