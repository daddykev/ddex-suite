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

| Mode | # Releases | # Tracks | Generation Time | Memory Usage |
|------|------------|----------|-----------------|--------------|
| **DB-C14N + Stable Hash** | | | | |
| | 1 | 12 | <15ms ±3ms | <3MB |
| | 100 | 1,200 | <150ms ±30ms | <20MB |
| | 1,000 | 12,000 | <1.5s ±300ms | <120MB |
| **DB-C14N + UUID** | | | | |
| | 1 | 12 | <10ms ±2ms | <2MB |
| | 100 | 1,200 | <100ms ±20ms | <15MB |
| | 1,000 | 12,000 | <1s ±200ms | <100MB |

### Benchmark Specifications

- **Hardware Baseline**: AWS m7g.large (2 vCPU, 8GB RAM)
- **Software**: Node 20 LTS, Python 3.11, Rust 1.75
- **Metrics**: P50, P95, P99 latency + peak RSS memory

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
}

// Python
class DDEXBuilder:
    def build(self, request: BuildRequest, options: Optional[BuildOptions] = None) -> BuildResult: ...
    async def build_async(self, request: BuildRequest, options: Optional[BuildOptions] = None) -> BuildResult: ...
    def preflight(self, request: BuildRequest) -> PreflightResult: ...
    def canonicalize(self, xml: Union[str, bytes]) -> str: ...
    def apply_preset(self, preset: str, lock: bool = False) -> None: ...
```

## Complete Project Structure

```
ddex-suite/ (MONOREPO ROOT)
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                    # Unified CI for monorepo
│   │   ├── parser-release.yml        # Parser-specific release
│   │   ├── builder-release.yml       # Builder-specific release
│   │   ├── rust.yml                  # Rust tests
│   │   ├── node.yml                  # Node bindings CI
│   │   ├── python.yml                # Python bindings CI
│   │   ├── security.yml              # Security scanning
│   │   └── determinism.yml           # Cross-platform determinism tests
│   └── dependabot.yml                # Dependency updates
│
├── packages/
│   ├── core/                         # Shared Rust crate
│   │   ├── src/
│   │   │   ├── models/               # DDEX data models (FROM parser)
│   │   │   │   ├── common/           # Shared types
│   │   │   │   │   ├── identifier.rs ✅
│   │   │   │   │   ├── localized.rs  ✅
│   │   │   │   │   ├── territory.rs  ✅
│   │   │   │   │   └── mod.rs        ✅
│   │   │   │   ├── flat/             # Flattened model
│   │   │   │   │   ├── deal.rs       ✅
│   │   │   │   │   ├── message.rs    ✅
│   │   │   │   │   ├── release.rs    ✅
│   │   │   │   │   ├── track.rs      ✅
│   │   │   │   │   └── mod.rs        ✅
│   │   │   │   ├── graph/            # Graph model
│   │   │   │   │   ├── deal.rs       ✅
│   │   │   │   │   ├── header.rs     ✅
│   │   │   │   │   ├── message.rs    ✅
│   │   │   │   │   ├── party.rs      ✅
│   │   │   │   │   ├── resource.rs   ✅
│   │   │   │   │   ├── release.rs    ✅
│   │   │   │   │   └── mod.rs        ✅
│   │   │   │   ├── versions/         # Version variations
│   │   │   │   │   ├── common.rs     ✅
│   │   │   │   │   ├── ern_382.rs    ✅
│   │   │   │   │   ├── ern_42.rs     ✅
│   │   │   │   │   ├── ern_43.rs     ✅
│   │   │   │   │   └── mod.rs        ✅
│   │   │   │   ├── extensions.rs     # Extension support (NEW)
│   │   │   │   └── mod.rs            ✅
│   │   │   ├── error.rs              # Common error types (FROM parser)
│   │   │   ├── ffi.rs                # FFI-friendly types (FROM parser)
│   │   │   └── lib.rs                # Library entry
│   │   ├── tests/
│   │   │   ├── model_consistency.rs  # Round-trip tests
│   │   │   └── ffi_contract.rs       # FFI boundary tests
│   │   └── Cargo.toml
│   │
│   ├── ddex-parser/                  # The DDEX Parser tool
│   │   ├── node/                     # Node.js package
│   │   │   ├── __tests__/
│   │   │   │   ├── basic.test.ts     ✅
│   │   │   │   ├── parser.test.ts    ✅
│   │   │   │   ├── streaming.test.ts
│   │   │   │   └── roundtrip.test.ts
│   │   │   ├── dist/                 ✅
│   │   │   ├── src/
│   │   │   │   ├── index.ts          ✅
│   │   │   │   ├── parser.ts         ✅
│   │   │   │   └── types.ts          ✅
│   │   │   ├── binding.gyp           ✅
│   │   │   ├── build.rs              ✅
│   │   │   ├── package.json          ✅ (rename to @ddex-suite/parser)
│   │   │   └── tsconfig.json         ✅
│   │   ├── python/                   # Python package
│   │   │   ├── ddex_parser/
│   │   │   ├── src/
│   │   │   │   ├── lib.rs            ✅
│   │   │   │   └── types.rs
│   │   │   ├── tests/
│   │   │   ├── Cargo.toml            ✅
│   │   │   └── pyproject.toml        ✅
│   │   ├── wasm/                     # WASM for browsers
│   │   │   ├── src/
│   │   │   │   └── lib.rs            ✅
│   │   │   ├── Cargo.toml            ✅
│   │   │   └── build.sh
│   │   ├── src/                      # Rust parser implementation
│   │   │   ├── parser/
│   │   │   │   ├── detector.rs       ✅
│   │   │   │   ├── dom.rs            ✅
│   │   │   │   ├── security.rs       ✅
│   │   │   │   ├── stream.rs         ✅
│   │   │   │   ├── version_aware.rs  ✅
│   │   │   │   └── mod.rs            ✅
│   │   │   ├── transform/
│   │   │   │   ├── extract.rs
│   │   │   │   ├── flatten.rs        ✅
│   │   │   │   ├── graph.rs          ✅
│   │   │   │   ├── resolve.rs        ✅
│   │   │   │   ├── extensions.rs     # Extension handling (NEW)
│   │   │   │   └── mod.rs            ✅
│   │   │   ├── bin/
│   │   │   │   └── main.rs           ✅
│   │   │   └── lib.rs                ✅ (update imports)
│   │   ├── tests/                    # Integration tests
│   │   │   ├── integration_test.rs   ✅
│   │   │   ├── vendor_quirks.rs      ✅
│   │   │   ├── version_detection.rs  ✅
│   │   │   ├── roundtrip_test.rs     # Round-trip tests (NEW)
│   │   │   └── extension_test.rs     # Extension preservation (NEW)
│   │   ├── benches/
│   │   │   ├── memory.rs             ✅
│   │   │   ├── parsing.rs            ✅
│   │   │   └── streaming.rs          ✅
│   │   ├── Cargo.toml                # Update dependencies
│   │   └── README.md
│   │
│   └── ddex-builder/                 # The DDEX Builder tool
│       ├── node/                     # Node.js package
│       │   ├── __tests__/
│       │   │   ├── builder.test.ts
│       │   │   ├── determinism.test.ts
│       │   │   └── preset.test.ts
│       │   ├── src/
│       │   │   ├── index.ts
│       │   │   ├── builder.ts
│       │   │   ├── presets.ts
│       │   │   └── types.ts
│       │   ├── package.json          # @ddex-suite/builder
│       │   └── tsconfig.json
│       ├── python/                   # Python package
│       │   ├── ddex_builder/
│       │   ├── src/
│       │   │   ├── lib.rs
│       │   │   ├── types.rs
│       │   │   └── dataframe.rs
│       │   ├── tests/
│       │   └── pyproject.toml
│       ├── wasm/                     # WASM for browsers
│       │   ├── src/
│       │   │   └── lib.rs
│       │   └── Cargo.toml
│       ├── src/                      # Rust builder implementation
│       │   ├── builder/
│       │   │   ├── ast.rs
│       │   │   ├── context.rs
│       │   │   ├── transform.rs
│       │   │   └── mod.rs
│       │   ├── canonical/
│       │   │   ├── spec.rs           # DB-C14N/1.0
│       │   │   ├── ordering.rs
│       │   │   ├── normalization.rs
│       │   │   └── hash.rs
│       │   ├── determinism/
│       │   │   ├── config.rs
│       │   │   ├── stable_hash.rs
│       │   │   ├── recipes.rs
│       │   │   └── mod.rs
│       │   ├── generator/
│       │   │   ├── element.rs
│       │   │   ├── namespace.rs
│       │   │   ├── writer.rs
│       │   │   └── mod.rs
│       │   ├── linker/
│       │   │   ├── id_generator.rs
│       │   │   ├── reference.rs
│       │   │   └── mod.rs
│       │   ├── presets/
│       │   │   ├── spotify.rs
│       │   │   ├── apple.rs
│       │   │   └── mod.rs
│       │   ├── preflight/
│       │   │   ├── avs.rs
│       │   │   ├── format.rs
│       │   │   ├── min_requirements.rs
│       │   │   └── mod.rs
│       │   ├── bin/
│       │   │   └── main.rs          # CLI entry
│       │   └── lib.rs
│       ├── tests/
│       │   ├── builder_test.rs
│       │   ├── determinism_test.rs
│       │   └── golden/               # Golden file tests
│       ├── benches/
│       │   ├── generation.rs
│       │   └── determinism.rs
│       ├── Cargo.toml
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
│   │   ├── javascript/
│   │   │   ├── basic.ts
│   │   │   ├── catalog-migration.ts
│   │   │   └── streaming.ts
│   │   ├── python/
│   │   │   ├── basic.py
│   │   │   ├── dataframe.py
│   │   │   └── batch.py
│   │   └── rust/
│   │       └── main.rs
│   ├── parser-only/
│   └── builder-only/
│
├── test-suite/                       # Shared test fixtures
│   ├── round-trip/                   # Round-trip test cases
│   ├── valid/                        # Valid DDEX files
│   │   ├── ern-4.3/                 ✅
│   │   ├── ern-42/                  ✅
│   │   └── ern-382/                 ✅
│   ├── nasty/                        # Attack vectors
│   │   ├── billion-laughs.xml       ✅
│   │   ├── deep-nesting.xml         ✅
│   │   └── xxe-attempts/
│   ├── vendor-quirks/               # Real-world edge cases
│   ├── golden/                      # Expected outputs
│   └── README.md                    ✅
│
├── scripts/                         # Build and release scripts
│   ├── setup-monorepo.sh           # Initialize workspace
│   ├── migrate-parser.sh           # Migrate existing code
│   ├── extract-core.sh             # Extract shared models
│   ├── build-all.sh                ✅
│   ├── test-all.sh
│   ├── release-parser.sh
│   ├── release-builder.sh
│   └── publish-all.sh
│
├── recipes/                         # Stable hash ID recipes
│   ├── release_v1.toml
│   ├── resource_v1.toml
│   └── party_v1.toml
│
├── supply-chain/                    # Supply chain security
│   ├── cargo-deny.toml             ✅
│   ├── SBOM.json
│   └── sigstore/
│
├── Cargo.toml                       # Root workspace config
├── package.json                     # Root npm workspace config
├── tsconfig.json                    # Shared TypeScript config
├── LICENSE                          # MIT License
└── README.md                        # Suite documentation
```

**Legend:**
- ✅ = File exists in current parser repo
- (NEW) = New file/feature for monorepo
- (FROM parser) = Migrated from parser to core

## Implementation Roadmap

### Phase 1: Foundation Refactor (Weeks 1-2) - IMMEDIATE

#### Week 1: Monorepo Setup
- [ ] Create `ddex-suite` repository
- [ ] Setup root `Cargo.toml` workspace
- [ ] Setup root `package.json` for npm workspaces
- [ ] Create `packages/` directory structure
- [ ] Configure unified CI/CD pipelines
- [ ] Setup cross-package testing infrastructure
- [ ] Create migration scripts

#### Week 2: Migration & Core Extraction
- [ ] Run migration script to move all ✅ files
- [ ] Extract models to `packages/core/src/models/`
- [ ] Extract errors to `packages/core/src/error.rs`
- [ ] Extract FFI types to `packages/core/src/ffi.rs`
- [ ] Update all import paths in `packages/ddex-parser`
- [ ] Add extension support to models
- [ ] Implement `toBuildRequest()` method
- [ ] Verify all 8 version tests pass
- [ ] Verify all 20 Node.js tests pass

### Phase 2: Complete DDEX Parser v1.0 (Weeks 3-10)

#### Weeks 3-4: Enhanced Parser Features
- [ ] Add `includeRawExtensions` option
- [ ] Add `includeComments` option
- [ ] Implement extension preservation
- [ ] Add `_graph` reference to flattened models
- [ ] Complete `toBuildRequest()` implementation
- [ ] Test round-trip fidelity
- [ ] Add 10+ round-trip tests

#### Weeks 5-6: JavaScript/TypeScript Bindings
- [ ] Complete WASM browser build (<500KB)
- [ ] Optimize with wasm-opt
- [ ] Unify npm package (native + WASM)
- [ ] Update package name to `@ddex-suite/parser`
- [ ] Add streaming examples
- [ ] Test in all major browsers
- [ ] Publish to npm

#### Weeks 7-8: Python Bindings
- [ ] Complete PyO3/maturin setup
- [ ] Configure cibuildwheel for all platforms
- [ ] Implement Python API
- [ ] Add DataFrame integration
- [ ] Generate type stubs
- [ ] Test on Linux/macOS/Windows
- [ ] Publish to PyPI as `ddex-parser`

#### Weeks 9-10: CLI & Polish
- [ ] Build comprehensive CLI with clap
- [ ] Add parse/extract/stream commands
- [ ] Create shell completions
- [ ] Complete documentation
- [ ] Security audit
- [ ] Performance optimization
- [ ] Tag parser v1.0.0

### Phase 3: DDEX Builder Development (Weeks 11-20)

#### Weeks 11-12: Builder Foundation
- [ ] Initialize `packages/ddex-builder`
- [ ] Import `packages/core` as dependency
- [ ] Implement DB-C14N/1.0 spec
- [ ] Build AST generation
- [ ] Implement determinism engine with IndexMap
- [ ] Add determinism lint (deny HashMap/HashSet)

#### Weeks 13-14: Core Builder Features
- [ ] Implement Flat→AST→XML pipeline
- [ ] Build reference linker
- [ ] Add stable-hash ID generation
- [ ] Implement preflight checks
- [ ] Support ERN 4.3 AudioAlbum profile
- [ ] Create golden file tests

#### Weeks 15-16: Builder Bindings
- [ ] Setup napi-rs for Node.js
- [ ] Setup PyO3 for Python
- [ ] Setup wasm-bindgen for browser
- [ ] Generate TypeScript definitions
- [ ] Implement DataFrame→DDEX for Python
- [ ] Test all bindings

#### Weeks 17-18: Advanced Builder Features
- [ ] Add partner presets (Spotify, Apple, etc.)
- [ ] Implement streaming writer
- [ ] Add semantic diff engine
- [ ] Support UpdateReleaseMessage
- [ ] Add JSON Schema generation
- [ ] Multi-version support (3.8.2, 4.2, 4.3)

#### Weeks 19-20: Builder Polish
- [ ] Complete CLI with all commands
- [ ] Add `--verify-determinism` flag
- [ ] Performance optimization
- [ ] Security audit
- [ ] Complete documentation
- [ ] Tag builder v1.0.0

### Phase 4: Suite Integration & Launch (Weeks 21-24)

#### Weeks 21-22: Integration Testing
- [ ] End-to-end round-trip tests
- [ ] Cross-package integration tests
- [ ] Performance benchmarks
- [ ] Memory leak testing
- [ ] Fuzz testing (24-hour run)

#### Weeks 23-24: Documentation & Launch
- [ ] Create unified documentation site
- [ ] Build interactive tutorials
- [ ] Record demo videos
- [ ] Prepare marketing materials
- [ ] Setup community channels
- [ ] Official v1.0.0 release

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
└── fuzzing/                      # Fuzz test corpus
```

### Test Requirements

- **Unit Tests**: 95%+ code coverage
- **Integration Tests**: All major workflows
- **Round-Trip Tests**: 100% data preservation
- **Determinism Tests**: 100% pass rate across OS/arch
- **Fuzz Testing**: 24-hour run without crashes
- **Performance Tests**: No regression >5%
- **Security Tests**: All OWASP XML vulnerabilities

## CI/CD & Supply Chain Security

### GitHub Actions Matrix

```yaml
name: Suite CI/CD
on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
        node: [18, 20, 22]
        python: [3.8, 3.9, 3.10, 3.11, 3.12]
```

### Supply Chain Security

- **cargo-deny**: Audit Rust dependencies ✅
- **dependabot**: Automated updates
- **SLSA**: Supply chain provenance
- **Sigstore**: Artifact signing
- **SBOM**: Software bill of materials

## Success Metrics

### Technical KPIs
- ✅ Parse 95% of real-world DDEX files
- ✅ Perfect round-trip fidelity
- ✅ Deterministic XML generation
- ✅ <50ms parsing for typical releases
- ✅ <15ms generation for typical releases
- ✅ Memory bounded streaming
- ✅ Zero security vulnerabilities

### Adoption KPIs
- ✅ 1,000+ npm downloads/month
- ✅ 500+ PyPI downloads/month
- ✅ 10+ companies using in production
- ✅ 500+ GitHub stars
- ✅ Integration with DDEX Workbench

### Community KPIs
- ✅ <48hr response to issues
- ✅ Monthly releases
- ✅ 10+ external contributors
- ✅ Comprehensive documentation
- ✅ Active community

## Go/No-Go Checklist

### Phase 1 (Monorepo Setup)
- [ ] Monorepo structure created
- [ ] All ✅ files successfully migrated
- [ ] Core package extracted
- [ ] All existing tests passing
- [ ] CI/CD pipelines working
- [ ] Round-trip tests added

### Phase 2 (Parser v1.0)
- [ ] Extension support working
- [ ] WASM <500KB
- [ ] Published to npm as @ddex-suite/parser
- [ ] Published to PyPI
- [ ] CLI functional
- [ ] Documentation complete

### Phase 3 (Builder v1.0)
- [ ] DB-C14N/1.0 implemented
- [ ] Deterministic output verified
- [ ] All presets working
- [ ] Published to npm as @ddex-suite/builder
- [ ] Published to PyPI
- [ ] CLI functional

### Phase 4 (Suite v1.0)
- [ ] Round-trip tests 100% passing
- [ ] Performance targets met
- [ ] Security audit passed
- [ ] Documentation complete
- [ ] Community launched

---

**Version**: 1.0.0  
**Last Updated**: September 7, 2025  
**Status**: Ready to Execute - Phase 1 Starting  
**Repository**: github.com/daddykev/ddex-suite (to be created)  
**Parser Target**: v1.0.0 in 10 weeks  
**Builder Target**: v1.0.0 in 20 weeks  
**Suite Target**: v1.0.0 in 24 weeks