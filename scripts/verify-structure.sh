#!/bin/bash

echo "Verifying monorepo structure..."

# Check if core package exists and has models
if [ -d "packages/core/src/models" ]; then
    echo "✅ Core package with models exists"
else
    echo "❌ Core package models missing"
    exit 1
fi

# Check if parser package exists
if [ -d "packages/ddex-parser/src" ]; then
    echo "✅ Parser package exists"
else
    echo "❌ Parser package missing"
    exit 1
fi

# Check if bindings were migrated
if [ -d "packages/ddex-parser/bindings" ]; then
    echo "✅ Bindings migrated"
else
    echo "❌ Bindings missing"
    exit 1
fi

# Try to build
echo "Attempting to build..."
cargo build --workspace

echo "Structure verification complete!"
