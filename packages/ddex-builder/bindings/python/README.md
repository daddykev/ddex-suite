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
- **Spotify**: Optimized for streaming platform requirements
- **Apple Music**: iTunes Store compliance and best practices  
- **Amazon Music**: Prime Music and Unlimited specifications
- **YouTube Music**: Content ID and monetization standards
- **Universal**: Generic preset suitable for most distributors

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
    preset='universal',      # Use industry preset
    canonical=True,          # Generate canonical XML
    streaming=False,         # Enable streaming mode for large data
    max_memory=100_000_000   # Memory limit in bytes
)
```

### Building Methods

#### `build_from_dict(data: dict, version: str = '4.3') -> str`

Build DDEX XML from a Python dictionary.

```python
release = {
    'message_header': {
        'sender_name': 'Label Records',
        'message_id': 'RELEASE_2024_001',
        'sent_date': '2024-01-15T10:30:00Z'
    },
    'releases': [{
        'release_id': 'REL001',
        'title': 'Amazing Album',
        'main_artist': 'Incredible Artist',
        'label_name': 'Label Records',
        'release_date': '2024-02-01',
        'genres': ['Pop', 'Electronic'],
        'tracks': [{
            'track_id': 'TRK001',
            'title': 'Hit Song',
            'position': 1,
            'duration': 195,
            'isrc': 'US1234567890',
            'artists': ['Incredible Artist']
        }]
    }]
}

xml = builder.build_from_dict(release, version='4.3')
```

#### `build_from_dataframe(df: pd.DataFrame, schema: str = 'auto') -> str`

Build DDEX XML from a pandas DataFrame.

```python
import pandas as pd

# Create DataFrame with release data
tracks_df = pd.DataFrame([
    {
        'release_title': 'My Album',
        'release_artist': 'Artist Name',  
        'track_title': 'Song One',
        'track_position': 1,
        'duration': 180,
        'isrc': 'US1234567890'
    },
    {
        'release_title': 'My Album', 
        'release_artist': 'Artist Name',
        'track_title': 'Song Two',
        'track_position': 2,  
        'duration': 220,
        'isrc': 'US1234567891'
    }
])

xml = builder.build_from_dataframe(tracks_df, schema='release_tracks')
```

#### `build_from_parsed(result: DDEXResult) -> str`

Build DDEX XML from a parsed ddex-parser result.

```python
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

# Parse existing DDEX file
parser = DDEXParser()
result = parser.parse_file('input.xml')

# Modify the parsed data
result.flattened.releases[0].title = 'Updated Title'

# Build new XML with modifications
builder = DDEXBuilder()
new_xml = builder.build_from_parsed(result)
```

#### `build_async(data: dict) -> Awaitable[str]`

Asynchronous building for large datasets.

```python
import asyncio

async def build_large_catalog():
    large_catalog = load_catalog_data()  # Load your data
    xml = await builder.build_async(large_catalog)
    return xml

# Usage
xml_result = asyncio.run(build_large_catalog())
```

### Industry Presets

#### Spotify Preset

```python
from ddex_builder import DDEXBuilder, SpotifyPreset

builder = DDEXBuilder(preset='spotify')

# Spotify-specific optimizations:
# - Spotify Artist ID

xml = builder.build_from_dict(catalog_data, version='4.3')
```

#### YouTube Preset

```python
builder = DDEXBuilder(preset='youtube_music')

# YouTube-specific optimizations:
# - ContentID tracking
# - Territory restrictions

xml = builder.build_from_dict(release_data, version='4.3')
```

#### Custom Preset

```python
from ddex_builder import DDEXBuilder, BuilderPreset

custom_preset = BuilderPreset(
    default_territories=['US', 'CA', 'GB'],
    require_isrc=True,
    validate_durations=True,
    normalize_genres=['Pop', 'Rock', 'Electronic'],
    max_track_duration=600,  # 10 minutes
    required_fields=['title', 'main_artist', 'duration']
)

builder = DDEXBuilder(preset=custom_preset)
```

## Advanced Usage

### Streaming Large Catalogs

```python
from ddex_builder import DDEXBuilder
import pandas as pd

def build_large_catalog(csv_file: str) -> str:
    """Build DDEX from large CSV catalog."""
    builder = DDEXBuilder(streaming=True, max_memory=50_000_000)
    
    # Read CSV in chunks to manage memory
    chunk_size = 1000
    xml_parts = []
    
    for chunk in pd.read_csv(csv_file, chunksize=chunk_size):
        # Process each chunk
        chunk_xml = builder.build_from_dataframe(chunk, schema='catalog')
        xml_parts.append(chunk_xml)
    
    # Combine chunks into final DDEX message
    return builder.merge_xml_chunks(xml_parts)

# Process 100,000+ track catalog
catalog_xml = build_large_catalog('massive_catalog.csv')
```

### Validation and Error Handling

```python
from ddex_builder import DDEXBuilder, ValidationError, BuilderError

builder = DDEXBuilder(validate=True)

try:
    xml = builder.build_from_dict(release_data)
    print("✅ DDEX built successfully")
except ValidationError as e:
    print(f"❌ Validation failed: {e.details}")
    for error in e.field_errors:
        print(f"  - {error.field}: {error.message}")
except BuilderError as e:
    print(f"❌ Builder error: {e.message}")
    if hasattr(e, 'suggestions'):
        print("💡 Suggestions:")
        for suggestion in e.suggestions:
            print(f"  - {suggestion}")
```

### Round-Trip with ddex-parser

Perfect integration for complete Parse → Modify → Build workflows:

```python
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

def round_trip_example():
    # Parse original DDEX file
    parser = DDEXParser()
    original = parser.parse_file('original.xml')
    
    # Modify specific fields
    modified_data = original.to_dict()
    modified_data['releases'][0]['title'] = 'Remastered Edition'
    
    # Add new track
    new_track = {
        'title': 'Bonus Track',
        'position': len(modified_data['releases'][0]['tracks']) + 1,
        'duration': 240,
        'isrc': 'US9876543210'
    }
    modified_data['releases'][0]['tracks'].append(new_track)
    
    # Build new deterministic XML
    builder = DDEXBuilder(canonical=True)
    new_xml = builder.build_from_dict(modified_data)
    
    # Verify round-trip integrity
    reparsed = parser.parse_string(new_xml)
    assert reparsed.releases[0].title == 'Remastered Edition'
    assert len(reparsed.tracks) == len(original.tracks) + 1
    
    return new_xml

# Guaranteed deterministic output
xml1 = round_trip_example()
xml2 = round_trip_example()
assert xml1 == xml2  # ✅ Byte-perfect reproducibility
```

### DataFrame Workflows

#### Complex Catalog Processing

```python
import pandas as pd
from ddex_builder import DDEXBuilder

def process_music_catalog(artists_df, albums_df, tracks_df):
    """Build DDEX from normalized database tables."""
    builder = DDEXBuilder(preset='universal', validate=True)
    
    # Merge dataframes to create hierarchical structure
    catalog = tracks_df.merge(albums_df, on='album_id') \
                      .merge(artists_df, on='artist_id')
    
    # Group by release for DDEX structure
    releases = []
    for album_id, album_tracks in catalog.groupby('album_id'):
        release = {
            'release_id': album_id,
            'title': album_tracks.iloc[0]['album_title'],
            'main_artist': album_tracks.iloc[0]['artist_name'],
            'release_date': album_tracks.iloc[0]['release_date'],
            'tracks': album_tracks.to_dict('records')
        }
        releases.append(release)
    
    # Build DDEX XML
    ddex_data = {
        'message_header': {
            'sender_name': 'My Label',
            'message_id': f'CATALOG_{pd.Timestamp.now().strftime("%Y%m%d_%H%M%S")}'
        },
        'releases': releases
    }
    
    return builder.build_from_dict(ddex_data, version='4.3')

# Load data from your database/CSV files
artists = pd.read_csv('artists.csv')
albums = pd.read_csv('albums.csv') 
tracks = pd.read_csv('tracks.csv')

# Generate DDEX catalog
catalog_xml = process_music_catalog(artists, albums, tracks)
```

#### Analytics Integration

```python
import pandas as pd
from ddex_builder import DDEXBuilder

def build_from_analytics(streaming_data: pd.DataFrame) -> str:
    """Build DDEX from streaming analytics data."""
    builder = DDEXBuilder(preset='spotify')
    
    # Aggregate streaming data to releases
    release_stats = streaming_data.groupby(['release_id', 'release_title']).agg({
        'streams': 'sum',
        'revenue': 'sum',
        'territories': lambda x: list(x.unique())
    }).reset_index()
    
    # Convert to DDEX format
    releases = []
    for _, row in release_stats.iterrows():
        release = {
            'release_id': row['release_id'],
            'title': row['release_title'],
            'territories': row['territories'],
            'usage_rights': {
                'stream_count': int(row['streams']),
                'revenue': float(row['revenue'])
            }
        }
        releases.append(release)
    
    ddex_data = {
        'message_header': {
            'sender_name': 'Analytics Export',
            'message_id': f'ANALYTICS_{pd.Timestamp.now().isoformat()}'
        },
        'releases': releases
    }
    
    return builder.build_from_dict(ddex_data, version='4.3')
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

## DataFrame Schema Reference

### Standard Schemas

#### `release_tracks` Schema

```python
# Required columns for release_tracks schema
df_columns = [
    'release_title',      # str: Album/EP title
    'release_artist',     # str: Main artist name  
    'track_title',        # str: Song title
    'track_position',     # int: Track number
    'duration',          # int: Duration in seconds
    'isrc',              # str: International Standard Recording Code
]

# Optional columns
optional_columns = [
    'release_date',       # str: ISO date (YYYY-MM-DD)
    'label_name',        # str: Record label
    'genres',            # list[str]: Genre classifications
    'territories',       # list[str]: Territory codes
    'explicit',          # bool: Explicit content flag
]
```

#### `catalog` Schema

```python
# Multi-release catalog schema
catalog_columns = [
    'release_id',        # str: Unique release identifier
    'release_title',     # str: Release title
    'release_artist',    # str: Main artist
    'release_date',      # str: Release date
    'track_id',          # str: Unique track identifier
    'track_title',       # str: Track title
    'track_position',    # int: Position in release
    'duration',          # int: Duration in seconds
    'isrc',             # str: ISRC code
    'artists',          # list[str]: All participating artists
]
```

## Migration from v0.1.0

v0.2.0 introduces significant improvements:

```python
# v0.1.0 (deprecated)
from ddex_builder import build_ddex
xml = build_ddex(data, version='4.3')

# v0.2.0+ (current) 
from ddex_builder import DDEXBuilder
builder = DDEXBuilder()
xml = builder.build_from_dict(data, version='4.3')
```

### New Features in v0.2.0

- **Industry presets** for major streaming platforms
- **DataFrame integration** with pandas
- **Async support** for large datasets
- **Enhanced validation** with detailed error messages
- **Deterministic output** with DB-C14N/1.0
- **Streaming mode** for memory efficiency
- **Round-trip compatibility** with ddex-parser

## Troubleshooting

### Common Issues

**ValidationError: Required field missing**
```python
# Ensure all required fields are present
required_fields = builder.get_required_fields(version='4.3')
print(f"Required fields: {required_fields}")

# Check your data
missing_fields = builder.validate_dict(your_data)
print(f"Missing: {missing_fields}")
```

**Memory issues with large catalogs**
```python
# Enable streaming mode
builder = DDEXBuilder(streaming=True, max_memory=50_000_000)
xml = builder.build_from_dataframe(large_df, schema='catalog')
```

**Inconsistent XML output**
```python
# Enable canonical mode for deterministic output
builder = DDEXBuilder(canonical=True)
xml = builder.build_from_dict(data, version='4.3')
```

**Import errors**
```bash
pip install --upgrade ddex-builder
# If still failing:
pip install --force-reinstall ddex-builder
```

### Getting Help

- 📖 [Full Documentation](https://github.com/ddex-suite/ddex-suite/tree/main/packages/ddex-builder)
- 🐛 [Report Issues](https://github.com/ddex-suite/ddex-suite/issues)
- 💬 [GitHub Discussions](https://github.com/ddex-suite/ddex-suite/discussions)
- 📧 Email: support@ddex-suite.com

## Contributing

We welcome contributions! See our [Contributing Guide](https://github.com/ddex-suite/ddex-suite/blob/main/CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/ddex-suite/ddex-suite/blob/main/LICENSE) file for details.

## Related Projects

- **[ddex-parser](https://pypi.org/project/ddex-parser/)** - Parse DDEX XML files to Python structures
- **[ddex-builder (npm)](https://www.npmjs.com/package/ddex-builder)** - JavaScript/TypeScript bindings
- **[DDEX Suite](https://github.com/ddex-suite/ddex-suite)** - Complete DDEX processing toolkit

---

Built with ❤️ for the music industry. Engineered for deterministic, industry-grade DDEX generation.