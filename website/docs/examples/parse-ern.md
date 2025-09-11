---
sidebar_position: 1
---

# Complete ERN Parsing Example

Learn how to parse DDEX ERN (Electronic Release Notification) messages and extract structured data for your applications.

## Basic ERN Parsing

### JavaScript / TypeScript

```typescript
import { DDEXParser } from 'ddex-parser';
import { readFileSync } from 'fs';

async function parseERNExample() {
  // Initialize the parser
  const parser = new DDEXParser({
    validation: 'strict',
    includeRawExtensions: true
  });

  // Load ERN file
  const xmlContent = readFileSync('release-notification.xml', 'utf-8');
  
  try {
    // Parse the ERN message
    const result = await parser.parseString(xmlContent);
    
    console.log('📄 Parsed ERN Message');
    console.log('===================');
    console.log(`Message ID: ${result.messageId}`);
    console.log(`DDEX Version: ${result.version}`);
    console.log(`Created: ${result.graph.messageHeader.messageCreatedDateTime}`);
    
    return result;
  } catch (error) {
    console.error('❌ Parsing failed:', error.message);
    throw error;
  }
}

// Run the example
parseERNExample();
```

### Python

```python
from ddex_parser import DDEXParser
import json

def parse_ern_example():
    """Parse a DDEX ERN message and extract data."""
    
    # Initialize parser
    parser = DDEXParser()
    
    # Load ERN file
    with open('release-notification.xml', 'r', encoding='utf-8') as f:
        xml_content = f.read()
    
    try:
        # Parse the ERN message
        result = parser.parse(xml_content)
        
        print('📄 Parsed ERN Message')
        print('===================')
        print(f'Message ID: {result.message_id}')
        print(f'DDEX Version: {result.version}')
        print(f'Releases: {len(result.releases)}')
        print(f'Sound Recordings: {len(result.sound_recordings)}')
        
        return result
        
    except Exception as e:
        print(f'❌ Parsing failed: {e}')
        raise

# Run the example
if __name__ == "__main__":
    result = parse_ern_example()
```

## Extracting Release Information

### Accessing Flattened Data

The flattened model provides easy access to denormalized data:

```typescript
async function extractReleaseInfo(xmlPath: string) {
  const parser = new DDEXParser();
  const result = await parser.parseFile(xmlPath);
  
  // Extract release information
  result.flat.releases.forEach((release, index) => {
    console.log(`\n🎵 Release ${index + 1}:`);
    console.log(`  Title: ${release.title}`);
    console.log(`  Artist: ${release.displayArtist}`);
    console.log(`  Label: ${release.label || 'Unknown'}`);
    console.log(`  Release Date: ${release.releaseDate || 'TBD'}`);
    console.log(`  UPC/EAN: ${release.releaseId?.find(id => id.namespace === 'UPC')?.value || 'N/A'}`);
    console.log(`  Territories: ${release.territories?.join(', ') || 'Not specified'}`);
    console.log(`  Genres: ${release.genres?.join(', ') || 'Not specified'}`);
    
    // Release identifiers
    if (release.releaseId && release.releaseId.length > 0) {
      console.log('  Identifiers:');
      release.releaseId.forEach(id => {
        console.log(`    ${id.namespace}: ${id.value}`);
      });
    }
    
    // P-Line and C-Line information
    if (release.pLine) console.log(`  P-Line: ${release.pLine}`);
    if (release.cLine) console.log(`  C-Line: ${release.cLine}`);
  });
  
  return result.flat.releases;
}

// Usage
const releases = await extractReleaseInfo('album-release.xml');
```

### Python DataFrame Export

```python
import pandas as pd
from ddex_parser import DDEXParser

def ern_to_dataframe(xml_path):
    """Convert ERN data to pandas DataFrames for analysis."""
    
    parser = DDEXParser()
    
    # Parse directly to DataFrame
    df = parser.to_dataframe(xml_path)
    
    print("📊 ERN Data Analysis")
    print("==================")
    print(f"Total releases: {len(df)}")
    
    # Release analysis
    if not df.empty:
        print(f"\nRelease Types:")
        print(df['release_type'].value_counts())
        
        print(f"\nGenre Distribution:")
        genre_df = df.explode('genres').groupby('genres').size()
        print(genre_df.head(10))
        
        print(f"\nTerritory Coverage:")
        territory_df = df.explode('territories').groupby('territories').size()
        print(territory_df.head(10))
        
        # Export to CSV for further analysis
        df.to_csv('ern_releases.csv', index=False)
        print(f"\n💾 Data exported to ern_releases.csv")
    
    return df

# Usage
df = ern_to_dataframe('catalog.xml')
```

## Working with Sound Recordings

### Extracting Track Information

```typescript
async function extractTrackInfo(xmlPath: string) {
  const parser = new DDEXParser();
  const result = await parser.parseFile(xmlPath);
  
  console.log('\n🎵 Sound Recordings:');
  console.log('===================');
  
  result.flat.soundRecordings.forEach((recording, index) => {
    console.log(`\nTrack ${index + 1}:`);
    console.log(`  Title: ${recording.title}`);
    console.log(`  Artist: ${recording.displayArtist}`);
    console.log(`  ISRC: ${recording.isrc || 'Not assigned'}`);
    console.log(`  Duration: ${recording.duration || 'Unknown'}`);
    
    // Track position information
    if (recording.trackNumber !== undefined) {
      console.log(`  Track Number: ${recording.trackNumber}`);
    }
    if (recording.volumeNumber !== undefined) {
      console.log(`  Volume/Disc: ${recording.volumeNumber}`);
    }
    
    // Genre and mood information
    if (recording.genres && recording.genres.length > 0) {
      console.log(`  Genres: ${recording.genres.join(', ')}`);
    }
    
    // Contributors (composers, producers, etc.)
    if (recording.contributors && recording.contributors.length > 0) {
      console.log('  Contributors:');
      recording.contributors.forEach(contributor => {
        console.log(`    ${contributor.role}: ${contributor.name}`);
      });
    }
    
    // Technical details
    if (recording.technicalDetails) {
      console.log('  Technical Details:');
      const tech = recording.technicalDetails;
      if (tech.audioCodecType) console.log(`    Codec: ${tech.audioCodecType}`);
      if (tech.bitRate) console.log(`    Bitrate: ${tech.bitRate} kbps`);
      if (tech.sampleRate) console.log(`    Sample Rate: ${tech.sampleRate} Hz`);
      if (tech.numberOfChannels) console.log(`    Channels: ${tech.numberOfChannels}`);
    }
  });
  
  return result.flat.soundRecordings;
}

// Usage
const tracks = await extractTrackInfo('album-with-tracks.xml');
```

### Creating Track Analysis Report

```python
def analyze_tracks(xml_path):
    """Analyze track data from ERN message."""
    
    parser = DDEXParser()
    result = parser.parse_file(xml_path)
    
    tracks = result.sound_recordings
    
    if not tracks:
        print("No sound recordings found")
        return
    
    print("🎵 Track Analysis Report")
    print("======================")
    
    # Duration analysis
    total_duration = sum(
        parse_duration(track.duration) for track in tracks 
        if track.duration
    )
    avg_duration = total_duration / len(tracks) if tracks else 0
    
    print(f"Total tracks: {len(tracks)}")
    print(f"Total duration: {format_duration(total_duration)}")
    print(f"Average duration: {format_duration(avg_duration)}")
    
    # ISRC analysis
    tracks_with_isrc = sum(1 for track in tracks if track.isrc)
    print(f"Tracks with ISRC: {tracks_with_isrc}/{len(tracks)} ({tracks_with_isrc/len(tracks)*100:.1f}%)")
    
    # Genre analysis
    all_genres = []
    for track in tracks:
        if track.genres:
            all_genres.extend(track.genres)
    
    if all_genres:
        genre_counts = pd.Series(all_genres).value_counts()
        print(f"\nTop Genres:")
        print(genre_counts.head())
    
    # Track numbering analysis
    numbered_tracks = [t for t in tracks if t.track_number is not None]
    if numbered_tracks:
        track_numbers = [t.track_number for t in numbered_tracks]
        print(f"\nTrack numbering: {min(track_numbers)} - {max(track_numbers)}")
    
    return tracks

def parse_duration(duration_str):
    """Parse ISO 8601 duration to seconds."""
    import re
    if not duration_str:
        return 0
    
    # Simple parser for PT#M#S format
    match = re.match(r'PT(?:(\d+)M)?(?:(\d+)S)?', duration_str)
    if match:
        minutes = int(match.group(1)) if match.group(1) else 0
        seconds = int(match.group(2)) if match.group(2) else 0
        return minutes * 60 + seconds
    return 0

def format_duration(seconds):
    """Format seconds as MM:SS."""
    minutes, secs = divmod(int(seconds), 60)
    return f"{minutes}:{secs:02d}"

# Usage
tracks = analyze_tracks('album.xml')
```

## Handling Commercial Terms

### Extracting Deal Information

```typescript
async function extractDealInfo(xmlPath: string) {
  const parser = new DDEXParser();
  const result = await parser.parseFile(xmlPath);
  
  console.log('\n💼 Commercial Deals:');
  console.log('===================');
  
  result.flat.deals.forEach((deal, index) => {
    console.log(`\nDeal ${index + 1}:`);
    console.log(`  Deal Type: ${deal.commercialModelType}`);
    console.log(`  Use Types: ${deal.useTypes?.join(', ') || 'Not specified'}`);
    console.log(`  Territories: ${deal.territories?.join(', ') || 'Worldwide'}`);
    
    // Validity period
    if (deal.validityPeriod) {
      console.log('  Validity Period:');
      if (deal.validityPeriod.startDate) {
        console.log(`    Start: ${deal.validityPeriod.startDate}`);
      }
      if (deal.validityPeriod.endDate) {
        console.log(`    End: ${deal.validityPeriod.endDate}`);
      }
      if (!deal.validityPeriod.endDate) {
        console.log('    End: Perpetual');
      }
    }
    
    // Price information
    if (deal.priceInformation) {
      console.log('  Pricing:');
      deal.priceInformation.forEach(price => {
        console.log(`    ${price.priceType}: ${price.amount} ${price.currency}`);
      });
    }
    
    // Deal conditions
    if (deal.conditions && deal.conditions.length > 0) {
      console.log('  Conditions:');
      deal.conditions.forEach(condition => {
        console.log(`    - ${condition}`);
      });
    }
  });
  
  return result.flat.deals;
}

// Usage
const deals = await extractDealInfo('distribution-deal.xml');
```

## Party Information

### Extracting Party Details

```typescript
async function extractPartyInfo(xmlPath: string) {
  const parser = new DDEXParser();
  const result = await parser.parseFile(xmlPath);
  
  console.log('\n👥 Parties:');
  console.log('===========');
  
  result.flat.parties.forEach((party, index) => {
    console.log(`\nParty ${index + 1}:`);
    console.log(`  Name: ${party.partyName}`);
    console.log(`  Type: ${party.partyType || 'Not specified'}`);
    console.log(`  Reference: ${party.partyReference || 'N/A'}`);
    
    // Party identifiers
    if (party.partyId && party.partyId.length > 0) {
      console.log('  Identifiers:');
      party.partyId.forEach(id => {
        console.log(`    ${id.namespace}: ${id.value}`);
      });
    }
    
    // Contact information
    if (party.contactInfo) {
      console.log('  Contact:');
      if (party.contactInfo.email) {
        console.log(`    Email: ${party.contactInfo.email}`);
      }
      if (party.contactInfo.phone) {
        console.log(`    Phone: ${party.contactInfo.phone}`);
      }
      if (party.contactInfo.website) {
        console.log(`    Website: ${party.contactInfo.website}`);
      }
    }
    
    // Roles in this message
    if (party.roles && party.roles.length > 0) {
      console.log(`  Roles: ${party.roles.join(', ')}`);
    }
  });
  
  return result.flat.parties;
}

// Usage
const parties = await extractPartyInfo('release-with-parties.xml');
```

## Error Handling and Validation

### Comprehensive Error Handling

```typescript
import { 
  DDEXParser, 
  DDEXError, 
  ValidationError, 
  SecurityError,
  UnsupportedVersionError 
} from 'ddex-parser';

async function robustERNParsing(xmlPath: string) {
  const parser = new DDEXParser({
    validation: 'strict',
    includeWarnings: true,
    maxMemoryMB: 100
  });
  
  try {
    // Pre-validate the XML
    const xmlContent = await fs.readFile(xmlPath, 'utf-8');
    
    // Check file size
    const fileSizeMB = Buffer.byteLength(xmlContent, 'utf-8') / (1024 * 1024);
    if (fileSizeMB > 50) {
      console.warn(`⚠️ Large file detected: ${fileSizeMB.toFixed(2)}MB`);
    }
    
    // Detect version before parsing
    const detectedVersion = await parser.detectVersion(xmlContent);
    console.log(`📋 Detected DDEX version: ${detectedVersion}`);
    
    // Parse with comprehensive error handling
    const result = await parser.parseString(xmlContent);
    
    console.log('✅ Successfully parsed ERN message');
    console.log(`📊 Summary: ${result.flat.releases.length} releases, ${result.flat.soundRecordings.length} tracks`);
    
    return result;
    
  } catch (error) {
    // Handle different types of errors
    if (error instanceof SecurityError) {
      console.error('🔒 Security Error:', error.message);
      console.error('This file may contain malicious XML constructs');
      
    } else if (error instanceof UnsupportedVersionError) {
      console.error('📅 Version Error:', error.message);
      console.error(`Detected: ${error.detectedVersion}`);
      console.error(`Supported: ${error.supportedVersions.join(', ')}`);
      
    } else if (error instanceof ValidationError) {
      console.error('📝 Validation Error:', error.message);
      if (error.details && error.details.length > 0) {
        console.error('Details:');
        error.details.forEach((detail, index) => {
          console.error(`  ${index + 1}. ${detail}`);
        });
      }
      
    } else if (error instanceof DDEXError) {
      console.error('❌ Parsing Error:', error.message);
      if (error.line !== undefined) {
        console.error(`Location: Line ${error.line}, Column ${error.column || 'unknown'}`);
      }
      if (error.context) {
        console.error(`Context: ${error.context}`);
      }
      
    } else {
      console.error('💥 Unexpected Error:', error.message);
    }
    
    // Re-throw for caller to handle
    throw error;
  }
}

// Usage with graceful degradation
async function processERNWithFallback(xmlPath: string) {
  try {
    return await robustERNParsing(xmlPath);
  } catch (error) {
    console.log('🔄 Attempting fallback parsing with relaxed validation...');
    
    try {
      const parser = new DDEXParser({ 
        validation: 'permissive',
        includeRawExtensions: true
      });
      return await parser.parseFile(xmlPath);
    } catch (fallbackError) {
      console.error('💀 All parsing attempts failed');
      throw fallbackError;
    }
  }
}
```

## Complete Example Application

Here's a complete Node.js application that demonstrates ERN parsing:

```typescript
import { DDEXParser } from 'ddex-parser';
import { promises as fs } from 'fs';
import path from 'path';

interface ERNProcessingResult {
  success: boolean;
  filePath: string;
  messageId?: string;
  version?: string;
  releases?: number;
  tracks?: number;
  error?: string;
}

class ERNProcessor {
  private parser: DDEXParser;
  
  constructor() {
    this.parser = new DDEXParser({
      validation: 'strict',
      includeRawExtensions: true,
      maxMemoryMB: 200
    });
  }
  
  async processDirectory(directoryPath: string): Promise<ERNProcessingResult[]> {
    const results: ERNProcessingResult[] = [];
    
    try {
      const files = await fs.readdir(directoryPath);
      const xmlFiles = files.filter(file => file.toLowerCase().endsWith('.xml'));
      
      console.log(`📁 Processing ${xmlFiles.length} XML files from ${directoryPath}`);
      
      for (const file of xmlFiles) {
        const filePath = path.join(directoryPath, file);
        const result = await this.processFile(filePath);
        results.push(result);
      }
      
      this.printSummary(results);
      
    } catch (error) {
      console.error(`❌ Failed to process directory: ${error.message}`);
    }
    
    return results;
  }
  
  async processFile(filePath: string): Promise<ERNProcessingResult> {
    console.log(`📄 Processing ${path.basename(filePath)}...`);
    
    try {
      const result = await this.parser.parseFile(filePath);
      
      return {
        success: true,
        filePath,
        messageId: result.messageId,
        version: result.version,
        releases: result.flat.releases.length,
        tracks: result.flat.soundRecordings.length
      };
      
    } catch (error) {
      console.error(`❌ Failed to parse ${path.basename(filePath)}: ${error.message}`);
      
      return {
        success: false,
        filePath,
        error: error.message
      };
    }
  }
  
  private printSummary(results: ERNProcessingResult[]) {
    const successful = results.filter(r => r.success);
    const failed = results.filter(r => !r.success);
    
    console.log('\n📊 Processing Summary');
    console.log('====================');
    console.log(`✅ Successfully processed: ${successful.length} files`);
    console.log(`❌ Failed to process: ${failed.length} files`);
    
    if (successful.length > 0) {
      const totalReleases = successful.reduce((sum, r) => sum + (r.releases || 0), 0);
      const totalTracks = successful.reduce((sum, r) => sum + (r.tracks || 0), 0);
      
      console.log(`🎵 Total releases found: ${totalReleases}`);
      console.log(`🎵 Total tracks found: ${totalTracks}`);
    }
    
    if (failed.length > 0) {
      console.log('\nFailed files:');
      failed.forEach(result => {
        console.log(`  - ${path.basename(result.filePath)}: ${result.error}`);
      });
    }
  }
}

// Usage
async function main() {
  const processor = new ERNProcessor();
  
  // Process individual file
  await processor.processFile('./samples/release.xml');
  
  // Process entire directory
  await processor.processDirectory('./ddex-files');
}

// Run if called directly
if (require.main === module) {
  main().catch(console.error);
}

export { ERNProcessor };
```

This comprehensive example shows how to:

- Parse ERN messages with proper error handling
- Extract release, track, deal, and party information
- Handle different DDEX versions
- Process multiple files in batch
- Export data for analysis
- Build robust production-ready parsing applications

For more advanced scenarios, see our other examples:
- [Building New Releases](./build-new-release)
- [Round-Trip Processing](./round-trip)
- [Batch Processing](./batch-processing)
- [Python DataFrame Integration](./python-dataframes)