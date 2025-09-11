---
sidebar_position: 2
---

# Building a New Release

Learn how to create DDEX ERN messages from scratch using the DDEX Builder. This guide covers building complete release notifications with all required metadata.

## Basic Release Creation

### JavaScript / TypeScript

```typescript
import { DdexBuilder } from 'ddex-builder';

async function createSimpleRelease() {
  const builder = new DdexBuilder({
    validate: true,
    canonical: true,
    preset: 'generic_audio_album'
  });

  const releaseData = {
    messageHeader: {
      messageId: `REL_${Date.now()}`,
      messageSenderName: 'Independent Music Label',
      messageRecipientName: 'Digital Music Platform',
      messageCreatedDateTime: new Date().toISOString()
    },
    
    releases: [{
      releaseId: 'REL_2024_001',
      title: 'Summer Vibes EP',
      artist: 'The Sunset Band',
      releaseType: 'EP',
      label: 'Independent Music Label',
      upc: '123456789012',
      releaseDate: '2024-06-21',
      originalReleaseDate: '2024-06-21',
      territories: ['US', 'CA', 'GB', 'DE', 'FR'],
      genres: ['Pop', 'Electronic', 'Indie'],
      parentalWarning: false,
      trackIds: ['SR_001', 'SR_002', 'SR_003', 'SR_004']
    }],
    
    resources: [
      {
        resourceId: 'SR_001',
        resourceType: 'SoundRecording',
        title: 'Sunset Boulevard',
        artist: 'The Sunset Band',
        isrc: 'US-IND-24-00001',
        duration: 'PT3M45S',
        trackNumber: 1,
        volumeNumber: 1,
        languageOfPerformance: 'en',
        metadata: {
          composer: 'Jane Smith, John Doe',
          lyricist: 'Jane Smith',
          producer: 'Mike Producer',
          recordingYear: '2024'
        }
      },
      {
        resourceId: 'SR_002',
        resourceType: 'SoundRecording',
        title: 'Ocean Dreams',
        artist: 'The Sunset Band',
        isrc: 'US-IND-24-00002',
        duration: 'PT4M12S',
        trackNumber: 2,
        volumeNumber: 1,
        languageOfPerformance: 'en',
        metadata: {
          composer: 'John Doe',
          lyricist: 'John Doe',
          producer: 'Mike Producer',
          recordingYear: '2024'
        }
      },
      {
        resourceId: 'SR_003',
        resourceType: 'SoundRecording',
        title: 'City Lights',
        artist: 'The Sunset Band',
        isrc: 'US-IND-24-00003',
        duration: 'PT3M58S',
        trackNumber: 3,
        volumeNumber: 1,
        languageOfPerformance: 'en',
        metadata: {
          composer: 'Jane Smith',
          lyricist: 'Jane Smith',
          producer: 'Mike Producer',
          recordingYear: '2024'
        }
      },
      {
        resourceId: 'SR_004',
        resourceType: 'SoundRecording',
        title: 'Midnight Drive',
        artist: 'The Sunset Band',
        isrc: 'US-IND-24-00004',
        duration: 'PT4M33S',
        trackNumber: 4,
        volumeNumber: 1,
        languageOfPerformance: 'en',
        metadata: {
          composer: 'Jane Smith, John Doe',
          lyricist: 'Jane Smith',
          producer: 'Mike Producer',
          recordingYear: '2024'
        }
      }
    ],
    
    deals: [{
      dealId: 'DEAL_001',
      releaseId: 'REL_2024_001',
      territories: ['US', 'CA', 'GB', 'DE', 'FR'],
      useTypes: ['Stream', 'PermanentDownload', 'ConditionalDownload'],
      commercialModelType: 'Subscription',
      dealStartDate: '2024-06-21',
      priceInformation: {
        priceCurrency: 'USD',
        wholesalePrice: 7.99
      }
    }]
  };

  try {
    const xml = await builder.build(releaseData);
    console.log('✅ Successfully built ERN message');
    console.log(`📄 Generated XML (${xml.length} bytes)`);
    
    // Save to file
    await fs.writeFile('summer-vibes-ep.xml', xml, 'utf-8');
    console.log('💾 Saved to summer-vibes-ep.xml');
    
    return xml;
  } catch (error) {
    console.error('❌ Build failed:', error.message);
    throw error;
  }
}

// Usage
createSimpleRelease();
```

### Python

```python
from ddex_builder import DdexBuilder
from datetime import datetime
import json

def create_simple_release():
    """Create a simple release ERN message."""
    
    builder = DdexBuilder(
        validate=True,
        canonical=True,
        preset='generic_audio_album'
    )
    
    release_data = {
        'message_header': {
            'message_id': f'REL_{int(datetime.now().timestamp())}',
            'message_sender_name': 'Independent Music Label',
            'message_recipient_name': 'Digital Music Platform',
            'message_created_date_time': datetime.now().isoformat()
        },
        
        'releases': [{
            'release_id': 'REL_2024_001',
            'title': 'Summer Vibes EP',
            'artist': 'The Sunset Band',
            'release_type': 'EP',
            'label': 'Independent Music Label',
            'upc': '123456789012',
            'release_date': '2024-06-21',
            'original_release_date': '2024-06-21',
            'territories': ['US', 'CA', 'GB', 'DE', 'FR'],
            'genres': ['Pop', 'Electronic', 'Indie'],
            'parental_warning': False,
            'track_ids': ['SR_001', 'SR_002', 'SR_003', 'SR_004']
        }],
        
        'resources': [
            {
                'resource_id': 'SR_001',
                'resource_type': 'SoundRecording',
                'title': 'Sunset Boulevard',
                'artist': 'The Sunset Band',
                'isrc': 'US-IND-24-00001',
                'duration': 'PT3M45S',
                'track_number': 1,
                'volume_number': 1,
                'language_of_performance': 'en',
                'metadata': {
                    'composer': 'Jane Smith, John Doe',
                    'lyricist': 'Jane Smith',
                    'producer': 'Mike Producer',
                    'recording_year': '2024'
                }
            },
            {
                'resource_id': 'SR_002',
                'resource_type': 'SoundRecording',
                'title': 'Ocean Dreams',
                'artist': 'The Sunset Band',
                'isrc': 'US-IND-24-00002',
                'duration': 'PT4M12S',
                'track_number': 2,
                'volume_number': 1,
                'language_of_performance': 'en',
                'metadata': {
                    'composer': 'John Doe',
                    'lyricist': 'John Doe',
                    'producer': 'Mike Producer',
                    'recording_year': '2024'
                }
            },
            {
                'resource_id': 'SR_003',
                'resource_type': 'SoundRecording',
                'title': 'City Lights',
                'artist': 'The Sunset Band',
                'isrc': 'US-IND-24-00003',
                'duration': 'PT3M58S',
                'track_number': 3,
                'volume_number': 1,
                'language_of_performance': 'en',
                'metadata': {
                    'composer': 'Jane Smith',
                    'lyricist': 'Jane Smith',
                    'producer': 'Mike Producer',
                    'recording_year': '2024'
                }
            },
            {
                'resource_id': 'SR_004',
                'resource_type': 'SoundRecording',
                'title': 'Midnight Drive',
                'artist': 'The Sunset Band',
                'isrc': 'US-IND-24-00004',
                'duration': 'PT4M33S',
                'track_number': 4,
                'volume_number': 1,
                'language_of_performance': 'en',
                'metadata': {
                    'composer': 'Jane Smith, John Doe',
                    'lyricist': 'Jane Smith',
                    'producer': 'Mike Producer',
                    'recording_year': '2024'
                }
            }
        ],
        
        'deals': [{
            'deal_id': 'DEAL_001',
            'release_id': 'REL_2024_001',
            'territories': ['US', 'CA', 'GB', 'DE', 'FR'],
            'use_types': ['Stream', 'PermanentDownload', 'ConditionalDownload'],
            'commercial_model_type': 'Subscription',
            'deal_start_date': '2024-06-21',
            'price_information': {
                'price_currency': 'USD',
                'wholesale_price': 7.99
            }
        }]
    }
    
    try:
        xml = builder.build(release_data)
        print('✅ Successfully built ERN message')
        print(f'📄 Generated XML ({len(xml)} bytes)')
        
        # Save to file
        with open('summer-vibes-ep.xml', 'w', encoding='utf-8') as f:
            f.write(xml)
        print('💾 Saved to summer-vibes-ep.xml')
        
        return xml
        
    except Exception as e:
        print(f'❌ Build failed: {e}')
        raise

# Usage
if __name__ == "__main__":
    xml = create_simple_release()
```

## Building with Platform Presets

### Spotify-Optimized Release

```typescript
async function createSpotifyRelease() {
  const builder = new DdexBuilder();
  builder.applyPreset('youtube_album');

  const spotifyRelease = {
    messageHeader: {
      messageId: `SPOTIFY_${Date.now()}`,
      messageSenderName: 'Streaming Music Label',
      messageRecipientName: 'Spotify',
      messageCreatedDateTime: new Date().toISOString()
    },
    
    releases: [{
      releaseId: 'SPOT_REL_001',
      title: 'Viral Indie Hits',
      artist: 'The Streaming Stars',
      releaseType: 'Album',
      label: 'Streaming Music Label',
      upc: '987654321098',
      releaseDate: '2024-07-15',
      territories: ['Worldwide'], // Spotify supports worldwide
      genres: ['Indie Pop', 'Alternative'], // Normalized for Spotify
      explicitContent: false, // Required by Spotify preset
      spotifyArtistId: 'spotify:artist:example123', // Platform-specific ID
      trackIds: ['SP_001', 'SP_002']
    }],
    
    resources: [
      {
        resourceId: 'SP_001',
        resourceType: 'SoundRecording',
        title: 'Indie Anthem',
        artist: 'The Streaming Stars',
        isrc: 'US-STR-24-00001',
        duration: 'PT3M30S', // Meets Spotify minimum
        trackNumber: 1,
        explicitContent: false, // Track-level flag
        metadata: {
          spotifyPreviewStartTime: '45', // 45 seconds in
          canvasVideoId: 'canvas_123' // For Spotify Canvas
        }
      },
      {
        resourceId: 'SP_002',
        resourceType: 'SoundRecording',
        title: 'Alternative Vibes',
        artist: 'The Streaming Stars',
        isrc: 'US-STR-24-00002',
        duration: 'PT4M15S',
        trackNumber: 2,
        explicitContent: false,
        metadata: {
          spotifyPreviewStartTime: '60'
        }
      }
    ],
    
    deals: [{
      dealId: 'SPOTIFY_DEAL_001',
      releaseId: 'SPOT_REL_001',
      territories: ['Worldwide'],
      useTypes: ['Stream'], // Spotify streaming only
      commercialModelType: 'Subscription',
      dealStartDate: '2024-07-15',
      platformSpecific: {
        spotify: {
          playlistEligible: true,
          algorithmicPlaylistEligible: true
        }
      }
    }]
  };

  const xml = await builder.build(spotifyRelease);
  await fs.writeFile('spotify-release.xml', xml, 'utf-8');
  console.log('🎵 Built Spotify-optimized release');
  
  return xml;
}
```

### Apple Music Release

```typescript
async function createAppleMusicRelease() {
  const builder = new DdexBuilder();
  builder.applyPreset('apple_music');

  const appleMusicRelease = {
    messageHeader: {
      messageId: `APPLE_${Date.now()}`,
      messageSenderName: 'Premium Audio Label',
      messageRecipientName: 'Apple Music',
      messageCreatedDateTime: new Date().toISOString()
    },
    
    releases: [{
      releaseId: 'APPLE_REL_001',
      title: 'Audiophile Collection',
      artist: 'High Fidelity Ensemble',
      releaseType: 'Album',
      label: 'Premium Audio Label',
      upc: '456789012345',
      releaseDate: '2024-08-01',
      preOrderDate: '2024-07-15', // Apple supports pre-orders
      territories: ['US', 'CA', 'GB', 'AU', 'JP'],
      genres: ['Jazz', 'Classical Crossover'],
      masteredForItunes: true, // Apple quality certification
      priceTier: 'tier_3', // Apple pricing tier
      artworkResolution: '3000x3000', // High-res for Apple
      trackIds: ['AP_001', 'AP_002']
    }],
    
    resources: [
      {
        resourceId: 'AP_001',
        resourceType: 'SoundRecording',
        title: 'Jazz Fusion Masterpiece',
        artist: 'High Fidelity Ensemble',
        isrc: 'US-HiFi-24-00001',
        duration: 'PT5M30S',
        trackNumber: 1,
        audioQuality: 'lossless', // Apple Lossless
        spatialAudio: 'dolby_atmos', // Dolby Atmos mixing
        appleCertified: true, // Apple Digital Masters
        metadata: {
          composer: 'Jazz Composer',
          conductor: 'Orchestra Leader',
          recordingLocation: 'Abbey Road Studios'
        }
      },
      {
        resourceId: 'AP_002',
        resourceType: 'SoundRecording',
        title: 'Classical Innovation',
        artist: 'High Fidelity Ensemble',
        isrc: 'US-HiFi-24-00002',
        duration: 'PT6M45S',
        trackNumber: 2,
        audioQuality: 'lossless',
        spatialAudio: 'dolby_atmos',
        appleCertified: true,
        metadata: {
          composer: 'Modern Classical Composer',
          conductor: 'Orchestra Leader',
          recordingLocation: 'Abbey Road Studios'
        }
      }
    ],
    
    deals: [{
      dealId: 'APPLE_DEAL_001',
      releaseId: 'APPLE_REL_001',
      territories: ['US', 'CA', 'GB', 'AU', 'JP'],
      useTypes: ['Stream', 'PermanentDownload'],
      commercialModelType: 'SubscriptionAndPurchase',
      dealStartDate: '2024-08-01',
      platformSpecific: {
        appleMusi: {
          eligibleForRadio: true,
          qualityTier: 'lossless_plus'
        }
      }
    }]
  };

  const xml = await builder.build(appleMusicRelease);
  await fs.writeFile('apple-music-release.xml', xml, 'utf-8');
  console.log('🍎 Built Apple Music-optimized release');
  
  return xml;
}
```

## Complex Album with Multiple Artists

```typescript
async function createComplexAlbum() {
  const builder = new DdexBuilder({
    validate: true,
    canonical: true,
    preset: 'generic_audio_album'
  });

  const albumData = {
    messageHeader: {
      messageId: `COMPLEX_${Date.now()}`,
      messageSenderName: 'Major Record Label',
      messageRecipientName: 'Global Distribution Network',
      messageCreatedDateTime: new Date().toISOString(),
      messageControlType: 'LiveMessage'
    },
    
    // Multiple parties involved
    parties: [
      {
        partyId: 'LABEL_001',
        partyName: 'Major Record Label',
        partyType: 'Label',
        contactInfo: {
          email: 'contact@majorlabel.com',
          website: 'https://majorlabel.com'
        }
      },
      {
        partyId: 'ARTIST_001',
        partyName: 'Lead Vocalist',
        partyType: 'MainArtist',
        contactInfo: {
          email: 'vocalist@example.com'
        }
      },
      {
        partyId: 'ARTIST_002',
        partyName: 'Featured Rapper',
        partyType: 'FeaturedArtist'
      },
      {
        partyId: 'PRODUCER_001',
        partyName: 'Star Producer',
        partyType: 'Producer'
      }
    ],
    
    releases: [{
      releaseId: 'COMPLEX_ALB_001',
      title: 'Genre-Defying Album',
      artist: 'Lead Vocalist',
      releaseType: 'Album',
      label: 'Major Record Label',
      labelId: 'LABEL_001',
      upc: '789012345678',
      releaseDate: '2024-09-15',
      originalReleaseDate: '2024-09-15',
      territories: ['WorldWide'],
      genres: ['Pop', 'Hip-Hop', 'Electronic', 'R&B'],
      parentalWarning: true, // Album has explicit content
      pLine: '℗ 2024 Major Record Label',
      cLine: '© 2024 Major Record Label',
      trackIds: ['COMP_001', 'COMP_002', 'COMP_003', 'COMP_004', 'COMP_005']
    }],
    
    resources: [
      {
        resourceId: 'COMP_001',
        resourceType: 'SoundRecording',
        title: 'Opening Statement',
        artist: 'Lead Vocalist',
        isrc: 'US-MAJ-24-00001',
        duration: 'PT1M30S', // Album intro
        trackNumber: 1,
        volumeNumber: 1,
        languageOfPerformance: 'en',
        parentalWarning: false,
        metadata: {
          composer: 'Lead Vocalist, Star Producer',
          lyricist: 'Lead Vocalist',
          producer: 'Star Producer',
          recordingYear: '2024',
          recordingLocation: 'Abbey Road Studios',
          mixedBy: 'Mix Engineer',
          masteredBy: 'Mastering Engineer'
        }
      },
      {
        resourceId: 'COMP_002',
        resourceType: 'SoundRecording',
        title: 'Mainstream Appeal',
        artist: 'Lead Vocalist',
        isrc: 'US-MAJ-24-00002',
        duration: 'PT3M45S',
        trackNumber: 2,
        volumeNumber: 1,
        languageOfPerformance: 'en',
        parentalWarning: false,
        metadata: {
          composer: 'Lead Vocalist, Pop Songwriter',
          lyricist: 'Pop Songwriter',
          producer: 'Star Producer',
          recordingYear: '2024',
          genre: 'Pop'
        }
      },
      {
        resourceId: 'COMP_003',
        resourceType: 'SoundRecording',
        title: 'Collaboration Fire',
        artist: 'Lead Vocalist feat. Featured Rapper',
        contributingArtists: [
          {
            name: 'Featured Rapper',
            role: 'FeaturedArtist',
            partyId: 'ARTIST_002'
          }
        ],
        isrc: 'US-MAJ-24-00003',
        duration: 'PT4M20S',
        trackNumber: 3,
        volumeNumber: 1,
        languageOfPerformance: 'en',
        parentalWarning: true, // Explicit lyrics
        metadata: {
          composer: 'Lead Vocalist, Featured Rapper, Star Producer',
          lyricist: 'Lead Vocalist, Featured Rapper',
          producer: 'Star Producer',
          recordingYear: '2024',
          genre: 'Hip-Hop'
        }
      },
      {
        resourceId: 'COMP_004',
        resourceType: 'SoundRecording',
        title: 'Electronic Experiment',
        artist: 'Lead Vocalist',
        isrc: 'US-MAJ-24-00004',
        duration: 'PT5M15S',
        trackNumber: 4,
        volumeNumber: 1,
        languageOfPerformance: 'en',
        parentalWarning: false,
        metadata: {
          composer: 'Lead Vocalist, Electronic Producer',
          lyricist: 'Lead Vocalist',
          producer: 'Electronic Producer',
          recordingYear: '2024',
          genre: 'Electronic',
          synthesizers: 'Moog, Roland, Korg',
          programmingBy: 'Electronic Producer'
        }
      },
      {
        resourceId: 'COMP_005',
        resourceType: 'SoundRecording',
        title: 'Soulful Outro',
        artist: 'Lead Vocalist',
        isrc: 'US-MAJ-24-00005',
        duration: 'PT4M45S',
        trackNumber: 5,
        volumeNumber: 1,
        languageOfPerformance: 'en',
        parentalWarning: false,
        metadata: {
          composer: 'Lead Vocalist, Soul Writer',
          lyricist: 'Soul Writer',
          producer: 'Star Producer',
          recordingYear: '2024',
          genre: 'R&B',
          hornSection: 'The Soul Horns',
          strings: 'London Symphony Orchestra'
        }
      }
    ],
    
    // Multiple deal structures
    deals: [
      {
        dealId: 'STREAMING_DEAL_001',
        releaseId: 'COMPLEX_ALB_001',
        territories: ['WorldWide'],
        useTypes: ['Stream'],
        commercialModelType: 'Subscription',
        dealStartDate: '2024-09-15',
        dealName: 'Global Streaming Deal'
      },
      {
        dealId: 'DOWNLOAD_DEAL_001',
        releaseId: 'COMPLEX_ALB_001',
        territories: ['US', 'CA', 'GB', 'AU', 'DE', 'FR', 'JP'],
        useTypes: ['PermanentDownload', 'ConditionalDownload'],
        commercialModelType: 'Purchase',
        dealStartDate: '2024-09-15',
        priceInformation: {
          priceCurrency: 'USD',
          wholesalePrice: 12.99
        },
        dealName: 'Premium Market Download Deal'
      },
      {
        dealId: 'VINYL_DEAL_001',
        releaseId: 'COMPLEX_ALB_001',
        territories: ['US', 'GB', 'DE', 'JP'],
        useTypes: ['PhysicalPurchase'],
        commercialModelType: 'Purchase',
        dealStartDate: '2024-10-15', // Vinyl release later
        priceInformation: {
          priceCurrency: 'USD',
          wholesalePrice: 24.99
        },
        dealName: 'Collector Vinyl Deal'
      }
    ]
  };

  const xml = await builder.build(albumData);
  await fs.writeFile('complex-album.xml', xml, 'utf-8');
  console.log('🎼 Built complex multi-artist album');
  
  return xml;
}
```

## Building from External Data Sources

### From Database Records

```typescript
import { DdexBuilder } from 'ddex-builder';

interface DatabaseRelease {
  id: number;
  title: string;
  artist_name: string;
  label_name: string;
  upc: string;
  release_date: string;
  genre: string;
  tracks: DatabaseTrack[];
}

interface DatabaseTrack {
  id: number;
  title: string;
  artist_name: string;
  isrc: string;
  duration_seconds: number;
  track_number: number;
}

class DatabaseToDDEX {
  private builder: DdexBuilder;
  
  constructor(preset: string = 'universal') {
    this.builder = new DdexBuilder({ preset });
  }
  
  async convertRelease(dbRelease: DatabaseRelease, recipient: string): Promise<string> {
    const ddexData = {
      messageHeader: {
        messageId: `DB_${dbRelease.id}_${Date.now()}`,
        messageSenderName: dbRelease.label_name,
        messageRecipientName: recipient,
        messageCreatedDateTime: new Date().toISOString()
      },
      
      releases: [{
        releaseId: `DB_REL_${dbRelease.id}`,
        title: dbRelease.title,
        artist: dbRelease.artist_name,
        releaseType: this.guessReleaseType(dbRelease.tracks.length),
        label: dbRelease.label_name,
        upc: dbRelease.upc,
        releaseDate: dbRelease.release_date,
        territories: ['WorldWide'], // Default
        genres: [dbRelease.genre],
        trackIds: dbRelease.tracks.map(t => `DB_TR_${t.id}`)
      }],
      
      resources: dbRelease.tracks.map(track => ({
        resourceId: `DB_TR_${track.id}`,
        resourceType: 'SoundRecording',
        title: track.title,
        artist: track.artist_name,
        isrc: track.isrc,
        duration: this.secondsToISO8601(track.duration_seconds),
        trackNumber: track.track_number,
        volumeNumber: 1,
        languageOfPerformance: 'en'
      })),
      
      deals: [{
        dealId: `DB_DEAL_${dbRelease.id}`,
        releaseId: `DB_REL_${dbRelease.id}`,
        territories: ['WorldWide'],
        useTypes: ['Stream', 'PermanentDownload'],
        commercialModelType: 'SubscriptionAndPurchase',
        dealStartDate: dbRelease.release_date
      }]
    };
    
    return await this.builder.build(ddexData);
  }
  
  private guessReleaseType(trackCount: number): string {
    if (trackCount === 1) return 'Single';
    if (trackCount <= 6) return 'EP';
    return 'Album';
  }
  
  private secondsToISO8601(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `PT${mins}M${secs}S`;
  }
}

// Usage
const converter = new DatabaseToDDEX('spotify');

// Assuming you have database records
const releases = await db.query('SELECT * FROM releases WHERE export_pending = true');

for (const release of releases) {
  const tracks = await db.query('SELECT * FROM tracks WHERE release_id = ?', [release.id]);
  const dbRelease: DatabaseRelease = { ...release, tracks };
  
  const xml = await converter.convertRelease(dbRelease, 'Spotify');
  
  await fs.writeFile(`export/release_${release.id}.xml`, xml);
  await db.query('UPDATE releases SET export_pending = false WHERE id = ?', [release.id]);
  
  console.log(`✅ Exported release: ${release.title}`);
}
```

### From CSV/Spreadsheet Data

```python
import pandas as pd
from ddex_builder import DdexBuilder
from datetime import datetime

def csv_to_ddex(csv_path, output_dir='./exports'):
    """Convert CSV data to DDEX ERN messages."""
    
    # Read CSV data
    df = pd.read_csv(csv_path)
    
    # Group by release
    releases = df.groupby(['release_title', 'artist_name', 'upc'])
    
    builder = DdexBuilder(preset='universal')
    
    for (release_title, artist_name, upc), tracks_df in releases:
        
        # Build release data
        release_data = {
            'message_header': {
                'message_id': f'CSV_{int(datetime.now().timestamp())}',
                'message_sender_name': 'CSV Data Import',
                'message_recipient_name': 'Digital Platform',
                'message_created_date_time': datetime.now().isoformat()
            },
            
            'releases': [{
                'release_id': f'CSV_REL_{upc}',
                'title': release_title,
                'artist': artist_name,
                'release_type': guess_release_type(len(tracks_df)),
                'upc': upc,
                'release_date': tracks_df.iloc[0]['release_date'],
                'territories': ['WorldWide'],
                'track_ids': [f'CSV_TR_{idx}' for idx in tracks_df.index]
            }],
            
            'resources': [
                {
                    'resource_id': f'CSV_TR_{row.Index}',
                    'resource_type': 'SoundRecording',
                    'title': row['track_title'],
                    'artist': row['track_artist'] or artist_name,
                    'isrc': row['isrc'],
                    'duration': seconds_to_iso8601(row['duration_seconds']),
                    'track_number': row['track_number'],
                    'volume_number': 1
                }
                for row in tracks_df.itertuples()
            ],
            
            'deals': [{
                'deal_id': f'CSV_DEAL_{upc}',
                'release_id': f'CSV_REL_{upc}',
                'territories': ['WorldWide'],
                'use_types': ['Stream', 'PermanentDownload'],
                'commercial_model_type': 'SubscriptionAndPurchase',
                'deal_start_date': tracks_df.iloc[0]['release_date']
            }]
        }
        
        try:
            xml = builder.build(release_data)
            
            # Save to file
            safe_filename = sanitize_filename(f"{artist_name}_{release_title}")
            output_path = f'{output_dir}/{safe_filename}.xml'
            
            with open(output_path, 'w', encoding='utf-8') as f:
                f.write(xml)
            
            print(f'✅ Exported: {release_title} by {artist_name}')
            
        except Exception as e:
            print(f'❌ Failed to export {release_title}: {e}')

def guess_release_type(track_count):
    """Guess release type based on track count."""
    if track_count == 1:
        return 'Single'
    elif track_count <= 6:
        return 'EP'
    else:
        return 'Album'

def seconds_to_iso8601(seconds):
    """Convert seconds to ISO 8601 duration format."""
    minutes, secs = divmod(int(seconds), 60)
    return f'PT{minutes}M{secs}S'

def sanitize_filename(filename):
    """Sanitize filename for filesystem."""
    import re
    return re.sub(r'[<>:"/\\|?*]', '_', filename)

# Usage
csv_to_ddex('catalog_export.csv', './ddex_exports')
```

## Validation and Testing

### Pre-flight Validation

```typescript
async function validateBeforeBuilding(releaseData: any) {
  const builder = new DdexBuilder({ validate: true });
  
  console.log('🔍 Validating release data...');
  
  // Pre-flight validation
  const validation = await builder.validate(releaseData);
  
  if (!validation.isValid) {
    console.error('❌ Validation failed:');
    validation.errors.forEach((error, index) => {
      console.error(`  ${index + 1}. ${error}`);
    });
    
    if (validation.warnings && validation.warnings.length > 0) {
      console.warn('⚠️ Warnings:');
      validation.warnings.forEach((warning, index) => {
        console.warn(`  ${index + 1}. ${warning}`);
      });
    }
    
    throw new Error('Data validation failed');
  }
  
  console.log('✅ Validation passed');
  
  // Build the XML
  const xml = await builder.build(releaseData);
  
  console.log('🎉 Successfully built DDEX ERN');
  return xml;
}
```

### Testing Different Presets

```typescript
async function testMultiplePresets(releaseData: any) {
  const presets = ['universal', 'spotify', 'apple_music', 'youtube_music'];
  const results = {};
  
  for (const preset of presets) {
    console.log(`\n🧪 Testing ${preset} preset...`);
    
    try {
      const builder = new DdexBuilder({ preset });
      const xml = await builder.build(releaseData);
      
      results[preset] = {
        success: true,
        size: xml.length,
        message: `✅ Built successfully (${xml.length} bytes)`
      };
      
      // Save preset-specific version
      await fs.writeFile(`test-${preset}.xml`, xml, 'utf-8');
      
    } catch (error) {
      results[preset] = {
        success: false,
        error: error.message,
        message: `❌ Failed: ${error.message}`
      };
    }
  }
  
  // Print summary
  console.log('\n📊 Preset Test Summary:');
  console.log('=====================');
  Object.entries(results).forEach(([preset, result]) => {
    console.log(`${preset}: ${result.message}`);
  });
  
  return results;
}

// Usage
const testData = {
  messageHeader: { /* ... */ },
  releases: [{ /* ... */ }],
  resources: [{ /* ... */ }],
  deals: [{ /* ... */ }]
};

await testMultiplePresets(testData);
```

This comprehensive guide shows how to:

- Build simple releases from scratch
- Use platform-specific presets for optimization
- Create complex multi-artist albums
- Convert data from external sources (databases, CSV files)
- Validate data before building
- Test across multiple platform presets

For more advanced scenarios, check out:
- [Round-Trip Processing](./round-trip)
- [Batch Processing](./batch-processing)
- [Python DataFrame Integration](./python-dataframes)