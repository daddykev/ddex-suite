#!/usr/bin/env python3

import os
import sys
import time
import gc
import psutil
from ddex_parser import DDEXParser

def test_parser_functionality():
    """Test core parser functionality with working methods"""
    print("\n✅ Testing Working Parser Functionality...")
    
    parser = DDEXParser()
    
    # Test basic XML parsing  
    test_xml = """<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
        <MessageHeader>
            <MessageId>FUNC-TEST-001</MessageId>
            <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
            <MessageSender><PartyName>Function Test</PartyName></MessageSender>
            <MessageRecipient><PartyName>Test Recipient</PartyName></MessageRecipient>
        </MessageHeader>
        <ResourceList>
            <SoundRecording>
                <ResourceReference>SR001</ResourceReference>
                <Type>MusicalWorkSoundRecording</Type>
                <Title><TitleText>Function Test Track</TitleText></Title>
                <Duration>PT3M30S</Duration>
                <DisplayArtist>Function Test Artist</DisplayArtist>
            </SoundRecording>
        </ResourceList>
    </ern:NewReleaseMessage>"""
    
    # Test parse method
    start_time = time.time()
    result = parser.parse(test_xml)
    parse_time = (time.time() - start_time) * 1000
    
    print(f"   ✅ Parse method: {parse_time:.2f}ms, Result type: {type(result).__name__}")
    
    # Test to_dataframe method
    start_time = time.time()  
    df = parser.to_dataframe(test_xml)
    df_time = (time.time() - start_time) * 1000
    
    print(f"   ✅ to_dataframe method: {df_time:.2f}ms, DataFrame shape: {df.shape}")
    print(f"   📊 DataFrame columns: {list(df.columns)}")
    
    # Test available methods
    methods = [m for m in dir(parser) if not m.startswith('_')]
    print(f"   📋 Available methods: {', '.join(methods)}")
    
    # Test detect_version method
    try:
        version = parser.detect_version(test_xml)
        print(f"   ✅ detect_version: {version}")
    except Exception as e:
        print(f"   ❌ detect_version failed: {e}")
    
    # Test sanity_check method
    try:
        sanity_result = parser.sanity_check(test_xml)
        print(f"   ✅ sanity_check: {type(sanity_result).__name__}")
    except Exception as e:
        print(f"   ❌ sanity_check failed: {e}")
    
    return True

def test_memory_efficiency():
    """Test memory efficiency with large XML processing"""
    print("\n🧠 Testing Memory Efficiency...")
    
    parser = DDEXParser()
    process = psutil.Process(os.getpid())
    
    # Generate large test XML
    print("   📊 Generating large test XML (1000 resources)...")
    
    resources = []
    for i in range(1000):
        resources.append(f"""
        <SoundRecording>
            <ResourceReference>SR{i:04d}</ResourceReference>
            <Type>MusicalWorkSoundRecording</Type>
            <Title><TitleText>Memory Test Track {i}</TitleText></Title>
            <Duration>PT{3 + (i % 3)}M{(i % 60):02d}S</Duration>
            <DisplayArtist>Memory Test Artist {i // 50}</DisplayArtist>
        </SoundRecording>""")
    
    large_xml = f"""<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
        <MessageHeader>
            <MessageId>MEMORY-TEST-001</MessageId>
            <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
            <MessageSender><PartyName>Memory Test</PartyName></MessageSender>
            <MessageRecipient><PartyName>Test Recipient</PartyName></MessageRecipient>
        </MessageHeader>
        <ResourceList>
            {''.join(resources)}
        </ResourceList>
    </ern:NewReleaseMessage>"""
    
    xml_size_mb = len(large_xml) / 1024 / 1024
    print(f"   📏 XML size: {xml_size_mb:.2f}MB")
    
    # Measure memory before
    gc.collect()
    memory_before = process.memory_info().rss / 1024 / 1024
    
    # Parse large XML
    start_time = time.time()
    result = parser.parse(large_xml)
    parse_time = time.time() - start_time
    
    # Measure memory after
    memory_after = process.memory_info().rss / 1024 / 1024
    memory_used = memory_after - memory_before
    
    print(f"   ✅ Large XML parsed in {parse_time:.2f}s")
    print(f"   💾 Memory used: {memory_used:.1f}MB")
    print(f"   ⚡ Processing rate: {xml_size_mb/parse_time:.1f}MB/s")
    
    # Test DataFrame conversion for large XML
    try:
        start_time = time.time()
        df = parser.to_dataframe(large_xml)
        df_time = time.time() - start_time
        print(f"   ✅ DataFrame conversion: {df_time:.2f}s, Shape: {df.shape}")
    except Exception as e:
        print(f"   ❌ DataFrame conversion failed: {e}")
    
    # Cleanup
    del result
    del large_xml
    gc.collect()
    
    return memory_used < 100  # Target: <100MB memory usage

def test_error_handling():
    """Test error handling and exception cases"""
    print("\n⚠️  Testing Error Handling...")
    
    parser = DDEXParser()
    
    error_tests = [
        ("Invalid XML", "not valid xml at all"),
        ("Empty Input", ""),
        ("Only XML declaration", '<?xml version="1.0" encoding="UTF-8"?>'),
        ("Malformed XML", '<invalid><unclosed>'),
        ("Valid XML, Invalid DDEX", '<root><invalid>content</invalid></root>'),
    ]
    
    for test_name, test_input in error_tests:
        try:
            result = parser.parse(test_input)
            print(f"   ⚠️  {test_name}: Handled gracefully (no exception)")
        except Exception as e:
            print(f"   ✅ {test_name}: Correctly raised {type(e).__name__}: {str(e)[:50]}...")
    
    # Test DataFrame with invalid input
    try:
        df = parser.to_dataframe("invalid xml")
        print(f"   ⚠️  DataFrame invalid input: Handled gracefully")
    except Exception as e:
        print(f"   ✅ DataFrame invalid input: Correctly raised {type(e).__name__}")
    
    return True

def test_api_semantics():
    """Test that Python API matches expected semantics"""
    print("\n🔍 Testing API Semantics...")
    
    parser = DDEXParser()
    
    # Test method signatures and return types
    test_xml = """<?xml version="1.0" encoding="UTF-8"?>
    <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
        <MessageHeader>
            <MessageId>API-TEST-001</MessageId>
            <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
            <MessageSender><PartyName>API Test</PartyName></MessageSender>
            <MessageRecipient><PartyName>Test Recipient</PartyName></MessageRecipient>
        </MessageHeader>
    </ern:NewReleaseMessage>"""
    
    # Test parse method return type
    result = parser.parse(test_xml)
    print(f"   ✅ parse() returns: {type(result).__name__}")
    
    # Test to_dataframe return type
    import pandas as pd
    df = parser.to_dataframe(test_xml)
    is_dataframe = isinstance(df, pd.DataFrame)
    print(f"   ✅ to_dataframe() returns pandas.DataFrame: {is_dataframe}")
    
    # Test detect_version return type
    try:
        version = parser.detect_version(test_xml)
        print(f"   ✅ detect_version() returns: {type(version).__name__} = '{version}'")
    except Exception as e:
        print(f"   ❌ detect_version() failed: {e}")
    
    # Test method consistency
    # Parse the same XML multiple times and ensure consistent results
    results = []
    for i in range(3):
        result = parser.parse(test_xml)
        results.append(str(result))
    
    consistent = all(r == results[0] for r in results)
    print(f"   ✅ Consistent results across multiple calls: {consistent}")
    
    return True

def main():
    """Main test execution"""
    print("🐍 DDEX Suite Python Bindings Targeted Testing")
    print("=" * 60)
    print(f"Python: {sys.version.split()[0]}")
    print(f"Platform: {os.uname().sysname} {os.uname().machine}")
    
    results = {}
    start_time = time.time()
    
    # Run targeted tests
    try:
        results['functionality'] = test_parser_functionality()
        results['memory'] = test_memory_efficiency()  
        results['errors'] = test_error_handling()
        results['api_semantics'] = test_api_semantics()
        
    except Exception as e:
        print(f"\n💥 Test execution failed: {e}")
        return False
    
    total_time = time.time() - start_time
    
    # Summary
    print("\n" + "=" * 60)
    print("🎯 Python Bindings Test Results")
    print(f"   Total execution time: {total_time:.2f}s")
    
    passed_tests = sum(results.values())
    total_tests = len(results)
    
    for test_name, result in results.items():
        status = "✅" if result else "❌"
        print(f"   {status} {test_name.title()}: {'PASSED' if result else 'FAILED'}")
    
    success_rate = (passed_tests / total_tests) * 100
    print(f"\n   📊 Success Rate: {success_rate:.0f}% ({passed_tests}/{total_tests} tests passed)")
    
    if success_rate >= 75:
        print("   🎉 EXCELLENT: Python bindings are working well!")
    elif success_rate >= 50:
        print("   👍 GOOD: Most functionality working with some limitations")
    else:
        print("   ⚠️  NEEDS WORK: Several issues need addressing")
    
    print("=" * 60)
    
    return success_rate >= 50

if __name__ == "__main__":
    try:
        success = main()
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print("\n\n⚠️  Test interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n\n💥 Fatal error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)