---
sidebar_position: 5
---

# API Reference

Complete API documentation for DDEX Suite components.

## Core Classes

### DDEXParser

```typescript
class DDEXParser {
  constructor(options?: ParserOptions);
  parse(xml: string): Promise<ParseResult>;
  parseFile(path: string): Promise<ParseResult>;
  validate(xml: string): Promise<ValidationResult>;
}
```

### DDEXBuilder

```typescript
class DDEXBuilder {
  constructor(options?: BuilderOptions);
  build(request: BuildRequest): Promise<string>;
  validate(request: BuildRequest): Promise<ValidationResult>;
  createTemplate(template: TemplateOptions): Template;
}
```

## Type Definitions

### ParseResult

```typescript
interface ParseResult {
  // Flattened, developer-friendly representation
  flat: {
    releases: Release[];
    soundRecordings: SoundRecording[];
    deals: Deal[];
    parties: Party[];
  };
  
  // Original DDEX graph structure
  graph: DDEXMessage;
  
  // Convert to build request for round-trip
  toBuildRequest(): BuildRequest;
}
```

### Release

```typescript
interface Release {
  releaseId: string;
  title: string;
  artist: string;
  releaseDate: string;
  releaseType: 'Single' | 'Album' | 'EP';
  territories: string[];
  genres: string[];
  labelName?: string;
  catalogNumber?: string;
  upc?: string;
  copyrightYear?: number;
  productionYear?: number;
}
```

### SoundRecording

```typescript
interface SoundRecording {
  soundRecordingId: string;
  title: string;
  artist: string;
  duration: string; // ISO 8601 duration
  isrc?: string;
  languageOfPerformance?: string;
  recordingDate?: string;
  contributors: Contributor[];
}
```

### Deal

```typescript
interface Deal {
  dealId: string;
  territories: string[];
  useTypes: UseType[];
  commercialModelType: CommercialModelType;
  dealStartDate: string;
  dealEndDate?: string;
  priceInformation?: PriceInformation;
}
```

## Configuration Options

### ParserOptions

```typescript
interface ParserOptions {
  validation?: 'strict' | 'permissive' | 'none';
  maxMemoryMb?: number;
  maxEntityExpansions?: number;
  maxNestingDepth?: number;
  preserveWhitespace?: boolean;
  streaming?: boolean;
  bufferSizeMb?: number;
}
```

### BuilderOptions

```typescript
interface BuilderOptions {
  canonicalization?: 'db-c14n-1.0' | 'none';
  validation?: 'strict' | 'permissive' | 'none';
  preset?: 'spotify' | 'youtube' | 'apple' | 'generic';
  prettyPrint?: boolean;
  encoding?: string;
  generateDeterministicIds?: boolean;
}
```

## Error Classes

### DDEXParseError

```typescript
class DDEXParseError extends Error {
  line: number;
  column: number;
  context: string;
  errorType: 'InvalidXml' | 'Validation' | 'UnsupportedVersion' | 'Security';
}
```

### DDEXBuildError

```typescript
class DDEXBuildError extends Error {
  field: string;
  value: any;
  errorType: 'RequiredField' | 'InvalidValue' | 'Reference' | 'Territory';
}
```

## Enumerations

### DDEX Versions

```typescript
enum DDEXVersion {
  ERN_382 = '3.8.2',
  ERN_42 = '4.2', 
  ERN_43 = '4.3'
}
```

### Use Types

```typescript
enum UseType {
  Stream = 'Stream',
  PermanentDownload = 'PermanentDownload',
  ConditionalDownload = 'ConditionalDownload',
  NonInteractiveStream = 'NonInteractiveStream',
  OnDemandStream = 'OnDemandStream'
}
```

### Commercial Model Types

```typescript
enum CommercialModelType {
  Subscription = 'Subscription',
  PayAsYouGo = 'PayAsYouGo',
  FreeOfCharge = 'FreeOfCharge',
  AdvertisementSupportedModel = 'AdvertisementSupportedModel'
}
```

## Utility Functions

### Territory Utilities

```typescript
// Expand territory codes
expandTerritories(codes: string[]): string[];

// Validate territory codes
validateTerritories(codes: string[]): boolean;

// Get territory hierarchy
getTerritoryHierarchy(code: string): string[];
```

### Validation Utilities

```typescript
// Validate ISRC
validateISRC(isrc: string): boolean;

// Validate UPC
validateUPC(upc: string): boolean;

// Validate duration format
validateDuration(duration: string): boolean;
```

## Python API

The Python API mirrors the TypeScript API with Pythonic conventions:

```python
from ddex_parser import DDEXParser, ParserOptions
from ddex_builder import DDEXBuilder, BuilderOptions

# Parser
parser = DDEXParser(ParserOptions(
    validation='strict',
    max_memory_mb=100
))

result = parser.parse(xml_content)

# Builder  
builder = DDEXBuilder(BuilderOptions(
    preset='spotify',
    validation='strict'
))

xml = builder.build(build_request)
```

### DataFrame Integration

```python
# Parse to DataFrames
dfs = parser.to_dataframe('file.xml')

# Build from DataFrames
xml = builder.from_dataframes(dfs, version='4.3')
```

## Next Steps

- **[Parser Examples](../examples/)** - Parser usage examples
- **[Builder Examples](../examples/)** - Builder usage examples  
- **[Guides](../guides/)** - How-to guides for common tasks
- **[GitHub Repository](https://github.com/daddykev/ddex-suite)** - Source code and issues