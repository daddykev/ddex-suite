---
sidebar_position: 3
---

# Quick Start

Get up and running with DDEX Parser in minutes. This guide walks you through parsing your first DDEX file and accessing the structured data.

## Your First Parse

### JavaScript / TypeScript

```typescript
import { DDEXParser } from 'ddex-parser';

// Create a parser instance
const parser = new DDEXParser();

// Parse from file
const result = await parser.parseFile('path/to/release.xml');

// Parse from string
const xmlString = `<?xml version="1.0"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">
  <!-- Your DDEX content -->
</NewReleaseMessage>`;
const result = await parser.parseString(xmlString);

console.log('🎵 Release:', result.flat.releases[0]?.title);
console.log('🎤 Artist:', result.flat.releases[0]?.displayArtist);
console.log('📀 Tracks:', result.flat.soundRecordings.length);
```

### Python

```python
from ddex_parser import DDEXParser

# Create parser instance
parser = DDEXParser()

# Parse from file
result = parser.parse_file('path/to/release.xml')

# Parse from string
xml_content = """<?xml version="1.0"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">
  <!-- Your DDEX content -->
</NewReleaseMessage>"""
result = parser.parse(xml_content)

print(f"🎵 Release: {result.releases[0].title if result.releases else 'N/A'}")
print(f"🎤 Artist: {result.releases[0].display_artist if result.releases else 'N/A'}")
print(f"📀 Tracks: {len(result.sound_recordings)}")
```

## Understanding the Results

The parser returns structured data in two complementary formats:

### Flattened Model (Developer-Friendly)

The `flat` representation provides easy access to common fields:

```typescript
// Access release information
const release = result.flat.releases[0];
console.log('Title:', release.title);
console.log('Artist:', release.displayArtist);
console.log('Release Date:', release.releaseDate);
console.log('Label:', release.label);
console.log('Territories:', release.territories);

// Access track information
result.flat.soundRecordings.forEach(track => {
  console.log(`Track: ${track.title} by ${track.displayArtist}`);
  console.log(`Duration: ${track.duration}s`);
  console.log(`ISRC: ${track.isrc}`);
});

// Access deal information
result.flat.deals.forEach(deal => {
  console.log('Deal Type:', deal.commercialModelType);
  console.log('Territories:', deal.territories);
  console.log('Use Types:', deal.useTypes);
  console.log('Start Date:', deal.validityPeriod.startDate);
});
```

### Graph Model (DDEX-Faithful)

The `graph` representation maintains the original DDEX structure:

```typescript
// Message header information
const header = result.graph.messageHeader;
console.log('Message ID:', header.messageId);
console.log('Sent by:', header.messageRecipient);
console.log('Created:', header.messageCreatedDateTime);

// Party information
result.graph.partyList.party.forEach(party => {
  console.log('Party:', party.partyName);
  console.log('Role:', party.partyType);
});

// Resource references
result.graph.resourceList.soundRecording.forEach(recording => {
  console.log('Resource ID:', recording.resourceReference);
  console.log('Technical Details:', recording.technicalSoundRecordingDetails);
});
```

## Common Parsing Patterns

### Extract Release Metadata

```typescript
import { DDEXParser } from 'ddex-parser';

async function extractReleaseInfo(xmlFile: string) {
  const parser = new DDEXParser();
  const result = await parser.parseFile(xmlFile);
  
  const release = result.flat.releases[0];
  if (!release) {
    throw new Error('No release found in DDEX file');
  }
  
  return {
    title: release.title,
    artist: release.displayArtist,
    label: release.label,
    releaseDate: release.releaseDate,
    upc: release.releaseId.find(id => id.namespace === 'UPC')?.value,
    trackCount: result.flat.soundRecordings.length,
    territories: release.territories,
    genres: release.genres
  };
}

// Usage
const info = await extractReleaseInfo('album.xml');
console.log(JSON.stringify(info, null, 2));
```

### Process Multiple Files

```typescript
import { DDEXParser } from 'ddex-parser';
import { readdir } from 'fs/promises';
import path from 'path';

async function processDirectory(directoryPath: string) {
  const parser = new DDEXParser();
  const files = await readdir(directoryPath);
  const results = [];
  
  for (const file of files.filter(f => f.endsWith('.xml'))) {
    try {
      const filePath = path.join(directoryPath, file);
      const result = await parser.parseFile(filePath);
      
      results.push({
        file: file,
        messageId: result.graph.messageHeader.messageId,
        releases: result.flat.releases.length,
        tracks: result.flat.soundRecordings.length
      });
      
      console.log(`✅ Processed ${file}`);
    } catch (error) {
      console.error(`❌ Failed to process ${file}:`, error.message);
    }
  }
  
  return results;
}

// Usage
const summary = await processDirectory('./ddex-files/');
console.table(summary);
```

### Extract Commercial Terms

```typescript
async function extractDeals(xmlFile: string) {
  const parser = new DDEXParser();
  const result = await parser.parseFile(xmlFile);
  
  return result.flat.deals.map(deal => ({
    dealType: deal.commercialModelType,
    territories: deal.territories,
    useTypes: deal.useTypes,
    validFrom: deal.validityPeriod.startDate,
    validUntil: deal.validityPeriod.endDate,
    priceInformation: deal.priceInformation
  }));
}

// Usage
const deals = await extractDeals('distribution-deal.xml');
deals.forEach(deal => {
  console.log(`${deal.dealType} deal for ${deal.territories.join(', ')}`);
  console.log(`Valid: ${deal.validFrom} to ${deal.validUntil}`);
});
```

## Python-Specific Examples

### DataFrame Integration

```python
from ddex_parser import DDEXParser
import pandas as pd

# Parse directly to DataFrame
parser = DDEXParser()
df = parser.to_dataframe('catalog.xml')

# Analyze the data
print("=== Release Analysis ===")
print(df.groupby('label')['track_count'].sum().sort_values(ascending=False))

print("\n=== Genre Distribution ===")
genre_df = df.explode('genres').groupby('genres').size()
print(genre_df.head(10))

print("\n=== Territory Coverage ===")
territory_df = df.explode('territories').groupby('territories').size()
print(territory_df.sort_values(ascending=False).head(10))

# Export for further analysis
df.to_csv('ddex_analysis.csv', index=False)
df.to_excel('ddex_analysis.xlsx', index=False)
```

### Async Processing

```python
import asyncio
from ddex_parser import DDEXParser
import aiofiles
from pathlib import Path

async def process_file_async(parser, file_path):
    """Process a single DDEX file asynchronously."""
    try:
        async with aiofiles.open(file_path, 'r', encoding='utf-8') as f:
            content = await f.read()
        
        result = await parser.parse_async(content)
        
        return {
            'file': file_path.name,
            'success': True,
            'message_id': result.message_id,
            'releases': len(result.releases),
            'tracks': len(result.sound_recordings)
        }
    except Exception as e:
        return {
            'file': file_path.name,
            'success': False,
            'error': str(e)
        }

async def process_directory_async(directory_path):
    """Process all DDEX files in a directory concurrently."""
    parser = DDEXParser()
    directory = Path(directory_path)
    xml_files = list(directory.glob('*.xml'))
    
    # Process files concurrently
    tasks = [process_file_async(parser, file) for file in xml_files]
    results = await asyncio.gather(*tasks)
    
    # Summarize results
    successful = [r for r in results if r['success']]
    failed = [r for r in results if not r['success']]
    
    print(f"✅ Successfully processed: {len(successful)} files")
    print(f"❌ Failed to process: {len(failed)} files")
    
    if failed:
        print("\nFailures:")
        for failure in failed:
            print(f"  {failure['file']}: {failure['error']}")
    
    return results

# Usage
results = asyncio.run(process_directory_async('./ddex-files/'))
```

### Data Export Patterns

```python
from ddex_parser import DDEXParser
import json
from pathlib import Path

def export_to_json(xml_file, output_file=None):
    """Export DDEX data to JSON format."""
    parser = DDEXParser()
    result = parser.parse_file(xml_file)
    
    # Create export data structure
    export_data = {
        'metadata': {
            'message_id': result.message_id,
            'version': result.version,
            'processed_at': str(datetime.now())
        },
        'releases': [
            {
                'title': r.title,
                'artist': r.display_artist,
                'label': r.label,
                'release_date': r.release_date,
                'territories': r.territories,
                'genres': r.genres
            }
            for r in result.releases
        ],
        'tracks': [
            {
                'title': t.title,
                'artist': t.display_artist,
                'duration': t.duration,
                'isrc': t.isrc
            }
            for t in result.sound_recordings
        ]
    }
    
    # Save to file
    output_path = output_file or f"{Path(xml_file).stem}.json"
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(export_data, f, indent=2, ensure_ascii=False)
    
    print(f"Exported to {output_path}")
    return export_data

# Usage
data = export_to_json('release.xml', 'release_data.json')
```

## Error Handling

### Basic Error Handling

```typescript
import { DDEXParser, DDEXError, ValidationError } from 'ddex-parser';

async function safeParseFile(filePath: string) {
  const parser = new DDEXParser();
  
  try {
    const result = await parser.parseFile(filePath);
    return { success: true, data: result };
  } catch (error) {
    if (error instanceof ValidationError) {
      return {
        success: false,
        type: 'validation',
        message: error.message,
        details: error.details
      };
    } else if (error instanceof DDEXError) {
      return {
        success: false,
        type: 'parsing',
        message: error.message,
        line: error.line,
        column: error.column
      };
    } else {
      return {
        success: false,
        type: 'unknown',
        message: error.message
      };
    }
  }
}

// Usage
const result = await safeParseFile('problematic.xml');
if (result.success) {
  console.log('Parsed successfully:', result.data.flat.releases.length, 'releases');
} else {
  console.error(`${result.type} error:`, result.message);
}
```

### Validation and Preprocessing

```typescript
import { DDEXParser } from 'ddex-parser';
import { readFile } from 'fs/promises';

async function validateAndParse(filePath: string) {
  const parser = new DDEXParser();
  
  // Read and preprocess
  let xmlContent = await readFile(filePath, 'utf-8');
  
  // Basic validation
  if (!xmlContent.includes('NewReleaseMessage')) {
    throw new Error('Not a valid DDEX NewReleaseMessage');
  }
  
  // Detect version
  const version = await parser.detectVersion(xmlContent);
  console.log(`Detected DDEX version: ${version}`);
  
  if (!['3.8.2', '4.2', '4.3'].includes(version)) {
    console.warn(`Unsupported version: ${version}, parsing may fail`);
  }
  
  // Parse with validation
  const result = await parser.parseString(xmlContent, {
    validation: 'strict',
    includeWarnings: true
  });
  
  return result;
}

// Usage
try {
  const result = await validateAndParse('release.xml');
  console.log('✅ Valid DDEX file parsed successfully');
} catch (error) {
  console.error('❌ Validation failed:', error.message);
}
```

## Performance Tips

### For Small Files (< 1MB)

```typescript
const parser = new DDEXParser({
  streaming: false,      // Faster for small files
  validation: 'basic',   // Skip heavy validation
  caching: true         // Cache parsed schemas
});
```

### For Large Files (> 10MB)

```typescript
const parser = new DDEXParser({
  streaming: true,       // Essential for large files
  bufferSize: 8192,     // Larger buffer for better I/O
  maxMemoryMB: 100      // Prevent memory issues
});

// Process in chunks
for await (const batch of parser.streamFile('large-catalog.xml')) {
  await processBatch(batch);
}
```

## Next Steps

Now that you can parse DDEX files, explore more advanced features:

- **[API Reference](./api-reference)** - Complete method documentation
- **[Advanced Usage](./advanced-usage)** - Streaming, performance optimization
- **[Examples](../examples/)** - Real-world integration patterns
- **[Builder Integration](../builder/)** - Generate DDEX files from parsed data