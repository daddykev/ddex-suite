# Extensions (partner and custom XML): status

- **Models** (`packages/core/src/models/extensions.rs`): `XmlFragment` and `Extensions`
  hold raw XML fragments, namespace declarations, comments and processing instructions.
  Their unit tests are `models::extensions::tests` and
  `models::extensions::comprehensive_tests` in ddex-core.
- **Parser** (`packages/ddex-parser/src/parser/extension_capture.rs`): extension capture
  for non-DDEX namespaces.
- **Builder**: extensions are **not** written back. The builder's generators do not read
  `BuildRequest.extensions`, so a Parse → Build round trip drops partner and custom
  extensions. `src/extension_restoration.rs`, which described a restoration writer, was
  never compiled (it had no `mod` declaration) and was removed in 0.6.0.

See `KNOWN_ISSUES.md` for what round trips do and do not carry.
