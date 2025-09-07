#!/bin/bash

echo "Updating parser to use core's ERNVersion..."

# Find all files in parser that reference ERNVersion
find packages/ddex-parser/src -name "*.rs" -type f -exec sed -i '' \
  -e 's|use crate::parser::detector::ERNVersion|use ddex_core::models::versions::ERNVersion|g' \
  -e 's|crate::parser::detector::ERNVersion|ddex_core::models::versions::ERNVersion|g' \
  -e 's|use super::detector::ERNVersion|use ddex_core::models::versions::ERNVersion|g' \
  {} \;

# If detector.rs has the enum definition, comment it out and import from core
if [ -f "packages/ddex-parser/src/parser/detector.rs" ]; then
    # Add import at the top
    sed -i '' '1i\
use ddex_core::models::versions::ERNVersion;\
' packages/ddex-parser/src/parser/detector.rs
    
    # Comment out the local enum definition
    sed -i '' '/^pub enum ERNVersion/,/^}/s/^/\/\/ /' packages/ddex-parser/src/parser/detector.rs
fi

echo "Parser imports updated!"
