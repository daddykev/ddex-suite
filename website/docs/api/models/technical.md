# Technical Models

Technical models define the digital format specifications, encoding parameters, and delivery requirements for DDEX resources.

## Core Types

### TechnicalSoundRecordingDetails

Technical specifications for audio resources:

```typescript
interface TechnicalSoundRecordingDetails {
  technicalResourceDetailsReference: string;
  audioCodec?: string;
  bitrate?: number;
  numberOfChannels?: number;
  samplingRate?: number;
  bitsPerSample?: number;
  duration?: Duration;
  audioChannelConfiguration?: ChannelConfiguration;
  previewStartTime?: Duration;
  previewDuration?: Duration;
}
```

### TechnicalImageDetails  

Technical specifications for visual resources:

```typescript
interface TechnicalImageDetails {
  technicalResourceDetailsReference: string;
  imageCodec?: string;
  imageHeight?: number;
  imageWidth?: number;
  colorDepth?: number;
  resolution?: number;
  resolutionUnit?: ResolutionUnit;
}
```

### TechnicalVideoDetails

Technical specifications for video resources:

```typescript
interface TechnicalVideoDetails {
  technicalResourceDetailsReference: string;
  videoCodec?: string;
  containerFormat?: string;
  frameRate?: number;
  aspectRatio?: string;
  imageHeight?: number;
  imageWidth?: number;
  duration?: Duration;
  averageBitrate?: number;
}
```

## Audio Specifications

### Common Audio Codecs

```typescript
enum AudioCodec {
  MP3 = 'MP3',
  AAC = 'AAC', 
  FLAC = 'FLAC',
  WAV = 'WAV',
  OGG = 'OGG',
  WMA = 'WMA',
  AIFF = 'AIFF'
}
```

### Audio Quality Parameters

```typescript
// CD Quality
const cdQuality: TechnicalSoundRecordingDetails = {
  technicalResourceDetailsReference: 'T001',
  audioCodec: 'WAV',
  samplingRate: 44100,      // 44.1 kHz
  bitsPerSample: 16,        // 16-bit
  numberOfChannels: 2,      // Stereo
  bitrate: 1411200         // ~1411 kbps
};

// High-Resolution Audio
const hiResAudio: TechnicalSoundRecordingDetails = {
  technicalResourceDetailsReference: 'T002',
  audioCodec: 'FLAC',
  samplingRate: 96000,      // 96 kHz
  bitsPerSample: 24,        // 24-bit  
  numberOfChannels: 2,
  bitrate: 4608000         // ~4608 kbps
};

// Streaming Quality
const streamingQuality: TechnicalSoundRecordingDetails = {
  technicalResourceDetailsReference: 'T003',
  audioCodec: 'AAC',
  samplingRate: 44100,
  bitsPerSample: 16,
  numberOfChannels: 2,
  bitrate: 256000          // 256 kbps
};
```

### Channel Configurations

```typescript
enum ChannelConfiguration {
  Mono = 'Mono',
  Stereo = 'Stereo',
  Surround5_1 = '5.1',
  Surround7_1 = '7.1',
  Binaural = 'Binaural',
  Ambisonic = 'Ambisonic'
}
```

## Image Specifications

### Common Image Formats

```typescript
enum ImageCodec {
  JPEG = 'JPEG',
  PNG = 'PNG',
  GIF = 'GIF',
  TIFF = 'TIFF',
  BMP = 'BMP',
  WEBP = 'WebP'
}
```

### Standard Image Dimensions

```typescript
// Album artwork standards
const albumArtwork: TechnicalImageDetails = {
  technicalResourceDetailsReference: 'T004',
  imageCodec: 'JPEG',
  imageWidth: 3000,
  imageHeight: 3000,
  colorDepth: 24,
  resolution: 300,
  resolutionUnit: 'DPI'
};

// Thumbnail image
const thumbnail: TechnicalImageDetails = {
  technicalResourceDetailsReference: 'T005', 
  imageCodec: 'JPEG',
  imageWidth: 300,
  imageHeight: 300,
  colorDepth: 24,
  resolution: 72,
  resolutionUnit: 'DPI'
};
```

## Video Specifications

### Common Video Codecs

```typescript
enum VideoCodec {
  H264 = 'H.264',
  H265 = 'H.265',
  VP9 = 'VP9',
  AV1 = 'AV1',
  MPEG2 = 'MPEG-2',
  MPEG4 = 'MPEG-4'
}
```

### Video Quality Presets

```typescript
// 4K Ultra HD
const uhd4k: TechnicalVideoDetails = {
  technicalResourceDetailsReference: 'T006',
  videoCodec: 'H.264',
  containerFormat: 'MP4',
  imageWidth: 3840,
  imageHeight: 2160,
  frameRate: 30,
  aspectRatio: '16:9',
  averageBitrate: 25000000    // 25 Mbps
};

// Full HD
const fullHd: TechnicalVideoDetails = {
  technicalResourceDetailsReference: 'T007',
  videoCodec: 'H.264', 
  containerFormat: 'MP4',
  imageWidth: 1920,
  imageHeight: 1080,
  frameRate: 30,
  aspectRatio: '16:9',
  averageBitrate: 8000000     // 8 Mbps
};
```

## Usage Examples

### Accessing Technical Details

```typescript
const result = await parser.parse(xmlContent);

// Get audio technical details
const recording = result.flat.resources.soundRecordings[0];
const audioTech = recording.technicalDetails;

if (audioTech) {
  console.log(`Codec: ${audioTech.audioCodec}`);
  console.log(`Quality: ${audioTech.samplingRate}Hz/${audioTech.bitsPerSample}bit`);
  console.log(`Bitrate: ${audioTech.bitrate} bps`);
  console.log(`Channels: ${audioTech.numberOfChannels}`);
}

// Get image technical details
const image = result.flat.resources.images[0];
const imageTech = image.technicalDetails;

if (imageTech) {
  console.log(`Format: ${imageTech.imageCodec}`);
  console.log(`Dimensions: ${imageTech.imageWidth}x${imageTech.imageHeight}`);
  console.log(`Resolution: ${imageTech.resolution} ${imageTech.resolutionUnit}`);
}
```

### Building Technical Data

```typescript
const buildRequest = {
  resources: {
    soundRecordings: [{
      resourceId: 'A123456789',
      title: 'Example Track',
      technicalDetails: {
        technicalResourceDetailsReference: 'T001',
        audioCodec: 'FLAC',
        samplingRate: 44100,
        bitsPerSample: 16,
        numberOfChannels: 2,
        bitrate: 1411200,
        duration: 'PT3M45S',
        previewStartTime: 'PT30S',
        previewDuration: 'PT30S'
      }
    }],
    images: [{
      resourceId: 'I123456789',
      type: 'FrontCoverImage',
      technicalDetails: {
        technicalResourceDetailsReference: 'T002',
        imageCodec: 'JPEG',
        imageWidth: 1400,
        imageHeight: 1400,
        colorDepth: 24,
        resolution: 300,
        resolutionUnit: 'DPI'
      }
    }]
  }
};
```

### Quality Validation

```typescript
function validateAudioQuality(tech: TechnicalSoundRecordingDetails): boolean {
  // Minimum streaming quality requirements
  const minRequirements = {
    samplingRate: 44100,
    bitsPerSample: 16,
    bitrate: 128000
  };
  
  return tech.samplingRate >= minRequirements.samplingRate &&
         tech.bitsPerSample >= minRequirements.bitsPerSample &&
         tech.bitrate >= minRequirements.bitrate;
}

function validateImageQuality(tech: TechnicalImageDetails): boolean {
  // Minimum artwork requirements  
  const minDimensions = 1400;
  const minResolution = 300;
  
  return tech.imageWidth >= minDimensions &&
         tech.imageHeight >= minDimensions &&
         tech.resolution >= minResolution;
}
```

### Format Conversion Information

```typescript
// Supported format matrices by platform
const platformSupport = {
  spotify: {
    audio: ['MP3', 'AAC', 'OGG'],
    bitrates: [96, 160, 320], // kbps
    samplingRates: [44100]
  },
  apple: {
    audio: ['AAC', 'ALAC'],
    bitrates: [256, 1411], // kbps for AAC, lossless for ALAC
    samplingRates: [44100, 48000, 96000]
  },
  tidal: {
    audio: ['AAC', 'FLAC', 'MQA'],
    bitrates: [320, 1411, 2304], // kbps
    samplingRates: [44100, 48000, 96000]
  }
};
```

## File Size Estimation

```typescript
function estimateFileSize(tech: TechnicalSoundRecordingDetails, durationSeconds: number): number {
  if (!tech.bitrate) return 0;
  
  // File size in bytes = (bitrate in bps × duration in seconds) ÷ 8
  return Math.ceil((tech.bitrate * durationSeconds) / 8);
}

// Usage
const duration = parseDuration(recording.duration?.value || 'PT0S');
const fileSize = estimateFileSize(recording.technicalDetails, duration);
console.log(`Estimated file size: ${(fileSize / 1024 / 1024).toFixed(2)} MB`);
```

## See Also

- [Resource Models](./resource) - Resource and technical detail relationships
- [DateTime Models](./datetime) - Duration specifications
- [Deal Models](./deal) - Technical requirements in licensing