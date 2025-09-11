---
sidebar_position: 3
---

# DDEX Parser

The DDEX Parser transforms DDEX XML messages into clean, structured data that's easy to work with in your applications. It handles all the complexity of XML parsing while preserving perfect fidelity for round-trip operations.

## Key Features

- **Fast**: Parse 10KB files in &lt;5ms, 100MB files in &lt;5s
- **Memory Efficient**: Stream large files with minimal memory usage
- **Type Safe**: Full TypeScript definitions and Python type hints
- **Faithful**: Preserves all DDEX extensions and custom fields
- **Flexible**: Both graph and flattened data representations

## Basic Usage

### Node.js / TypeScript

```typescript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();
const result = await parser.parse(xmlContent);

// Access flattened, developer-friendly data
console.log(result.flat.releases[0].title);
console.log(result.flat.soundRecordings[0].artist);

// Or access the original graph structure
console.log(result.graph.messageHeader.messageId);
```

### Python

```python
from ddex_parser import DDEXParser

parser = DDEXParser()
result = parser.parse(xml_content)

# Access structured data
print(result.flat.releases[0].title)
print(result.flat.sound_recordings[0].artist)
```

## Data Representations

The parser provides two complementary views of your DDEX data:

### Flattened Representation

The `flat` representation is optimized for developer productivity:

```typescript
// Easy access to common fields
result.flat.releases[0].title
result.flat.releases[0].artist
result.flat.releases[0].releaseDate
result.flat.releases[0].territories

// Sound recordings are flattened
result.flat.soundRecordings[0].title
result.flat.soundRecordings[0].artist
result.flat.soundRecordings[0].duration

// Deals and commercial terms
result.flat.deals[0].territories
result.flat.deals[0].useTypes
result.flat.deals[0].commercialModelType
```

### Graph Representation

The `graph` representation maintains the original DDEX structure:

```typescript
// Access original XML structure
result.graph.messageHeader.messageId
result.graph.messageHeader.sentOnBehalfOf
result.graph.newReleaseMessage.releaseList.release[0]
result.graph.newReleaseMessage.dealList.releaseDeal[0]
```

## Configuration Options

### Parser Settings

```typescript
const parser = new DDEXParser({
  // Validation level
  validation: 'strict', // 'strict' | 'permissive' | 'none'
  
  // Memory limits for large files
  maxMemoryMb: 100,
  
  // XML entity limits (security)
  maxEntityExpansions: 1000,
  maxNestingDepth: 50,
  
  // Preserve formatting
  preserveWhitespace: false,
});
```

### Performance Tuning

```typescript
// For large files, use streaming
const parser = new DDEXParser({
  streaming: true,
  bufferSizeMb: 10,
});

// For maximum speed on small files
const parser = new DDEXParser({
  streaming: false,
  validation: 'none',
});
```

## Error Handling

The parser provides detailed error information:

```typescript
try {
  const result = await parser.parse(xmlContent);
} catch (error) {
  if (error instanceof DDEXParseError) {
    console.log('Parse error:', error.message);
    console.log('Line:', error.line);
    console.log('Column:', error.column);
    console.log('Context:', error.context);
  }
}
```

### Common Error Types

- **`InvalidXmlError`**: Malformed XML structure
- **`ValidationError`**: DDEX schema validation failed  
- **`UnsupportedVersionError`**: DDEX version not supported
- **`SecurityError`**: XML security limits exceeded

## DataFrame Integration (Python)

Convert DDEX data to pandas DataFrames for analysis:

```python
import pandas as pd
from ddex_parser import DDEXParser

parser = DDEXParser()

# Parse directly to DataFrames
dfs = parser.to_dataframe('path/to/ddex-file.xml')

# Access structured tables
print(dfs.releases.head())
print(dfs.sound_recordings.head())
print(dfs.deals.head())

# Analyze with pandas
artist_counts = dfs.sound_recordings.groupby('artist').size()
territory_coverage = dfs.deals.explode('territories').groupby('territories').size()
```

## Supported DDEX Versions

| Version | Status | Notes |
|---------|--------|-------|
| ERN 3.8.2 | ✅ Full | Legacy support |
| ERN 4.2 | ✅ Full | Stable version |
| ERN 4.3 | ✅ Full | Latest version |

## Performance Benchmarks

| File Size | Parse Time | Memory Usage |
|-----------|------------|--------------|
| 10KB | &lt;5ms | &lt;1MB |
| 100KB | &lt;10ms | &lt;5MB |
| 1MB | &lt;50ms | &lt;10MB |
| 10MB | &lt;500ms | &lt;50MB |
| 100MB | &lt;5s | &lt;100MB |

## CLI Usage

The parser also provides a command-line interface:

```bash
# Parse and output JSON
ddex-parser parse input.xml --output json

# Validate DDEX file
ddex-parser validate input.xml

# Extract specific fields
ddex-parser extract input.xml --field releases.title

# Performance analysis
ddex-parser analyze input.xml --stats
```

## Next Steps

- **[Builder Documentation](../builder/)** - Learn to generate DDEX XML
- **[API Reference](../api/)** - Complete parser API
- **[Examples](../examples/)** - Real-world parser examples
- **[Guides](../guides/)** - How-to guides for common tasks