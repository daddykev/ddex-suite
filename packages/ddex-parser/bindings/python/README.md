# DDEX Parser for Python

[![PyPI version](https://img.shields.io/pypi/v/ddex-parser.svg)](https://pypi.org/project/ddex-parser/)
[![Python versions](https://img.shields.io/pypi/pyversions/ddex-parser.svg)](https://pypi.org/project/ddex-parser/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

High-performance DDEX XML parser for Python, built on Rust for blazing speed and memory safety.

## Installation

```bash
pip install ddex-parser
```

### Optional dependencies

For DataFrame support:
```bash
pip install ddex-parser[pandas]
```

For development:
```bash
pip install ddex-parser[dev]
```

## Quick Start

```python
from ddex_parser import DDEXParser

# Create parser
parser = DDEXParser()

# Parse DDEX XML
with open('release.xml', 'r') as f:
    result = parser.parse(f.read())

# Access data
print(f"Message ID: {result.message_id}")
print(f"Releases: {result.release_count}")

for release in result.releases:
    print(f"- {release['title']} by {release['artist']}")
```

## Advanced Usage

### Async parsing

```python
import asyncio
from ddex_parser import DDEXParser

async def parse_async():
    parser = DDEXParser()
    result = await parser.parse_async(xml_content)
    return result

result = asyncio.run(parse_async())
```

### Stream large files

```python
parser = DDEXParser()

for release in parser.stream('large_catalog.xml'):
    print(f"Processing: {release['title']}")
    # Process one release at a time
```

### Convert to pandas DataFrame

```python
parser = DDEXParser()
df = parser.to_dataframe(xml_content)

# Analyze with pandas
print(df.describe())
print(df.groupby('artist')['title'].count())
```

## Performance

Typical performance on modern hardware:

| File Size | Parse Time | Memory Usage |
|-----------|------------|--------------|
| 10KB | <5ms | 2MB |
| 100KB | <10ms | 5MB |
| 1MB | <50ms | 20MB |
| 100MB | <5s | 50MB |
| 1GB | <60s | 100MB |

## API Reference

### DDEXParser

Main parser class.

#### Methods

- `parse(xml, options=None)` - Parse XML synchronously
- `parse_async(xml, options=None)` - Parse XML asynchronously  
- `stream(source, options=None)` - Stream parse large files
- `to_dataframe(xml, schema='flat')` - Convert to pandas DataFrame
- `detect_version(xml)` - Detect DDEX version
- `sanity_check(xml)` - Validate XML structure

### ParseOptions

Configuration for parsing.

#### Parameters

- `include_raw_extensions` (bool) - Preserve unknown XML elements
- `include_comments` (bool) - Preserve XML comments
- `validate_references` (bool) - Validate all references
- `streaming` (bool) - Use streaming mode
- `max_memory` (int) - Maximum memory in bytes
- `timeout` (float) - Timeout in seconds

## License

MIT © Kevin Marques Moo

## Links

- [GitHub Repository](https://github.com/daddykev/ddex-suite)
- [Documentation](https://github.com/daddykev/ddex-suite/tree/main/packages/ddex-parser)
- [PyPI Package](https://pypi.org/project/ddex-parser/)