# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DDEX Suite is a high-performance DDEX XML processing toolkit built in Rust with bindings for JavaScript/TypeScript, Python, and WASM. It consists of two main components: `ddex-parser` (for parsing DDEX XML) and `ddex-builder` (for generating deterministic DDEX XML), both sharing a common core library.

**Current Status**: Phase 3.3 - Building Builder Bindings (napi-rs, PyO3, WASM)
- Parser: v0.1.0 published to npm, Python bindings 70% complete
- Builder: Core functionality complete, bindings needed
- Target: v1.0.0 in Q4 2025

## Architecture

This is a Rust workspace with the following structure:
- `packages/core/` - Shared DDEX data models and utilities
- `packages/ddex-parser/` - DDEX XML parser with CLI
- `packages/ddex-builder/` - DDEX XML builder with DB-C14N/1.0 canonicalization
- `packages/*/bindings/` - Language bindings (Node.js, Python, WASM)

The project provides both "graph" (faithful DDEX structure) and "flattened" (developer-friendly) representations with full round-trip fidelity.

## Common Commands

### Building and Testing
```bash
# Build entire workspace
cargo build

# Run all tests
cargo test

# Run parser CLI
cargo run --bin ddex-parser -- parse input.xml

# Run builder tests with snapshots
cd packages/ddex-builder && cargo test

# Test specific package
cd packages/ddex-parser && cargo test

# Parser-specific test script
./scripts/test-all-parser.sh

# Check bundle sizes
./scripts/check-parser-size.sh
```

### Development Workflows
```bash
# Lint and check code quality
cargo clippy -- -D warnings

# Format code
cargo fmt

# Run benchmarks
cargo bench

# Clean build artifacts
cargo clean
```

### Language Bindings
```bash
# Node.js bindings (parser published to npm as v0.1.0)
cd packages/ddex-parser/bindings/node
npm install
npm test

# Python bindings (currently disabled due to PyO3 0.21 compatibility)
cd packages/ddex-parser/bindings/python  
maturin develop

# WASM bindings (under development)
cd packages/ddex-parser/bindings/wasm
wasm-pack build
```

## Key Technical Details

### Determinism Requirements
- The builder enforces deterministic output using IndexMap throughout
- No HashMap or HashSet allowed in output paths (enforced by clippy.toml)
- DB-C14N/1.0 canonicalization for byte-perfect XML reproduction

### Security Features
- XXE (XML External Entity) protection built into parser core
- Entity expansion limits and deep nesting protection
- Memory-bounded streaming for large files
- Supply chain security with cargo-deny and SBOM

### Performance Targets
- Parse 10KB: <5ms, 100KB: <10ms, 1MB: <50ms
- Build typical release: <15ms (currently ~0.27s for test suite)
- WASM bundle: <500KB (achieved)
- Memory-bounded streaming for files >100MB

### Testing Strategy
- Golden file tests using `insta` crate for snapshot testing
- Round-trip tests ensuring Parse → Modify → Build fidelity
- Cross-platform determinism tests
- Security vulnerability tests for XML attacks

### Current Limitations
- Python bindings temporarily disabled due to PyO3 0.21 compatibility issues
- Builder bindings (Node.js, Python, WASM) not yet implemented
- Preset system for partner configurations (Spotify, YouTube) planned but not complete

## Development Notes

- Use `cargo test` for regular development
- The parser is production-ready and published
- The builder core is complete but needs bindings
- Focus is currently on Phase 3.3 (Builder Bindings)
- All XML generation uses deterministic ordering and stable hash IDs
- Round-trip fidelity is a core requirement - never break Parse → Build → Parse equality

## Dependencies

Primary Rust dependencies:
- `quick-xml` - XML parsing
- `serde` - Serialization
- `chrono` - Date/time handling  
- `thiserror` - Error handling
- `indexmap` - Deterministic ordering
- `insta` - Snapshot testing

Bindings use:
- `napi-rs` - Node.js native bindings
- `PyO3` - Python bindings (currently disabled)
- `wasm-bindgen` - WebAssembly bindings