# Python API Reference

Complete API documentation for the DDEX Parser Python bindings.

## Installation

```bash
pip install ddex-parser
```

## Imports

```python
from ddex_parser import DDEXParser, ParseOptions, ParseResult, parse
```

## Classes

### DDEXParser

Main parser class for Python with pandas integration support.

```python
class DDEXParser:
    def __init__(self) -> None: ...
    def parse(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParseResult: ...
    async def parse_async(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParseResult: ...
    def stream(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> Iterator[Dict[str, Any]]: ...
    def to_dataframe(self, xml: Union[str, bytes], schema: str = 'flat') -> 'pd.DataFrame': ...
    def detect_version(self, xml: Union[str, bytes]) -> str: ...
    def sanity_check(self, xml: Union[str, bytes]) -> Dict[str, Any]: ...
```

#### Constructor

```python
parser = DDEXParser()
```

Creates a new DDEX parser instance with Rust backend.

#### parse()

```python
def parse(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParseResult
```

Synchronously parses DDEX XML content.

**Parameters:**
- `xml: Union[str, bytes]` - DDEX XML content as string or bytes
- `options: Optional[ParseOptions]` - Parsing configuration options

**Returns:** `ParseResult` - Parsed DDEX message structure

**Example:**
```python
from ddex_parser import DDEXParser, ParseOptions

parser = DDEXParser()
options = ParseOptions(
    include_raw_extensions=True,
    validate_references=True
)

with open('release.xml', 'r') as f:
    xml_content = f.read()

result = parser.parse(xml_content, options)
print(f"Parsed {result.release_count} releases")
print(f"Message ID: {result.message_id}")
```

#### parse_async()

```python
async def parse_async(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParseResult
```

Asynchronously parses DDEX XML content.

**Parameters:**
- `xml: Union[str, bytes]` - DDEX XML content
- `options: Optional[ParseOptions]` - Parsing configuration options

**Returns:** `ParseResult` - Parsed DDEX message structure

**Example:**
```python
import asyncio
from ddex_parser import DDEXParser

async def parse_multiple_files(file_paths):
    parser = DDEXParser()
    tasks = []
    
    for file_path in file_paths:
        with open(file_path, 'r') as f:
            xml_content = f.read()
        tasks.append(parser.parse_async(xml_content))
    
    results = await asyncio.gather(*tasks)
    return results

# Usage
results = asyncio.run(parse_multiple_files(['file1.xml', 'file2.xml']))
```

#### stream()

```python
def stream(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> Iterator[Dict[str, Any]]
```

Creates an iterator for streaming through releases in large DDEX files.

**Parameters:**
- `xml: Union[str, bytes]` - DDEX XML content
- `options: Optional[ParseOptions]` - Parsing configuration options

**Returns:** `Iterator[Dict[str, Any]]` - Iterator yielding release dictionaries

**Example:**
```python
parser = DDEXParser()
with open('large_catalog.xml', 'r') as f:
    xml_content = f.read()

for release in parser.stream(xml_content):
    print(f"Processing: {release['title']}")
    print(f"Artist: {release['artist']}")
    print(f"Release ID: {release['release_id']}")
```

#### to_dataframe()

```python
def to_dataframe(self, xml: Union[str, bytes], schema: str = 'flat') -> 'pd.DataFrame'
```

Converts DDEX XML directly to a pandas DataFrame for analysis.

**Parameters:**
- `xml: Union[str, bytes]` - DDEX XML content
- `schema: str` - Output schema: 'flat' (default) or 'graph'

**Returns:** `pd.DataFrame` - Structured DataFrame with DDEX data

**Raises:** `ImportError` - If pandas is not installed

**Example:**
```python
import pandas as pd
from ddex_parser import DDEXParser

parser = DDEXParser()

# Load XML file
with open('catalog.xml', 'r') as f:
    xml_content = f.read()

# Convert to DataFrame
df = parser.to_dataframe(xml_content, schema='flat')

# Analyze the data
print(f"DataFrame shape: {df.shape}")
print(f"Columns: {list(df.columns)}")

# Basic statistics
print(f"Total releases: {df['release_id'].nunique()}")
print(f"Total tracks: {df['sound_recording_id'].nunique()}")
print(f"Unique artists: {df['display_artist'].nunique()}")

# Top genres
genre_counts = df['genre'].value_counts()
print(f"Top genres: {genre_counts.head()}")
```

#### detect_version()

```python
def detect_version(self, xml: Union[str, bytes]) -> str
```

Detects the DDEX version from XML content.

**Parameters:**
- `xml: Union[str, bytes]` - DDEX XML content

**Returns:** `str` - Detected version (e.g., "4.3", "4.2", "3.8.2", "Unknown")

**Example:**
```python
parser = DDEXParser()
with open('unknown_version.xml', 'r') as f:
    xml_content = f.read()

version = parser.detect_version(xml_content)
print(f"Detected DDEX version: {version}")

if version == "Unknown":
    print("Warning: Could not detect DDEX version")
```

#### sanity_check()

```python
def sanity_check(self, xml: Union[str, bytes]) -> Dict[str, Any]
```

Performs a quick validation check on DDEX XML without full parsing.

**Parameters:**
- `xml: Union[str, bytes]` - DDEX XML content

**Returns:** `Dict[str, Any]` - Validation results with keys:
- `is_valid: bool` - Whether the XML passes basic validation
- `version: str` - Detected DDEX version
- `errors: List[str]` - List of validation errors
- `warnings: List[str]` - List of validation warnings

**Example:**
```python
parser = DDEXParser()
with open('suspicious_file.xml', 'r') as f:
    xml_content = f.read()

check = parser.sanity_check(xml_content)

if check['is_valid']:
    print(f"✓ Valid DDEX {check['version']} file")
else:
    print("✗ Invalid DDEX file")
    for error in check['errors']:
        print(f"  Error: {error}")
    
if check['warnings']:
    print("Warnings:")
    for warning in check['warnings']:
        print(f"  Warning: {warning}")
```

---

### ParseOptions

Configuration options for parsing behavior.

```python
class ParseOptions:
    def __init__(
        self,
        include_raw_extensions: bool = False,
        include_comments: bool = False,
        validate_references: bool = True,
        streaming: bool = False,
        timeout: float = 30.0,
    ) -> None: ...
    
    def to_dict(self) -> Dict[str, Any]: ...
```

#### Constructor Parameters

- **`include_raw_extensions: bool`** (default: `False`)  
  Include raw XML for extension elements to preserve round-trip fidelity
  
- **`include_comments: bool`** (default: `False`)  
  Include XML comments in the parsed output
  
- **`validate_references: bool`** (default: `True`)  
  Validate that all resource references are resolvable
  
- **`streaming: bool`** (default: `False`)  
  Enable streaming mode for large files
  
- **`timeout: float`** (default: `30.0`)  
  Parsing timeout in seconds

**Example:**
```python
from ddex_parser import ParseOptions

# Default options
options = ParseOptions()

# Custom options for round-trip processing
roundtrip_options = ParseOptions(
    include_raw_extensions=True,
    include_comments=True,
    validate_references=True
)

# Options for large file processing
streaming_options = ParseOptions(
    streaming=True,
    validate_references=False,
    timeout=120.0
)
```

#### to_dict()

```python
def to_dict(self) -> Dict[str, Any]
```

Converts options to dictionary format for internal use.

**Returns:** `Dict[str, Any]` - Dictionary representation of options

---

### ParseResult

Result object containing parsed DDEX message data.

```python
class ParseResult:
    def __init__(self, data: Dict[str, Any]) -> None: ...
    
    message_id: str
    version: str
    release_count: int
    releases: List[Dict[str, Any]]
```

#### Properties

- **`message_id: str`** - Unique message identifier from DDEX XML
- **`version: str`** - Detected DDEX version
- **`release_count: int`** - Number of releases in the message
- **`releases: List[Dict[str, Any]]`** - List of release data dictionaries

**Example:**
```python
result = parser.parse(xml_content)

print(f"Message ID: {result.message_id}")
print(f"DDEX Version: {result.version}")
print(f"Release Count: {result.release_count}")

# Access individual releases
for release in result.releases:
    print(f"Release: {release.get('title', 'Unknown')}")
    print(f"Artist: {release.get('artist', 'Unknown')}")
```

---

## Convenience Functions

### parse()

```python
def parse(xml: Union[str, bytes], **kwargs) -> ParseResult
```

Convenience function for quick parsing without creating a parser instance.

**Parameters:**
- `xml: Union[str, bytes]` - DDEX XML content
- `**kwargs` - Keyword arguments passed to `ParseOptions`

**Returns:** `ParseResult` - Parsed DDEX message structure

**Example:**
```python
from ddex_parser import parse

# Quick parse with default options
result = parse(xml_content)

# Quick parse with custom options
result = parse(
    xml_content,
    include_raw_extensions=True,
    validate_references=False
)
```

---

## DataFrame Integration

### Schema Types

When using `to_dataframe()`, you can specify different output schemas:

#### Flat Schema (Default)

The flat schema provides a denormalized, analysis-friendly structure:

```python
df = parser.to_dataframe(xml_content, schema='flat')
```

**Common Columns:**
- `message_id` - Message identifier
- `release_id` - Release identifier
- `sound_recording_id` - Track identifier
- `isrc` - International Standard Recording Code
- `title` - Track or release title
- `display_artist` - Primary artist name
- `label_name` - Record label
- `release_date` - Release date (ISO format)
- `genre` - Musical genre
- `territory` - Geographic territory
- `deal_type` - Commercial deal type
- `distribution_channel` - Distribution method

#### Graph Schema

The graph schema preserves the hierarchical DDEX structure:

```python
df = parser.to_dataframe(xml_content, schema='graph')
```

This schema maintains the original XML hierarchy and is useful for round-trip processing.

### DataFrame Examples

```python
import pandas as pd
from ddex_parser import DDEXParser

parser = DDEXParser()

# Load and convert to DataFrame
with open('catalog.xml', 'r') as f:
    xml_content = f.read()

df = parser.to_dataframe(xml_content)

# Basic analysis
print(f"Total releases: {df['release_id'].nunique()}")
print(f"Total tracks: {df['sound_recording_id'].nunique()}")

# Genre analysis
genre_dist = df['genre'].value_counts()
print("Top genres:")
print(genre_dist.head())

# Artist analysis
artist_track_counts = df.groupby('display_artist')['sound_recording_id'].nunique()
top_artists = artist_track_counts.sort_values(ascending=False).head(10)
print("Most prolific artists:")
print(top_artists)

# Territory analysis
territory_releases = df.groupby('territory')['release_id'].nunique()
print("Releases by territory:")
print(territory_releases)

# Date analysis
df['release_year'] = pd.to_datetime(df['release_date']).dt.year
yearly_releases = df.groupby('release_year')['release_id'].nunique()
print("Releases by year:")
print(yearly_releases)
```

---

## Error Handling

The Python API raises standard Python exceptions for various error conditions:

```python
from ddex_parser import DDEXParser

parser = DDEXParser()

try:
    result = parser.parse(xml_content)
except ValueError as e:
    if "Invalid XML" in str(e):
        print(f"XML parsing failed: {e}")
    elif "Unsupported version" in str(e):
        print(f"Unsupported DDEX version: {e}")
    else:
        print(f"Validation error: {e}")
except TimeoutError as e:
    print(f"Parsing timeout: {e}")
except MemoryError as e:
    print(f"Out of memory: {e}")
except ImportError as e:
    print(f"Missing dependency: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

### Common Exception Types

- **`ValueError`**: Invalid XML, schema violations, or unsupported DDEX versions
- **`TimeoutError`**: Parsing exceeded specified timeout
- **`MemoryError`**: File too large for available memory
- **`ImportError`**: Missing optional dependencies (e.g., pandas for `to_dataframe()`)
- **`FileNotFoundError`**: File path issues when reading from disk

---

## Performance Optimization

### Memory Management

```python
# Use streaming for large files
parser = DDEXParser()
options = ParseOptions(streaming=True, timeout=300.0)

# Process without reference validation for speed
fast_options = ParseOptions(
    validate_references=False,
    include_raw_extensions=False
)
```

### Async Processing

```python
import asyncio
from ddex_parser import DDEXParser

async def process_catalog_files(file_paths):
    parser = DDEXParser()
    semaphore = asyncio.Semaphore(5)  # Limit concurrent operations
    
    async def process_file(file_path):
        async with semaphore:
            with open(file_path, 'r') as f:
                xml_content = f.read()
            return await parser.parse_async(xml_content)
    
    tasks = [process_file(path) for path in file_paths]
    results = await asyncio.gather(*tasks, return_exceptions=True)
    
    # Handle results and exceptions
    successful = [r for r in results if isinstance(r, ParseResult)]
    errors = [r for r in results if isinstance(r, Exception)]
    
    return successful, errors
```

### Batch DataFrame Processing

```python
import pandas as pd
from pathlib import Path
from ddex_parser import DDEXParser

def process_catalog_to_dataframe(directory_path):
    parser = DDEXParser()
    dataframes = []
    
    xml_files = Path(directory_path).glob("*.xml")
    
    for xml_file in xml_files:
        try:
            with open(xml_file, 'r') as f:
                xml_content = f.read()
            
            df = parser.to_dataframe(xml_content)
            df['source_file'] = xml_file.name
            dataframes.append(df)
            
        except Exception as e:
            print(f"Failed to process {xml_file}: {e}")
    
    if dataframes:
        combined_df = pd.concat(dataframes, ignore_index=True)
        return combined_df
    else:
        return pd.DataFrame()

# Usage
catalog_df = process_catalog_to_dataframe('./ddex_files/')
print(f"Combined catalog: {len(catalog_df)} records from {catalog_df['source_file'].nunique()} files")
```

---

## CLI Integration

The Python package includes a command-line interface:

```bash
# Parse a single file
python -m ddex_parser parse input.xml

# Convert to JSON
python -m ddex_parser parse input.xml --output output.json

# Convert to CSV (requires pandas)
python -m ddex_parser to-csv input.xml output.csv

# Validate DDEX file
python -m ddex_parser validate input.xml

# Detect version
python -m ddex_parser version input.xml
```

Access the CLI programmatically:

```python
from ddex_parser.cli import main
import sys

# Simulate command line arguments
sys.argv = ['ddex_parser', 'parse', 'input.xml', '--output', 'output.json']
main()
```