#!/bin/bash

echo "Fixing all test imports..."

# Add ERNVersion import to all test files that need it
for file in packages/ddex-parser/tests/*.rs; do
    # Check if the file uses ERNVersion but doesn't import it
    if grep -q "ERNVersion::" "$file" && ! grep -q "use ddex_core::models::versions::ERNVersion" "$file"; then
        echo "Adding ERNVersion import to $(basename $file)"
        # Add the import at the beginning of the file
        sed -i '' '1i\
use ddex_core::models::versions::ERNVersion;
' "$file"
    fi
done

# Also fix any remaining DDEXParser imports
find packages/ddex-parser/tests -name "*.rs" -type f -exec sed -i '' \
  -e 's/use ddex_parser::DDEXParser;/use ddex_parser::DDEXParser;\
use ddex_core::models::versions::ERNVersion;/' \
  {} \;

# Remove duplicate imports
for file in packages/ddex-parser/tests/*.rs; do
    # Remove consecutive duplicate lines
    awk '!seen[$0]++ || NF==0' "$file" > "$file.tmp" && mv "$file.tmp" "$file"
done

echo "Test imports fixed!"
