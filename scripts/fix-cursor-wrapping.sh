#!/bin/bash

echo "Fixing double Cursor wrapping..."

# Fix the double Cursor wrapping in vendor_quirks.rs
sed -i '' 's/std::io::Cursor::new(std::io::Cursor::new(/std::io::Cursor::new(/g' packages/ddex-parser/tests/vendor_quirks.rs
sed -i '' 's/))/)/g' packages/ddex-parser/tests/vendor_quirks.rs

# Fix any remaining parse calls that don't use Cursor
find packages/ddex-parser/tests -name "*.rs" -type f -exec sed -i '' \
  -e 's/parser\.parse(&xml_with_bom\[\.\.\])/parser.parse(std::io::Cursor::new(\&xml_with_bom[..]))/g' \
  {} \;

# Check for any remaining issues and fix them individually
echo "Checking for remaining non-Cursor parse calls..."

# Fix phase2_complete.rs specifically
sed -i '' 's/let result = parser\.parse(std::io::Cursor::new(xml\.as_bytes()));/let result = parser.parse(std::io::Cursor::new(xml));/g' packages/ddex-parser/tests/phase2_complete.rs

echo "Cursor wrapping fixed!"
