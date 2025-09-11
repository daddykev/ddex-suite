---
sidebar_position: 4
---

# API Reference

Complete reference documentation for the DDEX Parser API across all supported languages.

## DDEXParser Class

The main parser class provides methods for parsing DDEX XML files and strings.

### Constructor

#### JavaScript / TypeScript

```typescript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser(options?: ParseOptions);
```

#### Python

```python
from ddex_parser import DDEXParser

parser = DDEXParser()
```

### Configuration Options

#### ParseOptions (TypeScript)

```typescript
interface ParseOptions {
  // Validation level
  validation?: 'none' | 'basic' | 'strict';
  
  // Security settings
  maxMemoryMB?: number;
  maxEntityExpansions?: number;
  maxNestingDepth?: number;
  timeoutSeconds?: number;
  
  // Processing options
  streaming?: boolean;
  bufferSize?: number;
  includeComments?: boolean;
  includeRawExtensions?: boolean;
  preserveWhitespace?: boolean;
  
  // Performance options
  caching?: boolean;
  parallelProcessing?: boolean;
}
```

#### ParseOptions (Python)

```python
from ddex_parser import ParseOptions

options = ParseOptions(
    validation='strict',         # 'none' | 'basic' | 'strict'
    max_memory_mb=100,          # Memory limit in MB
    max_entity_expansions=1000, # Entity expansion limit
    max_nesting_depth=50,       # XML nesting depth limit
    timeout_seconds=30.0,       # Parse timeout
    streaming=False,            # Enable streaming mode
    include_comments=False,     # Preserve XML comments
    include_raw_extensions=True, # Preserve DDEX extensions
    validate_references=True    # Validate internal references
)
```

## Core Parsing Methods

### parseFile() / parse_file()

Parse a DDEX file from disk.

#### JavaScript / TypeScript

```typescript
async parseFile(filePath: string, options?: ParseOptions): Promise<ParseResult>
```

**Parameters:**
- `filePath`: Path to the DDEX XML file
- `options`: Optional parsing configuration

**Returns:** Promise resolving to ParseResult

**Example:**
```typescript
const result = await parser.parseFile('/path/to/release.xml', {
  validation: 'strict',
  includeRawExtensions: true
});
```

#### Python

```python
def parse_file(self, file_path: str, options: Optional[ParseOptions] = None) -> ParseResult
```

**Parameters:**
- `file_path`: Path to the DDEX XML file
- `options`: Optional parsing configuration

**Returns:** ParseResult object

**Example:**
```python
result = parser.parse_file('/path/to/release.xml', ParseOptions(validation='strict'))
```

### parseString() / parse()

Parse DDEX XML from a string or bytes.

#### JavaScript / TypeScript

```typescript
async parseString(xml: string, options?: ParseOptions): Promise<ParseResult>
```

**Parameters:**
- `xml`: DDEX XML content as string
- `options`: Optional parsing configuration

**Returns:** Promise resolving to ParseResult

#### Python

```python
def parse(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParseResult
```

**Parameters:**
- `xml`: DDEX XML content as string or bytes
- `options`: Optional parsing configuration

**Returns:** ParseResult object

### parseAsync() / parse_async()

Asynchronous parsing for non-blocking operation.

#### JavaScript / TypeScript

```typescript
// parseString() is already async in JavaScript
const result = await parser.parseString(xml);
```

#### Python

```python
async def parse_async(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParseResult
```

**Example:**
```python
import asyncio

async def parse_files():
    parser = DDEXParser()
    result = await parser.parse_async(xml_content)
    return result

result = asyncio.run(parse_files())
```

## Streaming Methods

### stream() / streamFile()

Stream parse large DDEX files to avoid memory issues.

#### JavaScript / TypeScript

```typescript
async *streamFile(filePath: string, options?: ParseOptions): AsyncGenerator<ParseBatch>

async *stream(source: string | ReadableStream, options?: ParseOptions): AsyncGenerator<ParseBatch>
```

**Example:**
```typescript
for await (const batch of parser.streamFile('large-catalog.xml')) {
  console.log(`Processing ${batch.releases.length} releases`);
  await processBatch(batch);
}
```

#### Python

```python
def stream(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> Iterator[Dict[str, Any]]
```

**Example:**
```python
for batch in parser.stream(large_xml_content):
    print(f"Processing batch with {len(batch.get('releases', []))} releases")
    process_batch(batch)
```

## Utility Methods

### detectVersion() / detect_version()

Automatically detect the DDEX version from XML content.

#### JavaScript / TypeScript

```typescript
async detectVersion(xml: string): Promise<string>
```

#### Python

```python
def detect_version(self, xml: Union[str, bytes]) -> str
```

**Returns:** DDEX version string ('3.8.2', '4.2', '4.3', or 'Unknown')

**Example:**
```typescript
const version = await parser.detectVersion(xmlContent);
console.log(`Detected version: ${version}`);
```

### validate() / sanity_check()

Validate DDEX XML without full parsing.

#### JavaScript / TypeScript

```typescript
async validate(xml: string, options?: ValidationOptions): Promise<ValidationResult>
```

#### Python

```python
def sanity_check(self, xml: Union[str, bytes]) -> Dict[str, Any]
```

**Returns:** Validation result with errors and warnings

**Example:**
```python
validation = parser.sanity_check(xml_content)
if validation['is_valid']:
    print("✅ Valid DDEX file")
else:
    print("❌ Validation errors:", validation['errors'])
```

## DataFrame Integration (Python Only)

### to_dataframe()

Convert DDEX XML directly to pandas DataFrames.

```python
def to_dataframe(self, xml: Union[str, bytes], schema: str = 'flat') -> 'pd.DataFrame'
```

**Parameters:**
- `xml`: DDEX XML content
- `schema`: Schema type ('flat' or 'graph')

**Returns:** pandas DataFrame with structured DDEX data

**Example:**
```python
import pandas as pd

# Parse to DataFrame
df = parser.to_dataframe(xml_content, schema='flat')

# Analyze data
print(df.columns)
print(df.head())

# Group by genre
genre_analysis = df.groupby('genre').size().sort_values(ascending=False)
```

## Data Structures

### ParseResult

The main result object returned by parsing operations.

#### JavaScript / TypeScript

```typescript
interface ParseResult {
  // Dual model representations
  graph: GraphModel;
  flat: FlattenedModel;
  
  // Metadata
  version: string;
  messageId: string;
  
  // Utility methods
  toJSON(): object;
  toBuildRequest(): BuildRequest; // For ddex-builder integration
}
```

#### Python

```python
class ParseResult:
    # Properties
    message_id: str
    version: str
    releases: List[Release]
    sound_recordings: List[SoundRecording]
    deals: List[Deal]
    
    # Methods
    def to_json(self) -> str
    def to_dict(self) -> Dict[str, Any]
    def to_build_request(self) -> Dict[str, Any]  # For ddex-builder integration
```

### Graph Model Structure

The graph model preserves the original DDEX XML structure:

```typescript
interface GraphModel {
  messageHeader: {
    messageId: string;
    messageCreatedDateTime: string;
    messageSender: PartyReference;
    messageRecipient: PartyReference;
    sentOnBehalfOf?: PartyReference;
  };
  
  partyList: {
    party: Party[];
  };
  
  resourceList: {
    soundRecording: SoundRecording[];
    image?: Image[];
    text?: Text[];
    video?: Video[];
  };
  
  releaseList: {
    release: Release[];
  };
  
  dealList: {
    releaseDeal: Deal[];
  };
  
  // Extension support
  extensions?: Record<string, any>;
}
```

### Flattened Model Structure

The flattened model provides developer-friendly access:

```typescript
interface FlattenedModel {
  // Release information
  releases: FlatRelease[];
  
  // Track information
  soundRecordings: FlatSoundRecording[];
  
  // Commercial terms
  deals: FlatDeal[];
  
  // Party information
  parties: FlatParty[];
  
  // Resources
  images: FlatImage[];
  videos: FlatVideo[];
}

interface FlatRelease {
  title: string;
  displayArtist: string;
  label: string;
  releaseDate: string;
  releaseId: ReleaseId[];
  territories: string[];
  genres: string[];
  tags: string[];
  pLine?: string;
  cLine?: string;
}

interface FlatSoundRecording {
  title: string;
  displayArtist: string;
  duration: number; // seconds
  isrc: string;
  territories: string[];
  genres: string[];
  contributors: Contributor[];
  technicalDetails: TechnicalDetails;
}

interface FlatDeal {
  commercialModelType: string;
  useTypes: string[];
  territories: string[];
  validityPeriod: {
    startDate: string;
    endDate?: string;
  };
  priceInformation?: PriceInformation;
  conditions?: string[];
}
```

## Error Types

### JavaScript / TypeScript Errors

```typescript
class DDEXError extends Error {
  line?: number;
  column?: number;
  context?: string;
}

class ValidationError extends DDEXError {
  details: ValidationDetail[];
}

class SecurityError extends DDEXError {
  violationType: 'xxe' | 'entity_expansion' | 'nesting_depth' | 'memory_limit';
}

class UnsupportedVersionError extends DDEXError {
  detectedVersion: string;
  supportedVersions: string[];
}
```

### Python Errors

```python
class DDEXParseError(Exception):
    """Base exception for DDEX parsing errors."""
    def __init__(self, message: str, line: int = None, column: int = None):
        super().__init__(message)
        self.line = line
        self.column = column

class ValidationError(DDEXParseError):
    """DDEX schema validation error."""
    def __init__(self, message: str, details: List[str] = None):
        super().__init__(message)
        self.details = details or []

class SecurityError(DDEXParseError):
    """XML security violation error."""
    pass
```

## Version Information

### JavaScript / TypeScript

```typescript
// Get parser version
console.log(parser.version);

// Check supported features
console.log(DDEXParser.supportedVersions);
console.log(DDEXParser.features);
```

### Python

```python
from ddex_parser import __version__

# Get parser version
print(f"DDEX Parser version: {__version__}")

# Instance version
print(f"Parser instance version: {parser.__version__}")
```

## Configuration Examples

### Maximum Performance (Small Files)

```typescript
const fastParser = new DDEXParser({
  validation: 'none',
  streaming: false,
  caching: true,
  parallelProcessing: true,
  includeComments: false,
  includeRawExtensions: false
});
```

### Maximum Security (Untrusted Input)

```typescript
const secureParser = new DDEXParser({
  validation: 'strict',
  maxMemoryMB: 50,
  maxEntityExpansions: 100,
  maxNestingDepth: 20,
  timeoutSeconds: 10,
  streaming: true
});
```

### Large File Processing

```typescript
const streamingParser = new DDEXParser({
  streaming: true,
  bufferSize: 16384,
  maxMemoryMB: 200,
  validation: 'basic',
  parallelProcessing: true
});
```

## Advanced Usage Patterns

### Batch Processing with Error Handling

```typescript
import { DDEXParser, DDEXError, ValidationError } from 'ddex-parser';

async function processBatch(files: string[]) {
  const parser = new DDEXParser({ validation: 'strict' });
  const results = [];
  
  for (const file of files) {
    try {
      const result = await parser.parseFile(file);
      results.push({
        file,
        success: true,
        messageId: result.messageId,
        releases: result.flat.releases.length
      });
    } catch (error) {
      let errorInfo = { file, success: false };
      
      if (error instanceof ValidationError) {
        errorInfo.error = 'validation';
        errorInfo.details = error.details;
      } else if (error instanceof DDEXError) {
        errorInfo.error = 'parsing';
        errorInfo.line = error.line;
        errorInfo.column = error.column;
      } else {
        errorInfo.error = 'unknown';
        errorInfo.message = error.message;
      }
      
      results.push(errorInfo);
    }
  }
  
  return results;
}
```

### Custom Extension Handling

```typescript
const parser = new DDEXParser({
  includeRawExtensions: true,
  validation: 'basic' // Allow non-standard extensions
});

const result = await parser.parseString(xmlWithExtensions);

// Access custom extensions
if (result.graph.extensions) {
  console.log('Custom extensions found:', Object.keys(result.graph.extensions));
}
```

### Memory Monitoring

```python
import psutil
from ddex_parser import DDEXParser

def parse_with_monitoring(xml_content):
    process = psutil.Process()
    initial_memory = process.memory_info().rss
    
    parser = DDEXParser()
    result = parser.parse(xml_content)
    
    final_memory = process.memory_info().rss
    memory_used = (final_memory - initial_memory) / 1024 / 1024  # MB
    
    print(f"Memory used for parsing: {memory_used:.2f} MB")
    return result
```

This comprehensive API reference covers all major features and methods available in the DDEX Parser. For more examples and advanced usage patterns, see the [Advanced Usage](./advanced-usage) guide.