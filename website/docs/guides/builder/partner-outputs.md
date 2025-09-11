# Partner-Specific Outputs

Generate DDEX XML optimized for specific Digital Service Providers (DSPs) like Spotify, YouTube Music, and Amazon Music.

## Platform Presets

DDEX Suite provides built-in presets for major streaming platforms:

```typescript
import { DDEXBuilder } from 'ddex-builder';

// Spotify preset
const spotifyBuilder = new DDEXBuilder();
spotifyBuilder.applyPreset('spotify');

// YouTube Music preset  
const youtubeBuilder = new DDEXBuilder();
youtubeBuilder.applyPreset('youtube_music');

// Amazon Music preset
const amazonBuilder = new DDEXBuilder();
amazonBuilder.applyPreset('amazon_music');

// Universal preset (compatible with all platforms)
const universalBuilder = new DDEXBuilder();
universalBuilder.applyPreset('universal');
```

## Spotify Integration

### Spotify-Specific Requirements

```typescript
class SpotifyIntegration {
  private builder = new DDEXBuilder();
  
  constructor() {
    this.builder.applyPreset('spotify');
  }
  
  async createSpotifyRelease(releaseData: SpotifyReleaseData): Promise<string> {
    const ddexData = {
      messageHeader: {
        messageId: `SPOTIFY_${Date.now()}`,
        messageSenderName: releaseData.label,
        messageRecipientName: 'Spotify',
        messageCreatedDateTime: new Date().toISOString(),
        // Spotify prefers specific identifiers
        messageSenderId: 'PADPIDA2014101001U', // Example Spotify DPID
      },
      releases: [{
        releaseId: releaseData.releaseId,
        title: releaseData.title,
        artist: releaseData.artist,
        releaseType: releaseData.releaseType,
        label: releaseData.label,
        upc: releaseData.upc,
        releaseDate: releaseData.releaseDate,
        
        // Spotify-specific fields
        explicit: releaseData.explicit || false,
        territories: releaseData.territories || ['Worldwide'],
        genres: this.mapToSpotifyGenres(releaseData.genres),
        
        // Copyright information (important for Spotify)
        pLine: releaseData.pLine || `℗ ${new Date().getFullYear()} ${releaseData.label}`,
        cLine: releaseData.cLine || `© ${new Date().getFullYear()} ${releaseData.label}`,
        
        trackIds: releaseData.trackIds || []
      }],
      resources: releaseData.tracks?.map(track => ({
        resourceId: track.resourceId,
        resourceType: 'SoundRecording',
        title: track.title,
        artist: this.formatSpotifyArtistCredits(track),
        isrc: track.isrc,
        duration: track.duration,
        trackNumber: track.trackNumber,
        
        // Spotify audio quality preferences
        audioQuality: 'HIGH', // Spotify's preferred quality tier
        explicit: track.explicit || false,
        
        // Genre mapping for Spotify's taxonomy
        genres: this.mapToSpotifyGenres(track.genres || [])
      })) || [],
      deals: [{
        dealId: `SPOTIFY_DEAL_${Date.now()}`,
        releaseId: releaseData.releaseId,
        territories: releaseData.territories || ['Worldwide'],
        useTypes: ['Stream'], // Spotify is streaming-only
        commercialModelType: 'Subscription',
        dealStartDate: releaseData.releaseDate,
        
        // Spotify-specific deal terms
        platformSpecific: {
          spotifyReleaseDate: releaseData.spotifyReleaseDate || releaseData.releaseDate,
          playlistPitching: releaseData.playlistPitching || false
        }
      }]
    };
    
    return this.builder.build(ddexData);
  }
  
  private mapToSpotifyGenres(genres: string[]): string[] {
    // Map generic genres to Spotify's preferred taxonomy
    const genreMap: Record<string, string> = {
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
      'Blues': 'Blues'
    };
    
    return genres.map(genre => genreMap[genre] || genre);
  }
  
  private formatSpotifyArtistCredits(track: any): string {
    // Spotify prefers specific artist credit formatting
    if (track.features && track.features.length > 0) {
      return `${track.artist} (feat. ${track.features.join(', ')})`;
    }
    return track.artist;
  }
}

interface SpotifyReleaseData {
  releaseId: string;
  title: string;
  artist: string;
  releaseType: string;
  label: string;
  upc: string;
  releaseDate: string;
  explicit?: boolean;
  territories?: string[];
  genres?: string[];
  pLine?: string;
  cLine?: string;
  tracks?: SpotifyTrackData[];
  trackIds?: string[];
  spotifyReleaseDate?: string;
  playlistPitching?: boolean;
}

interface SpotifyTrackData {
  resourceId: string;
  title: string;
  artist: string;
  isrc: string;
  duration: string;
  trackNumber: number;
  explicit?: boolean;
  genres?: string[];
  features?: string[];
}
```

## YouTube Music Integration

### YouTube Music Requirements

```typescript
class YouTubeMusicIntegration {
  private builder = new DDEXBuilder();
  
  constructor() {
    this.builder.applyPreset('youtube_music');
  }
  
  async createYouTubeRelease(releaseData: YouTubeReleaseData): Promise<string> {
    const ddexData = {
      messageHeader: {
        messageId: `YOUTUBE_${Date.now()}`,
        messageSenderName: releaseData.label,
        messageRecipientName: 'YouTube Music',
        messageCreatedDateTime: new Date().toISOString()
      },
      releases: [{
        releaseId: releaseData.releaseId,
        title: releaseData.title,
        artist: releaseData.artist,
        releaseType: releaseData.releaseType,
        label: releaseData.label,
        upc: releaseData.upc,
        releaseDate: releaseData.releaseDate,
        
        // YouTube Music specific requirements
        territories: releaseData.territories || ['Worldwide'],
        genres: releaseData.genres || [],
        
        // YouTube prefers high-resolution artwork
        artworkRequirements: {
          minimumResolution: '1400x1400',
          preferredFormat: 'JPG',
          maxFileSize: '10MB'
        },
        
        trackIds: releaseData.trackIds || []
      }],
      resources: releaseData.tracks?.map(track => ({
        resourceId: track.resourceId,
        resourceType: 'SoundRecording',
        title: track.title,
        artist: track.artist,
        isrc: track.isrc,
        duration: track.duration,
        trackNumber: track.trackNumber,
        
        // YouTube Music audio specifications
        audioFormat: 'AAC',
        audioQuality: 'HIGH',
        sampleRate: '44100',
        bitRate: '320',
        
        // Content ID integration
        contentId: track.contentId,
        youtubeAssetId: track.youtubeAssetId
      })) || [],
      deals: [{
        dealId: `YOUTUBE_DEAL_${Date.now()}`,
        releaseId: releaseData.releaseId,
        territories: releaseData.territories || ['Worldwide'],
        useTypes: ['Stream', 'MusicVideo'], // YouTube supports both audio and video
        commercialModelType: 'AdSupportedModel',
        dealStartDate: releaseData.releaseDate,
        
        // YouTube-specific monetization
        monetizationEnabled: true,
        contentIdMatch: true
      }]
    };
    
    return this.builder.build(ddexData);
  }
}

interface YouTubeReleaseData {
  releaseId: string;
  title: string;
  artist: string;
  releaseType: string;
  label: string;
  upc: string;
  releaseDate: string;
  territories?: string[];
  genres?: string[];
  tracks?: YouTubeTrackData[];
  trackIds?: string[];
}

interface YouTubeTrackData {
  resourceId: string;
  title: string;
  artist: string;
  isrc: string;
  duration: string;
  trackNumber: number;
  contentId?: string;
  youtubeAssetId?: string;
}
```

## Amazon Music Integration

### Amazon Music Specifications

```typescript
class AmazonMusicIntegration {
  private builder = new DDEXBuilder();
  
  constructor() {
    this.builder.applyPreset('amazon_music');
  }
  
  async createAmazonRelease(releaseData: AmazonReleaseData): Promise<string> {
    const ddexData = {
      messageHeader: {
        messageId: `AMAZON_${Date.now()}`,
        messageSenderName: releaseData.label,
        messageRecipientName: 'Amazon Music',
        messageCreatedDateTime: new Date().toISOString()
      },
      releases: [{
        releaseId: releaseData.releaseId,
        title: releaseData.title,
        artist: releaseData.artist,
        releaseType: releaseData.releaseType,
        label: releaseData.label,
        upc: releaseData.upc,
        releaseDate: releaseData.releaseDate,
        
        // Amazon Music requirements
        territories: releaseData.territories || ['US', 'GB', 'DE', 'JP'],
        genres: releaseData.genres || [],
        
        // Amazon-specific metadata
        amazonSpecific: {
          asin: releaseData.asin, // Amazon Standard Identification Number
          primaryGenre: releaseData.primaryGenre,
          secondaryGenre: releaseData.secondaryGenre
        },
        
        trackIds: releaseData.trackIds || []
      }],
      resources: releaseData.tracks?.map(track => ({
        resourceId: track.resourceId,
        resourceType: 'SoundRecording',
        title: track.title,
        artist: track.artist,
        isrc: track.isrc,
        duration: track.duration,
        trackNumber: track.trackNumber,
        
        // Amazon HD audio support
        audioQuality: track.hdQuality ? 'HD' : 'STANDARD',
        spatialAudio: track.spatialAudio || false,
        
        // Amazon-specific fields
        amazonTrackId: track.amazonTrackId
      })) || [],
      deals: [{
        dealId: `AMAZON_DEAL_${Date.now()}`,
        releaseId: releaseData.releaseId,
        territories: releaseData.territories || ['US', 'GB', 'DE', 'JP'],
        useTypes: ['Stream', 'PermanentDownload'],
        commercialModelType: 'SubscriptionAndPurchase',
        dealStartDate: releaseData.releaseDate,
        
        // Amazon pricing tiers
        pricingTier: releaseData.pricingTier || 'standard',
        hdPremium: releaseData.hdPremium || false
      }]
    };
    
    return this.builder.build(ddexData);
  }
}

interface AmazonReleaseData {
  releaseId: string;
  title: string;
  artist: string;
  releaseType: string;
  label: string;
  upc: string;
  releaseDate: string;
  territories?: string[];
  genres?: string[];
  asin?: string;
  primaryGenre?: string;
  secondaryGenre?: string;
  pricingTier?: string;
  hdPremium?: boolean;
  tracks?: AmazonTrackData[];
  trackIds?: string[];
}

interface AmazonTrackData {
  resourceId: string;
  title: string;
  artist: string;
  isrc: string;
  duration: string;
  trackNumber: number;
  hdQuality?: boolean;
  spatialAudio?: boolean;
  amazonTrackId?: string;
}
```

## Multi-Platform Release Strategy

Create releases optimized for multiple platforms simultaneously:

```typescript
class MultiPlatformReleaseManager {
  async createMultiPlatformRelease(baseReleaseData: any): Promise<{
    spotify: string;
    youtube: string;
    amazon: string;
    universal: string;
  }> {
    const spotify = new SpotifyIntegration();
    const youtube = new YouTubeMusicIntegration();
    const amazon = new AmazonMusicIntegration();
    const universal = new DDEXBuilder();
    universal.applyPreset('universal');
    
    // Generate platform-specific releases
    const results = await Promise.all([
      spotify.createSpotifyRelease(this.adaptForSpotify(baseReleaseData)),
      youtube.createYouTubeRelease(this.adaptForYouTube(baseReleaseData)),
      amazon.createAmazonRelease(this.adaptForAmazon(baseReleaseData)),
      universal.build(baseReleaseData)
    ]);
    
    return {
      spotify: results[0],
      youtube: results[1],
      amazon: results[2],
      universal: results[3]
    };
  }
  
  private adaptForSpotify(data: any): SpotifyReleaseData {
    return {
      ...data,
      // Spotify-specific adaptations
      explicit: this.determineExplicitContent(data.tracks),
      playlistPitching: true
    };
  }
  
  private adaptForYouTube(data: any): YouTubeReleaseData {
    return {
      ...data,
      // YouTube-specific adaptations
      tracks: data.tracks?.map((track: any) => ({
        ...track,
        contentId: this.generateContentId(track),
        youtubeAssetId: this.generateYouTubeAssetId(track)
      }))
    };
  }
  
  private adaptForAmazon(data: any): AmazonReleaseData {
    return {
      ...data,
      // Amazon-specific adaptations
      asin: this.generateAsin(data),
      primaryGenre: data.genres?.[0],
      secondaryGenre: data.genres?.[1],
      hdPremium: true // Enable HD for Amazon
    };
  }
  
  private determineExplicitContent(tracks: any[]): boolean {
    return tracks?.some(track => 
      track.title?.toLowerCase().includes('explicit') ||
      track.lyrics?.includes('explicit_content_marker')
    ) || false;
  }
  
  private generateContentId(track: any): string {
    return `CID_${track.isrc}_${Date.now()}`;
  }
  
  private generateYouTubeAssetId(track: any): string {
    return `YT_${track.isrc}_${Math.random().toString(36).substr(2, 9)}`;
  }
  
  private generateAsin(releaseData: any): string {
    return `B${Math.random().toString(36).substr(2, 9).toUpperCase()}`;
  }
}

// Usage
const multiPlatform = new MultiPlatformReleaseManager();
const releases = await multiPlatform.createMultiPlatformRelease(baseReleaseData);

console.log('Generated releases for:');
console.log(`- Spotify: ${releases.spotify.length} characters`);
console.log(`- YouTube Music: ${releases.youtube.length} characters`);
console.log(`- Amazon Music: ${releases.amazon.length} characters`);
console.log(`- Universal: ${releases.universal.length} characters`);
```

## Python Multi-Platform Support

```python
from ddex_builder import DDEXBuilder
from typing import Dict, Any

class MultiPlatformBuilder:
    def __init__(self):
        self.builders = {
            'spotify': DDEXBuilder(preset='spotify'),
            'youtube_music': DDEXBuilder(preset='youtube_music'),
            'amazon_music': DDEXBuilder(preset='amazon_music'),
            'universal': DDEXBuilder(preset='universal')
        }
    
    def build_for_all_platforms(self, release_data: Dict[str, Any]) -> Dict[str, str]:
        """Build DDEX XML for all supported platforms"""
        
        results = {}
        
        for platform, builder in self.builders.items():
            try:
                # Adapt data for specific platform
                adapted_data = self._adapt_for_platform(release_data, platform)
                
                # Build XML
                xml = builder.build(adapted_data)
                results[platform] = xml
                
                print(f"✓ Generated {platform} DDEX ({len(xml)} characters)")
                
            except Exception as e:
                print(f"✗ Failed to generate {platform} DDEX: {e}")
                results[platform] = None
        
        return results
    
    def _adapt_for_platform(self, data: Dict[str, Any], platform: str) -> Dict[str, Any]:
        """Adapt release data for specific platform requirements"""
        
        adapted = data.copy()
        
        if platform == 'spotify':
            # Spotify adaptations
            adapted['message_header']['message_recipient_name'] = 'Spotify'
            if 'deals' in adapted:
                for deal in adapted['deals']:
                    deal['use_types'] = ['Stream']  # Spotify is streaming only
                    deal['commercial_model_type'] = 'Subscription'
        
        elif platform == 'youtube_music':
            # YouTube Music adaptations
            adapted['message_header']['message_recipient_name'] = 'YouTube Music'
            if 'deals' in adapted:
                for deal in adapted['deals']:
                    deal['use_types'] = ['Stream', 'MusicVideo']
                    deal['commercial_model_type'] = 'AdSupportedModel'
        
        elif platform == 'amazon_music':
            # Amazon Music adaptations
            adapted['message_header']['message_recipient_name'] = 'Amazon Music'
            if 'deals' in adapted:
                for deal in adapted['deals']:
                    deal['use_types'] = ['Stream', 'PermanentDownload']
                    deal['commercial_model_type'] = 'SubscriptionAndPurchase'
        
        return adapted

# Usage
builder = MultiPlatformBuilder()

release_data = {
    "message_header": {
        "message_id": "MULTI_001",
        "message_sender_name": "Independent Records",
        "message_created_date_time": "2024-03-15T10:00:00Z"
    },
    "releases": [{
        "release_id": "REL_001",
        "title": "Cross-Platform Hit",
        "artist": "Multi-Platform Artist",
        "release_type": "Single",
        "label": "Independent Records",
        "upc": "123456789012",
        "release_date": "2024-04-01",
        "territories": ["Worldwide"],
        "genres": ["Pop", "Electronic"]
    }],
    "resources": [{
        "resource_id": "TRK_001",
        "resource_type": "SoundRecording",
        "title": "Cross-Platform Hit",
        "artist": "Multi-Platform Artist",
        "isrc": "US-IND-24-00001",
        "duration": "PT3M30S",
        "track_number": 1
    }],
    "deals": [{
        "deal_id": "DEAL_001",
        "release_id": "REL_001",
        "territories": ["Worldwide"],
        "deal_start_date": "2024-04-01"
    }]
}

# Generate for all platforms
results = builder.build_for_all_platforms(release_data)

# Save to files
for platform, xml in results.items():
    if xml:
        with open(f'release_{platform}.xml', 'w', encoding='utf-8') as f:
            f.write(xml)
        print(f"Saved {platform} release to file")
```

## Platform Validation

Validate releases against platform-specific requirements:

```typescript
class PlatformValidator {
  validateSpotifyRelease(releaseData: any): ValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];
    
    // Required fields for Spotify
    if (!releaseData.upc) {
      errors.push('UPC is required for Spotify');
    }
    
    if (!releaseData.explicit !== undefined) {
      warnings.push('Explicit flag recommended for Spotify');
    }
    
    // ISRC validation for tracks
    releaseData.resources?.forEach((track: any, index: number) => {
      if (!track.isrc) {
        errors.push(`Track ${index + 1}: ISRC required for Spotify`);
      }
      
      if (!track.explicit !== undefined) {
        warnings.push(`Track ${index + 1}: Explicit flag recommended`);
      }
    });
    
    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      platform: 'spotify'
    };
  }
  
  validateYouTubeRelease(releaseData: any): ValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];
    
    // YouTube-specific validations
    if (!releaseData.pLine) {
      warnings.push('P-Line recommended for YouTube Music');
    }
    
    if (!releaseData.cLine) {
      warnings.push('C-Line recommended for YouTube Music');
    }
    
    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      platform: 'youtube_music'
    };
  }
  
  validateAmazonRelease(releaseData: any): ValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];
    
    // Amazon-specific validations
    if (!releaseData.primaryGenre) {
      warnings.push('Primary genre recommended for Amazon Music');
    }
    
    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      platform: 'amazon_music'
    };
  }
}

interface ValidationResult {
  isValid: boolean;
  errors: string[];
  warnings: string[];
  platform: string;
}
```

## Next Steps

- [Deterministic Output](./deterministic) - Ensure consistent builds across platforms
- [Batch Processing](./batch-processing) - Process multiple releases efficiently
- [Validation Workflows](./validation) - Comprehensive platform validation