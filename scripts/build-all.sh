# Add to scripts/build-all.sh
#!/bin/bash

set -e

echo "Building all bindings..."

# Build core first
echo "Building Rust core..."
cd core
cargo build --release
cd ..

# Build Node.js bindings
echo "Building Node.js bindings..."
./scripts/build-node.sh

echo "✅ All bindings built successfully!"