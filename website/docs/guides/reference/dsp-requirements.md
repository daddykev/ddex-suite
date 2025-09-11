# DSP Requirements

Platform-specific requirements for major Digital Service Providers.

## Supported Platforms

DDEX Suite provides presets and specialized support for:

- **Spotify** - Global streaming leader
- **YouTube Music** - Google's music streaming service  
- **Amazon Music** - Amazon's streaming and download platform
- **Universal** - Compatible with all DDEX-compliant platforms

## Platform Comparison

| Feature | Spotify | YouTube Music | Amazon Music | Universal |
|---------|---------|---------------|--------------|-----------|
| **Primary Use** | Streaming | Streaming + Video | Streaming + Downloads | All platforms |
| **Audio Quality** | Up to 320kbps | Up to 256kbps AAC | Up to Hi-Res/HD | Variable |
| **Required IDs** | UPC, ISRC | UPC, ISRC | UPC, ISRC, ASIN | UPC, ISRC |
| **Explicit Flags** | Required | Recommended | Required | Recommended |
| **Territories** | Global | Global | Regional focus | Global |
| **Release Types** | All standard types | All + Music Videos | All standard types | All types |

## Spotify Requirements

### Essential Fields

```typescript
{
  // Message header
  messageRecipientName: 'Spotify',
  
  // Release requirements
  upc: '123456789012',           // Required - 12-digit UPC
  explicit: false,               // Required - boolean flag
  pLine: '℗ 2024 Label Name',   // Required - phonogram copyright
  cLine: '© 2024 Label Name',   // Required - copyright notice
  
  // Track requirements  
  isrc: 'US-ABC-24-12345',      // Required - valid ISRC format
  explicit: false,               // Required per track
  duration: 'PT3M45S',          // Required - ISO 8601 duration
  
  // Deal requirements
  useTypes: ['Stream'],          // Streaming only
  commercialModelType: 'Subscription',
  territories: ['Worldwide']     // Global distribution preferred
}
```

### Genre Mapping

Spotify prefers specific genre classifications:

```typescript
const spotifyGenres = {
  'Electronic': 'Electronic',
  'Pop': 'Pop', 
  'Rock': 'Rock',
  'Hip Hop': 'Hip-Hop/Rap',
  'R&B': 'R&B/Soul',
  'Country': 'Country',
  'Jazz': 'Jazz',
  'Classical': 'Classical',
  'Folk': 'Folk',
  'Reggae': 'Reggae',
  'Blues': 'Blues',
  'Alternative': 'Alternative & Indie',
  'World': 'World Music'
};
```

### Artist Credit Format

```typescript
// Primary artist only
artist: 'Taylor Swift'

// Featuring artists
artist: 'Taylor Swift (feat. Ed Sheeran)'

// Multiple primary artists
artist: 'Taylor Swift & Ed Sheeran'

// Avoid
artist: 'Taylor Swift featuring Ed Sheeran'  // Use 'feat.' instead
```

### Validation Rules

```typescript
const spotifyValidation = {
  upc: /^\d{12}$/,                    // Exactly 12 digits
  isrc: /^[A-Z]{2}-[A-Z0-9]{3}-\d{2}-\d{5}$/, // Standard ISRC format
  duration: /^PT\d+M\d+S$/,           // ISO 8601 duration
  explicit: 'boolean',                // Must be boolean, not string
  releaseDate: /^\d{4}-\d{2}-\d{2}$/, // YYYY-MM-DD format
};
```

## YouTube Music Requirements

### Essential Fields

```typescript
{
  // Message header
  messageRecipientName: 'YouTube Music',
  
  // Content ID integration
  contentId: 'CID_US-ABC-24-12345_1234567890',
  youtubeAssetId: 'YT_ASSET_1234567890',
  
  // Audio specifications
  audioFormat: 'AAC',
  audioQuality: 'HIGH',
  sampleRate: '44100',
  bitRate: '256',
  
  // Deal types
  useTypes: ['Stream', 'MusicVideo'],  // Supports both audio and video
  commercialModelType: 'AdSupportedModel', // Ad-supported or subscription
  
  // Monetization
  monetizationEnabled: true,
  contentIdMatch: true
}
```

### Artwork Requirements

```typescript
const youtubeArtwork = {
  minimumResolution: '1400x1400',     // Higher than most platforms
  preferredFormat: 'JPG',
  maxFileSize: '10MB',
  aspectRatio: '1:1',                 // Square only
  colorSpace: 'RGB'
};
```

### Video Content Support

YouTube Music uniquely supports music videos alongside audio:

```typescript
{
  resources: [
    {
      resourceType: 'SoundRecording',   // Audio version
      // ... audio-specific fields
    },
    {
      resourceType: 'MusicVideo',       // Video version (optional)
      videoFormat: 'MP4',
      videoResolution: '1920x1080',
      // ... video-specific fields
    }
  ]
}
```

## Amazon Music Requirements

### Essential Fields

```typescript
{
  // Message header
  messageRecipientName: 'Amazon Music',
  
  // Amazon identifiers
  asin: 'B08XXXXXXX',               // Amazon Standard Identification Number
  amazonTrackId: 'AMZ_123456789',
  
  // Genre classification
  primaryGenre: 'Pop',              // Main genre
  secondaryGenre: 'Electronic',     // Secondary genre (optional)
  
  // Audio quality tiers
  audioQuality: 'HD',               // 'STANDARD', 'HD', 'ULTRA_HD'
  spatialAudio: true,               // Supports spatial audio
  
  // Deal terms
  useTypes: ['Stream', 'PermanentDownload'], // Both streaming and sales
  commercialModelType: 'SubscriptionAndPurchase',
  pricingTier: 'standard',          // Pricing tier
  hdPremium: true                   // HD quality premium
}
```

### Territory Focus

Amazon Music has strong presence in specific regions:

```typescript
const amazonTerritories = {
  primary: ['US', 'GB', 'DE', 'JP', 'IN', 'CA', 'AU'],
  secondary: ['FR', 'IT', 'ES', 'MX', 'BR'],
  emerging: ['NL', 'BE', 'SE', 'NO', 'DK', 'FI']
};
```

### HD Audio Support

Amazon Music Unlimited supports high-quality audio:

```typescript
{
  // Standard quality (included in all plans)
  audioQuality: 'STANDARD',
  bitRate: '320',
  sampleRate: '44100',
  
  // HD quality (Ultra plan)
  audioQuality: 'HD', 
  bitRate: '850',     // Average
  sampleRate: '44100',
  bitDepth: '16',
  
  // Ultra HD quality (Ultra plan)
  audioQuality: 'ULTRA_HD',
  bitRate: '3730',    // Average for 24bit/192kHz
  sampleRate: '192000',
  bitDepth: '24'
}
```

## Universal Preset

The universal preset ensures compatibility across all platforms:

### Safe Defaults

```typescript
{
  // Conservative audio settings
  audioQuality: 'HIGH',
  audioFormat: 'MP3',               // Widely supported
  bitRate: '320',
  sampleRate: '44100',
  
  // Standard deal terms
  useTypes: ['Stream', 'PermanentDownload'],
  commercialModelType: 'SubscriptionAndPurchase',
  territories: ['Worldwide'],
  
  // Required identifiers
  upc: 'required',
  isrc: 'required',
  
  // Conservative metadata
  explicit: 'required',             // Always specify
  genres: 'max_3',                  // Limit to 3 genres max
  
  // Copyright lines
  pLine: 'required',
  cLine: 'required'
}
```

### Compatibility Features

```typescript
const universalCompatibility = {
  // Use most restrictive validation
  strictValidation: true,
  
  // Include all standard fields
  includeAllStandardFields: true,
  
  // Avoid platform-specific extensions  
  platformSpecificFields: false,
  
  // Use conservative durations
  maxTrackDuration: 'PT20M',        // 20 minutes max
  minTrackDuration: 'PT10S',        // 10 seconds min
  
  // Standard release types only
  allowedReleaseTypes: ['Single', 'EP', 'Album', 'Compilation']
};
```

## Validation by Platform

### Field Validation Matrix

| Field | Spotify | YouTube Music | Amazon Music | Universal |
|-------|---------|---------------|--------------|-----------|
| UPC | Required | Required | Required | Required |
| ISRC | Required | Required | Required | Required |
| Explicit | Required | Recommended | Required | Required |
| Duration | Required | Required | Required | Required |
| P-Line | Required | Recommended | Recommended | Required |
| C-Line | Required | Recommended | Recommended | Required |
| Genre | Required | Optional | Recommended | Recommended |
| Territories | Global preferred | Global | Regional focus | Global |

### Common Validation Errors

```typescript
const commonErrors = {
  spotify: [
    'UPC must be exactly 12 digits',
    'ISRC format invalid (use XX-XXX-YY-NNNNN)',  
    'Explicit flag must be boolean',
    'Duration must be in ISO 8601 format'
  ],
  
  youtubeMusic: [
    'Artwork resolution below 1400x1400',
    'Content ID required for monetization',
    'P-Line recommended for rights management'
  ],
  
  amazonMusic: [
    'ASIN format invalid',
    'Primary genre required for classification',
    'Territory selection affects HD availability'
  ],
  
  universal: [
    'Field required by one or more platforms',
    'Genre list too long (max 3 recommended)',
    'Release type not supported by all platforms'
  ]
};
```

## Testing Platform Compatibility

### Validation Script

```typescript
import { DDEXBuilder, PlatformValidator } from 'ddex-suite';

async function validateForAllPlatforms(releaseData: any) {
  const platforms = ['spotify', 'youtube_music', 'amazon_music'];
  const validator = new PlatformValidator();
  
  const results = await Promise.all(
    platforms.map(async platform => {
      const builder = new DDEXBuilder();
      builder.applyPreset(platform);
      
      try {
        const xml = await builder.build(releaseData);
        const validation = await validator.validate(xml, platform);
        
        return {
          platform,
          success: true,
          validation,
          xmlLength: xml.length
        };
      } catch (error) {
        return {
          platform,
          success: false,
          error: error.message
        };
      }
    })
  );
  
  return results;
}

// Usage
const testRelease = {
  messageHeader: { /* ... */ },
  releases: [{ /* ... */ }],
  resources: [{ /* ... */ }],
  deals: [{ /* ... */ }]
};

const results = await validateForAllPlatforms(testRelease);
results.forEach(result => {
  console.log(`${result.platform}: ${result.success ? '✓' : '✗'}`);
  if (!result.success) {
    console.log(`  Error: ${result.error}`);
  }
});
```

### Python Platform Testing

```python
from ddex_builder import DDEXBuilder, PlatformValidator

def test_platform_compatibility(release_data):
    """Test release compatibility across all platforms"""
    
    platforms = ['spotify', 'youtube_music', 'amazon_music', 'universal']
    validator = PlatformValidator()
    results = {}
    
    for platform in platforms:
        builder = DDEXBuilder(preset=platform)
        
        try:
            xml = builder.build(release_data)
            validation = validator.validate(xml, platform)
            
            results[platform] = {
                'success': True,
                'validation': validation,
                'xml_length': len(xml)
            }
            
            print(f"✓ {platform}: Generated {len(xml)} characters")
            
        except Exception as e:
            results[platform] = {
                'success': False,
                'error': str(e)
            }
            
            print(f"✗ {platform}: {str(e)}")
    
    return results

# Usage
test_data = {
    "message_header": {"message_id": "TEST_001"},
    "releases": [{"release_id": "REL_001"}]
}

results = test_platform_compatibility(test_data)
```

## Best Practices

### Multi-Platform Strategy

1. **Start with Universal preset** for broad compatibility
2. **Test with platform-specific presets** to optimize for each DSP
3. **Validate against all target platforms** before distribution
4. **Monitor platform updates** for changing requirements

### Field Population Strategy

1. **Required fields**: Always populate, never leave empty
2. **Recommended fields**: Populate when data is available  
3. **Platform-specific fields**: Only include for target platforms
4. **Optional fields**: Include if they add value

### Quality Assurance

1. **Automated validation** in your build pipeline
2. **Sample testing** with each platform's ingestion system  
3. **Regular updates** to platform presets as requirements change
4. **Error monitoring** in production deployments

## Next Steps

- [Common Patterns](./patterns) - Frequently used code patterns
- [Territory Codes](./territory-codes) - Complete territory reference
- [Schema Versions](./schema-versions) - DDEX version differences