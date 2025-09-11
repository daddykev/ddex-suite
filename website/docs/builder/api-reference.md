---
sidebar_position: 4
---

# API Reference

Complete reference documentation for the DDEX Builder API across all supported languages.

## DdexBuilder Class

The main builder class for generating deterministic DDEX XML from structured data.

### Constructor

#### JavaScript / TypeScript

```typescript
import { DdexBuilder } from 'ddex-builder';

const builder = new DdexBuilder(options?: BuilderOptions);
```

#### Python

```python
from ddex_builder import DdexBuilder

builder = DdexBuilder(**options)
```

### Configuration Options

#### BuilderOptions (TypeScript)

```typescript
interface BuilderOptions {
  // Validation settings
  validate?: boolean;           // Enable/disable validation (default: true)
  validationLevel?: 'strict' | 'permissive' | 'none';
  
  // Output format
  canonical?: boolean;          // Enable DB-C14N/1.0 (default: true)
  prettyPrint?: boolean;        // Format XML with indentation (default: false)
  encoding?: string;            // XML encoding (default: 'UTF-8')
  
  // Performance options
  parallel?: boolean;           // Enable parallel processing (default: false)
  streaming?: boolean;          // Enable streaming mode (default: false)
  maxMemoryMB?: number;         // Memory limit in MB (default: 512)
  
  // Preset configuration
  preset?: string;              // Apply preset ('youtube_album', 'generic_audio_album', etc.)
  
  // Advanced options
  deterministicIds?: boolean;   // Generate content-based IDs (default: true)
  preserveOrder?: boolean;      // Preserve input element order (default: true)
  includeMetadata?: boolean;    // Include builder metadata (default: false)
}
```

#### BuilderOptions (Python)

```python
from ddex_builder import DdexBuilder, BuilderOptions

options = BuilderOptions(
    validate=True,                    # Enable validation
    validation_level='strict',        # 'strict' | 'permissive' | 'none'
    canonical=True,                   # Enable DB-C14N/1.0
    pretty_print=False,               # Format output
    encoding='UTF-8',                 # XML encoding
    parallel=False,                   # Parallel processing
    streaming=False,                  # Streaming mode
    max_memory_mb=512,                # Memory limit
    preset=None,                      # Apply preset
    deterministic_ids=True,           # Content-based IDs
    preserve_order=True,              # Preserve order
    include_metadata=False            # Include metadata
)
```

## Core Building Methods

### build()

Generate DDEX XML from structured data.

#### JavaScript / TypeScript

```typescript
async build(data: BuildRequest): Promise<string>
```

**Parameters:**
- `data`: Structured data containing message header, releases, resources, etc.

**Returns:** Promise resolving to DDEX XML string

**Example:**
```typescript
const xml = await builder.build({
  messageHeader: {
    messageId: 'MSG_001',
    messageSenderName: 'My Label'
  },
  releases: [{
    releaseId: 'REL_001',
    title: 'My Album',
    artist: 'Artist Name'
  }],
  resources: [{
    resourceId: 'SR_001',
    resourceType: 'SoundRecording',
    title: 'Track Title'
  }]
});
```

#### Python

```python
def build(self, data: Dict[str, Any]) -> str
async def build_async(self, data: Dict[str, Any]) -> str
```

**Parameters:**
- `data`: Dictionary containing structured DDEX data

**Returns:** DDEX XML string

**Example:**
```python
xml = builder.build({
    'message_header': {
        'message_id': 'MSG_001',
        'message_sender_name': 'My Label'
    },
    'releases': [{
        'release_id': 'REL_001',
        'title': 'My Album',
        'artist': 'Artist Name'
    }]
})
```

### validate()

Validate structured data without building XML.

#### JavaScript / TypeScript

```typescript
async validate(data?: BuildRequest): Promise<ValidationResult>
```

**Returns:** ValidationResult with validation status and errors

**Example:**
```typescript
const result = await builder.validate(data);
if (!result.isValid) {
  console.log('Errors:', result.errors);
  console.log('Warnings:', result.warnings);
}
```

#### Python

```python
def validate(self, data: Dict[str, Any]) -> ValidationResult
```

**Example:**
```python
result = builder.validate(data)
if not result.is_valid:
    print('Errors:', result.errors)
    print('Warnings:', result.warnings)
```

## Streaming Methods

### StreamingDdexBuilder Class

For building large DDEX files with minimal memory usage.

#### JavaScript / TypeScript

```typescript
import { StreamingDdexBuilder } from 'ddex-builder';

const streamBuilder = new StreamingDdexBuilder(config?: StreamingConfig);
```

#### StreamingConfig

```typescript
interface StreamingConfig {
  maxBufferSize: number;              // Buffer size in bytes (default: 8192)
  deterministic: boolean;             // Enable deterministic output (default: true)
  validateDuringStream: boolean;      // Validate while streaming (default: true)
  progressCallbackFrequency: number;  // Progress callback frequency (default: 1000)
}
```

#### Streaming Methods

```typescript
// Start building a message
startMessage(header: MessageHeader, version: string): void

// Write a sound recording resource
writeResource(
  resourceId: string,
  title: string,
  artist: string,
  isrc?: string,
  duration?: string,
  filePath?: string
): string

// Finish resources section and start releases
finishResourcesStartReleases(): void

// Write a release
writeRelease(
  releaseId: string,
  title: string,
  artist: string,
  label: string,
  upc?: string,
  releaseDate?: string,
  genre?: string,
  resourceReferences: string[]
): string

// Finish the message and get final stats
finishMessage(): StreamingStats

// Get the complete XML
getXml(): string

// Reset for new message
reset(): void
```

#### Streaming Example

```typescript
const streamBuilder = new StreamingDdexBuilder({
  maxBufferSize: 16384,
  validateDuringStream: true
});

// Set progress callback
streamBuilder.setProgressCallback((progress: StreamingProgress) => {
  console.log(`Progress: ${progress.estimatedCompletionPercent}%`);
});

// Start message
streamBuilder.startMessage({
  messageId: 'STREAM_001',
  messageSenderName: 'My Label',
  messageRecipientName: 'Platform'
}, '4.3');

// Add resources
const resourceIds = [];
for (const track of largeTrackList) {
  const resourceId = streamBuilder.writeResource(
    track.id,
    track.title,
    track.artist,
    track.isrc,
    track.duration
  );
  resourceIds.push(resourceId);
}

// Switch to releases section
streamBuilder.finishResourcesStartReleases();

// Add releases
streamBuilder.writeRelease(
  'REL_001',
  'Large Album',
  'Artist Name',
  'Label Name',
  '123456789012',
  '2024-01-01',
  'Pop',
  resourceIds
);

// Finish and get results
const stats = streamBuilder.finishMessage();
const xml = streamBuilder.getXml();

console.log(`Generated ${stats.bytesWritten} bytes with ${stats.releasesWritten} releases`);
```

## Batch Processing

### batchBuild()

Build multiple DDEX messages in parallel.

#### JavaScript / TypeScript

```typescript
import { batchBuild } from 'ddex-builder';

async function batchBuild(
  requests: BuildRequest[],
  options?: BatchBuildOptions
): Promise<BatchBuildResult[]>
```

#### BatchBuildOptions

```typescript
interface BatchBuildOptions {
  preset?: string;           // Apply preset to all builds
  parallel?: boolean;        // Enable parallel processing (default: true)
  maxConcurrency?: number;   // Max concurrent builds (default: 4)
  stopOnError?: boolean;     // Stop on first error (default: false)
  progressCallback?: (progress: BatchProgress) => void;
}
```

#### Example

```typescript
const requests = [
  { messageHeader: {...}, releases: [...], resources: [...] },
  { messageHeader: {...}, releases: [...], resources: [...] },
  { messageHeader: {...}, releases: [...], resources: [...] }
];

const results = await batchBuild(requests, {
  preset: 'youtube_album',
  parallel: true,
  maxConcurrency: 10,
  progressCallback: (progress) => {
    console.log(`${progress.completed}/${progress.total} completed`);
  }
});

// Process results
results.forEach((result, index) => {
  if (result.success) {
    console.log(`✅ Built request ${index}: ${result.xml.length} bytes`);
  } else {
    console.error(`❌ Failed request ${index}: ${result.error}`);
  }
});
```

## Preset System

### Managing Presets

#### JavaScript / TypeScript

```typescript
// Get available presets
getAvailablePresets(): string[]

// Get preset information
getPresetInfo(presetName: string): PresetInfo

// Apply a preset
applyPreset(presetName: string): void

// Get preset validation rules
getPresetValidationRules(presetName: string): ValidationRule[]
```

#### Python

```python
# Get available presets
def get_available_presets(self) -> List[str]

# Get preset information  
def get_preset_info(self, preset_name: str) -> PresetInfo

# Apply a preset
def apply_preset(self, preset_name: str) -> None

# Get preset validation rules
def get_preset_validation_rules(self, preset_name: str) -> List[ValidationRule]
```

#### PresetInfo Structure

```typescript
interface PresetInfo {
  name: string;                    // Preset identifier
  description: string;             // Human-readable description
  version: string;                 // Preset version
  profile: string;                 // Target platform/profile
  requiredFields: string[];        // Required data fields
  disclaimer: string;              // Usage disclaimer
}
```

#### ValidationRule Structure

```typescript
interface ValidationRule {
  fieldName: string;               // Field path (e.g., 'releases.0.title')
  ruleType: string;                // Rule type (required, format, length, etc.)
  message: string;                 // Human-readable error message
  parameters?: Record<string, string>; // Rule parameters
}
```

#### Example

```typescript
// Explore available presets
const presets = builder.getAvailablePresets();
console.log('Available presets:', presets);

// Get Spotify preset details
const youtubeInfo = builder.getPresetInfo('youtube_album');
console.log('YouTube preset:', youtubeInfo.description);
console.log('Required fields:', youtubeInfo.requiredFields);
console.log('Specification:', youtubeInfo.specification);

// Apply preset and see rules
builder.applyPreset('youtube_album');
const rules = builder.getPresetValidationRules('youtube_album');
rules.forEach(rule => {
  console.log(`${rule.fieldName}: ${rule.message}`);
});
```

## Data Structures

### BuildRequest

The main data structure for building DDEX XML.

#### JavaScript / TypeScript

```typescript
interface BuildRequest {
  messageHeader: MessageHeader;
  releases: Release[];
  resources: Resource[];
  deals?: Deal[];
  parties?: Party[];
  metadata?: Record<string, any>;
}

interface MessageHeader {
  messageId?: string;                    // Auto-generated if not provided
  messageSenderName: string;             // Required: Sender identification
  messageRecipientName: string;          // Required: Recipient identification
  messageCreatedDateTime?: string;       // Auto-generated if not provided
  messageControlType?: string;           // Default: 'LiveMessage'
  sentOnBehalfOf?: string;               // Party ID if sending on behalf
}

interface Release {
  releaseId: string;                     // Unique release identifier
  releaseType: string;                   // 'Album', 'Single', 'EP', etc.
  title: string;                         // Release title
  artist: string;                        // Main artist name
  label?: string;                        // Record label name
  catalogNumber?: string;                // Catalog number
  upc?: string;                         // Universal Product Code
  releaseDate?: string;                 // Release date (YYYY-MM-DD)
  originalReleaseDate?: string;         // Original release date
  genre?: string;                       // Primary genre
  genres?: string[];                    // Multiple genres
  territories?: string[];               // Available territories
  parentalWarning?: boolean;            // Explicit content flag
  trackIds: string[];                   // References to resources
  metadata?: Record<string, any>;       // Additional metadata
}

interface Resource {
  resourceId: string;                   // Unique resource identifier
  resourceType: string;                 // 'SoundRecording', 'Image', 'Video', etc.
  title: string;                        // Resource title
  artist: string;                       // Performing artist
  isrc?: string;                       // International Standard Recording Code
  duration?: string;                   // Duration in ISO 8601 format
  trackNumber?: number;                // Position on release
  volumeNumber?: number;               // Volume/disc number
  languageOfPerformance?: string;      // Language code (ISO 639-1)
  filePath?: string;                   // File path for streaming
  metadata?: Record<string, any>;      // Additional metadata
}

interface Deal {
  dealId: string;                      // Unique deal identifier
  releaseId?: string;                  // Associated release
  territories: string[];               // Deal territories
  useTypes: string[];                  // Usage types (Stream, Download, etc.)
  commercialModelType: string;         // Business model
  dealStartDate?: string;              // Deal start date
  dealEndDate?: string;                // Deal end date
  priceInformation?: PriceInfo;        // Pricing details
  conditions?: string[];               // Deal conditions
}

interface Party {
  partyId: string;                     // Unique party identifier
  partyName: string;                   // Party name
  partyType: string;                   // 'Label', 'Artist', 'Publisher', etc.
  contactInfo?: ContactInfo;           // Contact information
  roles?: string[];                    // Party roles
}
```

### ValidationResult

Result of validation operations.

#### JavaScript / TypeScript

```typescript
interface ValidationResult {
  isValid: boolean;                    // Overall validation status
  errors: string[];                    // Validation errors (blocking)
  warnings: string[];                  // Validation warnings (non-blocking)
  fieldErrors?: FieldError[];          // Detailed field errors
}

interface FieldError {
  field: string;                       // Field path
  message: string;                     // Error message
  value?: any;                        // Invalid value
  suggestions?: string[];              // Suggested corrections
}
```

#### Python

```python
from ddex_builder import ValidationResult

class ValidationResult:
    is_valid: bool                     # Overall validation status
    errors: List[str]                  # Validation errors
    warnings: List[str]                # Validation warnings
    field_errors: List[FieldError]     # Detailed field errors
```

### StreamingStats

Statistics from streaming operations.

```typescript
interface StreamingStats {
  releasesWritten: number;             # Number of releases written
  resourcesWritten: number;            # Number of resources written
  dealsWritten: number;                # Number of deals written
  bytesWritten: number;                # Total bytes written
  warnings: string[];                  # Warnings encountered
  peakMemoryUsage: number;             # Peak memory usage in bytes
}
```

## DataFrame Integration (Python Only)

### from_dataframes()

Build DDEX XML directly from pandas DataFrames.

```python
def from_dataframes(
    self,
    dataframes: Dict[str, pd.DataFrame],
    message_header: Dict[str, Any],
    version: str = '4.3'
) -> str
```

**Parameters:**
- `dataframes`: Dictionary mapping entity types to DataFrames
- `message_header`: Message header information
- `version`: DDEX version to generate

**Example:**
```python
import pandas as pd

# Create DataFrames
releases_df = pd.DataFrame([{
    'release_id': 'REL_001',
    'title': 'My Album',
    'artist': 'Artist Name',
    'upc': '123456789012'
}])

resources_df = pd.DataFrame([{
    'resource_id': 'SR_001',
    'resource_type': 'SoundRecording',
    'title': 'Track 1',
    'isrc': 'US1234567890'
}])

# Build DDEX
xml = builder.from_dataframes({
    'releases': releases_df,
    'resources': resources_df
}, message_header={
    'message_id': 'MSG_001',
    'message_sender_name': 'My Label'
})
```

## Utility Functions

### validateStructure()

Validate DDEX XML structure without building.

#### JavaScript / TypeScript

```typescript
import { validateStructure } from 'ddex-builder';

async function validateStructure(xml: string): Promise<ValidationResult>
```

**Example:**
```typescript
const validation = await validateStructure(existingXml);
if (!validation.isValid) {
  console.log('XML validation errors:', validation.errors);
}
```

## Error Handling

### Error Types

#### JavaScript / TypeScript

```typescript
class BuilderError extends Error {
  code: string;                        // Error code
  field?: string;                      // Related field
  value?: any;                        // Related value
}

class ValidationError extends BuilderError {
  details: FieldError[];               // Detailed validation errors
}

class PresetError extends BuilderError {
  presetName: string;                  // Related preset name
}

class StreamingError extends BuilderError {
  streamPosition: number;              // Position in stream where error occurred
}
```

#### Python

```python
from ddex_builder import BuilderError, ValidationError, PresetError

class BuilderError(Exception):
    """Base exception for builder errors."""
    def __init__(self, message: str, code: str = None, field: str = None):
        super().__init__(message)
        self.code = code
        self.field = field

class ValidationError(BuilderError):
    """DDEX validation error."""
    def __init__(self, message: str, details: List[FieldError] = None):
        super().__init__(message)
        self.details = details or []

class PresetError(BuilderError):
    """Preset configuration error."""
    def __init__(self, message: str, preset_name: str):
        super().__init__(message)
        self.preset_name = preset_name
```

### Error Handling Example

```typescript
import { DdexBuilder, ValidationError, BuilderError } from 'ddex-builder';

try {
  const xml = await builder.build(data);
} catch (error) {
  if (error instanceof ValidationError) {
    console.error('Validation failed:', error.message);
    error.details.forEach(detail => {
      console.error(`  ${detail.field}: ${detail.message}`);
      if (detail.suggestions) {
        console.log(`    Suggestions: ${detail.suggestions.join(', ')}`);
      }
    });
  } else if (error instanceof BuilderError) {
    console.error('Build failed:', error.message);
    if (error.field) {
      console.error(`  Field: ${error.field}`);
    }
  } else {
    console.error('Unexpected error:', error.message);
  }
}
```

## Performance Monitoring

### getStats()

Get builder performance statistics.

#### JavaScript / TypeScript

```typescript
getStats(): BuilderStats
```

#### BuilderStats

```typescript
interface BuilderStats {
  releasesCount: number;               // Total releases built
  resourcesCount: number;              // Total resources built
  totalBuildTimeMs: number;            // Cumulative build time
  lastBuildSizeBytes: number;          // Size of last build
  validationErrors: number;            // Total validation errors
  validationWarnings: number;          // Total validation warnings
}
```

#### Example

```typescript
const stats = builder.getStats();
console.log(`Built ${stats.releasesCount} releases`);
console.log(`Average build time: ${stats.totalBuildTimeMs / stats.releasesCount}ms`);
console.log(`Last build: ${stats.lastBuildSizeBytes} bytes`);
```

This comprehensive API reference covers all major features and methods available in the DDEX Builder. For implementation examples and advanced usage patterns, see the [Advanced Usage](./advanced-usage) guide.