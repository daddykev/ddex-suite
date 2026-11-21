---
sidebar_position: 1
---

# Introduction to DDEX Suite

DDEX Suite is a high-performance toolkit for processing DDEX metadata with perfect fidelity. Built in Rust with native bindings for JavaScript/TypeScript and Python, it provides two complementary tools for working with DDEX XML files.

## What is DDEX?

DDEX (Digital Data Exchange) is the music industry standard for exchanging metadata between digital service providers, record labels, distributors, and other music industry stakeholders. DDEX XML files contain rich metadata about releases, tracks, artists, territories, and commercial terms.

## Why DDEX Suite?

### 🚀 **Performance**
- Parse 10KB files in &lt;5ms
- Process 100MB files in &lt;5s  
- Stream 1GB+ files with &lt;100MB memory usage

### 🎯 **Perfect Fidelity**
- Round-trip guarantee: Parse → Modify → Build produces identical results
- Preserves all DDEX extensions and custom fields
- Deterministic output with stable ordering

### 🔧 **Developer-Friendly**
- Clean TypeScript/Python objects instead of raw XML
- Both "graph" (faithful DDEX structure) and "flattened" (developer-friendly) representations
- Comprehensive error reporting and validation

### 🌍 **Multi-Language Support**
- Native Node.js bindings with TypeScript definitions
- Python bindings with DataFrame integration
- WebAssembly build for browser usage

## Core Components

### ddex-parser
Transforms DDEX XML into clean, structured data:

```typescript
import { DdexParser } from 'ddex-parser';

const parser = new DdexParser();
const result = await parser.parse(xmlContent);

// Access structured data
console.log(result.flat.releases[0].title);
console.log(result.flat.soundRecordings[0].artist);
```

### ddex-builder  
Generates deterministic DDEX XML from data structures:

```typescript
import { DdexBuilder } from 'ddex-builder';

const builder = new DdexBuilder();
const xml = await builder.build(result.toBuildRequest());
```

## Supported DDEX Versions

- **ERN 3.8.2** - Electronic Release Notification
- **ERN 4.2** - Electronic Release Notification  
- **ERN 4.3** - Electronic Release Notification (latest)

## Installation

### Node.js / TypeScript

```bash
npm install ddex-parser ddex-builder
```

### Python

```bash
pip install ddex-parser ddex-builder
```

## Quick Start

Ready to get started? Check out our [Getting Started Guide](./getting-started/) for installation instructions and your first DDEX processing workflow.

## Community & Support

- 📖 [Documentation](https://ddex-suite.org)
- 🐛 [Issues & Bug Reports](https://github.com/daddykev/ddex-suite/issues)
- 💬 [Discussions](https://github.com/daddykev/ddex-suite/discussions)
- 📦 [npm](https://www.npmjs.com/org/ddex-suite)
- 🐍 [PyPI](https://pypi.org/user/ddex-suite/)
