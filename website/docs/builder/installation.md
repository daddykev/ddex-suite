---
sidebar_position: 2
---

# Installation

Set up DDEX Builder in your development environment. The builder is available through standard package managers with prebuilt native binaries for optimal performance.

## Quick Install

### JavaScript / TypeScript (Node.js)

```bash
npm install ddex-builder
```

```bash
yarn add ddex-builder
```

```bash
pnpm add ddex-builder
```

### Python

```bash
pip install ddex-builder
```

```bash
conda install -c conda-forge ddex-builder  # Coming soon
```

### Rust

```bash
# Add to your Cargo.toml dependencies
cargo add ddex-builder ddex-core

# Or manually edit Cargo.toml
[dependencies]
ddex-builder = "0.2.5"
ddex-core = "0.2.5"

# Install CLI tool
cargo install ddex-builder
```

## System Requirements

### Node.js Requirements

- **Node.js**: 14.0.0 or higher (18.0.0+ recommended)
- **Architecture**: x64, arm64
- **Platforms**: Linux, macOS, Windows
- **TypeScript**: 4.5+ (optional, for type definitions)

The builder uses native binaries for maximum performance:

```json
{
  "engines": {
    "node": ">=14.0.0"
  }
}
```

### Python Requirements

- **Python**: 3.8 or higher (3.11+ recommended)
- **Architecture**: x64, arm64  
- **Platforms**: Linux, macOS, Windows
- **Optional Dependencies**: pandas 2.0+ (for DataFrame integration)

```bash
# Install with DataFrame support
pip install "ddex-builder[pandas]"

# Install with async support  
pip install "ddex-builder[async]"

# Install all optional features
pip install "ddex-builder[all]"
```

### Rust Requirements

- **Rust**: `1.70.0` or higher (2021 edition)
- **Architecture**: x64, arm64
- **Platforms**: Linux, macOS, Windows
- **Memory**: Depends on build size (typically \<100MB)
- **Optional Dependencies**: tokio (for async features)

```toml
[dependencies]
ddex-builder = { version = "0.2.5", features = ["async"] }
ddex-core = "0.2.5"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"  # For JSON input/output
```

### WebAssembly Requirements (Browser)

- **Modern browsers** with WebAssembly support
- **Bundle size**: ~380KB (gzipped)
- **Memory**: 20MB+ available heap space
- **ES Modules**: Required for optimal performance

## Platform-Specific Installation

### macOS

Native binaries support both Intel and Apple Silicon:

```bash
# Universal binary (recommended)
npm install ddex-builder

# Force specific architecture if needed
npm install ddex-builder --target_arch=arm64  # Apple Silicon
npm install ddex-builder --target_arch=x64    # Intel
```

For Python on Apple Silicon:
```bash
# Ensure you have the correct Python architecture
python -c "import platform; print(platform.machine())"

# Install with proper architecture
pip install ddex-builder
```

### Linux

Prebuilt binaries support major distributions:

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install -y build-essential  # If building from source
npm install ddex-builder

# RHEL/CentOS/Fedora
sudo dnf install -y gcc-c++ make  # If building from source  
npm install ddex-builder

# Alpine Linux
apk add --no-cache musl-dev gcc  # If building from source
npm install ddex-builder
```

The package includes binaries for both glibc and musl systems.

### Windows

Windows binaries support x64 and arm64:

```powershell
# Install with npm
npm install ddex-builder

# Install with Chocolatey (alternative)
choco install nodejs
npm install ddex-builder

# Windows Subsystem for Linux (WSL)
wsl npm install ddex-builder  # Uses Linux binary
```

## Verification

### Node.js Verification

```typescript
import { DdexBuilder } from 'ddex-builder';

// Test basic functionality
const builder = new DdexBuilder();
console.log('Available presets:', builder.getAvailablePresets());

// Test building
try {
  const testData = {
    messageHeader: {
      messageId: 'TEST_001',
      messageSenderName: 'Test Sender',
      messageRecipientName: 'Test Recipient'
    },
    releases: [{
      releaseId: 'REL001',
      title: 'Test Release',
      artist: 'Test Artist',
      trackIds: ['TR001']
    }],
    resources: [{
      resourceId: 'TR001',
      resourceType: 'SoundRecording',
      title: 'Test Track',
      artist: 'Test Artist'
    }]
  };

  const xml = await builder.build(testData);
  console.log('✅ Builder working correctly');
  console.log(`Generated XML: ${xml.length} bytes`);
} catch (error) {
  console.error('❌ Builder test failed:', error.message);
}
```

### Python Verification

```python
from ddex_builder import DdexBuilder, __version__
import sys

# Check version
print(f"DDEX Builder version: {__version__}")
print(f"Python version: {sys.version}")

# Test basic functionality
builder = DdexBuilder()
presets = builder.get_available_presets()
print(f"Available presets: {presets}")

# Test building
test_data = {
    'message_header': {
        'message_id': 'TEST_001',
        'message_sender_name': 'Test Sender',
        'message_recipient_name': 'Test Recipient'
    },
    'releases': [{
        'release_id': 'REL001',
        'title': 'Test Release',
        'artist': 'Test Artist',
        'track_ids': ['TR001']
    }],
    'resources': [{
        'resource_id': 'TR001',
        'resource_type': 'SoundRecording',
        'title': 'Test Track',
        'artist': 'Test Artist'
    }]
}

try:
    xml = builder.build(test_data)
    print("✅ Builder working correctly")
    print(f"Generated XML: {len(xml)} bytes")
except Exception as e:
    print(f"❌ Builder test failed: {e}")

# Test optional dependencies
try:
    import pandas as pd
    print("✅ Pandas integration available")
except ImportError:
    print("ℹ️ Pandas not installed (DataFrame features unavailable)")
```

### Rust Verification

```rust
use ddex_builder::DdexBuilder;
use ddex_core::models::BuildRequest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check basic functionality
    let builder = DdexBuilder::new();
    println!("✅ DDEX Builder loaded successfully");
    
    // Test building with minimal data
    let build_request = BuildRequest {
        message_header: ddex_core::models::MessageHeader {
            message_id: "TEST_001".to_string(),
            message_sender_name: "Test Sender".to_string(),
            message_recipient_name: "Test Recipient".to_string(),
            ..Default::default()
        },
        releases: vec![ddex_core::models::Release {
            release_id: "REL001".to_string(),
            title: vec![ddex_core::models::LocalizedString {
                text: "Test Release".to_string(),
                language_code: Some("en".to_string()),
                ..Default::default()
            }],
            display_artist: "Test Artist".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    match builder.build(&build_request) {
        Ok(xml) => {
            println!("✅ Builder working correctly");
            println!("Generated XML: {} bytes", xml.len());
        }
        Err(e) => println!("❌ Builder test failed: {}", e),
    }

    // Test presets
    let presets = builder.get_available_presets();
    println!("Available presets: {:?}", presets);

    Ok(())
}
```

## Development Installation

### Building from Source (Node.js)

If prebuilt binaries aren't available for your platform:

```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone repository
git clone https://github.com/ddex-suite/ddex-suite.git
cd ddex-suite/packages/ddex-builder/bindings/node

# Install dependencies
npm install

# Build native addon
npm run build

# Run tests
npm test
```

### Building from Source (Python)

```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
pip install maturin[patchelf]

# Clone repository
git clone https://github.com/ddex-suite/ddex-suite.git
cd ddex-suite/packages/ddex-builder/bindings/python

# Development build
maturin develop

# Production build
maturin build --release

# Install locally
pip install target/wheels/ddex_builder-*.whl
```

## Docker Installation

### Official Docker Images

```dockerfile
# Node.js with DDEX Builder
FROM node:18-alpine

RUN npm install -g ddex-builder

# Python with DDEX Builder
FROM python:3.11-slim

RUN pip install ddex-builder
```

### Multi-language Container

```dockerfile
FROM ubuntu:22.04

# Install Node.js and Python
RUN apt-get update && apt-get install -y \
    nodejs npm \
    python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install DDEX Builder for both languages
RUN npm install -g ddex-builder
RUN pip3 install ddex-builder

# Verify installations
RUN node -e "const {DdexBuilder} = require('ddex-builder'); console.log('Node.js OK')"
RUN python3 -c "from ddex_builder import DdexBuilder; print('Python OK')"
```

## IDE Configuration

### Visual Studio Code

Recommended extensions and settings:

```json
// .vscode/settings.json
{
  "typescript.preferences.includePackageJsonAutoImports": "on",
  "typescript.suggest.autoImports": true,
  "python.analysis.extraPaths": [
    "./node_modules/ddex-builder"
  ],
  "files.associations": {
    "*.ddex": "xml",
    "*.ern": "xml"
  }
}
```

### JetBrains IDEs (WebStorm, PyCharm)

```javascript
// Enable TypeScript support for better intellisense
// Settings → Languages & Frameworks → TypeScript
// Enable: Strict null checks, No implicit any
```

## Troubleshooting

### Common Installation Issues

#### Native Binary Not Found (Node.js)

```bash
# Error: "Cannot find module 'ddex-builder-...'"
# Solution: Reinstall with rebuild flag
npm install ddex-builder --rebuild

# Or clear cache and reinstall
npm cache clean --force
rm -rf node_modules package-lock.json
npm install
```

#### Python Build Errors

```bash
# Error: "Microsoft Visual C++ 14.0 is required" (Windows)
# Solution: Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Error: "error: rustc not found"
# Solution: Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Permission Errors (Linux/macOS)

```bash
# Error: "EACCES: permission denied"
# Solution: Use npm prefix or user installation
npm config set prefix ~/.npm-global
export PATH=$PATH:~/.npm-global/bin

# Or use user installation for Python
pip install --user ddex-builder
```

### Platform-Specific Issues

#### Apple Silicon (M1/M2) Macs

```bash
# If getting wrong architecture binary
npm uninstall ddex-builder
npm cache clean --force
arch -arm64 npm install ddex-builder

# For Python
pip uninstall ddex-builder
pip cache purge
pip install ddex-builder
```

#### Linux GLIBC Version

```bash
# Check GLIBC version
ldd --version

# If GLIBC is too old, use musl binary
npm install ddex-builder --force --platform=linux --arch=x64 --libc=musl

# Or build from source in compatible environment
```

### Memory and Performance Issues

#### Node.js Memory Limits

```bash
# Increase Node.js memory for large builds
node --max-old-space-size=4096 your-app.js

# Set memory limit in environment
export NODE_OPTIONS="--max-old-space-size=4096"
```

#### Python Memory Management

```python
import resource

# Set memory limit (2GB)
resource.setrlimit(resource.RLIMIT_AS, (2147483648, -1))

# Monitor memory usage
def monitor_memory():
    import psutil
    process = psutil.Process()
    memory_mb = process.memory_info().rss / 1024 / 1024
    print(f"Memory usage: {memory_mb:.1f} MB")
```

## Performance Optimization

### Node.js Optimization

```typescript
// Production configuration
const builder = new DdexBuilder({
  // Use fastest preset for your use case
  preset: 'performance',
  
  // Disable validation for trusted input
  validate: false,
  
  // Enable parallel processing
  parallel: true,
  
  // Optimize memory usage
  streaming: true
});
```

### Python Optimization

```python
# Use PyPy for better performance with large datasets
# Install PyPy 3.8+ then:
pypy3 -m pip install ddex-builder

# Or use threading for CPU-intensive tasks
import threading
from concurrent.futures import ThreadPoolExecutor

def build_parallel(data_list):
    with ThreadPoolExecutor(max_workers=4) as executor:
        builders = [DdexBuilder() for _ in range(4)]
        futures = []
        
        for i, data in enumerate(data_list):
            builder = builders[i % 4]
            future = executor.submit(builder.build, data)
            futures.append(future)
        
        return [f.result() for f in futures]
```

## Environment Variables

Configure the builder with environment variables:

```bash
# Enable debug logging
export DDEX_BUILDER_DEBUG=1

# Set default preset
export DDEX_BUILDER_DEFAULT_PRESET=spotify

# Configure validation level
export DDEX_BUILDER_VALIDATION=strict

# Memory limits (MB)
export DDEX_BUILDER_MAX_MEMORY=512

# Threading/parallelism
export DDEX_BUILDER_THREADS=4
```

## Next Steps

Once installed, you're ready to start building DDEX files:

- **[Quick Start Guide](./quick-start)** - Build your first DDEX file in minutes
- **[API Reference](./api-reference)** - Explore all available methods
- **[Presets Guide](./presets)** - Platform-specific configurations
- **[Canonicalization](./canonicalization)** - Understanding deterministic output

Need help? Check our [GitHub Issues](https://github.com/ddex-suite/ddex-suite/issues) or [Discussions](https://github.com/ddex-suite/ddex-suite/discussions) for community support.