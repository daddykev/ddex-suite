#!/bin/bash

echo "Fixing test imports and paths..."

# 1. Fix all imports from ddex_parser_core to ddex_parser
echo "Updating imports from ddex_parser_core to ddex_parser..."
find packages/ddex-parser/tests -name "*.rs" -type f -exec sed -i '' \
  -e 's/use ddex_parser_core::/use ddex_parser::/g' \
  -e 's/ddex_parser_core::/ddex_parser::/g' \
  {} \;

# 2. Fix file paths - add ../ to test-suite paths
echo "Fixing test file paths..."
find packages/ddex-parser/tests -name "*.rs" -type f -exec sed -i '' \
  -e 's|include_str!("../../test-suite/|include_str!("../../../test-suite/|g' \
  {} \;

# 3. Fix ERNVersion imports in tests
echo "Fixing ERNVersion imports..."
find packages/ddex-parser/tests -name "*.rs" -type f -exec sed -i '' \
  -e 's/parser::detector::ERNVersion/ddex_core::models::versions::ERNVersion/g' \
  {} \;

# 4. Fix models imports
echo "Fixing models imports..."
find packages/ddex-parser/tests -name "*.rs" -type f -exec sed -i '' \
  -e 's/ddex_parser::models::/ddex_core::models::/g' \
  {} \;

echo "Test fixes applied!"
