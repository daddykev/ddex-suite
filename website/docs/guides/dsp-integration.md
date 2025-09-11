# DSP Integration Guide

Learn how to integrate the DDEX Suite with major Digital Service Providers (DSPs) including Spotify, Apple Music, YouTube Music, and Amazon Music.

## Problem Statement

Each DSP has unique requirements for DDEX message format, validation rules, and delivery mechanisms. Common challenges include:

- **Platform-Specific Requirements**: Different validation rules and required fields
- **Format Variations**: Subtle differences in accepted DDEX structures
- **Delivery Protocols**: Various submission methods (FTP, API, portal uploads)
- **Error Handling**: Platform-specific error codes and responses
- **Testing**: Sandbox environments with different configurations
- **Compliance**: Ensuring messages meet platform certification requirements

## Solution Approach

The DDEX Suite provides platform-specific presets, validation rules, and delivery helpers to streamline DSP integration while maintaining compliance with each platform's requirements.

### Integration Benefits

| Challenge | Without DDEX Suite | With DDEX Suite |
|-----------|-------------------|-----------------|
| **Platform Compliance** | Manual rule implementation | Built-in presets |
| **Validation** | Custom validators | Platform-specific validation |
| **Format Consistency** | Manual XML generation | Deterministic output |
| **Error Handling** | Generic error parsing | Platform-aware errors |
| **Testing** | Manual test file creation | Preset-based test generation |

## Spotify Integration

### Basic Setup

```typescript
import { DdexBuilder } from 'ddex-builder';
import { DDEXParser } from 'ddex-parser';

class SpotifyIntegration {
  private builder: DdexBuilder;
  private parser: DDEXParser;

  constructor() {
    this.builder = new DdexBuilder();
    this.builder.applyPreset('spotify');
    this.parser = new DDEXParser();
  }

  async createNewReleaseMessage(releaseData: SpotifyReleaseData): Promise<string> {
    // Spotify requires specific fields
    const release = {
      releaseId: releaseData.releaseId,
      releaseType: releaseData.type, // 'Album', 'Single', 'EP'
      title: releaseData.title,
      artist: releaseData.primaryArtist,
      label: releaseData.labelName, // Required by Spotify
      upc: releaseData.upc, // Required for identification
      releaseDate: releaseData.releaseDate, // ISO 8601 format
      genre: this.mapToSpotifyGenre(releaseData.genre),
      explicitContent: releaseData.hasExplicitContent,
      trackIds: releaseData.tracks.map(t => t.resourceId),
      // Spotify-specific metadata
      metadata: {
        spotifyUri: releaseData.spotifyUri,
        primaryMarket: releaseData.primaryMarket,
        distributionStrategy: 'global'
      }
    };

    // Add individual tracks
    for (const track of releaseData.tracks) {
      const resource = {
        resourceId: track.resourceId,
        resourceType: 'SoundRecording',
        title: track.title,
        artist: track.featuredArtists ? 
          `${track.primaryArtist} feat. ${track.featuredArtists.join(', ')}` : 
          track.primaryArtist,
        isrc: track.isrc, // Required by Spotify
        duration: track.duration, // PT3M45S format
        trackNumber: track.trackNumber,
        explicitContent: track.hasExplicitContent,
        metadata: {
          spotifyUri: track.spotifyUri,
          previewStartTime: track.previewStart || 'PT30S'
        }
      };

      this.builder.addResource(resource);
    }

    this.builder.addRelease(release);

    // Build with Spotify-specific validation
    const xml = await this.builder.build({
      messageId: `SPOTIFY_${Date.now()}`,
      sender: 'YourLabelDPID',
      recipient: 'SpotifyDPID',
      version: '4.3'
    });

    return xml;
  }

  private mapToSpotifyGenre(genre: string): string {
    // Map to Spotify's approved genre list
    const genreMap: Record<string, string> = {
      'Alternative Rock': 'Alternative',
      'Electronic Dance Music': 'Electronic',
      'Rhythm and Blues': 'R&B',
      'Hip Hop': 'Hip-Hop',
      'Country Music': 'Country'
    };

    return genreMap[genre] || genre;
  }

  async validateForSpotify(xml: string): Promise<ValidationResult> {
    // Parse and validate with Spotify rules
    const result = await this.parser.parse(xml);
    const validation = await this.builder.validate();

    // Spotify-specific validation checks
    const spotifyErrors: string[] = [];

    result.flat.releases.forEach(release => {
      if (!release.labelName) {
        spotifyErrors.push(`Label name required for release ${release.releaseId}`);
      }
      
      if (!release.upc || release.upc.length !== 12) {
        spotifyErrors.push(`Valid UPC required for release ${release.releaseId}`);
      }

      if (!release.genre || !this.isValidSpotifyGenre(release.genre)) {
        spotifyErrors.push(`Invalid Spotify genre for release ${release.releaseId}`);
      }
    });

    result.flat.soundRecordings.forEach(track => {
      if (!track.isrc) {
        spotifyErrors.push(`ISRC required for track ${track.title}`);
      }
      
      if (!track.duration) {
        spotifyErrors.push(`Duration required for track ${track.title}`);
      }
    });

    return {
      isValid: validation.isValid && spotifyErrors.length === 0,
      errors: [...validation.errors, ...spotifyErrors],
      warnings: validation.warnings
    };
  }

  private isValidSpotifyGenre(genre: string): boolean {
    const validGenres = [
      'Pop', 'Rock', 'Hip-Hop', 'Electronic', 'Alternative', 
      'Country', 'R&B', 'Jazz', 'Classical', 'Folk'
    ];
    return validGenres.includes(genre);
  }
}

// Usage example
interface SpotifyReleaseData {
  releaseId: string;
  type: 'Album' | 'Single' | 'EP';
  title: string;
  primaryArtist: string;
  labelName: string;
  upc: string;
  releaseDate: string;
  genre: string;
  hasExplicitContent: boolean;
  spotifyUri?: string;
  primaryMarket: string;
  tracks: SpotifyTrackData[];
}

interface SpotifyTrackData {
  resourceId: string;
  title: string;
  primaryArtist: string;
  featuredArtists?: string[];
  isrc: string;
  duration: string;
  trackNumber: number;
  hasExplicitContent: boolean;
  spotifyUri?: string;
  previewStart?: string;
}

// Example usage
async function submitToSpotify(releaseData: SpotifyReleaseData) {
  const integration = new SpotifyIntegration();
  
  try {
    const xml = await integration.createNewReleaseMessage(releaseData);
    const validation = await integration.validateForSpotify(xml);
    
    if (!validation.isValid) {
      console.error('Validation failed:', validation.errors);
      return;
    }
    
    // Submit to Spotify (implementation depends on delivery method)
    await submitViaSpotifyAPI(xml);
    console.log('Successfully submitted to Spotify');
    
  } catch (error) {
    console.error('Spotify integration failed:', error);
  }
}
```

### Spotify Delivery Implementation

```typescript
class SpotifyDelivery {
  private apiEndpoint = 'https://partners.spotify.com/api/ddex';
  private authToken: string;

  constructor(clientId: string, clientSecret: string) {
    // Initialize authentication
    this.authToken = this.authenticate(clientId, clientSecret);
  }

  async submitDDEX(xml: string, releaseId: string): Promise<SpotifySubmissionResult> {
    const formData = new FormData();
    formData.append('ddex_xml', new Blob([xml], { type: 'application/xml' }));
    formData.append('release_id', releaseId);
    formData.append('submission_type', 'new_release');

    const response = await fetch(`${this.apiEndpoint}/submit`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.authToken}`,
        'Content-Type': 'multipart/form-data'
      },
      body: formData
    });

    if (!response.ok) {
      const error = await response.json();
      throw new SpotifyAPIError(error.message, error.code);
    }

    return await response.json();
  }

  async checkSubmissionStatus(submissionId: string): Promise<SpotifyStatus> {
    const response = await fetch(`${this.apiEndpoint}/status/${submissionId}`, {
      headers: {
        'Authorization': `Bearer ${this.authToken}`
      }
    });

    return await response.json();
  }

  private authenticate(clientId: string, clientSecret: string): string {
    // Implement Spotify OAuth flow
    // Returns access token
    return 'spotify_access_token';
  }
}

interface SpotifySubmissionResult {
  submissionId: string;
  status: 'submitted' | 'processing' | 'approved' | 'rejected';
  message: string;
  estimatedProcessingTime: string;
}

interface SpotifyStatus {
  submissionId: string;
  status: string;
  processedAt?: string;
  errors?: SpotifyError[];
  approvedTracks?: string[];
}

interface SpotifyError {
  code: string;
  message: string;
  field?: string;
  trackId?: string;
}

class SpotifyAPIError extends Error {
  constructor(message: string, public code: string) {
    super(message);
    this.name = 'SpotifyAPIError';
  }
}
```

## Apple Music Integration

### Apple Music Specific Requirements

```typescript
class AppleMusicIntegration {
  private builder: DdexBuilder;

  constructor() {
    this.builder = new DdexBuilder();
    this.builder.applyPreset('apple_music');
  }

  async createiTunesMessage(releaseData: AppleMusicReleaseData): Promise<string> {
    const release = {
      releaseId: releaseData.releaseId,
      releaseType: releaseData.type,
      title: releaseData.title,
      artist: releaseData.primaryArtist,
      label: releaseData.labelName,
      upc: releaseData.upc || releaseData.ean, // Apple accepts both
      releaseDate: releaseData.releaseDate,
      originalReleaseDate: releaseData.originalReleaseDate,
      genre: this.mapToiTunesGenre(releaseData.genre),
      
      // Apple-specific requirements
      copyrightLine: releaseData.copyrightLine, // Required
      producerCopyrightLine: releaseData.producerCopyrightLine, // Required
      explicitContent: releaseData.explicitContent,
      
      // Apple artwork requirements
      artwork: {
        frontCover: {
          imageId: 'ARTWORK_FRONT',
          format: 'JPEG',
          resolution: '3000x3000', // Apple minimum
          colorSpace: 'RGB'
        }
      },
      
      trackIds: releaseData.tracks.map(t => t.resourceId),
      
      // iTunes-specific metadata
      metadata: {
        iTunesGenreId: this.getiTunesGenreId(releaseData.genre),
        priceCategory: releaseData.priceCategory || 'PREMIUM',
        territoryReleaseStrategy: 'WORLDWIDE',
        preorderDate: releaseData.preorderDate
      }
    };

    // Add tracks with Apple-specific metadata
    for (const track of releaseData.tracks) {
      const resource = {
        resourceId: track.resourceId,
        resourceType: 'SoundRecording',
        title: track.title,
        artist: this.formatAppleArtistCredits(track),
        isrc: track.isrc,
        duration: track.duration,
        trackNumber: track.trackNumber,
        discNumber: track.discNumber || 1,
        
        // Apple-specific track metadata
        explicitContent: track.explicitContent,
        previewStartTime: track.previewStartTime || 'PT30S',
        previewDuration: 'PT30S',
        
        // Musical work information (required for some territories)
        musicalWorks: track.musicalWorks?.map(work => ({
          workId: work.workId,
          workTitle: work.title,
          composers: work.composers,
          publishers: work.publishers,
          iswc: work.iswc
        })),
        
        metadata: {
          iTunesTrackId: track.iTunesTrackId,
          isrc: track.isrc,
          audioQuality: 'HIGH', // Apple Music quality tier
          spatialAudio: track.hasSpatialAudio || false
        }
      };

      this.builder.addResource(resource);
    }

    this.builder.addRelease(release);

    return await this.builder.build({
      messageId: `APPLE_${Date.now()}`,
      sender: 'YourLabelDPID',
      recipient: 'AppleDPID',
      version: '4.3'
    });
  }

  private formatAppleArtistCredits(track: AppleMusicTrackData): string {
    // Apple Music specific artist credit formatting
    let credits = track.primaryArtist;
    
    if (track.featuredArtists?.length) {
      credits += ` (feat. ${track.featuredArtists.join(' & ')})`;
    }
    
    return credits;
  }

  private mapToiTunesGenre(genre: string): string {
    // Map to iTunes genre taxonomy
    const iTunesGenreMap: Record<string, string> = {
      'Electronic Dance Music': 'Electronic',
      'Hip Hop': 'Hip-Hop/Rap',
      'Rhythm and Blues': 'R&B/Soul',
      'Country Music': 'Country',
      'Alternative Rock': 'Alternative'
    };

    return iTunesGenreMap[genre] || genre;
  }

  private getiTunesGenreId(genre: string): number {
    // iTunes genre IDs for metadata
    const genreIds: Record<string, number> = {
      'Pop': 14,
      'Rock': 21,
      'Electronic': 7,
      'Hip-Hop/Rap': 18,
      'Country': 6,
      'R&B/Soul': 15,
      'Alternative': 20
    };

    return genreIds[genre] || 14; // Default to Pop
  }

  async validateForiTunes(xml: string): Promise<ValidationResult> {
    const result = await this.builder.validate();
    const appleErrors: string[] = [];

    // Apple-specific validation
    const parsed = await new DDEXParser().parse(xml);
    
    parsed.flat.releases.forEach(release => {
      if (!release.copyrightLine) {
        appleErrors.push(`Copyright line required for release ${release.releaseId}`);
      }
      
      if (!release.producerCopyrightLine) {
        appleErrors.push(`Producer copyright line required for release ${release.releaseId}`);
      }
      
      // Validate artwork requirements
      if (!release.artwork?.frontCover) {
        appleErrors.push(`Front cover artwork required for release ${release.releaseId}`);
      }
    });

    return {
      isValid: result.isValid && appleErrors.length === 0,
      errors: [...result.errors, ...appleErrors],
      warnings: result.warnings
    };
  }
}

interface AppleMusicReleaseData {
  releaseId: string;
  type: 'Album' | 'Single' | 'EP' | 'Compilation';
  title: string;
  primaryArtist: string;
  labelName: string;
  upc?: string;
  ean?: string;
  releaseDate: string;
  originalReleaseDate?: string;
  genre: string;
  copyrightLine: string;
  producerCopyrightLine: string;
  explicitContent: boolean;
  priceCategory?: string;
  preorderDate?: string;
  tracks: AppleMusicTrackData[];
}

interface AppleMusicTrackData {
  resourceId: string;
  title: string;
  primaryArtist: string;
  featuredArtists?: string[];
  isrc: string;
  duration: string;
  trackNumber: number;
  discNumber?: number;
  explicitContent: boolean;
  previewStartTime?: string;
  iTunesTrackId?: string;
  hasSpatialAudio?: boolean;
  musicalWorks?: MusicalWork[];
}

interface MusicalWork {
  workId: string;
  title: string;
  composers: string[];
  publishers: string[];
  iswc?: string;
}
```

## YouTube Music Integration

### YouTube Content ID Compliance

```typescript
class YouTubeMusicIntegration {
  private builder: DdexBuilder;

  constructor() {
    this.builder = new DdexBuilder();
    this.builder.applyPreset('youtube_music');
  }

  async createContentIDMessage(releaseData: YouTubeReleaseData): Promise<string> {
    const release = {
      releaseId: releaseData.releaseId,
      releaseType: releaseData.type,
      title: releaseData.title,
      artist: releaseData.primaryArtist,
      label: releaseData.labelName,
      releaseDate: releaseData.releaseDate,
      genre: releaseData.genre,
      
      // YouTube-specific requirements
      trackIds: releaseData.tracks.map(t => t.resourceId),
      
      metadata: {
        youtubeChannelId: releaseData.channelId,
        contentIdPolicy: releaseData.contentIdPolicy || 'MONETIZE',
        territoryRights: releaseData.territoryRights,
        referenceFileQuality: 'HIGH'
      }
    };

    // Add tracks with Content ID metadata
    for (const track of releaseData.tracks) {
      const resource = {
        resourceId: track.resourceId,
        resourceType: 'SoundRecording',
        title: track.title,
        artist: track.primaryArtist,
        isrc: track.isrc, // Required for Content ID
        duration: track.duration,
        
        // Content ID specific metadata
        metadata: {
          youtubeAssetId: track.assetId,
          referenceFile: {
            url: track.referenceFileUrl,
            format: 'WAV',
            quality: 'LOSSLESS',
            fingerprint: track.audioFingerprint
          },
          ownershipClaims: track.ownershipClaims,
          monetizationPolicy: track.monetizationPolicy || 'MONETIZE',
          matchPolicy: track.matchPolicy || 'TRACK'
        }
      };

      this.builder.addResource(resource);
    }

    // Add specific YouTube deals
    const deal = {
      dealId: `YOUTUBE_${releaseData.releaseId}`,
      dealType: 'License',
      commercialModelType: 'AdSupportedModel',
      usage: ['OnDemandStream', 'NonInteractiveStream'],
      territory: releaseData.territoryRights,
      distributionChannel: 'Internet',
      
      // YouTube-specific deal terms
      dealTerms: {
        monetizationRights: true,
        contentIdRights: true,
        userGeneratedContentRights: releaseData.allowUserUploads,
        syncRights: releaseData.allowSynchronization
      },
      
      releaseReferences: [releaseData.releaseId]
    };

    this.builder.addDeal(deal);
    this.builder.addRelease(release);

    return await this.builder.build({
      messageId: `YOUTUBE_${Date.now()}`,
      sender: 'YourLabelDPID',
      recipient: 'YouTubeDPID',
      version: '4.3'
    });
  }

  async submitToContentID(xml: string, releaseData: YouTubeReleaseData): Promise<YouTubeSubmissionResult> {
    // YouTube Content ID API integration
    const contentIdAPI = new YouTubeContentIDAPI();
    
    // Upload reference files first
    const assetIds: string[] = [];
    
    for (const track of releaseData.tracks) {
      const asset = await contentIdAPI.createAsset({
        title: track.title,
        artist: track.primaryArtist,
        isrc: track.isrc,
        referenceFileUrl: track.referenceFileUrl
      });
      
      assetIds.push(asset.assetId);
    }
    
    // Submit DDEX metadata
    const submission = await contentIdAPI.submitDDEX(xml, {
      assetIds,
      channelId: releaseData.channelId,
      contentIdPolicy: releaseData.contentIdPolicy
    });
    
    return submission;
  }
}

interface YouTubeReleaseData {
  releaseId: string;
  type: string;
  title: string;
  primaryArtist: string;
  labelName: string;
  releaseDate: string;
  genre: string;
  channelId: string;
  contentIdPolicy: 'MONETIZE' | 'TRACK' | 'BLOCK';
  territoryRights: string[];
  allowUserUploads: boolean;
  allowSynchronization: boolean;
  tracks: YouTubeTrackData[];
}

interface YouTubeTrackData {
  resourceId: string;
  title: string;
  primaryArtist: string;
  isrc: string;
  duration: string;
  assetId?: string;
  referenceFileUrl: string;
  audioFingerprint?: string;
  ownershipClaims: OwnershipClaim[];
  monetizationPolicy?: string;
  matchPolicy?: string;
}

interface OwnershipClaim {
  territory: string[];
  rightsType: 'SOUND_RECORDING' | 'MUSICAL_WORK';
  ownershipPercentage: number;
  ownerName: string;
}

interface YouTubeSubmissionResult {
  submissionId: string;
  status: 'submitted' | 'processing' | 'live' | 'rejected';
  assetIds: string[];
  estimatedProcessingTime: string;
  message: string;
}

class YouTubeContentIDAPI {
  async createAsset(assetData: any): Promise<{ assetId: string }> {
    // Implementation for YouTube Content ID API
    return { assetId: `ASSET_${Date.now()}` };
  }

  async submitDDEX(xml: string, options: any): Promise<YouTubeSubmissionResult> {
    // Implementation for DDEX submission to YouTube
    return {
      submissionId: `SUBMISSION_${Date.now()}`,
      status: 'submitted',
      assetIds: options.assetIds,
      estimatedProcessingTime: '24-48 hours',
      message: 'DDEX submission received'
    };
  }
}
```

## Amazon Music Integration

```typescript
class AmazonMusicIntegration {
  private builder: DdexBuilder;

  constructor() {
    this.builder = new DdexBuilder();
    this.builder.applyPreset('amazon_music');
  }

  async createAmazonMessage(releaseData: AmazonReleaseData): Promise<string> {
    const release = {
      releaseId: releaseData.releaseId,
      releaseType: releaseData.type,
      title: releaseData.title,
      artist: releaseData.primaryArtist,
      label: releaseData.labelName,
      catalogNumber: releaseData.catalogNumber, // Important for Amazon
      upc: releaseData.upc,
      releaseDate: releaseData.releaseDate,
      genre: releaseData.genre,
      
      // Amazon-specific metadata
      trackIds: releaseData.tracks.map(t => t.resourceId),
      
      metadata: {
        amazonASIN: releaseData.asin,
        productType: releaseData.productType, // 'DIGITAL_MUSIC_TRACK' or 'DIGITAL_MUSIC_ALBUM'
        priceCategory: releaseData.priceCategory,
        keywords: releaseData.searchKeywords,
        brandName: releaseData.brandName || releaseData.labelName
      }
    };

    // Add tracks with Amazon-specific requirements
    for (const track of releaseData.tracks) {
      const resource = {
        resourceId: track.resourceId,
        resourceType: 'SoundRecording',
        title: track.title,
        artist: track.primaryArtist,
        isrc: track.isrc,
        duration: track.duration,
        trackNumber: track.trackNumber,
        
        metadata: {
          amazonTrackASIN: track.asin,
          audioQuality: track.quality || 'STANDARD',
          fileFormat: 'MP3',
          bitrate: '320',
          sampleRate: '44100'
        }
      };

      this.builder.addResource(resource);
    }

    this.builder.addRelease(release);

    return await this.builder.build({
      messageId: `AMAZON_${Date.now()}`,
      sender: 'YourLabelDPID',
      recipient: 'AmazonDPID',
      version: '4.3'
    });
  }
}

interface AmazonReleaseData {
  releaseId: string;
  type: string;
  title: string;
  primaryArtist: string;
  labelName: string;
  catalogNumber: string;
  upc: string;
  releaseDate: string;
  genre: string;
  asin?: string;
  productType: 'DIGITAL_MUSIC_TRACK' | 'DIGITAL_MUSIC_ALBUM';
  priceCategory: string;
  searchKeywords: string[];
  brandName?: string;
  tracks: AmazonTrackData[];
}

interface AmazonTrackData {
  resourceId: string;
  title: string;
  primaryArtist: string;
  isrc: string;
  duration: string;
  trackNumber: number;
  asin?: string;
  quality?: 'STANDARD' | 'HIGH' | 'ULTRA_HD';
}
```

## Multi-Platform Batch Processing

```python
# Python implementation for processing multiple DSPs
from ddex_builder import DdexBuilder
from ddex_parser import DDEXParser
import asyncio
from typing import Dict, List

class MultiDSPProcessor:
    def __init__(self):
        self.parsers = {}
        self.builders = {}
        
        # Initialize platform-specific builders
        for platform in ['spotify', 'apple_music', 'youtube_music', 'amazon_music']:
            builder = DdexBuilder()
            builder.apply_preset(platform)
            self.builders[platform] = builder
            self.parsers[platform] = DDEXParser()
    
    async def process_release_for_all_platforms(self, release_data: dict) -> Dict[str, str]:
        """Generate DDEX for all platforms simultaneously"""
        
        tasks = []
        for platform in self.builders.keys():
            task = self.create_platform_ddex(platform, release_data)
            tasks.append(task)
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        platform_xmls = {}
        for platform, result in zip(self.builders.keys(), results):
            if isinstance(result, Exception):
                print(f"Failed to generate DDEX for {platform}: {result}")
            else:
                platform_xmls[platform] = result
        
        return platform_xmls
    
    async def create_platform_ddex(self, platform: str, release_data: dict) -> str:
        """Create DDEX for specific platform"""
        builder = self.builders[platform]
        builder.reset()  # Clear previous data
        
        # Adapt release data for platform
        adapted_release = self.adapt_release_for_platform(release_data, platform)
        
        # Add release and resources
        builder.add_release(adapted_release)
        
        for track in release_data['tracks']:
            adapted_track = self.adapt_track_for_platform(track, platform)
            builder.add_resource(adapted_track)
        
        # Generate XML
        xml = await builder.build({
            'message_id': f"{platform.upper()}_{release_data['release_id']}",
            'sender': release_data['sender_dpid'],
            'recipient': self.get_platform_dpid(platform),
            'version': '4.3'
        })
        
        return xml
    
    def adapt_release_for_platform(self, release_data: dict, platform: str) -> dict:
        """Adapt release data for platform-specific requirements"""
        
        base_release = {
            'release_id': release_data['release_id'],
            'release_type': release_data['type'],
            'title': release_data['title'],
            'artist': release_data['primary_artist'],
            'label': release_data['label_name'],
            'release_date': release_data['release_date'],
            'genre': release_data['genre'],
            'track_ids': [t['resource_id'] for t in release_data['tracks']]
        }
        
        # Platform-specific adaptations
        if platform == 'spotify':
            # Spotify requires UPC and specific genre mapping
            base_release['upc'] = release_data['upc']
            base_release['genre'] = self.map_genre_for_spotify(release_data['genre'])
            
        elif platform == 'apple_music':
            # Apple Music requires copyright lines
            base_release['copyright_line'] = release_data.get('copyright_line', f"© {release_data['copyright_year']} {release_data['label_name']}")
            base_release['producer_copyright_line'] = release_data.get('producer_copyright_line', f"℗ {release_data['copyright_year']} {release_data['label_name']}")
            base_release['upc'] = release_data.get('upc') or release_data.get('ean')
            
        elif platform == 'youtube_music':
            # YouTube Music requires ISRC for Content ID
            base_release['metadata'] = {
                'content_id_policy': 'MONETIZE',
                'territory_rights': release_data.get('territory_rights', ['WORLDWIDE'])
            }
            
        elif platform == 'amazon_music':
            # Amazon requires catalog number and product classification
            base_release['catalog_number'] = release_data['catalog_number']
            base_release['metadata'] = {
                'product_type': 'DIGITAL_MUSIC_ALBUM' if release_data['type'] == 'Album' else 'DIGITAL_MUSIC_TRACK',
                'search_keywords': release_data.get('keywords', [])
            }
        
        return base_release
    
    def adapt_track_for_platform(self, track_data: dict, platform: str) -> dict:
        """Adapt track data for platform-specific requirements"""
        
        base_track = {
            'resource_id': track_data['resource_id'],
            'resource_type': 'SoundRecording',
            'title': track_data['title'],
            'artist': track_data['primary_artist'],
            'isrc': track_data['isrc'],
            'duration': track_data['duration'],
            'track_number': track_data['track_number']
        }
        
        # Platform-specific track adaptations
        if platform == 'spotify':
            # Spotify-specific preview settings
            base_track['metadata'] = {
                'preview_start_time': track_data.get('preview_start', 'PT30S')
            }
            
        elif platform == 'apple_music':
            # Apple Music spatial audio support
            base_track['metadata'] = {
                'spatial_audio': track_data.get('has_spatial_audio', False),
                'preview_start_time': track_data.get('preview_start', 'PT30S')
            }
            
        elif platform == 'youtube_music':
            # YouTube Content ID reference file
            base_track['metadata'] = {
                'reference_file_url': track_data.get('reference_file_url'),
                'monetization_policy': 'MONETIZE'
            }
            
        elif platform == 'amazon_music':
            # Amazon quality specifications
            base_track['metadata'] = {
                'audio_quality': 'STANDARD',
                'file_format': 'MP3'
            }
        
        return base_track
    
    def map_genre_for_spotify(self, genre: str) -> str:
        """Map genres to Spotify's approved list"""
        genre_mapping = {
            'Electronic Dance Music': 'Electronic',
            'Hip Hop': 'Hip-Hop',
            'Rhythm and Blues': 'R&B',
            'Alternative Rock': 'Alternative'
        }
        return genre_mapping.get(genre, genre)
    
    def get_platform_dpid(self, platform: str) -> str:
        """Get DPID for each platform"""
        dpids = {
            'spotify': 'PADPIDA2014101001U',  # Example Spotify DPID
            'apple_music': 'PADPIDA2014101002U',  # Example Apple DPID
            'youtube_music': 'PADPIDA2014101003U',  # Example YouTube DPID
            'amazon_music': 'PADPIDA2014101004U'  # Example Amazon DPID
        }
        return dpids.get(platform, 'UNKNOWN')

# Usage example
async def distribute_release():
    processor = MultiDSPProcessor()
    
    release_data = {
        'release_id': 'REL_2024_001',
        'type': 'Album',
        'title': 'My New Album',
        'primary_artist': 'Artist Name',
        'label_name': 'Record Label',
        'upc': '123456789012',
        'release_date': '2024-03-15',
        'genre': 'Pop',
        'copyright_year': '2024',
        'catalog_number': 'CAT001',
        'sender_dpid': 'PADPIDA2024010101U',
        'tracks': [
            {
                'resource_id': 'TRK_001',
                'title': 'Hit Single',
                'primary_artist': 'Artist Name',
                'isrc': 'USRC12345678',
                'duration': 'PT3M45S',
                'track_number': 1,
                'preview_start': 'PT30S'
            }
        ]
    }
    
    platform_xmls = await processor.process_release_for_all_platforms(release_data)
    
    # Save platform-specific XML files
    for platform, xml in platform_xmls.items():
        with open(f'release_{platform}.xml', 'w') as f:
            f.write(xml)
        print(f"Generated DDEX for {platform}")

# Run the distribution
asyncio.run(distribute_release())
```

## Performance Considerations

### Parallel Processing

```typescript
// Process multiple platforms concurrently
class ConcurrentDSPProcessor {
  async generateForAllPlatforms(releaseData: ReleaseData): Promise<Map<string, string>> {
    const platforms = ['spotify', 'apple_music', 'youtube_music', 'amazon_music'];
    
    const promises = platforms.map(async platform => {
      const integration = this.createIntegration(platform);
      const xml = await integration.createMessage(releaseData);
      return [platform, xml] as [string, string];
    });
    
    const results = await Promise.allSettled(promises);
    const xmlMap = new Map<string, string>();
    
    results.forEach((result, index) => {
      if (result.status === 'fulfilled') {
        const [platform, xml] = result.value;
        xmlMap.set(platform, xml);
      } else {
        console.error(`Failed to generate for ${platforms[index]}:`, result.reason);
      }
    });
    
    return xmlMap;
  }
  
  private createIntegration(platform: string): BaseDSPIntegration {
    switch (platform) {
      case 'spotify': return new SpotifyIntegration();
      case 'apple_music': return new AppleMusicIntegration();
      case 'youtube_music': return new YouTubeMusicIntegration();
      case 'amazon_music': return new AmazonMusicIntegration();
      default: throw new Error(`Unknown platform: ${platform}`);
    }
  }
}
```

## Common Pitfalls and Solutions

### Pitfall 1: Genre Inconsistencies

```typescript
// WRONG: Using source genres directly
const release = {
  genre: sourceData.genre // May not match platform requirements
};

// RIGHT: Platform-specific genre mapping
const release = {
  genre: this.mapGenreForPlatform(sourceData.genre, platform)
};
```

### Pitfall 2: Missing Required Fields

```typescript
// WRONG: Generic validation
if (!release.title) throw new Error('Title required');

// RIGHT: Platform-specific validation
const validation = await this.validateForPlatform(xml, platform);
if (!validation.isValid) {
  throw new Error(`Validation failed for ${platform}: ${validation.errors.join(', ')}`);
}
```

### Pitfall 3: Hard-coded Values

```typescript
// WRONG: Hard-coded recipient
const xml = await builder.build({
  recipient: 'SpotifyDPID' // Breaks for other platforms
});

// RIGHT: Dynamic recipient based on platform
const xml = await builder.build({
  recipient: this.getPlatformDPID(platform)
});
```

## Testing and Validation

### Sandbox Testing

```typescript
class DSPTestingFramework {
  async testAllPlatforms(releaseData: ReleaseData): Promise<TestResults> {
    const results: TestResults = {};
    
    for (const platform of ['spotify', 'apple_music', 'youtube_music', 'amazon_music']) {
      try {
        const xml = await this.generateForPlatform(releaseData, platform);
        const validation = await this.validateForPlatform(xml, platform);
        
        results[platform] = {
          generated: true,
          valid: validation.isValid,
          errors: validation.errors,
          warnings: validation.warnings,
          xml: xml
        };
        
        // Test submission to sandbox if available
        if (this.hasSandbox(platform)) {
          const submission = await this.submitToSandbox(xml, platform);
          results[platform].sandboxResult = submission;
        }
        
      } catch (error) {
        results[platform] = {
          generated: false,
          valid: false,
          errors: [error.message],
          warnings: [],
          xml: null
        };
      }
    }
    
    return results;
  }
}

interface TestResults {
  [platform: string]: {
    generated: boolean;
    valid: boolean;
    errors: string[];
    warnings: string[];
    xml: string | null;
    sandboxResult?: any;
  };
}
```

## Links to API Documentation

- [Builder Presets](/api/builder/presets) - Platform-specific preset configurations
- [Validation API](/api/builder/typescript#validate) - Validation methods and error handling
- [Error Handling Guide](/guides/error-handling) - Comprehensive error handling patterns
- [Performance Tuning](/guides/performance-tuning) - Optimization for high-volume processing

This guide provides a complete framework for integrating with major DSPs using the DDEX Suite, ensuring compliance while maintaining development efficiency.