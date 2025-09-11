# Building from Scratch

Learn how to create DDEX messages programmatically without existing data.

## Basic Release Creation

Start with a minimal release structure:

```typescript
import { DDEXBuilder } from 'ddex-builder';

const builder = new DDEXBuilder();

// Create a basic release
const releaseData = {
  messageHeader: {
    messageId: 'MSG_001',
    messageSenderName: 'My Record Label',
    messageRecipientName: 'Streaming Platform',
    messageCreatedDateTime: new Date().toISOString()
  },
  releases: [{
    releaseId: 'REL_001',
    title: 'My First Album',
    artist: 'New Artist',
    releaseType: 'Album',
    label: 'My Record Label',
    upc: '123456789012',
    releaseDate: '2024-03-01',
    territories: ['US', 'CA', 'GB'],
    genres: ['Pop', 'Electronic']
  }]
};

const xml = await builder.build(releaseData);
console.log('Generated DDEX XML:', xml);
```

## Adding Tracks

Build a complete release with multiple tracks:

```typescript
const releaseWithTracks = {
  messageHeader: {
    messageId: 'MSG_002',
    messageSenderName: 'Indie Label Records',
    messageRecipientName: 'Music Distribution Service',
    messageCreatedDateTime: new Date().toISOString()
  },
  releases: [{
    releaseId: 'REL_002',
    title: 'Debut EP',
    artist: 'Rising Star',
    releaseType: 'EP',
    label: 'Indie Label Records',
    upc: '987654321098',
    releaseDate: '2024-04-15',
    territories: ['Worldwide'],
    genres: ['Indie Rock', 'Alternative'],
    trackIds: ['TRK_001', 'TRK_002', 'TRK_003']
  }],
  resources: [
    {
      resourceId: 'TRK_001',
      resourceType: 'SoundRecording',
      title: 'Opening Song',
      artist: 'Rising Star',
      isrc: 'US-IND-24-00001',
      duration: 'PT3M45S',
      trackNumber: 1,
      genres: ['Indie Rock']
    },
    {
      resourceId: 'TRK_002', 
      resourceType: 'SoundRecording',
      title: 'Main Hit',
      artist: 'Rising Star',
      isrc: 'US-IND-24-00002',
      duration: 'PT4M12S',
      trackNumber: 2,
      genres: ['Alternative']
    },
    {
      resourceId: 'TRK_003',
      resourceType: 'SoundRecording', 
      title: 'Closing Track',
      artist: 'Rising Star',
      isrc: 'US-IND-24-00003',
      duration: 'PT3M33S',
      trackNumber: 3,
      genres: ['Indie Rock']
    }
  ],
  deals: [{
    dealId: 'DEAL_001',
    releaseId: 'REL_002',
    territories: ['Worldwide'],
    useTypes: ['Stream', 'PermanentDownload'],
    commercialModelType: 'Subscription',
    dealStartDate: '2024-04-15'
  }]
};

const xml = await builder.build(releaseWithTracks);
```

## Step-by-Step Builder Pattern

Use the fluent builder API for complex releases:

```typescript
class FluentDDEXBuilder {
  private data: any = {
    messageHeader: {},
    releases: [],
    resources: [],
    deals: []
  };

  messageHeader(messageId: string, sender: string, recipient: string) {
    this.data.messageHeader = {
      messageId,
      messageSenderName: sender,
      messageRecipientName: recipient,
      messageCreatedDateTime: new Date().toISOString()
    };
    return this;
  }

  release(id: string, title: string, artist: string, type: string = 'Album') {
    this.data.releases.push({
      releaseId: id,
      title,
      artist,
      releaseType: type,
      trackIds: []
    });
    return this;
  }

  releaseMetadata(releaseId: string, metadata: any) {
    const release = this.data.releases.find(r => r.releaseId === releaseId);
    if (release) {
      Object.assign(release, metadata);
    }
    return this;
  }

  track(id: string, title: string, artist: string, duration: string, isrc: string) {
    this.data.resources.push({
      resourceId: id,
      resourceType: 'SoundRecording',
      title,
      artist,
      duration,
      isrc,
      trackNumber: this.data.resources.length + 1
    });
    
    // Add to the most recent release
    if (this.data.releases.length > 0) {
      const lastRelease = this.data.releases[this.data.releases.length - 1];
      lastRelease.trackIds = lastRelease.trackIds || [];
      lastRelease.trackIds.push(id);
    }
    
    return this;
  }

  deal(releaseId: string, territories: string[], useTypes: string[]) {
    this.data.deals.push({
      dealId: `DEAL_${this.data.deals.length + 1}`,
      releaseId,
      territories,
      useTypes,
      commercialModelType: 'Subscription',
      dealStartDate: new Date().toISOString().split('T')[0]
    });
    return this;
  }

  async build(): Promise<string> {
    const builder = new DDEXBuilder();
    return builder.build(this.data);
  }

  getData() {
    return this.data;
  }
}

// Usage example
const fluentBuilder = new FluentDDEXBuilder()
  .messageHeader('MSG_FLUENT_001', 'Creative Music Label', 'Global Distribution')
  .release('REL_ALBUM_001', 'Masterpiece Collection', 'Acclaimed Artist', 'Album')
  .releaseMetadata('REL_ALBUM_001', {
    label: 'Creative Music Label',
    upc: '555123456789',
    releaseDate: '2024-06-01',
    genres: ['Jazz', 'Contemporary'],
    territories: ['US', 'EU', 'JP']
  })
  .track('TRK_001', 'Prelude', 'Acclaimed Artist', 'PT2M15S', 'US-CML-24-00001')
  .track('TRK_002', 'Main Theme', 'Acclaimed Artist', 'PT5M45S', 'US-CML-24-00002')
  .track('TRK_003', 'Variations', 'Acclaimed Artist', 'PT6M30S', 'US-CML-24-00003')
  .deal('REL_ALBUM_001', ['Worldwide'], ['Stream', 'PermanentDownload']);

const xml = await fluentBuilder.build();
console.log('Fluent builder result:', xml.length, 'characters');
```

## Template-Based Creation

Create reusable templates for common release types:

```typescript
class DDEXTemplates {
  static singleTemplate(
    artist: string,
    trackTitle: string,
    label: string,
    isrc: string,
    duration: string
  ) {
    return {
      messageHeader: {
        messageId: `SINGLE_${Date.now()}`,
        messageSenderName: label,
        messageRecipientName: 'Digital Service Provider',
        messageCreatedDateTime: new Date().toISOString()
      },
      releases: [{
        releaseId: `SINGLE_${Date.now()}`,
        title: trackTitle,
        artist,
        releaseType: 'Single',
        label,
        upc: this.generateUPC(),
        releaseDate: this.getNextFriday(),
        territories: ['Worldwide'],
        genres: ['Pop'],
        trackIds: ['TRK_001']
      }],
      resources: [{
        resourceId: 'TRK_001',
        resourceType: 'SoundRecording',
        title: trackTitle,
        artist,
        isrc,
        duration,
        trackNumber: 1
      }],
      deals: [{
        dealId: 'DEAL_001',
        releaseId: `SINGLE_${Date.now()}`,
        territories: ['Worldwide'],
        useTypes: ['Stream', 'PermanentDownload'],
        commercialModelType: 'Subscription',
        dealStartDate: this.getNextFriday()
      }]
    };
  }

  static albumTemplate(
    artist: string,
    albumTitle: string,
    label: string,
    tracks: Array<{title: string, isrc: string, duration: string}>
  ) {
    const albumId = `ALBUM_${Date.now()}`;
    
    return {
      messageHeader: {
        messageId: `MSG_${Date.now()}`,
        messageSenderName: label,
        messageRecipientName: 'Digital Service Provider',
        messageCreatedDateTime: new Date().toISOString()
      },
      releases: [{
        releaseId: albumId,
        title: albumTitle,
        artist,
        releaseType: 'Album',
        label,
        upc: this.generateUPC(),
        releaseDate: this.getNextFriday(),
        territories: ['Worldwide'],
        genres: ['Pop'],
        trackIds: tracks.map((_, i) => `TRK_${i + 1}`)
      }],
      resources: tracks.map((track, i) => ({
        resourceId: `TRK_${i + 1}`,
        resourceType: 'SoundRecording',
        title: track.title,
        artist,
        isrc: track.isrc,
        duration: track.duration,
        trackNumber: i + 1
      })),
      deals: [{
        dealId: 'DEAL_001',
        releaseId: albumId,
        territories: ['Worldwide'],
        useTypes: ['Stream', 'PermanentDownload'],
        commercialModelType: 'Subscription',
        dealStartDate: this.getNextFriday()
      }]
    };
  }

  private static generateUPC(): string {
    return Math.floor(100000000000 + Math.random() * 900000000000).toString();
  }

  private static getNextFriday(): string {
    const today = new Date();
    const daysUntilFriday = (5 - today.getDay() + 7) % 7;
    const nextFriday = new Date(today.getTime() + daysUntilFriday * 24 * 60 * 60 * 1000);
    return nextFriday.toISOString().split('T')[0];
  }
}

// Usage
const singleData = DDEXTemplates.singleTemplate(
  'New Artist',
  'Hit Song',
  'Independent Records',
  'US-IND-24-12345',
  'PT3M30S'
);

const albumData = DDEXTemplates.albumTemplate(
  'Established Band',
  'Greatest Hits',
  'Major Label',
  [
    { title: 'Classic Hit 1', isrc: 'US-MAJ-90-00001', duration: 'PT4M15S' },
    { title: 'Classic Hit 2', isrc: 'US-MAJ-90-00002', duration: 'PT3M45S' },
    { title: 'Classic Hit 3', isrc: 'US-MAJ-95-00003', duration: 'PT5M00S' }
  ]
);

const builder = new DDEXBuilder();
const singleXml = await builder.build(singleData);
const albumXml = await builder.build(albumData);
```

## Python Release Creation

```python
from ddex_builder import DDEXBuilder
from datetime import datetime, timedelta

class DDEXFactory:
    def __init__(self):
        self.builder = DDEXBuilder()
    
    def create_single_release(self, artist, title, label, isrc, duration_seconds):
        """Create a single track release"""
        
        # Convert duration to ISO format
        minutes = duration_seconds // 60
        seconds = duration_seconds % 60
        duration_iso = f"PT{minutes}M{seconds}S"
        
        release_data = {
            "message_header": {
                "message_id": f"SINGLE_{int(datetime.now().timestamp())}",
                "message_sender_name": label,
                "message_recipient_name": "Streaming Service",
                "message_created_date_time": datetime.now().isoformat()
            },
            "releases": [{
                "release_id": f"REL_{int(datetime.now().timestamp())}",
                "title": title,
                "artist": artist,
                "release_type": "Single",
                "label": label,
                "upc": self._generate_upc(),
                "release_date": self._get_next_friday(),
                "territories": ["Worldwide"],
                "genres": ["Pop"],
                "track_ids": ["TRK_001"]
            }],
            "resources": [{
                "resource_id": "TRK_001",
                "resource_type": "SoundRecording",
                "title": title,
                "artist": artist,
                "isrc": isrc,
                "duration": duration_iso,
                "track_number": 1
            }],
            "deals": [{
                "deal_id": "DEAL_001",
                "release_id": f"REL_{int(datetime.now().timestamp())}",
                "territories": ["Worldwide"],
                "use_types": ["Stream", "PermanentDownload"],
                "commercial_model_type": "Subscription",
                "deal_start_date": self._get_next_friday()
            }]
        }
        
        return self.builder.build(release_data)
    
    def create_compilation_album(self, album_title, label, tracks):
        """Create a compilation album with multiple artists"""
        
        album_id = f"COMP_{int(datetime.now().timestamp())}"
        
        release_data = {
            "message_header": {
                "message_id": f"MSG_{int(datetime.now().timestamp())}",
                "message_sender_name": label,
                "message_recipient_name": "Streaming Service",
                "message_created_date_time": datetime.now().isoformat()
            },
            "releases": [{
                "release_id": album_id,
                "title": album_title,
                "artist": "Various Artists",
                "release_type": "Album",
                "label": label,
                "upc": self._generate_upc(),
                "release_date": self._get_next_friday(),
                "territories": ["Worldwide"],
                "genres": ["Compilation"],
                "track_ids": [f"TRK_{i+1:03d}" for i in range(len(tracks))]
            }],
            "resources": [
                {
                    "resource_id": f"TRK_{i+1:03d}",
                    "resource_type": "SoundRecording",
                    "title": track["title"],
                    "artist": track["artist"],
                    "isrc": track["isrc"],
                    "duration": track["duration"],
                    "track_number": i + 1
                }
                for i, track in enumerate(tracks)
            ],
            "deals": [{
                "deal_id": "DEAL_001",
                "release_id": album_id,
                "territories": ["Worldwide"],
                "use_types": ["Stream", "PermanentDownload"],
                "commercial_model_type": "Subscription",
                "deal_start_date": self._get_next_friday()
            }]
        }
        
        return self.builder.build(release_data)
    
    def _generate_upc(self):
        import random
        return ''.join([str(random.randint(0, 9)) for _ in range(12)])
    
    def _get_next_friday(self):
        today = datetime.now()
        days_ahead = 4 - today.weekday()  # Friday is 4
        if days_ahead <= 0:  # Target day already happened this week
            days_ahead += 7
        return (today + timedelta(days=days_ahead)).strftime('%Y-%m-%d')

# Usage
factory = DDEXFactory()

# Create single
single_xml = factory.create_single_release(
    artist="New Pop Star",
    title="Summer Hit",
    label="Pop Records LLC", 
    isrc="US-POP-24-00001",
    duration_seconds=210  # 3:30
)

# Create compilation
compilation_tracks = [
    {
        "title": "Dance Floor Anthem",
        "artist": "DJ Master",
        "isrc": "US-EDM-23-00001",
        "duration": "PT6M15S"
    },
    {
        "title": "Chill Vibes",
        "artist": "Lo-Fi Producer",
        "isrc": "US-CHI-23-00002", 
        "duration": "PT4M30S"
    }
]

compilation_xml = factory.create_compilation_album(
    album_title="Electronic Essentials 2024",
    label="Electronic Music Collective",
    tracks=compilation_tracks
)

print(f"Single XML length: {len(single_xml)} characters")
print(f"Compilation XML length: {len(compilation_xml)} characters")
```

## Validation and Testing

Always validate your generated DDEX:

```typescript
import { DDEXValidator } from 'ddex-validator';

async function buildAndValidate(releaseData: any): Promise<string> {
  const builder = new DDEXBuilder();
  const validator = new DDEXValidator();
  
  try {
    // Build DDEX XML
    const xml = await builder.build(releaseData);
    
    // Validate the generated XML
    const validationResult = await validator.validate(xml);
    
    if (validationResult.isValid) {
      console.log('✓ Generated valid DDEX XML');
      return xml;
    } else {
      console.error('✗ Validation failed:');
      validationResult.errors.forEach(error => {
        console.error(`  - ${error.field}: ${error.message}`);
      });
      throw new Error('Generated DDEX is invalid');
    }
  } catch (error) {
    console.error('Build or validation error:', error.message);
    throw error;
  }
}

// Test the build and validation
const testRelease = {
  messageHeader: {
    messageId: 'TEST_001',
    messageSenderName: 'Test Label',
    messageRecipientName: 'Test DSP',
    messageCreatedDateTime: new Date().toISOString()
  },
  releases: [{
    releaseId: 'TEST_REL_001',
    title: 'Test Album',
    artist: 'Test Artist',
    releaseType: 'Album',
    label: 'Test Label',
    upc: '123456789012',
    releaseDate: '2024-05-01'
  }]
};

const validatedXml = await buildAndValidate(testRelease);
```

## Common Patterns

### Multi-Format Releases

Create releases available in multiple formats:

```typescript
const multiFormatRelease = {
  // ... standard release data
  resources: [
    // Standard quality
    {
      resourceId: 'TRK_001_STD',
      resourceType: 'SoundRecording',
      title: 'Hit Song',
      artist: 'Popular Artist',
      isrc: 'US-LAB-24-00001',
      duration: 'PT3M45S',
      audioQuality: 'Standard'
    },
    // High quality
    {
      resourceId: 'TRK_001_HQ',
      resourceType: 'SoundRecording', 
      title: 'Hit Song',
      artist: 'Popular Artist',
      isrc: 'US-LAB-24-00001',
      duration: 'PT3M45S',
      audioQuality: 'HighQuality'
    },
    // Lossless
    {
      resourceId: 'TRK_001_LOSSLESS',
      resourceType: 'SoundRecording',
      title: 'Hit Song', 
      artist: 'Popular Artist',
      isrc: 'US-LAB-24-00001',
      duration: 'PT3M45S',
      audioQuality: 'Lossless'
    }
  ]
};
```

### Pre-Order Releases

Set up releases with pre-order periods:

```typescript
const preOrderRelease = {
  // ... standard release data
  deals: [{
    dealId: 'PREORDER_DEAL_001',
    releaseId: 'REL_001',
    territories: ['US', 'CA', 'GB'],
    useTypes: ['Stream', 'PermanentDownload'],
    commercialModelType: 'Purchase',
    dealStartDate: '2024-03-01',  // Pre-order start
    releaseDate: '2024-03-15',    // Actual release
    preOrderDate: '2024-03-01'    // Pre-order availability
  }]
};
```

## Next Steps

- [Partner-Specific Outputs](./partner-outputs) - Customize for different DSPs
- [Deterministic Output](./deterministic) - Ensure reproducible builds
- [Batch Processing](./batch-processing) - Handle multiple releases
- [Validation Workflows](./validation) - Comprehensive validation strategies