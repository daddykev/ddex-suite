# 📚 DDEX Suite API Documentation

*Complete API reference for all DDEX Suite components and language bindings*

## 📖 Documentation Index

### 🚀 Getting Started
- **[Quick Start Guide](./QUICK_START.md)** - Get up and running in 5 minutes
- **[User Guide](./user-guide.md)** - Comprehensive feature walkthrough  
- **[Developer Guide](./developer-guide.md)** - Advanced integration patterns
- **[Security Audit](./security-audit.md)** - Security features and best practices

### 🦀 Rust Core API
- **[Rust Documentation](../target/doc/index.html)** - Auto-generated rustdoc for all crates
  - [ddex-core](../target/doc/ddex_core/index.html) - Shared data models and utilities
  - [ddex-parser](../target/doc/ddex_parser/index.html) - DDEX XML parsing engine  
  - [ddex-builder](../target/doc/ddex_builder/index.html) - DDEX XML generation engine

### 🟢 Node.js/TypeScript API

#### DDEX Parser
- **Package**: `ddex-parser` on [npm](https://www.npmjs.com/package/ddex-parser)
- **Documentation**: [TypeScript Docs](../packages/ddex-parser/bindings/node/docs/index.html)
- **Types**: Full TypeScript definitions included
- **GitHub**: [Source Code](../packages/ddex-parser/bindings/node/)

```typescript
import { DDEXParser, ParseOptions, ParsedERNMessage } from 'ddex-parser';

const parser = new DDEXParser();
const result: ParsedERNMessage = await parser.parse(xmlContent);
```

#### DDEX Builder  
- **Package**: `ddex-builder` on [npm](https://www.npmjs.com/package/ddex-builder)
- **Documentation**: [TypeScript Docs](../packages/ddex-builder/bindings/node/docs/index.html)
- **Types**: Full TypeScript definitions included
- **GitHub**: [Source Code](../packages/ddex-builder/bindings/node/)

```typescript
import { DDEXBuilder, BuildRequest, BuildResult } from 'ddex-builder';

const builder = new DDEXBuilder();
const result: BuildResult = await builder.build(request);
```

### 🐍 Python API

#### DDEX Parser
- **Package**: `ddex-parser` on [PyPI](https://pypi.org/project/ddex-parser/)
- **Documentation**: [Python API Reference](./python_parser_api.txt)
- **Type Stubs**: Included for IDE support
- **GitHub**: [Source Code](../packages/ddex-parser/bindings/python/)

```python
from ddex_parser import DDEXParser
import pandas as pd

parser = DDEXParser()
result = parser.parse(xml_content)
df = parser.to_dataframe(xml_content, schema='flat')
```

#### DDEX Builder
- **Package**: `ddex-builder` on [PyPI](https://pypi.org/project/ddex-builder/)  
- **Documentation**: [Python API Reference](./python_builder_api.txt)
- **Type Stubs**: Included for IDE support
- **GitHub**: [Source Code](../packages/ddex-builder/bindings/python/)

```python
from ddex_builder import DDEXBuilder

builder = DDEXBuilder()
result = builder.build(request)
```

### 🌐 WebAssembly (WASM) API
- **Package**: Included with Node.js packages
- **Documentation**: [WASM API Guide](./WASM_API.md)
- **Bundle Size**: 114KB (77% under 500KB target)
- **Browser Support**: Chrome 69+, Firefox 60+, Safari 13+

```javascript
import { DDEXBuilder } from 'ddex-builder';  // Auto-detects WASM in browser

const builder = new DDEXBuilder();
const result = await builder.build(request);
```

---

## 🎯 API Quick Reference

### Core Classes

#### DDEXParser
Parse DDEX XML messages into structured data.

**Node.js/TypeScript:**
```typescript
class DDEXParser {
    constructor(options?: ParserOptions);
    parse(xml: string | Buffer): Promise<ParsedERNMessage>;
    parseSync(xml: string | Buffer): ParsedERNMessage;
    stream(source: ReadableStream): AsyncIterator<ParsedRelease>;
    detectVersion(xml: string | Buffer): ERNVersion;
    sanityCheck(xml: string | Buffer): Promise<SanityCheckResult>;
}
```

**Python:**
```python
class DDEXParser:
    def __init__(self, options: Optional[ParserOptions] = None)
    def parse(self, xml: Union[str, bytes]) -> ParsedERNMessage
    async def parse_async(self, xml: Union[str, bytes]) -> ParsedERNMessage
    def stream(self, source: IO) -> Iterator[ParsedRelease]
    def to_dataframe(self, xml: Union[str, bytes], schema: str = 'flat') -> pd.DataFrame
    def detect_version(self, xml: Union[str, bytes]) -> ERNVersion
```

#### DDEXBuilder  
Generate deterministic DDEX XML from structured data.

**Node.js/TypeScript:**
```typescript
class DDEXBuilder {
    constructor();
    build(request: BuildRequest, options?: BuildOptions): Promise<BuildResult>;
    buildSync(request: BuildRequest, options?: BuildOptions): BuildResult;
    stream(request: BuildRequest): WritableStream;
    preflight(request: BuildRequest): Promise<PreflightResult>;
    applyPreset(preset: string, options?: PresetOptions): Promise<void>;
    validateStructure(request: BuildRequest): Promise<ValidationResult>;
}
```

**Python:**
```python
class DDEXBuilder:
    def __init__(self)
    def build(self, request: BuildRequest) -> BuildResult
    async def build_async(self, request: BuildRequest) -> BuildResult
    def preflight(self, request: BuildRequest) -> PreflightResult
    def apply_preset(self, preset: str, lock: bool = False) -> None
    def from_dataframe(self, df: pd.DataFrame, mapping: Dict[str, str]) -> BuildRequest
    def validate_structure(self, request: BuildRequest) -> ValidationResult
```

### Data Models

#### ParsedERNMessage
Result from parsing a DDEX XML message.

```typescript
interface ParsedERNMessage {
    // Dual representation access
    graph: ERNMessage;                   // Faithful DDEX structure
    flat: FlattenedMessage;              // Developer-friendly view
    
    // Convenience accessors
    releases: ParsedRelease[];
    resources: Map<string, ParsedResource>;
    deals: ParsedDeal[];
    parties: Map<string, Party>;
    
    // Round-trip conversion
    toBuildRequest(): BuildRequest;
}
```

#### BuildRequest
Input structure for generating DDEX XML.

```typescript
interface BuildRequest {
    // Message metadata
    header?: Partial<MessageHeader>;
    version: '3.8.2' | '4.2' | '4.3';
    profile?: ERNProfile;
    
    // Core content
    releases: Partial<ReleaseRequest>[];
    deals?: Partial<DealRequest>[];
    parties?: Partial<PartyRequest>[];
    
    // Extensions
    extensions?: Map<string, XmlFragment>;
}
```

#### BuildResult
Output from DDEX XML generation.

```typescript
interface BuildResult {
    xml: string;                         // Generated XML
    warnings: BuildWarning[];            // Non-fatal issues
    errors: BuildError[];                // Fatal errors (if any)
    statistics: BuildStatistics;         // Generation metrics
    canonicalHash?: string;              // DB-C14N hash
    reproducibilityBanner?: string;      // Determinism info
}
```

---

## 🔧 Configuration & Options

### Parser Options
```typescript
interface ParseOptions {
    // Mode selection
    mode: 'auto' | 'dom' | 'stream';
    autoThreshold: number;                // Switch to streaming at this size
    
    // Data options  
    representation: 'both' | 'graph' | 'flat';
    resolve: boolean;                     // Resolve references
    includeRawExtensions?: boolean;       // Preserve unknown elements
    includeComments?: boolean;            // Include XML comments
    preserveUnknownElements?: boolean;    // Round-trip fidelity
    
    // Performance
    maxMemory: number;                    // Memory limit (bytes)
    timeout: number;                      // Parse timeout (ms)
    
    // Security
    allowExternalEntities: boolean;       // XXE protection
    maxEntityExpansions: number;          // Billion laughs protection
    maxElementDepth: number;              // Deep nesting protection
}
```

### Builder Options
```typescript
interface BuildOptions {
    // Determinism controls
    determinism?: DeterminismConfig;
    
    // Validation
    preflightLevel?: 'strict' | 'warn' | 'none';
    validateReferences?: boolean;         // Check reference integrity
    requireMinimumFields?: boolean;       // Enforce required fields
    
    // Performance
    streamingThreshold?: number;          // Switch to streaming
    maxMemory?: number;                   // Memory limit
    
    // ID Generation
    idStrategy?: 'uuid' | 'uuidv7' | 'sequential' | 'stable-hash';
    stableHashConfig?: StableHashConfig;
    
    // Partner presets
    partnerPreset?: string;               // Platform-specific config
}
```

### Determinism Configuration
```typescript
interface DeterminismConfig {
    // Canonicalization mode
    canonMode: 'db-c14n' | 'pretty' | 'compact';
    
    // Element ordering  
    sortStrategy: 'canonical' | 'input-order' | 'custom';
    customSortOrder?: Record<string, string[]>;
    
    // Formatting
    outputMode: 'db-c14n' | 'pretty' | 'compact';
    lineEnding: 'LF' | 'CRLF';
    indentChar: 'space' | 'tab';
    indentWidth: number;
    
    // Text normalization
    unicodeNormalization: 'NFC' | 'NFD' | 'NFKC' | 'NFKD';
    xmlCharacterPolicy: 'escape' | 'cdata' | 'reject';
    
    // Reproducibility
    emitReproducibilityBanner?: boolean;
    verifyDeterminism?: number;           // Test iterations
}
```

---

## 📊 Performance Characteristics

### Parser Performance (±20% variance)

| File Size | Parse Time | Memory Usage | Mode | Notes |
|-----------|------------|--------------|------|-------|
| 10KB      | <5ms       | <2MB         | DOM  | Single release |
| 100KB     | <10ms      | <5MB         | DOM  | Small catalog |
| 1MB       | <50ms      | <20MB        | DOM  | Medium catalog |
| 10MB      | <500ms     | <100MB       | Auto | Threshold for streaming |
| 100MB     | <5s        | <50MB        | Stream | Memory bounded |
| 1GB       | <60s       | <100MB       | Stream | CPU/cache sensitive |

### Builder Performance

| Mode | Releases | Tracks | Generation Time | Memory | Notes |
|------|----------|--------|----------------|---------|-------|
| **DB-C14N + Stable Hash** | | | | | |
| | 1 | 12 | <15ms | <3MB | Heavy normalization |
| | 100 | 1,200 | <150ms | <20MB | With hashing |
| | 1,000 | 12,000 | <1.5s | <120MB | With sorting |
| **DB-C14N + UUID** | | | | | |
| | 1 | 12 | <10ms | <2MB | Faster IDs |
| | 100 | 1,200 | <100ms | <15MB | No cache needed |
| | 1,000 | 12,000 | <1s | <100MB | Standard |
| **Pretty/Non-canonical** | | | | | |
| | 1 | 12 | <8ms | <2MB | No sorting |
| | 100 | 1,200 | <80ms | <12MB | Fastest |

### Package Sizes

| Component | Size | Target | Status |
|-----------|------|--------|--------|
| Node.js (npm) | 347KB | <1MB | ✅ Excellent |
| Python wheel | 235KB | <1MB | ✅ Compact |
| WASM bundle | 114KB | <500KB | ✅ 77% under target! |

---

## 🚨 Error Handling

### Structured Error Types

All APIs use consistent, structured error reporting (RFC 7807 style):

```typescript
interface BuildError {
    type: string;                        // URI reference
    title: string;                       // Human-readable summary
    detail: string;                      // Detailed explanation
    instance: string;                    // Error location path
    code: 'MISSING_REQUIRED' | 'INVALID_FORMAT' | 'BAD_REF' | 
          'CYCLE_DETECTED' | 'NAMESPACE_LOCK_VIOLATION';
    severity: 'error' | 'warning';
    hint?: string;                       // Suggested fix
    documentationUrl?: string;           // Error-specific docs
    validValue?: any;                    // Example of valid input
}
```

### Common Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| `MISSING_REQUIRED` | Required field not provided | Add the missing field |
| `INVALID_FORMAT` | Field format invalid (e.g., ISRC) | Correct the format |
| `UNKNOWN_FIELD` | Field not in schema | Check for typos |
| `BAD_REF` | Reference to non-existent resource | Verify reference exists |
| `CYCLE_DETECTED` | Circular reference detected | Break the cycle |

### Error Handling Patterns

**Node.js/TypeScript:**
```typescript
try {
    const result = await builder.build(request);
    if (result.warnings.length > 0) {
        console.warn('Build warnings:', result.warnings);
    }
} catch (error) {
    if (error.code === 'MISSING_REQUIRED') {
        console.error(`Missing field: ${error.detail}`);
        console.log(`Hint: ${error.hint}`);
    } else {
        console.error('Unexpected error:', error.message);
    }
}
```

**Python:**
```python
try:
    result = builder.build(request)
    if result.warnings:
        print(f"Build warnings: {len(result.warnings)}")
except ValidationError as e:
    print(f"Validation failed: {e.detail}")
    if e.hint:
        print(f"Hint: {e.hint}")
except BuildError as e:
    print(f"Build failed: {e.message}")
```

---

## 🎭 Data Model Architecture

DDEX Suite provides **dual model architecture** with full round-trip fidelity:

### Graph Model (Faithful DDEX Structure)
Preserves the exact DDEX structure with references and extensions:

```typescript
interface ERNMessage {
    messageHeader: MessageHeader;
    parties: Party[];               // All parties with IDs
    resources: Resource[];          // Audio, video, image resources  
    releases: Release[];            // Release metadata with references
    deals: Deal[];                  // Commercial terms
    extensions?: Map<string, XmlFragment>;  // Round-trip preservation
    toBuildRequest(): BuildRequest; // Convert for building
}
```

### Flattened Model (Developer-Friendly)
Denormalized and resolved for easy consumption:

```typescript
interface ParsedRelease {
    releaseId: string;
    title: string;
    displayArtist: string;
    tracks: ParsedTrack[];         // Fully resolved
    coverArt?: ParsedImage;        // Merged from resources
    _graph?: Release;              // Reference to original
    extensions?: Map<string, XmlFragment>; // Preserved
}
```

Both models support:
- ✅ **Full Round-Trip Fidelity** - Parse → Modify → Build with 100% data preservation
- ✅ **Extension Preservation** - Unknown elements maintained for compliance
- ✅ **Reference Resolution** - Automatic linking between entities  
- ✅ **Type Safety** - Strongly typed interfaces in all languages

---

## 🔄 Workflow Examples

### Parse → Modify → Build Workflow

**Complete Round-Trip Example:**

```typescript
import { DDEXParser } from 'ddex-parser';
import { DDEXBuilder } from 'ddex-builder';

// 1. PARSE existing DDEX message
const parser = new DDEXParser();
const parsed = await parser.parse(originalXml);

// 2. MODIFY the data (simple object manipulation)
const release = parsed.flat.releases[0];
release.title = "Updated Album Title (Deluxe Edition)";
release.tracks.push({
    position: release.tracks.length + 1,
    title: "Bonus Track",
    isrc: "USXYZ2400099",
    duration: 240,
    displayArtist: release.displayArtist
});

// 3. BUILD new XML with deterministic output
const builder = new DDEXBuilder();
await builder.applyPreset('spotify_audio_43');

const result = await builder.build(parsed.toBuildRequest(), {
    determinism: {
        canonMode: 'db-c14n',
        verifyDeterminism: 3  // Verify reproducibility
    }
});

// 4. VERIFY round-trip fidelity
const reparsed = await parser.parse(result.xml);
console.log('✅ Round-trip successful!');
console.log(`Canonical hash: ${result.canonicalHash}`);
```

### Streaming Large Catalogs

**Memory-Efficient Processing:**

```typescript
import { DDEXParser } from 'ddex-parser';
import { StreamingDdexBuilder } from 'ddex-builder';

// Parse large catalog with streaming
const parser = new DDEXParser();
const stream = parser.stream(largeXmlFile, { chunkSize: 100 });

// Build with streaming writer
const builder = new StreamingDdexBuilder();

for await (const release of stream) {
    // Modify each release as needed
    release.title = `Updated: ${release.title}`;
    
    // Add to streaming builder
    await builder.addRelease(release);
}

// Finalize the output
const result = await builder.finalize();
console.log(`Generated ${result.xml.length} bytes`);
```

### Batch Operations

**Process Multiple Releases:**

```python
from ddex_builder import DDEXBuilder, batchBuild
import pandas as pd

# Load catalog from DataFrame
df = pd.read_csv('catalog.csv')
builder = DDEXBuilder()

# Convert DataFrame to build requests
requests = []
for _, row in df.iterrows():
    request = {
        'version': '4.3',
        'releases': [{
            'release_id': row['upc'],
            'title': [{'text': row['album_title']}],
            'display_artist': row['artist_name'],
            'tracks': []  # Add tracks from related data
        }]
    }
    requests.append(request)

# Batch build all releases
results = batchBuild(requests)
print(f"Generated {len(results)} DDEX files")

# Save results
for i, result in enumerate(results):
    filename = f'release_{i+1:04d}.xml'
    with open(filename, 'w') as f:
        f.write(result.xml)
```

---

## 🛡️ Security Features

DDEX Suite includes comprehensive security hardening:

### XML Security Protection
- **XXE Prevention** - External entity processing disabled by default
- **Entity Expansion Limits** - Protection against billion laughs attacks
- **Deep Nesting Protection** - Prevent stack overflow attacks
- **Size Limits** - Configurable file and field size limits
- **Timeout Controls** - Prevent resource exhaustion

### Input Validation
- **ISRC Validation** - Format checking with regex patterns
- **UPC Checksum** - Automatic checksum validation
- **Territory Codes** - ISO 3166 country code validation
- **Duration Format** - ISO 8601 duration validation
- **Character Policy** - Configurable XML character handling

### Supply Chain Security
- **Dependency Auditing** - Regular security scans with `cargo-deny`
- **SBOM Generation** - Software Bill of Materials included
- **Artifact Signing** - Sigstore signatures for releases
- **License Compliance** - Automated license compatibility checking

### Configuration Security
```typescript
const parser = new DDEXParser({
    // Security settings
    allowExternalEntities: false,      // XXE protection
    maxEntityExpansions: 1000,         // Entity expansion limit
    maxElementDepth: 100,              // Nesting depth limit
    maxFileSize: 100 * 1024 * 1024,    // 100MB file size limit
    parseTimeout: 30000,               // 30 second timeout
    
    // Validation
    validateReferences: true,          // Check reference integrity
    requireMinimumFields: true,        // Enforce required fields
});
```

---

## 📋 Platform Support Matrix

| Feature | Node.js | Python | WASM | Rust |
|---------|---------|--------|------|------|
| **Parser** | ✅ v0.1.0 | ✅ v0.1.0 | 🔄 Coming | ✅ Core |
| **Builder** | ✅ v0.1.0 | ✅ v0.1.0 | ✅ v0.1.0 | ✅ Core |
| **Streaming** | ✅ | ✅ | ✅ | ✅ |
| **DataFrame Integration** | ❌ | ✅ | ❌ | ❌ |
| **Type Definitions** | ✅ | ✅ | ✅ | ✅ |
| **Presets** | ✅ | ✅ | ✅ | ✅ |
| **Determinism** | ✅ | ✅ | ✅ | ✅ |
| **Round-Trip** | ✅ | ✅ | ✅ | ✅ |

### Version Compatibility

| DDEX Version | Support Level | Notes |
|--------------|---------------|--------|
| ERN 4.3 | ✅ Full | Latest standard, recommended |
| ERN 4.2 | ✅ Full | Previous standard |
| ERN 3.8.2 | ✅ Legacy | Limited testing |
| ERN 4.1 | ❌ | Not implemented |

---

## 🤝 Contributing to Documentation

We welcome contributions to improve the API documentation:

### Documentation Structure
- **API Reference** - Generated from code comments
- **User Guides** - Hand-written tutorials and examples
- **Error Documentation** - Comprehensive error catalog
- **Performance Guides** - Optimization recommendations

### Contributing Guidelines
1. **Update code comments** for rustdoc generation
2. **Add TypeScript JSDoc** for better IDE support  
3. **Include Python docstrings** for help() output
4. **Provide realistic examples** in all guides
5. **Test all code examples** before submitting

### Documentation Sources
- **Rust**: Generated with `cargo doc --workspace`
- **TypeScript**: Generated with `typedoc`  
- **Python**: Generated with `pydoc` and manual curation
- **WASM**: Hand-written API reference

---

**Last Updated**: September 9, 2025  
**DDEX Suite Version**: v0.1.0  
**Documentation Version**: v1.0.0

*For the most up-to-date API documentation, visit the [GitHub repository](https://github.com/daddykev/ddex-suite) or check the generated docs in your package manager.*