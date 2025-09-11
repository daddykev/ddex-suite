# Legacy System Migration

Migrate from legacy music metadata systems to DDEX Suite.

## Overview

Legacy system migration involves:
- Migrating from proprietary formats
- Database schema transformations
- API integration updates
- Workflow modernization
- Data quality improvements

## Common Legacy Systems

### CSV/Spreadsheet Migration

```python
import pandas as pd
from ddex_builder import DDEXBuilder
from typing import Dict, Any, List

class CSVToDDEXMigrator:
    def __init__(self):
        self.builder = DDEXBuilder()
        
    def migrate_csv_to_ddex(self, csv_file: str) -> str:
        """Convert CSV metadata to DDEX XML"""
        
        # Read CSV data
        df = pd.read_csv(csv_file)
        
        # Validate required columns
        required_columns = ['album_title', 'artist_name', 'track_title', 'duration']
        missing_columns = [col for col in required_columns if col not in df.columns]
        
        if missing_columns:
            raise ValueError(f"Missing required columns: {missing_columns}")
        
        # Group by album
        releases = self._group_by_album(df)
        
        # Build DDEX data structure
        ddex_data = {
            'releases': releases
        }
        
        # Generate DDEX XML
        return self.builder.build(ddex_data)
    
    def _group_by_album(self, df: pd.DataFrame) -> List[Dict[str, Any]]:
        """Group tracks by album"""
        
        releases = []
        
        # Group by album identifier (title + artist)
        album_groups = df.groupby(['album_title', 'artist_name'])
        
        for (album_title, artist_name), tracks_df in album_groups:
            release = {
                'id': self._generate_release_id(album_title, artist_name),
                'title': album_title,
                'artist': artist_name,
                'release_date': self._extract_release_date(tracks_df),
                'label': self._extract_label(tracks_df),
                'upc': self._extract_upc(tracks_df),
                'tracks': []
            }
            
            # Add tracks
            for idx, track_row in tracks_df.iterrows():
                track = {
                    'id': self._generate_track_id(track_row),
                    'title': track_row['track_title'],
                    'artist': track_row.get('track_artist', artist_name),
                    'duration_ms': self._parse_duration(track_row['duration']),
                    'track_number': track_row.get('track_number', idx + 1),
                    'isrc': track_row.get('isrc'),
                    'genre': self._parse_genre(track_row.get('genre')),
                    'parental_warning_type': self._parse_explicit(track_row.get('explicit'))
                }
                
                release['tracks'].append(track)
            
            releases.append(release)
        
        return releases
    
    def _generate_release_id(self, title: str, artist: str) -> str:
        """Generate unique release ID"""
        import hashlib
        
        combined = f"{title}:{artist}".encode('utf-8')
        hash_obj = hashlib.md5(combined)
        return f"R{hash_obj.hexdigest()[:8].upper()}"
    
    def _generate_track_id(self, track_row) -> str:
        """Generate unique track ID"""
        import hashlib
        
        combined = f"{track_row['track_title']}:{track_row.get('track_artist', '')}".encode('utf-8')
        hash_obj = hashlib.md5(combined)
        return f"T{hash_obj.hexdigest()[:8].upper()}"
    
    def _parse_duration(self, duration_str: str) -> int:
        """Parse duration string to milliseconds"""
        
        if pd.isna(duration_str):
            return 0
        
        # Handle different formats: "3:45", "225", "3m45s"
        duration_str = str(duration_str).strip()
        
        if ':' in duration_str:
            # Format: "3:45" (minutes:seconds)
            parts = duration_str.split(':')
            minutes = int(parts[0])
            seconds = int(parts[1])
            return (minutes * 60 + seconds) * 1000
        
        elif duration_str.isdigit():
            # Format: "225" (seconds)
            return int(duration_str) * 1000
        
        else:
            # Try to extract numbers for formats like "3m45s"
            import re
            matches = re.findall(r'(\d+)[ms]', duration_str.lower())
            
            if len(matches) >= 2:
                minutes = int(matches[0])
                seconds = int(matches[1])
                return (minutes * 60 + seconds) * 1000
        
        return 0
    
    def _extract_release_date(self, tracks_df: pd.DataFrame) -> str:
        """Extract release date from tracks"""
        
        if 'release_date' in tracks_df.columns:
            date_val = tracks_df['release_date'].iloc[0]
            if pd.notna(date_val):
                # Try to parse and normalize date format
                try:
                    import datetime
                    parsed_date = pd.to_datetime(date_val)
                    return parsed_date.strftime('%Y-%m-%d')
                except:
                    pass
        
        # Default to current year if no date found
        import datetime
        return f"{datetime.datetime.now().year}-01-01"
    
    def _extract_label(self, tracks_df: pd.DataFrame) -> str:
        """Extract record label"""
        
        if 'label' in tracks_df.columns:
            label_val = tracks_df['label'].iloc[0]
            if pd.notna(label_val):
                return str(label_val)
        
        return "Unknown Label"
    
    def _extract_upc(self, tracks_df: pd.DataFrame) -> str:
        """Extract UPC code"""
        
        if 'upc' in tracks_df.columns:
            upc_val = tracks_df['upc'].iloc[0]
            if pd.notna(upc_val):
                return str(upc_val)
        
        return None
    
    def _parse_genre(self, genre_str) -> List[str]:
        """Parse genre string to list"""
        
        if pd.isna(genre_str):
            return []
        
        # Handle comma-separated genres
        genres = [g.strip() for g in str(genre_str).split(',')]
        return [g for g in genres if g]
    
    def _parse_explicit(self, explicit_val) -> str:
        """Parse explicit content flag"""
        
        if pd.isna(explicit_val):
            return 'Unknown'
        
        explicit_str = str(explicit_val).lower()
        
        if explicit_str in ['true', '1', 'yes', 'explicit']:
            return 'Explicit'
        elif explicit_str in ['false', '0', 'no', 'clean']:
            return 'NotExplicit'
        
        return 'Unknown'

# Usage example
migrator = CSVToDDEXMigrator()

# Convert CSV to DDEX
ddex_xml = migrator.migrate_csv_to_ddex('legacy_catalog.csv')

# Save DDEX output
with open('converted_catalog.xml', 'w') as f:
    f.write(ddex_xml)

print("Successfully migrated CSV to DDEX format")
```

### Database Migration

```typescript
import { DDEXBuilder } from 'ddex-builder';
import { Pool } from 'pg';

export class DatabaseToDDEXMigrator {
  private pool: Pool;
  private builder: DDEXBuilder;

  constructor(connectionConfig: any) {
    this.pool = new Pool(connectionConfig);
    this.builder = new DDEXBuilder();
  }

  async migrateFromDatabase(): Promise<void> {
    const client = await this.pool.connect();
    
    try {
      // Migrate albums/releases
      const releases = await this.fetchReleases(client);
      
      for (const release of releases) {
        const ddexData = await this.buildDDEXFromRelease(client, release);
        const ddexXml = await this.builder.build(ddexData);
        
        // Save or process DDEX XML
        await this.saveDDEXFile(release.id, ddexXml);
      }
      
    } finally {
      client.release();
    }
  }

  private async fetchReleases(client: any): Promise<any[]> {
    const query = `
      SELECT 
        r.id,
        r.title,
        r.release_date,
        r.label_id,
        r.upc_code,
        l.name as label_name,
        a.name as artist_name
      FROM releases r
      LEFT JOIN labels l ON r.label_id = l.id  
      LEFT JOIN artists a ON r.primary_artist_id = a.id
      ORDER BY r.created_at
    `;

    const result = await client.query(query);
    return result.rows;
  }

  private async buildDDEXFromRelease(client: any, release: any): Promise<any> {
    // Fetch tracks for this release
    const tracks = await this.fetchTracksForRelease(client, release.id);
    
    // Fetch additional metadata
    const genres = await this.fetchGenresForRelease(client, release.id);
    const territories = await this.fetchTerritoriesForRelease(client, release.id);

    return {
      releases: [{
        id: `R${release.id.toString().padStart(6, '0')}`,
        title: release.title,
        artist: release.artist_name,
        release_date: release.release_date,
        label: release.label_name,
        upc: release.upc_code,
        genre: genres,
        territory_codes: territories,
        tracks: tracks.map((track, index) => ({
          id: `T${track.id.toString().padStart(6, '0')}`,
          title: track.title,
          artist: track.artist_name || release.artist_name,
          duration_ms: track.duration_seconds * 1000,
          track_number: track.track_number || index + 1,
          isrc: track.isrc_code,
          parental_warning_type: this.mapExplicitFlag(track.is_explicit),
          genre: track.genre ? [track.genre] : []
        }))
      }]
    };
  }

  private async fetchTracksForRelease(client: any, releaseId: number): Promise<any[]> {
    const query = `
      SELECT 
        t.id,
        t.title,
        t.duration_seconds,
        t.track_number,
        t.isrc_code,
        t.is_explicit,
        t.genre,
        a.name as artist_name
      FROM tracks t
      LEFT JOIN artists a ON t.artist_id = a.id
      WHERE t.release_id = $1
      ORDER BY t.track_number
    `;

    const result = await client.query(query, [releaseId]);
    return result.rows;
  }

  private async fetchGenresForRelease(client: any, releaseId: number): Promise<string[]> {
    const query = `
      SELECT g.name
      FROM release_genres rg
      JOIN genres g ON rg.genre_id = g.id
      WHERE rg.release_id = $1
    `;

    const result = await client.query(query, [releaseId]);
    return result.rows.map(row => row.name);
  }

  private async fetchTerritoriesForRelease(client: any, releaseId: number): Promise<string[]> {
    const query = `
      SELECT t.code
      FROM release_territories rt
      JOIN territories t ON rt.territory_id = t.id
      WHERE rt.release_id = $1
    `;

    const result = await client.query(query, [releaseId]);
    return result.rows.map(row => row.code);
  }

  private mapExplicitFlag(isExplicit: boolean | null): string {
    if (isExplicit === true) return 'Explicit';
    if (isExplicit === false) return 'NotExplicit';
    return 'Unknown';
  }

  private async saveDDEXFile(releaseId: number, ddexXml: string): Promise<void> {
    const fs = require('fs').promises;
    const filename = `release_${releaseId}_ddex.xml`;
    await fs.writeFile(`./output/${filename}`, ddexXml, 'utf8');
  }

  async close(): Promise<void> {
    await this.pool.end();
  }
}

// Usage
const migrator = new DatabaseToDDEXMigrator({
  host: 'localhost',
  port: 5432,
  database: 'legacy_music_db',
  user: 'username',
  password: 'password'
});

await migrator.migrateFromDatabase();
await migrator.close();
```

## API Migration Strategy

### Legacy API Wrapper

```python
from flask import Flask, request, jsonify
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder

class LegacyAPIAdapter:
    """Adapter to maintain legacy API compatibility while using DDEX Suite internally"""
    
    def __init__(self):
        self.parser = DDEXParser()
        self.builder = DDEXBuilder()
        self.app = Flask(__name__)
        self._setup_routes()
    
    def _setup_routes(self):
        """Setup legacy API endpoints"""
        
        @self.app.route('/api/v1/albums', methods=['POST'])
        def create_album():
            """Legacy endpoint that now uses DDEX internally"""
            
            legacy_data = request.json
            
            try:
                # Convert legacy format to DDEX
                ddex_data = self._convert_legacy_to_ddex(legacy_data)
                
                # Use DDEX Suite for processing
                ddex_xml = self.builder.build(ddex_data)
                
                # Parse back for validation
                parsed = self.parser.parse(ddex_xml)
                
                # Convert back to legacy format for response
                response_data = self._convert_ddex_to_legacy(parsed.flat)
                
                return jsonify({
                    'status': 'success',
                    'data': response_data
                })
                
            except Exception as e:
                return jsonify({
                    'status': 'error',
                    'message': str(e)
                }), 400
        
        @self.app.route('/api/v1/albums/<album_id>', methods=['GET'])
        def get_album(album_id):
            """Legacy endpoint to retrieve album data"""
            
            try:
                # In real implementation, load from database/storage
                ddex_xml = self._load_ddex_for_album(album_id)
                
                if not ddex_xml:
                    return jsonify({'error': 'Album not found'}), 404
                
                # Parse with DDEX Suite
                parsed = self.parser.parse(ddex_xml)
                
                # Convert to legacy format
                legacy_data = self._convert_ddex_to_legacy(parsed.flat)
                
                return jsonify(legacy_data)
                
            except Exception as e:
                return jsonify({'error': str(e)}), 500
    
    def _convert_legacy_to_ddex(self, legacy_data: dict) -> dict:
        """Convert legacy API format to DDEX format"""
        
        ddex_release = {
            'id': legacy_data.get('album_id', f"R{hash(legacy_data['title']) % 100000:05d}"),
            'title': legacy_data['title'],
            'artist': legacy_data['artist'],
            'release_date': legacy_data.get('release_date'),
            'label': legacy_data.get('label'),
            'upc': legacy_data.get('upc'),
            'genre': legacy_data.get('genres', []),
            'tracks': []
        }
        
        # Convert tracks
        for track_data in legacy_data.get('songs', []):
            ddex_track = {
                'id': track_data.get('song_id', f"T{hash(track_data['title']) % 100000:05d}"),
                'title': track_data['title'],
                'artist': track_data.get('artist', legacy_data['artist']),
                'duration_ms': track_data.get('duration_seconds', 0) * 1000,
                'track_number': track_data.get('track_number'),
                'isrc': track_data.get('isrc'),
                'parental_warning_type': 'Explicit' if track_data.get('explicit') else 'NotExplicit'
            }
            
            ddex_release['tracks'].append(ddex_track)
        
        return {'releases': [ddex_release]}
    
    def _convert_ddex_to_legacy(self, ddex_flat) -> dict:
        """Convert DDEX flat format back to legacy API format"""
        
        if not ddex_flat.releases:
            return {}
        
        release = ddex_flat.releases[0]
        
        legacy_data = {
            'album_id': release.id,
            'title': release.title,
            'artist': release.artist,
            'release_date': release.release_date,
            'label': release.label,
            'upc': release.upc,
            'genres': release.genre or [],
            'songs': []
        }
        
        # Convert tracks back to legacy format
        for track in release.tracks or []:
            legacy_track = {
                'song_id': track.id,
                'title': track.title,
                'artist': track.artist,
                'duration_seconds': track.duration_ms // 1000 if track.duration_ms else 0,
                'track_number': track.track_number,
                'isrc': track.isrc,
                'explicit': track.parental_warning_type == 'Explicit'
            }
            
            legacy_data['songs'].append(legacy_track)
        
        return legacy_data
    
    def _load_ddex_for_album(self, album_id: str) -> str:
        """Load DDEX XML for album (implementation depends on storage)"""
        
        # This would load from your storage system
        # For demo purposes, return None
        return None
    
    def run(self, host='0.0.0.0', port=5000, debug=False):
        """Run the legacy API server"""
        self.app.run(host=host, port=port, debug=debug)

# Usage
adapter = LegacyAPIAdapter()
adapter.run(debug=True)
```

## Migration Validation

```python
import json
from typing import Dict, List, Any
from dataclasses import dataclass

@dataclass
class MigrationValidationResult:
    is_valid: bool
    errors: List[str]
    warnings: List[str]
    data_quality_score: float
    missing_fields: List[str]
    recommendations: List[str]

class MigrationValidator:
    """Validate migrated data quality and completeness"""
    
    def __init__(self):
        self.required_fields = {
            'release': ['id', 'title', 'artist'],
            'track': ['id', 'title', 'duration_ms']
        }
        
        self.recommended_fields = {
            'release': ['release_date', 'label', 'upc', 'genre'],
            'track': ['track_number', 'isrc', 'artist']
        }
    
    def validate_migration(self, 
                          original_data: Dict[str, Any], 
                          migrated_data: Dict[str, Any]) -> MigrationValidationResult:
        """Comprehensive migration validation"""
        
        errors = []
        warnings = []
        missing_fields = []
        
        # Data completeness check
        completeness_score = self._check_data_completeness(
            original_data, migrated_data, errors, warnings, missing_fields
        )
        
        # Data accuracy check
        accuracy_score = self._check_data_accuracy(
            original_data, migrated_data, errors, warnings
        )
        
        # Data quality check
        quality_score = self._check_data_quality(migrated_data, warnings)
        
        # Overall score
        overall_score = (completeness_score + accuracy_score + quality_score) / 3
        
        # Generate recommendations
        recommendations = self._generate_recommendations(
            errors, warnings, missing_fields, overall_score
        )
        
        return MigrationValidationResult(
            is_valid=len(errors) == 0,
            errors=errors,
            warnings=warnings,
            data_quality_score=overall_score,
            missing_fields=missing_fields,
            recommendations=recommendations
        )
    
    def _check_data_completeness(self, original, migrated, errors, warnings, missing_fields):
        """Check if all essential data was migrated"""
        
        score = 100.0
        
        # Check releases count
        orig_releases = len(original.get('releases', []))
        migr_releases = len(migrated.get('releases', []))
        
        if migr_releases < orig_releases:
            errors.append(f"Missing releases: {orig_releases - migr_releases}")
            score -= 20
        
        # Check tracks for each release
        for i, release in enumerate(migrated.get('releases', [])):
            release_path = f"releases[{i}]"
            
            # Required fields
            for field in self.required_fields['release']:
                if not release.get(field):
                    missing_fields.append(f"{release_path}.{field}")
                    score -= 10
            
            # Recommended fields
            for field in self.recommended_fields['release']:
                if not release.get(field):
                    warnings.append(f"Missing recommended field: {release_path}.{field}")
                    score -= 2
            
            # Track completeness
            orig_tracks = self._get_original_track_count(original, i)
            migr_tracks = len(release.get('tracks', []))
            
            if migr_tracks < orig_tracks:
                errors.append(f"{release_path}: Missing tracks ({orig_tracks - migr_tracks})")
                score -= 15
        
        return max(0, score)
    
    def _check_data_accuracy(self, original, migrated, errors, warnings):
        """Check if migrated data matches original data"""
        
        score = 100.0
        
        # Compare key fields for accuracy
        for i, (orig_rel, migr_rel) in enumerate(zip(
            original.get('releases', []),
            migrated.get('releases', [])
        )):
            release_path = f"releases[{i}]"
            
            # Title comparison
            if orig_rel.get('title') != migr_rel.get('title'):
                errors.append(f"{release_path}: Title mismatch")
                score -= 15
            
            # Artist comparison
            if orig_rel.get('artist') != migr_rel.get('artist'):
                errors.append(f"{release_path}: Artist mismatch")
                score -= 15
            
            # Track comparison
            for j, (orig_track, migr_track) in enumerate(zip(
                orig_rel.get('tracks', []),
                migr_rel.get('tracks', [])
            )):
                track_path = f"{release_path}.tracks[{j}]"
                
                if orig_track.get('title') != migr_track.get('title'):
                    warnings.append(f"{track_path}: Track title mismatch")
                    score -= 5
        
        return max(0, score)
    
    def _check_data_quality(self, migrated, warnings):
        """Check overall data quality"""
        
        score = 100.0
        
        for i, release in enumerate(migrated.get('releases', [])):
            release_path = f"releases[{i}]"
            
            # Check for suspiciously short titles
            if release.get('title') and len(release['title']) < 3:
                warnings.append(f"{release_path}: Very short title")
                score -= 2
            
            # Check for missing ISRCs
            tracks_with_isrc = sum(1 for track in release.get('tracks', []) 
                                 if track.get('isrc'))
            total_tracks = len(release.get('tracks', []))
            
            if total_tracks > 0:
                isrc_percentage = tracks_with_isrc / total_tracks
                if isrc_percentage < 0.5:
                    warnings.append(f"{release_path}: Low ISRC coverage ({isrc_percentage:.1%})")
                    score -= 10
            
            # Check track duration reasonableness
            for j, track in enumerate(release.get('tracks', [])):
                track_path = f"{release_path}.tracks[{j}]"
                duration = track.get('duration_ms', 0)
                
                if duration > 0 and (duration < 30000 or duration > 1800000):  # 30s - 30min
                    warnings.append(f"{track_path}: Unusual duration ({duration/1000:.1f}s)")
                    score -= 1
        
        return max(0, score)
    
    def _get_original_track_count(self, original, release_index):
        """Get track count from original data"""
        
        if release_index < len(original.get('releases', [])):
            return len(original['releases'][release_index].get('tracks', []))
        
        return 0
    
    def _generate_recommendations(self, errors, warnings, missing_fields, score):
        """Generate migration recommendations"""
        
        recommendations = []
        
        if score < 70:
            recommendations.append("Migration quality is below acceptable threshold - consider review")
        
        if errors:
            recommendations.append("Address all errors before proceeding with migration")
        
        if len(missing_fields) > 10:
            recommendations.append("High number of missing fields - review data mapping")
        
        if len(warnings) > 20:
            recommendations.append("Many warnings detected - review data quality")
        
        if score > 90:
            recommendations.append("Migration looks good - ready for production use")
        
        return recommendations

# Usage example
validator = MigrationValidator()

# Load original and migrated data for comparison
with open('original_data.json', 'r') as f:
    original = json.load(f)

with open('migrated_data.json', 'r') as f:
    migrated = json.load(f)

# Validate migration
result = validator.validate_migration(original, migrated)

print(f"Migration Valid: {result.is_valid}")
print(f"Quality Score: {result.data_quality_score:.1f}%")

for error in result.errors:
    print(f"ERROR: {error}")

for warning in result.warnings:
    print(f"WARNING: {warning}")

for rec in result.recommendations:
    print(f"RECOMMENDATION: {rec}")
```

## Best Practices

1. **Incremental Migration**: Migrate data in small batches to identify issues early
2. **Data Validation**: Thoroughly validate migrated data against original
3. **Backup Strategy**: Always backup original data before migration
4. **Schema Mapping**: Document field mappings between legacy and DDEX formats
5. **Quality Assurance**: Implement comprehensive quality checks
6. **Testing**: Test migration process with representative sample data
7. **Rollback Plan**: Have a clear rollback strategy if migration fails
8. **Documentation**: Document migration process and decisions
9. **Performance Monitoring**: Monitor migration performance and resource usage
10. **User Communication**: Keep stakeholders informed of migration progress and impacts