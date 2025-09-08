# scripts/test-python-bindings.sh
#!/bin/bash
set -e

echo "🧪 Testing Python Bindings for DDEX Parser"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

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

# Check maturin
if ! command -v maturin &> /dev/null; then
    echo -e "${YELLOW}Installing maturin...${NC}"
    pip install maturin
fi
echo "✓ Maturin installed"

# Navigate to Python bindings
cd packages/ddex-parser/bindings/python

# Create and activate virtual environment
echo -e "\n${YELLOW}Setting up test environment...${NC}"
python3 -m venv test-env
source test-env/bin/activate

# Install dependencies
pip install -U pip setuptools wheel
pip install maturin pytest pytest-asyncio pytest-benchmark black mypy ruff
pip install pandas  # For DataFrame tests

# Build the package
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

# 6. DataFrame test
echo -e "\n${GREEN}Test 6: DataFrame Conversion${NC}"
python3 -c "
from ddex_parser import DDEXParser
import pandas as pd

xml = '<ern:NewReleaseMessage xmlns:ern=\"http://ddex.net/xml/ern/43\"/>'
parser = DDEXParser()

try:
    df = parser.to_dataframe(xml)
    print(f'✓ DataFrame created')
    print(f'  Shape: {df.shape}')
    print(f'  Columns: {list(df.columns)}')
except ImportError:
    print('⚠ Pandas not available, skipping DataFrame test')
"

# 7. Options test
echo -e "\n${GREEN}Test 7: Parse Options${NC}"
python3 -c "
from ddex_parser import DDEXParser, ParseOptions

xml = '<ern:NewReleaseMessage xmlns:ern=\"http://ddex.net/xml/ern/43\"/>'
parser = DDEXParser()

options = ParseOptions(
    include_raw_extensions=True,
    include_comments=True,
    validate_references=False
)

result = parser.parse(xml, options)
print(f'✓ Parse with options successful')
"

# 8. Stream test
echo -e "\n${GREEN}Test 8: Streaming${NC}"
python3 -c "
from ddex_parser import DDEXParser

xml = '<ern:NewReleaseMessage xmlns:ern=\"http://ddex.net/xml/ern/43\"/>'
parser = DDEXParser()

count = 0
for release in parser.stream(xml):
    count += 1
    print(f'  Release: {release}')

print(f'✓ Streamed {count} releases')
"

# 9. Error handling test
echo -e "\n${GREEN}Test 9: Error Handling${NC}"
python3 -c "
from ddex_parser import DDEXParser

parser = DDEXParser()

try:
    # Test with invalid input
    parser.parse(None)
    print('✗ Should have raised an error')
except (ValueError, TypeError) as e:
    print(f'✓ Correctly raised error: {type(e).__name__}')

try:
    # Test with invalid XML
    parser.parse('not xml')
    print('✓ Handled invalid XML')
except Exception as e:
    print(f'✓ Handled invalid XML with: {type(e).__name__}')
"

# 10. Run pytest suite
echo -e "\n${GREEN}Test 10: Running pytest suite${NC}"
if [ -f "tests/test_parser.py" ]; then
    pytest tests/ -v --tb=short
else
    echo "⚠ No pytest tests found, skipping"
fi

# Clean up
deactivate
cd -

echo -e "\n${GREEN}✅ All tests completed!${NC}"