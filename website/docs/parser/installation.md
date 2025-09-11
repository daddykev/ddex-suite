---
sidebar_position: 2
---

# Installation

Get started with DDEX Parser by installing the package for your preferred programming language. The parser is available through standard package managers with prebuilt binaries for all major platforms.

## Quick Install

### JavaScript / TypeScript (Node.js)

```bash
npm install ddex-parser
```

```bash
yarn add ddex-parser
```

```bash
pnpm add ddex-parser
```

### Python

```bash
pip install ddex-parser
```

```bash
conda install -c conda-forge ddex-parser  # Coming soon
```

### Rust

```bash
# Add to your Cargo.toml dependencies
cargo add ddex-parser ddex-core

# Or manually edit Cargo.toml
[dependencies]
ddex-parser = "0.2.5"
ddex-core = "0.2.5"

# Install CLI tool
cargo install ddex-parser
```

## System Requirements

### Node.js Requirements

- **Node.js**: 18.0.0 or higher
- **Architecture**: x64, arm64
- **Platforms**: Linux, macOS, Windows
- **TypeScript**: 4.5+ (optional, for type definitions)

```json
{
  "engines": {
    "node": ">=18.0.0"
  }
}
```

### Python Requirements

- **Python**: 3.8 or higher  
- **Architecture**: x64, arm64
- **Platforms**: Linux, macOS, Windows
- **Optional Dependencies**: pandas 2.0+ (for DataFrame integration)

```bash
# Install with DataFrame support
pip install "ddex-parser[pandas]"

# Install with async support
pip install "ddex-parser[async]"

# Install all optional features
pip install "ddex-parser[all]"
```

### Rust Requirements

- **Rust**: `1.70.0` or higher
- **Architecture**: x64, arm64
- **Platforms**: Linux, macOS, Windows
- **Memory**: Depends on DDEX file size (typically \<50MB)
- **Optional Dependencies**: tokio (for async features)

```toml
[dependencies]
ddex-parser = { version = "0.2.5", features = ["async"] }
tokio = { version = "1.0", features = ["full"] }
```

### Browser Requirements (WebAssembly)

- **Modern browsers** with WebAssembly support
- **Bundle size**: ~489KB (gzipped)
- **Memory**: 10MB+ available heap space
- **ES Modules**: Required for optimal bundle splitting

## Platform-Specific Installation

### macOS

Native binaries are provided for both Intel and Apple Silicon:

```bash
# Install for current architecture
npm install ddex-parser

# Force specific architecture (if needed)
npm install ddex-parser --target_arch=arm64  # Apple Silicon
npm install ddex-parser --target_arch=x64    # Intel
```

### Linux

Prebuilt binaries support major distributions:

```bash
# Ubuntu/Debian
apt update && apt install -y build-essential  # If building from source
npm install ddex-parser

# RHEL/CentOS/Fedora  
dnf install -y gcc-c++ make  # If building from source
npm install ddex-parser

# Alpine Linux
apk add --no-cache musl-dev gcc  # If building from source
npm install ddex-parser
```

### Windows

Windows binaries support both x64 and arm64:

```powershell
# Install with npm
npm install ddex-parser

# Install with Chocolatey (alternative)
choco install nodejs
npm install ddex-parser
```

## Verification

### Node.js Verification

```typescript
import { DDEXParser } from 'ddex-parser';

const parser = new DDEXParser();
console.log(`DDEX Parser version: ${parser.version}`);

// Test basic functionality
const testXML = `<?xml version="1.0"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <MessageId>TEST001</MessageId>
  </MessageHeader>
</NewReleaseMessage>`;

try {
  const result = await parser.parseString(testXML);
  console.log('✅ Parser working correctly');
  console.log(`Message ID: ${result.graph.messageHeader?.messageId}`);
} catch (error) {
  console.error('❌ Parser test failed:', error.message);
}
```

### Python Verification

```python
from ddex_parser import DDEXParser
import sys

# Check version
parser = DDEXParser()
print(f"DDEX Parser version: {parser.__version__}")
print(f"Python version: {sys.version}")

# Test basic functionality
test_xml = """<?xml version="1.0"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <MessageId>TEST001</MessageId>
  </MessageHeader>
</NewReleaseMessage>"""

try:
    result = parser.parse(test_xml)
    print("✅ Parser working correctly")
    print(f"Message ID: {result.message_id}")
except Exception as e:
    print(f"❌ Parser test failed: {e}")

# Test optional dependencies
try:
    import pandas as pd
    print("✅ Pandas integration available")
except ImportError:
    print("ℹ️ Pandas not installed (DataFrame features unavailable)")
```

### Rust Verification

```rust
use ddex_parser::DDEXParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check version and capabilities
    let parser = DDEXParser::new();
    println!("✅ DDEX Parser loaded successfully");
    
    // Test basic functionality
    let test_xml = r#"<?xml version="1.0"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">
  <MessageHeader>
    <MessageId>TEST001</MessageId>
  </MessageHeader>
</NewReleaseMessage>"#;

    match parser.parse(test_xml) {
        Ok(result) => {
            println!("✅ Parser working correctly");
            if let Some(message_id) = &result.graph.message_header.message_id {
                println!("Message ID: {}", message_id);
            }
        }
        Err(e) => println!("❌ Parser test failed: {}", e),
    }

    Ok(())
}
```

## Development Installation

### Building from Source (Node.js)

If prebuilt binaries aren't available for your platform:

```bash
# Clone the repository
git clone https://github.com/ddex-suite/ddex-suite.git
cd ddex-suite/packages/ddex-parser/bindings/node

# Install dependencies
npm install

# Build native addon
npm run build:native

# Build TypeScript definitions  
npm run build:ts

# Run tests
npm test
```

### Building from Source (Python)

```bash
# Clone the repository
git clone https://github.com/ddex-suite/ddex-suite.git
cd ddex-suite/packages/ddex-parser/bindings/python

# Install build tools
pip install maturin[patchelf]

# Development build
maturin develop

# Production build
maturin build --release

# Install locally
pip install target/wheels/ddex_parser-*.whl
```

## Docker Installation

### Official Docker Images

```dockerfile
# Node.js with DDEX Parser
FROM node:18-alpine

# Install DDEX Parser
RUN npm install -g ddex-parser

# Python with DDEX Parser  
FROM python:3.11-slim

RUN pip install ddex-parser
```

### Multi-language Container

```dockerfile
FROM ubuntu:22.04

# Install Node.js and Python
RUN apt-get update && apt-get install -y \
    nodejs npm \
    python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install DDEX Parser for both languages
RUN npm install -g ddex-parser
RUN pip3 install ddex-parser

# Verify installations
RUN node -e "console.log(require('ddex-parser').DDEXParser)"
RUN python3 -c "from ddex_parser import DDEXParser; print('OK')"
```

## IDE Configuration

### Visual Studio Code

Install the DDEX extension for syntax highlighting and validation:

```json
// .vscode/settings.json
{
  "files.associations": {
    "*.ddex": "xml",
    "*.ern": "xml"
  },
  "xml.validation.enabled": true,
  "xml.format.enabled": true
}
```

Recommended extensions:
- **XML Tools** - XML formatting and validation
- **TypeScript Importer** - Auto-import DDEX Parser types

### JetBrains IDEs (WebStorm, PyCharm)

```javascript
// Enable TypeScript support
// Settings → Languages & Frameworks → TypeScript
// Enable: Strict null checks, No implicit any
```

## Troubleshooting

### Common Installation Issues

#### Native Binary Not Found (Node.js)

```bash
# Error: "Cannot find module '../build/Release/ddex_parser.node'"
# Solution: Install with rebuild flag
npm install ddex-parser --rebuild

# Or rebuild manually
npm rebuild ddex-parser
```

#### Python Build Errors

```bash
# Error: "Microsoft Visual C++ 14.0 is required" (Windows)
# Solution: Install Visual Studio Build Tools
# https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Error: "error: rustc not found" 
# Solution: Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Permission Errors (Linux/macOS)

```bash
# Error: "EACCES: permission denied"
# Solution: Use npm prefix or sudo
npm config set prefix ~/.npm-global
export PATH=$PATH:~/.npm-global/bin

# Or install with sudo (not recommended)
sudo npm install -g ddex-parser
```

### Memory Issues

If you encounter memory errors with large files:

```typescript
// Increase Node.js memory limit
node --max-old-space-size=4096 your-app.js

// Use streaming for large files
const parser = new DDEXParser({ streaming: true });
```

```python
# Python memory settings
import resource

# Set memory limit (bytes)
resource.setrlimit(resource.RLIMIT_AS, (2147483648, -1))  # 2GB
```

### Platform-Specific Issues

#### Apple Silicon (M1/M2) Macs

```bash
# If getting x64 binary on arm64
rm -rf node_modules package-lock.json
npm cache clean --force
npm install ddex-parser
```

#### Linux GLIBC Version

```bash
# Check GLIBC version
ldd --version

# If GLIBC is too old, build from source or use Docker
docker run --rm -v $(pwd):/workspace node:18 npm install ddex-parser
```

## Performance Optimization

### Node.js Optimization

```typescript
// Production settings
const parser = new DDEXParser({
  // Disable debugging
  debug: false,
  
  // Optimize for your use case
  streaming: false,  // Faster for small files
  validation: 'basic',  // Skip strict validation
  
  // Memory limits
  maxMemoryMB: 512
});
```

### Python Optimization

```python
# Use PyPy for better performance
pip install ddex-parser  # Works with PyPy 3.8+

# Or compile with optimizations
pip install --upgrade --force-reinstall --no-cache-dir ddex-parser
```

## Next Steps

Once installed, you're ready to start parsing DDEX files:

- **[Quick Start Guide](./quick-start)** - Parse your first file in minutes
- **[API Reference](./api-reference)** - Explore all available methods
- **[Advanced Usage](./advanced-usage)** - Streaming, performance tuning, and production tips

Need help? Check our [GitHub Issues](https://github.com/ddex-suite/ddex-suite/issues) or [Discussions](https://github.com/ddex-suite/ddex-suite/discussions) for community support.