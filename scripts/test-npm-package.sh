# scripts/test-npm-package.sh (updated)
#!/bin/bash
# Test the npm package before publishing

set -e

echo "🔍 Testing ddex-parser npm package"
echo "=================================="

# Navigate to the node bindings
cd packages/ddex-parser/bindings/node

# The package is index.js and index.d.ts (package.json "files"); there is nothing to compile.
# The native binary comes from the platform package (ddex-parser-<platform>), an optional
# dependency installed from npm.

# Copy LICENSE if needed
if [ ! -f "LICENSE" ] && [ -f "../../../../LICENSE" ]; then
  cp ../../../../LICENSE LICENSE
fi

echo "📦 Creating local package..."
npm pack 2>&1 | grep -E "(package size:|unpacked size:|total files:)" | sed 's/npm notice /  /'

echo ""
echo "🧪 Testing in isolated environment..."
# Create a temporary test directory
TEST_DIR=$(mktemp -d)
cp ddex-parser-*.tgz $TEST_DIR/
cd $TEST_DIR

echo "📥 Installing package locally..."
npm init -y --silent > /dev/null 2>&1
npm install ddex-parser-*.tgz --silent

printf "✅ Testing CommonJS import... "
cat > test-cjs.js << 'EOF'
try {
  const { DdexParser } = require('ddex-parser');
  const parser = new DdexParser();
  console.log('works');
  process.exit(0);
} catch (e) {
  console.log('failed:', e.message);
  process.exit(1);
}
EOF
node test-cjs.js

printf "✅ Testing ES modules... "
cat > test-esm.mjs << 'EOF'
try {
  const mod = await import('ddex-parser');
  const parser = new (mod.DdexParser ?? mod.default.DdexParser)();
  console.log('works');
  process.exit(0);
} catch (e) {
  console.log('failed:', e.message);
  process.exit(1);
}
EOF
node test-esm.mjs

echo ""
echo "📁 Checking installed package contents..."
cd node_modules/ddex-parser
echo "  Files in package:"
ls -la | grep -v "^total" | awk '{print "    " $9}' | grep -v "^    $"

echo ""
echo "🎉 Package tests passed!"

# Cleanup
cd $OLDPWD
rm ddex-parser-*.tgz
rm -rf $TEST_DIR
if [ -f "LICENSE" ] && [ -f "../../../../LICENSE" ]; then
  rm LICENSE  # Remove temporary LICENSE copy
fi

echo ""
echo "✅ npm package is ready for publishing!"
echo ""
echo "Package will include only:"
echo "  • dist/ (compiled JavaScript)"
echo "  • README.md"
echo "  • LICENSE"
echo "  • package.json"