# TypeScript API Reference

Complete API documentation for the DDEX Builder TypeScript/JavaScript bindings.

## Installation

```bash
npm install ddex-builder
```

## Imports

```typescript
import { 
  DdexBuilder, 
  StreamingDdexBuilder, 
  Release, 
  Resource,
  ValidationResult,
  BuilderStats,
  PresetInfo,
  batchBuild,
  validateStructure
} from 'ddex-builder';
```

## Classes

### DdexBuilder

Main builder class for creating deterministic DDEX XML messages.

```typescript
class DdexBuilder {
  constructor();
  addRelease(release: Release): void;
  addResource(resource: Resource): void;
  build(data?: any): Promise<string>;
  validate(): Promise<ValidationResult>;
  getStats(): BuilderStats;
  reset(): void;
  getAvailablePresets(): Array<string>;
  getPresetInfo(presetName: string): PresetInfo;
  applyPreset(presetName: string): void;
  getPresetValidationRules(presetName: string): Array<ValidationRule>;
}
```

#### Constructor

```typescript
const builder = new DdexBuilder();
```

Creates a new DDEX builder instance with default configuration.

#### addRelease()

```typescript
addRelease(release: Release): void
```

Adds a release to the message being built.

**Parameters:**
- `release: Release` - Release data structure

**Example:**
```typescript
const builder = new DdexBuilder();

const release: Release = {
  releaseId: 'REL001',
  releaseType: 'Album',
  title: 'My Album',
  artist: 'Artist Name',
  label: 'Record Label',
  catalogNumber: 'CAT001',
  upc: '123456789012',
  releaseDate: '2024-01-15',
  genre: 'Pop',
  parentalWarning: false,
  trackIds: ['TRK001', 'TRK002', 'TRK003'],
  metadata: {
    'custom_field': 'custom_value'
  }
};

builder.addRelease(release);
```

#### addResource()

```typescript
addResource(resource: Resource): void
```

Adds a resource (sound recording, video, etc.) to the message.

**Parameters:**
- `resource: Resource` - Resource data structure

**Example:**
```typescript
const resource: Resource = {
  resourceId: 'TRK001',
  resourceType: 'SoundRecording',
  title: 'Track Title',
  artist: 'Artist Name',
  isrc: 'USRC17607839',
  duration: 'PT3M45S',
  trackNumber: 1,
  volumeNumber: 1,
  metadata: {
    'composer': 'Composer Name',
    'producer': 'Producer Name'
  }
};

builder.addResource(resource);
```

#### build()

```typescript
build(data?: any): Promise<string>
```

Builds the DDEX XML message from added releases and resources.

**Parameters:**
- `data?: any` - Optional additional message data or BuildRequest object

**Returns:** `Promise<string>` - Generated DDEX XML

**Example:**
```typescript
const builder = new DdexBuilder();
builder.applyPreset('spotify');

// Add releases and resources...
builder.addRelease(release);
builder.addResource(resource);

// Build the XML
const xml = await builder.build({
  messageId: 'MSG_2024_001',
  sender: 'MyLabel',
  recipient: 'Spotify',
  version: '4.3'
});

console.log('Generated XML:', xml);
```

#### validate()

```typescript
validate(): Promise<ValidationResult>
```

Validates the current state of the builder without generating XML.

**Returns:** `Promise<ValidationResult>` - Validation results

**Example:**
```typescript
const builder = new DdexBuilder();
builder.addRelease(release);

const validation = await builder.validate();

if (validation.isValid) {
  console.log('✓ Validation passed');
} else {
  console.log('✗ Validation failed:');
  validation.errors.forEach(error => {
    console.log(`  - ${error}`);
  });
}

if (validation.warnings.length > 0) {
  console.log('Warnings:');
  validation.warnings.forEach(warning => {
    console.log(`  ! ${warning}`);
  });
}
```

#### getStats()

```typescript
getStats(): BuilderStats
```

Returns statistics about the builder's current state and performance.

**Returns:** `BuilderStats` - Builder statistics

**Example:**
```typescript
const builder = new DdexBuilder();
// Add data and build...

const stats = builder.getStats();
console.log(`Releases: ${stats.releasesCount}`);
console.log(`Resources: ${stats.resourcesCount}`);
console.log(`Build time: ${stats.totalBuildTimeMs}ms`);
console.log(`Output size: ${stats.lastBuildSizeBytes} bytes`);
console.log(`Validation errors: ${stats.validationErrors}`);
```

#### reset()

```typescript
reset(): void
```

Clears all added releases, resources, and statistics.

**Example:**
```typescript
const builder = new DdexBuilder();
// Add data...
builder.addRelease(release);

// Clear everything
builder.reset();

// Builder is now empty and ready for new data
```

#### getAvailablePresets()

```typescript
getAvailablePresets(): Array<string>
```

Returns list of available platform presets.

**Returns:** `Array<string>` - Array of preset names

**Example:**
```typescript
const builder = new DdexBuilder();
const presets = builder.getAvailablePresets();
console.log('Available presets:', presets);
// Output: ['spotify', 'apple_music', 'youtube_music', 'amazon_music', 'universal']
```

#### getPresetInfo()

```typescript
getPresetInfo(presetName: string): PresetInfo
```

Gets detailed information about a specific preset.

**Parameters:**
- `presetName: string` - Name of the preset

**Returns:** `PresetInfo` - Preset information

**Example:**
```typescript
const builder = new DdexBuilder();
const presetInfo = builder.getPresetInfo('spotify');

console.log(`Name: ${presetInfo.name}`);
console.log(`Description: ${presetInfo.description}`);
console.log(`Version: ${presetInfo.version}`);
console.log(`Profile: ${presetInfo.profile}`);
console.log(`Required fields: ${presetInfo.requiredFields.join(', ')}`);
console.log(`Disclaimer: ${presetInfo.disclaimer}`);
```

#### applyPreset()

```typescript
applyPreset(presetName: string): void
```

Applies a platform-specific preset configuration.

**Parameters:**
- `presetName: string` - Name of the preset to apply

**Example:**
```typescript
const builder = new DdexBuilder();

// Apply Spotify preset
builder.applyPreset('spotify');

// The builder is now configured for Spotify requirements
// - Specific validation rules
// - Required fields
// - Format preferences
```

#### getPresetValidationRules()

```typescript
getPresetValidationRules(presetName: string): Array<ValidationRule>
```

Gets the validation rules for a specific preset.

**Parameters:**
- `presetName: string` - Name of the preset

**Returns:** `Array<ValidationRule>` - Array of validation rules

**Example:**
```typescript
const builder = new DdexBuilder();
const rules = builder.getPresetValidationRules('spotify');

rules.forEach(rule => {
  console.log(`Field: ${rule.fieldName}`);
  console.log(`Rule: ${rule.ruleType}`);
  console.log(`Message: ${rule.message}`);
  if (rule.parameters) {
    console.log(`Parameters:`, rule.parameters);
  }
});
```

---

### StreamingDdexBuilder

Streaming builder for memory-efficient generation of large DDEX catalogs.

```typescript
class StreamingDdexBuilder {
  constructor(config?: StreamingConfig);
  setProgressCallback(callback: (...args: any[]) => any): void;
  setEstimatedTotal(total: number): void;
  startMessage(header: MessageHeader, version: string): void;
  writeResource(resourceId: string, title: string, artist: string, isrc?: string, duration?: string, filePath?: string): string;
  finishResourcesStartReleases(): void;
  writeRelease(releaseId: string, title: string, artist: string, label: string, upc: string, releaseDate: string, genre: string, resourceReferences: Array<string>): string;
  finishMessage(): StreamingStats;
  getXml(): string;
  reset(): void;
}
```

#### Constructor

```typescript
const streamBuilder = new StreamingDdexBuilder(config?: StreamingConfig);
```

Creates a new streaming builder with optional configuration.

**Parameters:**
- `config?: StreamingConfig` - Optional streaming configuration

**Example:**
```typescript
const streamBuilder = new StreamingDdexBuilder({
  maxBufferSize: 10 * 1024 * 1024,  // 10MB buffer
  deterministic: true,
  validateDuringStream: true,
  progressCallbackFrequency: 100    // Callback every 100 items
});
```

#### setProgressCallback()

```typescript
setProgressCallback(callback: (...args: any[]) => any): void
```

Sets a callback function to receive progress updates during streaming.

**Parameters:**
- `callback: Function` - Progress callback function

**Example:**
```typescript
const streamBuilder = new StreamingDdexBuilder();

streamBuilder.setProgressCallback((progress: StreamingProgress) => {
  const percent = progress.estimatedCompletionPercent || 0;
  console.log(`Progress: ${percent.toFixed(1)}%`);
  console.log(`Releases: ${progress.releasesWritten}`);
  console.log(`Memory: ${(progress.currentMemoryUsage / 1024 / 1024).toFixed(1)}MB`);
});
```

#### setEstimatedTotal()

```typescript
setEstimatedTotal(total: number): void
```

Sets the estimated total number of items for accurate progress reporting.

**Parameters:**
- `total: number` - Estimated total number of releases

**Example:**
```typescript
const streamBuilder = new StreamingDdexBuilder();
streamBuilder.setEstimatedTotal(10000); // Expecting 10,000 releases
```

#### startMessage()

```typescript
startMessage(header: MessageHeader, version: string): void
```

Starts a new DDEX message with the specified header and version.

**Parameters:**
- `header: MessageHeader` - Message header information
- `version: string` - DDEX version ('3.8.2', '4.2', or '4.3')

**Example:**
```typescript
const streamBuilder = new StreamingDdexBuilder();

const header: MessageHeader = {
  messageId: 'MSG_CATALOG_2024_001',
  messageSenderName: 'MyRecordLabel',
  messageRecipientName: 'Spotify',
  messageCreatedDateTime: new Date().toISOString()
};

streamBuilder.startMessage(header, '4.3');
```

#### writeResource()

```typescript
writeResource(resourceId: string, title: string, artist: string, isrc?: string, duration?: string, filePath?: string): string
```

Writes a resource (sound recording) to the streaming output.

**Parameters:**
- `resourceId: string` - Unique resource identifier
- `title: string` - Resource title
- `artist: string` - Artist name
- `isrc?: string` - Optional ISRC code
- `duration?: string` - Optional duration (ISO 8601 format)
- `filePath?: string` - Optional file path reference

**Returns:** `string` - Generated resource reference ID

**Example:**
```typescript
const streamBuilder = new StreamingDdexBuilder();
streamBuilder.startMessage(header, '4.3');

const resourceRef = streamBuilder.writeResource(
  'RES_001',
  'Track Title',
  'Artist Name',
  'USRC17607839',
  'PT3M45S',
  '/audio/track001.wav'
);

console.log(`Resource reference: ${resourceRef}`);
```

#### finishResourcesStartReleases()

```typescript
finishResourcesStartReleases(): void
```

Finishes the resources section and starts the releases section.

**Example:**
```typescript
const streamBuilder = new StreamingDdexBuilder();
streamBuilder.startMessage(header, '4.3');

// Write all resources...
streamBuilder.writeResource(...);
streamBuilder.writeResource(...);

// Transition to releases
streamBuilder.finishResourcesStartReleases();

// Now write releases...
```

#### writeRelease()

```typescript
writeRelease(releaseId: string, title: string, artist: string, label: string, upc: string, releaseDate: string, genre: string, resourceReferences: Array<string>): string
```

Writes a release to the streaming output.

**Parameters:**
- `releaseId: string` - Unique release identifier
- `title: string` - Release title
- `artist: string` - Primary artist
- `label: string` - Record label name
- `upc: string` - Universal Product Code
- `releaseDate: string` - Release date (ISO 8601)
- `genre: string` - Musical genre
- `resourceReferences: Array<string>` - Array of resource reference IDs

**Returns:** `string` - Generated release reference ID

**Example:**
```typescript
const releaseRef = streamBuilder.writeRelease(
  'REL_001',
  'Album Title',
  'Artist Name',
  'Record Label',
  '123456789012',
  '2024-01-15',
  'Pop',
  [resourceRef1, resourceRef2, resourceRef3]
);

console.log(`Release reference: ${releaseRef}`);
```

#### finishMessage()

```typescript
finishMessage(): StreamingStats
```

Finishes the message and returns statistics.

**Returns:** `StreamingStats` - Final streaming statistics

**Example:**
```typescript
const streamBuilder = new StreamingDdexBuilder();
// Build the message...

const stats = streamBuilder.finishMessage();
console.log(`Final stats:`);
console.log(`  Releases written: ${stats.releasesWritten}`);
console.log(`  Resources written: ${stats.resourcesWritten}`);
console.log(`  Deals written: ${stats.dealsWritten}`);
console.log(`  Total bytes: ${stats.bytesWritten}`);
console.log(`  Peak memory: ${stats.peakMemoryUsage} bytes`);

if (stats.warnings.length > 0) {
  console.log(`Warnings:`);
  stats.warnings.forEach(warning => console.log(`  - ${warning}`));
}
```

#### getXml()

```typescript
getXml(): string
```

Returns the generated XML content.

**Returns:** `string` - Complete DDEX XML

**Example:**
```typescript
const streamBuilder = new StreamingDdexBuilder();
// Build the message...
streamBuilder.finishMessage();

const xml = streamBuilder.getXml();
console.log(`Generated ${xml.length} characters of XML`);

// Save to file
import { writeFileSync } from 'fs';
writeFileSync('catalog.xml', xml, 'utf-8');
```

#### reset()

```typescript
reset(): void
```

Resets the streaming builder for a new message.

**Example:**
```typescript
const streamBuilder = new StreamingDdexBuilder();
// Build first message...
streamBuilder.finishMessage();

// Reset for next message
streamBuilder.reset();
streamBuilder.startMessage(newHeader, '4.3');
```

---

## Global Functions

### batchBuild()

```typescript
function batchBuild(requests: Array<string>): Promise<Array<string>>
```

Builds multiple DDEX messages in a single operation for improved performance.

**Parameters:**
- `requests: Array<string>` - Array of JSON-serialized build requests

**Returns:** `Promise<Array<string>>` - Array of generated XML strings

**Example:**
```typescript
import { batchBuild } from 'ddex-builder';

const requests = [
  JSON.stringify({ releases: [release1], version: '4.3' }),
  JSON.stringify({ releases: [release2], version: '4.3' }),
  JSON.stringify({ releases: [release3], version: '4.3' })
];

const xmlResults = await batchBuild(requests);
xmlResults.forEach((xml, index) => {
  console.log(`Request ${index + 1}: ${xml.length} characters`);
});
```

### validateStructure()

```typescript
function validateStructure(xml: string): Promise<ValidationResult>
```

Validates the structure of existing DDEX XML without building.

**Parameters:**
- `xml: string` - DDEX XML content to validate

**Returns:** `Promise<ValidationResult>` - Validation results

**Example:**
```typescript
import { validateStructure } from 'ddex-builder';
import { readFileSync } from 'fs';

const xml = readFileSync('existing_ddex.xml', 'utf-8');
const validation = await validateStructure(xml);

if (validation.isValid) {
  console.log('✓ XML structure is valid');
} else {
  console.log('✗ XML structure has errors:');
  validation.errors.forEach(error => console.log(`  - ${error}`));
}
```

---

## Error Handling

The builder throws specific errors for different failure conditions:

```typescript
import { DdexBuilder } from 'ddex-builder';

const builder = new DdexBuilder();

try {
  builder.addRelease(invalidRelease);
  const xml = await builder.build();
} catch (error) {
  if (error.message.includes('Validation failed')) {
    console.error('Validation error:', error.message);
    // Check validation results
    const validation = await builder.validate();
    validation.errors.forEach(err => console.error(`  - ${err}`));
  } else if (error.message.includes('Missing required field')) {
    console.error('Required field missing:', error.message);
  } else if (error.message.includes('Invalid preset')) {
    console.error('Preset error:', error.message);
  } else {
    console.error('Unexpected error:', error.message);
  }
}
```

### Common Error Types

- **Validation Errors**: Data doesn't meet DDEX requirements
- **Preset Errors**: Invalid or unknown preset names
- **Reference Errors**: Invalid resource references in releases
- **Format Errors**: Invalid date formats, durations, or identifiers
- **Memory Errors**: Insufficient memory for large catalogs
- **Configuration Errors**: Invalid streaming configuration

---

## Performance Tips

### Memory Management

```typescript
// Use streaming for large catalogs
const streamBuilder = new StreamingDdexBuilder({
  maxBufferSize: 50 * 1024 * 1024,  // 50MB
  validateDuringStream: false        // Validate at end for speed
});

// Process in chunks
for (const chunk of releaseChunks) {
  chunk.forEach(release => {
    streamBuilder.writeRelease(...);
  });
  
  // Optional: Force garbage collection
  if (global.gc) global.gc();
}
```

### Batch Processing

```typescript
// Build multiple messages efficiently
const buildRequests = releases.map(release => ({
  releases: [release],
  preset: 'spotify',
  version: '4.3'
}));

const xmlResults = await batchBuild(
  buildRequests.map(req => JSON.stringify(req))
);
```

### Validation Optimization

```typescript
// Skip validation during building for speed
const builder = new DdexBuilder();
builder.applyPreset('spotify');

// Add all data...
releases.forEach(release => builder.addRelease(release));

// Validate once at the end
const validation = await builder.validate();
if (validation.isValid) {
  const xml = await builder.build();
}
```