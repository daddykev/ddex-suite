# packages/ddex-parser/bindings/python/build.sh
#!/bin/bash
set -e

echo "Building ddex-parser Python package..."

# Clean previous builds
rm -rf dist/ build/ *.egg-info

# Build with maturin
maturin build --release --strip

# Build source distribution
maturin sdist

echo "Build complete! Wheels in dist/"
ls -la dist/