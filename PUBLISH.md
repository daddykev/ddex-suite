# Publishing DDEX Suite

Every package is released at the same version: `[workspace.package] version` in the root
`Cargo.toml` (0.5.1 at the time of writing). Nothing is published automatically. The
**Release build** workflow (`.github/workflows/release-build.yml`, run by hand with
"Run workflow") builds and install-tests every release file, writes CycloneDX SBOMs, and
signs a build-provenance attestation for each file. You download those files, verify them,
and publish **those exact files** by hand in the order below.

**Never build or pack a release file on your own machine.** The attestations prove where a
file was built only for the files the workflow produced; a file built locally has none.
crates.io is the one exception that cannot be avoided (`cargo publish` always packages from
source): there you re-package the same commit and publish only if the checksums match the
attested `.crate` files (step 3).

## 1. Before the release

- [ ] Bump the version everywhere, then check it:
      `python3 scripts/check_versions.py` must end with "all N version references are X".
      It covers the Cargo manifests (crates and binding crates), the Node packages and
      their `npm/<platform>` packages and `optionalDependencies`, the Python
      `pyproject.toml`s (Python `__version__` comes from the package metadata), the WASM
      package.json and the website.
- [ ] `CHANGELOG.md` has a dated section for the version; release notes in
      `dev-docs/RELEASE_NOTES_<version>.md`.
- [ ] CI is green on the release branch (`.github/workflows/ci.yml`: fmt, clippy, tests,
      XSD, Node, Python 3.10–3.14, ERN 4 oracle).

## 2. Build, download and verify the artifacts

Run **Release build** on the release branch. It produces:

| Artifact | Contents |
|---|---|
| `crates` | `ddex-core`, `ddex-parser`, `ddex-builder` `.crate` files and `cargo-version.txt` |
| `npm-packages` | the npm tarballs: `ddex-parser` and `ddex-builder`, and their four platform packages each (darwin-arm64, darwin-x64, linux-x64-gnu, win32-x64-msvc) |
| `wheel-<package>-<platform>` | abi3 wheels: manylinux x86_64 / aarch64, musllinux x86_64, macOS x86_64 / arm64, Windows x64 |
| `sdist-<package>` | source distributions |
| `node-<package>-<platform>` | the native binaries inside the platform tarballs |
| `sbom` | CycloneDX 1.5 SBOMs: each crate, and the Rust crate compiled into each npm package and each Python package |

Each wheel, the sdists and each Node platform are install-tested on a matching runner
(`scripts/release_smoke.py`, `scripts/release_smoke.js`: import, parse an official
sample, build an ERN 4.3 message and parse it back); the Node test installs the
`npm-packages` tarballs. Do not publish unless every job is green.

The `attest` job signs a build-provenance attestation for every file above and an SBOM
attestation for each package. Download the whole run and verify every file:

```bash
gh run download <run-id> --repo daddykev/ddex-suite --dir release-artifacts
scripts/verify_release_artifacts.sh release-artifacts
```

It runs `gh attestation verify <file> --repo daddykev/ddex-suite --signer-workflow
daddykev/ddex-suite/.github/workflows/release-build.yml` on each file (and again with
`--predicate-type https://cyclonedx.org/bom` on each package) and stops at the first
failure. Publish nothing if it fails.

macOS: every macOS job runs on `macos-latest` (Apple Silicon). The x86_64 wheels and the
darwin-x64 Node binaries are cross-compiled there (`--target x86_64-apple-darwin`), as
GitHub's last Intel image, `macos-15-intel`, is retired in August 2027. The build jobs
check each macOS native file with `file` and `lipo -archs` (x86_64 or arm64 only, not
universal). The x86_64 artifacts are then install-tested on the same Apple Silicon
runner under Rosetta 2, with python.org's universal2 Python 3.10 run as x86_64 and an
x64 Node 22 (the jobs check `platform.machine()` / `process.arch`). They are never run
on Intel hardware.

## 3. Publish, in this order

### crates.io

`cargo publish` packages from the source tree; it cannot upload a downloaded `.crate`.
cargo packages reproducibly (fixed file times), so check that it produces the attested
files, then publish:

```bash
git switch --detach <the run's commit>          # gh run view <run-id> --json headSha
cat release-artifacts/crates/cargo-version.txt  # use this cargo version
cargo package --no-verify -p ddex-core -p ddex-parser -p ddex-builder
for c in release-artifacts/crates/*.crate; do
  cmp "$c" "target/package/$(basename "$c")" || { echo "MISMATCH: $c"; exit 1; }
done
cargo publish -p ddex-core
cargo publish -p ddex-parser
cargo publish -p ddex-builder
```

Publish nothing if any file differs. After publishing, check what crates.io serves:

```bash
for c in ddex-core ddex-parser ddex-builder; do
  curl -fsSLo "$c.crate" "https://static.crates.io/crates/$c/$c-<version>.crate"
  gh attestation verify "$c.crate" --repo daddykev/ddex-suite
done
```

### npm

Publish the downloaded tarballs, never a package directory (that would pack it again on
your machine). For each package, the four platform packages first, then the main package
(its `optionalDependencies` name the platform packages at the same version):

```bash
cd release-artifacts/npm-packages
for pkg in ddex-parser ddex-builder; do
  for p in darwin-arm64 darwin-x64 linux-x64-gnu win32-x64-msvc; do
    npm publish "$pkg-$p-<version>.tgz" --access public
  done
  npm publish "$pkg-<version>.tgz" --access public
done
```

`npm publish` is run without `--provenance` (npm provenance needs publishing from CI); the
GitHub attestations are the provenance. After publishing, `npm pack <name>@<version>`
downloads a registry tarball for `gh attestation verify`.

### PyPI

Upload the downloaded wheels and sdists (no PyPI trusted-publishing attestations, which
need publishing from CI; the GitHub attestations are the provenance):

```bash
python -m pip install twine
cd release-artifacts
twine upload wheel-ddex-parser-*/*.whl sdist-ddex-parser/*.tar.gz
twine upload wheel-ddex-builder-*/*.whl sdist-ddex-builder/*.tar.gz
```

## 4. After publishing

- [ ] Tag the release: `git tag v<version> && git push origin v<version>`.
- [ ] Install from the registries on a clean machine (`npm install ddex-parser ddex-builder`,
      `pip install ddex-parser ddex-builder`) and run the smoke scripts against them.
- [ ] Refresh the Cloud Functions lockfile, which still resolves the previous release
      (`website/functions/package.json` asks for the new version, which exists only once
      published): `cd website/functions && npm install ddex-parser@<version>
      ddex-builder@<version>`, and commit `package-lock.json`.
- [ ] Check the Cloud Functions' production dependencies before deploying them (they
      parse untrusted input): `cd website/functions && npm ci && npm audit --omit=dev
      --audit-level=high`. Fix any high or critical finding before deploying (update the
      lockfile, or an `overrides` entry in `website/functions/package.json`). CI runs the
      same check, and the handler tests, in the `functions` job.
- [ ] Update the website (`cd website && npm ci && (cd functions && npm ci) && firebase deploy`;
      the hosting `predeploy` in `website/firebase.json` runs `npm run build` first).
