#!/bin/bash
# scripts/build-node.sh

set -e

echo "🚀 Building Node.js bindings..."

cd bindings/node

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf dist build prebuilds index.node

# Install dependencies
echo "📦 Installing dependencies..."
npm ci

# Build native module
echo "🔨 Building native module..."
npm run build

# Run tests
echo "🧪 Running tests..."
npm test

# Build prebuilds for distribution
echo "📦 Building prebuilds..."
npm run prebuild:all

echo "✅ Node.js bindings built successfully!"
echo "📊 Build artifacts:"
ls -la dist/
ls -la prebuilds/ 2>/dev/null || echo "No prebuilds yet"