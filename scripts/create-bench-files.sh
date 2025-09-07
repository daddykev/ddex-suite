#!/bin/bash
# scripts/create-bench-files.sh

echo "Setting up benchmark test files..."

# Create directories
mkdir -p benchmarks/results

# The simple_release.xml already exists and will be used for benchmarks
if [ -f "test-suite/valid/ern-4.3/simple_release.xml" ]; then
    echo "✓ Test file exists: test-suite/valid/ern-4.3/simple_release.xml"
else
    echo "✗ Test file missing - run: python test-suite/generate_test_corpus.py"
    exit 1
fi

echo "Benchmark setup complete!"