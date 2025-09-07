#!/bin/bash

echo "Fixing final compilation issues..."

# 1. Fix the doc comment in lib.rs
echo "Fixing doc comment..."
sed -i '' 's|^//! DDEX Parser Core Library|/// DDEX Parser Core Library|' packages/ddex-parser/src/lib.rs

# 2. Remove duplicate ERNVersionExt import in detector.rs
echo "Removing duplicate imports..."
# Remove all instances first, then add one back
sed -i '' '/use super::version_ext::ERNVersionExt;/d' packages/ddex-parser/src/parser/detector.rs
# Add it back once at the top
sed -i '' '1a\
use super::version_ext::ERNVersionExt;
' packages/ddex-parser/src/parser/detector.rs

# 3. Fix the double ddex_core references
echo "Fixing double ddex_core references..."
find packages/ddex-parser/src -name "*.rs" -type f -exec sed -i '' \
  -e 's/ddex_core::ddex_core::/ddex_core::/g' \
  {} \;

# 4. Determine the correct type names from core
echo "Finding correct type names..."
# Get just the first struct name from each file
FLAT_MESSAGE_TYPE=$(grep "^pub struct.*Message" packages/core/src/models/flat/message.rs | head -1 | awk '{print $3}')
GRAPH_MESSAGE_TYPE=$(grep "^pub struct.*Message" packages/core/src/models/graph/message.rs | head -1 | awk '{print $3}')

echo "Using flat type: $FLAT_MESSAGE_TYPE"
echo "Using graph type: $GRAPH_MESSAGE_TYPE"

# 5. Update all references to use the correct types
echo "Updating type references..."
if [ "$FLAT_MESSAGE_TYPE" = "ParsedERNMessage" ]; then
    find packages/ddex-parser/src -name "*.rs" -type f -exec sed -i '' \
      -e 's/ddex_core::models::flat::Message/ddex_core::models::flat::ParsedERNMessage/g' \
      {} \;
fi

if [ "$GRAPH_MESSAGE_TYPE" = "ERNMessage" ]; then
    find packages/ddex-parser/src -name "*.rs" -type f -exec sed -i '' \
      -e 's/ddex_core::models::graph::NewReleaseMessage/ddex_core::models::graph::ERNMessage/g' \
      {} \;
fi

# 6. Check for models:: references that need ddex_core:: prefix
echo "Fixing models:: references..."
# In lib.rs specifically, make sure models references are qualified
sed -i '' 's/models::flat::ParsedERNMessage/ddex_core::models::flat::ParsedERNMessage/g' packages/ddex-parser/src/lib.rs
sed -i '' 's/models::graph::ERNMessage/ddex_core::models::graph::ERNMessage/g' packages/ddex-parser/src/lib.rs

echo "All fixes applied!"
