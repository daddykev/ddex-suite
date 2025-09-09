# Claude Code Task: Implement Phase 3.3 Builder Bindings

## Overview
Implement language bindings for the DDEX Builder package to provide native performance across Node.js, Python, and web browsers.

## Project Structure
Work in the `packages/ddex-builder/bindings/` directory.

## Task 1: Node.js Binding (napi-rs)

### Location: `bindings/node/`

### Files to create:

1. **Cargo.toml**
```toml
[package]
name = "ddex-builder-node"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
ddex-builder = { path = "../..", features = ["ffi"] }
ddex-core = { path = "../../../core", features = ["typescript"] }
napi = { version = "2", features = ["async", "serde-json", "tokio_rt"] }
napi-derive = "2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["rt-multi-thread"] }

[build-dependencies]
napi-build = "2"
```

2. **src/lib.rs** - Implement:
   - `DdexBuilder` class with methods: new(), addRelease(), addResource(), build(), validate(), getStats(), reset()
   - `batch_build()` function for multiple documents
   - `validate_structure()` function
   - Use napi macros for JavaScript bindings

3. **build.rs**
```rust
extern crate napi_build;
fn main() {
    napi_build::setup();
}
```

4. **package.json**
   - Name: `@ddex-suite/builder`
   - Main exports with TypeScript support
   - napi configuration for multi-platform

5. **index.d.ts** - TypeScript definitions:
   - DdexBuilder class
   - Release and Resource interfaces
   - ValidationResult, BuilderStats types
   - Exported functions

6. **test.js** - Test suite covering all functionality

## Task 2: Python Binding (PyO3)

### Location: `bindings/python/`

### Files to create:

1. **Cargo.toml**
```toml
[package]
name = "ddex-builder-python"
version = "0.1.0"
edition = "2021"

[lib]
name = "ddex_builder"
crate-type = ["cdylib"]

[dependencies]
ddex-builder = { path = "../..", features = ["ffi"] }
ddex-core = { path = "../../../core" }
pyo3 = { version = "0.20", features = ["extension-module", "abi3-py38"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
pythonize = "0.20"
```

2. **src/lib.rs** - Implement:
   - `DdexBuilder` class with methods: new(), add_release(), add_resource(), build(), validate(), get_stats(), reset()
   - `from_dataframe()` method for pandas DataFrame integration
   - `batch_build()` function
   - `validate_structure()` function
   - Use PyO3 macros and proper error handling

3. **pyproject.toml**
   - Use maturin as build backend
   - Package name: `ddex-builder`
   - Python 3.8+ support

4. **test_builder.py** - Test suite including DataFrame tests

## Task 3: WASM Binding (wasm-bindgen)

### Location: `bindings/wasm/`

### Files to create:

1. **Cargo.toml**
```toml
[package]
name = "ddex-builder-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
ddex-builder = { path = "../..", features = ["wasm"] }
ddex-core = { path = "../../../core", features = ["typescript"] }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }
serde-wasm-bindgen = "0.6"
console_error_panic_hook = "0.1"
```

2. **src/lib.rs** - Implement:
   - `WasmDdexBuilder` class with wasm-bindgen
   - All core methods: new(), addRelease(), addResource(), build(), validate(), getStats(), reset()
   - `batchBuild()` async function
   - `validateStructure()` function
   - Console error panic hook for debugging

3. **test.html** - Interactive browser test suite

## Common Requirements

### Data Structures

**Release** (all bindings):
- releaseId: string
- releaseType: string  
- title: string
- artist: string
- label?: string (optional)
- catalogNumber?: string (optional)
- upc?: string (optional)
- releaseDate?: string (optional)
- genre?: string (optional)
- parentalWarning?: boolean (optional)
- trackIds: string[]
- metadata?: Map/Dict of string->string (optional)

**Resource** (all bindings):
- resourceId: string
- resourceType: string
- title: string
- artist: string
- isrc?: string (optional)
- duration?: string (optional)
- trackNumber?: number (optional)
- volumeNumber?: number (optional)
- metadata?: Map/Dict of string->string (optional)

### Supported Versions
- ERN 4.3 (default)
- ERN 4.2
- ERN 3.8.2

### Message Profiles
- AudioAlbum (default)
- AudioSingle

## Additional Files

1. **bindings/README.md** - Documentation with:
   - Installation instructions for each platform
   - Usage examples for all three bindings
   - API reference
   - Performance benchmarks

2. **build-bindings.sh** - Build script:
```bash
#!/bin/bash
echo "Building DDEX Builder bindings..."

# Build Node.js bindings
cd bindings/node
npm run build
cd ../..

# Build Python bindings  
cd bindings/python
maturin build --release
cd ../..

# Build WASM bindings
cd bindings/wasm
wasm-pack build --target web --out-dir pkg
cd ../..

echo "All bindings built successfully!"
```

## Success Criteria
- All three bindings compile without errors
- Tests pass for each binding
- TypeScript/Python types are correctly generated
- WASM bundle is <500KB
- APIs are consistent across all platforms
- DataFrame integration works in Python
- Async operations work in Node.js and WASM

## Notes
- Ensure all bindings reference the parent ddex-builder and ddex-core crates correctly
- Use consistent naming conventions (camelCase for JS/TS, snake_case for Python)
- Include proper error handling and helpful error messages
- Add logging/debugging support where appropriate