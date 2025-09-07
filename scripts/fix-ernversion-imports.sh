#!/bin/bash

echo "Fixing ERNVersion imports in tests..."

# 1. Fix imports in parser/tests.rs
sed -i '' 's/use crate::parser::detector::{ERNVersion, VersionDetector};/use crate::parser::detector::VersionDetector;\
    use ddex_core::models::versions::ERNVersion;/' packages/ddex-parser/src/parser/tests.rs

# 2. Fix any remaining ERNVersion imports from detector
find packages/ddex-parser -name "*.rs" -type f -exec sed -i '' \
  -e 's/crate::parser::detector::ERNVersion/ddex_core::models::versions::ERNVersion/g' \
  -e 's/parser::detector::ERNVersion/ddex_core::models::versions::ERNVersion/g' \
  {} \;

# 3. Update test files to use ddex_core for ERNVersion
find packages/ddex-parser/tests -name "*.rs" -type f -exec sed -i '' \
  -e 's/, ddex_core::models::versions::ERNVersion//' \
  -e '/^use ddex_parser::/a\
use ddex_core::models::versions::ERNVersion;' \
  {} \;

echo "ERNVersion imports fixed!"
