# DDEX Parser for Python

[![PyPI version](https://img.shields.io/pypi/v/ddex-parser.svg)](https://pypi.org/project/ddex-parser/)
[![Python versions](https://img.shields.io/pypi/pyversions/ddex-parser.svg)](https://pypi.org/project/ddex-parser/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-ddex--suite-blue)](https://github.com/daddykev/ddex-suite)

High-performance DDEX XML parser for Python, built on Rust for blazing speed and memory safety. Parse and transform DDEX messages (ERN 3.8.2, 4.2, 4.3) with a Pythonic API and optional pandas integration.

Part of the [DDEX Suite](https://github.com/daddykev/ddex-suite) - a comprehensive toolkit for working with DDEX metadata in the music industry.

## 🚧 v0.1.0 - Foundation Release

This is the initial PyPI release establishing the package structure and Python API. The full parser implementation with native Rust performance is actively in development.

**Current Status:**
- ✅ Package structure with PyO3/maturin setup
- ✅ Python API design finalized
- ✅ Type stubs for IDE support
- ✅ macOS ARM64 compatibility confirmed
- 🚧 Full parser implementation (coming in v0.2.0)
- 🚧 DataFrame integration (coming in v0.2.0)
- 🚧 Streaming support (coming in v0.3.0)

## ✨ Features (Roadmap)

### Available Now (v0.1.0)
- 📦 **Package Structure**: Clean Python package with type hints
- 🎯 **API Design**: Future-proof API that won't break when implementation lands
- 📊 **Dual Model Architecture**: Graph model for compliance, flattened model for ease of use

### Coming Soon
- 🚀 **Blazing Fast** (v0.2.0): Parse typical releases in <50ms with native Rust
- 🐼 **DataFrame Integration** (v0.2.0): Seamless pandas support for data analysis
- 🔄 **Streaming Support** (v0.3.0): Handle gigabyte catalogs with bounded memory
- 🛡️ **Security** (v0.2.0): Built-in XXE protection, entity expansion limits
- 🔗 **Perfect Round-Trip** (v1.0.0): Parse → Modify → Build with [`ddex-builder`](https://github.com/daddykev/ddex-suite)

## 📦 Installation

```bash
pip install ddex-parser
```

### Platform Support

v0.1.0 ships with experimental wheels for:
- ✅ macOS (ARM64 confirmed working)
- 🚧 macOS (x86_64)
- 🚧 Linux (x86_64, aarch64)  
- 🚧 Windows (x86_64)

Full platform support with CI-built wheels coming in v0.2.0.

### Optional Dependencies

```bash
# For DataFrame support (v0.2.0+)
pip install ddex-parser[pandas]

# For development
pip install ddex-parser[dev]
```

## 🚀 Quick Start

```python
from ddex_parser import DDEXParser

# Create parser
parser = DDEXParser()

# API is ready - implementation coming in v0.2.0
result = parser.parse(xml_content)

# Will return mock data in v0.1.0
# Full parsing in v0.2.0
print(f"Message ID: {result.message_id}")
print(f"Releases: {result.release_count}")

for release in result.releases:
    print(f"- {release['title']} by {release['artist']}")
```

## 🎭 Dual Model Architecture

The parser will provide two complementary views of DDEX data:

### Graph Model (Faithful Representation)
Preserves the exact DDEX structure with references - perfect for validation and compliance:

```python
# Access the graph model for full DDEX structure
graph = result.graph
print(graph.message_header.message_id)
print(graph.parties)  # All party definitions
print(graph.releases)  # Releases with references
```

### Flattened Model (Developer-Friendly)
Denormalized and resolved for easy consumption - ideal for applications:

```python
# Access the flattened model for convenience
flat = result.flat
for release in flat.releases:
    print(f"{release['title']} - {release['artist']}")
    for track in release['tracks']:
        print(f"  {track['position']}. {track['title']} ({track['duration']}s)")
```

## 💻 Usage Examples (v0.2.0+)

These examples show the API that will be fully functional in v0.2.0:

### Basic Parsing
```python
parser = DDEXParser()
result = parser.parse(xml_content, {
    'include_raw_extensions': True,  # Preserve unknown XML elements
    'include_comments': True,        # Preserve XML comments  
    'validate_references': True      # Validate all references
})

# Access both models
print(result.graph)  # Full DDEX structure
print(result.flat)   # Simplified view
```

### Async Support (v0.2.0)
```python
import asyncio

async def parse_async():
    parser = DDEXParser()
    result = await parser.parse_async(xml_content)
    return result

result = asyncio.run(parse_async())
```

### DataFrame Export (v0.2.0)
```python
# Convert to pandas DataFrame
df = parser.to_dataframe(xml_content)

# Analyze with pandas
print(df.describe())
print(df.groupby('artist')['title'].count())

# Export to CSV
df.to_csv('releases.csv', index=False)
```

### Streaming Large Files (v0.3.0)
```python
# Stream parse for memory efficiency
for release in parser.stream('huge_catalog.xml'):
    print(f"Processing: {release['title']}")
    # Process one release at a time
```

## 📊 Performance Targets

When fully implemented (v0.2.0+), the parser will achieve:

| File Size | Parse Time | Memory | Mode |
|-----------|------------|--------|------|
| 10KB | <5ms | 2MB | DOM |
| 100KB | <10ms | 5MB | DOM |
| 1MB | <50ms | 20MB | DOM |
| 100MB | <5s | 50MB | Stream |
| 1GB | <60s | 100MB | Stream |

## 🛣️ Development Roadmap

### v0.1.0 (Current Release)
- ✅ Package structure and Python API
- ✅ Type stubs for IDE support
- ✅ Basic wheel distribution

### v0.2.0 (Q1 2025)
- 🚧 Full Rust parser implementation
- 🚧 DataFrame integration
- 🚧 CI-built wheels for all platforms
- 🚧 Comprehensive test suite

### v0.3.0 (Q2 2025)
- 📅 Streaming parser
- 📅 Async support improvements
- 📅 Performance optimizations

### v1.0.0 (Q3 2025)
- 📅 Complete suite with [`ddex-builder`](https://github.com/daddykev/ddex-suite)
- 📅 Perfect round-trip: Parse → Modify → Build
- 📅 Production ready

## 👨‍💻 About This Project

DDEX Suite is being built as a rigorous, production-grade toolkit for music industry metadata processing. It combines a single Rust core with native bindings for JavaScript and Python, showcasing cross-language API design and deep ecosystem integration.

The project tackles the complementary challenges of:
- **Parser**: Transform complex DDEX XML into clean, strongly-typed models
- **Builder**: Generate deterministic, byte-perfect DDEX XML (coming soon)

Built with a focus on:
- 🔒 Security hardening (XXE protection, memory bounds)
- ⚡ Performance optimization (native Rust, streaming)
- 🎯 Developer experience (dual models, type hints)
- 🔄 Perfect round-trip fidelity

## 📚 API Reference

### DDEXParser

Main parser class.

#### Methods

- `parse(xml, options=None)` - Parse XML synchronously
- `parse_async(xml, options=None)` - Parse XML asynchronously (v0.2.0)
- `stream(source, options=None)` - Stream parse large files (v0.3.0)
- `to_dataframe(xml, schema='flat')` - Convert to pandas DataFrame (v0.2.0)
- `detect_version(xml)` - Detect DDEX version
- `sanity_check(xml)` - Validate XML structure

### ParseOptions

Configuration for parsing.

- `include_raw_extensions` (bool) - Preserve unknown XML elements
- `include_comments` (bool) - Preserve XML comments
- `validate_references` (bool) - Validate all references
- `streaming` (bool) - Use streaming mode
- `max_memory` (int) - Maximum memory in bytes
- `timeout` (float) - Timeout in seconds

## 📄 License

MIT © Kevin Marques Moo

## 🙏 Acknowledgments

This parser is designed to complement the official [DDEX Workbench](https://github.com/ddex/ddex-workbench) by providing structural parsing while Workbench handles XSD validation.

Special thanks to the DDEX community for their standards documentation and to everyone who provides feedback during this early development phase.

## 🔗 Links

- [GitHub Repository](https://github.com/daddykev/ddex-suite)
- [Documentation](https://github.com/daddykev/ddex-suite/tree/main/packages/ddex-parser)
- [PyPI Package](https://pypi.org/project/ddex-parser/)
- [npm Package](https://www.npmjs.com/package/ddex-parser) (JavaScript/TypeScript)

---

**Version**: 0.1.0  
**Status**: Early Alpha - Foundation Release  
**Repository**: https://github.com/daddykev/ddex-suite  
**PyPI**: https://pypi.org/project/ddex-parser/  
**Author**: Kevin Marques Moo

*Thank you for trying this early release! Your feedback helps shape the future of DDEX Suite.*