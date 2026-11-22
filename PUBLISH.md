# Publishing DDEX Suite

Every package is released at the same version: `[workspace.package] version` in the root
`Cargo.toml` (0.4.6 at the time of writing). Nothing is published automatically. The
**Release build** workflow (`.github/workflows/release-build.yml`, run by hand with
"Run workflow") builds and install-tests the npm and PyPI artifacts. You download them
and publish by hand in the order below.

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
      XSD, Node, Python 3.9–3.13, ERN 4 oracle).

## 2. Build the artifacts

Run **Release build** on the release branch. It produces:

| Artifact | Contents |
|---|---|
| `wheel-<package>-<platform>` | abi3 wheels: manylinux x86_64 / aarch64, musllinux x86_64, macOS x86_64 / arm64, Windows x64 |
| `sdist-<package>` | source distributions |
| `node-<package>-<platform>` | `<package>-node.<platform>.node` for darwin-arm64, darwin-x64, linux-x64-gnu, win32-x64-msvc |

Each wheel, the sdists and each Node platform are install-tested on a matching runner
(`scripts/release_smoke.py`, `scripts/release_smoke.js`: import, parse an official
sample, build an ERN 4.3 message and parse it back). Do not publish unless every job is
green.

## 3. Publish, in this order

### crates.io

```bash
cargo publish -p ddex-core
cargo publish -p ddex-parser
cargo publish -p ddex-builder
```

### npm

For each package (`ddex-parser`, `ddex-builder`), publish the four platform packages
first, then the main package (its `optionalDependencies` name the platform packages at
the same version):

```bash
cd packages/ddex-parser/bindings/node
for p in darwin-arm64 darwin-x64 linux-x64-gnu win32-x64-msvc; do
  cp /path/to/artifacts/node-ddex-parser-$p/ddex-parser-node.$p.node npm/$p/
  (cd npm/$p && npm publish --access public)
done
npm publish --access public
```

Repeat in `packages/ddex-builder/bindings/node` with `ddex-builder-node.<platform>.node`.

### PyPI

```bash
python -m pip install twine
twine upload wheel-ddex-parser-*/*.whl sdist-ddex-parser/*.tar.gz
twine upload wheel-ddex-builder-*/*.whl sdist-ddex-builder/*.tar.gz
```

## 4. After publishing

- [ ] Tag the release: `git tag v<version> && git push origin v<version>`.
- [ ] Install from the registries on a clean machine (`npm install ddex-parser ddex-builder`,
      `pip install ddex-parser ddex-builder`) and run the smoke scripts against them.
- [ ] Update the website (`cd website && npm run build && firebase deploy`).
