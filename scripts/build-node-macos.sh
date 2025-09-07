#!/bin/bash
# scripts/build-node-macos.sh

set -e

echo "🚀 Building Node.js bindings for macOS..."

cd bindings/node

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf target build dist index.node *.node

# Build the Rust library first
echo "🔨 Building Rust library..."
cd ../..
cargo build --release -p ddex-parser-node
cd bindings/node

# Build with napi
echo "📦 Building with napi-rs..."
npm run build:napi

# Build TypeScript
echo "📝 Building TypeScript..."
npm run build:ts

# Verify the build
echo "✅ Verifying build..."
if [ -f "index.node" ]; then
    echo "✓ index.node created successfully"
    ls -la index.node
else
    echo "✗ index.node not found!"
    exit 1
fi

if [ -d "dist" ]; then
    echo "✓ TypeScript compiled successfully"
    ls -la dist/
else
    echo "✗ dist/ directory not found!"
    exit 1
fi

echo "🎉 Build completed successfully!"