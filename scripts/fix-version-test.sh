#!/bin/bash

echo "Fixing version detection test..."

# Update the test to use proper namespace
sed -i '' '/test_version_detection_382/,/^    \}/ {
    s|"<ern:ReleaseList/>"|"<ern:ReleaseList xmlns:ern=\"http://ddex.net/xml/ern/382\"/>"|
}' packages/ddex-parser/src/parser/tests.rs

echo "Test fixed!"
