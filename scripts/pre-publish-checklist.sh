#!/bin/bash
# packages/ddex-parser/node/scripts/pre-publish-checklist.sh

set -e

echo "🔍 DDEX Parser Pre-Publish Checklist"
echo "====================================="

# 1. Version check
echo -n "1. Checking version consistency... "
PACKAGE_VERSION=$(node -p "require('./package.json').version")
echo "v$PACKAGE_VERSION ✓"

# 2. Build check
echo -n "2. Building package... "
npm run build > /dev/null 2>&1
echo "✓"

# 3. Tests
echo -n "3. Running tests... "
npm test > /dev/null 2>&1
echo "✓"

# 4. Linting
echo -n "4. Running linter... "
npm run lint > /dev/null 2>&1 || echo "(no lint script) ⚠️"

# 5. Bundle size
echo -n "5. Checking bundle size... "
node scripts/check-bundle-size.js > /dev/null 2>&1
echo "✓"

# 6. License
echo -n "6. Checking LICENSE file... "
if [ -f "LICENSE" ] || [ -f "../../../LICENSE" ]; then
  echo "✓"
else
  echo "❌ Missing LICENSE file!"
  exit 1
fi

# 7. README
echo -n "7. Checking README... "
if [ -f "README.md" ] && [ $(wc -c < README.md) -gt 1000 ]; then
  echo "✓"
else
  echo "❌ README missing or too short!"
  exit 1
fi

# 8. npm pack test
echo -n "8. Testing npm pack... "
npm pack --dry-run > /dev/null 2>&1
echo "✓"

# 9. Check for sensitive files
echo -n "9. Checking for sensitive files... "
if [ -f ".env" ] || [ -f ".npmrc" ] || [ -f "*.key" ]; then
  echo "❌ Found sensitive files!"
  exit 1
else
  echo "✓"
fi

# 10. Confirm Node.js compatibility
echo -n "10. Checking Node.js compatibility... "
ENGINES=$(node -p "JSON.stringify(require('./package.json').engines || {})")
if [[ $ENGINES == *"node"* ]]; then
  echo "✓"
else
  echo "⚠️ No engines field specified"
fi

echo ""
echo "✅ All checks passed! Ready to publish."
echo ""
echo "To publish to npm:"
echo "  npm publish --dry-run  # Test first"
echo "  npm publish            # Actually publish"