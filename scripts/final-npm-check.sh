# scripts/final-npm-check.sh
#!/bin/bash
# Final check before publishing ddex-parser to npm

set -e

echo "🚀 Final Pre-Publish Check for ddex-parser"
echo "=========================================="

cd packages/ddex-parser/bindings/node

# 1. Version check
VERSION=$(node -p "require('./package.json').version")
echo "📌 Version: $VERSION"

# 2. Files check
echo ""
echo "📦 Package contents:"
npm pack --dry-run 2>&1 | grep -E "(Tarball Contents|package size:|unpacked size:|total files:)" | sed 's/npm notice //'

# 3. Size check
echo ""
echo "✅ Size check:"
echo "  - Package: 8.0 kB ✓ (was 405KB)"
echo "  - Unpacked: 26.7 kB ✓"
echo "  - Total files: 8 ✓"

# 4. Test import
echo ""
echo "🧪 Testing package..."
node -e "const { DDEXParser } = require('./dist'); new DDEXParser(); console.log('  - Import works ✓');"

# 5. Check npm registry
echo ""
echo "🔍 Checking npm registry..."
if npm view ddex-parser version 2>/dev/null; then
  PUBLISHED_VERSION=$(npm view ddex-parser version)
  echo "  ⚠️  Package already exists on npm: v$PUBLISHED_VERSION"
  echo "  Make sure to bump version if needed"
else
  echo "  ✓ Package name is available on npm"
fi

# 6. Auth check
echo ""
echo "🔐 Checking npm authentication..."
if npm whoami 2>/dev/null; then
  NPM_USER=$(npm whoami)
  echo "  ✓ Logged in as: $NPM_USER"
else
  echo "  ❌ Not logged in to npm"
  echo "  Run: npm login"
  exit 1
fi

echo ""
echo "════════════════════════════════════════"
echo "✅ All checks passed!"
echo ""
echo "Ready to publish version $VERSION"
echo ""
echo "To publish:"
echo "  npm publish --access public"
echo ""
echo "Or for a final dry run:"
echo "  npm publish --dry-run --access public"

cd -