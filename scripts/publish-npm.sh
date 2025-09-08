#!/bin/bash
# scripts/publish-npm.sh

set -e

echo "Building all targets..."
cd packages/ddex-parser/bindings/node
npm run build

echo "Running tests..."
npm test

echo "Checking package size..."
npm pack --dry-run

echo "Publishing to npm..."
npm publish --access public --tag next

echo "Published @ddex-suite/parser successfully!"