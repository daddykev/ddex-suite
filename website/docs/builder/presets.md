---
sidebar_position: 6
---

# Presets

Platform-specific configurations for generating DDEX XML that meets partner requirements. Presets handle complex validation rules, metadata mappings, and compliance requirements automatically.

## What are Presets?

Presets are pre-configured builder settings that ensure your DDEX messages comply with specific platform requirements. Rather than manually configuring validation rules, territory mappings, and metadata requirements for each partner, presets provide tested configurations that handle these complexities automatically.

### Why Use Presets?

Each music platform has unique requirements:
- **Field requirements**: Some platforms require fields that others don't
- **Validation rules**: Different business logic and constraints
- **Metadata preferences**: Specific ways to handle genres, territories, and rights
- **Technical specifications**: Format preferences and encoding requirements

Presets eliminate the guesswork and ensure compliance.

## Available Presets

### Universal Preset (`universal`)

The default preset for broad compatibility across platforms.

```typescript
import { DdexBuilder } from 'ddex-builder';

const builder = new DdexBuilder();
builder.applyPreset('universal');

const xml = await builder.build(releaseData);
```

**Features:**
- Minimal field requirements for maximum compatibility
- Standard territory handling (ISO 3166-1 codes)
- Basic genre normalization
- Conservative validation rules

**Use cases:**
- Initial DDEX implementation
- Multiple platform distribution
- Testing and development
- When platform requirements are unknown

### Spotify Preset (`spotify`)

Optimized for Spotify's streaming platform requirements.

```typescript
const spotifyBuilder = new DdexBuilder();
spotifyBuilder.applyPreset('spotify');

const spotifyXml = await spotifyBuilder.build(releaseData);
```

**Key Requirements:**
- **Explicit content flagging**: Required for all releases
- **Territory restrictions**: Streaming-only territories supported
- **Artist ID validation**: Spotify Artist ID required when available
- **Genre normalization**: Maps to Spotify's genre taxonomy
- **ISRC validation**: Strict format checking
- **Duration requirements**: Minimum 30 seconds per track

**Additional Features:**
- Automatic canvas/video metadata inclusion
- Podcast-specific handling for spoken word content
- Pre-save campaign metadata support
- Spotify Connect device compatibility flags

**Example:**
```typescript
const spotifyData = {
  messageHeader: {
    messageSenderName: 'Independent Label',
    messageRecipientName: 'Spotify'
  },
  releases: [{
    releaseId: 'REL_SPOTIFY_001',
    title: 'Indie Rock Album',
    artist: 'The Indie Band',
    explicitContent: true,        // Required by Spotify preset
    spotifyArtistId: 'spotify:artist:123', // Platform-specific ID
    territories: ['US', 'CA', 'GB', 'DE'], // Streaming territories
    genres: ['indie rock', 'alternative'] // Auto-normalized
  }],
  resources: [{
    resourceId: 'SR_001',
    title: 'Lead Single',
    isrc: 'US-S1Z-99-00001',    // Validated format
    duration: 'PT3M45S',         // Meets minimum duration
    explicitContent: true        // Track-level flag
  }]
};

const xml = await spotifyBuilder.build(spotifyData);
```

### YouTube Music Preset (`youtube_music`)

Configured for YouTube Music and Content ID requirements.

```typescript
const youtubeBuilder = new DdexBuilder();
youtubeBuilder.applyPreset('youtube_music');

const youtubeXml = await youtubeBuilder.build(releaseData);
```

**Key Requirements:**
- **Content ID metadata**: Required for monetization
- **Territory handling**: YouTube-specific territory mappings
- **Usage rights**: Monetization-compatible rights only
- **Reference file handling**: Audio fingerprinting support
- **Channel linking**: YouTube channel association

**Additional Features:**
- Automatic Music Key (video topic) generation
- Art Track creation for audio-only content
- Short-form content (YouTube Shorts) metadata
- Live streaming and premiere support

**Example:**
```typescript
const youtubeData = {
  messageHeader: {
    messageSenderName: 'MCN Label',
    messageRecipientName: 'YouTube Music'
  },
  releases: [{
    releaseId: 'REL_YT_001', 
    title: 'Viral Hits Collection',
    artist: 'Various Artists',
    youtubeChannelId: 'UCxxxxxxxxxxxxxxx', // Required for Content ID
    contentIdEnabled: true,                 // Enable monetization
    territories: ['WorldWide']              // YouTube global distribution
  }],
  resources: [{
    resourceId: 'SR_001',
    title: 'TikTok Viral Song',
    referenceFile: '/path/to/reference.mp3', // For Content ID fingerprinting
    usageRights: ['monetization', 'user_generated_content']
  }]
};

const xml = await youtubeBuilder.build(youtubeData);
```

### Apple Music Preset (`apple_music`)

Designed for iTunes Store and Apple Music compliance.

```typescript
const appleBuilder = new DdexBuilder();
appleBuilder.applyPreset('apple_music');

const appleXml = await appleBuilder.build(releaseData);
```

**Key Requirements:**
- **iTunes Store compliance**: Strict metadata validation
- **Mastered for iTunes**: Audio quality specifications
- **Region-specific pricing**: Price tier mappings
- **Album artwork**: High-resolution artwork requirements
- **Pre-order support**: Release date handling for pre-orders

**Additional Features:**
- Dolby Atmos spatial audio metadata
- Lossless audio quality indicators
- Apple Digital Masters certification
- Animated artwork support for iOS

**Example:**
```typescript
const appleData = {
  messageHeader: {
    messageSenderName: 'Premium Label',
    messageRecipientName: 'Apple Music'
  },
  releases: [{
    releaseId: 'REL_APPLE_001',
    title: 'Audiophile Album',
    artist: 'Jazz Virtuoso',
    masteredForItunes: true,        // Quality certification
    priceTier: 'tier_2',           // Apple pricing tier
    preOrderDate: '2024-02-01',     // Pre-order available
    releaseDate: '2024-02-15',      // Official release
    artworkResolution: '3000x3000'  // High-res artwork
  }],
  resources: [{
    resourceId: 'SR_001', 
    title: 'Jazz Standard',
    audioQuality: 'lossless',       // Apple Lossless format
    spatialAudio: 'dolby_atmos',    // Spatial audio mixing
    appleCertified: true            // Apple Digital Masters
  }]
};

const xml = await appleBuilder.build(appleData);
```

### Amazon Music Preset (`amazon_music`)

Configured for Amazon Music and Prime Music distribution.

```typescript
const amazonBuilder = new DdexBuilder();
amazonBuilder.applyPreset('amazon_music');

const amazonXml = await amazonBuilder.build(releaseData);
```

**Key Requirements:**
- **Prime eligibility**: Metadata for Prime Music inclusion
- **Territory handling**: Amazon marketplace regions
- **Family filtering**: Content appropriate for Amazon Kids
- **Voice compatibility**: Alexa voice command optimization

**Additional Features:**
- Echo device optimization
- Amazon Original content flagging
- Prime Gaming soundtrack integration
- Kindle reading music synchronization

## Exploring Presets

### List Available Presets

```typescript
import { DdexBuilder } from 'ddex-builder';

const builder = new DdexBuilder();
const presets = builder.getAvailablePresets();

console.log('Available presets:', presets);
// Output: ['universal', 'spotify', 'youtube_music', 'apple_music', 'amazon_music']
```

### Get Preset Information

```typescript
// Get detailed preset information
const spotifyInfo = builder.getPresetInfo('spotify');

console.log('Preset:', spotifyInfo.name);
console.log('Description:', spotifyInfo.description);  
console.log('Version:', spotifyInfo.version);
console.log('Profile:', spotifyInfo.profile);
console.log('Required fields:', spotifyInfo.requiredFields);
console.log('Disclaimer:', spotifyInfo.disclaimer);
```

**Sample Output:**
```json
{
  "name": "spotify",
  "description": "Optimized for Spotify streaming platform with explicit content handling and territory restrictions",
  "version": "2024.1",
  "profile": "streaming",
  "requiredFields": [
    "releases.explicitContent",
    "releases.territories",
    "resources.isrc",
    "resources.duration"
  ],
  "disclaimer": "This preset implements Spotify's current requirements as of 2024. Please verify latest requirements with Spotify partner documentation."
}
```

### Get Preset Validation Rules

```typescript
// See what validation rules a preset applies
const spotifyRules = builder.getPresetValidationRules('spotify');

spotifyRules.forEach(rule => {
  console.log(`${rule.fieldName}: ${rule.message}`);
});
```

**Sample Output:**
```
releases.explicitContent: Explicit content flag is required for Spotify
resources.isrc: ISRC must be valid format (XX-XXX-YY-NNNNN)
resources.duration: Track duration must be at least PT0M30S
territories: Must include at least one streaming territory
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
const spotifyBuilder = new DdexBuilder();
spotifyBuilder.applyPreset('spotify');

const appleBuilder = new DdexBuilder(); 
appleBuilder.applyPreset('apple_music');
```

### Preset Version Compatibility

Check preset versions for compatibility:

```typescript
const presetInfo = builder.getPresetInfo('spotify');
console.log(`Using Spotify preset v${presetInfo.version}`);

if (presetInfo.version < '2024.1') {
  console.warn('⚠️ Spotify preset may be outdated');
}
```

Presets make it easy to generate compliant DDEX XML for specific platforms while handling the complexity of different requirements automatically. For complete workflows combining presets with parsing, see the [Parser Integration Guide](../parser/).