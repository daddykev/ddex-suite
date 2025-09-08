# DDEX Parser

[![npm version](https://img.shields.io/npm/v/ddex-parser.svg)](https://www.npmjs.com/package/ddex-parser)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub](https://img.shields.io/badge/GitHub-ddex--suite-blue)](https://github.com/daddykev/ddex-suite)

> ⚠️ **Early Alpha Release**: This is v0.1.0, an early release of the DDEX Parser. While the core parsing functionality is working, this package is under active development. API changes are expected before v1.0.0.

High-performance DDEX XML parser built on a Rust core, designed to transform complex DDEX messages into clean, strongly-typed data structures. Part of the [DDEX Suite](https://github.com/daddykev/ddex-suite) project.

## 🎯 Project Vision

DDEX Parser is being built as part of DDEX Suite - a rigorous, end-to-end learning project to create production-grade tools for music metadata. The goal is to ship a single Rust core that provides consistent behavior across JavaScript, Python, and Rust, making DDEX processing as simple as working with JSON.

## ✅ What's Working in v0.1.0

- **Core Parsing**: Parse DDEX ERN 3.8.2, 4.2, and 4.3 messages
- **Version Detection**: Automatic DDEX version detection
- **Dual Model Architecture**: Both graph (faithful) and flattened (developer-friendly) representations
- **TypeScript Support**: Full TypeScript definitions
- **Basic Security**: XXE protection and entity expansion limits
- **Round-trip Support**: Extension preservation for future builder integration

## 🚧 Coming Soon

- **WASM Support**: Browser compatibility (Phase 2.2 - in progress)
- **Native Bindings**: Faster performance with native Node.js addon
- **Streaming**: Handle large files with bounded memory
- **Python Package**: PyPI distribution as `ddex-parser`
- **CLI Tool**: Command-line interface for parsing and extraction
- **Full Documentation**: Comprehensive API documentation and examples

## 📦 Installation

```bash
npm install ddex-parser
```

Note: This early release uses a JavaScript wrapper with placeholders for native/WASM bindings. Performance will significantly improve when native bindings are complete.

## 🚀 Basic Usage

```javascript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();

// Parse DDEX XML (currently returns mock data in v0.1.0)
const result = await parser.parse(xmlContent);

// Access the flattened model (when fully implemented)
console.log(result.flat.releases);

// Access the graph model (when fully implemented)
console.log(result.graph);
```

## 🎭 Dual Model Architecture (Design)

The parser is designed to provide two complementary views of DDEX data:

### Graph Model (Faithful)
Preserves the exact DDEX structure with references - for compliance and round-trip operations.

### Flattened Model (Developer-Friendly)
Denormalized and resolved for easy consumption - ideal for applications.

*Note: In v0.1.0, these models return placeholder data. Full implementation coming in v0.2.0.*

## 🔧 Current Limitations

This is an early alpha release with the following limitations:

1. **Mock Implementation**: Currently returns placeholder data while Rust bindings are being completed
2. **No WASM Yet**: Browser support is not available (coming in v0.2.0)
3. **No Native Bindings**: Running in pure JavaScript mode (native bindings coming soon)
4. **Limited Streaming**: Streaming API exists but is not fully functional
5. **Basic Error Handling**: Error messages are simplified
6. **No CLI**: Command-line interface not yet included

## 📊 Roadmap

- **v0.1.0** (Current) - Initial release with core structure
- **v0.2.0** (2-3 weeks) - WASM support, basic native bindings
- **v0.3.0** (4-5 weeks) - Full native bindings, streaming
- **v0.4.0** (6-7 weeks) - Python package, CLI tool
- **v0.5.0** (8-9 weeks) - Performance optimization, documentation
- **v1.0.0** (Q4 2025) - Production ready with stable API

## 🏗️ Technical Architecture

Built on a Rust core with bindings for multiple languages:

```
DDEX Parser
├── Rust Core (packages/ddex-parser)
│   ├── Parser Engine
│   ├── Security Layer
│   └── Transform Pipeline
├── JavaScript/TypeScript (this package)
│   ├── Native Bindings (coming soon)
│   └── WASM Fallback (coming soon)
└── Python Bindings (coming soon)
```

## 🤝 Related Projects

- [DDEX Suite](https://github.com/daddykev/ddex-suite) - Parent monorepo
- [DDEX Builder](https://github.com/daddykev/ddex-suite/tree/main/packages/ddex-builder) - Deterministic XML generation (coming Q4 2025)
- [DDEX Workbench](https://github.com/ddex/ddex-workbench) - Official DDEX validation service

## 👨‍💻 Developer Note

I'm building DDEX Suite as a learning project to deepen my Rust skills while creating production-grade tools for music metadata. This parser is the first component, with the builder following soon. The goal is to provide a complete "Parse → Modify → Build" workflow with perfect round-trip fidelity.

While this is a learning project, I'm committed to production quality - the suite will feature proper security hardening, comprehensive testing, and real-world performance optimization. Follow the project for updates!

## ⚠️ Important Notes for Early Adopters

1. **API Stability**: The API will change before v1.0.0. Pin to exact versions in production.
2. **Performance**: Current performance is limited due to JavaScript implementation. Native bindings will provide 10-100x improvement.
3. **Features**: Many advertised features are in development. Check the "What's Working" section above.
4. **Support**: This is a personal project. While I respond to issues, there's no commercial support.
5. **Testing**: Please test thoroughly before using in production systems.

## 🐛 Reporting Issues

Found a bug or have a suggestion? Please open an issue on [GitHub](https://github.com/daddykev/ddex-suite/issues) with:
- Your Node.js version
- The DDEX version you're parsing (3.8.2, 4.2, or 4.3)
- A minimal code example
- The error message or unexpected behavior

## 📄 License

MIT © Kevin Marques Moo

## 🙏 Acknowledgments

This parser is designed to complement the official [DDEX Workbench](https://github.com/ddex/ddex-workbench) by providing structural parsing while Workbench handles XSD validation.

Special thanks to the DDEX community for their standards documentation and to everyone who provides feedback during this early development phase.

---

**Version**: 0.1.0
**Status**: Early Alpha - Not Production Ready  
**Repository**: https://github.com/daddykev/ddex-suite  
**NPM**: https://www.npmjs.com/package/ddex-parser  
**Author**: Kevin Marques Moo

*Thank you for trying this early release! Your feedback helps shape the future of DDEX Suite.*