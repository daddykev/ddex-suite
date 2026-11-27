# CLAUDE.md — DDEX Suite

## Principles (non-negotiable)
- Never invent data. Absent values stay None/null. Missing required build fields fail with an
  error naming the field. No fake success, placeholder scores or canned results.
- Every public claim (READMEs, docs, rustdoc, registry pages, website) must be true at the
  released version. No unmeasured performance numbers.
- Tests must mean something: real fixtures, independent checks. Known bugs are
  `#[ignore = "known bug: …"]` plus a KNOWN_ISSUES.md entry.
- Zero rustc warnings, both by default and with `--features perf-tests`.
- Every breaking change gets a CHANGELOG entry saying who is affected, plus a migration note.

## Scope discipline (read this before every task)
1. Do exactly what the prompt lists. No extra removals, refactors, renames, audits, triage or
   "while I'm here" fixes, even small or obviously good ones.
2. Out-of-scope findings: add ONE line to the "## Parked" section at the end of
   KNOWN_ISSUES.md (what, where, why it matters), then move on. Don't investigate further and
   don't fix it.
3. If the task can't be finished without going out of scope, stop and ask.
4. No subagents and no repo-wide sweeps unless the prompt asks for them.
5. If a prompt seems to expand scope beyond its stated goal, say so at the top of your report.
6. Size: if a PR would change more than ~60 files, stop before pushing and report.
7. Keep reports short: PR URL, CI status and rounds, what changed, what was parked,
   deviations. Leave out narration and inventories unless asked.

## Lean rules (the laptop's disk is limited)
- No full workspace builds, no test runs, no benches, no local clippy.
- Allowed: `cargo fmt`, then one `cargo check -p <crate> --all-targets` for each changed crate
  (a second run only to confirm a fix the first run caught), then `cargo clean`.
- Don't filter check output. Scratch files go in the session scratchpad.
- CI is the verification.
- Rust toolchain: `export PATH="/opt/homebrew/opt/rustup/bin:$PATH"`

## PR workflow
- One branch → one PR → CI green. Watch with `gh pr checks --watch`. Get failures with
  `gh run view <id> --log-failed`. Stop after 3 unexplained failures of the same check.
- Avoid stacked PRs. Start from an up-to-date main after the previous PR is merged.
- Never merge, tag, publish or deploy. Kev does that.
- Commit messages and PR bodies end with the attribution lines the session provides.

## Repo notes
- Rust workspace: `packages/core` (ddex-core: shared models, `xml_refs`),
  `packages/ddex-parser`, `packages/ddex-builder`, and bindings under
  `packages/*/bindings/{node,python,wasm}`. `website/` is the Docusaurus docs site.
- Parser and builder are separate concerns; avoid coupling them.
- ddex-builder output paths must not use `HashMap` / `HashSet`, clocks or RNGs:
  `packages/ddex-builder/clippy.toml` disallows them and CI enforces it. Remaining uses
  carry an `#[allow(..., reason = "...")]` naming their row in
  `dev-docs/DETERMINISM_AUDIT_0.6.0.md`.
- Measured performance figures live only in `dev-docs/PERFORMANCE.md`, recorded by the Bench
  workflow (`.github/workflows/bench.yml`). Quote nothing that is not there.
- Open bugs: `KNOWN_ISSUES.md`. Changes: `CHANGELOG.md`. Removal inventory for 0.6.0:
  `dev-docs/REMOVALS_0.6.0.md`.
- Versions: `python3 scripts/check_versions.py` must report every reference at the
  workspace version.
- Docs code examples are run in CI: every `js` / `python` / `rust` block in `website/docs`
  and `website/src/examples/*` (`scripts/readme_examples.py --strict`, job `docs-examples`),
  plus the README examples. Non-runnable blocks use `bash`, `text`, `xml`, `json` or `toml`.
- The website build fails on broken links and anchors (CI job `website`).
- CI (`.github/workflows/ci.yml`): fmt, clippy, workspace tests, XSD validation of builder
  output, Node, Python 3.10–3.14, ERN 4 oracle comparison, cargo-deny, docs examples.
