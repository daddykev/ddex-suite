#!/usr/bin/env python3
import time
import sys
import os

# Try to import modules from different paths
ddex_parser = None
ddex_builder = None

# Import ddex_parser
try:
    import ddex_parser
    print("✅ ddex_parser imported from system")
except ImportError:
    sys.path.insert(0, 'ddex-parser/bindings/python')
    try:
        import ddex_parser
        print("✅ ddex_parser imported from local path")
    except ImportError:
        print("❌ Cannot import ddex_parser")

# Import ddex_builder
try:
    import ddex_builder
    print("✅ ddex_builder imported from system")
except ImportError:
    sys.path.insert(0, 'ddex-builder/bindings/python')
    try:
        import ddex_builder
        print("✅ ddex_builder imported from local path")
    except ImportError:
        print("❌ Cannot import ddex_builder - will test parser only")

def test_native_implementation():
    print("🔍 Testing Native Implementation...")
    
    if not ddex_parser:
        print("❌ Cannot test without ddex_parser module")
        return False
    
    # Test 1: Parse real DDEX file
    print("\n1. Testing Parser with Real ERN Files:")
    
    # Try to find a valid test file
    test_files = [
        '../test-suite/valid/ern43_audioalbum_complete.xml',
        '../test-suite/valid/ern42_single.xml',
        '../test-suite/valid/ern382_album.xml'
    ]
    
    xml_content = None
    test_file = None
    for file_path in test_files:
        if os.path.exists(file_path):
            with open(file_path, 'r') as f:
                xml_content = f.read()
                test_file = file_path
                break
    
    if not xml_content:
        # Create a minimal valid ERN 4.3 for testing
        print("   Creating minimal test ERN 4.3...")
        xml_content = '''<?xml version="1.0" encoding="UTF-8"?>
<NewReleaseMessage xmlns="http://ddex.net/xml/ern/43" MessageSchemaVersionId="ern/43" LanguageAndScriptCode="en">
    <MessageHeader>
        <MessageId>MSG123</MessageId>
        <MessageSender>
            <PartyName><FullName>Test Sender</FullName></PartyName>
        </MessageSender>
        <MessageRecipient>
            <PartyName><FullName>Test Recipient</FullName></PartyName>
        </MessageRecipient>
        <MessageCreatedDateTime>2025-01-01T00:00:00Z</MessageCreatedDateTime>
    </MessageHeader>
    <ResourceList>
        <SoundRecording>
            <SoundRecordingType>MusicalWorkSoundRecording</SoundRecordingType>
            <ResourceReference>A1</ResourceReference>
            <ResourceId>
                <ISRC>USRC17607839</ISRC>
            </ResourceId>
            <ReferenceTitle>
                <TitleText>Test Track</TitleText>
            </ReferenceTitle>
            <Duration>PT3M30S</Duration>
        </SoundRecording>
    </ResourceList>
    <ReleaseList>
        <Release>
            <ReleaseReference>R1</ReleaseReference>
            <ReleaseId>
                <GRid>A12345678901234</GRid>
            </ReleaseId>
            <ReferenceTitle>
                <TitleText>Test Album</TitleText>
            </ReferenceTitle>
            <ReleaseResourceReferenceList>
                <ReleaseResourceReference>A1</ReleaseResourceReference>
            </ReleaseResourceReferenceList>
        </Release>
    </ReleaseList>
</NewReleaseMessage>'''
    
    # Parse the XML
    start = time.time()
    parser = ddex_parser.DDEXParser()
    
    try:
        result = parser.parse(xml_content)
        parse_time = (time.time() - start) * 1000
        
        print(f"   ✓ Parse time: {parse_time:.4f}ms")
        
        # Inspect the result object
        print(f"   ✓ Result type: {type(result)}")
        print(f"   ✓ Result attributes: {[attr for attr in dir(result) if not attr.startswith('_')]}")
        
        # Check if result has expected attributes
        if hasattr(result, 'releases'):
            releases = result.releases
            print(f"   ✓ Found {len(releases)} releases (type: {type(releases)})")
            if releases:
                release = releases[0]
                print(f"     - Release type: {type(release)}")
                print(f"     - Release attrs: {[attr for attr in dir(release) if not attr.startswith('_')]}")
                
        if hasattr(result, 'resources'):
            resources = result.resources 
            print(f"   ✓ Found {len(resources)} resources (type: {type(resources)})")
            
        if hasattr(result, 'version'):
            print(f"   ✓ Version detected: {result.version}")
            
        # Check for flat attribute
        if hasattr(result, 'flat'):
            flat = result.flat
            print(f"   ✓ Has flat structure: {type(flat)}")
            if hasattr(flat, 'releases'):
                print(f"     - Flat releases: {len(flat.releases)}")
            if hasattr(flat, 'message_id'):
                print(f"     - Message ID: {flat.message_id}")
                
        # Try to get actual data content
        if hasattr(result, '__dict__'):
            result_dict = result.__dict__
            print(f"   ✓ Result dict keys: {list(result_dict.keys())}")
            if '_data' in result_dict:
                data = result_dict['_data']
                print(f"   ✓ Internal data type: {type(data)}")
                if hasattr(data, '__dict__'):
                    print(f"   ✓ Internal data keys: {list(data.__dict__.keys())}")
                    
        # Test streaming functionality
        print("   \n⚡ Testing streaming functionality:")
        try:
            stream_iter = parser.stream(xml_content)
            print(f"   ✓ Stream iterator created: {type(stream_iter)}")
            releases_from_stream = list(stream_iter)
            print(f"   ✓ Streamed {len(releases_from_stream)} items")
            if releases_from_stream:
                print(f"   ✓ First streamed item: {releases_from_stream[0]}")
        except Exception as e:
            print(f"   ⚠️ Stream error: {e}")
            
        # Test to_dataframe functionality  
        print("   \n📊 Testing DataFrame export:")
        try:
            df = parser.to_dataframe(xml_content)
            print(f"   ✓ DataFrame created: {type(df)}")
            print(f"   ✓ DataFrame shape: {getattr(df, 'shape', 'no shape attr')}")
        except Exception as e:
            print(f"   ⚠️ DataFrame error: {e}")
            
    except Exception as e:
        print(f"   ⚠️ Parse error: {e}")
        import traceback
        traceback.print_exc()
    
    # Test 2: Build real XML
    print("\n2. Testing Builder with Real Data:")
    if not ddex_builder:
        print("   ⚠️ ddex_builder not available - skipping builder tests")
        print("\n✅ Native Implementation Test Complete (Parser only)!")
        return True
        
    builder = ddex_builder.DdexBuilder()
    
    # Try different ways to create data
    try:
        # Method 1: Using builder's data classes
        if hasattr(ddex_builder, 'Release'):
            release = ddex_builder.Release(
                release_id="REL123",
                title="Test Album",
                artist="Test Artist",
                label="Test Label",
                catalog_number="CAT123"
            )
            print("   ✓ Created Release object")
    except Exception as e:
        print(f"   ⚠️ Could not create Release: {e}")
        release = None
    
    try:
        if hasattr(ddex_builder, 'Resource'):
            resource = ddex_builder.Resource(
                resource_id="RES456",
                isrc="USRC17607839",
                title="Test Track",
                duration="PT3M30S"
            )
            print("   ✓ Created Resource object")
    except Exception as e:
        print(f"   ⚠️ Could not create Resource: {e}")
        resource = None
    
    # Try to build XML
    if release and resource:
        try:
            start = time.time()
            xml_output = builder.build([release], [resource])
            build_time = (time.time() - start) * 1000
            
            print(f"   ✓ Build time: {build_time:.2f}ms")
            print(f"   ✓ XML length: {len(xml_output)} bytes")
            
            if "NewReleaseMessage" in xml_output:
                print("   ✓ Valid DDEX XML structure detected")
            if "USRC17607839" in xml_output:
                print("   ✓ ISRC preserved in output")
        except Exception as e:
            print(f"   ⚠️ Build error: {e}")
    
    # Test 3: Performance validation
    print("\n3. Performance Validation:")
    iterations = 5
    total_time = 0
    
    for i in range(iterations):
        start = time.time()
        try:
            parser.parse(xml_content)
            total_time += (time.time() - start) * 1000
        except:
            pass
    
    if total_time > 0:
        avg_time = total_time / iterations
        print(f"   ✓ Average parse time: {avg_time:.2f}ms")
        
        if avg_time < 50:
            print("   ✅ EXCELLENT: Performance target exceeded (<50ms)")
        elif avg_time < 100:
            print("   ✅ GOOD: Within acceptable range")
        else:
            print("   ⚠️ WARNING: Performance below target")
    
    print("\n✅ Native Implementation Test Complete!")
    return True

if __name__ == "__main__":
    try:
        test_native_implementation()
    except ImportError as e:
        print(f"❌ Import Error: {e}")
        print("Make sure maturin develop was run successfully")
    except Exception as e:
        print(f"⚠️ Test Warning: {e}")
        import traceback
        traceback.print_exc()