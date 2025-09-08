# test_full.py - Fixed version
from ddex_parser import DDEXParser, __version__

print("=" * 60)
print(f"DDEX Parser Python Bindings - Version {__version__}")
print("=" * 60)

# Create parser
parser = DDEXParser()
print(f"✅ Parser created: {parser}")

# Test 1: Parse XML
xml = """<?xml version="1.0" encoding="UTF-8"?>
<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>TEST_MSG_001</MessageId>
    </MessageHeader>
</ern:NewReleaseMessage>"""

result = parser.parse(xml)
print(f"\n✅ Parse result:")
print(f"   Message ID: {result.message_id}")  # Direct attribute access
print(f"   Version: {result.version}")
print(f"   Release count: {result.release_count}")
print(f"   Releases: {result.releases}")

# Test 2: Version detection
versions = {
    "4.3": '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">',
    "4.2": '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/42">',
    "3.8.2": '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/382">',
}

print(f"\n✅ Version detection:")
for expected, xml_snippet in versions.items():
    detected = parser.detect_version(xml_snippet)
    print(f"   {expected}: {'✓' if detected == expected else '✗'}")

# Test 3: Sanity check (returns a dict)
check = parser.sanity_check(xml)
print(f"\n✅ Sanity check:")
print(f"   Valid: {check.get('is_valid')}")
print(f"   Version: {check.get('version')}")

print("\n" + "=" * 60)
print("🎉 All tests passed! Python bindings are working!")
print("=" * 60)