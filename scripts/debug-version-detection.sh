#!/bin/bash

echo "Debugging Version Detection"
echo "==========================="

# Run just the failing test with output
cd packages/ddex-parser
echo "Running failing test with backtrace:"
RUST_BACKTRACE=1 cargo test test_version_detection_382 -- --nocapture

echo ""
echo "Checking version detection patterns..."
grep -n "3.8.2\|382\|3_8_2" src/parser/detector.rs
grep -n "namespace" src/parser/detector.rs

