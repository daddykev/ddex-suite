#!/bin/bash
# packages/ddex-parser/bindings/wasm/build.sh

set -e

echo "Building WASM module..."
wasm-pack build \
    --target web \
    --out-dir pkg \
    --release \
    -- --features wasm

echo "Optimizing with wasm-opt..."
wasm-opt \
    -Oz \
    --enable-simd \
    --enable-bulk-memory \
    pkg/*_bg.wasm \
    -o pkg/ddex_parser_optimized.wasm

# Check size
SIZE=$(stat -f%z pkg/ddex_parser_optimized.wasm 2>/dev/null || stat -c%s pkg/ddex_parser_optimized.wasm)
SIZE_KB=$((SIZE / 1024))

echo "WASM size: ${SIZE_KB}KB"

if [ $SIZE_KB -gt 500 ]; then
    echo "Warning: WASM size exceeds 500KB target!"
    exit 1
fi

mv pkg/ddex_parser_optimized.wasm pkg/ddex_parser_bg.wasm