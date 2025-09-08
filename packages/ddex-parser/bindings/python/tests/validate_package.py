# validate_package.py - Quick package validation
"""Validate the built Python package."""

import subprocess
import sys
import tempfile
import os
from pathlib import Path

def validate_package():
    """Validate the Python package is correctly built."""
    
    print("🔍 Validating Python Package Structure")
    print("=" * 50)
    
    # Check package directory
    pkg_dir = Path("packages/ddex-parser/bindings/python")
    if not pkg_dir.exists():
        print(f"❌ Package directory not found: {pkg_dir}")
        return False
    
    os.chdir(pkg_dir)
    
    # Check required files
    required_files = [
        "Cargo.toml",
        "pyproject.toml",
        "src/lib.rs",
        "python/ddex_parser/__init__.py",
    ]
    
    print("\n📁 Checking required files:")
    for file in required_files:
        if Path(file).exists():
            print(f"  ✓ {file}")
        else:
            print(f"  ✗ {file} - MISSING!")
            return False
    
    # Build package
    print("\n🔨 Building package with maturin...")
    result = subprocess.run(
        ["maturin", "build", "--release"],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"❌ Build failed:\n{result.stderr}")
        return False
    
    print("✓ Package built successfully")
    
    # Check wheel was created
    dist_dir = Path("dist")
    if dist_dir.exists():
        wheels = list(dist_dir.glob("*.whl"))
        if wheels:
            print(f"\n📦 Created wheel: {wheels[0].name}")
            print(f"   Size: {wheels[0].stat().st_size / 1024:.1f} KB")
        else:
            print("❌ No wheel file created")
            return False
    
    # Test in isolated environment
    print("\n🧪 Testing in isolated environment...")
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create virtual environment
        venv_dir = Path(tmpdir) / "venv"
        subprocess.run([sys.executable, "-m", "venv", str(venv_dir)], check=True)
        
        # Install wheel
        pip_path = venv_dir / ("Scripts" if sys.platform == "win32" else "bin") / "pip"
        python_path = venv_dir / ("Scripts" if sys.platform == "win32" else "bin") / "python"
        
        wheel_path = wheels[0] if wheels else None
        if wheel_path:
            subprocess.run([str(pip_path), "install", str(wheel_path)], check=True)
            
            # Test import
            test_code = """
import sys
from ddex_parser import DDEXParser, __version__
parser = DDEXParser()
print(f'Version: {__version__}')
print('✓ Import successful!')
"""
            result = subprocess.run(
                [str(python_path), "-c", test_code],
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                print(result.stdout)
            else:
                print(f"❌ Import test failed:\n{result.stderr}")
                return False
    
    print("\n✅ Package validation complete!")
    return True

if __name__ == "__main__":
    success = validate_package()
    sys.exit(0 if success else 1)