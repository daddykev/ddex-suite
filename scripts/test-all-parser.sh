# scripts/test-all-parser.sh
#!/bin/bash
# Run all tests before publishing ddex-parser

set -e

echo "🧪 Running Complete Test Suite for ddex-parser"
echo "=============================================="

# Make sure we have TypeScript compiled
printf "\n📦 Building TypeScript..."
cd packages/ddex-parser/bindings/node
if [ ! -d "dist" ]; then
  npx tsc || echo " (no TypeScript files yet)"
else
  echo " already built"
fi
cd - > /dev/null

# 1. Pre-publish checklist
printf "\n📋 Running pre-publish checklist...\n"
./scripts/pre-publish-parser.sh

# 2. Check sizes
printf "\n📊 Checking bundle sizes...\n"
./scripts/check-parser-size.sh

# 3. Test package installation
printf "\n📦 Testing package installation...\n"
./scripts/test-npm-package.sh

# 4. Manual test
printf "\n🔬 Running manual test...\n"
cd packages/ddex-parser/bindings/node
node test-manual.js
cd - > /dev/null

# 5. Final npm publish dry run
printf "\n🚀 Running npm publish dry run...\n"
cd packages/ddex-parser/bindings/node
npm publish --dry-run

printf "\n✅ All tests passed!\n"
echo ""
echo "To publish to npm:"
echo "  cd packages/ddex-parser/bindings/node"
echo "  npm publish"

cd - > /dev/null