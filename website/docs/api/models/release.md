# Release Models

Complete documentation of release data structures representing albums, singles, EPs, and other musical releases.

## Overview

Release models represent collections of musical content with associated metadata, commercial terms, and technical specifications. They form the primary organizational unit for music distribution.

## Release Types

| Type | Description | Typical Track Count | Use Cases |
|------|-------------|-------------------|-----------|
| `Album` | Full-length album | 8-20+ tracks | Major releases, LPs |
| `Single` | Single track release | 1-3 tracks | Radio singles, promotional |
| `EP` | Extended play | 3-8 tracks | Independent releases |
| `Compilation` | Collection of tracks | Variable | Greatest hits, anthologies |
| `Soundtrack` | Film/TV soundtrack | Variable | Media tie-ins |
| `Live` | Live recordings | Variable | Concert recordings |
| `Remix` | Remix collection | Variable | DJ releases, special editions |

## Graph Model

### Release Structure

```typescript
interface ReleaseDescriptor {
  ReleaseId: ReleaseId;
  ReleaseReference?: ReleaseReference;
  ReleaseType: ReleaseType;
  ReleaseDetailsByTerritory: ReleaseDetailsByTerritory[];
  ResourceGroup?: ResourceGroup[];
  ExternalResourceLink?: ExternalResourceLink[];
  Synopsis?: Synopsis;
  Genre?: Genre[];
  SubGenre?: SubGenre[];
  ReleaseDate?: EventDate;
  Duration?: Duration;
  Extensions?: ExtensionData[];
}
```

### ReleaseDetailsByTerritory

The core metadata structure varying by geographic territory:

```typescript
interface ReleaseDetailsByTerritory {
  TerritoryCode: AllTerritoryCode[];
  DisplayArtist: ArtistDescriptor[];
  LabelName: LabelName[];
  Title: TitleDescriptor[];
  ReleaseDate?: EventDate;
  OriginalReleaseDate?: EventDate;
  Genre?: Genre[];
  SubGenre?: SubGenre[];
  ParentalWarningType?: ParentalWarningType[];
  AvRating?: AvRating[];
  MarketingComment?: MarketingComment[];
  PLineDescriptor?: PLineDescriptor[];
  CLineDescriptor?: CLineDescriptor[];
  ResourceGroup?: ResourceGroup[];
  DisplayArtistName?: string;
  Keywords?: Keyword[];
  Moods?: Mood[];
  BriefSynopsis?: Synopsis;
  CatalogNumber?: CatalogNumber;
  ICPN?: ICPN;
}
```

#### Example Graph Structure

```typescript
{
  ReleaseId: "REL_INDIE_ALBUM_2024_001",
  ReleaseType: "Album",
  ReleaseDetailsByTerritory: [
    {
      TerritoryCode: ["Worldwide"],
      DisplayArtist: [
        {
          PartyName: [
            { FullName: "The Indie Band" }
          ],
          ArtistRole: ["MainArtist"]
        }
      ],
      LabelName: [
        { Value: "Independent Records" }
      ],
      Title: [
        {
          TitleText: "Breakthrough Album",
          TitleType: "DisplayTitle"
        }
      ],
      ReleaseDate: {
        Date: "2024-03-15"
      },
      Genre: [
        { Value: "Alternative Rock" }
      ],
      ParentalWarningType: ["NotExplicit"],
      PLineDescriptor: [
        {
          Year: "2024",
          PLineText: "2024 Independent Records"
        }
      ],
      CLineDescriptor: [
        {
          Year: "2024", 
          CLineText: "© 2024 The Indie Band"
        }
      ],
      CatalogNumber: "INDIE2024001"
    }
  ],
  ResourceGroup: [
    {
      ResourceGroup: [
        {
          ResourceGroupContentItem: [
            {
              ReleaseResourceReference: "RES_TRACK_001",
              LinkedReleaseResourceReference: ["RES_TRACK_002"]
            }
          ]
        }
      ]
    }
  ]
}
```

## Flattened Model

### Simplified Release Structure

```typescript
interface FlatRelease {
  releaseId: string;
  releaseType: string;
  releaseReference?: string;
  title: string;
  displayTitle?: string;
  sortingTitle?: string;
  displayArtist: string;
  displayArtistName?: string;
  labelName: string;
  catalogNumber?: string;
  upc?: string;
  ean?: string;
  icpn?: string;
  releaseDate: string;
  originalReleaseDate?: string;
  territory: string;
  genre: string;
  subGenre?: string;
  parentalWarning: boolean;
  explicitContent: boolean;
  duration: string;
  durationSeconds: number;
  trackCount: number;
  copyrightLine: string;
  producerCopyrightLine: string;
  marketingComment?: string;
  keywords: string[];
  moods: string[];
  synopsis?: string;
  resourceReferences: string[];
  artwork?: ArtworkReference[];
  contributors: FlatContributor[];
  metadata: Record<string, any>;
}
```

#### Example Flattened Structure

```typescript
{
  releaseId: "REL_INDIE_ALBUM_2024_001",
  releaseType: "Album",
  title: "Breakthrough Album",
  displayArtist: "The Indie Band",
  labelName: "Independent Records",
  catalogNumber: "INDIE2024001",
  releaseDate: "2024-03-15",
  territory: "Worldwide",
  genre: "Alternative Rock",
  parentalWarning: false,
  explicitContent: false,
  duration: "PT45M30S",
  durationSeconds: 2730,
  trackCount: 12,
  copyrightLine: "© 2024 The Indie Band",
  producerCopyrightLine: "℗ 2024 Independent Records",
  keywords: ["indie", "alternative", "rock", "breakthrough"],
  moods: ["energetic", "introspective"],
  resourceReferences: ["RES_TRACK_001", "RES_TRACK_002", "..."],
  contributors: [
    {
      name: "The Indie Band",
      role: "MainArtist",
      isPrimary: true
    },
    {
      name: "Producer Name",
      role: "Producer", 
      isPrimary: false
    }
  ],
  metadata: {
    recordingLocation: "Abbey Road Studios",
    recordingYear: "2023",
    masteredBy: "Mastering Engineer"
  }
}
```

## Title Structures

### Title Types

| Title Type | Purpose | Example |
|------------|---------|---------|
| `DisplayTitle` | Primary display title | "Breakthrough Album" |
| `SortTitle` | Sorting/alphabetization | "Breakthrough Album, The" |
| `SearchTitle` | Search optimization | "breakthrough album indie band" |
| `SubTitle` | Additional context | "Deluxe Edition" |
| `AlternativeTitle` | Alternative names | "Das Durchbruch Album" |

### Multi-language Titles

```typescript
interface TitleDescriptor {
  TitleText: string;
  TitleType?: TitleType;
  LanguageAndScriptCode?: string;
  SubTitle?: string;
  PartNumber?: number;
}

// Example: Multi-language release
{
  Title: [
    {
      TitleText: "Breakthrough Album",
      TitleType: "DisplayTitle",
      LanguageAndScriptCode: "en"
    },
    {
      TitleText: "Das Durchbruch Album", 
      TitleType: "DisplayTitle",
      LanguageAndScriptCode: "de"
    },
    {
      TitleText: "L'Album Révolutionnaire",
      TitleType: "DisplayTitle", 
      LanguageAndScriptCode: "fr"
    }
  ]
}
```

## Artist Descriptors

### Artist Roles

| Role | Description | Example |
|------|-------------|---------|
| `MainArtist` | Primary performing artist | "The Beatles" |
| `FeaturedArtist` | Guest/featured performer | "Ed Sheeran" |
| `Composer` | Music composer | "John Lennon" |
| `Lyricist` | Lyrics writer | "Paul McCartney" |
| `Producer` | Record producer | "George Martin" |
| `Remixer` | Remix artist | "David Guetta" |
| `Conductor` | Orchestra conductor | "Herbert von Karajan" |

### Artist Structure

```typescript
interface ArtistDescriptor {
  SequenceNumber?: number;
  PartyId?: PartyId[];
  PartyName?: PartyName[];
  ArtistRole?: ArtistRole[];
  DisplayArtist?: DisplayArtist[];
  TitleDisplayInformation?: TitleDisplayInformation[];
}

// Example: Complex artist credits
{
  DisplayArtist: [
    {
      PartyName: [{ FullName: "The Indie Band" }],
      ArtistRole: ["MainArtist"],
      SequenceNumber: 1
    },
    {
      PartyName: [{ FullName: "Famous Singer" }], 
      ArtistRole: ["FeaturedArtist"],
      SequenceNumber: 2
    },
    {
      PartyName: [{ FullName: "Renowned Producer" }],
      ArtistRole: ["Producer"],
      SequenceNumber: 3
    }
  ]
}
```

## Resource Grouping

### Resource Group Structure

```typescript
interface ResourceGroup {
  ResourceGroup?: ResourceGroup[];
  ResourceGroupContentItem?: ResourceGroupContentItem[];
  DisplayTitleText?: string;
  SequenceNumber?: number;
  NoDisplaySequence?: boolean;
  CarrierType?: CarrierType;
}

interface ResourceGroupContentItem {
  SequenceNumber?: number;
  ResourceType?: ResourceType;
  ReleaseResourceReference?: ReleaseResourceReference;
  LinkedReleaseResourceReference?: ReleaseResourceReference[];
  DurationUsedFromResource?: Duration;
  StartPoint?: StartPoint;
  EndPoint?: EndPoint;
}
```

### Grouping Patterns

#### Album with Multiple Discs

```typescript
{
  ResourceGroup: [
    {
      DisplayTitleText: "Disc 1",
      CarrierType: "CD",
      ResourceGroupContentItem: [
        { ReleaseResourceReference: "TRACK_01", SequenceNumber: 1 },
        { ReleaseResourceReference: "TRACK_02", SequenceNumber: 2 },
        { ReleaseResourceReference: "TRACK_03", SequenceNumber: 3 }
      ]
    },
    {
      DisplayTitleText: "Disc 2", 
      CarrierType: "CD",
      ResourceGroupContentItem: [
        { ReleaseResourceReference: "TRACK_04", SequenceNumber: 1 },
        { ReleaseResourceReference: "TRACK_05", SequenceNumber: 2 }
      ]
    }
  ]
}
```

#### Single with B-sides

```typescript
{
  ResourceGroup: [
    {
      DisplayTitleText: "Main Track",
      ResourceGroupContentItem: [
        { ReleaseResourceReference: "MAIN_SINGLE", SequenceNumber: 1 }
      ]
    },
    {
      DisplayTitleText: "B-Sides",
      ResourceGroupContentItem: [
        { ReleaseResourceReference: "B_SIDE_1", SequenceNumber: 1 },
        { ReleaseResourceReference: "B_SIDE_2", SequenceNumber: 2 }
      ]
    }
  ]
}
```

## Identifiers and Codes

### Standard Release Identifiers

| Identifier | Type | Format | Example |
|------------|------|--------|---------|
| UPC | Universal Product Code | 12 digits | `123456789012` |
| EAN | European Article Number | 13 digits | `1234567890123` |
| GRid | Global Release Identifier | 18 chars | `A1-2425G-ABC1234002-M` |
| ICPN | International Cataloguing of Musical Works | Variable | `ICPN12345678` |

### Catalog Numbers

```typescript
interface CatalogNumber {
  Value: string;
  Namespace?: string;
}

// Examples
{
  CatalogNumber: [
    { Value: "INDIE2024001" },                    // Label catalog number
    { Value: "DIST2024001", Namespace: "DIST" }, // Distributor number
    { Value: "UPC123456789012", Namespace: "UPC" } // UPC reference
  ]
}
```

## Date Handling

### Event Dates

```typescript
interface EventDate {
  Date?: string;              // YYYY-MM-DD
  ApproximateDate?: string;   // YYYY or YYYY-MM
  IsKnown?: boolean;
  LocationOfEvent?: AllTerritoryCode[];
}

// Examples
{
  ReleaseDate: { Date: "2024-03-15" },                    // Exact date
  OriginalReleaseDate: { ApproximateDate: "1969" },      // Year only
  RecordingDate: { Date: "2023-08-10", IsKnown: false }  // Estimated
}
```

### Date Validation Rules

- **Release Date**: Must not be more than 1 year in future
- **Original Release Date**: Must be before or equal to current release date
- **Recording Date**: Must be before release date
- **Format**: ISO 8601 date format (YYYY-MM-DD)

## Artwork and Visual Assets

### Artwork References

```typescript
interface ArtworkReference {
  artworkType: 'FrontCoverImage' | 'BackCoverImage' | 'BookletImage';
  imageId: string;
  imageFormat: 'JPEG' | 'PNG' | 'TIFF';
  imageResolution: string;
  colorDepth: number;
  imageSize: {
    width: number;
    height: number;
  };
  fileSize: number;
  url?: string;
  isMain: boolean;
}

// Example flattened artwork
{
  artwork: [
    {
      artworkType: 'FrontCoverImage',
      imageId: 'IMG_FRONT_001',
      imageFormat: 'JPEG',
      imageResolution: '300dpi',
      colorDepth: 24,
      imageSize: { width: 3000, height: 3000 },
      fileSize: 2048576,
      isMain: true
    }
  ]
}
```

## Copyright and Legal

### Copyright Lines

```typescript
interface CLineDescriptor {
  Year: string;
  CLineText: string;
  CLineType?: CLineType;
  CLineCompany?: PartyDescriptor;
}

interface PLineDescriptor {
  Year: string; 
  PLineText: string;
  PLineType?: PLineType;
  PLineCompany?: PartyDescriptor;
}

// Example usage
{
  CLineDescriptor: [
    {
      Year: "2024",
      CLineText: "© 2024 The Indie Band. All rights reserved.",
      CLineType: "Copyright"
    }
  ],
  PLineDescriptor: [
    {
      Year: "2024",
      PLineText: "℗ 2024 Independent Records",
      PLineType: "ProducerCopyright"
    }
  ]
}
```

## Release Assembly Patterns

### TypeScript Builder Pattern

```typescript
import { ReleaseBuilder } from 'ddex-builder';

const release = new ReleaseBuilder()
  .setReleaseId('REL_INDIE_2024_001')
  .setReleaseType('Album')
  .setTitle('Breakthrough Album')
  .setDisplayArtist('The Indie Band')
  .setLabel('Independent Records')
  .setCatalogNumber('INDIE2024001')
  .setReleaseDate('2024-03-15')
  .setGenre('Alternative Rock')
  .setTerritory('Worldwide')
  .addCopyright('© 2024 The Indie Band')
  .addProducerCopyright('℗ 2024 Independent Records')
  .addTrack('RES_TRACK_001')
  .addTrack('RES_TRACK_002')
  .addArtwork({
    type: 'FrontCoverImage',
    imageId: 'IMG_FRONT_001',
    format: 'JPEG',
    width: 3000,
    height: 3000
  })
  .build();
```

### Python Builder Pattern

```python
from ddex_builder import ReleaseBuilder

release = ReleaseBuilder() \
    .set_release_id('REL_INDIE_2024_001') \
    .set_release_type('Album') \
    .set_title('Breakthrough Album') \
    .set_display_artist('The Indie Band') \
    .set_label('Independent Records') \
    .set_catalog_number('INDIE2024001') \
    .set_release_date('2024-03-15') \
    .set_genre('Alternative Rock') \
    .set_territory('Worldwide') \
    .add_copyright('© 2024 The Indie Band') \
    .add_producer_copyright('℗ 2024 Independent Records') \
    .add_track('RES_TRACK_001') \
    .add_track('RES_TRACK_002') \
    .add_artwork(
        artwork_type='FrontCoverImage',
        image_id='IMG_FRONT_001',
        image_format='JPEG',
        width=3000,
        height=3000
    ) \
    .build()
```

## Validation Rules

### Required Fields

#### Minimum Requirements (All Versions)
- ReleaseId
- ReleaseType  
- Title (at least one)
- DisplayArtist (at least one)
- Territory specification

#### Platform-Specific Requirements

| Platform | Additional Required Fields |
|----------|---------------------------|
| Spotify | Label name, release date, genre |
| Apple Music | UPC, copyright lines, explicit flag |
| YouTube Music | ISRC for all tracks |
| Amazon Music | Catalog number, product classification |

### Business Logic Validation

```typescript
interface ReleaseValidationRule {
  field: string;
  rule: 'required' | 'format' | 'reference' | 'business';
  message: string;
  validator: (release: Release) => boolean;
}

const validationRules: ReleaseValidationRule[] = [
  {
    field: 'releaseDate',
    rule: 'business',
    message: 'Release date cannot be more than 1 year in future',
    validator: (release) => {
      const releaseDate = new Date(release.releaseDate);
      const oneYearFromNow = new Date();
      oneYearFromNow.setFullYear(oneYearFromNow.getFullYear() + 1);
      return releaseDate <= oneYearFromNow;
    }
  },
  {
    field: 'trackCount',
    rule: 'business', 
    message: 'Album must have at least 8 tracks',
    validator: (release) => {
      return release.releaseType !== 'Album' || release.trackCount >= 8;
    }
  }
];
```

## Performance Considerations

### Memory Optimization

```typescript
// Lazy loading for large releases
class LazyRelease {
  private _resourceDetails?: Map<string, Resource>;
  
  async getResourceDetails(resourceId: string): Promise<Resource> {
    if (!this._resourceDetails) {
      this._resourceDetails = await this.loadResourceDetails();
    }
    return this._resourceDetails.get(resourceId);
  }
  
  private async loadResourceDetails(): Promise<Map<string, Resource>> {
    // Load resources on demand
    return new Map();
  }
}
```

### Batch Processing

```python
# Process releases in batches for large catalogs
def process_release_batch(releases: List[dict], batch_size: int = 100):
    for i in range(0, len(releases), batch_size):
        batch = releases[i:i + batch_size]
        
        # Process batch
        processed_releases = []
        for release_data in batch:
            release = create_release_from_data(release_data)
            processed_releases.append(release)
        
        # Yield batch results
        yield processed_releases
```

## Best Practices

### Release Organization

1. **Consistent Naming**: Use consistent release ID patterns
2. **Territory Strategy**: Plan territory coverage carefully
3. **Genre Classification**: Use standard genre taxonomies
4. **Date Accuracy**: Ensure accurate and consistent dates
5. **Rights Information**: Include complete copyright information

### Multi-Territory Releases

1. **Territory Planning**: Consider regional release strategies
2. **Localization**: Provide localized titles and descriptions
3. **Rights Management**: Ensure territory-specific rights
4. **Date Coordination**: Coordinate release dates across territories
5. **Cultural Sensitivity**: Consider cultural preferences and restrictions

### Metadata Quality

1. **Title Consistency**: Maintain consistent title formatting
2. **Artist Credits**: Provide complete and accurate artist credits
3. **Genre Accuracy**: Use appropriate and specific genres
4. **Artwork Quality**: Ensure high-quality artwork assets
5. **Description Completeness**: Include comprehensive descriptions and keywords