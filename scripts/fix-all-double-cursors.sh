#!/bin/bash

echo "Fixing all double Cursor wrappings..."

# Find and fix all double Cursor wrappings
find packages/ddex-parser/tests -name "*.rs" -type f | while read file; do
    # Fix double Cursor wrapping patterns
    sed -i '' 's/std::io::Cursor::new(std::io::Cursor::new(\([^)]*\)))/std::io::Cursor::new(\1)/g' "$file"
done

# Also check for any remaining issues in specific files
echo "Checking model_consistency.rs..."
sed -i '' 's/parser\.parse(std::io::Cursor::new(std::io::Cursor::new(xml\.as_bytes())))/parser.parse(std::io::Cursor::new(xml.as_bytes()))/g' packages/ddex-parser/tests/model_consistency.rs

echo "Checking vendor_quirks.rs..."
sed -i '' 's/parser\.parse(std::io::Cursor::new(std::io::Cursor::new(xml\.as_bytes())))/parser.parse(std::io::Cursor::new(xml.as_bytes()))/g' packages/ddex-parser/tests/vendor_quirks.rs

# Show what parse calls look like now in one of the problem files
echo "Sample of parse calls in model_consistency.rs:"
grep "parser.parse" packages/ddex-parser/tests/model_consistency.rs | head -2

echo "All double Cursor wrappings should be fixed!"
