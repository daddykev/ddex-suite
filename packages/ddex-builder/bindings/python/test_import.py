#!/usr/bin/env python3
"""
Simple test script to verify Python bindings work correctly.
"""

def test_basic_import():
    """Test that the module imports correctly."""
    try:
        import ddex_builder
        print("✓ ddex_builder module imported successfully")
        return True
    except ImportError as e:
        print(f"✗ Failed to import ddex_builder: {e}")
        return False

def test_class_creation():
    """Test creating instances of the main classes."""
    import ddex_builder
    
    try:
        # Test DdexBuilder creation
        builder = ddex_builder.DdexBuilder()
        print("✓ DdexBuilder instance created successfully")
        
        # Test Release creation with required parameters
        release = ddex_builder.Release(
            release_id="TEST001",
            release_type="Album", 
            title="Test Album",
            artist="Test Artist"
        )
        print("✓ Release instance created successfully")
        
        # Test Resource creation with required parameters
        resource = ddex_builder.Resource(
            resource_id="RES001",
            resource_type="SoundRecording",
            title="Test Track",
            artist="Test Artist"
        )
        print("✓ Resource instance created successfully")
        
        return True
    except Exception as e:
        print(f"✗ Failed to create class instances: {e}")
        return False

def test_available_methods():
    """Test that expected methods are available."""
    import ddex_builder
    
    builder = ddex_builder.DdexBuilder()
    expected_methods = ['add_release', 'add_resource', 'apply_preset', 'build', 'get_available_presets']
    
    available_methods = [method for method in dir(builder) if not method.startswith('_')]
    print(f"Available methods: {available_methods}")
    
    missing_methods = []
    for method in expected_methods:
        if method not in available_methods:
            missing_methods.append(method)
    
    if missing_methods:
        print(f"✗ Missing expected methods: {missing_methods}")
        return False
    else:
        print("✓ All expected methods are available")
        return True

def test_module_attributes():
    """Test that module has expected attributes."""
    import ddex_builder
    
    expected_classes = ['DdexBuilder', 'Release', 'Resource', 'ValidationResult']
    available_attrs = dir(ddex_builder)
    
    missing_attrs = []
    for attr in expected_classes:
        if attr not in available_attrs:
            missing_attrs.append(attr)
    
    if missing_attrs:
        print(f"✗ Missing expected module attributes: {missing_attrs}")
        return False
    else:
        print("✓ All expected module attributes are available")
        return True

def main():
    """Run all tests."""
    print("Python DDEX Builder Bindings Test")
    print("=" * 40)
    
    tests = [
        test_basic_import,
        test_class_creation,
        test_available_methods,
        test_module_attributes
    ]
    
    passed = 0
    total = len(tests)
    
    for test in tests:
        print(f"\nRunning {test.__name__}...")
        try:
            if test():
                passed += 1
            else:
                print(f"Test {test.__name__} failed")
        except Exception as e:
            print(f"Test {test.__name__} crashed: {e}")
    
    print("\n" + "=" * 40)
    print(f"Results: {passed}/{total} tests passed")
    
    if passed == total:
        print("🎉 All tests passed! Python bindings are working correctly.")
        return 0
    else:
        print("❌ Some tests failed. Check the output above.")
        return 1

if __name__ == "__main__":
    exit(main())