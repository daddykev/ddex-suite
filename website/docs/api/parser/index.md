# DDEX Parser API Reference

Complete API documentation for the DDEX Parser across all supported languages.

## Overview

The DDEX Parser provides high-performance parsing of DDEX XML files with native Rust implementations and bindings for JavaScript/TypeScript and Python. The parser supports both synchronous and streaming modes for handling files of any size.

## Language Bindings

- [**Rust API**](./rust) - Native Rust API with full type safety and zero-cost abstractions
- [**JavaScript/TypeScript API**](./typescript) - Complete Node.js and browser API reference
- [**Python API**](./python) - Python bindings with pandas integration
- [**Core Types**](./types) - Shared type definitions and interfaces

## Quick Navigation

| Component | Description |
|-----------|-------------|
| [`DDEXParser`](./typescript#ddexparser) | Main parser class for JavaScript/TypeScript |
| [`DdexParser`](./typescript#ddexparser-native) | Native Rust binding class |
| [`ParseOptions`](./types#parseoptions) | Configuration options for parsing |
| [`ParseResult`](./types#parseresult) | Parsed DDEX message structure |
| [`Stream API`](./typescript#streaming) | Streaming parser for large files |

## Key Features

- **Multiple Output Formats**: Graph (faithful) and flat (developer-friendly) representations
- **Streaming Support**: Memory-efficient processing of large DDEX catalogs
- **Version Detection**: Automatic detection of ERN 3.8.2, 4.2, and 4.3
- **Extension Preservation**: Maintains custom extensions for round-trip fidelity
- **Performance Optimized**: Native Rust implementation with minimal overhead
- **Type Safety**: Complete TypeScript definitions and Python type hints

## Performance Characteristics

| File Size | Parse Time | Memory Usage |
|-----------|------------|--------------|
| 10KB | &lt;5ms | &lt;1MB |
| 100KB | &lt;10ms | &lt;5MB |
| 1MB | &lt;50ms | &lt;20MB |
| 100MB | &lt;5s | &lt;100MB |

## Common Usage Patterns

```typescript
import { DDEXParser } from 'ddex-parser';

// Basic parsing
const parser = new DDEXParser();
const result = await parser.parse(xmlContent);

// Stream large files
const stream = parser.stream(largeXmlContent);
for await (const release of stream) {
  console.log(release.title);
}
```

```python
from ddex_parser import DDEXParser

# Basic parsing
parser = DDEXParser()
result = parser.parse(xml_content)

# Convert to DataFrame
df = parser.to_dataframe(xml_content)
```

```rust
use ddex_parser::DDEXParser;

// Basic parsing with full type safety
let parser = DDEXParser::new();
let result = parser.parse(&xml_content)?;

// Stream large files efficiently
let mut stream = parser.stream_file("large-catalog.xml").await?;
while let Some(batch) = stream.next().await {
    for release in batch? {
        println!("Processing: {}", release.title);
    }
}
```