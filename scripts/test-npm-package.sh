# scripts/test-npm-package.sh
#!/bin/bash
# Test the npm package before publishing

set -e

echo "🔍 Testing ddex-parser npm package"
echo "=================================="

# Navigate to the node bindings
cd packages/ddex-parser/bindings/node

# Check if dist exists
if [ ! -d "dist" ]; then
  echo "⚠️  No dist folder found. Building TypeScript..."
  npm run build:ts 2>/dev/null || npx tsc
fi

echo "📦 Creating local package..."
npm pack

echo "🧪 Testing in isolated environment..."
# Create a temporary test directory
TEST_DIR=$(mktemp -d)
cp ddex-parser-*.tgz $TEST_DIR/
cd $TEST_DIR

echo "📥 Installing package locally..."
npm init -y --silent
npm install ddex-parser-*.tgz --silent

printf "✅ Testing CommonJS import... "
cat > test-cjs.js << 'EOF'
try {
  const { DDEXParser } = require('ddex-parser');
  const parser = new DDEXParser();
  console.log('works');
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
  const parser = new mod.DDEXParser();
  console.log('works');
} catch (e) {
  console.log('failed:', e.message);
  process.exit(1);
}
EOF
node test-esm.mjs

echo "🎉 Package tests passed!"

# Cleanup
cd - > /dev/null
rm ddex-parser-*.tgz
rm -rf $TEST_DIR

echo ""
echo "✅ npm package is ready for publishing!"