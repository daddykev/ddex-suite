#!/usr/bin/env python3
"""
Generate synthetic DDEX test files with various characteristics.
These are synthetic files for testing, not real DDEX data.
"""

import os
import xml.etree.ElementTree as ET
from pathlib import Path
from datetime import datetime, timedelta
import random
import string

def create_dirs():
    """Create test directory structure."""
    dirs = [
        'test-suite/valid/ern-4.3',
        'test-suite/valid/ern-4.2',
        'test-suite/valid/ern-3.8.2',
        'test-suite/edge-cases/large',
        'test-suite/edge-cases/complex',
        'test-suite/edge-cases/vendor-quirks',
        'test-suite/nasty',
        'test-suite/golden/graph',
        'test-suite/golden/flat',
    ]
    for dir_path in dirs:
        Path(dir_path).mkdir(parents=True, exist_ok=True)

def generate_simple_ern_43():
    """Generate a simple ERN 4.3 test file."""
    root = ET.Element('ern:NewReleaseMessage', {
        'xmlns:ern': 'http://ddex.net/xml/ern/43',
        'MessageSchemaVersionId': 'ern/43',
        'LanguageAndScriptCode': 'en'
    })
    
    # Message Header
    header = ET.SubElement(root, 'MessageHeader')
    ET.SubElement(header, 'MessageThreadId').text = 'TEST_MSG_001'
    ET.SubElement(header, 'MessageId').text = 'MSG_' + ''.join(random.choices(string.ascii_uppercase + string.digits, k=10))
    
    sender = ET.SubElement(header, 'MessageSender')
    ET.SubElement(sender, 'PartyId').text = 'PADPIDA2014120301'
    ET.SubElement(sender, 'PartyName').text = 'Test Sender'
    
    recipient = ET.SubElement(header, 'MessageRecipient')
    ET.SubElement(recipient, 'PartyId').text = 'PADPIDA2014120302'
    ET.SubElement(recipient, 'PartyName').text = 'Test Recipient'
    
    ET.SubElement(header, 'MessageCreatedDateTime').text = datetime.now().isoformat()
    
    # Release List
    release_list = ET.SubElement(root, 'ReleaseList')
    release = ET.SubElement(release_list, 'Release')
    ET.SubElement(release, 'ReleaseReference').text = 'R1'
    
    release_id = ET.SubElement(release, 'ReleaseId')
    ET.SubElement(release_id, 'GRid').text = 'A1-TEST-GRID-0001'
    
    title = ET.SubElement(release, 'ReferenceTitle')
    ET.SubElement(title, 'TitleText').text = 'Test Release Title'
    
    # Resource List
    resource_list = ET.SubElement(root, 'ResourceList')
    resource = ET.SubElement(resource_list, 'SoundRecording')
    ET.SubElement(resource, 'ResourceReference').text = 'A1'
    
    resource_id = ET.SubElement(resource, 'SoundRecordingId')
    ET.SubElement(resource_id, 'ISRC').text = 'USTEST0000001'
    
    title = ET.SubElement(resource, 'ReferenceTitle')
    ET.SubElement(title, 'TitleText').text = 'Test Track Title'
    
    # Deal List
    deal_list = ET.SubElement(root, 'DealList')
    deal = ET.SubElement(deal_list, 'ReleaseDeal')
    ET.SubElement(deal, 'DealReleaseReference').text = 'R1'
    
    deal_terms = ET.SubElement(deal, 'Deal')
    ET.SubElement(deal_terms, 'TerritoryCode').text = 'Worldwide'
    ET.SubElement(deal_terms, 'StartDate').text = datetime.now().date().isoformat()
    
    tree = ET.ElementTree(root)
    ET.indent(tree, '  ')
    return tree

def generate_billion_laughs():
    """Generate a billion laughs attack test file."""
    xml_content = '''<?xml version="1.0"?>
<!DOCTYPE lolz [
  <!ENTITY lol "lol">
  <!ELEMENT lolz (#PCDATA)>
  <!ENTITY lol1 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
  <!ENTITY lol2 "&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;">
  <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
  <!ENTITY lol4 "&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;">
  <!ENTITY lol5 "&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;&lol4;">
]>
<lolz>&lol5;</lolz>'''
    return xml_content

def generate_deep_nesting():
    """Generate deeply nested XML for testing."""
    depth = 1000
    xml = '<?xml version="1.0"?>\n<root>\n'
    for i in range(depth):
        xml += '  ' * i + f'<level_{i}>\n'
    xml += '  ' * depth + '<data>Deep nested content</data>\n'
    for i in range(depth - 1, -1, -1):
        xml += '  ' * i + f'</level_{i}>\n'
    xml += '</root>'
    return xml

def generate_test_files():
    """Generate all test files."""
    create_dirs()
    
    # Simple valid ERN 4.3
    tree = generate_simple_ern_43()
    tree.write('test-suite/valid/ern-4.3/simple_release.xml', 
               encoding='utf-8', xml_declaration=True)
    
    # Billion laughs attack
    with open('test-suite/nasty/billion-laughs.xml', 'w') as f:
        f.write(generate_billion_laughs())
    
    # Deep nesting attack
    with open('test-suite/nasty/deep-nesting.xml', 'w') as f:
        f.write(generate_deep_nesting())
    
    # Create README for test suite
    readme_content = """# DDEX Parser Test Suite

This directory contains synthetic test files for the DDEX Parser.

## Structure

- `valid/` - Valid DDEX files for each supported version
- `edge-cases/` - Edge cases and real-world quirks
- `nasty/` - Security test cases (XXE, billion laughs, etc.)
- `golden/` - Expected output for regression testing

## License

These synthetic test files are part of the DDEX Parser project and are
licensed under the MIT License. They are NOT real DDEX data and should
not be used for any purpose other than testing this parser.

## Generating Test Files

Run the Python script to regenerate test files:

```bash
python test-suite/generate_test_corpus.py
```
"""
    
    with open('test-suite/README.md', 'w') as f:
        f.write(readme_content)
    
    print("✅ Test corpus generated successfully!")
    print("Files created:")
    print("  - test-suite/valid/ern-4.3/simple_release.xml")
    print("  - test-suite/nasty/billion-laughs.xml")
    print("  - test-suite/nasty/deep-nesting.xml")
    print("  - test-suite/README.md")

if __name__ == '__main__':
    generate_test_files()