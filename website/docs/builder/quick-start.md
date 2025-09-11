---
sidebar_position: 3
---

# Quick Start

Get up and running with DDEX Builder in minutes. This guide walks you through creating your first DDEX ERN messages and understanding the core building concepts.

## Your First Build

### JavaScript / TypeScript

```typescript
import { DdexBuilder } from 'ddex-builder';

// Create a builder instance
const builder = new DdexBuilder();

// Build a simple release
const releaseData = {
  messageHeader: {
    messageId: 'MSG_2024_001',
    messageSenderName: 'My Record Label',
    messageRecipientName: 'Music Platform',
    messageCreatedDateTime: new Date().toISOString()
  },
  releases: [{
    releaseId: 'REL_001',
    title: 'My Amazing Album',
    artist: 'Incredible Artist',
    releaseType: 'Album',
    label: 'My Record Label',
    upc: '123456789012',
    releaseDate: '2024-01-15',
    trackIds: ['TR_001', 'TR_002']
  }],
  resources: [{
    resourceId: 'TR_001',
    resourceType: 'SoundRecording',
    title: 'Hit Single',
    artist: 'Incredible Artist',
    isrc: 'US1234567890',
    duration: 'PT3M45S'
  }, {
    resourceId: 'TR_002', 
    resourceType: 'SoundRecording',
    title: 'Another Great Track',
    artist: 'Incredible Artist',
    isrc: 'US1234567891',
    duration: 'PT4M12S'
  }]
};

// Generate deterministic DDEX XML
const xml = await builder.build(releaseData);
console.log('🎵 Generated DDEX XML:', xml.length, 'bytes');

// The XML is always identical for the same input
const xml2 = await builder.build(releaseData);
console.assert(xml === xml2); // ✅ Always true
```

### Python

```python
from ddex_builder import DdexBuilder
from datetime import datetime

# Create builder instance  
builder = DdexBuilder()

# Build a simple release
release_data = {
    'message_header': {
        'message_id': 'MSG_2024_001',
        'message_sender_name': 'My Record Label',
        'message_recipient_name': 'Music Platform',
        'message_created_date_time': datetime.now().isoformat()
    },
    'releases': [{
        'release_id': 'REL_001',
        'title': 'My Amazing Album',
        'artist': 'Incredible Artist',
        'release_type': 'Album',
        'label': 'My Record Label',
        'upc': '123456789012',
        'release_date': '2024-01-15',
        'track_ids': ['TR_001', 'TR_002']
    }],
    'resources': [{
        'resource_id': 'TR_001',
        'resource_type': 'SoundRecording',
        'title': 'Hit Single',
        'artist': 'Incredible Artist',
        'isrc': 'US1234567890',
        'duration': 'PT3M45S'
    }, {
        'resource_id': 'TR_002',
        'resource_type': 'SoundRecording', 
        'title': 'Another Great Track',
        'artist': 'Incredible Artist',
        'isrc': 'US1234567891',
        'duration': 'PT4M12S'
    }]
}

# Generate deterministic DDEX XML
xml = builder.build(release_data)
print(f'🎵 Generated DDEX XML: {len(xml)} bytes')

# Verify deterministic output
xml2 = builder.build(release_data)
assert xml == xml2  # ✅ Always true
```

## Understanding the Build Process

### Core Data Structure

DDEX Builder expects a structured object with these main sections:

```typescript
interface BuildRequest {
  // Message metadata
  messageHeader: MessageHeader;
  
  // Release information (albums, singles, etc.)
  releases: Release[];
  
  // Sound recordings, images, videos, etc.  
  resources: Resource[];
  
  // Commercial terms and deals (optional)
  deals?: Deal[];
  
  // Party information (optional)
  parties?: Party[];
}
```

### Message Header

Every DDEX message needs a header with identification and routing:

```typescript
const messageHeader = {
  messageId: 'MSG_2024_001',           // Unique identifier
  messageSenderName: 'My Label',       // Who is sending
  messageRecipientName: 'Platform',    // Who receives
  messageCreatedDateTime: new Date().toISOString(), // When created
  messageControlType: 'LiveMessage'    // Message type
};
```

### Releases

Releases represent albums, singles, or other release units:

```typescript  
const release = {
  releaseId: 'REL_001',              // Unique identifier
  title: 'Album Title',              // Release name
  artist: 'Artist Name',             // Main artist
  releaseType: 'Album',              // Album, Single, EP, etc.
  label: 'Label Name',               // Record label
  upc: '123456789012',               // Universal Product Code
  releaseDate: '2024-01-15',         // Release date (YYYY-MM-DD)
  territories: ['WorldWide'],         // Where it's available
  genres: ['Pop', 'Rock'],           // Musical genres
  trackIds: ['TR_001', 'TR_002']     // References to resources
};
```

### Resources

Resources are the actual content (tracks, artwork, etc.):

```typescript
const soundRecording = {
  resourceId: 'TR_001',              // Unique identifier  
  resourceType: 'SoundRecording',     // Type of resource
  title: 'Track Title',              // Song name
  artist: 'Artist Name',             // Performer
  isrc: 'US1234567890',              // International Standard Recording Code
  duration: 'PT3M45S',               // Duration (ISO 8601)
  trackNumber: 1,                    // Position on album
  metadata: {                        // Additional metadata
    composer: 'Composer Name',
    publisher: 'Publisher Name'
  }
};
```

## Using Presets

Presets configure the builder for specific platforms and use cases:

### Platform-Specific Builds

```typescript
// Spotify preset - streaming platform requirements
const spotifyBuilder = new DdexBuilder();
spotifyBuilder.applyPreset('spotify');

const spotifyXml = await spotifyBuilder.build(releaseData);

// YouTube Music preset - Content ID requirements  
const youtubeBuilder = new DdexBuilder();
youtubeBuilder.applyPreset('youtube_music');

const youtubeXml = await youtubeBuilder.build(releaseData);

// Apple Music preset - iTunes Store compliance
const appleBuilder = new DdexBuilder();
appleBuilder.applyPreset('apple_music');

const appleXml = await appleBuilder.build(releaseData);
```

### Preset Comparison

```typescript
// See what presets are available
console.log('Available presets:', builder.getAvailablePresets());

// Get detailed preset information
const spotifyInfo = builder.getPresetInfo('spotify');
console.log('Spotify preset:', spotifyInfo);
console.log('Required fields:', spotifyInfo.requiredFields);
console.log('Validation rules:', builder.getPresetValidationRules('spotify'));
```

## Building from Scratch

### Create a Complete ERN Message

```typescript
import { DdexBuilder } from 'ddex-builder';

async function createCompleteERN() {
  const builder = new DdexBuilder();
  
  const ernData = {
    messageHeader: {
      messageId: `MSG_${Date.now()}`,
      messageSenderName: 'Awesome Records',
      messageRecipientName: 'Global Music Platform',
      messageCreatedDateTime: new Date().toISOString(),
      messageControlType: 'LiveMessage'
    },
    
    // Party information
    parties: [{
      partyId: 'LABEL_001',
      partyName: 'Awesome Records',
      partyType: 'Label',
      contactInfo: {
        email: 'contact@awesomerecords.com',
        website: 'https://awesomerecords.com'
      }
    }],
    
    // Release information
    releases: [{
      releaseId: 'REL_2024_001',
      title: 'Summer Vibes',
      artist: 'The Beachside Band',
      releaseType: 'Album',
      label: 'Awesome Records',
      labelId: 'LABEL_001',
      upc: '885686123456',
      releaseDate: '2024-06-21',
      originalReleaseDate: '2024-06-21',
      territories: ['WorldWide'],
      genres: ['Pop', 'Alternative', 'Indie'],
      parentalWarning: false,
      trackIds: ['SR_001', 'SR_002', 'SR_003']
    }],
    
    // Sound recordings
    resources: [{
      resourceId: 'SR_001',
      resourceType: 'SoundRecording', 
      title: 'Ocean Waves',
      artist: 'The Beachside Band',
      isrc: 'US-AWE-24-00001',
      duration: 'PT3M42S',
      trackNumber: 1,
      volumeNumber: 1,
      languageOfPerformance: 'en',
      metadata: {
        composer: 'John Beach, Sarah Shore',
        lyricist: 'Sarah Shore',
        producer: 'Mike Ocean',
        recordingYear: '2024'
      }
    }, {
      resourceId: 'SR_002',
      resourceType: 'SoundRecording',
      title: 'Sunset Dreams', 
      artist: 'The Beachside Band',
      isrc: 'US-AWE-24-00002',
      duration: 'PT4M18S',
      trackNumber: 2,
      volumeNumber: 1,
      languageOfPerformance: 'en',
      metadata: {
        composer: 'John Beach',
        lyricist: 'John Beach',
        producer: 'Mike Ocean',
        recordingYear: '2024'
      }
    }, {
      resourceId: 'SR_003',
      resourceType: 'SoundRecording',
      title: 'Coastal Highway',
      artist: 'The Beachside Band', 
      isrc: 'US-AWE-24-00003',
      duration: 'PT3M56S',
      trackNumber: 3,
      volumeNumber: 1,
      languageOfPerformance: 'en',
      metadata: {
        composer: 'Sarah Shore, Mike Ocean',
        lyricist: 'Sarah Shore', 
        producer: 'Mike Ocean',
        recordingYear: '2024'
      }
    }],
    
    // Commercial deals
    deals: [{
      dealId: 'DEAL_001',
      releaseId: 'REL_2024_001',
      territories: ['WorldWide'],
      useTypes: ['Stream', 'PermanentDownload', 'ConditionalDownload'],
      commercialModelType: 'Subscription',
      dealStartDate: '2024-06-21',
      priceInformation: {
        priceCurrency: 'USD',
        wholesalePrice: 9.99
      }
    }]
  };
  
  const xml = await builder.build(ernData);
  console.log('📄 Complete ERN generated:', xml.length, 'bytes');
  
  return xml;
}

// Generate the ERN
const ernXml = await createCompleteERN();
```

## Round-Trip Workflows

Combine with DDEX Parser for complete workflows:

### Parse → Modify → Build

```typescript
import { DDEXParser } from 'ddex-parser';
import { DdexBuilder } from 'ddex-builder';

async function modifyExistingERN(originalXmlPath: string) {
  // Parse existing DDEX file
  const parser = new DDEXParser();
  const parsed = await parser.parseFile(originalXmlPath);
  
  console.log('Original release:', parsed.flat.releases[0].title);
  
  // Convert to build request format
  const buildRequest = parsed.toBuildRequest();
  
  // Modify specific fields
  buildRequest.releases[0].title = 'Remastered Edition';
  buildRequest.releases[0].releaseDate = '2024-07-01';
  
  // Add remastered flag to all tracks
  buildRequest.resources.forEach(resource => {
    if (resource.resourceType === 'SoundRecording') {
      resource.metadata = resource.metadata || {};
      resource.metadata.remastered = 'true';
    }
  });
  
  // Build new deterministic XML
  const builder = new DdexBuilder();
  const newXml = await builder.build(buildRequest);
  
  // Verify round-trip fidelity
  const reparsed = await parser.parseString(newXml);
  console.log('Modified release:', reparsed.flat.releases[0].title);
  console.assert(reparsed.flat.releases[0].title === 'Remastered Edition');
  
  return newXml;
}
```

## Batch Building

Build multiple releases efficiently:

### Sequential Processing

```typescript
async function buildCatalog(catalogData: any[]) {
  const builder = new DdexBuilder();
  const results = [];
  
  for (const releaseData of catalogData) {
    try {
      const xml = await builder.build(releaseData);
      results.push({
        success: true,
        releaseId: releaseData.releases[0].releaseId,
        xml: xml,
        size: xml.length
      });
      
      console.log(`✅ Built ${releaseData.releases[0].title}`);
    } catch (error) {
      results.push({
        success: false,
        releaseId: releaseData.releases[0].releaseId,
        error: error.message
      });
      
      console.error(`❌ Failed ${releaseData.releases[0].title}:`, error.message);
    }
  }
  
  return results;
}
```

### Parallel Processing

```typescript
import { batchBuild } from 'ddex-builder';

async function buildCatalogParallel(catalogData: any[]) {
  // Build multiple releases in parallel
  const results = await batchBuild(catalogData, {
    preset: 'universal',
    parallel: true,
    maxConcurrency: 5
  });
  
  // Process results
  const successful = results.filter(r => r.success);
  const failed = results.filter(r => !r.success);
  
  console.log(`✅ Successfully built: ${successful.length} releases`);
  console.log(`❌ Failed to build: ${failed.length} releases`);
  
  if (failed.length > 0) {
    console.log('\nFailures:');
    failed.forEach(result => {
      console.log(`  ${result.releaseId}: ${result.error}`);
    });
  }
  
  return results;
}
```

## Python-Specific Examples

### DataFrame Integration

```python
from ddex_builder import DdexBuilder
import pandas as pd

# Create releases DataFrame
releases_df = pd.DataFrame([
    {
        'release_id': 'REL_001',
        'title': 'Summer Hits',
        'artist': 'Various Artists',
        'release_type': 'Compilation',
        'upc': '123456789012',
        'release_date': '2024-06-01',
        'territories': ['US', 'CA', 'GB']
    },
    {
        'release_id': 'REL_002', 
        'title': 'Indie Rock Collection',
        'artist': 'Various Artists',
        'release_type': 'Compilation',
        'upc': '123456789013',
        'release_date': '2024-06-15',
        'territories': ['WorldWide']
    }
])

# Create tracks DataFrame
tracks_df = pd.DataFrame([
    {
        'resource_id': 'TR_001',
        'title': 'Beach Party',
        'artist': 'The Summer Band',
        'isrc': 'US1234567890',
        'duration': 'PT3M30S',
        'release_id': 'REL_001'
    },
    {
        'resource_id': 'TR_002',
        'title': 'Sunshine Road',
        'artist': 'Indie Rock Heroes', 
        'isrc': 'US1234567891',
        'duration': 'PT4M15S',
        'release_id': 'REL_002'
    }
])

# Build from DataFrames
builder = DdexBuilder()
xml = builder.from_dataframes({
    'releases': releases_df,
    'resources': tracks_df
}, message_header={
    'message_id': 'CATALOG_2024_001',
    'message_sender_name': 'My Label',
    'message_recipient_name': 'Distribution Platform'
})

print(f'Generated catalog: {len(xml)} bytes')
```

### Async Processing

```python
import asyncio
from ddex_builder import DdexBuilder

async def build_async_catalog(release_data_list):
    """Build multiple releases asynchronously."""
    builder = DdexBuilder()
    
    async def build_single(release_data):
        try:
            xml = await builder.build_async(release_data)
            return {
                'success': True,
                'release_id': release_data['releases'][0]['release_id'],
                'xml': xml,
                'size': len(xml)
            }
        except Exception as e:
            return {
                'success': False,
                'release_id': release_data['releases'][0]['release_id'],
                'error': str(e)
            }
    
    # Run builds concurrently
    tasks = [build_single(data) for data in release_data_list]
    results = await asyncio.gather(*tasks)
    
    # Summary
    successful = [r for r in results if r['success']]
    failed = [r for r in results if not r['success']]
    
    print(f"✅ Built {len(successful)} releases successfully")
    print(f"❌ Failed {len(failed)} releases")
    
    return results

# Usage
results = asyncio.run(build_async_catalog([release1_data, release2_data]))
```

## Validation and Error Handling

### Pre-flight Validation

```typescript
import { DdexBuilder, ValidationResult } from 'ddex-builder';

async function validateBeforeBuilding(releaseData: any) {
  const builder = new DdexBuilder();
  
  // Validate structure first
  const validation: ValidationResult = await builder.validate(releaseData);
  
  if (!validation.isValid) {
    console.log('❌ Validation failed:');
    validation.errors.forEach(error => {
      console.log(`  Error: ${error}`);
    });
    
    if (validation.warnings.length > 0) {
      console.log('⚠️ Warnings:');
      validation.warnings.forEach(warning => {
        console.log(`  Warning: ${warning}`);
      });
    }
    
    throw new Error('Data validation failed');
  }
  
  console.log('✅ Validation passed');
  
  // Now safe to build
  const xml = await builder.build(releaseData);
  return xml;
}
```

### Error Recovery

```typescript
async function buildWithRetry(releaseData: any, maxRetries: number = 3) {
  const builder = new DdexBuilder();
  
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await builder.build(releaseData);
    } catch (error) {
      console.log(`Attempt ${attempt} failed:`, error.message);
      
      if (attempt === maxRetries) {
        throw new Error(`Failed after ${maxRetries} attempts: ${error.message}`);
      }
      
      // Wait before retry (exponential backoff)
      await new Promise(resolve => setTimeout(resolve, 1000 * attempt));
    }
  }
}
```

## Next Steps

Now that you can build DDEX files, explore more advanced features:

- **[API Reference](./api-reference)** - Complete method documentation  
- **[Canonicalization](./canonicalization)** - Understanding deterministic output
- **[Presets](./presets)** - Platform-specific configurations
- **[Parser Integration](../parser/)** - Parse existing DDEX files for round-trip workflows