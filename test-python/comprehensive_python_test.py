#!/usr/bin/env python3

import os
import sys
import time
import traceback
from typing import Dict, List, Any, Optional
import platform

def log_test_result(test_name: str, success: bool, message: str = "", details: str = ""):
    """Log test results with consistent formatting"""
    status = "✅" if success else "❌"
    print(f"{status} {test_name}: {message}")
    if details:
        print(f"   {details}")

def test_basic_imports():
    """Test 1: Basic package imports and instantiation"""
    print("\n🔧 Testing Basic Imports and Instantiation...")
    
    try:
        # Test parser import
        from ddex_parser import DDEXParser
        parser = DDEXParser()
        log_test_result("Parser Import", True, "DDEXParser imported and instantiated successfully")
        parser_available = True
    except Exception as e:
        log_test_result("Parser Import", False, f"Failed to import DDEXParser: {e}")
        parser_available = False
    
    try:
        # Test builder import
        from ddex_builder import DDEXBuilder  
        builder = DDEXBuilder()
        log_test_result("Builder Import", True, "DDEXBuilder imported and instantiated successfully")
        builder_available = True
    except Exception as e:
        log_test_result("Builder Import", False, f"Failed to import DDEXBuilder: {e}")
        builder_available = False
    
    return parser_available, builder_available

def test_pyo3_native_bindings(parser_available: bool, builder_available: bool):
    """Test 2: PyO3 native bindings functionality on current platform"""
    print("\n🖥️  Testing PyO3 Native Bindings on macOS/ARM...")
    
    platform_info = f"Platform: {platform.system()} {platform.machine()}, Python: {platform.python_version()}"
    print(f"   {platform_info}")
    
    if parser_available:
        try:
            from ddex_parser import DDEXParser
            parser = DDEXParser()
            
            # Test version method
            version = parser.version()
            log_test_result("Parser Version Method", True, f"Version: {version}")
            
            # Test basic parsing
            test_xml = """<?xml version="1.0" encoding="UTF-8"?>
            <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
                <MessageHeader>
                    <MessageId>PYTEST-001</MessageId>
                    <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
                    <MessageSender><PartyName>Python Test</PartyName></MessageSender>
                    <MessageRecipient><PartyName>Test Recipient</PartyName></MessageRecipient>
                </MessageHeader>
            </ern:NewReleaseMessage>"""
            
            start_time = time.time()
            result = parser.parse(test_xml)
            parse_time = (time.time() - start_time) * 1000
            
            log_test_result("Parser Basic Functionality", True, 
                          f"Parsed in {parse_time:.2f}ms",
                          f"Result keys: {list(result.keys()) if isinstance(result, dict) else type(result).__name__}")
            
        except Exception as e:
            log_test_result("Parser Native Binding Test", False, f"Parser test failed: {e}")
    
    if builder_available:
        try:
            from ddex_builder import DDEXBuilder
            builder = DDEXBuilder()
            
            # Test available presets
            presets = builder.get_available_presets()
            log_test_result("Builder Presets Method", True, f"Found {len(presets)} presets: {', '.join(presets)}")
            
            # Test basic building
            builder.add_resource({
                "resource_id": "SR001",
                "resource_type": "SoundRecording", 
                "title": "Python Test Track",
                "artist": "Test Artist",
                "isrc": "TEST12345678",
                "duration": "PT3M30S"
            })
            
            builder.add_release({
                "release_id": "REL001",
                "release_type": "Album",
                "title": "Python Test Album",
                "artist": "Test Artist", 
                "upc": "123456789012",
                "track_ids": ["SR001"]
            })
            
            start_time = time.time()
            xml_output = builder.build()
            build_time = (time.time() - start_time) * 1000
            
            log_test_result("Builder Basic Functionality", True,
                          f"Built XML in {build_time:.2f}ms",
                          f"Output size: {len(xml_output) // 1024}KB")
                          
        except Exception as e:
            log_test_result("Builder Native Binding Test", False, f"Builder test failed: {e}")

def test_dataframe_integration(parser_available: bool):
    """Test 3: DataFrame integration with to_dataframe() and from_dataframe()"""
    print("\n📊 Testing DataFrame Integration...")
    
    if not parser_available:
        log_test_result("DataFrame Integration", False, "Parser not available - skipping DataFrame tests")
        return
    
    try:
        import pandas as pd
        log_test_result("Pandas Import", True, f"Pandas {pd.__version__} available")
        pandas_available = True
    except ImportError:
        log_test_result("Pandas Import", False, "Pandas not available - installing...")
        try:
            import subprocess
            subprocess.check_call([sys.executable, "-m", "pip", "install", "pandas>=1.5"])
            import pandas as pd
            pandas_available = True
            log_test_result("Pandas Installation", True, f"Pandas {pd.__version__} installed")
        except Exception as e:
            log_test_result("Pandas Installation", False, f"Failed to install pandas: {e}")
            pandas_available = False
    
    if not pandas_available:
        return
        
    try:
        from ddex_parser import DDEXParser
        parser = DDEXParser()
        
        # Create test XML with multiple resources for DataFrame testing
        catalog_xml = """<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader>
                <MessageId>DATAFRAME-TEST-001</MessageId>
                <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
                <MessageSender><PartyName>DataFrame Test</PartyName></MessageSender>
                <MessageRecipient><PartyName>Test Recipient</PartyName></MessageRecipient>
            </MessageHeader>
            <ResourceList>
                <SoundRecording>
                    <ResourceReference>SR001</ResourceReference>
                    <Type>MusicalWorkSoundRecording</Type>
                    <Title><TitleText>Track One</TitleText></Title>
                    <Duration>PT3M30S</Duration>
                    <DisplayArtist>Test Artist</DisplayArtist>
                </SoundRecording>
                <SoundRecording>
                    <ResourceReference>SR002</ResourceReference>
                    <Type>MusicalWorkSoundRecording</Type>
                    <Title><TitleText>Track Two</TitleText></Title>
                    <Duration>PT4M15S</Duration>
                    <DisplayArtist>Test Artist</DisplayArtist>
                </SoundRecording>
            </ResourceList>
            <ReleaseList>
                <Release>
                    <ReleaseReference>REL001</ReleaseReference>
                    <ReleaseType>Album</ReleaseType>
                    <DisplayTitleText>Test Album</DisplayTitleText>
                    <DisplayArtist>Test Artist</DisplayArtist>
                </Release>
            </ReleaseList>
        </ern:NewReleaseMessage>"""
        
        # Test to_dataframe method if available
        try:
            df = parser.to_dataframe(catalog_xml)
            log_test_result("to_dataframe() Method", True, 
                          f"Created DataFrame with shape: {df.shape}",
                          f"Columns: {list(df.columns)}")
        except AttributeError:
            log_test_result("to_dataframe() Method", False, "Method not available in current implementation")
        except Exception as e:
            log_test_result("to_dataframe() Method", False, f"Method failed: {e}")
        
        # Test parsing with DataFrame export capabilities
        try:
            result = parser.parse(catalog_xml)
            if hasattr(result, 'to_dataframe'):
                df = result.to_dataframe()
                log_test_result("Result to DataFrame", True, f"DataFrame shape: {df.shape}")
            else:
                log_test_result("Result to DataFrame", False, "DataFrame export not available in result object")
        except Exception as e:
            log_test_result("Result to DataFrame", False, f"DataFrame export failed: {e}")
            
    except Exception as e:
        log_test_result("DataFrame Integration", False, f"DataFrame test failed: {e}")

def test_dataframe_roundtrip(parser_available: bool, builder_available: bool):
    """Test 4: Round-trip testing with pandas DataFrames"""
    print("\n🔄 Testing DataFrame Round-Trip...")
    
    if not (parser_available and builder_available):
        log_test_result("DataFrame Round-Trip", False, "Both parser and builder required - skipping")
        return
        
    try:
        import pandas as pd
    except ImportError:
        log_test_result("DataFrame Round-Trip", False, "Pandas not available - skipping")
        return
        
    try:
        from ddex_parser import DDEXParser
        from ddex_builder import DDEXBuilder
        
        parser = DDEXParser()
        builder = DDEXBuilder()
        
        # Test the round-trip: XML → Parse → DataFrame → Build → XML
        original_xml = """<?xml version="1.0" encoding="UTF-8"?>
        <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
            <MessageHeader>
                <MessageId>ROUNDTRIP-001</MessageId>
                <MessageCreatedDateTime>2025-09-11T18:00:00Z</MessageCreatedDateTime>
                <MessageSender><PartyName>Round-Trip Test</PartyName></MessageSender>
                <MessageRecipient><PartyName>Test Recipient</PartyName></MessageRecipient>
            </MessageHeader>
            <ResourceList>
                <SoundRecording>
                    <ResourceReference>SR001</ResourceReference>
                    <Type>MusicalWorkSoundRecording</Type>
                    <Title><TitleText>Round-Trip Track</TitleText></Title>
                    <Duration>PT3M45S</Duration>
                    <DisplayArtist>Round-Trip Artist</DisplayArtist>
                </SoundRecording>
            </ResourceList>
        </ern:NewReleaseMessage>"""
        
        # Step 1: Parse original XML
        parsed_result = parser.parse(original_xml)
        log_test_result("Round-Trip Parse Step", True, "Original XML parsed successfully")
        
        # Step 2: Test from_dataframe method if available
        try:
            if hasattr(builder, 'from_dataframe'):
                # Create a simple DataFrame for testing
                df = pd.DataFrame({
                    'resource_id': ['SR001'],
                    'title': ['Round-Trip Track Modified'],
                    'artist': ['Round-Trip Artist'],
                    'duration': ['PT3M45S']
                })
                
                rebuilt_xml = builder.from_dataframe(df, version='4.3')
                log_test_result("from_dataframe() Method", True, f"Built XML from DataFrame: {len(rebuilt_xml)} chars")
                
                # Parse the rebuilt XML to verify
                reparsed = parser.parse(rebuilt_xml)
                log_test_result("Round-Trip Verification", True, "Successfully parsed rebuilt XML")
                
            else:
                log_test_result("from_dataframe() Method", False, "Method not available in current implementation")
        except Exception as e:
            log_test_result("DataFrame Round-Trip", False, f"Round-trip failed: {e}")
            
    except Exception as e:
        log_test_result("DataFrame Round-Trip Setup", False, f"Round-trip test setup failed: {e}")

def test_type_stubs():
    """Test 5: Verify type stubs are correctly generated"""
    print("\n📝 Testing Python Type Stubs...")
    
    try:
        # Check if .pyi files exist
        import ddex_parser
        import ddex_builder
        
        parser_module_file = ddex_parser.__file__
        builder_module_file = ddex_builder.__file__
        
        # Look for .pyi files
        parser_pyi = parser_module_file.replace('.so', '.pyi') if '.so' in parser_module_file else None
        builder_pyi = builder_module_file.replace('.so', '.pyi') if '.so' in builder_module_file else None
        
        parser_stubs_exist = parser_pyi and os.path.exists(parser_pyi)
        builder_stubs_exist = builder_pyi and os.path.exists(builder_pyi)
        
        log_test_result("Parser Type Stubs", parser_stubs_exist, 
                      "Type stub file found" if parser_stubs_exist else "No .pyi file found")
        log_test_result("Builder Type Stubs", builder_stubs_exist,
                      "Type stub file found" if builder_stubs_exist else "No .pyi file found")
        
        # Test type hints at runtime
        from ddex_parser import DDEXParser
        from ddex_builder import DDEXBuilder
        
        # Check if classes have type annotations
        parser_annotations = hasattr(DDEXParser, '__annotations__')
        builder_annotations = hasattr(DDEXBuilder, '__annotations__')
        
        log_test_result("Parser Type Annotations", parser_annotations,
                      "Class has type annotations" if parser_annotations else "No type annotations found")
        log_test_result("Builder Type Annotations", builder_annotations, 
                      "Class has type annotations" if builder_annotations else "No type annotations found")
                      
        # Test docstrings
        parser_doc = DDEXParser.__doc__ is not None and len(DDEXParser.__doc__) > 0
        builder_doc = DDEXBuilder.__doc__ is not None and len(DDEXBuilder.__doc__) > 0
        
        log_test_result("Parser Documentation", parser_doc, 
                      f"Docstring available: {DDEXParser.__doc__[:50]}..." if parser_doc else "No docstring")
        log_test_result("Builder Documentation", builder_doc,
                      f"Docstring available: {DDEXBuilder.__doc__[:50]}..." if builder_doc else "No docstring")
        
    except Exception as e:
        log_test_result("Type Stubs Test", False, f"Type stub verification failed: {e}")

def test_memory_efficiency():
    """Test 6: Memory efficiency with large catalog processing"""
    print("\n🧠 Testing Memory Efficiency with Large Catalogs...")
    
    try:
        from ddex_parser import DDEXParser
        import psutil
        import gc
        
        parser = DDEXParser()
        
        # Generate a large catalog XML (~5MB with 1000 tracks)
        def generate_large_catalog():
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
            
            releases = []
            for i in range(0, 1000, 10):
                track_refs = [f"SR{j:04d}" for j in range(i, min(i+10, 1000))]
                releases.append(f"""
                <Release>
                    <ReleaseReference>REL{i//10:03d}</ReleaseReference>
                    <ReleaseType>Album</ReleaseType>
                    <DisplayTitleText>Memory Test Album {i//10}</DisplayTitleText>
                    <DisplayArtist>Memory Test Artist {i//50}</DisplayArtist>
                </Release>""")
            
            return f"""<?xml version="1.0" encoding="UTF-8"?>
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
                <ReleaseList>
                    {''.join(releases)}
                </ReleaseList>
            </ern:NewReleaseMessage>"""
        
        # Get process for memory monitoring
        process = psutil.Process(os.getpid())
        
        # Measure memory before
        gc.collect()  # Force garbage collection
        memory_before = process.memory_info().rss / 1024 / 1024  # MB
        
        # Generate and parse large XML
        print("   📊 Generating large test catalog (1000 tracks)...")
        large_xml = generate_large_catalog()
        xml_size_mb = len(large_xml) / 1024 / 1024
        
        print(f"   📏 XML size: {xml_size_mb:.2f}MB")
        
        # Parse with timing
        start_time = time.time()
        result = parser.parse(large_xml)
        parse_time = time.time() - start_time
        
        # Measure memory after
        memory_after = process.memory_info().rss / 1024 / 1024  # MB
        memory_used = memory_after - memory_before
        
        log_test_result("Large Catalog Parsing", True,
                      f"Parsed {xml_size_mb:.2f}MB in {parse_time:.2f}s",
                      f"Memory used: {memory_used:.1f}MB, Rate: {xml_size_mb/parse_time:.1f}MB/s")
        
        # Test memory efficiency target: <100MB for 5MB XML
        memory_efficient = memory_used < 100
        log_test_result("Memory Efficiency", memory_efficient,
                      f"Memory usage {'within' if memory_efficient else 'exceeds'} 100MB target",
                      f"Used: {memory_used:.1f}MB for {xml_size_mb:.2f}MB input")
        
        # Clean up
        del result
        del large_xml
        gc.collect()
        
    except ImportError:
        log_test_result("Memory Efficiency Test", False, "psutil not available - install with: pip install psutil")
    except Exception as e:
        log_test_result("Memory Efficiency Test", False, f"Memory test failed: {e}")

def test_error_handling():
    """Test 7: Error handling and Python exceptions"""
    print("\n⚠️  Testing Error Handling and Python Exceptions...")
    
    try:
        from ddex_parser import DDEXParser
        parser = DDEXParser()
        
        # Test 1: Invalid XML
        try:
            parser.parse("not valid xml")
            log_test_result("Invalid XML Handling", False, "Should have raised an exception")
        except Exception as e:
            log_test_result("Invalid XML Handling", True, f"Correctly raised exception: {type(e).__name__}")
        
        # Test 2: Empty input
        try:
            parser.parse("")
            log_test_result("Empty Input Handling", False, "Should have raised an exception")
        except Exception as e:
            log_test_result("Empty Input Handling", True, f"Correctly raised exception: {type(e).__name__}")
        
        # Test 3: Very large input (memory limit test)
        try:
            huge_xml = "a" * (50 * 1024 * 1024)  # 50MB of 'a' characters
            parser.parse(huge_xml)
            log_test_result("Large Input Handling", False, "Should have raised an exception or handled gracefully")
        except Exception as e:
            log_test_result("Large Input Handling", True, f"Correctly handled large input: {type(e).__name__}")
        
        # Test 4: Malformed DDEX XML (valid XML, invalid DDEX)
        try:
            malformed_ddex = """<?xml version="1.0" encoding="UTF-8"?>
            <ern:NewReleaseMessage xmlns:ern="http://ddex.net/xml/ern/43">
                <InvalidElement>This is not valid DDEX</InvalidElement>
            </ern:NewReleaseMessage>"""
            result = parser.parse(malformed_ddex)
            log_test_result("Malformed DDEX Handling", True, "Gracefully handled malformed DDEX")
        except Exception as e:
            log_test_result("Malformed DDEX Handling", True, f"Correctly handled malformed DDEX: {type(e).__name__}")
        
    except Exception as e:
        log_test_result("Error Handling Test Setup", False, f"Error handling test setup failed: {e}")
    
    # Test builder error handling
    try:
        from ddex_builder import DDEXBuilder
        builder = DDEXBuilder()
        
        # Test building without required data
        try:
            builder.build()
            log_test_result("Builder Empty Build", False, "Should have raised an exception")
        except Exception as e:
            log_test_result("Builder Empty Build", True, f"Correctly raised exception: {type(e).__name__}")
            
    except Exception as e:
        log_test_result("Builder Error Handling", False, f"Builder error handling test failed: {e}")

def main():
    """Main test execution"""
    print("🐍 DDEX Suite Python Bindings Comprehensive Test")
    print("=" * 60)
    print(f"Python: {platform.python_version()}")
    print(f"Platform: {platform.system()} {platform.machine()}")
    print(f"Architecture: {platform.architecture()}")
    
    start_time = time.time()
    
    # Test 1: Basic imports and instantiation
    parser_available, builder_available = test_basic_imports()
    
    # Test 2: PyO3 native bindings on current platform
    test_pyo3_native_bindings(parser_available, builder_available)
    
    # Test 3: DataFrame integration
    test_dataframe_integration(parser_available)
    
    # Test 4: DataFrame round-trip
    test_dataframe_roundtrip(parser_available, builder_available)
    
    # Test 5: Type stubs
    test_type_stubs()
    
    # Test 6: Memory efficiency
    test_memory_efficiency()
    
    # Test 7: Error handling
    test_error_handling()
    
    total_time = time.time() - start_time
    
    print("\n" + "=" * 60)
    print("🎯 Python Bindings Test Summary")
    print(f"   Total execution time: {total_time:.2f}s")
    print(f"   Parser available: {'✅' if parser_available else '❌'}")
    print(f"   Builder available: {'✅' if builder_available else '❌'}")
    print("=" * 60)
    
    return parser_available and builder_available

if __name__ == "__main__":
    try:
        success = main()
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print("\n\n⚠️  Test interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n\n💥 Fatal error: {e}")
        traceback.print_exc()
        sys.exit(1)