# Shared Types and Interfaces

Common type definitions used across TypeScript and Python APIs for the DDEX Parser.

## Core Configuration Types

### DDEXParserOptions / ParseOptions {#parseoptions}

Parser configuration interface with language-specific variations.

#### TypeScript
```typescript
interface DDEXParserOptions {
  includeRawExtensions?: boolean;
  includeComments?: boolean;
  validateReferences?: boolean;
  streaming?: boolean;
}
```

#### Python
```python
class ParseOptions:
    include_raw_extensions: bool = False
    include_comments: bool = False
    validate_references: bool = True
    streaming: bool = False
    timeout: float = 30.0
```

#### Configuration Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `includeRawExtensions` / `include_raw_extensions` | `boolean` | `false` | Include raw XML for extension elements |
| `includeComments` / `include_comments` | `boolean` | `false` | Include XML comments in output |
| `validateReferences` / `validate_references` | `boolean` | `true` | Validate resource references |
| `streaming` | `boolean` | `false` | Enable streaming mode for large files |
| `timeout` (Python only) | `number` | `30.0` | Parsing timeout in seconds |

---

## Result Types

### ParseResult Structure {#parseresult}

The main result object returned by both TypeScript and Python parsers.

#### TypeScript
```typescript
interface ParseResult {
  graph: MessageGraph;
  flat: MessageFlat;
}
```

#### Python
```python
class ParseResult:
    message_id: str
    version: str
    release_count: int
    releases: List[Dict[str, Any]]
    # Internal: _data contains graph/flat structure
```

### Graph vs Flat Representations

#### Graph Representation
Faithful to the original DDEX XML structure, preserving hierarchy and relationships.

```typescript
interface MessageGraph {
  MessageHeader: {
    MessageId: string;
    MessageSender: PartyDescriptor;
    MessageRecipient: PartyDescriptor;
    MessageCreatedDateTime: string;
  };
  UpdateIndicator: string;
  MessageControlType: string;
  ReleaseList?: ReleaseDescriptor[];
  ResourceList?: ResourceDescriptor[];
  DealList?: DealDescriptor[];
}
```

#### Flat Representation
Denormalized, analysis-friendly structure with joined data.

```typescript
interface MessageFlat {
  messageInfo: {
    messageId: string;
    version: string;
    sender: string;
    recipient: string;
    date: string;
  };
  releases: FlatRelease[];
  soundRecordings: FlatSoundRecording[];
  dealTerms: FlatDealTerm[];
}
```

---

## Data Structure Types

### Release Types

#### Graph Release
```typescript
interface ReleaseDescriptor {
  ReleaseId: ReleaseId;
  ReleaseType: ReleaseType;
  ReleaseReference?: string;
  ReleaseDetailsByTerritory: ReleaseDetailsByTerritory[];
  ResourceGroup?: ResourceGroup[];
  ExternalResourceLink?: ExternalResourceLink[];
}

interface ReleaseDetailsByTerritory {
  TerritoryCode: AllTerritoryCode[];
  DisplayArtist: ArtistDescriptor[];
  LabelName: LabelName[];
  Title: TitleDescriptor[];
  ReleaseDate?: EventDate;
  Genre?: Genre[];
  PLineDescriptor?: PLineDescriptor[];
  CLineDescriptor?: CLineDescriptor[];
}
```

#### Flat Release
```typescript
interface FlatRelease {
  releaseId: string;
  releaseType: string;
  title: string;
  displayArtist: string;
  labelName: string;
  releaseDate: string;
  territory: string;
  genre?: string;
  catalogNumber?: string;
  upc?: string;
  pLine?: string;
  cLine?: string;
  trackCount: number;
  totalDuration: number;
}
```

### Sound Recording Types

#### Graph Sound Recording
```typescript
interface SoundRecordingDescriptor {
  SoundRecordingId: SoundRecordingId;
  SoundRecordingType: SoundRecordingType;
  SoundRecordingDetailsByTerritory: SoundRecordingDetailsByTerritory[];
  MusicalWorkContained?: MusicalWork[];
}

interface SoundRecordingDetailsByTerritory {
  TerritoryCode: AllTerritoryCode[];
  Title: TitleDescriptor[];
  DisplayArtist: ArtistDescriptor[];
  LabelName?: LabelName[];
  ISRC?: ISRC;
  Duration?: Duration;
  Genre?: Genre[];
  TrackNumber?: number;
  VolumeNumber?: number;
}
```

#### Flat Sound Recording
```typescript
interface FlatSoundRecording {
  soundRecordingId: string;
  isrc: string;
  title: string;
  displayArtist: string;
  duration: string;
  durationSeconds: number;
  trackNumber?: number;
  volumeNumber?: number;
  genre?: string;
  territory: string;
  contributors: FlatContributor[];
}
```

### Deal Types

#### Graph Deal
```typescript
interface DealDescriptor {
  DealId: DealId;
  DealType: DealType;
  CommercialModelType: CommercialModelType[];
  Usage: Usage[];
  DealTerms: DealTerms;
  DealReleaseReference: DealReleaseReference[];
}

interface DealTerms {
  CommercialModelType: CommercialModelType[];
  ValidityPeriod?: Period;
  TerritoryCode?: AllTerritoryCode[];
  DistributionChannel?: DistributionChannel[];
  PriceInformation?: PriceInformation[];
}
```

#### Flat Deal Term
```typescript
interface FlatDealTerm {
  dealId: string;
  dealType: string;
  commercialModelType: string;
  territory: string;
  distributionChannel: string;
  validFrom?: string;
  validUntil?: string;
  currency?: string;
  priceType?: string;
  wholeSalePrice?: number;
  consumerPrice?: number;
  releaseIds: string[];
}
```

---

## Identifier Types

### Standard Identifiers
```typescript
// DDEX Standard Identifiers
type ISRC = string;           // International Standard Recording Code
type ISWC = string;           // International Standard Musical Work Code
type UPC = string;            // Universal Product Code
type EAN = string;            // European Article Number
type GRid = string;           // Global Release Identifier
type SICI = string;           // Serial Item and Contribution Identifier

// Party Identifiers
type DPID = string;           // DDEX Party Identifier
type ISNI = string;           // International Standard Name Identifier
type IPI = string;            // Interested Parties Information

// Custom Identifiers
interface ProprietaryId {
  Namespace: string;
  Value: string;
}
```

### ID Types by Context
```typescript
type ReleaseId = string | ProprietaryId;
type SoundRecordingId = string | ProprietaryId;
type MusicalWorkId = string | ProprietaryId;
type DealId = string | ProprietaryId;
type PartyId = string | ProprietaryId;
```

---

## Geographic and Territory Types

### Territory Codes
```typescript
// ISO 3166-1 alpha-2 country codes
type TerritoryCode = 'US' | 'GB' | 'DE' | 'FR' | 'JP' | 'CA' | 'AU' | /* ... */;

// Special territory designations
type SpecialTerritory = 'Worldwide' | 'Europe' | 'NorthAmerica' | 'AsiaPacific';

type AllTerritoryCode = TerritoryCode | SpecialTerritory;
```

### Territory Exclusions
```typescript
interface TerritoryScope {
  IncludedTerritoryCode?: AllTerritoryCode[];
  ExcludedTerritoryCode?: AllTerritoryCode[];
}
```

---

## Temporal Types

### Dates and Periods
```typescript
// ISO 8601 date formats
type Date = string;           // YYYY-MM-DD
type DateTime = string;       // YYYY-MM-DDTHH:mm:ssZ
type PartialDate = string;    // YYYY or YYYY-MM

interface EventDate {
  Date?: Date;
  ApproximateDate?: PartialDate;
}

interface Period {
  StartDate?: EventDate;
  EndDate?: EventDate;
}
```

### Duration Types
```typescript
// ISO 8601 duration format: PT[hours]H[minutes]M[seconds]S
type Duration = string;       // e.g., "PT3M45S" for 3:45

interface DurationInfo {
  duration: Duration;
  durationSeconds: number;    // Converted to seconds for calculations
}
```

---

## Commercial Types

### Deal and Usage Types
```typescript
type DealType = 'License' | 'Assignment' | 'UserDefined';

type CommercialModelType = 
  | 'SubscriptionModel'
  | 'DownloadModel' 
  | 'StreamingModel'
  | 'PhysicalModel'
  | 'UserDefined';

type Usage = 
  | 'OnDemandStream'
  | 'NonInteractiveStream'
  | 'PermanentDownload'
  | 'ConditionalDownload'
  | 'PhysicalDistribution'
  | 'UserDefined';

type DistributionChannel = 
  | 'Internet'
  | 'Mobile'
  | 'Satellite'
  | 'PhysicalDistribution'
  | 'UserDefined';
```

### Price Information
```typescript
interface PriceInformation {
  PriceType: 'WholesalePrice' | 'ConsumerPrice' | 'SuggestedRetailPrice';
  CurrencyCode: string;        // ISO 4217 currency code
  PriceAmount: number;
}
```

---

## Artist and Contributor Types

### Artist Descriptors
```typescript
interface ArtistDescriptor {
  PartyId?: PartyId[];
  PartyName?: PartyName[];
  ArtistRole?: ArtistRole[];
}

interface PartyName {
  FullName: string;
  FullNameAsciiTranscribed?: string;
  FullNameIndexed?: string;
  NamesBeforeKeyName?: string;
  KeyName?: string;
  NamesAfterKeyName?: string;
}

type ArtistRole = 
  | 'MainArtist'
  | 'FeaturedArtist' 
  | 'Remixer'
  | 'Producer'
  | 'Composer'
  | 'Lyricist'
  | 'UserDefined';
```

### Flat Contributor
```typescript
interface FlatContributor {
  name: string;
  role: string;
  partyId?: string;
  isPrimary: boolean;
}
```

---

## Technical Types

### Resource Types
```typescript
type ResourceType = 
  | 'SoundRecording'
  | 'MusicalWork'
  | 'Video'
  | 'Image'
  | 'Text'
  | 'Software'
  | 'UserDefined';

type SoundRecordingType = 
  | 'MusicalWorkSoundRecording'
  | 'NonMusicalWorkSoundRecording'
  | 'UserDefined';

type ReleaseType = 
  | 'Album'
  | 'Single'
  | 'EP'
  | 'Compilation'
  | 'Soundtrack'
  | 'UserDefined';
```

### File and Technical Data
```typescript
interface TechnicalDetails {
  Duration?: Duration;
  BitRate?: number;
  SampleRate?: number;
  NumberOfChannels?: number;
  BitsPerSample?: number;
  FileFormat?: string;
  FileSize?: number;
  FingerPrint?: FingerPrint[];
}

interface FingerPrint {
  Algorithm: string;
  Parameter?: string;
  DataType?: string;
  Value: string;
}
```

---

## Validation and Error Types

### Validation Results
```typescript
interface ValidationResult {
  isValid: boolean;
  version: string;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

interface ValidationError {
  code: string;
  message: string;
  line?: number;
  column?: number;
  xpath?: string;
}

interface ValidationWarning {
  code: string;
  message: string;
  suggestion?: string;
}
```

### Streaming Types
```typescript
interface StreamingProgress {
  bytesProcessed: number;
  elementsProcessed: number;
  estimatedTotal?: number;
  percentComplete?: number;
  elapsedMs: number;
}

interface StreamingOptions {
  chunkSize?: number;
  maxMemoryMB?: number;
  bufferSize?: number;
}
```

---

## Version-Specific Variations

### ERN 3.8.2 Differences
- Limited territory support
- Simplified deal structures
- Different namespace URIs

### ERN 4.2 Differences  
- Enhanced metadata support
- Additional resource types
- Extended commercial model types

### ERN 4.3 Features
- Full streaming deal support
- Enhanced artist roles
- Improved territory handling
- Additional identifier types

### Version Detection
```typescript
type DDEXVersion = '3.8.2' | '4.2' | '4.3' | 'Unknown';

interface VersionInfo {
  version: DDEXVersion;
  namespace: string;
  profile?: string;
  detectedFeatures: string[];
}
```

---

## Utility Types

### Optional and Required Variants
```typescript
// Make all properties optional for partial updates
type PartialRelease = Partial<FlatRelease>;

// Make specific properties required
type RequiredRelease = Required<Pick<FlatRelease, 'releaseId' | 'title' | 'displayArtist'>>;

// Combine with optional properties
type CreateReleaseRequest = RequiredRelease & Partial<FlatRelease>;
```

### Array and Collection Types
```typescript
type ReleaseCollection = FlatRelease[];
type SoundRecordingCollection = FlatSoundRecording[];
type DealTermCollection = FlatDealTerm[];

// Lookup maps
type ReleaseMap = Record<string, FlatRelease>;
type SoundRecordingMap = Record<string, FlatSoundRecording>;
```

### Filter and Query Types
```typescript
interface ReleaseFilter {
  artist?: string;
  genre?: string;
  territory?: string;
  releaseDateFrom?: string;
  releaseDateTo?: string;
  labelName?: string;
}

interface SoundRecordingFilter {
  isrc?: string;
  title?: string;
  artist?: string;
  durationMin?: number;
  durationMax?: number;
  genre?: string;
}
```