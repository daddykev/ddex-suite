# Changelog - v0.3.0

## [0.3.0] - 2025-01-XX

### 🎉 Major Improvements

#### Python Bindings - Now Production Ready!
- **BREAKING**: Replaced mock implementation with native PyO3 bindings
- Full native performance: <50ms parsing for 10MB files
- Complete pandas DataFrame integration with 3 schema options
- Fixed all compilation issues across macOS/Linux/Windows
- Added Python 3.8+ support with abi3 compatibility

#### DataFrame Integration (Python)
- Added `ParsedERNMessage.to_dataframe()` method
- Implemented three DataFrame schemas:
  - `flat`: Mixed message/release rows (default)
  - `releases`: One row per release with full details
  - `tracks`: One row per track with release context
- Fixed column consistency across all DataFrame methods
- Added `DdexBuilder.from_dataframe()` for round-trip support

### 🐛 Bug Fixes
- Fixed namespace detection in parser (`test_default_namespace_detection`)
- Fixed namespace resolution using document namespaces
- Resolved StreamIterator using real data instead of mock
- Fixed Duration type mismatches in Python bindings
- Corrected mutable/immutable borrow conflicts

### ⚠️ Known Issues
- Canonicalization may drop text content in some cases (fix planned for v0.4.0)
- Some documentation tests need updating
- WASM builds require additional setup

### 📦 Package Updates
All packages updated to v0.3.0:
- `ddex-core`: 0.3.0 (crates.io)
- `ddex-parser`: 0.3.0 (npm, PyPI, crates.io)
- `ddex-builder`: 0.3.0 (npm, PyPI, crates.io)

### 💔 Breaking Changes
- Python: `format` parameter renamed to `schema` in DataFrame methods
- Python: `ParseResult` now returns `PyParsedERNMessage` type
- Python: Mock implementation removed, all methods now use native code

### 📈 Performance Improvements
- Python parsing now achieves <50ms for 10MB files (previously mock)
- Memory usage optimized with bounded allocation
- GIL released during intensive operations

### 📚 Documentation
- Added DataFrame schema specifications
- Updated Python integration examples
- Enhanced API documentation for all bindings

---
**Full Diff**: https://github.com/daddykev/ddex-suite/compare/v0.2.5...v0.3.0