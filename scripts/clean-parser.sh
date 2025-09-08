# scripts/clean-parser.sh
#!/bin/bash
# Clean the parser package before rebuild

echo "🧹 Cleaning ddex-parser package"

cd packages/ddex-parser/bindings/node

# Remove build artifacts
rm -rf dist/
rm -f *.tgz
rm -f LICENSE  # Remove copied LICENSE

# Remove test artifacts
rm -rf node_modules/

echo "✅ Cleaned!"
cd -