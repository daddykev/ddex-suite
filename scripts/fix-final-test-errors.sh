#!/bin/bash

echo "Fixing final test compilation errors..."

# 1. Fix vendor_quirks.rs - missing closing parentheses
echo "Fixing vendor_quirks.rs syntax errors..."
sed -i '' 's/parser.parse(std::io::Cursor::new(xml.as_bytes());/parser.parse(std::io::Cursor::new(xml.as_bytes()));/g' packages/ddex-parser/tests/vendor_quirks.rs

# 2. Fix phase2_complete.rs - missing Cursor wrapper
echo "Fixing phase2_complete.rs..."
sed -i '' 's/let result = parser.parse(xml.as_bytes());/let result = parser.parse(std::io::Cursor::new(xml));/g' packages/ddex-parser/tests/phase2_complete.rs

# 3. Clean up unused imports with cargo fix
echo "Cleaning up unused imports..."
cargo fix --tests -p ddex-parser --allow-dirty 2>/dev/null || true

echo "Fixes applied!"
