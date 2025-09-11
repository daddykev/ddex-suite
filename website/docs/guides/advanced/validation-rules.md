# Validation Rules

Advanced validation rule configuration and custom rule development for DDEX Suite.

## Overview

The DDEX Suite validation system is built on extensible rules that can be:
- Configured for different strictness levels
- Extended with custom business logic
- Adapted for partner-specific requirements
- Combined into validation profiles

## Built-in Validation Rules

### Core Schema Rules

```typescript
import { ValidationRule, ValidationContext, ValidationError } from 'ddex-builder';

// Required field validation
export class RequiredFieldRule extends ValidationRule {
  name = 'required-fields';
  description = 'Validates required fields are present';

  validate(data: any, context: ValidationContext): ValidationError[] {
    const errors: ValidationError[] = [];
    
    // Release-level required fields
    if (data.releases) {
      data.releases.forEach((release: any, index: number) => {
        const path = `releases[${index}]`;
        
        if (!release.title) {
          errors.push({
            path: `${path}.title`,
            message: 'Release title is required',
            severity: 'error',
            code: 'MISSING_REQUIRED_FIELD'
          });
        }
        
        if (!release.artist) {
          errors.push({
            path: `${path}.artist`,
            message: 'Release artist is required',
            severity: 'error',
            code: 'MISSING_REQUIRED_FIELD'
          });
        }
      });
    }
    
    return errors;
  }
}
```

### Data Type Validation

```python
from ddex_builder import ValidationRule, ValidationContext
from typing import List, Any, Dict

class DataTypeRule(ValidationRule):
    name = "data-types"
    description = "Validates data types are correct"
    
    def validate(self, data: Dict[str, Any], context: ValidationContext) -> List[Dict]:
        errors = []
        
        if 'releases' in data:
            for i, release in enumerate(data['releases']):
                path = f"releases[{i}]"
                
                # String fields
                string_fields = ['title', 'artist', 'label', 'id']
                for field in string_fields:
                    if field in release and not isinstance(release[field], str):
                        errors.append({
                            'path': f"{path}.{field}",
                            'message': f"{field} must be a string, got {type(release[field]).__name__}",
                            'severity': 'error',
                            'code': 'INVALID_DATA_TYPE'
                        })
                
                # Date fields
                if 'release_date' in release:
                    if not self._is_valid_date(release['release_date']):
                        errors.append({
                            'path': f"{path}.release_date",
                            'message': "release_date must be a valid ISO date",
                            'severity': 'error',
                            'code': 'INVALID_DATE_FORMAT'
                        })
                
                # Numeric fields in tracks
                if 'tracks' in release:
                    for j, track in enumerate(release['tracks']):
                        track_path = f"{path}.tracks[{j}]"
                        
                        if 'duration_ms' in track:
                            if not isinstance(track['duration_ms'], int):
                                errors.append({
                                    'path': f"{track_path}.duration_ms",
                                    'message': "duration_ms must be an integer",
                                    'severity': 'error',
                                    'code': 'INVALID_DATA_TYPE'
                                })
        
        return errors
    
    def _is_valid_date(self, date_str: str) -> bool:
        from datetime import datetime
        try:
            datetime.fromisoformat(date_str.replace('Z', '+00:00'))
            return True
        except ValueError:
            return False
```

### Format Validation Rules

```rust
use regex::Regex;
use serde_json::Value;

pub struct ISRCFormatRule {
    isrc_regex: Regex,
}

impl ISRCFormatRule {
    pub fn new() -> Self {
        Self {
            isrc_regex: Regex::new(r"^[A-Z]{2}[A-Z0-9]{3}[0-9]{7}$").unwrap(),
        }
    }
}

impl ValidationRule for ISRCFormatRule {
    fn name(&self) -> &str {
        "isrc-format"
    }

    fn description(&self) -> &str {
        "Validates ISRC format compliance"
    }

    fn validate(&self, data: &Value, _context: &ValidationContext) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if let Some(releases) = data.get("releases").and_then(Value::as_array) {
            for (release_idx, release) in releases.iter().enumerate() {
                if let Some(tracks) = release.get("tracks").and_then(Value::as_array) {
                    for (track_idx, track) in tracks.iter().enumerate() {
                        if let Some(isrc) = track.get("isrc").and_then(Value::as_str) {
                            if !self.isrc_regex.is_match(isrc) {
                                errors.push(ValidationError {
                                    path: format!("releases[{}].tracks[{}].isrc", release_idx, track_idx),
                                    message: format!("Invalid ISRC format: {}", isrc),
                                    severity: ValidationSeverity::Error,
                                    code: "INVALID_ISRC_FORMAT".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        errors
    }
}

pub struct TerritoryCodeRule {
    valid_territories: std::collections::HashSet<String>,
}

impl TerritoryCodeRule {
    pub fn new() -> Self {
        let mut territories = std::collections::HashSet::new();
        // ISO 3166-1 alpha-2 country codes
        territories.insert("US".to_string());
        territories.insert("GB".to_string());
        territories.insert("DE".to_string());
        territories.insert("FR".to_string());
        territories.insert("JP".to_string());
        // Add more as needed...

        Self {
            valid_territories: territories,
        }
    }
}

impl ValidationRule for TerritoryCodeRule {
    fn name(&self) -> &str {
        "territory-codes"
    }

    fn description(&self) -> &str {
        "Validates territory codes are valid ISO 3166-1 alpha-2 codes"
    }

    fn validate(&self, data: &Value, _context: &ValidationContext) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if let Some(releases) = data.get("releases").and_then(Value::as_array) {
            for (idx, release) in releases.iter().enumerate() {
                if let Some(territories) = release.get("territory_codes").and_then(Value::as_array) {
                    for territory in territories {
                        if let Some(code) = territory.as_str() {
                            if !self.valid_territories.contains(code) {
                                errors.push(ValidationError {
                                    path: format!("releases[{}].territory_codes", idx),
                                    message: format!("Invalid territory code: {}", code),
                                    severity: ValidationSeverity::Warning,
                                    code: "INVALID_TERRITORY_CODE".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        errors
    }
}
```

## Custom Validation Rules

### Business Logic Rules

```typescript
import { ValidationRule, ValidationContext, ValidationError } from 'ddex-builder';

export class BusinessLogicRule extends ValidationRule {
  name = 'business-logic';
  description = 'Custom business validation rules';

  validate(data: any, context: ValidationContext): ValidationError[] {
    const errors: ValidationError[] = [];

    // Custom rule: Albums must have at least 3 tracks
    if (data.releases) {
      data.releases.forEach((release: any, index: number) => {
        const path = `releases[${index}]`;
        
        if (release.release_type === 'Album') {
          const trackCount = release.tracks?.length || 0;
          if (trackCount < 3) {
            errors.push({
              path: `${path}.tracks`,
              message: `Albums must have at least 3 tracks, found ${trackCount}`,
              severity: 'warning',
              code: 'ALBUM_TRACK_COUNT'
            });
          }
        }

        // Custom rule: Release date cannot be in the future
        if (release.release_date) {
          const releaseDate = new Date(release.release_date);
          const now = new Date();
          
          if (releaseDate > now) {
            errors.push({
              path: `${path}.release_date`,
              message: 'Release date cannot be in the future',
              severity: 'error',
              code: 'FUTURE_RELEASE_DATE'
            });
          }
        }

        // Custom rule: Explicit content warning
        if (release.parental_warning_type === 'Explicit') {
          const hasExplicitTracks = release.tracks?.some((track: any) => 
            track.parental_warning_type === 'Explicit'
          );
          
          if (!hasExplicitTracks) {
            errors.push({
              path: `${path}.parental_warning_type`,
              message: 'Release marked explicit but no explicit tracks found',
              severity: 'warning',
              code: 'MISMATCHED_EXPLICIT_WARNING'
            });
          }
        }
      });
    }

    return errors;
  }
}
```

### Partner-Specific Rules

```python
from ddex_builder import ValidationRule, ValidationContext
from typing import List, Dict, Any

class SpotifyValidationRule(ValidationRule):
    name = "spotify-requirements"
    description = "Spotify-specific validation requirements"
    
    def validate(self, data: Dict[str, Any], context: ValidationContext) -> List[Dict]:
        errors = []
        
        if 'releases' in data:
            for i, release in enumerate(data['releases']):
                path = f"releases[{i}]"
                
                # Spotify requires UPC/EAN for releases
                if not release.get('upc') and not release.get('ean'):
                    errors.append({
                        'path': f"{path}.upc",
                        'message': "Spotify requires UPC or EAN for releases",
                        'severity': 'error',
                        'code': 'SPOTIFY_MISSING_UPC'
                    })
                
                # Track duration requirements
                if 'tracks' in release:
                    for j, track in enumerate(release['tracks']):
                        track_path = f"{path}.tracks[{j}]"
                        duration = track.get('duration_ms', 0)
                        
                        # Minimum 30 seconds
                        if duration < 30000:
                            errors.append({
                                'path': f"{track_path}.duration_ms",
                                'message': "Spotify requires tracks to be at least 30 seconds",
                                'severity': 'error',
                                'code': 'SPOTIFY_MIN_DURATION'
                            })
                        
                        # Maximum 10 minutes for normal tracks
                        if duration > 600000 and not track.get('is_classical'):
                            errors.append({
                                'path': f"{track_path}.duration_ms",
                                'message': "Spotify recommends tracks under 10 minutes (use is_classical flag for longer classical pieces)",
                                'severity': 'warning',
                                'code': 'SPOTIFY_MAX_DURATION'
                            })
                        
                        # ISRC required
                        if not track.get('isrc'):
                            errors.append({
                                'path': f"{track_path}.isrc",
                                'message': "Spotify requires ISRC for all tracks",
                                'severity': 'error',
                                'code': 'SPOTIFY_MISSING_ISRC'
                            })
        
        return errors

class YouTubeMusicValidationRule(ValidationRule):
    name = "youtube-music-requirements"
    description = "YouTube Music-specific validation requirements"
    
    def validate(self, data: Dict[str, Any], context: ValidationContext) -> List[Dict]:
        errors = []
        
        if 'releases' in data:
            for i, release in enumerate(data['releases']):
                path = f"releases[{i}]"
                
                # YouTube Music territory requirements
                territories = release.get('territory_codes', [])
                if 'WW' in territories and len(territories) > 1:
                    errors.append({
                        'path': f"{path}.territory_codes",
                        'message': "YouTube Music: WW (worldwide) cannot be combined with specific territories",
                        'severity': 'error',
                        'code': 'YOUTUBE_TERRITORY_CONFLICT'
                    })
                
                # Content rating requirements
                if release.get('parental_warning_type') == 'NotExplicit':
                    explicit_tracks = [
                        j for j, track in enumerate(release.get('tracks', []))
                        if track.get('parental_warning_type') == 'Explicit'
                    ]
                    
                    if explicit_tracks:
                        errors.append({
                            'path': f"{path}.parental_warning_type",
                            'message': f"YouTube Music: Release marked NotExplicit but tracks {explicit_tracks} are Explicit",
                            'severity': 'error',
                            'code': 'YOUTUBE_CONTENT_RATING_MISMATCH'
                        })
        
        return errors
```

## Rule Configuration

### Validation Profiles

```typescript
export interface ValidationProfile {
  name: string;
  description: string;
  rules: ValidationRuleConfig[];
  level: 'basic' | 'standard' | 'strict';
}

export interface ValidationRuleConfig {
  name: string;
  enabled: boolean;
  severity?: 'error' | 'warning' | 'info';
  options?: Record<string, any>;
}

export const VALIDATION_PROFILES: Record<string, ValidationProfile> = {
  basic: {
    name: 'Basic',
    description: 'Essential validation only',
    level: 'basic',
    rules: [
      { name: 'required-fields', enabled: true },
      { name: 'data-types', enabled: true },
      { name: 'isrc-format', enabled: true, severity: 'warning' }
    ]
  },

  standard: {
    name: 'Standard',
    description: 'DDEX specification compliance',
    level: 'standard',
    rules: [
      { name: 'required-fields', enabled: true },
      { name: 'data-types', enabled: true },
      { name: 'isrc-format', enabled: true },
      { name: 'territory-codes', enabled: true },
      { name: 'duration-limits', enabled: true },
      { name: 'business-logic', enabled: true, severity: 'warning' }
    ]
  },

  strict: {
    name: 'Strict',
    description: 'All validation rules plus best practices',
    level: 'strict',
    rules: [
      { name: 'required-fields', enabled: true },
      { name: 'data-types', enabled: true },
      { name: 'isrc-format', enabled: true },
      { name: 'territory-codes', enabled: true },
      { name: 'duration-limits', enabled: true },
      { name: 'business-logic', enabled: true },
      { name: 'metadata-quality', enabled: true },
      { name: 'best-practices', enabled: true, severity: 'warning' }
    ]
  },

  spotify: {
    name: 'Spotify',
    description: 'Spotify submission requirements',
    level: 'strict',
    rules: [
      { name: 'required-fields', enabled: true },
      { name: 'data-types', enabled: true },
      { name: 'isrc-format', enabled: true },
      { name: 'spotify-requirements', enabled: true },
      { name: 'duration-limits', enabled: true, options: { min_ms: 30000, max_ms: 600000 } }
    ]
  }
};

// Usage
import { DDEXBuilder } from 'ddex-builder';

const builder = new DDEXBuilder({
  validationProfile: 'spotify'
});

// Or custom configuration
const customBuilder = new DDEXBuilder({
  validationRules: [
    { name: 'required-fields', enabled: true },
    { name: 'custom-business-rule', enabled: true, severity: 'warning' }
  ]
});
```

### Rule Options

```python
from ddex_builder import DDEXBuilder, ValidationRuleConfig

# Configure rule-specific options
rules_config = [
    ValidationRuleConfig(
        name="duration-limits",
        enabled=True,
        options={
            "min_duration_ms": 15000,  # 15 seconds minimum
            "max_duration_ms": 1200000,  # 20 minutes maximum
            "classical_exception": True  # Allow longer classical pieces
        }
    ),
    ValidationRuleConfig(
        name="territory-codes",
        enabled=True,
        options={
            "allowed_territories": ["US", "GB", "DE", "FR", "CA"],
            "require_worldwide": False
        }
    ),
    ValidationRuleConfig(
        name="metadata-quality",
        enabled=True,
        severity="warning",
        options={
            "min_title_length": 1,
            "max_title_length": 200,
            "check_encoding": True,
            "suggest_improvements": True
        }
    )
]

builder = DDEXBuilder(validation_rules=rules_config)
```

## Advanced Rule Development

### Conditional Rules

```typescript
export class ConditionalValidationRule extends ValidationRule {
  name = 'conditional-validation';
  description = 'Rules that apply conditionally based on context';

  validate(data: any, context: ValidationContext): ValidationError[] {
    const errors: ValidationError[] = [];

    if (data.releases) {
      data.releases.forEach((release: any, index: number) => {
        const path = `releases[${index}]`;

        // Conditional rule: Classical releases need composer info
        if (this.isClassicalRelease(release)) {
          if (!release.composer && (!release.tracks?.some((t: any) => t.composer))) {
            errors.push({
              path: `${path}.composer`,
              message: 'Classical releases should include composer information',
              severity: 'warning',
              code: 'CLASSICAL_MISSING_COMPOSER'
            });
          }
        }

        // Conditional rule: Compilation albums need various artists
        if (release.release_type === 'Compilation') {
          const artists = new Set(release.tracks?.map((t: any) => t.artist) || []);
          if (artists.size < 2) {
            errors.push({
              path: `${path}.release_type`,
              message: 'Compilation albums should have tracks from multiple artists',
              severity: 'warning',
              code: 'COMPILATION_SINGLE_ARTIST'
            });
          }
        }

        // Conditional rule: Podcast episodes need episode numbers
        if (release.content_type === 'Podcast') {
          release.tracks?.forEach((track: any, trackIndex: number) => {
            if (!track.episode_number) {
              errors.push({
                path: `${path}.tracks[${trackIndex}].episode_number`,
                message: 'Podcast episodes should include episode numbers',
                severity: 'warning',
                code: 'PODCAST_MISSING_EPISODE_NUMBER'
              });
            }
          });
        }
      });
    }

    return errors;
  }

  private isClassicalRelease(release: any): boolean {
    const classicalGenres = ['Classical', 'Opera', 'Chamber Music', 'Symphony'];
    return release.genre?.some((g: string) => 
      classicalGenres.includes(g)
    ) || release.tracks?.some((track: any) =>
      track.genre?.some((g: string) => classicalGenres.includes(g))
    );
  }
}
```

### Contextual Rules

```python
class ContextualValidationRule(ValidationRule):
    name = "contextual-validation"
    description = "Validation that considers broader context"
    
    def validate(self, data: Dict[str, Any], context: ValidationContext) -> List[Dict]:
        errors = []
        
        # Get context information
        target_partner = context.get('target_partner')
        release_date = context.get('release_date')
        territory = context.get('primary_territory')
        
        if 'releases' in data:
            for i, release in enumerate(data['releases']):
                path = f"releases[{i}]"
                
                # Partner-specific contextual rules
                if target_partner == 'apple_music':
                    # Apple Music requires higher quality artwork
                    if release.get('artwork_resolution', 0) < 3000:
                        errors.append({
                            'path': f"{path}.artwork_resolution",
                            'message': "Apple Music requires artwork at least 3000x3000 pixels",
                            'severity': 'error',
                            'code': 'APPLE_ARTWORK_RESOLUTION'
                        })
                
                # Territory-specific contextual rules
                if territory == 'DE':
                    # German market requires GEMA information
                    if not release.get('gema_registered'):
                        errors.append({
                            'path': f"{path}.gema_registered",
                            'message': "German releases should specify GEMA registration status",
                            'severity': 'warning',
                            'code': 'GERMANY_GEMA_INFO'
                        })
                
                # Time-based contextual rules
                if release_date:
                    from datetime import datetime, timedelta
                    release_dt = datetime.fromisoformat(release_date)
                    
                    # Pre-order rules
                    if release_dt > datetime.now() + timedelta(days=90):
                        errors.append({
                            'path': f"{path}.release_date",
                            'message': "Pre-orders more than 90 days in advance may not be accepted by all stores",
                            'severity': 'warning',
                            'code': 'LONG_PREORDER_PERIOD'
                        })
        
        return errors
```

### Performance-Optimized Rules

```rust
use std::collections::HashMap;
use rayon::prelude::*;

pub struct OptimizedValidationRule {
    cache: HashMap<String, bool>,
    compiled_patterns: Vec<regex::Regex>,
}

impl OptimizedValidationRule {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            compiled_patterns: vec![
                regex::Regex::new(r"^[A-Z]{2}[A-Z0-9]{3}[0-9]{7}$").unwrap(), // ISRC
                regex::Regex::new(r"^[0-9]{13}$").unwrap(), // EAN
                regex::Regex::new(r"^[0-9]{12}$").unwrap(), // UPC
            ],
        }
    }
}

impl ValidationRule for OptimizedValidationRule {
    fn name(&self) -> &str {
        "optimized-format-validation"
    }

    fn validate(&self, data: &Value, _context: &ValidationContext) -> Vec<ValidationError> {
        if let Some(releases) = data.get("releases").and_then(Value::as_array) {
            // Parallel processing for large datasets
            releases
                .par_iter()
                .enumerate()
                .flat_map(|(release_idx, release)| {
                    self.validate_release(release, release_idx)
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl OptimizedValidationRule {
    fn validate_release(&self, release: &Value, release_idx: usize) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Batch validate all format fields
        let format_checks = vec![
            ("upc", &self.compiled_patterns[2], "Invalid UPC format"),
            ("ean", &self.compiled_patterns[1], "Invalid EAN format"),
        ];

        for (field, pattern, message) in format_checks {
            if let Some(value) = release.get(field).and_then(Value::as_str) {
                if !pattern.is_match(value) {
                    errors.push(ValidationError {
                        path: format!("releases[{}].{}", release_idx, field),
                        message: message.to_string(),
                        severity: ValidationSeverity::Error,
                        code: format!("INVALID_{}_FORMAT", field.to_uppercase()),
                    });
                }
            }
        }

        // Validate tracks in parallel for large track lists
        if let Some(tracks) = release.get("tracks").and_then(Value::as_array) {
            if tracks.len() > 10 {
                // Use parallel processing for large track lists
                let track_errors: Vec<ValidationError> = tracks
                    .par_iter()
                    .enumerate()
                    .flat_map(|(track_idx, track)| {
                        self.validate_track(track, release_idx, track_idx)
                    })
                    .collect();
                
                errors.extend(track_errors);
            } else {
                // Sequential processing for small track lists
                for (track_idx, track) in tracks.iter().enumerate() {
                    errors.extend(self.validate_track(track, release_idx, track_idx));
                }
            }
        }

        errors
    }

    fn validate_track(&self, track: &Value, release_idx: usize, track_idx: usize) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // ISRC validation with caching
        if let Some(isrc) = track.get("isrc").and_then(Value::as_str) {
            let cache_key = format!("isrc:{}", isrc);
            
            // Check cache first
            let is_valid = self.cache.get(&cache_key).copied()
                .unwrap_or_else(|| {
                    let valid = self.compiled_patterns[0].is_match(isrc);
                    // In a real implementation, you'd need thread-safe caching
                    valid
                });

            if !is_valid {
                errors.push(ValidationError {
                    path: format!("releases[{}].tracks[{}].isrc", release_idx, track_idx),
                    message: format!("Invalid ISRC format: {}", isrc),
                    severity: ValidationSeverity::Error,
                    code: "INVALID_ISRC_FORMAT".to_string(),
                });
            }
        }

        errors
    }
}
```

## Testing Validation Rules

```typescript
import { describe, it, expect } from 'vitest';
import { BusinessLogicRule } from './business-logic-rule';
import { ValidationContext } from 'ddex-builder';

describe('BusinessLogicRule', () => {
  const rule = new BusinessLogicRule();
  const context = new ValidationContext();

  it('should require albums to have at least 3 tracks', () => {
    const data = {
      releases: [{
        release_type: 'Album',
        title: 'Test Album',
        tracks: [
          { title: 'Track 1' },
          { title: 'Track 2' }
        ]
      }]
    };

    const errors = rule.validate(data, context);
    
    expect(errors).toHaveLength(1);
    expect(errors[0].code).toBe('ALBUM_TRACK_COUNT');
    expect(errors[0].severity).toBe('warning');
  });

  it('should reject future release dates', () => {
    const futureDate = new Date();
    futureDate.setFullYear(futureDate.getFullYear() + 1);

    const data = {
      releases: [{
        title: 'Future Release',
        release_date: futureDate.toISOString()
      }]
    };

    const errors = rule.validate(data, context);
    
    expect(errors).toHaveLength(1);
    expect(errors[0].code).toBe('FUTURE_RELEASE_DATE');
    expect(errors[0].severity).toBe('error');
  });

  it('should validate explicit content consistency', () => {
    const data = {
      releases: [{
        title: 'Test Release',
        parental_warning_type: 'Explicit',
        tracks: [
          { title: 'Clean Track', parental_warning_type: 'NotExplicit' }
        ]
      }]
    };

    const errors = rule.validate(data, context);
    
    expect(errors).toHaveLength(1);
    expect(errors[0].code).toBe('MISMATCHED_EXPLICIT_WARNING');
  });
});
```

## Best Practices

1. **Rule Granularity**: Create focused, single-purpose rules
2. **Performance**: Optimize for large datasets with parallel processing
3. **Caching**: Cache validation results for repeated patterns
4. **Extensibility**: Design rules to be easily extended and configured
5. **Error Messages**: Provide clear, actionable error messages
6. **Testing**: Thoroughly test rules with edge cases
7. **Documentation**: Document rule behavior and configuration options
8. **Versioning**: Version your custom rules for compatibility
9. **Context Awareness**: Use validation context for smarter rules
10. **Partner Integration**: Align rules with partner requirements