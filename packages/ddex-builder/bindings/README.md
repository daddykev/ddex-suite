# DDEX Builder language bindings

The bindings in this directory expose the Rust `ddex-builder` to other languages. Each one
takes the core build request (`BuildRequest`, snake_case field names), runs the same
builder with the same required-field checks, and writes the same output.

| Binding | Directory | Package | Documentation |
|---|---|---|---|
| Node.js (napi-rs) | `node/` | [`ddex-builder` on npm](https://www.npmjs.com/package/ddex-builder) | [node/README.md](node/README.md) |
| Python (PyO3) | `python/` | [`ddex-builder` on PyPI](https://pypi.org/project/ddex-builder/) | [python/README.md](python/README.md) |
| WebAssembly (wasm-bindgen) | `wasm/` | Not published; build it with `wasm-pack` | [wasm/README.md](wasm/README.md) |

- Node.js 22 or newer; prebuilt binaries for macOS (arm64, x64), Linux x64 (glibc) and
  Windows x64.
- Python 3.11 or newer; abi3 wheels for Linux x86_64 and aarch64 (glibc 2.28+), Linux
  x86_64 (musl), macOS x86_64 and arm64, and Windows x64. The Python binding also builds
  from a pandas DataFrame (`from_dataframe`).
- The Node.js and Python bindings map a parsed message to a build request
  (`requestFromXml` / `request_from_xml`); the WASM binding does not include the parser.

The request types are in `node/build-request.d.ts` (also used by the WASM build) and
`python/python/ddex_builder/build_request.py`. Measured build times for the native builder
are in [dev-docs/PERFORMANCE.md](https://github.com/daddykev/ddex-suite/blob/main/dev-docs/PERFORMANCE.md);
the bindings were not benchmarked separately.

Changes: [CHANGELOG.md](https://github.com/daddykev/ddex-suite/blob/main/CHANGELOG.md).
Open bugs: [KNOWN_ISSUES.md](https://github.com/daddykev/ddex-suite/blob/main/KNOWN_ISSUES.md).

## License

MIT
