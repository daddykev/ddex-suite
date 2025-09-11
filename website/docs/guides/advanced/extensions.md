# Extensions

Extend DDEX Suite functionality with custom parsers, builders, and processing plugins.

## Overview

DDEX Suite supports extensions through:
- Custom data transformers
- Plugin architecture
- Hook system for processing pipeline
- Custom output formats
- Integration middleware

## Custom Data Transformers

### Transform Pipeline

```typescript
import { DataTransformer, TransformContext } from 'ddex-suite';

export interface CustomTransformer extends DataTransformer {
  transform(data: any, context: TransformContext): any;
  reverse?(data: any, context: TransformContext): any;
}

// Custom transformer for music streaming platforms
export class StreamingPlatformTransformer implements CustomTransformer {
  name = 'streaming-platform';
  version = '1.0.0';

  transform(ddexData: any, context: TransformContext) {
    return {
      releases: ddexData.flat.releases.map(release => ({
        id: release.id,
        title: release.title,
        artist: release.artist,
        albumArt: this.extractAlbumArt(release),
        tracks: release.tracks?.map(track => ({
          id: track.id,
          title: track.title,
          duration: track.durationMs,
          streamingUrl: this.generateStreamingUrl(track, context.platform),
          metadata: {
            isrc: track.isrc,
            explicitContent: track.parentalWarningType === 'Explicit'
          }
        })) || []
      }))
    };
  }

  private extractAlbumArt(release: any) {
    return release.resources?.find((r: any) => r.type === 'Image')?.url;
  }

  private generateStreamingUrl(track: any, platform: string) {
    // Platform-specific URL generation logic
    return `https://${platform}.com/stream/${track.isrc}`;
  }
}
```

### Data Format Extensions

```python
from ddex_suite import DataFormatExtension, FormatContext
from typing import Dict, Any
import pandas as pd

class DataFrameExtension(DataFormatExtension):
    """Convert DDEX data to DataFrame format for analytics"""
    
    name = "dataframe"
    description = "Convert to pandas DataFrame"
    
    def export(self, ddex_data: Dict[str, Any], context: FormatContext) -> pd.DataFrame:
        """Convert DDEX flat data to DataFrame"""
        
        rows = []
        
        for release in ddex_data.get('flat', {}).get('releases', []):
            for track in release.get('tracks', []):
                row = {
                    'release_id': release.get('id'),
                    'release_title': release.get('title'),
                    'release_artist': release.get('artist'),
                    'release_date': release.get('release_date'),
                    'track_id': track.get('id'),
                    'track_title': track.get('title'),
                    'track_artist': track.get('artist'),
                    'isrc': track.get('isrc'),
                    'duration_ms': track.get('duration_ms'),
                    'track_number': track.get('track_number'),
                    'explicit': track.get('parental_warning_type') == 'Explicit'
                }
                rows.append(row)
        
        return pd.DataFrame(rows)
    
    def import_data(self, df: pd.DataFrame, context: FormatContext) -> Dict[str, Any]:
        """Convert DataFrame back to DDEX format"""
        
        releases = {}
        
        for _, row in df.iterrows():
            release_id = row['release_id']
            
            if release_id not in releases:
                releases[release_id] = {
                    'id': release_id,
                    'title': row['release_title'],
                    'artist': row['release_artist'],
                    'release_date': row['release_date'],
                    'tracks': []
                }
            
            track = {
                'id': row['track_id'],
                'title': row['track_title'],
                'artist': row['track_artist'],
                'isrc': row['isrc'],
                'duration_ms': row['duration_ms'],
                'track_number': row['track_number'],
                'parental_warning_type': 'Explicit' if row['explicit'] else 'NotExplicit'
            }
            
            releases[release_id]['tracks'].append(track)
        
        return {
            'releases': list(releases.values())
        }

# Usage
from ddex_parser import DDEXParser
from ddex_suite.extensions import register_extension

# Register the extension
register_extension(DataFrameExtension())

# Use the extension
parser = DDEXParser()
result = parser.parse(xml_content)

# Export to DataFrame
df = result.export('dataframe')
print(df.head())

# Modify and import back
df['explicit'] = False  # Mark all as non-explicit
modified_data = result.import_from('dataframe', df)
```

## Plugin Architecture

### Plugin Interface

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait DDEXPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), PluginError>;
    fn process(&self, data: &mut DDEXData, context: &ProcessContext) -> Result<(), PluginError>;
    fn finalize(&self, context: &ProcessContext) -> Result<(), PluginError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub options: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct ProcessContext {
    pub operation: String,
    pub metadata: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Example plugin: Metadata enrichment
pub struct MetadataEnrichmentPlugin {
    api_key: String,
    cache: HashMap<String, EnrichmentData>,
}

impl DDEXPlugin for MetadataEnrichmentPlugin {
    fn name(&self) -> &str {
        "metadata-enrichment"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn initialize(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        if let Some(api_key) = config.options.get("api_key") {
            self.api_key = api_key.as_str()
                .ok_or(PluginError::ConfigError("api_key must be string".into()))?
                .to_string();
        }
        Ok(())
    }

    fn process(&self, data: &mut DDEXData, _context: &ProcessContext) -> Result<(), PluginError> {
        for release in &mut data.releases {
            if let Some(enrichment) = self.enrich_release(&release.artist, &release.title)? {
                release.genre = Some(enrichment.genres);
                release.additional_metadata = Some(enrichment.metadata);
            }
        }
        Ok(())
    }

    fn finalize(&self, _context: &ProcessContext) -> Result<(), PluginError> {
        // Cleanup or final processing
        Ok(())
    }
}

impl MetadataEnrichmentPlugin {
    fn enrich_release(&self, artist: &str, title: &str) -> Result<Option<EnrichmentData>, PluginError> {
        let cache_key = format!("{}:{}", artist, title);
        
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(Some(cached.clone()));
        }

        // Call external API for enrichment
        // Implementation details...
        Ok(None)
    }
}

#[derive(Clone, Debug)]
struct EnrichmentData {
    genres: Vec<String>,
    metadata: HashMap<String, String>,
}
```

### Plugin Registry

```typescript
export class PluginRegistry {
  private plugins: Map<string, Plugin> = new Map();
  private hooks: Map<string, Plugin[]> = new Map();

  register(plugin: Plugin): void {
    this.plugins.set(plugin.name, plugin);
    
    // Register hooks
    if (plugin.hooks) {
      plugin.hooks.forEach(hookName => {
        if (!this.hooks.has(hookName)) {
          this.hooks.set(hookName, []);
        }
        this.hooks.get(hookName)!.push(plugin);
      });
    }
  }

  async executeHook(hookName: string, data: any, context: any): Promise<any> {
    const hookPlugins = this.hooks.get(hookName) || [];
    
    let result = data;
    for (const plugin of hookPlugins) {
      if (plugin[hookName]) {
        result = await plugin[hookName](result, context);
      }
    }
    
    return result;
  }

  getPlugin(name: string): Plugin | undefined {
    return this.plugins.get(name);
  }

  listPlugins(): Plugin[] {
    return Array.from(this.plugins.values());
  }
}

// Usage
import { PluginRegistry } from 'ddex-suite/plugins';
import { MetadataEnricherPlugin } from './plugins/metadata-enricher';
import { ValidationPlugin } from './plugins/validation';

const registry = new PluginRegistry();

registry.register(new MetadataEnricherPlugin({
  apiKey: process.env.METADATA_API_KEY,
  cacheSize: 1000
}));

registry.register(new ValidationPlugin({
  strictMode: true,
  customRules: ['business-logic', 'partner-specific']
}));

// Process with plugins
const processedData = await registry.executeHook('beforeParse', inputData, context);
```

## Custom Output Formats

### JSON-LD Extension

```python
from ddex_suite import OutputExtension, OutputContext
from typing import Dict, Any
import json

class JSONLDExtension(OutputExtension):
    """Export DDEX data as JSON-LD for semantic web"""
    
    name = "jsonld"
    mime_type = "application/ld+json"
    file_extension = ".jsonld"
    
    def export(self, ddex_data: Dict[str, Any], context: OutputContext) -> str:
        """Convert DDEX data to JSON-LD format"""
        
        jsonld = {
            "@context": {
                "@vocab": "https://schema.org/",
                "ddex": "https://ddex.net/schema/",
                "musicbrainz": "https://musicbrainz.org/doc/",
            },
            "@type": "MusicAlbum",
            "@id": f"ddex:release/{ddex_data['releases'][0]['id']}",
        }
        
        release = ddex_data['releases'][0]
        
        # Basic release information
        jsonld.update({
            "name": release['title'],
            "byArtist": {
                "@type": "MusicGroup",
                "name": release['artist']
            },
            "datePublished": release.get('release_date'),
            "recordLabel": release.get('label'),
        })
        
        # Tracks as part of the album
        if 'tracks' in release:
            jsonld["track"] = []
            for i, track in enumerate(release['tracks']):
                track_ld = {
                    "@type": "MusicRecording",
                    "@id": f"ddex:track/{track['id']}",
                    "name": track['title'],
                    "byArtist": {
                        "@type": "MusicGroup", 
                        "name": track['artist']
                    },
                    "position": track.get('track_number', i + 1),
                    "duration": f"PT{track.get('duration_ms', 0) / 1000}S",
                }
                
                if track.get('isrc'):
                    track_ld["musicbrainz:isrc"] = track['isrc']
                
                jsonld["track"].append(track_ld)
        
        return json.dumps(jsonld, indent=2)

# Register and use
from ddex_builder import DDEXBuilder
from ddex_suite.extensions import register_output_extension

register_output_extension(JSONLDExtension())

builder = DDEXBuilder()
xml = builder.build(data)

# Export as JSON-LD
jsonld_output = builder.export(xml, format='jsonld')
```

### CSV Export Extension

```typescript
export class CSVExtension implements OutputExtension {
  name = 'csv';
  mimeType = 'text/csv';
  fileExtension = '.csv';

  export(ddexData: any, context: OutputContext): string {
    const rows: string[][] = [];
    
    // Header row
    rows.push([
      'Release ID', 'Release Title', 'Release Artist', 'Release Date',
      'Track ID', 'Track Title', 'Track Artist', 'ISRC', 
      'Duration (ms)', 'Track Number', 'Explicit'
    ]);

    // Data rows
    for (const release of ddexData.releases) {
      for (const track of release.tracks || []) {
        rows.push([
          release.id || '',
          release.title || '',
          release.artist || '',
          release.releaseDate || '',
          track.id || '',
          track.title || '',
          track.artist || '',
          track.isrc || '',
          String(track.durationMs || ''),
          String(track.trackNumber || ''),
          track.parentalWarningType === 'Explicit' ? 'Yes' : 'No'
        ]);
      }
    }

    // Convert to CSV string
    return rows.map(row => 
      row.map(cell => 
        cell.includes(',') || cell.includes('"') || cell.includes('\n')
          ? `"${cell.replace(/"/g, '""')}"`
          : cell
      ).join(',')
    ).join('\n');
  }

  import(csvData: string, context: OutputContext): any {
    const lines = csvData.split('\n');
    const headers = this.parseCSVLine(lines[0]);
    
    const releases: { [key: string]: any } = {};
    
    for (let i = 1; i < lines.length; i++) {
      if (!lines[i].trim()) continue;
      
      const values = this.parseCSVLine(lines[i]);
      const row: { [key: string]: string } = {};
      
      headers.forEach((header, index) => {
        row[header] = values[index] || '';
      });

      const releaseId = row['Release ID'];
      if (!releases[releaseId]) {
        releases[releaseId] = {
          id: releaseId,
          title: row['Release Title'],
          artist: row['Release Artist'],
          releaseDate: row['Release Date'],
          tracks: []
        };
      }

      releases[releaseId].tracks.push({
        id: row['Track ID'],
        title: row['Track Title'],
        artist: row['Track Artist'],
        isrc: row['ISRC'],
        durationMs: parseInt(row['Duration (ms)']) || 0,
        trackNumber: parseInt(row['Track Number']) || 0,
        parentalWarningType: row['Explicit'] === 'Yes' ? 'Explicit' : 'NotExplicit'
      });
    }

    return { releases: Object.values(releases) };
  }

  private parseCSVLine(line: string): string[] {
    const result: string[] = [];
    let current = '';
    let inQuotes = false;
    
    for (let i = 0; i < line.length; i++) {
      const char = line[i];
      
      if (char === '"') {
        if (inQuotes && line[i + 1] === '"') {
          current += '"';
          i++; // Skip next quote
        } else {
          inQuotes = !inQuotes;
        }
      } else if (char === ',' && !inQuotes) {
        result.push(current);
        current = '';
      } else {
        current += char;
      }
    }
    
    result.push(current);
    return result;
  }
}
```

## Integration Middleware

```typescript
export interface Middleware {
  name: string;
  process(data: any, context: MiddlewareContext, next: NextFunction): Promise<any>;
}

export class MiddlewareChain {
  private middlewares: Middleware[] = [];

  use(middleware: Middleware): void {
    this.middlewares.push(middleware);
  }

  async execute(data: any, context: MiddlewareContext): Promise<any> {
    let index = 0;
    
    const next: NextFunction = async () => {
      if (index >= this.middlewares.length) {
        return data;
      }
      
      const middleware = this.middlewares[index++];
      return await middleware.process(data, context, next);
    };
    
    return await next();
  }
}

// Example middleware implementations
export class LoggingMiddleware implements Middleware {
  name = 'logging';

  async process(data: any, context: MiddlewareContext, next: NextFunction): Promise<any> {
    console.log(`Processing ${context.operation} at ${new Date().toISOString()}`);
    
    const start = Date.now();
    const result = await next();
    const duration = Date.now() - start;
    
    console.log(`Completed ${context.operation} in ${duration}ms`);
    
    return result;
  }
}

export class CachingMiddleware implements Middleware {
  name = 'caching';
  private cache = new Map<string, any>();

  async process(data: any, context: MiddlewareContext, next: NextFunction): Promise<any> {
    const cacheKey = this.generateCacheKey(data, context);
    
    if (this.cache.has(cacheKey)) {
      console.log('Cache hit');
      return this.cache.get(cacheKey);
    }
    
    const result = await next();
    this.cache.set(cacheKey, result);
    
    return result;
  }

  private generateCacheKey(data: any, context: MiddlewareContext): string {
    return `${context.operation}:${JSON.stringify(data).slice(0, 100)}`;
  }
}

// Usage
const middleware = new MiddlewareChain();
middleware.use(new LoggingMiddleware());
middleware.use(new CachingMiddleware());

// Apply middleware to processing pipeline
const processedData = await middleware.execute(inputData, {
  operation: 'parse',
  userId: 'user123',
  timestamp: new Date()
});
```

## Extension Configuration

```yaml
# ddex-extensions.yml
extensions:
  transformers:
    - name: streaming-platform
      enabled: true
      config:
        platforms: ['spotify', 'apple', 'youtube']
        
    - name: analytics-format
      enabled: true
      config:
        include_metadata: true
        flatten_structure: true

  plugins:
    - name: metadata-enrichment
      enabled: true
      config:
        api_key: "${METADATA_API_KEY}"
        cache_size: 1000
        timeout_ms: 5000
        
    - name: custom-validation
      enabled: true
      config:
        strict_mode: true
        custom_rules:
          - business-logic
          - partner-specific

  outputs:
    - name: jsonld
      enabled: true
      config:
        context_url: "https://schema.org"
        
    - name: csv
      enabled: true
      config:
        delimiter: ","
        include_header: true

  middleware:
    - name: logging
      enabled: true
      config:
        level: "info"
        
    - name: caching  
      enabled: true
      config:
        max_size: 100
        ttl_seconds: 3600
```

## Best Practices

1. **Plugin Interface**: Define clear, consistent interfaces
2. **Error Handling**: Implement robust error handling in extensions
3. **Performance**: Optimize extensions for large datasets
4. **Configuration**: Make extensions configurable and flexible
5. **Testing**: Thoroughly test custom extensions
6. **Documentation**: Document extension APIs and usage
7. **Versioning**: Version extensions for compatibility
8. **Security**: Validate inputs in custom extensions
9. **Memory Management**: Handle memory efficiently in plugins
10. **Backwards Compatibility**: Maintain compatibility across versions