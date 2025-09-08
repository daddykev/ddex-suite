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

#### Universal Music Group - Catalog Migration
**Scenario**: UMG needs to migrate their entire back catalog (3M+ recordings) from a legacy system to a new distribution platform requiring DDEX ERN 4.3.

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
      messageSender: { partyName: [{ text: 'Universal Music Group' }] },
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

#### Sony Music - Weekly New Release Feed
**Scenario**: Sony needs to generate weekly DDEX feeds for all new releases across their labels for 50+ DSP partners.

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
            'message_sender': {'party_name': [{'text': 'Sony Music Entertainment'}]},
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

#### DistroKid - Independent Artist Onboarding
**Scenario**: DistroKid processes 10,000+ new releases daily from independent artists and needs to generate DDEX feeds for multiple platforms.

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
        namespace: 'distrokid', 
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
  const platforms = ['spotify', 'apple', 'amazon', 'youtube'];
  
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

#### Spotify - Ingestion Pipeline
**Scenario**: Spotify receives 100,000+ DDEX messages daily and needs to normalize them for internal processing.

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
        # Apply Spotify-specific business rules
        normalized = {
            **release,
            'spotify_id': generate_spotify_id(release),
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

#### Warner Music Group - Multi-Format Delivery
**Scenario**: WMG needs to deliver the same release in different formats (physical, digital, streaming) with format-specific metadata.

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

### The "Parse → Modify → Build" Workflow

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
├── packages/
│   ├── core/                         # Shared Rust crate
│   │   ├── src/
│   │   │   ├── models/               # DDEX data models
│   │   │   │   ├── common/           # Shared types
│   │   │   │   ├── flat/             # Flattened model
│   │   │   │   ├── graph/            # Graph model
│   │   │   │   ├── versions/         # Version variations
│   │   │   │   └── extensions.rs     # Extension support
│   │   │   ├── error.rs              # Common error types
│   │   │   ├── ffi.rs                # FFI-friendly types
│   │   │   └── lib.rs                # Library entry
│   │   └── Cargo.toml
│   │
│   ├── ddex-parser/                  # The DDEX Parser tool
│   │   ├── node/                     # Node.js package
│   │   ├── python/                   # Python package
│   │   ├── wasm/                     # WASM for browsers
│   │   ├── src/                      # Rust parser implementation
│   │   └── README.md
│   │
│   └── ddex-builder/                 # The DDEX Builder tool
│       ├── node/                     # Node.js package
│       ├── python/                   # Python package
│       ├── wasm/                     # WASM for browsers
│       ├── src/                      # Rust builder implementation
│       │   ├── builder/
│       │   ├── canonical/            # DB-C14N/1.0
│       │   ├── determinism/
│       │   ├── generator/
│       │   ├── linker/
│       │   ├── presets/
│       │   └── preflight/
│       └── README.md
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
│   ├── ARCHITECTURE.md              # Monorepo architecture
│   ├── SUITE_OVERVIEW.md            # DDEX Suite vision
│   ├── ROUND_TRIP.md                # Parse→Modify→Build guide
│   └── CONTRIBUTING.md              # Contribution guidelines
│
├── examples/                         # Example usage
│   ├── parse-modify-build/          # Round-trip examples
│   ├── parser-only/
│   └── builder-only/
│
├── test-suite/                       # Shared test fixtures
│   ├── round-trip/                   # Round-trip test cases
│   ├── valid/                        # Valid DDEX files
│   ├── nasty/                        # Attack vectors
│   ├── vendor-quirks/                # Real-world edge cases
│   ├── golden/                       # Expected outputs
│   └── README.md
│
├── scripts/                          # Build and release scripts
│   ├── setup-monorepo.sh            # Initialize workspace
│   ├── migrate-parser.sh            # Migrate existing code
│   ├── extract-core.sh              # Extract shared models
│   ├── build-all.sh
│   ├── test-all.sh
│   ├── release-parser.sh
│   ├── release-builder.sh
│   └── publish-all.sh
│
├── recipes/                          # Stable hash ID recipes
│   ├── release_v1.toml
│   ├── resource_v1.toml
│   └── party_v1.toml
│
├── supply-chain/                     # Supply chain security
│   ├── cargo-deny.toml
│   ├── SBOM.json
│   └── sigstore/
│
├── clippy.toml                       # Determinism lint config
├── Cargo.toml                        # Root workspace config
├── package.json                      # Root npm workspace config
├── tsconfig.json                     # Shared TypeScript config
├── LICENSE                           # MIT License
└── README.md                         # Suite documentation
```

## Implementation Roadmap

### Phase 1: Foundation Refactor

#### 1.1 Monorepo Setup
- [x] Create `ddex-suite` repository
- [x] Setup root `Cargo.toml` workspace
- [x] Setup root `package.json` for npm workspaces
- [x] Create `packages/` directory structure
- [x] Configure unified CI/CD pipelines
- [x] Setup cross-package testing infrastructure
- [x] Create migration scripts

#### 1.2 Migration & Core Extraction
- [x] Run migration script to move all ✅ files
- [x] Extract models to `packages/core/src/models/`
- [x] Extract errors to `packages/core/src/error.rs`
- [x] Extract FFI types to `packages/core/src/ffi.rs`
- [x] Update all import paths in `packages/ddex-parser`
- [x] Add extension support to models
- [x] Implement `toBuildRequest()` method
- [x] Verify all tests pass

### Phase 2: Complete DDEX Parser v1.0

#### 2.1 Enhanced Parser Features ✅ COMPLETED (Sept 7, 2025)
- [x] Add `includeRawExtensions` option
- [x] Add `includeComments` option
- [x] Implement extension preservation
- [x] Add `_graph` reference to flattened models
- [x] Complete `toBuildRequest()` implementation
- [x] Test round-trip fidelity
- [x] Add 10+ round-trip tests (basic tests complete, comprehensive tests pending)

#### 2.2 JavaScript/TypeScript Bindings
- [x] Complete WASM browser build (<500KB)
- [x] Optimize with wasm-opt  
- [x] Unify npm package (native + WASM)
- [x] Publish to npm ✅ (v0.1.0 published!)

#### 2.3 Python Bindings
- [x] Complete PyO3/maturin setup
- [x] Configure cibuildwheel for all platforms (config done, CI pending)
- [x] Implement Python API
- [x] Add DataFrame integration (stub ready, full implementation pending)
- [x] Generate type stubs (created, need to test)
- [x] Test on macOS/ARM (working!)
- [ ] Test on Linux/macOS/Windows (CI needed)
- [ ] Publish to PyPI as `ddex-parser`

#### 2.4 CLI & Polish
- [ ] Build comprehensive CLI with clap
- [ ] Add parse/extract/stream commands
- [ ] Create shell completions
- [ ] Complete documentation
- [ ] Security audit
- [ ] Performance optimization

### Phase 3: DDEX Builder Development

#### 3.1 Builder Foundation
- [ ] Initialize `packages/ddex-builder`
- [ ] Import `packages/core` as dependency
- [ ] Implement DB-C14N/1.0 spec
- [ ] Build AST generation
- [ ] Implement determinism engine with IndexMap
- [ ] Add determinism lint (deny HashMap/HashSet)

#### 3.2 Core Builder Features
- [ ] Implement Flat→AST→XML pipeline
- [ ] Build reference linker
- [ ] Add stable-hash ID generation
- [ ] Implement preflight checks
- [ ] Support ERN 4.3 AudioAlbum profile
- [ ] Create golden file tests

#### 3.3 Builder Bindings
- [ ] Setup napi-rs for Node.js
- [ ] Setup PyO3 for Python
- [ ] Setup wasm-bindgen for browser
- [ ] Generate TypeScript definitions
- [ ] Implement DataFrame→DDEX for Python
- [ ] Test all bindings

#### 3.4 Advanced Builder Features
- [ ] Add partner presets (Spotify, Apple, etc.)
- [ ] Implement streaming writer
- [ ] Add semantic diff engine
- [ ] Support UpdateReleaseMessage
- [ ] Add JSON Schema generation
- [ ] Multi-version support (3.8.2, 4.2, 4.3)

#### 3.5 Builder Polish
- [ ] Complete CLI with all commands
- [ ] Add `--verify-determinism` flag
- [ ] Performance optimization
- [ ] Security audit
- [ ] Complete documentation
- [ ] Tag builder v1.0.0

### Phase 4: Suite Integration & Launch

#### 4.1 Integration Testing
- [ ] End-to-end round-trip tests
- [ ] Cross-package integration tests
- [ ] Performance benchmarks
- [ ] Memory leak testing
- [ ] Fuzz testing (24-hour run)

#### 4.2 Documentation & Launch
- [ ] Create unified documentation site
- [ ] Build interactive tutorials
- [ ] Record demo videos
- [ ] Prepare marketing materials
- [ ] Setup community channels
- [ ] Official v1.0.0 release

## Success Metrics

### Technical KPIs
- ✅ Parse 95% of real-world DDEX files
- ✅ Perfect round-trip fidelity
- ✅ Deterministic XML generation (DB-C14N/1.0 compliance)
- ✅ <50ms parsing for typical releases
- ✅ <15ms generation for typical releases
- ✅ Memory bounded streaming
- ✅ Zero security vulnerabilities
- ✅ WASM bundle <500KB
- ✅ 100% determinism across CI matrix

### Adoption KPIs
- ✅ 1,000+ npm downloads/month
- ✅ 500+ PyPI downloads/month
- ✅ 10+ companies using in production
- ✅ 5+ major labels or distributors
- ✅ 10+ DSPs using for normalization
- ✅ 500+ GitHub stars
- ✅ Integration with DDEX Workbench

### Community KPIs
- ✅ <48hr response to issues
- ✅ Monthly releases
- ✅ 10+ external contributors
- ✅ Comprehensive documentation
- ✅ Active community

## Go/No-Go Checklist

### Phase 1 (Monorepo Setup) ✅
- [x] Monorepo structure created
- [x] All files successfully migrated
- [x] Core package extracted
- [x] All existing tests passing
- [x] CI/CD pipelines working
- [x] Round-trip tests added

### Phase 2 (Parser v1.0) 🔄 IN PROGRESS
- [x] Extension support working (Phase 2.1 complete)
- [x] WASM <500KB
- [ ] Published to npm as ddex-parser
- [ ] Published to PyPI
- [ ] CLI functional
- [ ] Documentation complete

### Phase 3 (Builder v1.0)
- [ ] DB-C14N/1.0 implemented
- [ ] Deterministic output verified
- [ ] All presets working
- [ ] Published to npm as ddex-builder
- [ ] Published to PyPI
- [ ] CLI functional

### Phase 4 (Suite v1.0)
- [ ] Round-trip tests 100% passing
- [ ] Performance targets met
- [ ] Security audit passed
- [ ] Documentation complete
- [ ] Community launched

---

**Version**: 2.0.1  
**Last Updated**: September 7, 2025  
**Status**: Phase 2.1 Complete - Phase 2.2 Starting  
**Repository**: github.com/daddykev/ddex-suite  
**Parser Target**: v1.0.0 in 3-4 weeks  
**Builder Target**: v1.0.0 in 7-8 weeks  
**Suite Target**: v1.0.0 in 9-10 weeks  
**License**: MIT