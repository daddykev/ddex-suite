# Migration from Raw XML

Migrate from manual XML processing to DDEX Suite's structured approach.

## Overview

Migrating from raw XML processing provides:
- Type-safe data structures
- Built-in validation
- Simplified data access
- Better error handling
- Automated schema compliance

## Before and After Comparison

### Raw XML Processing (Before)

```python
import xml.etree.ElementTree as ET

def extract_release_data_xml(xml_content):
    """Old approach: Manual XML parsing"""
    
    root = ET.fromstring(xml_content)
    
    # Navigate complex XML structure manually
    releases = []
    
    # Find releases (complex XPath navigation)
    release_list = root.find('.//{http://ddex.net/xml/ern/43}ReleaseList')
    if release_list is not None:
        for release_elem in release_list.findall('.//{http://ddex.net/xml/ern/43}Release'):
            
            # Manual field extraction
            release_id = None
            release_ref = release_elem.find('.//{http://ddex.net/xml/ern/43}ReleaseId')
            if release_ref is not None:
                release_id = release_ref.text
            
            # Title extraction (complex due to multiple title types)
            title = None
            title_elem = release_elem.find('.//{http://ddex.net/xml/ern/43}Title')
            if title_elem is not None:
                title_text = title_elem.find('.//{http://ddex.net/xml/ern/43}TitleText')
                if title_text is not None:
                    title = title_text.text
            
            # Artist extraction (even more complex)
            artist = None
            display_artist = release_elem.find('.//{http://ddex.net/xml/ern/43}DisplayArtist')
            if display_artist is not None:
                party_name = display_artist.find('.//{http://ddex.net/xml/ern/43}PartyName')
                if party_name is not None:
                    full_name = party_name.find('.//{http://ddex.net/xml/ern/43}FullName')
                    if full_name is not None:
                        artist = full_name.text
            
            # Track extraction (very complex)
            tracks = []
            resource_list = release_elem.find('.//{http://ddex.net/xml/ern/43}ResourceList')
            if resource_list is not None:
                for resource in resource_list.findall('.//{http://ddex.net/xml/ern/43}SoundRecording'):
                    # More complex parsing...
                    pass
            
            releases.append({
                'id': release_id,
                'title': title,
                'artist': artist,
                'tracks': tracks
            })
    
    return releases
```

### DDEX Suite Approach (After)

```python
from ddex_parser import DDEXParser

def extract_release_data_ddex(xml_content):
    """New approach: Structured parsing with DDEX Suite"""
    
    parser = DDEXParser()
    result = parser.parse(xml_content)
    
    # Direct access to structured data
    releases = []
    for release in result.flat.releases:
        releases.append({
            'id': release.id,
            'title': release.title,
            'artist': release.artist,
            'tracks': [
                {
                    'id': track.id,
                    'title': track.title,
                    'artist': track.artist,
                    'duration_ms': track.duration_ms,
                    'isrc': track.isrc
                }
                for track in release.tracks
            ]
        })
    
    return releases
```

## Migration Strategies

### Gradual Migration

```typescript
import { DDEXParser } from 'ddex-parser';
import { parseXMLLegacy } from './legacy-parser';

export class MigrationService {
  private parser = new DDEXParser();
  private useNewParser = process.env.USE_NEW_PARSER === 'true';

  async parseWithFallback(xmlContent: string): Promise<any> {
    if (this.useNewParser) {
      try {
        // Try new DDEX Suite parser
        const result = await this.parser.parse(xmlContent);
        
        // Log successful migration
        console.log('Successfully used DDEX Suite parser');
        
        return this.transformToLegacyFormat(result);
        
      } catch (error) {
        console.warn('DDEX Suite parser failed, falling back to legacy:', error.message);
        
        // Fallback to legacy parser
        return parseXMLLegacy(xmlContent);
      }
    } else {
      // Still using legacy parser
      return parseXMLLegacy(xmlContent);
    }
  }

  private transformToLegacyFormat(ddexResult: any): any {
    // Transform DDEX Suite output to match legacy format
    // This allows gradual migration without breaking existing consumers
    
    return {
      releases: ddexResult.flat.releases.map(release => ({
        // Map to legacy field names
        releaseId: release.id,
        albumTitle: release.title,
        artistName: release.artist,
        releaseYear: new Date(release.releaseDate).getFullYear(),
        
        tracks: release.tracks.map(track => ({
          trackId: track.id,
          songTitle: track.title,
          artistName: track.artist,
          durationSeconds: Math.floor(track.durationMs / 1000),
          isrcCode: track.isrc
        }))
      }))
    };
  }
}
```

### Data Validation During Migration

```python
from ddex_parser import DDEXParser
from typing import Dict, List, Any
import logging

class MigrationValidator:
    def __init__(self):
        self.parser = DDEXParser()
        self.validation_errors = []
        self.migration_stats = {
            'total_files': 0,
            'successful_migrations': 0,
            'validation_errors': 0,
            'parsing_errors': 0
        }
    
    def migrate_xml_files(self, xml_files: List[str]) -> Dict[str, Any]:
        """Migrate multiple XML files and validate results"""
        
        results = []
        
        for xml_file in xml_files:
            self.migration_stats['total_files'] += 1
            
            try:
                with open(xml_file, 'r', encoding='utf-8') as f:
                    xml_content = f.read()
                
                # Parse with DDEX Suite
                ddex_result = self.parser.parse(xml_content)
                
                # Validate migration
                validation_result = self.validate_migration(xml_file, ddex_result)
                
                if validation_result['is_valid']:
                    self.migration_stats['successful_migrations'] += 1
                    results.append({
                        'file': xml_file,
                        'status': 'success',
                        'data': ddex_result.flat.to_dict(),
                        'warnings': validation_result.get('warnings', [])
                    })
                else:
                    self.migration_stats['validation_errors'] += 1
                    results.append({
                        'file': xml_file,
                        'status': 'validation_error',
                        'errors': validation_result['errors']
                    })
                
            except Exception as e:
                self.migration_stats['parsing_errors'] += 1
                results.append({
                    'file': xml_file,
                    'status': 'parsing_error',
                    'error': str(e)
                })
                
                logging.error(f"Failed to migrate {xml_file}: {e}")
        
        return {
            'results': results,
            'stats': self.migration_stats,
            'success_rate': self.migration_stats['successful_migrations'] / self.migration_stats['total_files'] * 100
        }
    
    def validate_migration(self, xml_file: str, ddex_result) -> Dict[str, Any]:
        """Validate that migration preserved essential data"""
        
        errors = []
        warnings = []
        
        # Check for required data
        if not ddex_result.flat.releases:
            errors.append("No releases found in migrated data")
        
        for i, release in enumerate(ddex_result.flat.releases):
            release_path = f"releases[{i}]"
            
            # Required fields
            if not release.title:
                errors.append(f"{release_path}: Missing title")
            
            if not release.artist:
                errors.append(f"{release_path}: Missing artist")
            
            # Track validation
            if not release.tracks:
                warnings.append(f"{release_path}: No tracks found")
            else:
                for j, track in enumerate(release.tracks):
                    track_path = f"{release_path}.tracks[{j}]"
                    
                    if not track.title:
                        warnings.append(f"{track_path}: Missing track title")
                    
                    if track.duration_ms and track.duration_ms < 1000:
                        warnings.append(f"{track_path}: Very short duration ({track.duration_ms}ms)")
        
        return {
            'is_valid': len(errors) == 0,
            'errors': errors,
            'warnings': warnings
        }
    
    def generate_migration_report(self, output_file: str = None):
        """Generate detailed migration report"""
        
        report = {
            'migration_summary': self.migration_stats,
            'success_rate': f"{self.migration_stats['successful_migrations'] / self.migration_stats['total_files'] * 100:.1f}%",
            'validation_errors': self.validation_errors,
            'recommendations': self.get_migration_recommendations()
        }
        
        if output_file:
            import json
            with open(output_file, 'w') as f:
                json.dump(report, f, indent=2)
        
        return report
    
    def get_migration_recommendations(self) -> List[str]:
        """Get recommendations based on migration results"""
        
        recommendations = []
        
        success_rate = self.migration_stats['successful_migrations'] / self.migration_stats['total_files'] * 100
        
        if success_rate < 90:
            recommendations.append("Consider reviewing XML files with parsing errors")
        
        if self.migration_stats['validation_errors'] > 0:
            recommendations.append("Address validation errors before proceeding with migration")
        
        if success_rate > 95:
            recommendations.append("Migration looks good - ready to switch to DDEX Suite")
        
        return recommendations

# Usage
validator = MigrationValidator()
xml_files = ['release1.xml', 'release2.xml', 'release3.xml']

migration_result = validator.migrate_xml_files(xml_files)
print(f"Migration success rate: {migration_result['success_rate']:.1f}%")

# Generate report
report = validator.generate_migration_report('migration_report.json')
```

## Common Migration Patterns

### Field Mapping

```typescript
export interface FieldMapping {
  [legacyField: string]: string | ((value: any) => any);
}

export const COMMON_FIELD_MAPPINGS: FieldMapping = {
  // Direct field mappings
  'releaseId': 'id',
  'albumTitle': 'title',
  'artistName': 'artist',
  'releaseYear': (release: any) => new Date(release.releaseDate).getFullYear(),
  
  // Track mappings
  'trackId': 'id',
  'songTitle': 'title',
  'durationSeconds': (track: any) => Math.floor(track.durationMs / 1000),
  'isrcCode': 'isrc',
  
  // Complex mappings
  'genres': (release: any) => release.genre || [],
  'isExplicit': (item: any) => item.parentalWarningType === 'Explicit'
};

export function transformDDEXToLegacy(ddexData: any, mapping: FieldMapping): any {
  function applyMapping(source: any, target: any, mapping: FieldMapping): void {
    for (const [legacyField, ddexField] of Object.entries(mapping)) {
      if (typeof ddexField === 'string') {
        // Simple field mapping
        if (source[ddexField] !== undefined) {
          target[legacyField] = source[ddexField];
        }
      } else if (typeof ddexField === 'function') {
        // Transform function
        target[legacyField] = ddexField(source);
      }
    }
  }

  const result: any = {};
  
  if (ddexData.flat?.releases) {
    result.releases = ddexData.flat.releases.map((release: any) => {
      const mappedRelease: any = {};
      
      applyMapping(release, mappedRelease, {
        'releaseId': 'id',
        'albumTitle': 'title',
        'artistName': 'artist',
        'releaseYear': (r: any) => new Date(r.releaseDate).getFullYear()
      });
      
      if (release.tracks) {
        mappedRelease.tracks = release.tracks.map((track: any) => {
          const mappedTrack: any = {};
          
          applyMapping(track, mappedTrack, {
            'trackId': 'id',
            'songTitle': 'title',
            'artistName': 'artist',
            'durationSeconds': (t: any) => Math.floor(t.durationMs / 1000),
            'isrcCode': 'isrc'
          });
          
          return mappedTrack;
        });
      }
      
      return mappedRelease;
    });
  }
  
  return result;
}
```

## Migration Testing

```python
import unittest
from ddex_parser import DDEXParser

class MigrationTests(unittest.TestCase):
    def setUp(self):
        self.parser = DDEXParser()
        
        # Sample legacy data for comparison
        self.legacy_data = {
            'releaseId': 'R12345',
            'albumTitle': 'Test Album',
            'artistName': 'Test Artist',
            'tracks': [
                {
                    'trackId': 'T001',
                    'songTitle': 'Track 1',
                    'durationSeconds': 180,
                    'isrcCode': 'USRC17607839'
                }
            ]
        }
        
        # Sample XML that should produce equivalent data
        self.sample_xml = """<?xml version="1.0" encoding="UTF-8"?>
        <NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">
            <!-- Sample DDEX XML -->
        </NewReleaseMessage>"""
    
    def test_data_completeness(self):
        """Test that migration preserves all essential data"""
        
        result = self.parser.parse(self.sample_xml)
        ddex_data = result.flat.to_dict()
        
        # Compare essential fields
        self.assertEqual(len(ddex_data['releases']), 1)
        
        release = ddex_data['releases'][0]
        self.assertIsNotNone(release['title'])
        self.assertIsNotNone(release['artist'])
        
        # Check tracks
        self.assertGreater(len(release['tracks']), 0)
        
        track = release['tracks'][0]
        self.assertIsNotNone(track['title'])
        self.assertIsNotNone(track['duration_ms'])
    
    def test_data_accuracy(self):
        """Test that migrated data matches expected values"""
        
        result = self.parser.parse(self.sample_xml)
        
        # Transform to legacy format for comparison
        legacy_equivalent = self.transform_to_legacy(result.flat)
        
        # Compare key fields
        self.assertEqual(
            legacy_equivalent['albumTitle'], 
            self.legacy_data['albumTitle']
        )
        self.assertEqual(
            legacy_equivalent['artistName'], 
            self.legacy_data['artistName']
        )
    
    def test_performance_comparison(self):
        """Test that new parser performs at least as well as legacy"""
        
        import time
        
        # Time DDEX Suite parser
        start_time = time.time()
        for _ in range(100):
            self.parser.parse(self.sample_xml)
        ddex_time = time.time() - start_time
        
        # Time legacy parser (if available)
        # legacy_time = time_legacy_parser(self.sample_xml)
        
        # Assert performance is acceptable
        self.assertLess(ddex_time, 10.0)  # Should complete 100 parses in <10s
    
    def test_error_handling(self):
        """Test that error handling is improved"""
        
        invalid_xml = "<invalid>xml</invalid>"
        
        with self.assertRaises(Exception) as context:
            self.parser.parse(invalid_xml)
        
        # Should provide meaningful error message
        self.assertIn('DDEX', str(context.exception).lower())
    
    def transform_to_legacy(self, flat_data):
        """Transform DDEX flat data to legacy format"""
        
        if not flat_data.releases:
            return {}
        
        release = flat_data.releases[0]
        
        result = {
            'releaseId': release.id,
            'albumTitle': release.title,
            'artistName': release.artist,
            'tracks': []
        }
        
        for track in release.tracks or []:
            result['tracks'].append({
                'trackId': track.id,
                'songTitle': track.title,
                'durationSeconds': track.duration_ms // 1000,
                'isrcCode': track.isrc
            })
        
        return result

if __name__ == '__main__':
    unittest.main()
```

## Migration Checklist

### Pre-Migration

- [ ] Inventory existing XML processing code
- [ ] Identify data transformation requirements
- [ ] Set up DDEX Suite in test environment
- [ ] Create sample test files
- [ ] Define success criteria

### During Migration

- [ ] Implement gradual rollout strategy
- [ ] Set up monitoring and alerts
- [ ] Validate data accuracy
- [ ] Monitor performance impact
- [ ] Collect feedback from users

### Post-Migration

- [ ] Remove legacy XML processing code
- [ ] Update documentation
- [ ] Train team on new approach
- [ ] Monitor for issues
- [ ] Plan next optimization phase

## Best Practices

1. **Gradual Migration**: Use feature flags to gradually migrate functionality
2. **Data Validation**: Thoroughly validate migrated data accuracy
3. **Performance Testing**: Ensure new approach meets performance requirements
4. **Fallback Strategy**: Maintain legacy parser as fallback during transition
5. **Monitoring**: Monitor migration progress and issues
6. **Documentation**: Document migration process and new approach
7. **Training**: Train team on DDEX Suite usage
8. **Testing**: Comprehensive testing before full migration
9. **Rollback Plan**: Have plan to rollback if issues arise
10. **Incremental Approach**: Migrate one component at a time