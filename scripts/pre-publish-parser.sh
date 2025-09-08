# scripts/pre-publish-parser.sh
#!/bin/bash
# Pre-publish checklist for ddex-parser npm package

set -e

echo "🔍 DDEX Parser Pre-Publish Checklist"
echo "====================================="

cd packages/ddex-parser/bindings/node

# 1. Version check - handle workspace inheritance
printf "1. Checking version consistency... "
PACKAGE_VERSION=$(node -p "require('./package.json').version")

# Check if Cargo.toml uses workspace version
if grep -q "version.workspace = true" ../../Cargo.toml; then
  # Get version from root workspace
  CARGO_VERSION=$(grep "^version" ../../../../Cargo.toml | head -1 | cut -d'"' -f2)
else
  # Get version from local Cargo.toml
  CARGO_VERSION=$(grep "^version" ../../Cargo.toml | head -1 | cut -d'"' -f2)
fi

if [ "$PACKAGE_VERSION" != "$CARGO_VERSION" ]; then
  echo "❌ Version mismatch!"
  echo "   package.json: $PACKAGE_VERSION"
  echo "   Cargo.toml (workspace): $CARGO_VERSION"
  echo ""
  echo "   To fix: Update package.json to match workspace version"
  exit 1
fi
echo "v$PACKAGE_VERSION ✓"

# 2. Build check
printf "2. Checking build output... "
if [ -d "dist" ] && [ -f "dist/index.js" ]; then
  echo "✓"
else
  echo "⚠️ No dist folder - run build first"
fi

# Rest of the checks...
printf "3. Checking README... "
if [ -f "README.md" ]; then
  README_SIZE=$(wc -c < README.md)
  if [ $README_SIZE -gt 1000 ]; then
    echo "✓"
  else
    echo "❌ README too short!"
    exit 1
  fi
else
  echo "❌ Missing README.md!"
  exit 1
fi

printf "4. Checking for sensitive files... "
if ls .env .npmrc *.key *.pem 2>/dev/null | grep -q .; then
  echo "❌ Found sensitive files!"
  exit 1
fi
echo "✓"

printf "5. Testing npm pack... "
npm pack --dry-run > /dev/null 2>&1 || {
  echo "❌ npm pack failed!"
  exit 1
}
echo "✓"

echo ""
echo "✅ All checks passed!"
echo ""
echo "Next steps:"
echo "  1. npm publish --dry-run"
echo "  2. npm publish"

cd -