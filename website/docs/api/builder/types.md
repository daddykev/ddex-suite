# Builder Types and Interfaces

Type definitions and interfaces for the DDEX Builder API across TypeScript and Python.

## Core Build Types

### Release Interface

Data structure for releases in both TypeScript and Python.

#### TypeScript
```typescript
interface Release {
  releaseId: string;
  releaseType: string;
  title: string;
  artist: string;
  label?: string;
  catalogNumber?: string;
  upc?: string;
  releaseDate?: string;
  genre?: string;
  parentalWarning?: boolean;
  trackIds: Array<string>;
  metadata?: Record<string, string>;
}
```

#### Python
```python
class Release(TypedDict):
    release_id: str
    release_type: str
    title: str
    artist: str
    label: NotRequired[str]
    catalog_number: NotRequired[str]
    upc: NotRequired[str]
    release_date: NotRequired[str]
    genre: NotRequired[str]
    parental_warning: NotRequired[bool]
    track_ids: List[str]
    metadata: NotRequired[Dict[str, str]]
```

#### Field Descriptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `releaseId` / `release_id` | `string` | ✓ | Unique release identifier |
| `releaseType` / `release_type` | `string` | ✓ | Type: "Album", "Single", "EP", etc. |
| `title` | `string` | ✓ | Release title |
| `artist` | `string` | ✓ | Primary artist name |
| `label` | `string` | ○ | Record label name |
| `catalogNumber` / `catalog_number` | `string` | ○ | Label catalog number |
| `upc` | `string` | ○ | Universal Product Code |
| `releaseDate` / `release_date` | `string` | ○ | Release date (ISO 8601: YYYY-MM-DD) |
| `genre` | `string` | ○ | Musical genre |
| `parentalWarning` / `parental_warning` | `boolean` | ○ | Contains explicit content |
| `trackIds` / `track_ids` | `string[]` | ✓ | Array of resource IDs |
| `metadata` | `Record<string, string>` | ○ | Custom metadata fields |

---

### Resource Interface

Data structure for resources (sound recordings, videos, etc.).

#### TypeScript
```typescript
interface Resource {
  resourceId: string;
  resourceType: string;
  title: string;
  artist: string;
  isrc?: string;
  duration?: string;
  trackNumber?: number;
  volumeNumber?: number;
  metadata?: Record<string, string>;
}
```

#### Python
```python
class Resource(TypedDict):
    resource_id: str
    resource_type: str
    title: str
    artist: str
    isrc: NotRequired[str]
    duration: NotRequired[str]
    track_number: NotRequired[int]
    volume_number: NotRequired[int]
    metadata: NotRequired[Dict[str, str]]
```

#### Field Descriptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `resourceId` / `resource_id` | `string` | ✓ | Unique resource identifier |
| `resourceType` / `resource_type` | `string` | ✓ | "SoundRecording", "Video", "Image", etc. |
| `title` | `string` | ✓ | Resource title |
| `artist` | `string` | ✓ | Artist/performer name |
| `isrc` | `string` | ○ | International Standard Recording Code |
| `duration` | `string` | ○ | Duration in ISO 8601 format (PT3M45S) |
| `trackNumber` / `track_number` | `number` | ○ | Track number on release |
| `volumeNumber` / `volume_number` | `number` | ○ | Volume/disc number |
| `metadata` | `Record<string, string>` | ○ | Custom metadata fields |

---

## Validation Types

### ValidationResult

Result of validation operations.

#### TypeScript
```typescript
interface ValidationResult {
  isValid: boolean;
  errors: Array<string>;
  warnings: Array<string>;
}
```

#### Python
```python
class ValidationResult:
    is_valid: bool
    errors: List[str]
    warnings: List[str]
```

#### Properties

- **`isValid` / `is_valid`**: Whether validation passed
- **`errors`**: List of validation error messages
- **`warnings`**: List of validation warning messages

**Example Usage:**
```typescript
const validation = await builder.validate();
if (!validation.isValid) {
  console.error('Validation failed:', validation.errors);
}
```

```python
validation = await builder.validate()
if not validation.is_valid:
    print('Validation failed:', validation.errors)
```

---

### ValidationRule

Individual validation rule definition.

#### TypeScript
```typescript
interface ValidationRule {
  fieldName: string;
  ruleType: string;
  message: string;
  parameters?: Record<string, string>;
}
```

#### Python
```python
class ValidationRule:
    field_name: str
    rule_type: str
    message: str
    parameters: Optional[Dict[str, str]]
```

#### Rule Types

| Rule Type | Description | Example |
|-----------|-------------|---------|
| `required` | Field must be present | title, artist, releaseId |
| `format` | Field must match format | ISRC, UPC, date formats |
| `length` | Field length constraints | title max 100 chars |
| `enum` | Field must be from allowed values | releaseType, genre |
| `reference` | Field must reference valid ID | trackIds must exist |
| `custom` | Platform-specific rule | Spotify metadata requirements |

---

## Statistics and Metrics

### BuilderStats

Builder performance and content statistics.

#### TypeScript
```typescript
interface BuilderStats {
  releasesCount: number;
  resourcesCount: number;
  totalBuildTimeMs: number;
  lastBuildSizeBytes: number;
  validationErrors: number;
  validationWarnings: number;
}
```

#### Python
```python
class BuilderStats:
    releases_count: int
    resources_count: int
    total_build_time_ms: int
    last_build_size_bytes: int
    validation_errors: int
    validation_warnings: int
```

#### Properties

- **`releasesCount` / `releases_count`**: Number of releases added
- **`resourcesCount` / `resources_count`**: Number of resources added
- **`totalBuildTimeMs` / `total_build_time_ms`**: Total build time in milliseconds
- **`lastBuildSizeBytes` / `last_build_size_bytes`**: Size of last generated XML
- **`validationErrors` / `validation_errors`**: Number of validation errors
- **`validationWarnings` / `validation_warnings`**: Number of validation warnings

---

## Streaming Types

### StreamingConfig

Configuration for streaming builder.

#### TypeScript
```typescript
interface StreamingConfig {
  maxBufferSize: number;
  deterministic: boolean;
  validateDuringStream: boolean;
  progressCallbackFrequency: number;
}
```

#### Python
```python
class StreamingConfig(TypedDict):
    max_buffer_size: int
    deterministic: bool
    validate_during_stream: bool
    progress_callback_frequency: int
```

#### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `maxBufferSize` / `max_buffer_size` | `number` | 10MB | Maximum memory buffer size |
| `deterministic` | `boolean` | `true` | Enable deterministic output |
| `validateDuringStream` / `validate_during_stream` | `boolean` | `true` | Validate while streaming |
| `progressCallbackFrequency` / `progress_callback_frequency` | `number` | 100 | Progress callback frequency |

---

### StreamingProgress

Progress information during streaming operations.

#### TypeScript
```typescript
interface StreamingProgress {
  releasesWritten: number;
  resourcesWritten: number;
  bytesWritten: number;
  currentMemoryUsage: number;
  estimatedCompletionPercent?: number;
}
```

#### Python
```python
class StreamingProgress(TypedDict):
    releases_written: int
    resources_written: int
    bytes_written: int
    current_memory_usage: int
    estimated_completion_percent: NotRequired[float]
```

---

### StreamingStats

Final statistics after streaming completion.

#### TypeScript
```typescript
interface StreamingStats {
  releasesWritten: number;
  resourcesWritten: number;
  dealsWritten: number;
  bytesWritten: number;
  warnings: Array<string>;
  peakMemoryUsage: number;
}
```

#### Python
```python
class StreamingStats:
    releases_written: int
    resources_written: int
    deals_written: int
    bytes_written: int
    warnings: List[str]
    peak_memory_usage: int
```

---

## Message Structure Types

### MessageHeader

Header information for DDEX messages.

#### TypeScript
```typescript
interface MessageHeader {
  messageId?: string;
  messageSenderName: string;
  messageRecipientName: string;
  messageCreatedDateTime?: string;
}
```

#### Python
```python
class MessageHeader(TypedDict):
    message_id: NotRequired[str]
    message_sender_name: str
    message_recipient_name: str
    message_created_date_time: NotRequired[str]
```

#### Field Descriptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `messageId` / `message_id` | `string` | ○ | Unique message identifier (auto-generated if not provided) |
| `messageSenderName` / `message_sender_name` | `string` | ✓ | Name of message sender |
| `messageRecipientName` / `message_recipient_name` | `string` | ✓ | Name of message recipient |
| `messageCreatedDateTime` / `message_created_date_time` | `string` | ○ | Creation timestamp (ISO 8601) |

---

### BuildRequest

Complete build request structure.

#### TypeScript
```typescript
interface BuildRequest {
  messageHeader?: MessageHeader;
  version?: string;
  updateIndicator?: string;
  messageControlType?: string;
  releases?: Release[];
  resources?: Resource[];
  deals?: Deal[];
  preset?: string;
  validation?: ValidationOptions;
}
```

#### Python
```python
class BuildRequest(TypedDict):
    message_header: NotRequired[MessageHeader]
    version: NotRequired[str]
    update_indicator: NotRequired[str]
    message_control_type: NotRequired[str]
    releases: NotRequired[List[Release]]
    resources: NotRequired[List[Resource]]
    deals: NotRequired[List[Deal]]
    preset: NotRequired[str]
    validation: NotRequired[ValidationOptions]
```

#### Build Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `messageHeader` / `message_header` | `MessageHeader` | Auto-generated | Message header information |
| `version` | `string` | "4.3" | DDEX version ("3.8.2", "4.2", "4.3") |
| `updateIndicator` / `update_indicator` | `string` | "OriginalMessage" | Update type indicator |
| `messageControlType` / `message_control_type` | `string` | "LiveMessage" | Message control type |
| `releases` | `Release[]` | `[]` | Array of releases |
| `resources` | `Resource[]` | `[]` | Array of resources |
| `deals` | `Deal[]` | `[]` | Array of deal terms |
| `preset` | `string` | none | Platform preset to apply |
| `validation` | `ValidationOptions` | default | Validation configuration |

---

## Preset Types

### PresetInfo

Information about platform presets.

#### TypeScript
```typescript
interface PresetInfo {
  name: string;
  description: string;
  version: string;
  profile: string;
  requiredFields: Array<string>;
  disclaimer: string;
}
```

#### Python
```python
class PresetInfo:
    name: str
    description: str
    version: str
    profile: str
    required_fields: List[str]
    disclaimer: str
```

#### Available Presets

| Preset | Name | Description | Profile |
|--------|------|-------------|---------|
| Spotify | `spotify` | Optimized for Spotify ingestion | ERN 4.3 Streaming |
| Apple Music | `apple_music` | iTunes/Apple Music requirements | ERN 4.3 Download |
| YouTube Music | `youtube_music` | YouTube Content ID compliance | ERN 4.3 Streaming |
| Amazon Music | `amazon_music` | Amazon DSP specifications | ERN 4.3 Mixed |
| Universal | `universal` | Generic streaming platform | ERN 4.3 Universal |

---

## Commercial Deal Types

### Deal

Commercial deal structure for licensing terms.

#### TypeScript
```typescript
interface Deal {
  dealId: string;
  dealType: string;
  commercialModelType: string;
  territory: string;
  distributionChannel: string;
  validFrom?: string;
  validUntil?: string;
  priceInformation?: PriceInfo[];
  releaseReferences: string[];
}
```

#### Python
```python
class Deal(TypedDict):
    deal_id: str
    deal_type: str
    commercial_model_type: str
    territory: str
    distribution_channel: str
    valid_from: NotRequired[str]
    valid_until: NotRequired[str]
    price_information: NotRequired[List[PriceInfo]]
    release_references: List[str]
```

### PriceInfo

Price information for commercial deals.

#### TypeScript
```typescript
interface PriceInfo {
  priceType: string;
  currencyCode: string;
  priceAmount: number;
}
```

#### Python
```python
class PriceInfo(TypedDict):
    price_type: str
    currency_code: str
    price_amount: float
```

---

## Validation Configuration

### ValidationOptions

Options for controlling validation behavior.

#### TypeScript
```typescript
interface ValidationOptions {
  strictMode?: boolean;
  customRules?: ValidationRule[];
  skipFields?: string[];
  errorOnWarnings?: boolean;
}
```

#### Python
```python
class ValidationOptions(TypedDict):
    strict_mode: NotRequired[bool]
    custom_rules: NotRequired[List[ValidationRule]]
    skip_fields: NotRequired[List[str]]
    error_on_warnings: NotRequired[bool]
```

#### Validation Levels

| Level | Description | Use Case |
|-------|-------------|----------|
| `relaxed` | Basic structure validation | Development, testing |
| `standard` | DDEX specification compliance | Production, general use |
| `strict` | Platform-specific requirements | Platform submission |
| `custom` | User-defined rules | Specialized workflows |

---

## DataFrame Integration Types (Python)

### DataFrameSchema

Schema definition for DataFrame-to-DDEX conversion.

```python
class DataFrameSchema:
    release_fields: Dict[str, str]
    resource_fields: Dict[str, str]
    deal_fields: Dict[str, str]
    required_columns: List[str]
    date_columns: List[str]
    numeric_columns: List[str]
```

#### Standard DataFrame Columns

| Column | Type | Description |
|--------|------|-------------|
| `release_id` | `string` | Release identifier |
| `title` | `string` | Release/track title |
| `artist` | `string` | Artist name |
| `label` | `string` | Record label |
| `release_date` | `string` | Release date (YYYY-MM-DD) |
| `genre` | `string` | Musical genre |
| `upc` | `string` | Universal Product Code |
| `sound_recording_id` | `string` | Track identifier |
| `isrc` | `string` | International Standard Recording Code |
| `duration` | `string` | Track duration (PT3M45S) |
| `track_number` | `int` | Track number |
| `territory` | `string` | Geographic territory |
| `deal_type` | `string` | Commercial deal type |

---

## Utility Types

### Result Types

Generic result types for operations that may fail.

#### TypeScript
```typescript
type BuildResult<T> = {
  success: true;
  data: T;
} | {
  success: false;
  error: string;
  details?: ValidationResult;
}

type AsyncBuildResult<T> = Promise<BuildResult<T>>;
```

#### Python
```python
from typing import Union, Generic, TypeVar

T = TypeVar('T')

class BuildSuccess(Generic[T]):
    success: Literal[True]
    data: T

class BuildFailure:
    success: Literal[False]
    error: str
    details: Optional[ValidationResult]

BuildResult = Union[BuildSuccess[T], BuildFailure]
```

### Collection Types

```typescript
// TypeScript
type ReleaseCollection = Release[];
type ResourceCollection = Resource[];
type ValidationRuleCollection = ValidationRule[];

// Lookup maps
type ReleaseMap = Record<string, Release>;
type ResourceMap = Record<string, Resource>;
type PresetMap = Record<string, PresetInfo>;
```

```python
# Python
ReleaseCollection = List[Release]
ResourceCollection = List[Resource]
ValidationRuleCollection = List[ValidationRule]

# Lookup maps
ReleaseMap = Dict[str, Release]
ResourceMap = Dict[str, Resource]
PresetMap = Dict[str, PresetInfo]
```

### Filter Types

```typescript
// TypeScript
interface ReleaseFilter {
  artist?: string;
  genre?: string;
  releaseDateFrom?: string;
  releaseDateTo?: string;
  labelName?: string;
}

interface ResourceFilter {
  isrc?: string;
  title?: string;
  artist?: string;
  durationMin?: number;
  durationMax?: number;
}
```

```python
# Python
class ReleaseFilter(TypedDict, total=False):
    artist: str
    genre: str
    release_date_from: str
    release_date_to: str
    label_name: str

class ResourceFilter(TypedDict, total=False):
    isrc: str
    title: str
    artist: str
    duration_min: int
    duration_max: int
```