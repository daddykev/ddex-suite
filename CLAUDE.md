# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DDEX Suite is a high-performance DDEX XML processing toolkit built in Rust with bindings for JavaScript/TypeScript, Python, and WASM. It consists of two main components: `ddex-parser` (for parsing DDEX XML) and `ddex-builder` (for generating deterministic DDEX XML), both sharing a common core library.

**Current Status**: v0.2.0 Published - Complete Suite Integration ✅
- **Parser**: v0.2.0 published to npm and PyPI with full language bindings
- **Builder**: v0.2.0 published with deterministic output and Python bindings working
- **Python Bindings**: ✅ Fully functional for both ddex-parser and ddex-builder
- **Node.js Bindings**: ✅ Native binaries with TypeScript definitions
- **WASM**: ✅ Browser-ready bundle at 166KB (67% under target)
- **Round-trip**: ✅ Complete capability with 94 core tests passing
- **Target**: v1.0.0 official release in Q1 2026

## Architecture

This is a Rust workspace with the following structure:
- `packages/core/` - Shared DDEX data models and utilities
- `packages/ddex-parser/` - DDEX XML parser with CLI
- `packages/ddex-builder/` - DDEX XML builder with DB-C14N/1.0 canonicalization
- `packages/*/bindings/` - Language bindings (Node.js, Python, WASM)

The project provides both "graph" (faithful DDEX structure) and "flattened" (developer-friendly) representations with full round-trip fidelity.

## Distribution Channels

- **NPM**: https://www.npmjs.com/package/ddex-builder
- **PyPI**: https://pypi.org/project/ddex-builder/0.1.0/
- **GitHub**: https://github.com/daddykev/ddex-suite

## Common Commands

### Building and Testing
```bash
# Build entire workspace
cargo build

# Run all tests
cargo test

# Run parser CLI
cargo run --bin ddex-parser -- parse input.xml

# Run builder CLI
cargo run --bin ddex-builder -- build input.json output.xml

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
# Node.js bindings (published to npm)
cd packages/ddex-parser/bindings/node
npm install
npm test

cd packages/ddex-builder/bindings/node
npm install
npm test

# Python bindings (published to PyPI)
cd packages/ddex-parser/bindings/python  
maturin develop
python -m pytest

cd packages/ddex-builder/bindings/python
maturin develop
python -m pytest

# WASM bindings (browser-ready)
cd packages/ddex-parser/bindings/wasm
wasm-pack build

cd packages/ddex-builder/bindings/wasm
wasm-pack build
```

## Key Technical Details

### Determinism Requirements
- The builder enforces deterministic output using IndexMap throughout
- No HashMap or HashSet allowed in output paths (enforced by clippy.toml)
- DB-C14N/1.0 canonicalization for byte-perfect XML reproduction
- Content-based deterministic IDs for all elements

### Security Features
- XXE (XML External Entity) protection built into parser core
- Entity expansion limits and deep nesting protection
- Memory-bounded streaming for large files
- Supply chain security with cargo-deny and SBOM

### Performance Targets (Achieved)
- Parse 10KB: <5ms ✅
- Parse 100KB: <10ms ✅
- Parse 1MB: <50ms ✅
- Parse 100MB: <5s ✅
- Stream 1GB: <60s with <100MB memory ✅
- Build typical release: <15ms 🔄 (currently ~0.27s)
- WASM bundle: <500KB ✅ (166KB achieved)
- Round-trip fidelity: 100% 🔄 (basic tests passing)
- Deterministic output: 100% identical 🔄 (basic tests passing)

### Testing Strategy
- Golden file tests using `insta` crate for snapshot testing
- Round-trip tests ensuring Parse → Modify → Build fidelity
- Cross-platform determinism tests
- Security vulnerability tests for XML attacks
- 94 core tests passing across both packages

### Current Features

#### Parser Features ✅
- Full ERN 3.8.2, 4.2, and 4.3 support
- Graph and flattened models
- Extension preservation for round-trip fidelity
- DataFrame integration for Python
- Streaming support for large files
- Comprehensive error reporting

#### Builder Features ✅
- Deterministic XML generation
- DB-C14N/1.0 canonicalization
- Preflight validation with detailed errors
- Partner presets (Spotify, YouTube)
- Multi-version support (3.8.2, 4.2, 4.3)
- Streaming writer for large documents
- DataFrame→DDEX for Python

### Python Integration

Both `ddex-parser` and `ddex-builder` have full Python support with PyO3 0.21:

```python
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

# Parse to structured data
parser = DDEXParser()
message = parser.parse(xml_content)

# Export to DataFrame for analysis
df = parser.to_dataframe(xml_content)

# Build from DataFrame
builder = DDEXBuilder()
xml = builder.from_dataframe(df, version='4.3')

# Round-trip with modifications
result = parser.parse(xml_content)
result.flat.releases[0].title = "Updated Title"
new_xml = builder.build(result.toBuildRequest())
```

### Node.js/TypeScript Integration

```typescript
import { DDEXParser } from 'ddex-parser';
import { DDEXBuilder } from 'ddex-builder';

// Parse DDEX XML
const parser = new DDEXParser();
const result = await parser.parse(xmlContent);

// Modify the parsed data
result.flat.releases[0].title = "Updated Title";

// Build deterministic XML
const builder = new DDEXBuilder();
const xml = await builder.build(result.toBuildRequest());
```

## Development Notes

- Use `cargo test` for regular development
- Both parser and builder are production-ready and published
- Python bindings are fully functional with PyO3 0.21 compatibility
- Focus is currently on documentation and tutorials (Phase 4)
- All XML generation uses deterministic ordering and stable hash IDs
- Round-trip fidelity is a core requirement - never break Parse → Build → Parse equality
- Enhanced CLI features available for both parser and builder

## Dependencies

Primary Rust dependencies:
- `quick-xml` - XML parsing
- `serde` - Serialization
- `chrono` - Date/time handling  
- `thiserror` - Error handling
- `indexmap` - Deterministic ordering
- `insta` - Snapshot testing
- `sha2` - Hash generation for deterministic IDs

Bindings use:
- `napi-rs` - Node.js native bindings ✅
- `PyO3 0.21` - Python bindings ✅
- `wasm-bindgen` - WebAssembly bindings ✅

## Next Steps (Q1 2026)

1. Create unified documentation site
2. Build interactive tutorials and demo videos
3. Setup community channels (Discord/Slack)
4. Official v1.0.0 release announcement
5. Advanced features roadmap:
   - Full DB-C14N/1.0 specification implementation
   - Enterprise features (multi-tenant, audit logs)
   - Cloud-native deployment options
   - Visual DDEX editor/viewer

## Contributing

The project is currently in active development. Community contributions will be welcomed starting in Q1 2026 once the core architecture stabilizes. Follow the project at https://github.com/daddykev/ddex-suite for updates!

## License

MIT License - See LICENSE file for details