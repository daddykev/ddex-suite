# Basic Parsing

Learn the fundamentals of parsing DDEX XML files with DDEX Suite.

## Quick Start

The simplest way to parse a DDEX XML file:

```typescript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();
const result = await parser.parse(xmlContent);

console.log('Release title:', result.flat.releases[0]?.title);
console.log('Artist:', result.flat.releases[0]?.artist);
```

## Understanding the Parse Result

DDEX Suite provides two representations of your data:

### Flattened Representation (`result.flat`)

Simplified, developer-friendly format perfect for most applications:

```typescript
const flatData = result.flat;
console.log('Releases:', flatData.releases.length);

flatData.releases.forEach(release => {
  console.log(`Release: ${release.title} by ${release.artist}`);
  console.log(`UPC: ${release.upc}`);
  console.log(`Tracks: ${release.tracks.length}`);
  
  release.tracks.forEach(track => {
    console.log(`  - ${track.title} (${track.duration})`);
  });
});
```

### Graph Representation (`result.graph`)

Preserves the exact DDEX XML structure for advanced use cases:

```typescript
const graphData = result.graph;
console.log('Message ID:', graphData.messageHeader.messageId);
console.log('Parties:', graphData.parties.length);
console.log('Resources:', graphData.resources.length);
console.log('Deals:', graphData.deals.length);
```

## Parsing Options

Customize parsing behavior with options:

```typescript
const result = await parser.parse(xmlContent, {
  includeRawExtensions: true,     // Preserve unknown XML elements
  includeComments: false,         // Strip XML comments
  validateReferences: true,       // Validate internal references
  streaming: false                // Use streaming for large files
});
```

## File Input Methods

### From File System (Node.js)

```typescript
import { readFileSync } from 'fs';

const xmlContent = readFileSync('release.xml', 'utf-8');
const result = await parser.parse(xmlContent);
```

### From URL

```typescript
const response = await fetch('https://example.com/release.xml');
const xmlContent = await response.text();
const result = await parser.parse(xmlContent);
```

### From Buffer

```typescript
const buffer = Buffer.from(xmlContent, 'utf-8');
const result = await parser.parse(buffer.toString());
```

## Error Handling

Always wrap parsing in try-catch blocks:

```typescript
try {
  const result = await parser.parse(xmlContent);
  console.log('Parsed successfully!');
} catch (error) {
  if (error.code === 'INVALID_XML') {
    console.error('Invalid XML format:', error.message);
  } else if (error.code === 'DDEX_VALIDATION_ERROR') {
    console.error('DDEX validation failed:', error.details);
  } else {
    console.error('Parsing failed:', error.message);
  }
}
```

## Version Detection

DDEX Suite automatically detects the DDEX version:

```typescript
const result = await parser.parse(xmlContent);
console.log('DDEX Version:', result.version); // e.g., "4.3", "4.2", "3.8.2"

// Access version-specific fields
if (result.version === '4.3') {
  // Use ERN 4.3 specific features
}
```

## Performance Tips

### For Small Files (< 10MB)
Use the default synchronous parsing - it's the fastest option.

### For Medium Files (10-100MB)
Consider using streaming if memory is limited:

```typescript
const result = await parser.parse(xmlContent, { streaming: true });
```

### For Large Files (> 100MB)
Always use streaming mode - see the [Large File Processing](./large-files) guide.

## Python Usage

```python
from ddex_parser import DDEXParser

parser = DDEXParser()
result = parser.parse(xml_content)

print(f"Release: {result.flat.releases[0].title}")
print(f"Artist: {result.flat.releases[0].artist}")

# Access graph data
print(f"Message ID: {result.graph.message_header.message_id}")
```

## Next Steps

- [Large File Processing](./large-files) - Handle files over 100MB efficiently
- [Error Handling](./error-handling) - Robust error handling patterns
- [Performance Optimization](./performance) - Optimize for your specific use case
- [DataFrame Integration](./dataframes) - Analytics with pandas integration

## Common Issues

**XML Parse Errors**: Ensure your XML is well-formed and uses UTF-8 encoding.

**Memory Issues**: For large files, use streaming mode or see the [Large File Processing](./large-files) guide.

**Missing Data**: Check if you need to enable `includeRawExtensions` for custom fields.

**Reference Errors**: Use `validateReferences: false` if dealing with incomplete test data.