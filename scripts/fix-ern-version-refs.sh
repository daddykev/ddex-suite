#!/bin/bash

echo "Updating all ERNVersion references..."

# Update all references in the entire workspace
find packages -name "*.rs" -type f -exec sed -i '' \
  -e 's/ERNVersion::ERN382/ERNVersion::V3_8_2/g' \
  -e 's/ERNVersion::ERN42/ERNVersion::V4_2/g' \
  -e 's/ERNVersion::ERN43/ERNVersion::V4_3/g' \
  {} \;

echo "ERNVersion references updated!"
