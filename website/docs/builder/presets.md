---
sidebar_position: 6
---

# Presets

Simplified preset system for generating DDEX XML with transparent, maintainable configurations. Our refined approach focuses on public specifications and extensible frameworks.

## What are Presets?

Presets are pre-configured builder settings that provide tested configurations for DDEX compliance. Our updated preset strategy emphasizes honesty, maintainability, and extensibility.

### Our Preset Philosophy

The new architecture follows these principles:
- **Honesty**: Only provide presets for platforms with public specifications
- **Flexibility**: Generic presets provide DDEX-compliant baselines
- **Extensibility**: Custom framework allows organizations to build their own
- **Maintainability**: Fewer presets to maintain and validate
- **Legal Safety**: No risk of misrepresenting confidential specifications

## Available Presets

### YouTube Presets (Public Specification)

Based on YouTube's publicly available DDEX specifications.

```typescript
import { DdexBuilder } from 'ddex-builder';

// YouTube Album preset
const albumBuilder = new DdexBuilder();
albuMBuilder.applyPreset('youtube_album');

// YouTube Single preset
const singleBuilder = new DdexBuilder();
singleBuilder.applyPreset('youtube_single');

// YouTube Video preset
const videoBuilder = new DdexBuilder();
videoBuilder.applyPreset('youtube_video');

const xml = await albumBuilder.build(releaseData);
```

**Features:**
- Based on YouTube's public DDEX specification
- Content ID integration support
- Territory mapping for YouTube's global reach
- Video and audio content handling

**Use cases:**
- YouTube Music distribution
- Content ID monetization
- Video-first releases
- Public specification compliance

### Generic DDEX-Compliant Presets

Baseline configurations for standard DDEX compliance across platforms.

```typescript
import { DdexBuilder } from 'ddex-builder';

// Generic Audio Album
const albumBuilder = new DdexBuilder();
albumBuilder.applyPreset('generic_audio_album');

// Generic Audio Single  
const singleBuilder = new DdexBuilder();
singleBuilder.applyPreset('generic_audio_single');

// Generic Video Single
const videoBuilder = new DdexBuilder();
videoBuilder.applyPreset('generic_video_single');

// Generic Compilation
const compilationBuilder = new DdexBuilder();
compilationBuilder.applyPreset('generic_compilation');

const xml = await albumBuilder.build(releaseData);
```

**Features:**
- DDEX-compliant baseline configurations
- Standard territory handling (ISO 3166-1 codes)
- Basic genre normalization
- Conservative validation rules
- Maximum platform compatibility

**Use cases:**
- Initial DDEX implementation
- Multiple platform distribution
- Testing and development
- When platform requirements are unknown

### Custom Preset Framework

Extensible framework for creating organization-specific presets.

```typescript
import { DdexBuilder, CustomPreset } from 'ddex-builder';

// Load a custom preset template
const customBuilder = new DdexBuilder();
customBuilder.loadCustomPreset('./my-label-preset.json');

// Or define inline
const myPreset: CustomPreset = {
  name: 'my_label_preset',
  basePreset: 'generic_audio_album',
  validationRules: {
    'releases.label': { required: true, value: 'My Record Label' },
    'resources.duration': { minDuration: 'PT2M00S' }
  },
  defaults: {
    territories: ['US', 'CA', 'GB'],
    commercialModelType: 'Subscription'
  }
};

customBuilder.applyCustomPreset(myPreset);
const xml = await customBuilder.build(releaseData);
```

**Framework Features:**
- Base preset inheritance
- Custom validation rules
- Default value injection
- Business logic hooks
- Field mapping support

**Use cases:**
- Label-specific requirements
- Internal compliance rules
- Regional variations
- Workflow optimizations


## Exploring Presets

### List Available Presets

```typescript
import { DdexBuilder } from 'ddex-builder';

const builder = new DdexBuilder();
const presets = builder.getAvailablePresets();

console.log('Available presets:', presets);
// Output: ['youtube_album', 'youtube_single', 'youtube_video', 'generic_audio_album', 'generic_audio_single', 'generic_video_single', 'generic_compilation']
```

### Get Preset Information

```typescript
// Get detailed preset information
const youtubeInfo = builder.getPresetInfo('youtube_album');

console.log('Preset:', youtubeInfo.name);
console.log('Description:', youtubeInfo.description);  
console.log('Version:', youtubeInfo.version);
console.log('Profile:', youtubeInfo.profile);
console.log('Required fields:', youtubeInfo.requiredFields);
console.log('Specification:', youtubeInfo.specification);
```

**Sample Output:**
```json
{
  "name": "youtube_album",
  "description": "YouTube Music album preset based on public DDEX specification",
  "version": "2024.1",
  "profile": "content_id",
  "requiredFields": [
    "releases.title",
    "releases.territories",
    "resources.isrc",
    "resources.duration"
  ],
  "specification": "Based on YouTube's publicly available DDEX Content ID specification"
}
```

### Get Preset Validation Rules

```typescript
// See what validation rules a preset applies
const youtubeRules = builder.getPresetValidationRules('youtube_album');

youtubeRules.forEach(rule => {
  console.log(`${rule.fieldName}: ${rule.message}`);
});
```

**Sample Output:**
```
releases.title: Album title is required for YouTube Music
resources.isrc: ISRC must be valid format (XX-XXX-YY-NNNNN)
resources.duration: Track duration must be specified
territories: Must include at least one territory for Content ID
```

## Creating Custom Presets

### Basic Custom Preset

```typescript
import { DdexBuilder, CustomPreset } from 'ddex-builder';

const myLabelPreset: CustomPreset = {
  name: 'my_record_label',
  description: 'Custom preset for My Record Label releases',
  version: '1.0',
  
  // Default values applied to all releases
  defaults: {
    label: 'My Record Label',
    territories: ['US', 'CA', 'GB'],
    commercialModelType: 'Subscription'
  },
  
  // Required fields validation
  requiredFields: {
    'releases': ['title', 'artist', 'upc', 'releaseDate'],
    'resources': ['title', 'artist', 'isrc', 'duration']
  },
  
  // Custom validation rules
  validationRules: {
    'releases.title': {
      ruleType: 'maxLength',
      value: 100,
      message: 'Release title must be 100 characters or less'
    },
    'resources.duration': {
      ruleType: 'minDuration', 
      value: 'PT1M00S',
      message: 'Tracks must be at least 60 seconds long'
    },
    'releases.upc': {
      ruleType: 'format',
      value: '^[0-9]{12}$',
      message: 'UPC must be exactly 12 digits'
    }
  },
  
  // Business logic rules
  businessRules: {
    enforceISRC: true,
    validateTerritoryRights: true,
    requireExplicitFlag: false,
    genreNormalization: ['Pop', 'Rock', 'Hip-Hop', 'Electronic', 'Jazz']
  },
  
  // Partner-specific metadata
  metadata: {
    labelCode: 'MRL001',
    distributionChannel: 'digital_only',
    qualityTier: 'premium'
  }
};

// Apply custom preset
const builder = new DdexBuilder();
builder.applyCustomPreset(myLabelPreset);
```

### Advanced Custom Preset

```typescript
const advancedPreset: CustomPreset = {
  name: 'premium_classical',
  description: 'Preset for premium classical music releases',
  version: '2.0',
  
  // Conditional defaults based on data
  conditionalDefaults: {
    // Apply different defaults based on genre
    genres: {
      classical: {
        territories: ['WorldWide'],
        commercialModelType: 'SubscriptionAndPurchase',
        audioQuality: 'lossless'
      },
      opera: {
        territories: ['US', 'EU'],
        commercialModelType: 'PurchaseOnly', 
        parentalWarning: false
      }
    }
  },
  
  // Complex validation rules
  validationRules: {
    // Composer information required for classical
    'resources.metadata.composer': {
      ruleType: 'required',
      condition: 'genre === "classical"',
      message: 'Composer information required for classical music'
    },
    
    // Longer durations expected for classical
    'resources.duration': {
      ruleType: 'minDuration',
      value: 'PT2M00S', 
      condition: 'genre === "classical"',
      message: 'Classical tracks should be at least 2 minutes'
    },
    
    // High-resolution artwork for premium releases
    'releases.artworkResolution': {
      ruleType: 'minResolution',
      value: '1400x1400',
      message: 'Premium releases require high-resolution artwork'
    }
  },
  
  // Custom field mappings
  fieldMappings: {
    // Map common fields to classical-specific terminology
    'artist': 'performer',
    'album': 'collection',
    'track': 'movement'
  },
  
  // Preprocessing hooks
  preprocessors: {
    // Normalize classical composer names
    'resources.metadata.composer': (value) => {
      return normalizeComposerName(value);
    },
    
    // Generate opus numbers for classical works
    'resources.metadata.opusNumber': (resource) => {
      if (!resource.metadata?.opusNumber && resource.metadata?.composer) {
        return generateOpusNumber(resource.metadata.composer, resource.title);
      }
      return resource.metadata?.opusNumber;
    }
  },
  
  // Post-processing hooks
  postprocessors: {
    // Add movement numbering to track titles
    'resources.title': (resource, index, collection) => {
      if (resource.metadata?.movementNumber) {
        return `${resource.metadata.movementNumber}. ${resource.title}`;
      }
      return resource.title;
    }
  }
};

// Apply advanced preset
builder.applyCustomPreset(advancedPreset);
```

## Preset Comparison

### Side-by-Side Comparison

```typescript
async function comparePresets(releaseData: any) {
  const presets = ['universal', 'spotify', 'apple_music'];
  const results = {};
  
  for (const preset of presets) {
    const builder = new DdexBuilder();
    builder.applyPreset(preset);
    
    try {
      const xml = await builder.build(releaseData);
      results[preset] = {
        success: true,
        size: xml.length,
        xml: xml
      };
    } catch (error) {
      results[preset] = {
        success: false,
        error: error.message
      };
    }
  }
  
  // Analyze differences
  console.log('Preset Comparison Results:');
  Object.entries(results).forEach(([preset, result]) => {
    if (result.success) {
      console.log(`✅ ${preset}: ${result.size} bytes`);
    } else {
      console.log(`❌ ${preset}: ${result.error}`);
    }
  });
  
  return results;
}
```

### Validation Comparison

```typescript
function compareValidationRules() {
  const builder = new DdexBuilder();
  const presets = ['spotify', 'apple_music', 'youtube_music'];
  
  presets.forEach(preset => {
    console.log(`\n${preset.toUpperCase()} PRESET RULES:`);
    const rules = builder.getPresetValidationRules(preset);
    
    rules.forEach(rule => {
      console.log(`  ${rule.fieldName}: ${rule.message}`);
    });
  });
}
```

## Preset Best Practices

### 1. Start with Universal

Begin development with the universal preset:

```typescript
// Development phase
const devBuilder = new DdexBuilder();
devBuilder.applyPreset('universal'); // Minimal requirements

// Production phase - apply specific presets
const prodBuilder = new DdexBuilder();
prodBuilder.applyPreset('spotify'); // Platform-specific
```

### 2. Test Against Multiple Presets

Validate your data against multiple presets:

```typescript
async function validateForAllPlatforms(releaseData: any) {
  const platforms = ['spotify', 'apple_music', 'youtube_music', 'amazon_music'];
  const validationResults = {};
  
  for (const platform of platforms) {
    const builder = new DdexBuilder();
    builder.applyPreset(platform);
    
    const validation = await builder.validate(releaseData);
    validationResults[platform] = validation;
    
    if (validation.isValid) {
      console.log(`✅ ${platform}: Valid`);
    } else {
      console.log(`❌ ${platform}: ${validation.errors.length} errors`);
      validation.errors.forEach(error => {
        console.log(`    ${error}`);
      });
    }
  }
  
  return validationResults;
}
```

### 3. Handle Preset-Specific Data

Different presets may require different data structures:

```typescript
function prepareDataForPreset(baseData: any, preset: string) {
  const data = { ...baseData };
  
  switch (preset) {
    case 'spotify':
      data.releases.forEach(release => {
        release.explicitContent = release.parentalWarning || false;
        release.streamingOnly = true;
      });
      break;
      
    case 'youtube_music':
      data.releases.forEach(release => {
        release.contentIdEnabled = true;
        release.youtubeChannelId = process.env.YOUTUBE_CHANNEL_ID;
      });
      break;
      
    case 'apple_music':
      data.releases.forEach(release => {
        release.masteredForItunes = data.audioQuality === 'lossless';
        release.priceTier = determinePriceTier(release.releaseType);
      });
      break;
  }
  
  return data;
}

// Usage
const spotifyData = prepareDataForPreset(baseData, 'spotify');
const spotifyBuilder = new DdexBuilder();
spotifyBuilder.applyPreset('spotify');
const spotifyXml = await spotifyBuilder.build(spotifyData);
```

### 4. Document Preset Choices

Document why you chose specific presets:

```typescript
/**
 * Preset Selection Strategy:
 * 
 * - Universal: Default for new releases, testing
 * - Spotify: Streaming-focused releases, indie labels  
 * - Apple Music: Premium releases, audiophile content
 * - YouTube Music: Viral content, video-first releases
 * - Amazon Music: Family-friendly content, Prime eligibility
 */
const PRESET_STRATEGY = {
  development: 'universal',
  mainstream_streaming: 'spotify', 
  premium_audio: 'apple_music',
  video_content: 'youtube_music',
  family_content: 'amazon_music'
};

function selectPreset(releaseType: string, targetAudience: string): string {
  // Business logic for preset selection
  if (targetAudience === 'audiophile') return 'apple_music';
  if (releaseType === 'viral') return 'youtube_music';
  if (targetAudience === 'family') return 'amazon_music';
  return 'spotify'; // Default for most streaming
}
```

## Troubleshooting Presets

### Preset Validation Errors

When a preset validation fails:

```typescript
try {
  const xml = await builder.build(releaseData);
} catch (error) {
  if (error instanceof PresetValidationError) {
    console.log(`Preset ${error.presetName} validation failed:`);
    error.violations.forEach(violation => {
      console.log(`  ${violation.field}: ${violation.message}`);
      if (violation.suggestions) {
        console.log(`    Suggestions: ${violation.suggestions.join(', ')}`);
      }
    });
  }
}
```

### Conflicting Presets

Avoid applying multiple presets:

```typescript
// ❌ Don't do this - conflicting rules
builder.applyPreset('spotify');
builder.applyPreset('apple_music'); // Overwrites Spotify rules

// ✅ Use separate builders for different presets
const youtubeBuilder = new DdexBuilder();
youtubeBuilder.applyPreset('youtube_album');

const genericBuilder = new DdexBuilder(); 
genericBuilder.applyPreset('generic_audio_album');
```

### Preset Version Compatibility

Check preset versions for compatibility:

```typescript
const presetInfo = builder.getPresetInfo('youtube_album');
console.log(`Using YouTube preset v${presetInfo.version}`);

if (presetInfo.version < '2024.1') {
  console.warn('⚠️ YouTube preset may be outdated');
}
```

## Why This Approach?

Our refined preset strategy offers several key benefits:

1. **Transparency**: Only YouTube presets are based on publicly available specifications
2. **Honesty**: No guessing about confidential platform requirements
3. **Maintainability**: Fewer presets to maintain and update
4. **Extensibility**: Custom framework allows organizations to build their own
5. **Legal Safety**: No risk of misrepresenting proprietary specifications
6. **DDEX Compliance**: Generic presets ensure baseline DDEX compliance

This approach makes DDEX Suite more credible and sustainable while still providing valuable preset functionality through transparent, publicly available specifications and extensible custom frameworks.

For complete workflows combining presets with parsing, see the [Parser Integration Guide](../parser/).