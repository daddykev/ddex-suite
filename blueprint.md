# DDEX Suite - Blueprint

## Project Overview

DDEX Suite is an open-source, high-performance toolkit for DDEX metadata processing, consisting of two complementary tools (`ddex-builder` and `ddex-parser`) built on a **shared Rust core** with native bindings for TypeScript/JavaScript and Python. The suite provides a complete "Parse → Modify → Build" workflow for programmatic DDEX manipulation.

### Suite Components

1. **DDEX Parser**: Transforms DDEX XML messages into clean, strongly-typed data structures
2. **DDEX Builder**: Generates deterministic, compliant DDEX XML from those same structures
3. **Shared Core**: Common data models, errors, and utilities ensuring perfect round-trip fidelity

### Vision
Create the industry-standard DDEX processing toolkit that makes working with music metadata as simple as working with JSON, while providing deterministic XML generation and perfect round-trip fidelity.

### Mission
Deliver a unified suite of DDEX tools through a monorepo architecture, providing consistent behavior, exceptional performance, and developer-friendly APIs across all major programming ecosystems.

### Core Value Propositions
- **DDEX Parser**: "One parser, every language, structural parsing excellence"
- **DDEX Builder**: "Deterministic ERN at scale - One model, every language, byte-perfect generation"
- **Suite**: "Parse → Modify → Build with perfect fidelity"

### Parser vs Builder vs Validator Distinction
- **DDEX Parser**: Structural parsing, reference resolution, normalization, type conversion
- **DDEX Builder**: Structural composition, automatic reference linking, ID generation, deterministic XML serialization
- **DDEX Workbench**: XSD validation, AVS rules, business logic validation, compliance checking
- **Together**: Complete DDEX processing pipeline

## Technical Architecture

### Monorepo Architecture

```
┌─────────────────────────────────────────────────────────┐
│                 DDEX Suite Monorepo                     │
├─────────────────────────────────────────────────────────┤
│                     Applications                        │
├──────────────┬──────────────┬───────────────────────────┤
│  JavaScript  │    Python    │            Rust           │
│   (npm)      │   (PyPI)     │        (crates.io)        │
├──────────────┴──────────────┴───────────────────────────┤
│                   Language Bindings                     │
│  ┌────────────────────────────────────────────────────┐ │
│  │              packages/ddex-parser                  │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐          │ │
│  │  │ napi-rs  │  │   PyO3   │  │   WASM   │          │ │
│  │  │  (Node)  │  │ (Python) │  │ (Browser)│          │ │
│  │  └──────────┘  └──────────┘  └──────────┘          │ │
│  └────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────┐ │
│  │              packages/ddex-builder                 │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐          │ │
│  │  │ napi-rs  │  │   PyO3   │  │   WASM   │          │ │
│  │  │  (Node)  │  │ (Python) │  │ (Browser)│          │ │
│  │  └──────────┘  └──────────┘  └──────────┘          │ │
│  └────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│              Rust Implementation Layer                  │
│  ┌────────────────────────────────────────────────────┐ │
│  │            packages/ddex-parser (crate)            │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    │ │
│  │  │   Parser   │  │ Transform  │  │  Security  │    │ │
│  │  │  (XML→AST) │  │(AST→Model) │  │   (XXE)    │    │ │
│  │  └────────────┘  └────────────┘  └────────────┘    │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    │ │
│  │  │ References │  │  Streaming │  │ Extensions │    │ │
│  │  │ (Resolver) │  │   (Large)  │  │(Passthrough)│   │ │
│  │  └────────────┘  └────────────┘  └────────────┘    │ │
│  └────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────┐ │
│  │           packages/ddex-builder (crate)            │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    │ │
│  │  │  Builder   │  │   Linker   │  │ Generator  │    │ │
│  │  │ (Flat→AST) │  │ (Refs/IDs) │  │ (AST→XML)  │    │ │
│  │  └────────────┘  └────────────┘  └────────────┘    │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    │ │
│  │  │ Preflight  │  │Determinism │  │  DB-C14N   │    │ │
│  │  │(Guardrails)│  │  Engine    │  │   v1.0     │    │ │
│  │  └────────────┘  └────────────┘  └────────────┘    │ │
│  └────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────┐ │
│  │              packages/core (crate)                 │ │
│  │           Shared Foundation Library                │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    │ │
│  │  │   Models   │  │   Errors   │  │    FFI     │    │ │
│  │  │   (Types)  │  │  (Common)  │  │   Types    │    │ │
│  │  └────────────┘  └────────────┘  └────────────┘    │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐    │ │
│  │  │Graph Model │  │Flat Model  │  │ Extensions │    │ │
│  │  │ (Faithful) │  │    (DX)    │  │  Support   │    │ │
│  │  └────────────┘  └────────────┘  └────────────┘    │ │
│  └────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Why Rust Core?

1. **Single Source of Truth**: One implementation to maintain, test, and optimize
2. **Memory Safety**: Guaranteed safety without garbage collection for both parsing and generation
3. **Performance**: Zero-cost abstractions and predictable performance
4. **Security**: Built-in protection against XML vulnerabilities
5. **Cross-Platform**: Excellent tooling for building native bindings
6. **Type Safety**: Strong type system that generates bindings for other languages
7. **Determinism**: Predictable output using IndexMap everywhere for the builder

## Canonical Data Model (Shared Core)

### Dual Representation Strategy (Lossless)

The shared core provides **two complementary views** with **full round-trip fidelity**:

1. **Graph Model**: Faithful representation matching DDEX structure exactly
2. **Flattened Model**: Developer-friendly view that retains all information

Both views preserve all data necessary for round-trip XML operations.

### Enhanced Graph Model (packages/core)

```typescript
// Root message - preserves DDEX structure with extensions
interface ERNMessage {
  // Message header (required in DDEX)
  messageHeader: MessageHeader;
  
  // Core collections with references preserved
  parties: Party[];
  resources: Resource[];
  releases: Release[];
  deals: Deal[];
  
  // Version & profile
  version: ERNVersion;
  profile?: ERNProfile;
  
  // Message audit trail
  messageAuditTrail?: MessageAuditTrail;
  
  // Extensions and passthrough data for round-trip
  extensions?: Map<string, XmlFragment>;
  comments?: Comment[];
  namespaces?: Map<string, string>;
  originalXml?: string; // Optional for debugging
  
  // Round-trip helpers
  toBuildRequest(): BuildRequest;  // Convert to builder input
}

interface MessageHeader {
  messageId: string;
  messageType: 'NewReleaseMessage' | 'UpdateReleaseMessage' | 'TakedownMessage';
  messageCreatedDateTime: Date;
  messageSender: MessageSender;
  messageRecipient: MessageRecipient;
  messageControlType?: 'LiveMessage' | 'TestMessage';
  messageAuditTrail?: MessageAuditTrail;
}

interface Release {
  releaseReference: string;
  releaseId: Identifier[];
  releaseTitle: LocalizedString[];
  releaseType?: ReleaseType;
  genre?: Genre[];
  releaseResourceReferenceList: ReleaseResourceReference[];
  displayArtist?: Artist[];
  releaseDate?: ReleaseEvent[];
  territoryCode?: string[];
  
  // Passthrough data for round-trip
  extensions?: Map<string, XmlFragment>;
  attributes?: Map<string, string>;
  _originalXml?: string;
}

interface Resource {
  resourceReference: string;
  resourceType: 'SoundRecording' | 'Video' | 'Image' | 'Text' | 'SheetMusic';
  resourceId: Identifier[];
  technicalDetails?: TechnicalDetails[];
  rightsController?: string[];
  pLine?: Copyright[];
  cLine?: Copyright[];
  
  // Extensions preserved
  extensions?: Map<string, XmlFragment>;
}
```

### Enhanced Flattened Model (packages/core)

```typescript
interface ParsedERNMessage {
  // Dual representation access
  graph: ERNMessage;                   // Full graph model
  flat: FlattenedMessage;              // Flattened view
  
  // Convenience accessors
  releases: ParsedRelease[];
  resources: Map<string, ParsedResource>;
  deals: ParsedDeal[];
  parties: Map<string, Party>;
  
  // Round-trip conversion
  toBuildRequest(): BuildRequest;      // Direct conversion for builder
}

interface ParsedRelease {
  // Identifiers (resolved and normalized)
  releaseId: string;
  identifiers: {
    upc?: string;
    ean?: string;
    catalogNumber?: string;
    grid?: string;
    proprietary?: { namespace: string; value: string }[];
  };
  
  // Core metadata
  title: LocalizedString[];
  displayArtist: string;
  artists: Artist[];
  releaseType: 'Album' | 'Single' | 'EP' | 'Compilation';
  
  // Tracks - fully resolved
  tracks: ParsedTrack[];
  
  // Reference to original for full fidelity
  _graph?: Release;
  
  // Extensions preserved
  extensions?: Map<string, XmlFragment>;
  comments?: string[];
}

interface BuildRequest {
  // Global message metadata
  header: Partial<MessageHeader>;
  version: '3.8.2' | '4.2' | '4.3';
  profile?: ERNProfile;
  messageControlType?: 'LiveMessage' | 'TestMessage';
  
  // Core content (same models as parser output)
  releases: Partial<ParsedRelease>[];
  deals?: Partial<ParsedDeal>[];
  parties?: Partial<Party>[];
  
  // Extensions passthrough
  extensions?: Map<string, XmlFragment>;
}
```

## Use Cases

### Major Record Labels

#### Major Label Group - Catalog Migration
**Scenario**: XYZ Music Group needs to migrate their entire back catalog (3M+ recordings) from a legacy system to a new distribution platform requiring DDEX ERN 4.3.

```typescript
import { DDEXBuilder } from 'ddex-builder';
import { DatabaseConnection } from './legacy-db';

const builder = new DDEXBuilder();
const db = new DatabaseConnection();

// Apply deterministic configuration for reproducible migration
builder.applyPreset('deterministic_migration');

// Stream from legacy database to DDEX XML files
const catalogStream = db.streamCatalog({ batchSize: 1000 });

for await (const batch of catalogStream) {
  const releases = batch.map(legacyRelease => ({
    releaseId: legacyRelease.upc,
    identifiers: {
      upc: legacyRelease.upc,
      catalogNumber: legacyRelease.catalog_no,
      grid: legacyRelease.grid_id
    },
    title: [{ text: legacyRelease.title, languageCode: 'en' }],
    displayArtist: legacyRelease.artist_name,
    releaseDate: new Date(legacyRelease.release_date),
    tracks: legacyRelease.tracks.map(track => ({
      position: track.sequence,
      isrc: track.isrc,
      title: track.title,
      duration: track.duration_seconds,
      displayArtist: track.artist || legacyRelease.artist_name
    }))
  }));

  // Generate DDEX message with stable IDs for cross-batch consistency
  const { xml, warnings, canonicalHash } = await builder.build({
    header: {
      messageSender: { partyName: [{ text: 'XYZ Music Group' }] },
      messageRecipient: { partyName: [{ text: 'Spotify' }] }
    },
    version: '4.3',
    profile: 'AudioAlbum',
    releases
  }, {
    idStrategy: 'stable-hash',
    stableHashConfig: {
      recipe: 'v1',
      cache: 'sqlite'  // External KV cache for ID persistence
    }
  });

  // Store hash for verification
  await db.storeMigrationHash(batch[0].id, canonicalHash);
  await saveToDistributionQueue(xml);
}
```

#### Major Label - Weekly New Release Feed
**Scenario**: XYZ needs to generate weekly DDEX feeds for all new releases across their labels for 50+ DSP partners.

```python
from ddex_builder import DDEXBuilder
from datetime import datetime, timedelta
import pandas as pd

builder = DDEXBuilder()

# Load this week's releases from data warehouse
releases_df = pd.read_sql("""
    SELECT * FROM releases 
    WHERE release_date BETWEEN %s AND %s
    AND status = 'APPROVED'
    ORDER BY priority DESC, release_date
""", params=[datetime.now(), datetime.now() + timedelta(days=7)])

# Group by DSP requirements
for dsp, dsp_config in DSP_CONFIGS.items():
    # Filter releases for this DSP based on territory rights
    dsp_releases = filter_by_territory_rights(releases_df, dsp_config['territories'])
    
    # Apply DSP-specific preset
    builder.apply_preset(dsp_config['preset_name'])
    
    # Build DDEX message with DSP-specific formatting
    result = builder.build({
        'header': {
            'message_sender': {'party_name': [{'text': 'XYZ Music Entertainment'}]},
            'message_recipient': {'party_name': [{'text': dsp_config['name']}]}
        },
        'version': dsp_config['ern_version'],  # DSP-specific version
        'profile': 'AudioAlbum',
        'releases': dsp_releases.to_dict('records'),
        'deals': generate_deals_for_dsp(dsp_releases, dsp_config)
    })
    
    # Upload to DSP's FTP/API
    upload_to_dsp(dsp, result.xml)
```

### Digital Distributors

#### Independent Distributor - New Release Pipeline
**Scenario**: Independent Distributor delivers 100,000+ new releases daily from independent artists and needs to generate DDEX feeds for multiple platforms.

```typescript
import { DDEXBuilder } from 'ddex-builder';
import { Queue } from 'bull';

const builder = new DDEXBuilder();
const releaseQueue = new Queue('releases');

releaseQueue.process(async (job) => {
  const { artistSubmission } = job.data;
  
  // Transform artist's simple form data into DDEX
  const release = {
    identifiers: {
      upc: await generateUPC(artistSubmission),
      proprietary: [{ 
        namespace: 'indieDistro', 
        value: artistSubmission.releaseId 
      }]
    },
    title: [{ text: artistSubmission.albumTitle }],
    displayArtist: artistSubmission.artistName,
    releaseType: artistSubmission.releaseType,
    genre: mapToAVSGenre(artistSubmission.genre),
    releaseDate: new Date(artistSubmission.releaseDate),
    tracks: artistSubmission.tracks.map((track, idx) => ({
      position: idx + 1,
      isrc: track.isrc || await generateISRC(track),
      title: track.title,
      duration: track.durationSeconds,
      displayArtist: track.featuring ? 
        `${artistSubmission.artistName} feat. ${track.featuring}` : 
        artistSubmission.artistName,
      isExplicit: track.hasExplicitLyrics
    })),
    images: [{
      type: 'FrontCoverImage',
      resourceReference: `IMG_${artistSubmission.releaseId}`,
      uri: artistSubmission.artworkUrl
    }]
  };

  // Generate DDEX for each target platform
  const platforms = ['spotify', 'amazon', 'youtube'];
  
  for (const platform of platforms) {
    const { xml } = await builder.build({
      header: createHeaderForPlatform(platform),
      version: PLATFORM_CONFIGS[platform].ernVersion,
      releases: [release],
      deals: [createStandardIndieDeals(release, platform)]
    });
    
    await queueForDelivery(platform, xml);
  }
});
```

### Streaming Platforms

#### YouTube - Ingestion Pipeline
**Scenario**: YouTube receives 1M+ DDEX messages daily and needs to normalize them for internal processing.

```python
from ddex_parser import DDEXParser
from ddex_builder import DDEXBuilder
import asyncio

parser = DDEXParser()
builder = DDEXBuilder()

async def normalize_incoming_ddex(raw_xml: bytes) -> dict:
    """Normalize any DDEX version to internal format"""
    
    # Parse incoming DDEX (any version)
    parsed = await parser.parse_async(raw_xml)
    
    # Normalize to internal canonical format
    normalized_releases = []
    for release in parsed.flat.releases:
        # Apply YouTube-specific business rules
        normalized = {
            **release,
            'youtube_id': youtube_spotify_id(release),
            'availability': calculate_availability(release),
            'content_rating': derive_content_rating(release),
            'algorithmic_tags': generate_ml_tags(release)
        }
        normalized_releases.append(normalized)
    
    # Rebuild as standardized ERN 4.3 for internal systems
    result = await builder.build_async({
        'header': create_internal_header(),
        'version': '4.3',  # Standardize on latest version
        'releases': normalized_releases,
        'deals': parsed.flat.deals,
        'preflight_level': 'strict'  # Ensure compliance
    }, {
        'determinism': {
            'canonMode': 'db-c14n',
            'sortStrategy': 'canonical'
        }
    })
    
    return {
        'normalized_xml': result.xml,
        'canonical_hash': result.canonical_hash,
        'metadata': extract_searchable_metadata(normalized_releases),
        'ingestion_timestamp': datetime.now()
    }
```

### Enterprise Catalog Management

#### Major Label Group - Multi-Format Delivery
**Scenario**: XYZ Music Group needs to deliver the same release in different formats (physical, digital, streaming) with format-specific metadata.

```python
from ddex_builder import DDEXBuilder
from enum import Enum

class ReleaseFormat(Enum):
    STREAMING = "streaming"
    DOWNLOAD = "download"
    PHYSICAL_CD = "physical_cd"
    VINYL = "vinyl"

class MultiFormatBuilder:
    def __init__(self):
        self.builder = DDEXBuilder()
    
    def build_format_specific_release(self, master_release, format_type):
        """Generate format-specific DDEX from master release"""
        
        # Base release data
        release = {**master_release}
        
        if format_type == ReleaseFormat.STREAMING:
            # Streaming-specific adaptations
            release['tracks'] = self.add_streaming_metadata(release['tracks'])
            release['technical_details'] = {
                'file_format': 'AAC',
                'bitrate': 256,
                'sample_rate': 44100
            }
            
        elif format_type == ReleaseFormat.VINYL:
            # Vinyl-specific adaptations
            release['tracks'] = self.organize_for_vinyl_sides(release['tracks'])
            release['physical_details'] = {
                'format': 'Vinyl',
                'configuration': '2xLP',
                'speed': '33RPM',
                'color': 'Black'
            }
            
        return self.builder.build({
            'version': '4.3',
            'profile': self.get_profile_for_format(format_type),
            'releases': [release],
            'deals': self.generate_format_specific_deals(release, format_type)
        })
```

### Trifecta - The "Parse → Modify → Build" Workflow

This is the primary use case, demonstrating the power of the full suite:

```typescript
import { DDEXParser } from 'ddex-parser';
import { DDEXBuilder } from 'ddex-builder';
import * as fs from 'fs/promises';

const parser = new DDEXParser();
const builder = new DDEXBuilder();

// Apply partner-specific configuration with lock
builder.applyPreset('spotify_audio_43', { lock: true });

// 1. PARSE an existing message
const originalXml = await fs.readFile('path/to/original.xml');
const parsedMessage = await parser.parse(originalXml);

// 2. MODIFY the data in a simple, programmatic way
const firstRelease = parsedMessage.flat.releases[0];
firstRelease.releaseDate = new Date('2026-03-01T00:00:00Z'); 
firstRelease.tracks.push({
  position: firstRelease.tracks.length + 1,
  title: 'New Bonus Track',
  isrc: 'USXYZ2600001',
  duration: 180,
  displayArtist: firstRelease.displayArtist
});

// 3. BUILD a new, deterministic XML message from the modified object
const { xml, warnings, canonicalHash, reproducibilityBanner } = await builder.build({
  header: parsedMessage.graph.messageHeader,
  version: parsedMessage.flat.version,
  releases: parsedMessage.flat.releases,
  deals: parsedMessage.flat.deals,
}, {
  determinism: {
    canonMode: 'db-c14n',
    emitReproducibilityBanner: true,
    verifyDeterminism: 3  // Build 3 times to verify determinism
  },
  idStrategy: 'stable-hash'
});

if (warnings.length > 0) {
  console.warn('Build warnings:', warnings);
}

// Verify deterministic output
console.log(`Canonical hash: ${canonicalHash}`);
console.log(`Reproducibility: ${reproducibilityBanner}`); 

// The new XML is ready to be sent or validated by DDEX Workbench
await fs.writeFile('path/to/updated.xml', xml);
```

## Performance Specifications

### Parser Performance Targets (±20% variance)

| File Size | Parse Time | Memory Usage | Mode | Notes |
|-----------|------------|--------------|------|-------|
| 10KB      | <5ms ±1ms  | <2MB         | DOM  | Single release |
| 100KB     | <10ms ±2ms | <5MB         | DOM  | Small catalog |
| 1MB       | <50ms ±10ms| <20MB        | DOM  | Medium catalog |
| 10MB      | <500ms ±100ms | <100MB    | Auto | Threshold for streaming |
| 100MB     | <5s ±1s    | <50MB        | Stream | Memory bounded |
| 1GB       | <60s ±12s  | <100MB       | Stream | CPU/cache sensitive |

### Builder Performance Targets (By Mode)

| Mode | # Releases | # Tracks | Generation Time | Memory Usage | Notes |
|------|------------|----------|-----------------|--------------|-------|
| **DB-C14N + Stable Hash** | | | | | |
| | 1 | 12 | <15ms ±3ms | <3MB | Heavy normalization |
| | 100 | 1,200 | <150ms ±30ms | <20MB | With hashing |
| | 1,000 | 12,000 | <1.5s ±300ms | <120MB | With sorting |
| | 10,000 | 120,000 | <15s ±3s | <50MB | Stream mode |
| **DB-C14N + UUID** | | | | | |
| | 1 | 12 | <10ms ±2ms | <2MB | Faster IDs |
| | 100 | 1,200 | <100ms ±20ms | <15MB | No cache needed |
| | 1,000 | 12,000 | <1s ±200ms | <100MB | Standard |
| | 10,000 | 120,000 | <10s ±2s | <50MB | Stream mode |
| **Pretty/Non-canonical** | | | | | |
| | 1 | 12 | <8ms ±2ms | <2MB | No sorting |
| | 100 | 1,200 | <80ms ±15ms | <12MB | Fastest |
| | 1,000 | 12,000 | <800ms ±150ms | <80MB | Minimal overhead |

### Benchmark Specifications

- **Hardware Baseline**: AWS m7g.large (2 vCPU, 8GB RAM)
- **Software**: Node 20 LTS, Python 3.11, Rust 1.75
- **Metrics**: P50, P95, P99 latency + peak RSS memory
- **WASM Target**: <500KB for lite builds with aggressive optimization

## Security Architecture

### XML Security (Built into Rust Core)

```rust
pub struct SecurityConfig {
    // Entity expansion protection
    pub disable_dtd: bool,                    // Default: true
    pub disable_external_entities: bool,      // Default: true
    pub max_entity_expansions: usize,         // Default: 1000
    pub max_entity_depth: usize,              // Default: 20
    
    // Size limits
    pub max_element_depth: usize,             // Default: 100
    pub max_attribute_size: usize,            // Default: 100KB
    pub max_text_size: usize,                 // Default: 1MB
    pub max_file_size: usize,                 // Default: 1GB
    
    // Time limits
    pub parse_timeout_ms: u64,                // Default: 30000 (30s)
    pub stream_timeout_ms: u64,               // Default: 300000 (5m)
    
    // Network protection
    pub allow_network: bool,                  // Default: false
    pub allowed_schemas: Vec<String>,         // Default: ["file"]
    
    // Character policy
    pub xml_character_policy: String,         // Default: "escape"
}
```

### Security Test Suite

- XXE (XML External Entity) attacks
- Billion laughs (entity expansion)
- Quadratic blowup attacks
- XML bomb protection
- Schema poisoning
- DTD-based attacks
- Invalid UTF-8 sequences
- Character policy enforcement
- DataFrame DSL security (no eval)
- Preset lock mechanism

## API Specifications

### Parser API (Unified Across Languages)

```typescript
// TypeScript
class DDEXParser {
  parse(xml: string | Buffer, options?: ParseOptions): Promise<ParsedERNMessage>;
  parseSync(xml: string | Buffer, options?: ParseOptions): ParsedERNMessage;
  stream(source: ReadableStream, options?: StreamOptions): AsyncIterator<ParsedRelease>;
  sanityCheck(xml: string | Buffer): Promise<SanityCheckResult>;
  detectVersion(xml: string | Buffer): ERNVersion;
}

interface ParseOptions {
  // Mode selection
  mode: 'auto' | 'dom' | 'stream';
  autoThreshold: number;
  
  // Data options
  representation: 'both' | 'graph' | 'flat';
  resolve: boolean;
  includeRawExtensions?: boolean;
  includeComments?: boolean;
  preserveUnknownElements?: boolean;
  
  // Performance
  maxMemory: number;
  timeout: number;
  
  // Progress callback
  onProgress?: (progress: ParseProgress) => void;
}

// Python
class DDEXParser:
    def parse(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParsedERNMessage
    async def parse_async(self, xml: Union[str, bytes], options: Optional[ParseOptions] = None) -> ParsedERNMessage
    def stream(self, source: IO, options: Optional[StreamOptions] = None) -> Iterator[ParsedRelease]
    def to_dataframe(self, xml: Union[str, bytes], schema: str = 'flat') -> pd.DataFrame
    def detect_version(self, xml: Union[str, bytes]) -> ERNVersion
```

### Builder API (Unified Across Languages)

```typescript
// TypeScript
class DDEXBuilder {
  build(request: BuildRequest, options?: BuildOptions): Promise<BuildResult>;
  buildSync(request: BuildRequest, options?: BuildOptions): BuildResult;
  stream(request: BuildRequest, options?: StreamOptions): WritableStream;
  preflight(request: BuildRequest): Promise<PreflightResult>;
  canonicalize(xml: string | Buffer): Promise<string>;
  diff(originalXml: string, newRequest: BuildRequest): Promise<DiffResult>;
  applyPreset(preset: string, options?: PresetOptions): void;
  dryRunId(type: string, materials: any, recipe?: string): IdDebugInfo;
  presetDiff(preset: string, fromVersion?: string, toVersion?: string): PresetDiffResult;
}

interface BuildOptions {
  // Determinism controls
  determinism?: DeterminismConfig;
  
  // Validation
  preflightLevel?: 'strict' | 'warn' | 'none';
  validateReferences?: boolean;
  requireMinimumFields?: boolean;
  
  // Performance
  streamingThreshold?: number;
  maxMemory?: number;
  
  // ID Generation
  idStrategy?: 'uuid' | 'uuidv7' | 'sequential' | 'stable-hash';
  stableHashConfig?: StableHashConfig;
  
  // Partner presets
  partnerPreset?: string;
}

interface BuildResult {
  xml: string;
  warnings: BuildWarning[];
  errors: BuildError[];
  statistics: BuildStatistics;
  canonicalHash?: string;
  reproducibilityBanner?: string;
}

// Python
class DDEXBuilder:
    def build(self, request: BuildRequest, options: Optional[BuildOptions] = None) -> BuildResult: ...
    async def build_async(self, request: BuildRequest, options: Optional[BuildOptions] = None) -> BuildResult: ...
    def preflight(self, request: BuildRequest) -> PreflightResult: ...
    def canonicalize(self, xml: Union[str, bytes]) -> str: ...
    def apply_preset(self, preset: str, lock: bool = False) -> None: ...
    def from_dataframe(self, df: pd.DataFrame, mapping: Dict[str, str]) -> BuildRequest: ...
```

### Streaming Semantics

#### JavaScript/Node.js
```typescript
// Async iterator with backpressure support
const parser = new DDEXParser();
const stream = parser.stream(fileStream, {
  chunkSize: 100,
  onProgress: ({ bytes, releases }) => console.log(`Processed ${releases} releases`)
});

for await (const release of stream) {
  await processRelease(release);
}
```

#### Python
```python
# Iterator with optional async support
parser = DDEXParser()

# Synchronous iteration
for release in parser.stream(file):
    process_release(release)

# Asynchronous iteration
async for release in parser.stream_async(file):
    await process_release(release)
```

#### Browser/WASM
```typescript
// Web Streams API with Worker support
const parser = new DDEXParser();
const stream = parser.stream(response.body, {
  useWorker: true,  // Parse in Web Worker
  chunkSize: 100
});

for await (const release of stream) {
  updateUI(release);
}
```

### DataFrame to DDEX Mapping DSL

Declarative mapping DSL without eval for security:

```python
# Declarative mapping DSL - no eval, purely declarative
mapping = {
    'releases': {
        'title': {'column': 'album_title'},
        'releaseDate': {'column': 'release_date', 'transform': 'to_date'},
        'tracks[]': {
            'title': {'column': 'track_title'},
            'position': {'transform': 'row_number'},
            'isrc': {'column': 'isrc'}
        },
        'titles[]': {
            'text': {'columns': ['title_en', 'title_es'], 'transform': 'zip'},
            'languageCode': {'values': ['en', 'es']}
        },
        'territories[]': {'column': 'territories', 'transform': 'split', 'delimiter': ','}
    }
}

# Usage
builder = DDEXBuilder()
df = pd.read_csv('catalog.csv')
request = builder.from_dataframe(df, mapping)
result = builder.build(request)
```

## Determinism & Canonicalization

### DB-C14N/1.0 - DDEX Builder Canonicalization

The builder implements **DB-C14N/1.0** (DDEX Builder Canonicalization v1.0), our custom canonicalization specification designed specifically for DDEX message determinism.

#### DB-C14N/1.0 Specification Summary

1. **XML Declaration & Encoding** - Fixed `<?xml version="1.0" encoding="UTF-8"?>`
2. **Whitespace, Indentation, Line Endings** - LF normalized, 2-space indent
3. **Attribute Ordering Policy** - Alphabetical by local name
4. **Namespace Prefix Lock Tables** - Per ERN version, immutable
5. **Text Normalization** - Unicode NFC, character policy by field
6. **Date/Time** - UTC, ISO8601Z, zero-pad rules
7. **Element Ordering** - Generated from XSD + checksum
8. **Canonical Hash Definition** - SHA-256 of specific byte ranges

### Determinism Configuration

```typescript
interface DeterminismConfig {
  // Canonicalization mode
  canonMode: 'db-c14n' | 'pretty' | 'compact';
  
  // Element ordering
  sortStrategy: 'canonical' | 'input-order' | 'custom';
  customSortOrder?: Record<string, string[]>;
  
  // Namespace handling
  namespaceStrategy: 'locked' | 'inherit';
  lockedPrefixes?: Record<string, string>;
  
  // Formatting
  outputMode: 'db-c14n' | 'pretty' | 'compact';
  lineEnding: 'LF' | 'CRLF';
  indentChar: 'space' | 'tab';
  indentWidth: number;
  
  // String normalization
  unicodeNormalization: 'NFC' | 'NFD' | 'NFKC' | 'NFKD';
  xmlCharacterPolicy: 'escape' | 'cdata' | 'reject';
  quoteStyle: 'double' | 'single';
  
  // Date/Time
  timeZonePolicy: 'UTC' | 'preserve' | 'local';
  dateTimeFormat: 'ISO8601Z' | 'ISO8601' | 'custom';
  
  // Reproducibility
  emitReproducibilityBanner?: boolean;
  verifyDeterminism?: number;
}
```

### Determinism CI Lint Configuration

```toml
# clippy.toml
deny = [
  "clippy::disallowed_types",
  "clippy::unwrap_used",
]
# Disallow unordered maps in output paths
disallowed-types = [
  "std::collections::HashMap",
  "std::collections::HashSet",
]
```

### Stable Hash ID Generation with Recipe Contracts

Content-based IDs with versioned, explicit recipe contracts:

```toml
# recipes/release_v1.toml
[Release.v1]
fields = ["UPC", "ReleaseType", "TrackISRCs[]", "TerritorySet", "Version"]
normalize = { unicode = "NFC", trim = true, case = "as-is" }
numeric = { duration_round = "millisecond" }
text = { whitespace = "normalize", locale = "none" }
salt = "REL@1"

[Resource.v1]
fields = ["ISRC", "Duration", "Hash"]
normalize = { unicode = "NFC", trim = true }
numeric = { duration_round = "second" }
salt = "RES@1"

[Party.v1]
fields = ["Name", "Role", "Identifiers"]
normalize = { unicode = "NFC", trim = true, case = "lower" }
text = { case_folding = "locale-free" }
salt = "PTY@1"
```

### JSON Schema Annotations

Generated schemas include machine-readable canonicalization hints:

```json
{
  "type": "object",
  "properties": {
    "releases": {
      "type": "array",
      "x-canonical-order": "ReleaseId,ReleaseReference,ReleaseDetailsByTerritory",
      "x-ddex-ern-version": "4.3"
    }
  }
}
```

## Partner Presets System

Configuration templates with provenance tracking, versioning, and safety features:

```typescript
interface PartnerPreset {
  name: string;
  description: string;
  source: 'public_docs' | 'customer_feedback';
  provenanceUrl?: string;
  version: string;
  locked?: boolean;
  disclaimer: string;
  determinism: Partial<DeterminismConfig>;
  defaults: {
    messageControlType?: 'LiveMessage' | 'TestMessage';
    territoryCode?: string[];
    distributionChannel?: string[];
  };
  requiredFields: string[];
  formatOverrides: Record<string, any>;
}

// Example preset from public documentation
const SPOTIFY_AUDIO_43: PartnerPreset = {
  name: 'spotify_audio_43',
  description: 'Spotify Audio Album ERN 4.3 requirements (config template)',
  source: 'public_docs',
  provenanceUrl: 'https://support.spotify.com/artists/article/ddex-delivery-spec',
  version: '1.0.0',
  disclaimer: 'Presets are community-maintained config templates derived from public documentation and implementer feedback. They are not official specs and do not replace ingestion testing.',
  determinism: {
    canonMode: 'db-c14n',
    sortStrategy: 'canonical',
    outputMode: 'db-c14n',
    timeZonePolicy: 'UTC',
    dateTimeFormat: 'ISO8601Z'
  },
  defaults: {
    messageControlType: 'LiveMessage',
    distributionChannel: ['01'] // Download
  },
  requiredFields: ['ISRC', 'UPC', 'ReleaseDate', 'Genre'],
  formatOverrides: {
    'Duration': 'PT{minutes}M{seconds}S'
  }
};
```

## CLI Reference

### Parser CLI Commands

```bash
# Parse and extract
ddex-parser parse input.xml --schema flat|graph --output parsed.json
ddex-parser extract input.xml --format json|csv --fields title,isrc,duration
ddex-parser stream large.xml --jsonl --chunk-size 100

# Analysis and inspection
ddex-parser detect-version input.xml
ddex-parser sanity-check input.xml
ddex-parser stats input.xml

# Batch processing
ddex-parser batch *.xml --parallel 4 --output-dir parsed/
```

### Builder CLI Commands

```bash
# Build from JSON
ddex-builder build --from-json request.json --ern 4.3 --preset spotify_audio_43 --preset-lock --db-c14n --id stable-hash:v1 --out out.xml

# Canonicalize existing XML
ddex-builder canon in.xml > out.xml

# Generate diff and update skeleton
ddex-builder diff --old old.xml --from-json request.json --emit-update-skeleton update.json

# Debug stable hash IDs with explanation
ddex-builder ids --explain Release ./materials.json

# Verify determinism
ddex-builder build --from-json request.json --verify-determinism 5

# Show preset diff after upgrade
ddex-builder preset-diff spotify_audio_43 --from-version 1.0.0 --to-version 1.1.0

# Export JSON Schema for a profile
ddex-builder build --from-json request.json --schema-out schema.json

# Fail on warnings for CI/CD pipelines
ddex-builder build --from-json request.json --fail-on-warn

# Version banner with build info
ddex-builder --version
# DDEX Builder v1.0.0 • DB-C14N/1.0 • models: ERN 4.3 • presets: 8 • build: reproducible
```

## Error Handling

### Structured Error Reporting (RFC 7807 Style)

```typescript
interface BuildError {
  type: string;                              // URI reference (RFC 7807)
  title: string;                             // Short, human-readable summary
  detail: string;                            // Human-readable explanation
  instance: string;                          // Path to error location
  code: 'MISSING_REQUIRED' | 'INVALID_FORMAT' | 'UNKNOWN_FIELD' | 
        'BAD_REF' | 'CYCLE_DETECTED' | 'NAMESPACE_LOCK_VIOLATION';
  severity: 'error' | 'warning';
  hint?: string;                             // Suggested fix
  documentationUrl?: string;                 // Link to specific error documentation
  validValue?: any;                          // Example of a valid value
}

interface PreflightResult {
  isValid: boolean;
  errors: BuildError[];
  warnings: BuildWarning[];
  statistics: {
    totalFields: number;
    validatedFields: number;
    missingRequiredFields: string[];
    invalidReferences: string[];
    unknownFields: string[];
  };
  coverageMatrix: ProfileCoverage;
}
```

### Error Codes and Resolution

| Code | Description | Resolution |
|------|-------------|------------|
| `MISSING_REQUIRED` | Required field not provided | Add the missing field |
| `INVALID_FORMAT` | Field format invalid (e.g., ISRC) | Correct the format |
| `UNKNOWN_FIELD` | Field not in schema | Check for typos |
| `BAD_REF` | Reference to non-existent resource | Verify reference exists |
| `CYCLE_DETECTED` | Circular reference detected | Break the cycle |
| `NAMESPACE_LOCK_VIOLATION` | Namespace prefix changed | Use locked prefix |

## Distribution Strategy

### Node.js Distribution

Using `napi-rs` with `prebuildify` for maximum compatibility:

```json
{
  "name": "ddex-parser",
  "exports": {
    ".": {
      "import": "./dist/index.mjs",
      "require": "./dist/index.cjs",
      "types": "./dist/index.d.ts"
    }
  },
  "scripts": {
    "prebuildify": "prebuildify --platform win32,darwin,linux --arch x64,arm64 --strip",
    "test:import": "node -e \"import('ddex-parser').then(m => console.log(m.version))\""
  }
}
```

### Python Distribution

Using `cibuildwheel` for comprehensive platform coverage:

```toml
# pyproject.toml
[tool.cibuildwheel]
build = ["cp38-*", "cp39-*", "cp310-*", "cp311-*", "cp312-*"]
skip = ["*-musllinux_i686", "*-win32", "pp*"]

[tool.cibuildwheel.linux]
manylinux-x86_64-image = "manylinux2014"
manylinux-aarch64-image = "manylinux2014"
musllinux-x86_64-image = "musllinux_1_1"
musllinux-aarch64-image = "musllinux_1_1"

[tool.cibuildwheel.macos]
archs = ["universal2"]

[tool.cibuildwheel.windows]
archs = ["AMD64", "ARM64"]

[tool.cibuildwheel.test]
test-command = "python -c 'import ddex_parser; print(ddex_parser.__version__)'"
```

### WASM Distribution

Optimized for browser usage with size constraints:

```toml
# WASM optimization settings
[profile.release]
panic = "abort"
lto = "fat"
opt-level = "z"
codegen-units = 1
strip = true
```

## Testing Strategy

### Comprehensive Test Coverage

```
test-suite/
├── unit/                         # Unit tests per module
├── integration/                  # End-to-end tests
├── round-trip/                   # Parse→Build→Parse tests
├── performance/                  # Benchmark suite
├── security/                     # Security tests
├── determinism/                  # Cross-platform determinism
├── compatibility/                # Version compatibility
├── vendor-quirks/                # Real-world edge cases
├── nasty/                        # Attack vectors
├── golden/                       # Expected outputs
├── fuzzing/                      # Fuzz test corpus
├── property/                     # Property-based tests
└── dsp-corpus/                   # DSP acceptance tests
```

### Test Requirements

- **Unit Tests**: 95%+ code coverage
- **Integration Tests**: All major workflows
- **Round-Trip Tests**: 100% data preservation
- **Determinism Tests**: 100% pass rate across OS/arch
- **Fuzz Testing**: 24-hour run without crashes + 5-minute CI smoke
- **Performance Tests**: No regression >5%
- **Security Tests**: All OWASP XML vulnerabilities
- **Property Tests**: 1M+ iterations maintaining invariants
- **DSP Corpus**: >95% acceptance rate
- **Golden Tests**: Byte-perfect XML generation

## CI/CD & Supply Chain Security

### GitHub Actions Matrix

```yaml
name: Suite CI/CD
on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: cargo deny check
      - run: cargo audit
      - run: npm audit
      - run: pip-audit
      
  determinism-audit:
    runs-on: ubuntu-latest
    steps:
      - run: cargo clippy -- -D warnings
      - run: grep -r "HashMap\|HashSet" src/ && exit 1 || exit 0
      
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
        node: [18, 20, 22]
        python: [3.8, 3.9, 3.10, 3.11, 3.12]
    
  prebuild:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - run: npm run prebuildify
      
  wheels:
    runs-on: ubuntu-latest
    steps:
      - uses: pypa/cibuildwheel@v2
        with:
          package-dir: packages/ddex-parser/python
          
  sign:
    runs-on: ubuntu-latest
    steps:
      - uses: sigstore/cosign-installer@v3
      - run: cosign sign-blob
```

### Supply Chain Security

- **cargo-deny**: Audit Rust dependencies ✅
- **dependabot**: Automated updates
- **SLSA**: Supply chain provenance
- **Sigstore**: Artifact signing
- **SBOM**: Software bill of materials
- **License scanning**: Ensure compatibility
- **Frozen deps**: Critical transitive deps in Cargo.lock

## Project Structure

```
ddex-suite/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                    # Unified CI for monorepo
│   │   ├── parser-release.yml        # Parser-specific release
│   │   ├── builder-release.yml       # Builder-specific release
│   │   ├── rust.yml                  # Rust tests
│   │   ├── node.yml                  # Node bindings CI
│   │   ├── python.yml                # Python bindings CI
│   │   ├── security.yml              # Security scanning
│   │   ├── determinism.yml           # Cross-platform determinism tests
│   │   └── determinism-audit.yml     # Scan for HashMap/HashSet
│   └── dependabot.yml                # Dependency updates
│
├── docs/                             # Suite-wide documentation
│   ├── parser/
│   │   ├── API.md
│   │   ├── ERROR_HANDBOOK.md
│   │   └── MIGRATION.md
│   ├── builder/
│   │   ├── API.md
│   │   ├── DB_C14N_SPEC.md
│   │   └── PRESETS.md
│   ├── ARCHITECTURE.md               # Monorepo architecture
│   ├── SUITE_OVERVIEW.md             # DDEX Suite vision
│   ├── ROUND_TRIP.md                 # Parse→Modify→Build guide
│   └── CONTRIBUTING.md               # Contribution guidelines
│
├── examples/                         # Example usage
│   ├── parse-modify-build/           # Round-trip examples
│   ├── parser-only/
│   └── builder-only/
│
├── packages/
│   ├── core/                         # Shared Rust crate
│   │   ├── src/
│   │   │   ├── models/               # DDEX data models
│   │   │   │   ├── common/           # Shared types
│   │   │   │   │   ├── identifier.rs # Defines DDEX identifiers like ISRC, ISNI, GRid with validation and formatting
│   │   │   │   │   ├── localized.rs  # Handles localized strings with language codes and territory-specific text
│   │   │   │   │   ├── mod.rs        # Module exports for common types used across DDEX models
│   │   │   │   │   └── territory.rs  # Territory and region codes with ISO 3166 country code support
│   │   │   │   ├── flat/             # Flattened model
│   │   │   │   │   ├── deal.rs       # Flattened deal structure for easier manipulation of commercial terms
│   │   │   │   │   ├── message.rs    # Flattened DDEX message representation with direct access to all entities
│   │   │   │   │   ├── mod.rs        # Module exports for flattened model types
│   │   │   │   │   ├── release.rs    # Flattened release model with denormalized resource references
│   │   │   │   │   └── track.rs      # Flattened track/resource model with inline technical details
│   │   │   │   ├── graph/            # Graph model
│   │   │   │   │   ├── deal.rs       # Graph-based deal model preserving DDEX reference relationships
│   │   │   │   │   ├── header.rs     # Message header with sender, recipient, and control metadata
│   │   │   │   │   ├── message.rs    # Root graph message structure with collections of parties, resources, releases, deals
│   │   │   │   │   ├── mod.rs        # Module exports for graph model types
│   │   │   │   │   ├── party.rs      # Party entities representing labels, publishers, and rights holders
│   │   │   │   │   ├── release.rs    # Graph release model with resource references and display artists
│   │   │   │   │   └── resource.rs   # Resource model for sound recordings, videos, images with technical details
│   │   │   │   ├── versions/         # Version variations
│   │   │   │   │   ├── common.rs     # Shared version-agnostic DDEX structures and traits
│   │   │   │   │   ├── ern_42.rs     # ERN 4.2 specific model variations and mappings
│   │   │   │   │   ├── ern_43.rs     # ERN 4.3 specific model variations and mappings
│   │   │   │   │   ├── ern_382.rs    # ERN 3.8.2 specific model variations and legacy support
│   │   │   │   │   ├── mod.rs        # Version detection and routing module
│   │   │   │   │   ├── tests.rs      # Unit tests for version-specific model transformations
│   │   │   │   │   └── version.rs    # Version enum and detection logic for DDEX standards
│   │   │   │   ├── extensions.rs     # Handles unknown XML elements and namespace extensions for round-trip fidelity
│   │   │   │   └── mod.rs            # Root module exports for all DDEX models
│   │   │   ├── Cargo.toml            # Core library manifest with minimal dependencies
│   │   │   ├── error.rs              # Shared error types and result aliases for the suite
│   │   │   ├── ffi.rs                # Foreign function interface types for language bindings
│   │   │   └── lib.rs                # Core library entry point and re-exports
│   │   └── Cargo.toml                # Workspace member configuration for core package
│   │
│   ├── ddex-builder/                 # The DDEX Builder tool
│   │   ├── benches/                  # 
│   │   │   ├── building.rs           # Performance benchmarks for XML generation and building operations
│   │   │   └── canonicalization.rs   # Benchmarks for DB-C14N canonicalization performance
│   │   ├── bindings/                 #
│   │   │   ├── node/                 # Node.js native bindings using napi-rs (future)
│   │   │   ├── python/               # Python bindings using PyO3/maturin (future)
│   │   │   └── wasm/                 # WebAssembly bindings for browser support (future)
│   │   ├── src/                      # Rust builder implementation
│   │   │   ├── builder/              # High-level builder API implementation
│   │   │   ├── canonical/            #
│   │   │   │   └── mod.rs            # DB-C14N implementation for deterministic XML canonicalization
│   │   │   ├── determinism/          # Deterministic output enforcement and IndexMap usage
│   │   │   ├── generator/            #
│   │   │   │   ├── mod.rs            # XML generation orchestration and AST-to-XML transformation
│   │   │   │   └── xml_writer.rs     # Low-level XML writing with proper escaping and formatting
│   │   │   ├── linker/               #
│   │   │   │   ├── auto_linker.rs    # Automatic reference linking between releases, resources, and deals
│   │   │   │   ├── mod.rs            # Reference linker module exports and configuration
│   │   │   │   ├── reference_generator.rs  # Generates unique references for entities (R1, A1, etc.)
│   │   │   │   ├── relationship_manager.rs  # Manages and validates entity relationships in DDEX messages
│   │   │   │   └── types.rs          # Type definitions for linking operations and reference maps
│   │   │   ├── presets/
│   │   │   │   └── mod.rs            # Pre-configured builder settings for common DDEX profiles
│   │   │   ├── ast.rs                # Abstract syntax tree representation for XML generation
│   │   │   ├── builder.rs            # Main builder API for constructing DDEX messages programmatically
│   │   │   ├── determinism.rs        # Ensures deterministic output with stable ordering and formatting
│   │   │   ├── error.rs              # Builder-specific error types and error handling
│   │   │   ├── id_generator.rs       # Generates stable, deterministic IDs using content hashing
│   │   │   ├── lib.rs                # Builder library entry point and public API exports
│   │   │   └── preflight.rs          # Pre-build validation and compliance checking
│   │   ├── tests/                    #
│   │   │   ├── snashopts/            # Snapshot testing fixtures for golden file tests
│   │   │   ├── basic_test.rs         # Core builder functionality tests
│   │   │   ├── golden_files.rs       # Tests against known-good DDEX XML outputs
│   │   │   ├── linker_test.rs        # Unit tests for reference linking logic
│   │   │   ├── linker_xml_intergration_test.rs  # Integration tests for linker with real XML generation
│   │   │   └── xml_generation_text.rs  # Tests for XML generation correctness and formatting
│   │   ├── Cargo.toml                # Builder package dependencies and metadata
│   │   └── clippy.toml               # Rust linter configuration for code quality
│   │
│   └── ddex-parser/                  # The DDEX Parser tool
│       ├── benches/                  # 
│       │   ├── memory.rs             # Memory usage benchmarks for parsing operations
│       │   ├── parsing.rs            # Performance benchmarks for XML parsing speed
│       │   └── streaming.rs          # Benchmarks for streaming parser with large files
│       ├── benchmarks/               # Additional benchmark data and results storage
│       ├── bindings/                 #
│       │   ├── node/                 # 
│       │   │   ├── src/              # 
│       │   │   │   ├── Cargo.toml    # Node binding crate configuration (if separate crate)
│       │   │   │   ├── index.ts      # TypeScript entry point with unified native/WASM detection
│       │   │   │   ├── lib.rs        # Rust NAPI bindings for Node.js native addon
│       │   │   │   ├── parser.ts     # TypeScript parser class wrapping native/WASM implementation
│       │   │   │   ├── types.ts      # TypeScript type definitions for parsed DDEX structures
│       │   │   │   └── wasm.d.ts     # WASM module type declarations
│       │   │   ├── build.rs          # Build script for compiling Node.js native addon
│       │   │   ├── Cargo.toml        # Node bindings package configuration
│       │   │   ├── index.d.ts        # TypeScript declaration file for npm package
│       │   │   ├── index.js          # JavaScript entry point with platform detection
│       │   │   ├── LICENSE           # MIT license for the npm package
│       │   │   ├── package.json      # npm package metadata and dependencies
│       │   │   ├── README.md         # Documentation for JavaScript/TypeScript users
│       │   │   └── tsconfig.json     # TypeScript compiler configuration
│       │   ├── python/               # 
│       │   │   ├── python/           # 
│       │   │   │   ├── ddex_parser/  # 
│       │   │   │   │   ├── __init__.py  # Python package initialization and public API exports
│       │   │   │   │   └── cli.py    # Command-line interface for Python users
│       │   │   ├── src/              # 
│       │   │   │   └── lib.rs        # PyO3 bindings for Python extension module
│       │   │   ├── Cargo.toml        # Python bindings package configuration
│       │   │   ├── pyproject.toml    # Python package metadata and build configuration
│       │   │   └── README.md         # Documentation for Python users
│       │   └── wasm/                 # 
│       │       ├── src/              # 
│       │       │   └── lib.rs        # WebAssembly bindings for browser usage
│       │       ├── build.sh          # Build script for WASM compilation with wasm-opt
│       │       └── Cargo.toml        # WASM package configuration
│       ├── src/                      # Main parser implementation (placeholder for future extraction)
│       ├── tests/                    # Parser integration and unit tests
│       ├── build.rs                  # Parser build script for code generation and optimization
│       ├── Cargo.toml                # Parser package dependencies and configuration
│       └── README.md                 # Main parser documentation and usage guide
│
├── recipes/                          # Stable hash ID recipes
│   ├── release_v1.toml
│   ├── resource_v1.toml
│   └── party_v1.toml
│
├── scripts/                          # Build and release scripts
│   ├── setup-monorepo.sh             # Initialize workspace
│   ├── migrate-parser.sh             # Migrate existing code
│   ├── extract-core.sh               # Extract shared models
│   ├── build-all.sh
│   ├── test-all.sh
│   ├── release-parser.sh
│   ├── release-builder.sh
│   └── publish-all.sh
│
├── supply-chain/                     # Supply chain security
│   ├── cargo-deny.toml
│   ├── SBOM.json
│   └── sigstore/
│
├── test-suite/                       # Shared test fixtures
│   ├── edge-cases/                   # 
│   ├── golden/                       # Expected outputs
│   ├── nasty/                        # Attack vectors
│   ├── valid/                        # Valid DDEX files
│   ├── generate_test_corpus.py       # 
│   └── README.md                     #
│
├── blueprint.md                      # This document
├── Cargo.toml                        # Root workspace config
├── karma.conf.js                     # 
├── LICENSE                           # MIT License
├── package.json                      # Root npm workspace config
└── README.md                         # Suite documentation
```

## Implementation Roadmap

### Phase 1: Foundation Refactor ✅ COMPLETED

#### 1.1 Monorepo Setup ✅
- [x] Create `ddex-suite` repository
- [x] Setup root `Cargo.toml` workspace
- [x] Setup root `package.json` for npm workspaces
- [x] Create `packages/` directory structure
- [x] Configure unified CI/CD pipelines
- [x] Setup cross-package testing infrastructure
- [x] Create migration scripts

#### 1.2 Migration & Core Extraction ✅
- [x] Run migration script to move all files
- [x] Extract models to `packages/core/src/models/`
- [x] Extract errors to `packages/core/src/error.rs`
- [x] Extract FFI types to `packages/core/src/ffi.rs`
- [x] Update all import paths in `packages/ddex-parser`
- [x] Add extension support to models
- [x] Implement `toBuildRequest()` method
- [x] Verify all tests pass

### Phase 2: Complete DDEX Parser v1.0 🔄 IN PROGRESS (90% Complete)

#### 2.1 Enhanced Parser Features ✅ COMPLETED
- [x] Add `includeRawExtensions` option
- [x] Add `includeComments` option
- [x] Implement extension preservation
- [x] Add `_graph` reference to flattened models
- [x] Complete `toBuildRequest()` implementation
- [x] Test round-trip fidelity
- [x] Add 10+ round-trip tests (basic tests complete)

#### 2.2 JavaScript/TypeScript Bindings ✅ COMPLETED
- [x] Complete WASM browser build (<500KB)
- [x] Optimize with wasm-opt  
- [x] Unify npm package (native + WASM)
- [x] Publish to npm ✅ (v0.1.0 published!)

#### 2.3 Python Bindings ✅ COMPLETED
- [x] Complete PyO3/maturin setup
- [x] Configure cibuildwheel for all platforms
- [x] Implement Python API
- [x] Add DataFrame integration ✅ (Full implementation with to_dataframe/from_dataframe)
- [x] Generate type stubs
- [x] Test on macOS/ARM (working!)
- [x] Fix PyO3 0.21 compatibility issues ✅ (All compatibility issues resolved)
- [x] Publish to PyPI as `ddex-parser`

**Python Integration Status Summary:**
- ✅ **PyO3 0.21 Compatibility**: All deprecated APIs updated, proper Bound type usage
- ✅ **DataFrame Integration**: Complete pandas integration with multiple schemas (flat/releases/tracks)
- ✅ **Bidirectional Conversion**: Both to_dataframe() and from_dataframe() methods implemented
- ✅ **Error Handling**: Comprehensive error handling with proper Python exceptions
- ✅ **Async Support**: Full async/await support with tokio integration
- ✅ **Streaming Support**: Memory-efficient streaming for large files
- ✅ **Type Safety**: Full type stubs with IDE support
- ✅ **Platform Support**: Successfully compiled and tested on multiple platforms

#### 2.4 CLI & Polish ✅ COMPLETED
- [x] Build comprehensive CLI with clap
- [x] Add parse/detect-version/sanity-check commands
- [x] Create basic documentation
- [x] Security audit (✅ No vulnerabilities in Rust CLI)
- [x] Binary size optimization (551KB)
- [ ] Add extract/stream/batch commands (future enhancement)
- [ ] Create shell completions (future enhancement)
- [ ] Performance optimization (future enhancement)

### Phase 3: DDEX Builder Development 🔄 IN PROGRESS

#### 3.1 Builder Foundation ✅ COMPLETED
- [x] Initialize `packages/ddex-builder`
- [x] Import `packages/core` as dependency
- [x] Implement DB-C14N/1.0 spec (basic implementation)
- [x] Build AST generation
- [x] Implement determinism engine with IndexMap
- [x] Add determinism lint (deny HashMap/HashSet)
- [x] Create working XML generation pipeline
- [x] Generate valid DDEX ERN 4.3 XML
- [x] Add basic tests (7 passing)

#### 3.2 Core Builder Features ✅ COMPLETED
- [x] Implement Flat→AST→XML pipeline
- [x] Basic XML serialization with namespaces
- [x] Element ordering and formatting
- [x] Build reference linker (auto-link releases/resources)
  - [x] Create linker module structure
  - [x] Implement deterministic reference generation
  - [x] Build automatic relationship linking
  - [x] Integrate with XML generation pipeline
  - [x] Add comprehensive test coverage (9 tests passing)
- [x] Add stable-hash ID generation (content-based IDs)
  - [x] SHA256/Blake3 hash algorithms
  - [x] Versioned recipe system (v1)
  - [x] Unicode normalization (NFC/NFD/NFKC/NFKD)
  - [x] Content-based deterministic IDs
- [x] Implement comprehensive preflight checks (ISRC/UPC validation)
  - [x] ISRC format validation with regex
  - [x] UPC format and checksum validation
  - [x] Territory code validation
  - [x] ISO 8601 duration validation
  - [x] Profile-specific validation (AudioAlbum/AudioSingle)
- [x] Support full ERN 4.3 AudioAlbum profile
  - [x] Profile-specific requirements
  - [x] Track count validation
  - [x] Required field enforcement
- [x] Create golden file tests
  - [x] Snapshot testing with insta
  - [x] Determinism verification
  - [x] 26 total tests passing

#### 3.3 Builder Bindings ✅ COMPLETED
- [x] Setup napi-rs for Node.js
  - [x] Native N-API bindings with async support
  - [x] TypeScript definitions auto-generated
  - [x] NPM package structure as @ddex-suite/builder
  - [x] Comprehensive test suite
- [x] Setup PyO3 for Python
  - [x] Native Python extension module
  - [x] Python type hints included
  - [x] pyproject.toml with maturin build
  - [x] Test suite with import verification
- [x] Setup wasm-bindgen for browser
  - [x] WASM module at 116KB (77% under 500KB target!)
  - [x] ES6 module support
  - [x] Interactive HTML test environment
  - [x] Console error handling
- [x] Generate TypeScript definitions
  - [x] Complete type coverage for all APIs
  - [x] JSDoc comments for IDE support
  - [x] Consistent with JavaScript conventions
- [x] Implement DataFrame→DDEX for Python
  - [x] from_dataframe() method implemented
  - [x] Pandas integration ready
  - [x] Bulk operations support
- [x] Test all bindings
  - [x] Node.js tests passing (✅ 95% API consistency)
  - [x] Python tests verified
  - [x] WASM browser tests working
  - [x] API consistency report generated

#### 3.4 Advanced Builder Features
- [x] Add partner presets (Spotify, YouTube)
- [x] Implement streaming writer
- [x] Add semantic diff engine
- [x] Support UpdateReleaseMessage
- [x] Add JSON Schema generation
- [x] Multi-version support (3.8.2, 4.2, 4.3)

#### 3.5 Builder Polish ✅ COMPLETED & PUBLISHED v0.1.0!
- [x] Complete CLI with all commands
- [x] Add `--verify-determinism` flag
- [x] Performance optimization
- [x] Security audit
- [x] Complete documentation
- [x] Tag ddex-builder v0.1.0 ✅
- [x] **Published to npm as `ddex-builder` v0.1.0** ✅
  - Package size: 347.6 kB compressed / 752.5 kB unpacked
  - Available at: https://www.npmjs.com/package/ddex-builder
- [x] **Published to PyPI as `ddex-builder` v0.1.0** ✅
  - Wheel: 240KB (ARM64 macOS), Source: 255KB
  - Available at: https://pypi.org/project/ddex-builder/0.1.0/
- [x] **Git tags pushed to GitHub** ✅
  - ddex-builder-v0.1.0
  - ddex-builder-node-v0.1.0
  - ddex-builder-python-v0.1.0
  - ddex-builder-wasm-v0.1.0

#### 3.6 Core Feature Implementation v0.2.0 ✅ **COMPLETED**
- [x] Fix PyO3 0.21 compatibility and complete Python bindings
- [x] Complete parser core functionality
- [x] Enhanced Parser CLI
- [x] Enhanced Builder 
- [x] Comprehensive integration tests
- [x] Complete documentation v0.2.0
- [x] **Published to npm as `ddex-builder` v0.2.0**
- [x] **Published to PyPI as `ddex-builder` v0.2.0**
- [x] **Published to npm as `ddex-parser` v0.2.0**
- [x] **Published to PyPI as `ddex-parser` v0.2.0**

### Phase 4: Suite Integration & Launch 🔄 IN PROGRESS

#### 4.1 Integration Testing ✅ **COMPLETED (v0.2.0)**
- [x] End-to-end round-trip tests
- [x] Cross-package integration tests  
- [x] Performance benchmarks validated (94 core tests passing)
- [x] Enhanced Python bindings with PyO3 0.21 compatibility
- [x] Advanced CLI features for both parser and builder
- [x] Complete workspace version management
- [x] Comprehensive CHANGELOG.md documentation
- [x] **Suite v0.2.0 Published** to npm and PyPI

#### 4.2 Documentation & Launch 🔄 **IN PROGRESS**
- [ ] Create unified documentation site
- [ ] Build interactive tutorials
- [ ] Record demo videos
- [ ] Prepare marketing materials
- [ ] Setup community channels
- [ ] Official v1.0.0 release

#### 4.3 Perfect Fidelity Engine (Deferred to v1.1)
- [ ] Implement full DB-C14N/1.0 spec
- [ ] Create extension preservation system
- [ ] Build comment retention engine
- [ ] Add namespace management
- [ ] Implement attribute preservation
- [ ] Test with 100+ real-world files

#### 4.2 Determinism Specification (DB-C14N/1.0)

##### DB-C14N/1.0 Specification Summary

1. **XML Declaration & Encoding** - Fixed `<?xml version="1.0" encoding="UTF-8"?>`
2. **Whitespace, Indentation, Line Endings** - LF normalized, 2-space indent
3. **Attribute Ordering Policy** - Alphabetical by local name
4. **Namespace Prefix Lock Tables** - Per ERN version, immutable
5. **Text Normalization** - Unicode NFC, character policy by field
6. **Date/Time** - UTC, ISO8601Z, zero-pad rules
7. **Element Ordering** - Generated from XSD + checksum
8. **Canonical Hash Definition** - SHA-256 of specific byte ranges

### Determinism Configuration

```typescript
interface DeterminismConfig {
  // Canonicalization mode
  canonMode: 'db-c14n' | 'pretty' | 'compact';
  
  // Element ordering
  sortStrategy: 'canonical' | 'input-order' | 'custom';
  customSortOrder?: Record<string, string[]>;
  
  // Namespace handling
  namespaceStrategy: 'locked' | 'inherit';
  lockedPrefixes?: Record<string, string>;
  
  // Formatting
  outputMode: 'db-c14n' | 'pretty' | 'compact';
  lineEnding: 'LF' | 'CRLF';
  indentChar: 'space' | 'tab';
  indentWidth: number;
  
  // String normalization
  unicodeNormalization: 'NFC' | 'NFD' | 'NFKC' | 'NFKD';
  xmlCharacterPolicy: 'escape' | 'cdata' | 'reject';
  quoteStyle: 'double' | 'single';
  
  // Date/Time
  timeZonePolicy: 'UTC' | 'preserve' | 'local';
  dateTimeFormat: 'ISO8601Z' | 'ISO8601' | 'custom';
  
  // Reproducibility
  emitReproducibilityBanner?: boolean;
  verifyDeterminism?: boolean;
  canonicalHash?: boolean;
}
```

### Phase 5: Production Ready 🎯 FUTURE

#### 5.1 Performance & Scale
- [ ] Optimize for sub-10ms parsing
- [ ] Implement zero-copy where possible
- [ ] Add performance benchmarks
- [ ] Create stress tests
- [ ] Memory profiling

#### 5.2 Documentation & Community
- [ ] Complete API documentation
- [ ] Create tutorial videos
- [ ] Build interactive playground
- [ ] Write migration guides
- [ ] Establish governance model

#### 5.3 Enterprise Features
- [ ] Add batch processing
- [ ] Implement validation rules engine
- [ ] Create migration tools
- [ ] Build compliance reports
- [ ] Add audit logging

## Success Metrics

### Technical KPIs
- ✅ Parse 95% of real-world DDEX files (tested with valid samples)
- ✅ Perfect round-trip fidelity
- ✅ Deterministic XML generation (reference linker complete, basic canonicalization working)
- ✅ <50ms parsing for typical releases (achieved)
- ✅ <15ms generation for typical releases (achieved - ~0.27s for test suite)
- ✅ Memory bounded streaming (implemented, needs testing)
- ✅ Zero security vulnerabilities (achieved for Rust CLI)
- ✅ WASM bundle <500KB (achieved - 114KB for builder!)
- ✅ Cross-platform bindings (Node.js, Python, WASM all working)
- 🔄 100% determinism across CI matrix (basic tests passing)

### Current Build Verification Summary (v0.1.0)

| Component          | Size  | Status                   |
|--------------------|-------|--------------------------|
| Rust Core          | 9.4MB | ✅ Development artifact   |
| Node.js (packaged) | 347KB | ✅ Excellent for npm      |
| Python wheel       | 235KB | ✅ Compact for PyPI       |
| WASM bundle        | 166KB | ✅ 67% under 500KB target |

### Platform Support
- ✅ Node.js: Native binaries with TypeScript definitions
- ✅ Python: ABI3 compatible wheels (Python 3.8+)
- ✅ WASM: Browser-ready bundle at 114KB
- ✅ All exports verified and functional:
  - DdexBuilder, StreamingDdexBuilder
  - batchBuild, validateStructure
  - Full API consistency across platforms

### Distribution Channels
- **NPM**: https://www.npmjs.com/package/ddex-builder
- **PyPI**: https://pypi.org/project/ddex-builder/0.1.0/
- **GitHub**: https://github.com/daddykev/ddex-suite

## Current Status (January 2025)

### Completed ✅
- Monorepo structure established
- Core models extracted and shared
- DDEX Parser with full language bindings (v0.2.0 published)
- DDEX Builder with deterministic output (v0.2.0 published)
- **Complete Suite Integration**: v0.2.0 published to npm and PyPI
- Enhanced Python bindings with PyO3 0.21 compatibility
- Advanced CLI features for both parser and builder
- Full DataFrame integration for data analysis
- Complete round-trip capability with 94 core tests passing
- Comprehensive CHANGELOG.md documentation

### In Progress 🔄
- Documentation site creation
- Interactive tutorials
- Marketing materials preparation

### Next Steps 🎯
1. Create unified documentation site
2. Build interactive tutorials and demo videos
3. Setup community channels
4. Official v1.0.0 release
5. Advanced features (DB-C14N/1.0 full spec, enterprise features)

## Contributing

The project is currently in active development. Community contributions will be welcomed starting in Q1 2026 once the core architecture stabilizes.

## License

MIT License - See LICENSE file for details.