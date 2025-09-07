#!/bin/bash

# Update Rust imports in ddex-parser
find packages/ddex-parser/src -name "*.rs" -type f -exec sed -i '' \
  -e 's/use crate::models/use ddex_core::models/g' \
  -e 's/use crate::error/use ddex_core::error/g' \
  -e 's/crate::models::/ddex_core::models::/g' \
  -e 's/crate::error::/ddex_core::error::/g' \
  {} \;

# Update test imports
find packages/ddex-parser/tests -name "*.rs" -type f -exec sed -i '' \
  -e 's/use ddex_parser::models/use ddex_core::models/g' \
  -e 's/ddex_parser::models::/ddex_core::models::/g' \
  {} \;

# Update benchmark imports
find packages/ddex-parser/benches -name "*.rs" -type f -exec sed -i '' \
  -e 's/use ddex_parser::models/use ddex_core::models/g' \
  -e 's/ddex_parser::models::/ddex_core::models::/g' \
  {} \;

echo "Import paths updated!"
