# Version Upgrades

Migrate between different DDEX schema versions using DDEX Suite.

## Overview

DDEX Suite supports multiple schema versions and provides tools for:
- Parsing different DDEX versions (3.8.2, 4.2, 4.3)
- Converting between schema versions
- Maintaining backward compatibility
- Validating version-specific requirements

## Supported Versions

| Version | Support Level | Features |
|---------|---------------|----------|
| ERN 3.8.2 | Full Support | Legacy compatibility, stable parsing |
| ERN 4.2 | Full Support | Enhanced metadata, improved structure |
| ERN 4.3 | Full Support | Latest features, recommended for new implementations |

## Version Detection

```typescript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();

async function detectAndParseVersion(xmlContent: string) {
  try {
    const result = await parser.parse(xmlContent);
    
    console.log(`Detected DDEX version: ${result.version}`);
    console.log(`Schema: ${result.schema}`);
    
    return result;
    
  } catch (error) {
    console.error('Failed to parse or detect version:', error);
    throw error;
  }
}

// Parse with specific version if detection fails
async function parseWithExplicitVersion(xmlContent: string, version: string) {
  const result = await parser.parse(xmlContent, { 
    version,
    strict: false // Allow minor version differences
  });
  
  return result;
}
```

## Version Conversion

### Upgrading from 3.8.2 to 4.3

```python
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

class DDEXVersionUpgrader:
    def __init__(self):
        self.parser = DDEXParser()
        self.builder = DDEXBuilder()
    
    def upgrade_382_to_43(self, xml_content: str) -> str:
        """Upgrade DDEX 3.8.2 to 4.3"""
        
        # Parse existing 3.8.2 format
        result = self.parser.parse(xml_content)
        
        if result.version != '3.8.2':
            raise ValueError(f"Expected version 3.8.2, got {result.version}")
        
        # Transform data for 4.3 compatibility
        upgraded_data = self._transform_382_to_43(result.flat.to_dict())
        
        # Build new 4.3 XML
        new_xml = self.builder.build(upgraded_data, version='4.3')
        
        return new_xml
    
    def _transform_382_to_43(self, data_382: dict) -> dict:
        """Transform 3.8.2 data structure to 4.3"""
        
        transformed = {
            'message_header': self._upgrade_message_header(data_382.get('message_header', {})),
            'releases': []
        }
        
        for release in data_382.get('releases', []):
            upgraded_release = {
                'id': release.get('id'),
                'title': release.get('title'),
                'artist': release.get('artist'),
                'release_date': release.get('release_date'),
                
                # New 4.3 fields
                'release_type': release.get('release_type', 'Album'),
                'genre': self._normalize_genres(release.get('genre', [])),
                
                # Enhanced metadata structure in 4.3
                'metadata': {
                    'label': release.get('label'),
                    'catalog_number': release.get('catalog_number'),
                    'upc': release.get('upc'),
                    'territory_codes': release.get('territories', [])
                },
                
                'tracks': self._upgrade_tracks(release.get('tracks', []))
            }
            
            transformed['releases'].append(upgraded_release)
        
        return transformed
    
    def _upgrade_message_header(self, header_382: dict) -> dict:
        """Upgrade message header to 4.3 format"""
        
        return {
            'message_id': header_382.get('message_id'),
            'message_sender': header_382.get('sender_party_id'),
            'message_recipient': header_382.get('recipient_party_id'),
            'message_created_date_time': header_382.get('message_created_date_time'),
            'message_schema_version': 'ern/43',  # Upgrade to 4.3
            
            # New 4.3 fields
            'message_control_type': 'TestMessage'  # Default for testing
        }
    
    def _upgrade_tracks(self, tracks_382: list) -> list:
        """Upgrade track structure to 4.3"""
        
        upgraded_tracks = []
        
        for track in tracks_382:
            upgraded_track = {
                'id': track.get('id'),
                'title': track.get('title'),
                'artist': track.get('artist', track.get('display_artist')),
                'duration_ms': track.get('duration_ms'),
                'track_number': track.get('sequence_number'),  # Field renamed in 4.3
                'isrc': track.get('isrc'),
                
                # Enhanced fields in 4.3
                'parental_warning_type': self._upgrade_parental_warning(
                    track.get('parental_advisory')
                ),
                
                # New metadata structure
                'metadata': {
                    'language': track.get('language_of_performance'),
                    'genre': self._normalize_genres(track.get('genre', []))
                }
            }
            
            upgraded_tracks.append(upgraded_track)
        
        return upgraded_tracks
    
    def _upgrade_parental_warning(self, advisory_382: str) -> str:
        """Upgrade parental advisory format"""
        
        mapping = {
            'Explicit': 'Explicit',
            'Clean': 'NotExplicit',
            'Edited': 'NotExplicit',
            None: 'Unknown'
        }
        
        return mapping.get(advisory_382, 'Unknown')
    
    def _normalize_genres(self, genres: list) -> list:
        """Normalize genre formats between versions"""
        
        if not genres:
            return []
        
        # 4.3 uses standardized genre codes
        genre_mapping = {
            'Rock': 'Rock',
            'Pop': 'Pop',
            'Electronic': 'Electronic',
            'Hip Hop': 'HipHop',
            'R&B': 'RAndB',
            # Add more mappings as needed
        }
        
        normalized = []
        for genre in genres:
            if isinstance(genre, str):
                normalized.append(genre_mapping.get(genre, genre))
            elif isinstance(genre, dict) and 'text' in genre:
                normalized.append(genre_mapping.get(genre['text'], genre['text']))
        
        return normalized

# Usage example
upgrader = DDEXVersionUpgrader()

with open('legacy_382_release.xml', 'r') as f:
    legacy_xml = f.read()

# Upgrade to 4.3
upgraded_xml = upgrader.upgrade_382_to_43(legacy_xml)

# Save upgraded version
with open('upgraded_43_release.xml', 'w') as f:
    f.write(upgraded_xml)

print("Successfully upgraded from DDEX 3.8.2 to 4.3")
```

### Downgrading for Compatibility

```typescript
import { DDEXParser, DDEXBuilder } from 'ddex-suite';

export class DDEXVersionDowngrader {
  private parser = new DDEXParser();
  private builder = new DDEXBuilder();

  async downgrade43To382(xmlContent: string): Promise<string> {
    // Parse 4.3 format
    const result = await this.parser.parse(xmlContent);
    
    if (result.version !== '4.3') {
      throw new Error(`Expected version 4.3, got ${result.version}`);
    }

    // Transform to 3.8.2 compatible structure
    const downgraded = this.transform43To382(result.flat);

    // Build 3.8.2 XML
    const xml382 = await this.builder.build(downgraded, { version: '3.8.2' });

    return xml382;
  }

  private transform43To382(data43: any): any {
    return {
      messageHeader: this.downgradeMessageHeader(data43.messageHeader),
      releases: data43.releases.map(release => this.downgradeRelease(release))
    };
  }

  private downgradeMessageHeader(header43: any): any {
    return {
      messageId: header43.messageId,
      senderPartyId: header43.messageSender,
      recipientPartyId: header43.messageRecipient,
      messageCreatedDateTime: header43.messageCreatedDateTime,
      messageSchemaVersion: 'ern/382'  // Downgrade to 3.8.2
    };
  }

  private downgradeRelease(release43: any): any {
    return {
      id: release43.id,
      title: release43.title,
      artist: release43.artist,
      releaseDate: release43.releaseDate,
      
      // Map 4.3 fields to 3.8.2 equivalents
      label: release43.metadata?.label,
      catalogNumber: release43.metadata?.catalogNumber,
      upc: release43.metadata?.upc,
      territories: release43.metadata?.territoryCodes,
      
      // Handle fields that don't exist in 3.8.2
      // (Some data may be lost in downgrade)
      
      tracks: release43.tracks?.map(track => this.downgradeTrack(track)) || []
    };
  }

  private downgradeTrack(track43: any): any {
    return {
      id: track43.id,
      title: track43.title,
      displayArtist: track43.artist,
      durationMs: track43.durationMs,
      sequenceNumber: track43.trackNumber,  // Field renamed from 4.3
      isrc: track43.isrc,
      
      // Downgrade parental warning format
      parentalAdvisory: this.downgradeParentalWarning(track43.parentalWarningType),
      
      // Some 4.3 metadata may not fit in 3.8.2 structure
      languageOfPerformance: track43.metadata?.language,
      genre: track43.metadata?.genre
    };
  }

  private downgradeParentalWarning(warning43: string): string | undefined {
    const mapping: { [key: string]: string } = {
      'Explicit': 'Explicit',
      'NotExplicit': 'Clean',
      'Unknown': undefined
    };

    return mapping[warning43];
  }
}
```

## Batch Version Conversion

```python
import os
import json
from concurrent.futures import ThreadPoolExecutor
from typing import List, Dict, Any

class BatchVersionConverter:
    def __init__(self, max_workers: int = 4):
        self.upgrader = DDEXVersionUpgrader()
        self.max_workers = max_workers
        
    def convert_directory(self, 
                         input_dir: str, 
                         output_dir: str, 
                         target_version: str = '4.3') -> Dict[str, Any]:
        """Convert all DDEX files in directory to target version"""
        
        # Find all XML files
        xml_files = []
        for root, dirs, files in os.walk(input_dir):
            for file in files:
                if file.lower().endswith('.xml'):
                    xml_files.append(os.path.join(root, file))
        
        print(f"Found {len(xml_files)} XML files to convert")
        
        # Process files in parallel
        results = []
        with ThreadPoolExecutor(max_workers=self.max_workers) as executor:
            futures = {
                executor.submit(
                    self._convert_single_file, 
                    xml_file, 
                    input_dir, 
                    output_dir, 
                    target_version
                ): xml_file 
                for xml_file in xml_files
            }
            
            for future in futures:
                xml_file = futures[future]
                try:
                    result = future.result()
                    results.append(result)
                except Exception as e:
                    results.append({
                        'file': xml_file,
                        'status': 'error',
                        'error': str(e)
                    })
        
        # Generate summary
        successful = len([r for r in results if r['status'] == 'success'])
        failed = len([r for r in results if r['status'] == 'error'])
        
        summary = {
            'total_files': len(xml_files),
            'successful': successful,
            'failed': failed,
            'success_rate': f"{successful / len(xml_files) * 100:.1f}%",
            'results': results
        }
        
        # Save conversion report
        report_file = os.path.join(output_dir, 'conversion_report.json')
        with open(report_file, 'w') as f:
            json.dump(summary, f, indent=2)
        
        return summary
    
    def _convert_single_file(self, 
                           xml_file: str, 
                           input_dir: str, 
                           output_dir: str, 
                           target_version: str) -> Dict[str, Any]:
        """Convert a single file"""
        
        try:
            # Read source file
            with open(xml_file, 'r', encoding='utf-8') as f:
                xml_content = f.read()
            
            # Determine conversion method based on target version
            if target_version == '4.3':
                converted_xml = self.upgrader.upgrade_382_to_43(xml_content)
            else:
                raise ValueError(f"Unsupported target version: {target_version}")
            
            # Create output path
            relative_path = os.path.relpath(xml_file, input_dir)
            output_file = os.path.join(output_dir, relative_path)
            
            # Create output directory if needed
            os.makedirs(os.path.dirname(output_file), exist_ok=True)
            
            # Write converted file
            with open(output_file, 'w', encoding='utf-8') as f:
                f.write(converted_xml)
            
            return {
                'file': xml_file,
                'status': 'success',
                'output_file': output_file,
                'target_version': target_version
            }
            
        except Exception as e:
            return {
                'file': xml_file,
                'status': 'error',
                'error': str(e)
            }

# Usage
converter = BatchVersionConverter(max_workers=4)

summary = converter.convert_directory(
    input_dir='./legacy_ddex_files',
    output_dir='./upgraded_ddex_files',
    target_version='4.3'
)

print(f"Conversion completed: {summary['success_rate']} success rate")
```

## Version-Specific Considerations

### ERN 3.8.2 → ERN 4.2/4.3

**Key Changes:**
- Message structure improvements
- Enhanced metadata fields
- Territory code standardization
- Parental advisory format changes

**Common Issues:**
- Missing required fields in newer versions
- Genre format changes
- Territory code format updates
- Date format standardization

### ERN 4.2 → ERN 4.3

**Key Changes:**
- Minor schema updates
- Additional optional fields
- Improved validation rules

**Migration Notes:**
- Generally backward compatible
- Few breaking changes
- Mostly additive improvements

## Testing Version Conversions

```python
import unittest
from ddex_parser import DDEXParser

class VersionConversionTests(unittest.TestCase):
    def setUp(self):
        self.parser = DDEXParser()
        self.upgrader = DDEXVersionUpgrader()
    
    def test_382_to_43_upgrade(self):
        """Test upgrading from 3.8.2 to 4.3"""
        
        # Load 3.8.2 test file
        with open('test_data/sample_382.xml', 'r') as f:
            xml_382 = f.read()
        
        # Upgrade to 4.3
        xml_43 = self.upgrader.upgrade_382_to_43(xml_382)
        
        # Parse both versions
        result_382 = self.parser.parse(xml_382)
        result_43 = self.parser.parse(xml_43)
        
        # Verify version upgrade
        self.assertEqual(result_382.version, '3.8.2')
        self.assertEqual(result_43.version, '4.3')
        
        # Verify data preservation
        self.assertEqual(
            len(result_382.flat.releases),
            len(result_43.flat.releases)
        )
        
        # Check specific field mappings
        release_382 = result_382.flat.releases[0]
        release_43 = result_43.flat.releases[0]
        
        self.assertEqual(release_382.title, release_43.title)
        self.assertEqual(release_382.artist, release_43.artist)
    
    def test_round_trip_conversion(self):
        """Test that upgrade + downgrade preserves data"""
        
        with open('test_data/sample_382.xml', 'r') as f:
            original_382 = f.read()
        
        # Upgrade to 4.3 then downgrade back to 3.8.2
        xml_43 = self.upgrader.upgrade_382_to_43(original_382)
        downgrader = DDEXVersionDowngrader()
        xml_382_restored = downgrader.downgrade_43_to_382(xml_43)
        
        # Parse both original and restored
        original = self.parser.parse(original_382)
        restored = self.parser.parse(xml_382_restored)
        
        # Verify essential data is preserved
        self.assertEqual(
            len(original.flat.releases),
            len(restored.flat.releases)
        )
        
        # Check key fields are preserved
        orig_release = original.flat.releases[0]
        rest_release = restored.flat.releases[0]
        
        self.assertEqual(orig_release.title, rest_release.title)
        self.assertEqual(orig_release.artist, rest_release.artist)

if __name__ == '__main__':
    unittest.main()
```

## Best Practices

1. **Version Detection**: Always detect the source version before conversion
2. **Data Validation**: Validate data before and after conversion
3. **Backup Strategy**: Keep original files as backup
4. **Incremental Migration**: Convert files in small batches first
5. **Testing**: Thoroughly test conversions with representative data
6. **Documentation**: Document version-specific changes and requirements
7. **Monitoring**: Monitor conversion success rates
8. **Rollback Plan**: Have a rollback plan if conversion issues arise
9. **Schema Validation**: Validate against target schema after conversion
10. **Performance Testing**: Test conversion performance with large datasets