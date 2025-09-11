# DDEX Builder API Reference

Complete TypeScript and Python API documentation for the DDEX Builder with deterministic XML generation.

## Overview

The DDEX Builder provides deterministic XML generation with DB-C14N/1.0 canonicalization, ensuring identical output for identical input data. It supports industry presets, streaming for large catalogs, and comprehensive validation.

## Key Features

- **Deterministic Output**: Guaranteed identical XML for identical input data
- **DB-C14N/1.0 Canonicalization**: Industry-standard XML canonicalization
- **Industry Presets**: Pre-configured settings for major platforms (Spotify, Apple Music, YouTube Music)
- **Streaming Support**: Memory-efficient generation of large catalogs
- **Validation**: Comprehensive validation with detailed error reporting
- **Multi-Version Support**: ERN 3.8.2, 4.2, and 4.3 output formats

## Language Bindings

- [**JavaScript/TypeScript API**](./typescript) - Complete Node.js API reference
- [**Python API**](./python) - Python bindings with pandas integration
- [**Core Types**](./types) - Shared type definitions and interfaces

## Quick Navigation

| Component | Description |
|-----------|-------------|
| [`DdexBuilder`](./typescript#ddexbuilder) | Main builder class for creating DDEX XML |
| [`StreamingDdexBuilder`](./typescript#streamingddexbuilder) | Streaming builder for large catalogs |
| [`BuildRequest`](./types#buildrequest) | Input data structure for building |
| [`ValidationResult`](./types#validationresult) | Validation results and errors |
| [**Presets**](./presets) | Platform-specific configuration presets |

## Performance Characteristics

| Catalog Size | Build Time | Memory Usage | Output Size |
|--------------|------------|--------------|-------------|
| 100 releases | &lt;50ms | &lt;10MB | ~500KB |
| 1K releases | &lt;200ms | &lt;50MB | ~5MB |
| 10K releases | &lt;2s | &lt;200MB | ~50MB |
| 100K releases* | &lt;20s | &lt;500MB | ~500MB |

_*Using streaming builder for large catalogs_

## Common Usage Patterns

```typescript
import { DdexBuilder } from 'ddex-builder';

// Basic building
const builder = new DdexBuilder();
builder.applyPreset('spotify');
builder.addRelease(releaseData);
const xml = await builder.build();

// Streaming large catalogs
const streamBuilder = new StreamingDdexBuilder();
streamBuilder.startMessage(header, '4.3');
// Add releases...
const stats = streamBuilder.finishMessage();
const xml = streamBuilder.getXml();
```

```python
from ddex_builder import DdexBuilder

# Basic building
builder = DdexBuilder()
builder.apply_preset('spotify')
builder.add_release(release_data)
xml = await builder.build()

# Validation
result = await builder.validate()
if not result.is_valid:
    print("Validation errors:", result.errors)
```

## Deterministic Guarantees

The DDEX Builder ensures:

1. **Byte-Perfect Reproducibility**: Identical input → identical output
2. **Stable Ordering**: Consistent element and attribute ordering
3. **Content-Based IDs**: Deterministic ID generation based on content
4. **Canonical XML**: DB-C14N/1.0 compliant output format

## Industry Presets

| Platform | Preset Name | Description |
|----------|-------------|-------------|
| Spotify | `spotify` | Optimized for Spotify ingestion |
| Apple Music | `apple_music` | iTunes/Apple Music requirements |
| YouTube Music | `youtube_music` | YouTube Content ID compliance |
| Amazon Music | `amazon_music` | Amazon DSP specifications |
| Universal | `universal` | Generic streaming platform preset |

## Validation Levels

- **Structure Validation**: XML schema compliance
- **Business Rules**: Platform-specific requirements  
- **Reference Integrity**: Valid ID references and relationships
- **Format Compliance**: Date formats, identifiers, and codes