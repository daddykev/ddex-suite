# scripts/build-python.sh
#!/bin/bash
set -e

echo "🐍 Building Python bindings for ddex-parser"
echo "=========================================="

cd packages/ddex-parser/bindings/python

# Install dependencies
echo "📦 Installing dependencies..."
pip install -U pip setuptools wheel maturin

# Build the package
echo "🔨 Building package..."
maturin build --release

# Test locally
echo "🧪 Testing package..."
./test-local.sh

# Check the wheel
echo "📊 Package info:"
pip show ddex-parser || echo "Not installed yet"

echo ""
echo "✅ Python package built successfully!"
echo ""
echo "To publish to PyPI:"
echo "  1. Test on TestPyPI first:"
echo "     maturin publish --repository testpypi"
echo "  2. Then publish to PyPI:"
echo "     maturin publish"

cd -