# DDEX Core

Shared DDEX data models and utilities for the DDEX Suite toolkit.

## Overview

This crate provides the core data structures and utilities shared across the DDEX Suite components:

- **DDEX Parser** (`ddex-parser`) - High-performance DDEX XML parser
- **DDEX Builder** (`ddex-builder`) - Deterministic DDEX XML builder

## Features

- Complete DDEX data models for ERN 3.8.2, 4.2, and 4.3
- Serde serialization/deserialization support
- TypeScript type definitions (optional feature)
- Comprehensive error handling with `thiserror`

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ddex-core = "0.2.5"
```

## Usage

```rust
use ddex_core::*;

// Work with DDEX data structures
// This crate provides the foundation for parsing and building DDEX XML
```

## Features

- `typescript` - Enable TypeScript type generation with `ts-rs`

## License

MIT License - See LICENSE file for details.

## Related Crates

- [`ddex-parser`](https://crates.io/crates/ddex-parser) - Parse DDEX XML files
- [`ddex-builder`](https://crates.io/crates/ddex-builder) - Build deterministic DDEX XML

## Documentation

For complete documentation and examples, visit:
- [Documentation](https://docs.rs/ddex-core)
- [GitHub Repository](https://github.com/daddykev/ddex-suite)
- [Project Website](https://ddex-suite.web.app)