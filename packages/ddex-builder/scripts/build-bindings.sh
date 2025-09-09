#!/bin/bash

echo "Building DDEX Builder bindings..."

# Build Node.js bindings
echo "Building Node.js bindings..."
cd bindings/node
npm run build
cd ../..

# Build Python bindings
echo "Building Python bindings..."
cd bindings/python
maturin build --release
cd ../..

# Build WASM bindings
echo "Building WASM bindings..."
cd bindings/wasm
wasm-pack build --target web --out-dir pkg
wasm-opt pkg/ddex_builder_wasm_bg.wasm -O3 -o pkg/ddex_builder_wasm_bg.wasm
cd ../..

echo "All bindings built successfully!"