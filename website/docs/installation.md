---
sidebar_position: 2
title: Installation
description: Install ddex-parser and ddex-builder for Rust, Node.js or Python, and the platforms with prebuilt binaries.
---

# Installation

Install the parser, the builder, or both. The two are independent packages and are released
together at the same version (0.5.1).

## Rust

```bash
cargo add ddex-parser ddex-builder
```

or in `Cargo.toml`:

```toml
[dependencies]
ddex-parser = "0.5.1"
ddex-builder = "0.5.1"
```

The minimum supported Rust version is **1.82** (checked in CI). `ddex-core`, the shared
models, comes in as a dependency; add it directly only if you name its types in your own
code.

## Node.js

```bash
npm install ddex-parser ddex-builder
```

Node.js **18 or newer**. TypeScript types are included (`index.d.ts`; the builder also has
`build-request.d.ts`). The packages are native addons: npm installs a prebuilt binary
through an optional platform package (`ddex-parser-<platform>`, `ddex-builder-<platform>`).

Prebuilt binaries:

| Platform | Package suffix |
|---|---|
| macOS Apple Silicon | `darwin-arm64` |
| macOS Intel | `darwin-x64` |
| Linux x64 (glibc) | `linux-x64-gnu` |
| Windows x64 | `win32-x64-msvc` |

There are no npm binaries for Linux arm64 or Linux musl (Alpine) yet.

## Python

```bash
pip install ddex-parser ddex-builder
pip install "ddex-parser[pandas]" "ddex-builder[pandas]"   # for the DataFrame APIs
```

Python **3.10 to 3.14** (CI tests each one). The wheels use the stable ABI (`cp310-abi3`),
so one wheel per platform covers every supported Python version.

Prebuilt wheels:

| Platform | Wheel |
|---|---|
| Linux x86_64 (glibc 2.28+) | `manylinux_2_28_x86_64` |
| Linux aarch64 (glibc 2.28+) | `manylinux_2_28_aarch64` |
| Linux x86_64 (musl, e.g. Alpine) | `musllinux_1_2_x86_64` |
| macOS Intel (10.12+) | `macosx_10_12_x86_64` |
| macOS Apple Silicon (11+) | `macosx_11_0_arm64` |
| Windows x64 | `win_amd64` |

On other platforms pip builds from the source distribution, which needs a Rust toolchain.

The packages need their native extension. If it cannot be loaded, `import ddex_parser`
(or `import ddex_builder`) raises an `ImportError` that names the cause; there is no
pure-Python fallback.

Every wheel and npm package is installed and smoke-tested on a matching runner (parse and
build an official sample) before a release is published.

## WebAssembly

The WASM builds in `packages/*/bindings/wasm` are not published to npm. Build them from a
checkout with `wasm-pack`. The playground does not use them; it calls the Node.js packages
on a server (see [Playground](./playground.md)).

## Check the installation

```bash
npm ls ddex-parser ddex-builder
python -c "import ddex_parser, ddex_builder; print(ddex_parser.__version__, ddex_builder.__version__)"
```

Then continue with the [parsing quick start](./quick-start-parsing.mdx).
