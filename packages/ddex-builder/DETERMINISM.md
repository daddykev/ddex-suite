# Determinism

Byte-for-byte deterministic output is being implemented for 0.6.0. Until then, two builds
of the same request can differ: a message ID, release and resource references and
`MessageCreatedDateTime` that the request leaves out are generated (random UUIDs, the
current time), and the streaming writer does the same for its header. See
[KNOWN_ISSUES.md](../../KNOWN_ISSUES.md) for the current list.

What is in place today:

- Output is canonicalized with DB-C14N/1.0 (fixed namespace prefixes, sorted attributes,
  two-space indentation, elements in XSD order).
- `clippy.toml` disallows `HashMap` / `HashSet`, clocks and random number generators in
  the builder, and CI enforces it (`cargo clippy -p ddex-builder --all-targets`). The
  remaining uses carry an `#[allow]` naming their row in
  `dev-docs/DETERMINISM_AUDIT_0.6.0.md`; the ones that reach output are listed in
  `KNOWN_ISSUES.md`.
- `tests/it/determinism_0_6.rs` pins the 0.6.0 behaviour. The cases that do not hold yet
  are `#[ignore = "known bug: …"]`.

In 0.6.0 determinism is a documented, CI-tested property of `DDEXBuilder::build`, not a
runtime API. The runtime determinism verifier, the determinism-guarantee report (and the
CLI `guarantees` command) and the configuration options that no build read were removed;
see `CHANGELOG.md`.
