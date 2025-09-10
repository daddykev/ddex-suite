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

#### `parse_bytes(data: bytes) -> DDEXResult`

Parse DDEX XML from bytes.

```python
with open("release.xml", "rb") as f:
    xml_bytes = f.read()
result = parser.parse_bytes(xml_bytes)
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

### DDEXResult

The parsed result provides both graph and flattened representations:

```python
# Graph representation (faithful to DDEX structure)
graph_data = result.graph
print(graph_data.message_header.sender_name)

# Flattened representation (developer-friendly)
flat_data = result.flattened
print(flat_data.release_title)
print(flat_data.main_artist)

# Convert to different formats
df = result.to_dataframe()           # pandas DataFrame
json_data = result.to_json()         # JSON string
dict_data = result.to_dict()         # Python dictionary
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

## Advanced Examples

### Batch Processing

```python
import os
from pathlib import Path
import pandas as pd

def process_ddex_catalog(directory: str) -> pd.DataFrame:
    """Process all DDEX files in a directory and combine results."""
    parser = DDEXParser(streaming=True)
    all_tracks = []
    
    for xml_file in Path(directory).glob("*.xml"):
        try:
            result = parser.parse_file(str(xml_file))
            tracks = result.to_dataframe("tracks")
            tracks['source_file'] = xml_file.name
            all_tracks.append(tracks)
        except Exception as e:
            print(f"Error processing {xml_file}: {e}")
    
    return pd.concat(all_tracks, ignore_index=True)

# Process entire catalog
catalog_df = process_ddex_catalog("./ddex_files/")
print(f"Processed {len(catalog_df)} tracks from catalog")
```

### Streaming Large Files

```python
async def process_large_ddex_file(filepath: str):
    """Handle large DDEX files with streaming."""
    parser = DDEXParser(streaming=True)
    
    # Process asynchronously to avoid blocking
    result = await parser.parse_async(filepath)
    
    # Stream results to avoid memory issues
    for batch in result.stream_tracks(batch_size=1000):
        # Process each batch of tracks
        batch_df = pd.DataFrame(batch)
        # Save to database, file, etc.
        yield batch_df

# Usage
async for batch in process_large_ddex_file("large_catalog.xml"):
    print(f"Processing batch of {len(batch)} tracks")
```

### Custom Error Handling

```python
from ddex_parser import DDEXParser, DDEXError, SecurityError

parser = DDEXParser()

try:
    result = parser.parse_file("suspicious.xml")
except SecurityError as e:
    print(f"Security issue detected: {e}")
    # Handle XXE or other security concerns
except DDEXError as e:
    print(f"DDEX parsing error: {e}")
    # Handle malformed DDEX files
except Exception as e:
    print(f"Unexpected error: {e}")
```

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

## Performance Benchmarks

Performance comparison on a MacBook Pro M2:

| File Size | ddex-parser | lxml | xml.etree | Speedup |
|-----------|-------------|------|-----------|---------|
| 10KB      | 0.8ms       | 8ms  | 12ms      | 10x-15x |
| 100KB     | 3ms         | 45ms | 78ms      | 15x-26x |
| 1MB       | 28ms        | 380ms| 650ms     | 13x-23x |
| 10MB      | 180ms       | 3.2s | 5.8s      | 18x-32x |

Memory usage is consistently 60-80% lower than traditional parsers.

## Type Hints & IDE Support

Full type hints for better development experience:

```python
from ddex_parser import DDEXParser, DDEXResult
from typing import Optional, List, Dict, Any

def analyze_release(file_path: str) -> Dict[str, Any]:
    parser: DDEXParser = DDEXParser()
    result: DDEXResult = parser.parse_file(file_path)
    
    # IDE autocomplete and type checking work perfectly
    release_info: Dict[str, Any] = {
        'title': result.flattened.release_title,
        'artist': result.flattened.main_artist,
        'track_count': len(result.flattened.tracks),
        'genres': [track.genre for track in result.flattened.tracks],
    }
    
    return release_info
```

## Migration from v0.1.0

Upgrading from v0.1.0? Here are the key changes:

```python
# v0.1.0
from ddex_parser import parse_ddex
result = parse_ddex("file.xml")
title = result["release"]["title"]

# v0.2.0 (current)
from ddex_parser import DDEXParser
parser = DDEXParser()
result = parser.parse_file("file.xml")
title = result.flattened.release_title  # More intuitive access
```

New features in v0.2.0:
- Async support with `parse_async()`
- DataFrame integration with `to_dataframe()`
- Streaming for large files
- Enhanced security features
- Better error handling
- Type hints throughout

## Troubleshooting

### Common Issues

**ImportError: No module named '_ddex_parser'**
```bash
pip install --upgrade ddex-parser
# If still failing, try:
pip install --force-reinstall ddex-parser
```

**Memory issues with large files**
```python
# Enable streaming mode
parser = DDEXParser(streaming=True)
result = parser.parse_file("large_file.xml")
```

**Encoding issues**
```python
# Specify encoding explicitly
with open("file.xml", "r", encoding="utf-8") as f:
    content = f.read()
result = parser.parse_string(content)
```

### Getting Help

- 📖 [Full Documentation](https://github.com/ddex-suite/ddex-suite/tree/main/packages/ddex-parser)
- 🐛 [Report Issues](https://github.com/ddex-suite/ddex-suite/issues)
- 💬 [GitHub Discussions](https://github.com/ddex-suite/ddex-suite/discussions)
- 📧 Email: support@ddex-suite.com

## Contributing

We welcome contributions! See our [Contributing Guide](https://github.com/ddex-suite/ddex-suite/blob/main/CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/ddex-suite/ddex-suite/blob/main/LICENSE) file for details.

## Related Projects

- **[ddex-builder](https://pypi.org/project/ddex-builder/)** - Build deterministic DDEX XML files
- **[ddex-parser (npm)](https://www.npmjs.com/package/ddex-parser)** - JavaScript/TypeScript bindings
- **[DDEX Suite](https://github.com/ddex-suite/ddex-suite)** - Complete DDEX processing toolkit

---

Built for the music industry. Powered by Rust for maximum performance and safety.