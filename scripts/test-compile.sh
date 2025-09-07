#!/bin/bash
# scripts/test-compile.sh

echo "Testing compilation..."

cd core

# Just try to build first
echo "Building core library..."
cargo build --features bench

if [ $? -eq 0 ]; then
    echo "✓ Core library compiled successfully"
    
    echo "Building benchmarks..."
    cargo build --benches
    
    if [ $? -eq 0 ]; then
        echo "✓ Benchmarks compiled successfully"
        echo "You can now run: cargo bench"
    else
        echo "✗ Benchmark compilation failed"
    fi
else
    echo "✗ Core library compilation failed"
fi