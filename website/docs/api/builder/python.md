# Python API Reference

Complete API documentation for the DDEX Builder Python bindings with pandas integration.

## Installation

```bash
pip install ddex-builder
```

## Imports

```python
from ddex_builder import (
    DdexBuilder, 
    StreamingDdexBuilder,
    ValidationResult,
    BuilderStats,
    PresetInfo,
    batch_build,
    validate_structure
)
```

## Classes

### DdexBuilder

Main builder class for creating deterministic DDEX XML messages in Python.

```python
class DdexBuilder:
    def __init__(self) -> None: ...
    def add_release(self, release: Dict[str, Any]) -> None: ...
    def add_resource(self, resource: Dict[str, Any]) -> None: ...
    async def build(self, data: Optional[Dict[str, Any]] = None) -> str: ...
    def build_sync(self, data: Optional[Dict[str, Any]] = None) -> str: ...
    async def validate(self) -> ValidationResult: ...
    def get_stats(self) -> BuilderStats: ...
    def reset(self) -> None: ...
    def get_available_presets(self) -> List[str]: ...
    def get_preset_info(self, preset_name: str) -> PresetInfo: ...
    def apply_preset(self, preset_name: str) -> None: ...
    def get_preset_validation_rules(self, preset_name: str) -> List[ValidationRule]: ...
    def from_dataframe(self, df: 'pd.DataFrame', version: str = '4.3') -> str: ...
```

#### Constructor

```python
builder = DdexBuilder()
```

Creates a new DDEX builder instance with default configuration.

#### add_release()

```python
def add_release(self, release: Dict[str, Any]) -> None
```

Adds a release to the message being built.

**Parameters:**
- `release: Dict[str, Any]` - Release data dictionary

**Example:**
```python
from ddex_builder import DdexBuilder

builder = DdexBuilder()

release = {
    'release_id': 'REL001',
    'release_type': 'Album',
    'title': 'My Album',
    'artist': 'Artist Name',
    'label': 'Record Label',
    'catalog_number': 'CAT001',
    'upc': '123456789012',
    'release_date': '2024-01-15',
    'genre': 'Pop',
    'parental_warning': False,
    'track_ids': ['TRK001', 'TRK002', 'TRK003'],
    'metadata': {
        'custom_field': 'custom_value'
    }
}

builder.add_release(release)
```

#### add_resource()

```python
def add_resource(self, resource: Dict[str, Any]) -> None
```

Adds a resource (sound recording, video, etc.) to the message.

**Parameters:**
- `resource: Dict[str, Any]` - Resource data dictionary

**Example:**
```python
resource = {
    'resource_id': 'TRK001',
    'resource_type': 'SoundRecording',
    'title': 'Track Title',
    'artist': 'Artist Name',
    'isrc': 'USRC17607839',
    'duration': 'PT3M45S',
    'track_number': 1,
    'volume_number': 1,
    'metadata': {
        'composer': 'Composer Name',
        'producer': 'Producer Name'
    }
}

builder.add_resource(resource)
```

#### build()

```python
async def build(self, data: Optional[Dict[str, Any]] = None) -> str
```

Asynchronously builds the DDEX XML message from added releases and resources.

**Parameters:**
- `data: Optional[Dict[str, Any]]` - Optional additional message data

**Returns:** `str` - Generated DDEX XML

**Example:**
```python
import asyncio
from ddex_builder import DdexBuilder

async def build_catalog():
    builder = DdexBuilder()
    builder.apply_preset('youtube_album')
    
    # Add releases and resources...
    builder.add_release(release)
    builder.add_resource(resource)
    
    # Build the XML
    xml = await builder.build({
        'message_id': 'MSG_2024_001',
        'sender': 'MyLabel',
        'recipient': 'Spotify',
        'version': '4.3'
    })
    
    print(f'Generated XML: {len(xml)} characters')
    return xml

# Run async function
xml = asyncio.run(build_catalog())
```

#### build_sync()

```python
def build_sync(self, data: Optional[Dict[str, Any]] = None) -> str
```

Synchronously builds the DDEX XML message.

**Parameters:**
- `data: Optional[Dict[str, Any]]` - Optional additional message data

**Returns:** `str` - Generated DDEX XML

**Example:**
```python
builder = DdexBuilder()
builder.apply_preset('youtube_album')
builder.add_release(release)

# Synchronous build
xml = builder.build_sync({
    'message_id': 'MSG_2024_001',
    'version': '4.3'
})

# Save to file
with open('output.xml', 'w') as f:
    f.write(xml)
```

#### validate()

```python
async def validate(self) -> ValidationResult
```

Validates the current state of the builder without generating XML.

**Returns:** `ValidationResult` - Validation results object

**Example:**
```python
import asyncio

async def validate_and_build():
    builder = DdexBuilder()
    builder.add_release(release)
    
    validation = await builder.validate()
    
    if validation.is_valid:
        print('✓ Validation passed')
        xml = await builder.build()
        return xml
    else:
        print('✗ Validation failed:')
        for error in validation.errors:
            print(f'  - {error}')
        
        if validation.warnings:
            print('Warnings:')
            for warning in validation.warnings:
                print(f'  ! {warning}')
        
        return None

result = asyncio.run(validate_and_build())
```

#### get_stats()

```python
def get_stats(self) -> BuilderStats
```

Returns statistics about the builder's current state and performance.

**Returns:** `BuilderStats` - Builder statistics object

**Example:**
```python
builder = DdexBuilder()
# Add data and build...

stats = builder.get_stats()
print(f'Releases: {stats.releases_count}')
print(f'Resources: {stats.resources_count}')
print(f'Build time: {stats.total_build_time_ms}ms')
print(f'Output size: {stats.last_build_size_bytes} bytes')
print(f'Validation errors: {stats.validation_errors}')
print(f'Validation warnings: {stats.validation_warnings}')
```

#### reset()

```python
def reset(self) -> None
```

Clears all added releases, resources, and statistics.

**Example:**
```python
builder = DdexBuilder()
# Add data...
builder.add_release(release)

# Clear everything
builder.reset()

# Builder is now empty and ready for new data
```

#### get_available_presets()

```python
def get_available_presets(self) -> List[str]
```

Returns list of available platform presets.

**Returns:** `List[str]` - List of preset names

**Example:**
```python
builder = DdexBuilder()
presets = builder.get_available_presets()
print('Available presets:', presets)
# Output: ['youtube_album', 'generic_audio_album', 'youtube_music', 'generic_audio_single', 'generic_audio_album']
```

#### get_preset_info()

```python
def get_preset_info(self, preset_name: str) -> PresetInfo
```

Gets detailed information about a specific preset.

**Parameters:**
- `preset_name: str` - Name of the preset

**Returns:** `PresetInfo` - Preset information object

**Example:**
```python
builder = DdexBuilder()
preset_info = builder.get_preset_info('youtube_album')

print(f'Name: {preset_info.name}')
print(f'Description: {preset_info.description}')
print(f'Version: {preset_info.version}')
print(f'Profile: {preset_info.profile}')
print(f'Required fields: {", ".join(preset_info.required_fields)}')
print(f'Disclaimer: {preset_info.disclaimer}')
```

#### apply_preset()

```python
def apply_preset(self, preset_name: str) -> None
```

Applies a platform-specific preset configuration.

**Parameters:**
- `preset_name: str` - Name of the preset to apply

**Raises:** `ValueError` - If preset name is invalid

**Example:**
```python
builder = DdexBuilder()

try:
    # Apply Spotify preset
    builder.apply_preset('youtube_album')
    print('✓ Spotify preset applied')
    
    # The builder is now configured for Spotify requirements
    # - Specific validation rules
    # - Required fields
    # - Format preferences
    
except ValueError as e:
    print(f'✗ Invalid preset: {e}')
```

#### get_preset_validation_rules()

```python
def get_preset_validation_rules(self, preset_name: str) -> List[ValidationRule]
```

Gets the validation rules for a specific preset.

**Parameters:**
- `preset_name: str` - Name of the preset

**Returns:** `List[ValidationRule]` - List of validation rule objects

**Example:**
```python
builder = DdexBuilder()
rules = builder.get_preset_validation_rules('youtube_album')

for rule in rules:
    print(f'Field: {rule.field_name}')
    print(f'Rule: {rule.rule_type}')
    print(f'Message: {rule.message}')
    if rule.parameters:
        print(f'Parameters: {rule.parameters}')
    print('---')
```

#### from_dataframe()

```python
def from_dataframe(self, df: 'pd.DataFrame', version: str = '4.3') -> str
```

Builds DDEX XML directly from a pandas DataFrame.

**Parameters:**
- `df: pd.DataFrame` - DataFrame with DDEX data
- `version: str` - DDEX version to generate ('3.8.2', '4.2', or '4.3')

**Returns:** `str` - Generated DDEX XML

**Raises:** `ImportError` - If pandas is not installed

**Example:**
```python
import pandas as pd
from ddex_builder import DdexBuilder

# Create sample DataFrame
data = [
    {
        'release_id': 'REL001',
        'title': 'Album 1',
        'artist': 'Artist A',
        'label': 'Label X',
        'release_date': '2024-01-15',
        'genre': 'Pop',
        'sound_recording_id': 'TRK001',
        'track_title': 'Track 1',
        'isrc': 'USRC17607839',
        'duration': 'PT3M45S'
    },
    {
        'release_id': 'REL001',
        'title': 'Album 1',
        'artist': 'Artist A',
        'label': 'Label X',
        'release_date': '2024-01-15',
        'genre': 'Pop',
        'sound_recording_id': 'TRK002',
        'track_title': 'Track 2',
        'isrc': 'USRC17607840',
        'duration': 'PT4M12S'
    }
]

df = pd.DataFrame(data)

# Build DDEX from DataFrame
builder = DdexBuilder()
builder.apply_preset('youtube_album')
xml = builder.from_dataframe(df, version='4.3')

print(f'Generated XML from DataFrame: {len(xml)} characters')
```

---

### StreamingDdexBuilder

Streaming builder for memory-efficient generation of large DDEX catalogs.

```python
class StreamingDdexBuilder:
    def __init__(self, config: Optional[Dict[str, Any]] = None) -> None: ...
    def set_progress_callback(self, callback: Callable) -> None: ...
    def set_estimated_total(self, total: int) -> None: ...
    def start_message(self, header: Dict[str, Any], version: str) -> None: ...
    def write_resource(self, resource_id: str, title: str, artist: str, isrc: Optional[str] = None, duration: Optional[str] = None, file_path: Optional[str] = None) -> str: ...
    def finish_resources_start_releases(self) -> None: ...
    def write_release(self, release_id: str, title: str, artist: str, label: Optional[str], upc: Optional[str], release_date: Optional[str], genre: Optional[str], resource_references: List[str]) -> str: ...
    def finish_message(self) -> StreamingStats: ...
    def get_xml(self) -> str: ...
    def reset(self) -> None: ...
```

#### Constructor

```python
stream_builder = StreamingDdexBuilder(config=None)
```

Creates a new streaming builder with optional configuration.

**Parameters:**
- `config: Optional[Dict[str, Any]]` - Optional streaming configuration

**Example:**
```python
from ddex_builder import StreamingDdexBuilder

config = {
    'max_buffer_size': 10 * 1024 * 1024,  # 10MB buffer
    'deterministic': True,
    'validate_during_stream': True,
    'progress_callback_frequency': 100    # Callback every 100 items
}

stream_builder = StreamingDdexBuilder(config)
```

#### set_progress_callback()

```python
def set_progress_callback(self, callback: Callable) -> None
```

Sets a callback function to receive progress updates during streaming.

**Parameters:**
- `callback: Callable` - Progress callback function

**Example:**
```python
def progress_callback(progress):
    percent = progress.get('estimated_completion_percent', 0)
    print(f'Progress: {percent:.1f}%')
    print(f'Releases: {progress["releases_written"]}')
    print(f'Memory: {progress["current_memory_usage"] / 1024 / 1024:.1f}MB')

stream_builder = StreamingDdexBuilder()
stream_builder.set_progress_callback(progress_callback)
```

#### set_estimated_total()

```python
def set_estimated_total(self, total: int) -> None
```

Sets the estimated total number of items for accurate progress reporting.

**Parameters:**
- `total: int` - Estimated total number of releases

**Example:**
```python
stream_builder = StreamingDdexBuilder()
stream_builder.set_estimated_total(10000)  # Expecting 10,000 releases
```

#### start_message()

```python
def start_message(self, header: Dict[str, Any], version: str) -> None
```

Starts a new DDEX message with the specified header and version.

**Parameters:**
- `header: Dict[str, Any]` - Message header information
- `version: str` - DDEX version ('3.8.2', '4.2', or '4.3')

**Example:**
```python
from datetime import datetime

stream_builder = StreamingDdexBuilder()

header = {
    'message_id': 'MSG_CATALOG_2024_001',
    'message_sender_name': 'MyRecordLabel',
    'message_recipient_name': 'Spotify',
    'message_created_date_time': datetime.now().isoformat()
}

stream_builder.start_message(header, '4.3')
```

#### write_resource()

```python
def write_resource(self, resource_id: str, title: str, artist: str, isrc: Optional[str] = None, duration: Optional[str] = None, file_path: Optional[str] = None) -> str
```

Writes a resource (sound recording) to the streaming output.

**Parameters:**
- `resource_id: str` - Unique resource identifier
- `title: str` - Resource title
- `artist: str` - Artist name
- `isrc: Optional[str]` - Optional ISRC code
- `duration: Optional[str]` - Optional duration (ISO 8601 format)
- `file_path: Optional[str]` - Optional file path reference

**Returns:** `str` - Generated resource reference ID

**Example:**
```python
stream_builder = StreamingDdexBuilder()
stream_builder.start_message(header, '4.3')

resource_ref = stream_builder.write_resource(
    resource_id='RES_001',
    title='Track Title',
    artist='Artist Name',
    isrc='USRC17607839',
    duration='PT3M45S',
    file_path='/audio/track001.wav'
)

print(f'Resource reference: {resource_ref}')
```

#### finish_resources_start_releases()

```python
def finish_resources_start_releases(self) -> None
```

Finishes the resources section and starts the releases section.

**Example:**
```python
stream_builder = StreamingDdexBuilder()
stream_builder.start_message(header, '4.3')

# Write all resources...
resource_refs = []
for track_data in tracks:
    ref = stream_builder.write_resource(
        track_data['resource_id'],
        track_data['title'],
        track_data['artist'],
        track_data.get('isrc'),
        track_data.get('duration')
    )
    resource_refs.append(ref)

# Transition to releases
stream_builder.finish_resources_start_releases()

# Now write releases...
```

#### write_release()

```python
def write_release(self, release_id: str, title: str, artist: str, label: Optional[str], upc: Optional[str], release_date: Optional[str], genre: Optional[str], resource_references: List[str]) -> str
```

Writes a release to the streaming output.

**Parameters:**
- `release_id: str` - Unique release identifier
- `title: str` - Release title
- `artist: str` - Primary artist
- `label: Optional[str]` - Record label name
- `upc: Optional[str]` - Universal Product Code
- `release_date: Optional[str]` - Release date (ISO 8601)
- `genre: Optional[str]` - Musical genre
- `resource_references: List[str]` - List of resource reference IDs

**Returns:** `str` - Generated release reference ID

**Example:**
```python
release_ref = stream_builder.write_release(
    release_id='REL_001',
    title='Album Title',
    artist='Artist Name',
    label='Record Label',
    upc='123456789012',
    release_date='2024-01-15',
    genre='Pop',
    resource_references=[resource_ref1, resource_ref2, resource_ref3]
)

print(f'Release reference: {release_ref}')
```

#### finish_message()

```python
def finish_message(self) -> StreamingStats
```

Finishes the message and returns statistics.

**Returns:** `StreamingStats` - Final streaming statistics object

**Example:**
```python
stream_builder = StreamingDdexBuilder()
# Build the message...

stats = stream_builder.finish_message()
print(f'Final stats:')
print(f'  Releases written: {stats.releases_written}')
print(f'  Resources written: {stats.resources_written}')
print(f'  Deals written: {stats.deals_written}')
print(f'  Total bytes: {stats.bytes_written}')
print(f'  Peak memory: {stats.peak_memory_usage} bytes')

if stats.warnings:
    print(f'Warnings:')
    for warning in stats.warnings:
        print(f'  - {warning}')
```

#### get_xml()

```python
def get_xml(self) -> str
```

Returns the generated XML content.

**Returns:** `str` - Complete DDEX XML

**Example:**
```python
stream_builder = StreamingDdexBuilder()
# Build the message...
stream_builder.finish_message()

xml = stream_builder.get_xml()
print(f'Generated {len(xml)} characters of XML')

# Save to file
with open('catalog.xml', 'w', encoding='utf-8') as f:
    f.write(xml)
```

#### reset()

```python
def reset(self) -> None
```

Resets the streaming builder for a new message.

**Example:**
```python
stream_builder = StreamingDdexBuilder()
# Build first message...
stream_builder.finish_message()

# Reset for next message
stream_builder.reset()
stream_builder.start_message(new_header, '4.3')
```

---

## Global Functions

### batch_build()

```python
async def batch_build(requests: List[str]) -> List[str]
```

Builds multiple DDEX messages in a single operation for improved performance.

**Parameters:**
- `requests: List[str]` - List of JSON-serialized build requests

**Returns:** `List[str]` - List of generated XML strings

**Example:**
```python
import asyncio
import json
from ddex_builder import batch_build

async def build_multiple_catalogs():
    requests = [
        json.dumps({'releases': [release1], 'version': '4.3'}),
        json.dumps({'releases': [release2], 'version': '4.3'}),
        json.dumps({'releases': [release3], 'version': '4.3'})
    ]
    
    xml_results = await batch_build(requests)
    
    for i, xml in enumerate(xml_results):
        print(f'Request {i + 1}: {len(xml)} characters')
        with open(f'catalog_{i + 1}.xml', 'w') as f:
            f.write(xml)
    
    return xml_results

results = asyncio.run(build_multiple_catalogs())
```

### validate_structure()

```python
async def validate_structure(xml: str) -> ValidationResult
```

Validates the structure of existing DDEX XML without building.

**Parameters:**
- `xml: str` - DDEX XML content to validate

**Returns:** `ValidationResult` - Validation results object

**Example:**
```python
import asyncio
from ddex_builder import validate_structure

async def validate_existing_file():
    with open('existing_ddex.xml', 'r', encoding='utf-8') as f:
        xml_content = f.read()
    
    validation = await validate_structure(xml_content)
    
    if validation.is_valid:
        print('✓ XML structure is valid')
        print(f'Detected version: {validation.version}')
    else:
        print('✗ XML structure has errors:')
        for error in validation.errors:
            print(f'  - {error}')
    
    if validation.warnings:
        print('Warnings:')
        for warning in validation.warnings:
            print(f'  ! {warning}')
    
    return validation

result = asyncio.run(validate_existing_file())
```

---

## DataFrame Integration

### Building from DataFrame

The Python API provides seamless integration with pandas DataFrames:

```python
import pandas as pd
from ddex_builder import DdexBuilder

def build_from_csv(csv_file_path: str, output_file: str):
    # Load CSV data
    df = pd.read_csv(csv_file_path)
    
    # Basic data validation
    required_columns = ['release_id', 'title', 'artist', 'label']
    missing_columns = [col for col in required_columns if col not in df.columns]
    
    if missing_columns:
        raise ValueError(f'Missing required columns: {missing_columns}')
    
    # Build DDEX from DataFrame
    builder = DdexBuilder()
    builder.apply_preset('youtube_album')
    
    xml = builder.from_dataframe(df, version='4.3')
    
    # Save output
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(xml)
    
    print(f'✓ Built DDEX XML from {len(df)} records')
    print(f'✓ Saved to {output_file}')

# Usage
build_from_csv('catalog.csv', 'catalog.xml')
```

### Advanced DataFrame Processing

```python
import pandas as pd
from ddex_builder import DdexBuilder

class DataFrameDdexBuilder:
    def __init__(self, preset: str = 'generic_audio_album'):
        self.builder = DdexBuilder()
        self.builder.apply_preset(preset)
    
    def process_catalog_dataframe(self, df: pd.DataFrame) -> str:
        """Process a complex catalog DataFrame into DDEX XML"""
        
        # Data cleaning and validation
        df = self._clean_dataframe(df)
        df = self._validate_dataframe(df)
        
        # Group by release to handle multi-track releases
        releases = []
        for release_id, release_group in df.groupby('release_id'):
            release_data = self._build_release_data(release_group)
            releases.append(release_data)
        
        # Build using the cleaned data
        xml = self.builder.from_dataframe(df, version='4.3')
        return xml
    
    def _clean_dataframe(self, df: pd.DataFrame) -> pd.DataFrame:
        """Clean and standardize DataFrame data"""
        df = df.copy()
        
        # Standardize date formats
        if 'release_date' in df.columns:
            df['release_date'] = pd.to_datetime(df['release_date']).dt.strftime('%Y-%m-%d')
        
        # Clean artist names
        if 'artist' in df.columns:
            df['artist'] = df['artist'].str.strip()
            df['artist'] = df['artist'].str.replace(r'\s+', ' ', regex=True)
        
        # Validate ISRCs
        if 'isrc' in df.columns:
            isrc_pattern = r'^[A-Z]{2}[A-Z0-9]{3}\d{7}$'
            invalid_isrcs = ~df['isrc'].str.match(isrc_pattern, na=False)
            if invalid_isrcs.any():
                print(f'Warning: {invalid_isrcs.sum()} invalid ISRCs found')
        
        return df
    
    def _validate_dataframe(self, df: pd.DataFrame) -> pd.DataFrame:
        """Validate DataFrame has required fields"""
        required_fields = ['release_id', 'title', 'artist']
        
        for field in required_fields:
            if field not in df.columns:
                raise ValueError(f'Missing required field: {field}')
            
            missing_values = df[field].isna().sum()
            if missing_values > 0:
                print(f'Warning: {missing_values} missing values in {field}')
        
        return df
    
    def _build_release_data(self, release_group: pd.DataFrame) -> dict:
        """Build release data from grouped DataFrame"""
        first_row = release_group.iloc[0]
        
        return {
            'release_id': first_row['release_id'],
            'title': first_row['title'],
            'artist': first_row['artist'],
            'label': first_row.get('label', ''),
            'release_date': first_row.get('release_date', ''),
            'genre': first_row.get('genre', ''),
            'track_count': len(release_group),
            'tracks': release_group.to_dict('records')
        }

# Usage
builder = DataFrameDdexBuilder('youtube_album')

# Load complex catalog data
df = pd.read_csv('complex_catalog.csv')
xml = builder.process_catalog_dataframe(df)

with open('processed_catalog.xml', 'w') as f:
    f.write(xml)
```

---

## Error Handling

The Python API raises specific exceptions for different error conditions:

```python
from ddex_builder import DdexBuilder, ValidationError

async def safe_build_example():
    builder = DdexBuilder()
    
    try:
        builder.apply_preset('youtube_album')
        builder.add_release(release_data)
        
        # Validate before building
        validation = await builder.validate()
        if not validation.is_valid:
            raise ValidationError(f'Validation failed: {validation.errors}')
        
        xml = await builder.build()
        return xml
        
    except ValueError as e:
        if 'Invalid preset' in str(e):
            print(f'Preset error: {e}')
            print(f'Available presets: {builder.get_available_presets()}')
        elif 'Missing required field' in str(e):
            print(f'Required field missing: {e}')
        else:
            print(f'Value error: {e}')
            
    except ValidationError as e:
        print(f'Validation failed: {e}')
        # Could retry with different data or preset
        
    except MemoryError as e:
        print(f'Out of memory: {e}')
        print('Consider using StreamingDdexBuilder for large catalogs')
        
    except ImportError as e:
        if 'pandas' in str(e):
            print('pandas is required for DataFrame operations')
            print('Install with: pip install pandas')
        else:
            print(f'Missing dependency: {e}')
            
    except Exception as e:
        print(f'Unexpected error: {e}')
        
    return None
```

### Common Exception Types

- **`ValueError`**: Invalid data, preset names, or configuration
- **`ValidationError`**: DDEX validation failures
- **`MemoryError`**: Insufficient memory for large catalogs
- **`ImportError`**: Missing optional dependencies (pandas)
- **`FileNotFoundError`**: File I/O errors
- **`TypeError`**: Incorrect parameter types

---

## Performance Optimization

### Memory Management

```python
# Use streaming for large catalogs
config = {
    'max_buffer_size': 50 * 1024 * 1024,  # 50MB
    'validate_during_stream': False        # Validate at end for speed
}

stream_builder = StreamingDdexBuilder(config)

# Process in chunks
chunk_size = 1000
for i in range(0, len(releases), chunk_size):
    chunk = releases[i:i + chunk_size]
    for release in chunk:
        stream_builder.write_release(...)
    
    # Optional: Force garbage collection
    import gc
    gc.collect()
```

### Async Processing

```python
import asyncio
from ddex_builder import DdexBuilder

async def process_multiple_catalogs(release_batches):
    semaphore = asyncio.Semaphore(5)  # Limit concurrent operations
    
    async def process_batch(releases):
        async with semaphore:
            builder = DdexBuilder()
            builder.apply_preset('youtube_album')
            
            for release in releases:
                builder.add_release(release)
            
            return await builder.build()
    
    tasks = [process_batch(batch) for batch in release_batches]
    results = await asyncio.gather(*tasks, return_exceptions=True)
    
    # Handle results and exceptions
    successful = [r for r in results if isinstance(r, str)]
    errors = [r for r in results if isinstance(r, Exception)]
    
    return successful, errors
```

### Batch DataFrame Processing

```python
import pandas as pd
from ddex_builder import DdexBuilder

def process_large_catalog_efficiently(csv_path: str, output_dir: str):
    """Process large CSV files in chunks"""
    
    chunk_size = 10000
    chunk_num = 0
    
    for chunk_df in pd.read_csv(csv_path, chunksize=chunk_size):
        builder = DdexBuilder()
        builder.apply_preset('youtube_album')
        
        try:
            xml = builder.from_dataframe(chunk_df)
            
            output_file = f'{output_dir}/catalog_chunk_{chunk_num:03d}.xml'
            with open(output_file, 'w') as f:
                f.write(xml)
            
            print(f'✓ Processed chunk {chunk_num}: {len(chunk_df)} records')
            chunk_num += 1
            
        except Exception as e:
            print(f'✗ Failed to process chunk {chunk_num}: {e}')
            continue

# Usage
process_large_catalog_efficiently('huge_catalog.csv', './output/')
```