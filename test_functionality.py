#!/usr/bin/env python3
"""
Test functionality of both ddex_parser and ddex_builder Python bindings.
"""

import ddex_parser
import ddex_builder

def test_parser():
    """Test basic parser functionality."""
    print("Testing ddex_parser...")
    try:
        parser = ddex_parser.DDEXParser()
        print("✅ Parser instance created successfully")
        
        # Test with simple XML
        simple_xml = """<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43">
    <MessageHeader>
        <MessageId>MSG123</MessageId>
    </MessageHeader>
</NewReleaseMessage>"""
        
        result = parser.parse(simple_xml)
        print(f"✅ Parser test: {result is not None}")
        return True
        
    except Exception as e:
        print(f"❌ Parser test failed: {e}")
        return False

def test_builder():
    """Test basic builder functionality."""
    print("Testing ddex_builder...")
    try:
        builder = ddex_builder.DdexBuilder()
        print("✅ Builder instance created successfully")
        
        # Test available presets
        presets = builder.get_available_presets()
        print(f"✅ Available presets: {presets}")
        
        # Test basic build
        xml = builder.build()
        print(f"✅ Builder test: {len(xml) > 0}")
        print(f"Generated XML length: {len(xml)} characters")
        return True
        
    except Exception as e:
        print(f"❌ Builder test failed: {e}")
        return False

def main():
    """Run all functionality tests."""
    print("=== Python Bindings Functionality Test ===")
    
    parser_ok = test_parser()
    print()
    builder_ok = test_builder()
    print()
    
    if parser_ok and builder_ok:
        print("🎉 All functionality tests PASSED!")
        return 0
    else:
        print("💥 Some functionality tests FAILED!")
        return 1

if __name__ == "__main__":
    exit(main())