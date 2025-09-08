# packages/ddex-parser/node/scripts/test-local-pack.sh
#!/bin/bash

set -e

echo "🔨 Building the package..."
npm run build

echo "📦 Creating local package..."
npm pack

echo "🧪 Testing in isolated environment..."
# Create a temporary test directory
TEST_DIR=$(mktemp -d)
cd $TEST_DIR

echo "📥 Installing package locally..."
npm init -y
npm install ../ddex-parser-*.tgz

echo "✅ Testing basic import..."
cat > test-import.js << 'EOF'
const { DDEXParser } = require('ddex-parser');
const parser = new DDEXParser();
console.log('✓ CommonJS import works');
console.log('✓ Parser version:', parser.version);
EOF

node test-import.js

echo "✅ Testing ES modules..."
cat > test-esm.mjs << 'EOF'
import { DDEXParser } from 'ddex-parser';
const parser = new DDEXParser();
console.log('✓ ESM import works');
EOF

node test-esm.mjs

echo "🎉 Local package tests passed!"
cd ..
rm -rf $TEST_DIR