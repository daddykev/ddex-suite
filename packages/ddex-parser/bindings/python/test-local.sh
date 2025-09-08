# packages/ddex-parser/bindings/python/test-local.sh
#!/bin/bash
set -e

echo "Testing local Python package..."

# Create virtual environment
python -m venv test-venv
source test-venv/bin/activate

# Install local wheel
pip install dist/*.whl

# Run tests
python -c "
from ddex_parser import DDEXParser, __version__
print(f'Version: {__version__}')
parser = DDEXParser()
xml = '<ern:NewReleaseMessage xmlns:ern=\"http://ddex.net/xml/ern/43\"/>'
result = parser.parse(xml)
print(f'Parse result: {result}')
print('✅ Basic import and parse works!')
"

# Run pytest if available
pip install pytest pytest-asyncio
pytest tests/ -v

deactivate
rm -rf test-venv

echo "✅ Local tests passed!"