# scripts/test-python-bindings-fixed.sh
#!/bin/bash
set -e

echo "🧪 Testing Python Bindings for DDEX Parser"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Handle conda/virtual environment conflict
if [ ! -z "$CONDA_PREFIX" ]; then
    echo -e "${YELLOW}Detected Conda environment. Temporarily unsetting for maturin...${NC}"
    ORIGINAL_CONDA_PREFIX="$CONDA_PREFIX"
    ORIGINAL_CONDA_DEFAULT_ENV="$CONDA_DEFAULT_ENV"
    unset CONDA_PREFIX
    unset CONDA_DEFAULT_ENV
fi

# Check prerequisites
echo -e "\n${YELLOW}Checking prerequisites...${NC}"

# Check Python version
python_version=$(python3 --version 2>&1 | grep -oE '[0-9]+\.[0-9]+')
echo "✓ Python version: $python_version"

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ Rust not installed${NC}"
    echo "  Install from: https://rustup.rs/"
    exit 1
fi
echo "✓ Rust installed: $(rustc --version)"

# Navigate to Python bindings
cd packages/ddex-parser/bindings/python

# Clean up any existing virtual environment
if [ -d "test-env" ]; then
    echo "Cleaning up existing test environment..."
    rm -rf test-env
fi

# Create and activate virtual environment
echo -e "\n${YELLOW}Setting up test environment...${NC}"
python3 -m venv test-env
source test-env/bin/activate

# Install dependencies
pip install -U pip setuptools wheel
pip install maturin pytest pytest-asyncio pytest-benchmark black mypy ruff
pip install pandas numpy  # For DataFrame tests

# Build the package with maturin
echo -e "\n${YELLOW}Building package with maturin...${NC}"
maturin develop --release

# Run tests
echo -e "\n${YELLOW}Running test suite...${NC}"

# 1. Basic import test
echo -e "\n${GREEN}Test 1: Basic Import${NC}"
python3 -c "
from ddex_parser import DDEXParser, __version__
print(f'✓ Import successful')
print(f'✓ Version: {__version__}')
"

# 2. Create parser test
echo -e "\n${GREEN}Test 2: Create Parser Instance${NC}"
python3 -c "
from ddex_parser import DDEXParser
parser = DDEXParser()
print(f'✓ Parser created: {parser}')
"

# 3. Parse XML test
echo -e "\n${GREEN}Test 3: Parse Sample XML${NC}"
python3 -c "
from ddex_parser import DDEXParser

xml = '''<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<ern:NewReleaseMessage xmlns:ern=\"http://ddex.net/xml/ern/43\">
    <MessageHeader>
        <MessageId>MSG001</MessageId>
    </MessageHeader>
</ern:NewReleaseMessage>'''

parser = DDEXParser()
result = parser.parse(xml)
print(f'✓ Parse successful')
print(f'  Message ID: {result.message_id}')
print(f'  Version: {result.version}')
print(f'  Releases: {result.release_count}')
"

# 4. Async parse test
echo -e "\n${GREEN}Test 4: Async Parsing${NC}"
python3 -c "
import asyncio
from ddex_parser import DDEXParser

async def test_async():
    xml = '<ern:NewReleaseMessage xmlns:ern=\"http://ddex.net/xml/ern/43\"/>'
    parser = DDEXParser()
    result = await parser.parse_async(xml)
    print(f'✓ Async parse successful')
    print(f'  Message ID: {result.message_id}')
    return result

asyncio.run(test_async())
"

# 5. Version detection test
echo -e "\n${GREEN}Test 5: Version Detection${NC}"
python3 -c "
from ddex_parser import DDEXParser

parser = DDEXParser()
test_cases = [
    ('<ern:NewReleaseMessage xmlns:ern=\"http://ddex.net/xml/ern/43\">', '4.3'),
    ('<ern:NewReleaseMessage xmlns:ern=\"http://ddex.net/xml/ern/42\">', '4.2'),
    ('<ern:NewReleaseMessage xmlns:ern=\"http://ddex.net/xml/ern/382\">', '3.8.2'),
]

for xml, expected in test_cases:
    version = parser.detect_version(xml)
    assert version == expected, f'Expected {expected}, got {version}'
    print(f'✓ Detected {expected} correctly')
"

# More tests...
echo -e "\n${GREEN}✅ Core tests completed!${NC}"

# Clean up
deactivate

# Restore conda environment variables if they were set
if [ ! -z "$ORIGINAL_CONDA_PREFIX" ]; then
    export CONDA_PREFIX="$ORIGINAL_CONDA_PREFIX"
    export CONDA_DEFAULT_ENV="$ORIGINAL_CONDA_DEFAULT_ENV"
    echo -e "\n${YELLOW}Restored Conda environment variables${NC}"
fi

cd -

echo -e "\n${GREEN}✅ All tests completed!${NC}"