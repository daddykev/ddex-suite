# ddex-builder security helpers

`DDEXBuilder::build` does not depend on the helpers below. They are opt-in APIs in
`ddex_builder::security` and `ddex_builder::api_security`:

- `PathValidator` / `PathValidationConfig`: path traversal and platform-specific path checks.
- `InputValidator`: size, string, URL, XML and JSON content checks (regex-based).
- `SecureXmlReader`: a quick-xml reader with depth, element-count, size and DTD limits.
- `OutputSanitizer`, `RateLimiter`, `SecureTempFile`, and `ApiSecurityManager` /
  `FfiValidator` for API boundaries.

Their known problems are listed, with the tests that pin each one, in
[KNOWN_ISSUES.md](../../KNOWN_ISSUES.md) and summarised in the website's
[security model](https://ddex-suite.org/docs/security#known-security-issues).

The entity classifier (`EntityClassifier` and the `InputValidator` entity methods) and the
error sanitizer (`ErrorSanitizer`, `SanitizedError`, `SanitizerConfig` and related types)
were removed in 0.6.0: the builder did not use them, and they gave incorrect results under
some configurations.

How the parser and builder handle XML entities and DTDs is described in the website's
[security model](https://ddex-suite.org/docs/security).

To report a security problem, see the repository's
[SECURITY.md](https://github.com/daddykev/ddex-suite/blob/main/SECURITY.md).
