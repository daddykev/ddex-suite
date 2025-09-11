# Platform Presets

Complete reference for industry platform presets in the DDEX Builder.

## Overview

The DDEX Builder now uses a simplified, transparent preset system that focuses on public specifications and extensible frameworks. This approach ensures honesty, maintainability, and legal safety while providing valuable preset functionality.

## Available Presets

### YouTube Presets (Public Specification)

| Preset | Type | Profile | DDEX Version | Specification |
|--------|------|---------|--------------|---------------|
| `youtube_album` | Album | Content ID | ERN 4.3 | YouTube Public DDEX Spec |
| `youtube_single` | Single | Content ID | ERN 4.3 | YouTube Public DDEX Spec |
| `youtube_video` | Video | Content ID | ERN 4.3 | YouTube Public DDEX Spec |

### Generic DDEX-Compliant Presets

| Preset | Type | Profile | DDEX Version | Specification |
|--------|------|---------|--------------|---------------|
| `generic_audio_album` | Audio Album | Universal | ERN 4.3 | DDEX Standard |
| `generic_audio_single` | Audio Single | Universal | ERN 4.3 | DDEX Standard |
| `generic_video_single` | Video Single | Universal | ERN 4.3 | DDEX Standard |
| `generic_compilation` | Compilation | Universal | ERN 4.3 | DDEX Standard |

## Spotify Preset

### Configuration

```typescript
// TypeScript
builder.applyPreset('spotify');

// Python
builder.apply_preset('spotify')
```

### Requirements

#### Required Fields

| Field | Level | Notes |
|-------|-------|-------|
| Release ID | Release | Must be unique within message |
| Title | Release | Max 100 characters |
| Artist | Release | Primary artist only |
| Label | Release | Record label name required |
| UPC | Release | Valid UPC-A format |
| Release Date | Release | ISO 8601 format (YYYY-MM-DD) |
| Genre | Release | From Spotify genre list |
| ISRC | Resource | Valid ISRC format |
| Duration | Resource | ISO 8601 duration (PT3M45S) |
| Track Number | Resource | Sequential numbering |

#### Validation Rules

```json
{
  "release_title": {
    "type": "length",
    "max": 100,
    "message": "Release title cannot exceed 100 characters"
  },
  "upc": {
    "type": "format",
    "pattern": "^\\d{12}$",
    "message": "UPC must be 12 digits"
  },
  "isrc": {
    "type": "format", 
    "pattern": "^[A-Z]{2}[A-Z0-9]{3}\\d{7}$",
    "message": "Invalid ISRC format"
  },
  "genre": {
    "type": "enum",
    "values": ["Pop", "Rock", "Hip-Hop", "Electronic", "Country", "..."],
    "message": "Genre must be from Spotify approved list"
  }
}
```

#### Commercial Model

- **Primary Model**: Subscription Streaming
- **Secondary Models**: Ad-supported streaming
- **Territories**: Worldwide (excluding embargoed territories)
- **Distribution Channel**: Internet streaming

### Example Usage

```typescript
import { DdexBuilder } from 'ddex-builder';

const builder = new DdexBuilder();
builder.applyPreset('spotify');

const release = {
  releaseId: 'SPT_REL_001',
  releaseType: 'Album',
  title: 'My Streaming Album',
  artist: 'Artist Name',
  label: 'Independent Records',
  upc: '123456789012',
  releaseDate: '2024-03-15',
  genre: 'Pop',
  trackIds: ['SPT_TRK_001', 'SPT_TRK_002']
};

builder.addRelease(release);
const xml = await builder.build();
```

```python
from ddex_builder import DdexBuilder

builder = DdexBuilder()
builder.apply_preset('spotify')

release = {
    'release_id': 'SPT_REL_001',
    'release_type': 'Album', 
    'title': 'My Streaming Album',
    'artist': 'Artist Name',
    'label': 'Independent Records',
    'upc': '123456789012',
    'release_date': '2024-03-15',
    'genre': 'Pop',
    'track_ids': ['SPT_TRK_001', 'SPT_TRK_002']
}

builder.add_release(release)
xml = await builder.build()
```

---

## Apple Music Preset

### Configuration

```typescript
// TypeScript
builder.applyPreset('apple_music');

// Python
builder.apply_preset('apple_music')
```

### Requirements

#### Required Fields

| Field | Level | Notes |
|-------|-------|-------|
| Release ID | Release | iTunes-compatible format |
| Title | Release | Support for international characters |
| Artist | Release | Support for featured artists |
| Label | Release | Record label required |
| UPC | Release | EAN-13 or UPC-A |
| Release Date | Release | Must not be future date |
| Genre | Release | iTunes genre taxonomy |
| Copyright | Release | Copyright line required |
| Producer Copyright | Release | P-line required |
| ISRC | Resource | Mandatory for all tracks |
| Duration | Resource | Accurate to milliseconds |

#### Special Requirements

- **Artwork**: High-resolution artwork specifications
- **Preview**: Preview start time and duration
- **Explicit Content**: Explicit content flagging required
- **Featured Artists**: Proper featured artist handling
- **Multi-language**: Support for multiple language metadata

#### Commercial Models

- **Primary**: Permanent Download
- **Secondary**: Subscription Streaming
- **Territories**: Worldwide iTunes Store presence
- **Price Tiers**: Standard iTunes pricing structure

### Example Usage

```typescript
const builder = new DdexBuilder();
builder.applyPreset('apple_music');

const release = {
  releaseId: 'ITMS_REL_001',
  releaseType: 'Album',
  title: 'iTunes Album',
  artist: 'Artist Name',
  label: 'Major Label Records',
  upc: '1234567890123',
  releaseDate: '2024-03-15',
  genre: 'Alternative',
  parentalWarning: false,
  copyrightLine: '© 2024 Major Label Records',
  producerCopyrightLine: '℗ 2024 Major Label Records',
  trackIds: ['ITMS_TRK_001', 'ITMS_TRK_002']
};

builder.addRelease(release);
```

---

## YouTube Music Preset

### Configuration

```typescript
// TypeScript
builder.applyPreset('youtube_music');

// Python
builder.apply_preset('youtube_music')
```

### Requirements

#### Required Fields

| Field | Level | Notes |
|-------|-------|-------|
| Release ID | Release | YouTube Content ID compatible |
| Title | Release | Must match video content |
| Artist | Release | Channel/artist verification |
| ISRC | Resource | Required for Content ID |
| Duration | Resource | Must match audio duration |
| Territory Rights | Deal | Detailed territory mapping |
| Content Type | Resource | Audio/Video classification |

#### Content ID Requirements

- **Audio Fingerprinting**: Support for Content ID fingerprinting
- **Reference Files**: High-quality reference audio required
- **Territory Mapping**: Detailed geographic rights specification
- **Ownership Claims**: Clear ownership and rights information
- **Monetization**: Monetization policy specification

#### Commercial Models

- **Primary**: Ad-supported streaming
- **Secondary**: YouTube Premium subscription
- **Territories**: Based on Content ID availability
- **Rights**: Performance and synchronization rights

### Example Usage

```typescript
const builder = new DdexBuilder();
builder.applyPreset('youtube_music');

const release = {
  releaseId: 'YTM_REL_001',
  releaseType: 'Single',
  title: 'YouTube Hit Single',
  artist: 'Content Creator',
  label: 'Creator Network',
  releaseDate: '2024-03-15',
  genre: 'Electronic',
  contentType: 'Audio',
  trackIds: ['YTM_TRK_001']
};

const deal = {
  dealId: 'YTM_DEAL_001',
  dealType: 'License',
  commercialModelType: 'AdSupportedModel',
  territory: 'Worldwide',
  distributionChannel: 'Internet',
  monetizationPolicy: 'Monetize',
  releaseReferences: ['YTM_REL_001']
};

builder.addRelease(release);
builder.addDeal(deal);
```

---

## Amazon Music Preset

### Configuration

```typescript
// TypeScript
builder.applyPreset('amazon_music');

// Python
builder.apply_preset('amazon_music')
```

### Requirements

#### Required Fields

| Field | Level | Notes |
|-------|-------|-------|
| Release ID | Release | Amazon ASIN compatible |
| Title | Release | Amazon catalog standards |
| Artist | Release | Artist name verification |
| Label | Release | Label registration required |
| UPC | Release | Amazon barcode standards |
| Release Date | Release | Amazon release calendar |
| Genre | Release | Amazon music categories |
| ISRC | Resource | Global ISRC database |
| Product Type | Release | Physical/Digital classification |

#### Amazon-Specific Requirements

- **Product Classification**: Proper Amazon product categorization
- **Search Keywords**: SEO-optimized metadata
- **Brand Registry**: Trademark and brand compliance
- **Regional Availability**: Territory-specific availability
- **Pricing Strategy**: Amazon pricing tier compatibility

#### Commercial Models

- **Primary**: Unlimited Streaming (Amazon Music Unlimited)
- **Secondary**: Prime Music inclusion
- **Tertiary**: Purchase downloads
- **Territories**: Amazon Music service areas

### Example Usage

```typescript
const builder = new DdexBuilder();
builder.applyPreset('amazon_music');

const release = {
  releaseId: 'AMZ_REL_001',
  releaseType: 'Album',
  title: 'Amazon Music Album',
  artist: 'Professional Artist',
  label: 'Established Records',
  upc: '123456789012',
  releaseDate: '2024-03-15',
  genre: 'Country',
  productType: 'DigitalAlbum',
  searchKeywords: ['country', 'acoustic', 'folk'],
  trackIds: ['AMZ_TRK_001', 'AMZ_TRK_002']
};

builder.addRelease(release);
```

---

## Universal Preset

### Configuration

```typescript
// TypeScript
builder.applyPreset('universal');

// Python
builder.apply_preset('universal')
```

### Purpose

The universal preset provides a generic configuration suitable for most streaming platforms and digital distributors. It includes common requirements and best practices without platform-specific restrictions.

### Requirements

#### Core Fields

| Field | Level | Required | Notes |
|-------|-------|----------|-------|
| Release ID | Release | ✓ | Unique identifier |
| Title | Release | ✓ | No length restrictions |
| Artist | Release | ✓ | Primary artist |
| Release Date | Release | ○ | Recommended |
| Genre | Release | ○ | Standard genres |
| ISRC | Resource | ○ | Recommended |
| Duration | Resource | ○ | For audio content |

#### Flexible Requirements

- **Minimal Validation**: Basic structure validation only
- **Format Flexibility**: Accepts various identifier formats
- **Genre Freedom**: No restricted genre lists
- **Territory Neutral**: No territory restrictions
- **Commercial Agnostic**: Works with any commercial model

### Example Usage

```typescript
const builder = new DdexBuilder();
builder.applyPreset('universal');

const release = {
  releaseId: 'UNI_REL_001',
  releaseType: 'Album',
  title: 'Universal Release',
  artist: 'Any Artist',
  label: 'Any Label',
  releaseDate: '2024-03-15',
  genre: 'World Music',
  trackIds: ['UNI_TRK_001']
};

builder.addRelease(release);
```

---

## Preset Comparison

### Field Requirements Matrix

| Field | Spotify | Apple Music | YouTube Music | Amazon Music | Universal |
|-------|---------|-------------|---------------|--------------|-----------|
| Release ID | ✓ | ✓ | ✓ | ✓ | ✓ |
| Title | ✓ | ✓ | ✓ | ✓ | ✓ |
| Artist | ✓ | ✓ | ✓ | ✓ | ✓ |
| Label | ✓ | ✓ | ○ | ✓ | ○ |
| UPC | ✓ | ✓ | ○ | ✓ | ○ |
| Release Date | ✓ | ✓ | ✓ | ✓ | ○ |
| Genre | ✓ | ✓ | ✓ | ✓ | ○ |
| ISRC | ✓ | ✓ | ✓ | ✓ | ○ |
| Duration | ✓ | ✓ | ✓ | ○ | ○ |
| Copyright | ○ | ✓ | ○ | ○ | ○ |
| Explicit Flag | ○ | ✓ | ○ | ○ | ○ |

### Validation Strictness

| Preset | Structure | Format | Business Rules | Custom Rules |
|--------|-----------|--------|----------------|--------------|
| Spotify | Strict | Strict | Strict | Platform-specific |
| Apple Music | Strict | Strict | Strict | iTunes-specific |
| YouTube Music | Medium | Medium | Medium | Content ID-specific |
| Amazon Music | Strict | Strict | Medium | Amazon-specific |
| Universal | Basic | Flexible | Minimal | None |

---

## Custom Preset Creation

### Defining Custom Rules

```typescript
// TypeScript
import { DdexBuilder, ValidationRule } from 'ddex-builder';

const customRules: ValidationRule[] = [
  {
    fieldName: 'title',
    ruleType: 'length',
    message: 'Title must be between 1 and 50 characters',
    parameters: { min: '1', max: '50' }
  },
  {
    fieldName: 'genre',
    ruleType: 'enum',
    message: 'Genre must be from approved list',
    parameters: { values: 'Pop,Rock,Electronic' }
  }
];

const builder = new DdexBuilder();
builder.applyCustomRules(customRules);
```

```python
# Python
from ddex_builder import DdexBuilder, ValidationRule

custom_rules = [
    ValidationRule(
        field_name='title',
        rule_type='length',
        message='Title must be between 1 and 50 characters',
        parameters={'min': '1', 'max': '50'}
    ),
    ValidationRule(
        field_name='genre',
        rule_type='enum', 
        message='Genre must be from approved list',
        parameters={'values': 'Pop,Rock,Electronic'}
    )
]

builder = DdexBuilder()
builder.apply_custom_rules(custom_rules)
```

### Preset Inheritance

```typescript
// Start with universal preset and add custom rules
const builder = new DdexBuilder();
builder.applyPreset('universal');

// Add platform-specific requirements
builder.addValidationRule({
  fieldName: 'label',
  ruleType: 'required',
  message: 'Label is required for our platform'
});

// Override existing rules
builder.overrideValidationRule('title', {
  fieldName: 'title',
  ruleType: 'length',
  message: 'Title must be under 80 characters',
  parameters: { max: '80' }
});
```

---

## Preset Best Practices

### Choosing the Right Preset

1. **Single Platform**: Use specific preset (spotify, apple_music, etc.)
2. **Multiple Platforms**: Start with universal, add platform-specific validation
3. **Development/Testing**: Use universal for flexibility
4. **Production**: Use specific presets for compliance

### Preset Workflow

```typescript
// Development phase
builder.applyPreset('universal');
// ... build and test

// Platform-specific validation
const spotifyBuilder = new DdexBuilder();
spotifyBuilder.applyPreset('spotify');
const validation = await spotifyBuilder.validate();

// Production build
if (validation.isValid) {
  const xml = await spotifyBuilder.build();
}
```

### Error Handling

```typescript
try {
  builder.applyPreset('spotify');
  const xml = await builder.build();
} catch (error) {
  if (error.message.includes('Validation failed')) {
    // Check specific preset requirements
    const presetInfo = builder.getPresetInfo('spotify');
    console.log('Required fields:', presetInfo.requiredFields);
    
    // Get detailed validation results
    const validation = await builder.validate();
    validation.errors.forEach(err => console.error(err));
  }
}
```

---

## Platform-Specific Notes

### Spotify Considerations

- **Genre Mapping**: Use Spotify's official genre taxonomy
- **Featured Artists**: Include featured artists in title
- **Release Types**: Distinguish between Album, Single, EP
- **Territory Rights**: Specify streaming territories clearly

### Apple Music Considerations  

- **Artwork Standards**: 3000x3000px minimum resolution
- **Preview Points**: Specify preview start times
- **Language Support**: Provide localized metadata
- **Price Tiers**: Align with iTunes pricing structure

### YouTube Music Considerations

- **Content ID**: Ensure Content ID compatibility
- **Rights Management**: Detailed ownership information
- **Territory Mapping**: Granular geographic rights
- **Reference Quality**: High-quality reference files

### Amazon Music Considerations

- **Product Types**: Classify as Single, Album, etc.
- **Search Optimization**: Include relevant keywords
- **Brand Compliance**: Follow Amazon brand guidelines
- **Regional Variations**: Account for regional Amazon stores

This preset system ensures platform-specific compliance while maintaining flexibility for custom requirements and multi-platform distribution strategies.