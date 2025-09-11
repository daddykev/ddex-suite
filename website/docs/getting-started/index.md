---
sidebar_position: 2
---

# Getting Started

Get up and running with DDEX Suite in minutes. This guide will walk you through installation, basic usage, and your first complete workflow.

## Prerequisites

- **Node.js**: Version 16 or higher (for JavaScript/TypeScript)
- **Python**: Version 3.8 or higher (for Python bindings)
- Basic understanding of DDEX metadata (helpful but not required)

## Installation

Choose your preferred language and package manager:

### Node.js / TypeScript

```bash
# Install both parser and builder
npm install ddex-parser ddex-builder

# Or install individually
npm install ddex-parser
npm install ddex-builder
```

### Python

```bash
# Install both parser and builder
pip install ddex-parser ddex-builder

# Or install individually  
pip install ddex-parser
pip install ddex-builder
```

### Browser (WebAssembly)

```html
<!-- Load from CDN -->
<script src="https://unpkg.com/ddex-parser/dist/ddex_parser.js"></script>
<script src="https://unpkg.com/ddex-builder/dist/ddex_builder.js"></script>
```

## Your First DDEX Workflow

Let's walk through a complete example: parsing DDEX XML, modifying the data, and building it back to XML.

### Step 1: Parse DDEX XML

```typescript
import { DDEXParser } from 'ddex-parser';
import fs from 'fs';

// Load your DDEX file
const xmlContent = fs.readFileSync('path/to/your/ddex-file.xml', 'utf8');

// Create parser instance
const parser = new DDEXParser();

// Parse the XML
const result = await parser.parse(xmlContent);

// Access the structured data
console.log('Release title:', result.flat.releases[0].title);
console.log('Artist name:', result.flat.soundRecordings[0].artist);
console.log('Number of tracks:', result.flat.soundRecordings.length);
```

### Step 2: Modify the Data

```typescript
// Make changes to the parsed data
result.flat.releases[0].title = "My Updated Release Title";
result.flat.releases[0].releaseDate = "2024-01-01";

// Add new territories
result.flat.deals[0].territories.push("US", "CA", "GB");
```

### Step 3: Build Back to XML

```typescript
import { DDEXBuilder } from 'ddex-builder';

// Create builder instance
const builder = new DDEXBuilder();

// Convert back to XML
const newXml = await builder.build(result.toBuildRequest());

// Save the result
fs.writeFileSync('output/modified-ddex.xml', newXml);
```

## Python Example

Here's the same workflow in Python:

```python
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

# Parse DDEX XML
parser = DDEXParser()
with open('path/to/your/ddex-file.xml', 'r') as f:
    xml_content = f.read()

result = parser.parse(xml_content)

# Access structured data
print(f"Release title: {result.flat.releases[0].title}")
print(f"Artist: {result.flat.sound_recordings[0].artist}")

# Modify the data
result.flat.releases[0].title = "My Updated Release Title"

# Build back to XML
builder = DDEXBuilder()
new_xml = builder.build(result.to_build_request())

# Save the result
with open('output/modified-ddex.xml', 'w') as f:
    f.write(new_xml)
```

## Working with DataFrames (Python)

DDEX Suite provides seamless integration with pandas DataFrames:

```python
import pandas as pd
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

# Parse to DataFrame
parser = DDEXParser()
df = parser.to_dataframe('path/to/ddex-file.xml')

# Analyze with pandas
print(df.releases.head())
print(df.sound_recordings.groupby('artist').count())

# Modify the DataFrame
df.releases.loc[0, 'title'] = 'Updated Title'

# Build from DataFrame
builder = DDEXBuilder()
xml = builder.from_dataframe(df, version='4.3')
```

## Next Steps

Now that you have the basics down, explore these topics:

- **[Parser Documentation](../parser/)** - Deep dive into parsing features
- **[Builder Documentation](../builder/)** - Learn advanced building techniques  
- **[API Reference](../api/)** - Complete API documentation
- **[Examples](../examples/)** - Real-world usage examples
- **[Guides](../guides/)** - How-to guides for common tasks

## Need Help?

- 📖 Browse the [full documentation](/)
- 🐛 [Report issues](https://github.com/daddykev/ddex-suite/issues)
- 💬 [Ask questions](https://github.com/daddykev/ddex-suite/discussions)
- 🚀 Try the [interactive playground](/playground)