# DDEX Parser - Python Bindings

[![PyPI version](https://img.shields.io/pypi/v/ddex-parser.svg)](https://pypi.org/project/ddex-parser/)
[![Python versions](https://img.shields.io/pypi/pyversions/ddex-parser.svg)](https://pypi.org/project/ddex-parser/)
[![Downloads](https://img.shields.io/pypi/dm/ddex-parser.svg)](https://pypi.org/project/ddex-parser/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

High-performance DDEX XML parser for Python with built-in security features and comprehensive metadata extraction. Parse DDEX files 10x faster than traditional XML parsers with full support for all DDEX versions and profiles.

## Installation

```bash
pip install ddex-parser
```

## Security Notice
**v0.3.5 fixes a critical security vulnerability (RUSTSEC-2025-0020).** 
All users should upgrade immediately for PyO3 0.24 compatibility and enhanced security.

## Quick Start

```python
from ddex_parser import DDEXParser
import pandas as pd

# Parse DDEX file
parser = DDEXParser()
result = parser.parse_file("release.xml")

# Access parsed data
print(f"Release: {result.release_title}")
print(f"Artist: {result.main_artist}")
print(f"Tracks: {len(result.tracks)}")

# Convert to DataFrame for analysis
tracks_df = result.to_dataframe()
print(tracks_df.head())
```

## Features

### 🚀 High Performance
- **10x faster** than standard XML parsers
- Streaming support for large files (>100MB)
- Memory-efficient processing
- Native Rust implementation with Python bindings

### 🔒 Security Built-in
- XXE (XML External Entity) attack protection
- Entity expansion limits
- Memory-bounded parsing
- Deep nesting protection

### 📊 Data Science Ready
- Direct pandas DataFrame export
- Structured metadata extraction
- JSON serialization support
- Type hints for better IDE experience

### 🎵 Music Industry Focused
- Support for all DDEX versions (3.2, 3.3, 4.0+)
- Release, track, and artist metadata
- Rights and usage information
- Territory and deal terms
- Image and audio resource handling

## API Reference

### DDEXParser

```python
from ddex_parser import DDEXParser

parser = DDEXParser(
    max_entity_expansions=1000,  # Limit entity expansions for security
    max_depth=100,               # Maximum XML nesting depth
    streaming=True               # Enable streaming for large files
)
```

### Parsing Methods

#### `parse_file(path: str) -> DDEXResult`

Parse a DDEX XML file from disk.

```python
result = parser.parse_file("path/to/release.xml")
```

#### `parse_string(xml: str) -> DDEXResult`

Parse DDEX XML from a string.

```python
with open("release.xml", "r") as f:
    xml_content = f.read()
result = parser.parse_string(xml_content)
```

#### `parse_async(path: str) -> Awaitable[DDEXResult]`

Asynchronous parsing for non-blocking operations.

```python
import asyncio

async def parse_ddex():
    result = await parser.parse_async("release.xml")
    return result

# Usage
result = asyncio.run(parse_ddex())
```

## DataFrame Integration

Perfect for data analysis workflows:

```python
import pandas as pd
from ddex_parser import DDEXParser

parser = DDEXParser()
result = parser.parse_file("catalog.xml")

# Get tracks as DataFrame
tracks_df = result.to_dataframe("tracks")
print(tracks_df.columns)
# ['track_id', 'title', 'artist', 'duration', 'isrc', 'genre', ...]

# Analyze your catalog
genre_counts = tracks_df['genre'].value_counts()
avg_duration = tracks_df['duration'].mean()

# Export for further analysis
tracks_df.to_csv("catalog_analysis.csv")
tracks_df.to_parquet("catalog_analysis.parquet")
```

## Performance Benchmarks

Performance comparison on a MacBook Pro M2:

| File Size | ddex-parser | lxml | xml.etree | Speedup |
|-----------|-------------|------|-----------|----------|
| 10KB      | 0.8ms       | 8ms  | 12ms      | 10x-15x |
| 100KB     | 3ms         | 45ms | 78ms      | 15x-26x |
| 1MB       | 28ms        | 380ms| 650ms     | 13x-23x |
| 10MB      | 180ms       | 3.2s | 5.8s      | 18x-32x |

Memory usage is consistently 60-80% lower than traditional parsers.

## Integration with ddex-builder

Round-trip compatibility with ddex-builder for complete workflows:

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

- **[ddex-builder](https://pypi.org/project/ddex-builder/)** - Build deterministic DDEX XML files
- **[ddex-parser (npm)](https://www.npmjs.com/package/ddex-parser)** - JavaScript/TypeScript bindings
- **[DDEX Suite](https://ddex-suite.org)** - Complete DDEX processing toolkit

---

Built for the music industry. Powered by Rust for maximum performance and safety.