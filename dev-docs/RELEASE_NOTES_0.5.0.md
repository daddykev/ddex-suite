# DDEX Suite 0.5.0 release notes

Date: 2026-11-22. Every package is released at 0.5.0: crates (`ddex-core`, `ddex-parser`,
`ddex-builder`), npm (`ddex-parser`, `ddex-builder` and their platform packages), PyPI
(`ddex-parser`, `ddex-builder`) and the WASM builder. Full list: `CHANGELOG.md` [0.5.0].

**0.5.0 contains breaking changes** (listed below).
Read "Upgrading" before you update.

## Highlights

- **Parser completeness.** Release dates, P and C lines, genres, parental warnings,
  labels, `MessageControlType`, deal end dates and the 3.8.2 territory details are read
  in ERN 3.8.2, 4.2 and 4.3. Absent values stay empty: nothing is invented.
- **Text kept as written.** Values follow their XSD `whiteSpace` facet: strings keep
  their whitespace; only references are collapsed.
- **No hidden mode switch.** `parse()` uses the complete parser for every input size. It
  used to switch to a streaming path above 10 MB, which returned fewer fields.
- **ERN 3.8.2 builder output is schema-valid.** The WASM builder now runs the core builder.
- **Bindings that fail loudly.**
  - Python: importing without the native extension raises `ImportError` with the cause.
  - Node: parse options that had no effect throw.
- **Packages for every common platform.**
  - Wheels: manylinux x86_64 / aarch64, musllinux x86_64, macOS Intel / Apple Silicon,
    Windows x64, plus sdists.
  - npm binaries: darwin-arm64, darwin-x64, linux-x64-gnu, win32-x64-msvc.
  - 0.4.5 had a macOS arm64 wheel only.
- **Measured performance.** The documentation quotes criterion bench results only
  (`dev-docs/PERFORMANCE.md`).

## Upgrading

### All languages
- Text is no longer trimmed. If you compared titles or names after the parser trimmed
  them, trim them yourself.
- Large inputs (over 10 MB) now return the complete model. They take about as long per
  MB as small inputs (about 20–24 MiB/s).

### Node.js (`ddex-parser`)
- **Remove every parse option.** Each field of `ParseOptions` now throws, for example
  `parseSync(xml, { collectStatistics: true })`.
  - Streaming options: `mode`, `autoThreshold`, `enableStreaming`, `streamingThreshold`.
  - Options that had no effect: `resolveReferences`, `includeRaw`, `maxMemory`,
    `timeoutMs`, `allowBlocking`, `chunkSize`, `fidelityLevel`, `preserve*`,
    `canonicalization`, `collectStatistics`, `validationLevel`, `extensionValidation`,
    `enableChecksums`, `memoryLimit`, `enableDetailedErrors`.
  - `parse(xml)` and `parseSync(xml)` without options behave as before.
- `ParsedMessage.statistics` and `fidelityInfo` were removed. They held zeros and an echo
  of the options.
- `DdexParser.stream()` was removed: it returned hard-coded test data. Use `parse` /
  `parseSync` instead.
- `ParsedMessage.resources` values use camelCase keys, for example
  `resources.A1.resource_id` → `resources.A1.resourceId`.
- `displayArtist`, `artist`, `senderName` and `recipientName` are optional.

### Python
- `import ddex_parser` / `import ddex_builder` raise `ImportError` if the native extension
  cannot load. The message names the cause and the fix: reinstall a wheel for your
  platform, or run `maturin develop` in a checkout.
  - The mock classes and fake results are gone.
  - `ddex_builder.is_rust_available()` always returns `True`.
- `__version__` reports the installed package version.
- The flat DataFrame's `sender` column holds the sender's name.

### WASM builder
- `build(request)` takes the core `BuildRequest` and is synchronous.
- These members returned placeholder values and were removed: `batchBuild`, the
  `Release` / `Resource` classes and the preset and fidelity methods.

### Rust
- New optional fields on the graph and flat models. Struct literals that construct these
  types need the new fields.
- `ParseMode::Auto` no longer selects `Stream`.

## Performance

Measured on an Apple M1 Pro with `cargo bench` (release profile):

| Operation | Time |
|---|---:|
| Parse `1 Audio.xml` (75 KB) | 3.4 ms |
| Parse official ERN 4.3 samples (10–330 KB) | 0.5–14 ms |
| Parse a ~10 MB ERN 4.3 catalog | 441 ms |
| Build ERN 4.3, 1 / 12 / 100 tracks | 2.7 / 11.4 / 80 ms |

- The earlier figures were not reproduced and are withdrawn: 25–30 / 500–700 /
  1,265 MB/s, sub-3 ms parsing and "6.25× on 8 cores".
- The parallel streaming iterator does not currently scale with the thread count: 1 to
  8 threads all take 11.6 ms. Fixing this is planned for 0.6.0.

## Known issues

`KNOWN_ISSUES.md` lists the bugs pinned by ignored tests and the ones without a test.
The most visible ones:
- The `SecurityConfig::relaxed()` fast streaming path returns no releases.
- The ddex-builder CLI `build` writes a placeholder document.
- The Python `detect_version` returns `"V4_3"`.
- No npm binaries for Linux musl or arm64.

## Verification

- CI on the release branch:
  - Rust: fmt, clippy, workspace tests, XSD validation.
  - Bindings: Node, Python 3.9–3.13.
  - The ERN 4 oracle.
- The **Release build** workflow builds every published artifact and install-tests each
  one on a matching runner. See `PUBLISH.md` for the publishing order.
