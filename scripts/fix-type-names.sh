#!/bin/bash

echo "Fixing type names based on actual core models..."

# Look for the actual message types in core
FLAT_MESSAGE_TYPE=$(grep "pub struct.*Message" packages/core/src/models/flat/message.rs | awk '{print $3}')
GRAPH_MESSAGE_TYPE=$(grep "pub struct.*Message" packages/core/src/models/graph/message.rs | head -1 | awk '{print $3}')

echo "Found flat message type: $FLAT_MESSAGE_TYPE"
echo "Found graph message type: $GRAPH_MESSAGE_TYPE"

# Update references if we found them
if [ ! -z "$FLAT_MESSAGE_TYPE" ]; then
    find packages/ddex-parser/src -name "*.rs" -type f -exec sed -i '' \
      -e "s/ddex_core::models::flat::Message/ddex_core::models::flat::${FLAT_MESSAGE_TYPE}/g" \
      {} \;
fi

if [ ! -z "$GRAPH_MESSAGE_TYPE" ]; then
    find packages/ddex-parser/src -name "*.rs" -type f -exec sed -i '' \
      -e "s/ddex_core::models::graph::NewReleaseMessage/ddex_core::models::graph::${GRAPH_MESSAGE_TYPE}/g" \
      {} \;
fi

echo "Type names fixed!"
