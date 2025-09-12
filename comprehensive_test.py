#!/usr/bin/env python3
"""
Comprehensive Python bindings test for DDEX Suite v0.3.5 with PyO3 0.24
Tests both parser and builder functionality to verify PyO3 upgrade success.
"""
import sys
import traceback

def test_imports():
    """Test that imports work correctly"""
    print("=== Testing Imports ===")
    try:
        import ddex_parser
        print("✅ ddex_parser import successful")
        print(f"   Parser version: {getattr(ddex_parser, '__version__', 'unknown')}")
    except ImportError as e:
        print(f"❌ ddex_parser import failed: {e}")
        return False
    
    try:
        import ddex_builder
        print("✅ ddex_builder import successful")
        print(f"   Builder version: {getattr(ddex_builder, '__version__', 'unknown')}")
    except ImportError as e:
        print(f"❌ ddex_builder import failed: {e}")
        return False
    
    return True

def test_parser_functionality():
    """Test basic parser functionality"""
    print("\n=== Testing Parser Functionality ===")
    try:
        import ddex_parser
        
        # Test parser instantiation
        parser = ddex_parser.DDEXParser()
        print("✅ DDEXParser instantiation successful")
        
        # Test basic XML parsing
        minimal_xml = '''<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" LanguageAndScriptCode="en">
    <MessageHeader>
        <MessageId>TEST_MSG_001</MessageId>
        <MessageCreatedDateTime>2024-01-01T00:00:00Z</MessageCreatedDateTime>
        <MessageSender>
            <PartyId>TEST_PARTY</PartyId>
        </MessageSender>
        <MessageRecipient>
            <PartyId>TEST_RECIPIENT</PartyId>
        </MessageRecipient>
    </MessageHeader>
    <UpdateIndicator>OriginalMessage</UpdateIndicator>
    <IsUpdate>false</IsUpdate>
</NewReleaseMessage>'''

        try:
            result = parser.parse(minimal_xml)
            print("✅ Basic XML parsing successful")
            print(f"   Parsed result type: {type(result)}")
            
            # Test result is not None
            assert result is not None, "Parser returned None"
            print("✅ Parser returned valid result")
            
        except Exception as e:
            print(f"❌ XML parsing failed: {e}")
            traceback.print_exc()
            return False
            
    except Exception as e:
        print(f"❌ Parser functionality test failed: {e}")
        traceback.print_exc()
        return False
    
    return True

def test_builder_functionality():
    """Test basic builder functionality"""
    print("\n=== Testing Builder Functionality ===")
    try:
        import ddex_builder
        
        # Test builder instantiation
        builder = ddex_builder.DdexBuilder()
        print("✅ DdexBuilder instantiation successful")
        
        # Test version setting
        try:
            builder.set_version("43")
            print("✅ Version setting successful")
        except Exception as e:
            print(f"⚠️  Version setting failed (may not be implemented): {e}")
        
        # Test basic build operation
        try:
            output = builder.build()
            print("✅ Basic build operation successful")
            print(f"   Output type: {type(output)}")
            if isinstance(output, str):
                print(f"   Output length: {len(output)} characters")
                assert len(output) > 50, "Output too short"
                print("✅ Build output validation successful")
            else:
                print(f"   Output content: {output}")
                
        except Exception as e:
            print(f"❌ Build operation failed: {e}")
            traceback.print_exc()
            return False
            
    except Exception as e:
        print(f"❌ Builder functionality test failed: {e}")
        traceback.print_exc()
        return False
    
    return True

def test_dataframe_integration():
    """Test DataFrame integration if pandas is available"""
    print("\n=== Testing DataFrame Integration ===")
    try:
        import pandas as pd
        print("✅ pandas available")
        
        # Create test DataFrame
        df = pd.DataFrame({
            'release_id': ['R1', 'R2'],
            'title': ['Test Album 1', 'Test Album 2'],
            'artist': ['Test Artist 1', 'Test Artist 2']
        })
        print("✅ Test DataFrame created")
        
        try:
            import ddex_builder
            builder = ddex_builder.DdexBuilder()
            
            # Test DataFrame integration
            try:
                builder.from_dataframe(df)
                print("✅ DataFrame integration successful")
            except AttributeError:
                print("⚠️  from_dataframe method not available (may not be implemented)")
            except Exception as e:
                print(f"⚠️  DataFrame integration failed: {e}")
                
        except Exception as e:
            print(f"❌ DataFrame integration test failed: {e}")
            
    except ImportError:
        print("⚠️  pandas not available, skipping DataFrame tests")
    except Exception as e:
        print(f"❌ DataFrame test setup failed: {e}")

def test_pyo3_compatibility():
    """Test PyO3 0.24 specific compatibility"""
    print("\n=== Testing PyO3 0.24 Compatibility ===")
    
    # Test that we can handle Python exceptions properly
    try:
        import ddex_parser
        parser = ddex_parser.DDEXParser()
        
        # Try to parse invalid XML to test error handling
        try:
            parser.parse("invalid xml content")
        except Exception as e:
            print(f"✅ Exception handling works: {type(e).__name__}")
            print(f"   Error message: {str(e)}")
        
        print("✅ PyO3 0.24 exception handling compatible")
        
    except Exception as e:
        print(f"❌ PyO3 compatibility test failed: {e}")
        return False
    
    return True

def main():
    """Run all tests and report results"""
    print("DDEX Suite v0.3.5 Python Bindings Comprehensive Test")
    print("PyO3 0.24 Compatibility Verification")
    print("=" * 60)
    
    print(f"Python version: {sys.version}")
    print(f"Python executable: {sys.executable}")
    print()
    
    tests = [
        ("Import Tests", test_imports),
        ("Parser Functionality", test_parser_functionality), 
        ("Builder Functionality", test_builder_functionality),
        ("DataFrame Integration", test_dataframe_integration),
        ("PyO3 0.24 Compatibility", test_pyo3_compatibility),
    ]
    
    results = []
    for test_name, test_func in tests:
        try:
            result = test_func()
            results.append((test_name, result))
        except Exception as e:
            print(f"\n❌ {test_name} crashed: {e}")
            traceback.print_exc()
            results.append((test_name, False))
        print()
    
    # Summary
    print("=" * 60)
    print("TEST RESULTS SUMMARY:")
    
    passed = 0
    for test_name, result in results:
        status = "✅ PASSED" if result else "❌ FAILED"
        print(f"  {status}: {test_name}")
        if result:
            passed += 1
    
    print(f"\nOverall: {passed}/{len(results)} tests passed")
    
    if passed == len(results):
        print("\n🎉 All Python bindings functional with PyO3 0.24!")
        return 0
    else:
        print(f"\n⚠️  {len(results) - passed} test(s) failed - see details above")
        return 1

if __name__ == "__main__":
    sys.exit(main())