# DDEX Builder - Python Bindings

[![PyPI version](https://badge.fury.io/py/ddex-builder.svg)](https://badge.fury.io/py/ddex-builder)
[![Python versions](https://img.shields.io/pypi/pyversions/ddex-builder.svg)](https://pypi.org/project/ddex-builder/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/daddykev/ddex-suite/workflows/CI/badge.svg)](https://github.com/daddykev/ddex-suite/actions)

High-performance DDEX XML builder for Python with deterministic output and DB-C14N/1.0 canonicalization. Built with Rust and PyO3 for maximum performance. Part of the [DDEX Suite](https://github.com/daddykev/ddex-suite) toolkit.

## Features

- 🚀 **Ultra-Fast Performance**: Native Rust implementation with Python bindings
- 🔒 **Deterministic Output**: Consistent XML generation with stable ordering
- 📋 **DB-C14N/1.0 Canonicalization**: Standards-compliant XML canonicalization
- 🐼 **Pandas Integration**: Native DataFrame support for batch processing
- 🌊 **Memory Efficient**: Streaming support for large datasets
- ✅ **Built-in Validation**: Real-time validation with detailed error reporting
- 🎯 **Partner Presets**: Pre-configured settings for major platforms
- 📊 **Data Science Friendly**: Seamless integration with pandas, numpy, and Jupyter
- 🔧 **Cross-Platform**: Supports Windows, macOS, and Linux (Python 3.8+)

## Installation

### Via pip (Recommended)

```bash
pip install ddex-builder
```

### With Optional Dependencies

```bash
# For DataFrame integration
pip install ddex-builder[pandas]

# For development and testing
pip install ddex-builder[dev]

# All optional dependencies
pip install ddex-builder[pandas,dev]
```

### Requirements

- Python ≥ 3.8
- No additional dependencies required (native wheels included)
- Optional: pandas ≥ 1.0, numpy ≥ 1.20 for DataFrame integration

## Quick Start

### Basic Usage

```python
import ddex_builder

# Create builder instance
builder = ddex_builder.DdexBuilder()

# Add a release
release = ddex_builder.Release(
    release_id='R001',
    release_type='Album',
    title='My Album',
    artist='Artist Name',
    label='Record Label',
    catalog_number='CAT001',
    upc='123456789012',
    release_date='2024-01-01',
    genre='Electronic',
    track_ids=['T001', 'T002']
)

builder.add_release(release)

# Add resources (tracks)
track = ddex_builder.Resource(
    resource_id='T001',
    resource_type='SoundRecording',
    title='Track 1',
    artist='Artist Name',
    isrc='USRC17607839',
    duration='PT3M30S',
    track_number=1
)

builder.add_resource(track)

# Validate and build
validation = builder.validate()
if validation.is_valid:
    xml = builder.build()
    print(f"Generated XML: {len(xml)} bytes")
else:
    print(f"Validation errors: {validation.errors}")
```

### DataFrame Integration

```python
import pandas as pd
import ddex_builder

# Create DataFrame with release data
releases_df = pd.DataFrame({
    'release_id': ['R001', 'R002'],
    'release_type': ['Album', 'Single'],
    'title': ['Greatest Hits', 'New Single'],
    'artist': ['The Band', 'Solo Artist'],
    'label': ['Major Label', 'Indie Label'],
    'upc': ['123456789012', '123456789013'],
    'release_date': ['2024-03-01', '2024-03-15'],
    'genre': ['Rock', 'Pop'],
    'track_ids': [['T001', 'T002', 'T003'], ['T004']]
})

# Create DataFrame with resource data
resources_df = pd.DataFrame({
    'resource_id': ['T001', 'T002', 'T003', 'T004'],
    'resource_type': 'SoundRecording',
    'title': ['Hit Song 1', 'Hit Song 2', 'Deep Cut', 'New Single'],
    'artist': ['The Band', 'The Band', 'The Band', 'Solo Artist'],
    'isrc': ['USRC17607001', 'USRC17607002', 'USRC17607003', 'USRC17607004'],
    'duration': ['PT3M45S', 'PT4M12S', 'PT5M30S', 'PT3M22S'],
    'track_number': [1, 2, 3, 1]
})

# Build from DataFrames
builder = ddex_builder.DdexBuilder()
builder.from_dataframe(releases_df)
builder.from_dataframe(resources_df)

# Generate XML
xml = builder.build()
```

### Streaming for Large Datasets

```python
import ddex_builder

# Configure streaming
config = ddex_builder.StreamingConfig(
    max_buffer_size=1024 * 1024,  # 1MB buffer
    deterministic=True,
    validate_during_stream=True,
    progress_callback_frequency=100
)

builder = ddex_builder.StreamingDdexBuilder(config)

# Set up progress callback
def progress_callback(progress):
    print(f"Progress: {progress.estimated_completion_percent:.1f}% "
          f"({progress.releases_written} releases, "
          f"{progress.resources_written} resources)")

builder.set_progress_callback(progress_callback)

# Initialize message
header = ddex_builder.MessageHeader(
    message_sender_name='Your Company',
    message_recipient_name='Distribution Partner',
    message_created_date_time='2024-01-01T10:00:00Z'
)

builder.start_message(header, '4.2')

# Stream resources
for i in range(1000):  # Large dataset
    builder.write_resource(
        f'T{i:04d}',
        f'Track {i+1}',
        'Artist Name',
        f'USRC1760{i:04d}',
        'PT3M30S',
        f'/path/to/track_{i+1}.wav'
    )

builder.finish_resources_start_releases()

# Stream releases  
for i in range(100):  # 100 albums
    track_refs = [f'T{j:04d}' for j in range(i*10, (i+1)*10)]
    builder.write_release(
        f'R{i:03d}',
        f'Album {i+1}',
        'Artist Name',
        'Label Name',
        f'12345678{i:04d}',
        '2024-01-01',
        'Pop',
        track_refs
    )

# Finalize
stats = builder.finish_message()
xml = builder.get_xml()

print(f"Generated XML with {stats.releases_written} releases "
      f"and {stats.resources_written} resources")
```

## API Reference

### Core Classes

#### `DdexBuilder`

Main builder class for creating DDEX XML documents.

```python
class DdexBuilder:
    def __init__(self) -> None: ...
    
    def add_release(self, release: Release) -> None: ...
    def add_resource(self, resource: Resource) -> None: ...
    
    def build(self) -> str: ...
    def validate(self) -> ValidationResult: ...
    
    def get_stats(self) -> BuilderStats: ...
    def reset(self) -> None: ...
    
    # DataFrame integration
    def from_dataframe(self, df: pd.DataFrame) -> None: ...
    
    # Preset support
    def get_available_presets(self) -> list[str]: ...
    def apply_preset(self, preset_name: str) -> None: ...
    def get_preset_info(self, preset_name: str) -> PresetInfo: ...
```

#### `Release`

Represents a DDEX release (album, single, etc.).

```python
class Release:
    def __init__(
        self,
        release_id: str,
        release_type: str,
        title: str,
        artist: str,
        label: str | None = None,
        catalog_number: str | None = None,
        upc: str | None = None,
        release_date: str | None = None,
        genre: str | None = None,
        parental_warning: bool | None = None,
        track_ids: list[str] | None = None,
        metadata: dict[str, str] | None = None,
    ) -> None: ...
    
    # All fields are accessible as properties
    release_id: str
    release_type: str
    title: str
    artist: str
    label: str | None
    # ... etc
```

#### `Resource`

Represents a DDEX resource (sound recording, video, etc.).

```python
class Resource:
    def __init__(
        self,
        resource_id: str,
        resource_type: str,
        title: str,
        artist: str,
        isrc: str | None = None,
        duration: str | None = None,
        track_number: int | None = None,
        volume_number: int | None = None,
        metadata: dict[str, str] | None = None,
    ) -> None: ...
    
    # All fields accessible as properties
    resource_id: str
    resource_type: str
    title: str
    artist: str
    # ... etc
```

#### `StreamingDdexBuilder`

Memory-efficient builder for large datasets.

```python
class StreamingDdexBuilder:
    def __init__(self, config: StreamingConfig | None = None) -> None: ...
    
    def set_progress_callback(self, callback: callable) -> None: ...
    def set_estimated_total(self, total: int) -> None: ...
    
    def start_message(self, header: MessageHeader, version: str) -> None: ...
    
    def write_resource(
        self,
        resource_id: str,
        title: str,
        artist: str,
        isrc: str | None = None,
        duration: str | None = None,
        file_path: str | None = None,
    ) -> str: ...
    
    def write_release(
        self,
        release_id: str,
        title: str,
        artist: str,
        label: str | None,
        upc: str | None,
        release_date: str | None,
        genre: str | None,
        resource_references: list[str],
    ) -> str: ...
    
    def finish_message(self) -> StreamingStats: ...
    def get_xml(self) -> str: ...
```

### Data Classes

#### `ValidationResult`

```python
class ValidationResult:
    is_valid: bool
    errors: list[str]
    warnings: list[str]
```

#### `BuilderStats`

```python
class BuilderStats:
    releases_count: int
    resources_count: int
    total_build_time_ms: float
    last_build_size_bytes: float
    validation_errors: int
    validation_warnings: int
```

#### `StreamingConfig`

```python
class StreamingConfig:
    max_buffer_size: int = 1048576  # 1MB default
    deterministic: bool = True
    validate_during_stream: bool = True
    progress_callback_frequency: int = 100
```

#### `MessageHeader`

```python
class MessageHeader:
    message_id: str | None = None
    message_sender_name: str
    message_recipient_name: str
    message_created_date_time: str | None = None
```

### Utility Functions

#### `batch_build(requests: list[dict]) -> list[str]`

Process multiple build requests in parallel.

```python
import ddex_builder

requests = [
    {"releases": [{"release_id": "R001", "title": "Album 1"}]},
    {"releases": [{"release_id": "R002", "title": "Album 2"}]}
]

results = ddex_builder.batch_build(requests)
```

#### `validate_structure(xml: str) -> ValidationResult`

Validate XML structure without building.

```python
xml = '<?xml version="1.0"?><root><element>test</element></root>'
result = ddex_builder.validate_structure(xml)
print(f"Valid: {result.is_valid}")
```

## DataFrame Integration Guide

### Expected DataFrame Formats

#### Releases DataFrame

```python
releases_df = pd.DataFrame({
    'release_id': ['R001', 'R002'],           # Required
    'release_type': ['Album', 'Single'],     # Required  
    'title': ['Album Title', 'Single Title'], # Required
    'artist': ['Artist 1', 'Artist 2'],      # Required
    'label': ['Label 1', 'Label 2'],         # Optional
    'catalog_number': ['CAT001', 'CAT002'],  # Optional
    'upc': ['123456789012', '123456789013'], # Optional
    'release_date': ['2024-01-01', '2024-02-01'], # Optional
    'genre': ['Rock', 'Pop'],                # Optional
    'parental_warning': [False, False],      # Optional
    'track_ids': [['T001', 'T002'], ['T003']] # Required (list of track IDs)
})
```

#### Resources DataFrame

```python
resources_df = pd.DataFrame({
    'resource_id': ['T001', 'T002', 'T003'],     # Required
    'resource_type': ['SoundRecording'] * 3,    # Required
    'title': ['Track 1', 'Track 2', 'Track 3'], # Required  
    'artist': ['Artist 1', 'Artist 1', 'Artist 2'], # Required
    'isrc': ['USRC17607001', 'USRC17607002', 'USRC17607003'], # Optional
    'duration': ['PT3M30S', 'PT4M15S', 'PT3M45S'],           # Optional
    'track_number': [1, 2, 1],               # Optional
    'volume_number': [1, 1, 1],              # Optional
})
```

### DataFrame Processing

```python
import pandas as pd
import ddex_builder

# Load data from various sources
releases_df = pd.read_csv('releases.csv')
resources_df = pd.read_excel('tracks.xlsx')

# Or from database
# releases_df = pd.read_sql('SELECT * FROM releases', connection)

# Clean and validate data
releases_df = releases_df.dropna(subset=['release_id', 'title', 'artist'])
resources_df = resources_df.dropna(subset=['resource_id', 'title', 'artist'])

# Build DDEX
builder = ddex_builder.DdexBuilder()
builder.from_dataframe(releases_df)
builder.from_dataframe(resources_df)

# Validate and build
validation = builder.validate()
if validation.is_valid:
    xml = builder.build()
    # Save to file
    with open('output.xml', 'w', encoding='utf-8') as f:
        f.write(xml)
```

## Partner Presets

Pre-configured settings for major music platforms:

```python
import ddex_builder

builder = ddex_builder.DdexBuilder()

# List available presets
presets = builder.get_available_presets()
print(presets)  # ['spotify', 'apple', 'youtube', 'generic']

# Get preset information
spotify_info = builder.get_preset_info('spotify')
print(f"Preset: {spotify_info.name}")
print(f"Description: {spotify_info.description}")
print(f"Required fields: {spotify_info.required_fields}")

# Apply preset
builder.apply_preset('spotify')

# Get validation rules for preset
rules = builder.get_preset_validation_rules('spotify')
for rule in rules:
    print(f"Field: {rule.field_name}, Rule: {rule.rule_type}")
```

## Performance Guidelines

### Memory Usage

- **Small datasets (< 100 releases)**: Use `DdexBuilder`
- **Medium datasets (100-1000 releases)**: Use `DdexBuilder` with batch processing
- **Large datasets (> 1000 releases)**: Use `StreamingDdexBuilder`

### Performance Benchmarks

| Dataset Size | Builder Type | Build Time | Memory Usage |
|-------------|-------------|------------|--------------|
| 10 releases | DdexBuilder | ~5ms | ~1MB |
| 100 releases | DdexBuilder | ~50ms | ~10MB |
| 1,000 releases | Streaming | ~500ms | ~50MB |
| 10,000 releases | Streaming | ~5s | ~100MB |

### Optimization Tips

```python
# Use streaming for large datasets
if len(releases_df) > 1000:
    builder = ddex_builder.StreamingDdexBuilder()
else:
    builder = ddex_builder.DdexBuilder()

# Batch DataFrame operations
builder.from_dataframe(releases_df)  # Process all at once
builder.from_dataframe(resources_df)  # Rather than row by row

# Disable validation during streaming if not needed
config = ddex_builder.StreamingConfig(validate_during_stream=False)
builder = ddex_builder.StreamingDdexBuilder(config)

# Use appropriate buffer sizes
config = ddex_builder.StreamingConfig(
    max_buffer_size=2 * 1024 * 1024  # 2MB for large datasets
)
```

## Error Handling

```python
import ddex_builder

try:
    builder = ddex_builder.DdexBuilder()
    
    # Add data...
    release = ddex_builder.Release(
        release_id='R001',
        release_type='Album', 
        title='Test Album',
        artist='Test Artist'
    )
    builder.add_release(release)
    
    # Validate first
    validation = builder.validate()
    if not validation.is_valid:
        print("Validation errors:")
        for error in validation.errors:
            print(f"  - {error}")
        return
    
    # Build XML
    xml = builder.build()
    
except ddex_builder.ValidationError as e:
    print(f"Validation failed: {e}")
except ddex_builder.BuildError as e:
    print(f"Build failed: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## Examples

### Complete Album Processing

```python
import pandas as pd
import ddex_builder
from datetime import datetime

def process_album_release():
    """Process a complete album release with multiple tracks"""
    
    # Album metadata
    album_data = {
        'release_id': 'ALB2024001',
        'release_type': 'Album',
        'title': 'Digital Dreams',
        'artist': 'Future Sounds',
        'label': 'Electronic Records',
        'catalog_number': 'ER2024001',
        'upc': '123456789012',
        'release_date': '2024-03-15',
        'genre': 'Electronic',
        'parental_warning': False
    }
    
    # Track data
    tracks_data = [
        {
            'resource_id': 'TRK001',
            'title': 'Digital Awakening',
            'isrc': 'USRC17607001',
            'duration': 'PT4M15S',
            'track_number': 1
        },
        {
            'resource_id': 'TRK002', 
            'title': 'Neon Nights',
            'isrc': 'USRC17607002',
            'duration': 'PT3M45S',
            'track_number': 2
        },
        {
            'resource_id': 'TRK003',
            'title': 'Cyber Dreams',
            'isrc': 'USRC17607003',
            'duration': 'PT5M22S',
            'track_number': 3
        }
    ]
    
    # Build DDEX
    builder = ddex_builder.DdexBuilder()
    
    # Add release
    track_ids = [track['resource_id'] for track in tracks_data]
    release = ddex_builder.Release(
        track_ids=track_ids,
        **album_data
    )
    builder.add_release(release)
    
    # Add tracks
    for track in tracks_data:
        resource = ddex_builder.Resource(
            resource_type='SoundRecording',
            artist=album_data['artist'],
            volume_number=1,
            **track
        )
        builder.add_resource(resource)
    
    # Validate and build
    validation = builder.validate()
    if validation.is_valid:
        xml = builder.build()
        
        # Save with timestamp
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        filename = f"ddex_release_{album_data['release_id']}_{timestamp}.xml"
        
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(xml)
            
        print(f"✅ DDEX XML generated: {filename}")
        print(f"   Size: {len(xml):,} bytes")
        
        # Show stats
        stats = builder.get_stats()
        print(f"   Releases: {stats.releases_count}")
        print(f"   Resources: {stats.resources_count}")
        print(f"   Build time: {stats.total_build_time_ms:.2f}ms")
        
        return filename
    else:
        print("❌ Validation failed:")
        for error in validation.errors:
            print(f"   - {error}")
        return None

# Run the example
if __name__ == "__main__":
    process_album_release()
```

### Batch Processing from CSV

```python
import pandas as pd
import ddex_builder
import os

def batch_process_from_csv():
    """Process multiple releases from CSV files"""
    
    # Load data from CSV files
    releases_df = pd.read_csv('releases.csv')
    resources_df = pd.read_csv('resources.csv')
    
    # Data validation
    print(f"Loaded {len(releases_df)} releases and {len(resources_df)} resources")
    
    # Group by release for batch processing
    release_groups = releases_df.groupby('release_id')
    
    results = []
    
    for release_id, release_group in release_groups:
        builder = ddex_builder.DdexBuilder()
        
        # Process each release
        for _, release_row in release_group.iterrows():
            # Get associated tracks
            track_resources = resources_df[
                resources_df['resource_id'].isin(release_row['track_ids'])
            ]
            
            # Add release
            release = ddex_builder.Release(
                release_id=release_row['release_id'],
                release_type=release_row['release_type'],
                title=release_row['title'],
                artist=release_row['artist'],
                label=release_row.get('label'),
                upc=release_row.get('upc'),
                release_date=release_row.get('release_date'),
                genre=release_row.get('genre'),
                track_ids=release_row['track_ids']
            )
            builder.add_release(release)
            
            # Add resources
            for _, resource_row in track_resources.iterrows():
                resource = ddex_builder.Resource(
                    resource_id=resource_row['resource_id'],
                    resource_type=resource_row['resource_type'],
                    title=resource_row['title'],
                    artist=resource_row['artist'],
                    isrc=resource_row.get('isrc'),
                    duration=resource_row.get('duration'),
                    track_number=resource_row.get('track_number')
                )
                builder.add_resource(resource)
        
        # Build XML
        try:
            xml = builder.build()
            filename = f"ddex_{release_id}.xml"
            
            with open(filename, 'w', encoding='utf-8') as f:
                f.write(xml)
                
            results.append({
                'release_id': release_id,
                'filename': filename,
                'size': len(xml),
                'success': True
            })
            
            print(f"✅ Generated {filename}")
            
        except Exception as e:
            results.append({
                'release_id': release_id,
                'error': str(e),
                'success': False
            })
            print(f"❌ Failed to generate {release_id}: {e}")
    
    # Summary
    successful = sum(1 for r in results if r['success'])
    print(f"\nProcessed {len(results)} releases: {successful} successful, {len(results) - successful} failed")
    
    return results

# Example CSV structure needed:
# releases.csv: release_id,release_type,title,artist,label,upc,release_date,genre,track_ids
# resources.csv: resource_id,resource_type,title,artist,isrc,duration,track_number
```

### Jupyter Notebook Integration

```python
# Cell 1: Setup
import pandas as pd
import ddex_builder
import matplotlib.pyplot as plt
from IPython.display import display, HTML

# Cell 2: Load and explore data
releases_df = pd.read_csv('music_releases.csv')
display(releases_df.head())

print(f"Dataset: {len(releases_df)} releases")
print(f"Genres: {releases_df['genre'].value_counts().to_dict()}")

# Cell 3: Visualize data
genre_counts = releases_df['genre'].value_counts()
plt.figure(figsize=(10, 6))
genre_counts.plot(kind='bar')
plt.title('Releases by Genre')
plt.xlabel('Genre')
plt.ylabel('Number of Releases')
plt.xticks(rotation=45)
plt.tight_layout()
plt.show()

# Cell 4: Build DDEX
builder = ddex_builder.DdexBuilder()
builder.from_dataframe(releases_df)

# Show progress
stats = builder.get_stats()
print(f"Builder loaded: {stats.releases_count} releases, {stats.resources_count} resources")

# Cell 5: Validate and generate
validation = builder.validate()
if validation.is_valid:
    xml = builder.build()
    
    # Show XML sample in notebook
    xml_preview = xml[:1000] + "..." if len(xml) > 1000 else xml
    display(HTML(f"<pre>{xml_preview}</pre>"))
    
    # Download link
    with open('notebook_output.xml', 'w') as f:
        f.write(xml)
    
    print(f"✅ Generated DDEX XML ({len(xml):,} bytes)")
    print("📁 Saved as 'notebook_output.xml'")
else:
    print("❌ Validation failed:")
    for error in validation.errors:
        print(f"   - {error}")
```

## Testing

Run the included test suite:

```bash
# Basic tests
python -m pytest test_builder.py

# With coverage
pip install pytest-cov
python -m pytest test_builder.py --cov=ddex_builder

# Run specific test
python test_builder.py  # Run all tests directly
```

## Troubleshooting

### Common Issues

**ImportError: No module named 'ddex_builder'**
```bash
# Ensure package is installed
pip install ddex-builder

# Check installation
python -c "import ddex_builder; print(ddex_builder.__file__)"
```

**Build failures with pandas DataFrames**
```python
# Ensure required columns are present
required_release_cols = ['release_id', 'release_type', 'title', 'artist', 'track_ids']
missing_cols = set(required_release_cols) - set(releases_df.columns)
if missing_cols:
    print(f"Missing required columns: {missing_cols}")
```

**Memory issues with large datasets**
```python
# Use streaming builder
config = ddex_builder.StreamingConfig(
    max_buffer_size=512 * 1024,  # Reduce buffer size
    validate_during_stream=False  # Disable validation to save memory
)
builder = ddex_builder.StreamingDdexBuilder(config)
```

**XML validation errors**
```python
# Check validation details
validation = builder.validate()
if not validation.is_valid:
    print("Errors:")
    for error in validation.errors:
        print(f"  {error}")
    print("Warnings:")
    for warning in validation.warnings:
        print(f"  {warning}")
```

### Debug Mode

```python
import os
import logging

# Enable debug logging
logging.basicConfig(level=logging.DEBUG)

# Set environment variable for Rust debug output
os.environ['RUST_LOG'] = 'debug'

# Create builder with debug info
builder = ddex_builder.DdexBuilder()
stats = builder.get_stats()
print(f"Debug stats: {stats}")
```

## Related Projects

- [ddex-parser](https://pypi.org/project/ddex-parser/) - Parse existing DDEX XML files
- [DDEX Suite](https://github.com/daddykev/ddex-suite) - Complete DDEX processing toolkit

## Contributing

Contributions welcome! Please see the [main repository](https://github.com/daddykev/ddex-suite) for contribution guidelines.

## License

MIT License - see [LICENSE](https://github.com/daddykev/ddex-suite/blob/main/LICENSE) for details.

## Support

- 📖 [Documentation](https://github.com/daddykev/ddex-suite/tree/main/docs)
- 🐛 [Issue Tracker](https://github.com/daddykev/ddex-suite/issues)
- 💬 [Discussions](https://github.com/daddykev/ddex-suite/discussions)
- 📊 [PyPI Package](https://pypi.org/project/ddex-builder/)