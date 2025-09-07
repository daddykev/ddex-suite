#!/bin/bash

echo "Fixing remaining compilation issues..."

# 1. Fix version_ext module location and imports
echo "Fixing version_ext module..."
sed -i '' '/^mod version_ext;/d' packages/ddex-parser/src/parser/detector.rs
sed -i '' '/^use version_ext::ERNVersionExt;/d' packages/ddex-parser/src/parser/detector.rs

# Add to parser/mod.rs if not already there
if ! grep -q "pub mod version_ext;" packages/ddex-parser/src/parser/mod.rs; then
    echo "pub mod version_ext;" >> packages/ddex-parser/src/parser/mod.rs
fi

# Add import to detector.rs
sed -i '' '1a\
use super::version_ext::ERNVersionExt;
' packages/ddex-parser/src/parser/detector.rs

# 2. Restore VersionDetector struct in detector.rs
echo "Restoring VersionDetector..."
cat >> packages/ddex-parser/src/parser/detector.rs << 'RUST'

pub struct VersionDetector;

impl VersionDetector {
    pub fn detect<R: std::io::Read>(reader: R) -> crate::error::Result<ERNVersion> {
        let mut buf = Vec::new();
        let mut reader = std::io::BufReader::new(reader);
        use std::io::Read;
        reader.read_to_end(&mut buf)?;
        
        let xml_str = String::from_utf8_lossy(&buf);
        
        // Check for version in namespace
        if xml_str.contains("http://ddex.net/xml/ern/382") {
            Ok(ERNVersion::V3_8_2)
        } else if xml_str.contains("http://ddex.net/xml/ern/42") {
            Ok(ERNVersion::V4_2)
        } else if xml_str.contains("http://ddex.net/xml/ern/43") {
            Ok(ERNVersion::V4_3)
        } else {
            // Default to latest
            Ok(ERNVersion::V4_3)
        }
    }
}
RUST

# 3. Fix models imports in lib.rs
echo "Fixing models imports in lib.rs..."
sed -i '' '1a\
use ddex_core::models;
' packages/ddex-parser/src/lib.rs

# Also add specific imports
sed -i '' '/^use ddex_core::models;/a\
use ddex_core::models::flat;
use ddex_core::models::graph;
' packages/ddex-parser/src/lib.rs

# 4. Fix ParsedERNMessage references
echo "Fixing ParsedERNMessage references..."
# These might need to be fully qualified or we need to check what the actual type name is
find packages/ddex-parser/src -name "*.rs" -type f -exec sed -i '' \
  -e 's/models::flat::ParsedERNMessage/ddex_core::models::flat::Message/g' \
  -e 's/models::graph::ERNMessage/ddex_core::models::graph::NewReleaseMessage/g' \
  {} \;

# 5. Check what types actually exist in core
echo "Checking core model types..."
ls packages/core/src/models/flat/
ls packages/core/src/models/graph/

echo "Fixes applied!"
