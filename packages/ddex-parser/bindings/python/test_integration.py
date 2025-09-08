# test_integration.py - Save this in packages/ddex-parser/bindings/python/
"""Integration test for Python bindings."""

import sys
import traceback
from pathlib import Path

def test_basic_functionality():
    """Test basic parser functionality."""
    print("=" * 60)
    print("DDEX Parser Python Bindings - Integration Test")
    print("=" * 60)
    
    try:
        # Test 1: Import
        print("\n[1/10] Testing import...")
        from ddex_parser import DDEXParser, ParseOptions, __version__
        print(f"✓ Successfully imported ddex_parser v{__version__}")
        
        # Test 2: Create instance
        print("\n[2/10] Testing parser creation...")
        parser = DDEXParser()
        print(f"✓ Created parser: {parser}")
        
        # Test 3: Parse XML
        print("\n[3/10] Testing XML parsing...")
        xml = """<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader>
                <MessageId>TEST_MSG_001</MessageId>
            </MessageHeader>
        </ern:NewReleaseMessage>"""
        
        result = parser.parse(xml)
        print(f"✓ Parsed XML successfully")
        print(f"  - Message ID: {result.message_id}")
        print(f"  - Version: {result.version}")
        print(f"  - Releases: {result.release_count}")
        
        # Test 4: Parse with options
        print("\n[4/10] Testing parse options...")
        options = ParseOptions(
            include_raw_extensions=True,
            include_comments=True
        )
        result = parser.parse(xml, options)
        print(f"✓ Parsed with options")
        
        # Test 5: Version detection
        print("\n[5/10] Testing version detection...")
        versions = {
            '4.3': '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">',
            '4.2': '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/42">',
            '3.8.2': '<ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/382">',
        }
        
        for expected, xml_snippet in versions.items():
            detected = parser.detect_version(xml_snippet)
            assert detected == expected, f"Expected {expected}, got {detected}"
            print(f"✓ Correctly detected version {expected}")
        
        # Test 6: Sanity check
        print("\n[6/10] Testing sanity check...")
        check_result = parser.sanity_check(xml)
        print(f"✓ Sanity check completed")
        print(f"  - Valid: {check_result.get('is_valid')}")
        print(f"  - Version: {check_result.get('version')}")
        
        # Test 7: Bytes input
        print("\n[7/10] Testing bytes input...")
        result = parser.parse(xml.encode('utf-8'))
        print(f"✓ Parsed bytes successfully")
        
        # Test 8: Async support
        print("\n[8/10] Testing async parsing...")
        import asyncio
        
        async def async_test():
            result = await parser.parse_async(xml)
            return result
        
        result = asyncio.run(async_test())
        print(f"✓ Async parse successful")
        
        # Test 9: DataFrame (if pandas available)
        print("\n[9/10] Testing DataFrame conversion...")
        try:
            import pandas as pd
            df = parser.to_dataframe(xml)
            print(f"✓ DataFrame created with shape {df.shape}")
        except ImportError:
            print("⚠ Pandas not installed, skipping DataFrame test")
        
        # Test 10: Streaming
        print("\n[10/10] Testing streaming...")
        releases = list(parser.stream(xml))
        print(f"✓ Streamed {len(releases)} releases")
        
        print("\n" + "=" * 60)
        print("✅ ALL TESTS PASSED!")
        print("=" * 60)
        return True
        
    except Exception as e:
        print(f"\n❌ TEST FAILED: {e}")
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = test_basic_functionality()
    sys.exit(0 if success else 1)