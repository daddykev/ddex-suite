import sys
sys.path.insert(0, './python/target/release')

from ddex_builder import DDEXBuilder

def test_python_bindings():
    print("Testing Python bindings...")
    
    builder = DDEXBuilder()
    
    request = {
        "releases": [{
            "releaseId": "R1",
            "title": "Test Album",
            "displayArtist": "Test Artist",
            "tracks": []
        }],
        "profile": "AudioAlbum",
        "version": "4.3"
    }
    
    result = builder.build(request, None)
    
    assert "<?xml" in result["xml"]
    assert "Test Album" in result["xml"]
    
    print("✓ Python bindings working")

if __name__ == "__main__":
    test_python_bindings()