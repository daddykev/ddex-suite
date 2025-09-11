# Resource Models

The Resource models represent different types of digital assets in DDEX messages, including sound recordings, music videos, images, and other media files.

## Core Types

### SoundRecording

Represents audio recordings and their metadata:

```typescript
interface SoundRecording {
  resourceId: string;
  title: string;
  duration?: Duration;
  contributors?: Contributor[];
  technicalDetails?: TechnicalSoundRecordingDetails;
  rightsController?: Party[];
}
```

### Image

Represents visual assets like album artwork:

```typescript
interface Image {
  resourceId: string;
  type: ImageType;
  technicalDetails?: TechnicalImageDetails;
  width?: number;
  height?: number;
}
```

### MusicVideo

Represents video content:

```typescript
interface MusicVideo {
  resourceId: string;
  title: string;
  duration?: Duration;
  technicalDetails?: TechnicalVideoDetails;
}
```

## Technical Details

Each resource type includes technical metadata specific to that format:

- **Audio**: Sample rate, bit depth, codec information
- **Image**: Dimensions, color space, format
- **Video**: Frame rate, resolution, codec, aspect ratio

## Relationships

Resources are linked to:
- **Releases** through resource references
- **Deals** through usage rights
- **Parties** through rights and contributor relationships

## Usage Examples

### Accessing Resource Data

```typescript
const result = await parser.parse(xmlContent);

// Access sound recordings
const recordings = result.flat.resources.soundRecordings;
recordings.forEach(recording => {
  console.log(recording.title, recording.duration);
});

// Access images
const images = result.flat.resources.images;
const artwork = images.find(img => img.type === 'FrontCoverImage');
```

### Building with Resources

```typescript
const buildRequest = {
  resources: {
    soundRecordings: [{
      resourceId: 'A123456789',
      title: 'Example Track',
      duration: 'PT3M45S',
      technicalDetails: {
        audioCodec: 'MP3',
        bitrate: 320000,
        sampleRate: 44100
      }
    }]
  }
};
```

## See Also

- [Technical Details](./technical) - Detailed technical specifications
- [Party Models](./party) - Rights holders and contributors
- [Deal Models](./deal) - Usage rights and licensing