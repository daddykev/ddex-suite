#!/bin/bash
# scripts/test-versions.sh

echo "Testing DDEX ERN Version Support..."
echo "=================================="

# First create the test files
echo "Creating test files..."
bash scripts/create-version-test-files.sh

# Navigate to core directory
cd core

# Run version detection tests
echo ""
echo "Testing version detection..."
cargo test version_detection --lib -- --nocapture

# Run version-specific model tests
echo ""
echo "Testing version-specific models..."
cargo test models::versions --lib -- --nocapture

# Run vendor quirk tests
echo ""
echo "Testing vendor quirks..."
cargo test vendor_quirks -- --nocapture

# Run migration tests
echo ""
echo "Testing version migration..."
cargo test version_migration -- --nocapture

# Run all parser tests
echo ""
echo "Running all parser tests..."
cargo test parser -- --nocapture

# Run with verbose output to see what's happening
echo ""
echo "Running comprehensive test with trace logging..."
RUST_LOG=debug cargo test --test vendor_quirks -- --nocapture

echo ""
echo "=================================="
echo "Version Support Testing Complete!"