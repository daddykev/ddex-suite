#!/bin/bash

echo "Fixing remaining double ddex_core references..."

# Fix all double ddex_core:: references more aggressively
find packages/ddex-parser -name "*.rs" -type f | while read file; do
    # Replace any occurrence of ddex_core::ddex_core:: with ddex_core::
    sed -i '' 's/ddex_core::ddex_core::/ddex_core::/g' "$file"
done

# Specifically fix lib.rs which seems to have issues
sed -i '' 's/ddex_core::ddex_core::/ddex_core::/g' packages/ddex-parser/src/lib.rs

echo "Double references fixed!"
