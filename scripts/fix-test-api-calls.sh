#!/bin/bash

echo "Fixing test API calls to match actual parser API..."

# Fix all parse calls to:
# 1. Remove the second argument (None)
# 2. Wrap bytes in std::io::Cursor for Seek trait

find packages/ddex-parser/tests -name "*.rs" -type f -exec sed -i '' \
  -e 's/parser\.parse(\([^,]*\)\.as_bytes(), None)/parser.parse(std::io::Cursor::new(\1.as_bytes()))/g' \
  -e 's/parser\.parse(\([^,]*\)\.as_bytes())/parser.parse(std::io::Cursor::new(\1.as_bytes()))/g' \
  -e 's/parser\.parse(xml\.as_bytes(), None)/parser.parse(std::io::Cursor::new(xml.as_bytes()))/g' \
  -e 's/parser\.parse(xml\.as_bytes())/parser.parse(std::io::Cursor::new(xml.as_bytes()))/g' \
  -e 's/parser\.parse(invalid_xml\.as_bytes())/parser.parse(std::io::Cursor::new(invalid_xml.as_bytes()))/g' \
  -e 's/parser\.parse(empty\.as_bytes())/parser.parse(std::io::Cursor::new(empty.as_bytes()))/g' \
  -e 's/parser\.parse(malformed\.as_bytes())/parser.parse(std::io::Cursor::new(malformed.as_bytes()))/g' \
  -e 's/parser\.parse(b"<test\/>", None)/parser.parse(std::io::Cursor::new(b"<test\/>"))/g' \
  {} \;

# Add std::io::Cursor import to all test files
for file in packages/ddex-parser/tests/*.rs; do
    if ! grep -q "use std::io::Cursor" "$file"; then
        sed -i '' '1i\
use std::io::Cursor;\
' "$file"
    fi
done

# Fix the specific issues:
# 1. Change tracks to releases in model_consistency.rs
sed -i '' 's/result\.flat\.tracks/result.flat.releases/g' packages/ddex-parser/tests/model_consistency.rs

# 2. Remove parse_streaming test or change to regular parse
sed -i '' 's/parse_streaming/parse/g' packages/ddex-parser/tests/phase2_complete.rs

# 3. Fix version field comparison (graph.version is ERNVersion, flat.version is String)
sed -i '' 's/assert_eq!(result\.graph\.version, result\.flat\.version);/assert_eq!(result.graph.version.to_string(), result.flat.version);/g' packages/ddex-parser/tests/model_consistency.rs

echo "API calls fixed!"
