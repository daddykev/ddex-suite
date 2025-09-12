# DDEX Builder - Python Bindings

[![PyPI version](https://img.shields.io/pypi/v/ddex-builder.svg)](https://pypi.org/project/ddex-builder/)
[![Python versions](https://img.shields.io/pypi/pyversions/ddex-builder.svg)](https://pypi.org/project/ddex-builder/)
[![Downloads](https://img.shields.io/pypi/dm/ddex-builder.svg)](https://pypi.org/project/ddex-builder/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Generate deterministic, industry-compliant DDEX XML files from Python data structures with byte-perfect reproducibility. Build DDEX messages from dictionaries, DataFrames, or parsed objects with built-in validation and partner-specific presets.

## Installation

```bash
pip install ddex-builder
```

## Security Notice
**v0.3.5 includes PyO3 0.24 upgrade for enhanced security and compatibility.**

## Quick Start

```python
from ddex_builder import DDEXBuilder
import pandas as pd

# Create builder with validation
builder = DDEXBuilder(validate=True)

# Build from dictionary
release_data = {
    'message_header': {
        'sender_name': 'My Label',
        'message_id': 'MSG123',
    },
    'releases': [{
        'title': 'My Album',
        'main_artist': 'Great Artist',
        'tracks': [{
            'title': 'Track 1',
            'duration': 180,
            'isrc': 'US1234567890'
        }]
    }]
}

xml_output = builder.build_from_dict(release_data, version='4.3')
print(xml_output[:100] + '...')
```

## Features

### 🎯 Deterministic Output
- **100% reproducible** XML generation with stable hash IDs
- DB-C14N/1.0 canonicalization for byte-perfect consistency
- IndexMap-based ordering ensures identical output across runs
- Content-addressable resource IDs for reliable references

### 🏭 Industry Presets
- **YouTube Music**: Content ID and monetization standards
- **Generic**: Default preset suitable for most distributors

### 📊 DataFrame Integration
- Build directly from pandas DataFrames
- Automatic schema detection and validation
- Support for hierarchical data structures
- Export/import workflows with CSV and Parquet

### 🔒 Built-in Validation
- Comprehensive DDEX schema validation
- Business rule enforcement
- Reference integrity checking
- Territory and rights validation

### 🚀 High Performance
- Native Rust implementation with Python bindings
- Streaming generation for large catalogs
- Memory-efficient processing
- Parallel resource processing

## API Reference

### DDEXBuilder

```python
from ddex_builder import DDEXBuilder

builder = DDEXBuilder(
    validate=True,           # Enable validation (recommended)
    preset='generic',        # Use generic industry preset
    canonical=True,          # Generate canonical XML
    streaming=False,         # Enable streaming mode for large data
    max_memory=100_000_000   # Memory limit in bytes
)
```

## Performance Benchmarks

Building performance on different dataset sizes:

| Dataset Size | Build Time | Memory Usage | Output Size |
|--------------|------------|-------------|------------|
| Single release (10 tracks) | 2ms | 5MB | 25KB |
| Album catalog (100 releases) | 15ms | 25MB | 2.5MB |
| Label catalog (1000 releases) | 120ms | 80MB | 25MB |
| Large catalog (10000 releases) | 1.2s | 200MB | 250MB |

Memory usage scales linearly, with streaming mode keeping usage constant for any dataset size.

## Integration with ddex-parser

Round-trip compatibility with ddex-parser for complete workflows:

```python
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

# Parse existing DDEX file
parser = DDEXParser()
original = parser.parse_file("input.xml")

# Modify data
modified_data = original.to_dict()
modified_data['tracks'][0]['title'] = "New Title"

# Build new DDEX file
builder = DDEXBuilder()
new_xml = builder.build_from_dict(modified_data)

# Verify round-trip integrity
new_result = parser.parse_string(new_xml)
assert new_result.tracks[0].title == "New Title"
```

## Requirements
- Python 3.8+
- pandas (optional, for DataFrame support)
- PyO3 0.24 compatible runtime

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE) file for details.

## Related Projects

- **[ddex-parser](https://pypi.org/project/ddex-parser/)** - Parse DDEX XML files to Python structures
- **[ddex-builder (npm)](https://www.npmjs.com/package/ddex-builder)** - JavaScript/TypeScript bindings
- **[DDEX Suite](https://ddex-suite.org)** - Complete DDEX processing toolkit

---

Built with ❤️ for the music industry. Engineered for deterministic, industry-grade DDEX generation.