#!/bin/bash

# DDEX Builder Bindings Build Script
# Builds all three language bindings: Node.js, Python, and WASM

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BUILD_NODE=${BUILD_NODE:-true}
BUILD_PYTHON=${BUILD_PYTHON:-true}
BUILD_WASM=${BUILD_WASM:-true}
SKIP_TESTS=${SKIP_TESTS:-false}
RELEASE_MODE=${RELEASE_MODE:-false}

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to get script directory
get_script_dir() {
    cd "$(dirname "${BASH_SOURCE[0]}")" && pwd
}

# Function to check system dependencies
check_dependencies() {
    print_status "Checking system dependencies..."
    
    # Check Rust
    if ! command_exists rustc; then
        print_error "Rust is required but not installed. Install from https://rustup.rs/"
        exit 1
    fi
    
    # Check cargo
    if ! command_exists cargo; then
        print_error "Cargo is required but not installed."
        exit 1
    fi
    
    print_success "Rust toolchain found: $(rustc --version)"
    
    # Check Node.js dependencies
    if [ "$BUILD_NODE" = true ]; then
        if ! command_exists node; then
            print_warning "Node.js not found. Skipping Node.js binding."
            BUILD_NODE=false
        else
            if ! command_exists npm; then
                print_error "npm is required for Node.js binding"
                exit 1
            fi
            print_success "Node.js found: $(node --version)"
        fi
    fi
    
    # Check Python dependencies
    if [ "$BUILD_PYTHON" = true ]; then
        if ! command_exists python3; then
            print_warning "Python 3 not found. Skipping Python binding."
            BUILD_PYTHON=false
        else
            if ! command_exists maturin; then
                print_warning "maturin not found. Installing..."
                if command_exists pip3; then
                    pip3 install maturin
                else
                    print_error "pip3 is required to install maturin"
                    BUILD_PYTHON=false
                fi
            fi
            if [ "$BUILD_PYTHON" = true ]; then
                print_success "Python found: $(python3 --version)"
            fi
        fi
    fi
    
    # Check WASM dependencies
    if [ "$BUILD_WASM" = true ]; then
        if ! command_exists wasm-pack; then
            print_warning "wasm-pack not found. Installing..."
            if command_exists curl; then
                curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
            else
                print_error "curl is required to install wasm-pack"
                BUILD_WASM=false
            fi
        fi
        
        # Check for wasm32 target
        if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
            print_status "Adding wasm32-unknown-unknown target..."
            rustup target add wasm32-unknown-unknown
        fi
        
        if [ "$BUILD_WASM" = true ]; then
            print_success "WASM toolchain ready"
        fi
    fi
}

# Function to build Node.js binding
build_node() {
    print_status "Building Node.js binding..."
    
    cd node/
    
    # Install dependencies
    print_status "Installing Node.js dependencies..."
    npm install
    
    # Build native module
    print_status "Building native module..."
    if [ "$RELEASE_MODE" = true ]; then
        npm run build:release
    else
        npm run build
    fi
    
    # Run tests
    if [ "$SKIP_TESTS" != true ]; then
        print_status "Running Node.js tests..."
        npm test
    fi
    
    # Check bundle size
    if [ -f "ddex-builder.*.node" ]; then
        size=$(du -h ddex-builder.*.node | cut -f1)
        print_success "Node.js binding built successfully (${size})"
    else
        print_error "Node.js binding build failed - no .node file found"
        return 1
    fi
    
    cd ..
}

# Function to build Python binding
build_python() {
    print_status "Building Python binding..."
    
    cd python/
    
    # Build with maturin
    print_status "Building Python wheel..."
    if [ "$RELEASE_MODE" = true ]; then
        maturin build --release
    else
        maturin develop
    fi
    
    # Run tests
    if [ "$SKIP_TESTS" != true ]; then
        print_status "Running Python tests..."
        python3 test_builder.py
    fi
    
    # Check for wheel file
    if [ -d "target/wheels" ] && [ "$(ls -A target/wheels)" ]; then
        wheel_file=$(ls target/wheels/*.whl | head -1)
        size=$(du -h "$wheel_file" | cut -f1)
        print_success "Python binding built successfully (${size})"
    elif [ "$RELEASE_MODE" != true ]; then
        print_success "Python binding installed in development mode"
    else
        print_error "Python binding build failed - no wheel file found"
        return 1
    fi
    
    cd ..
}

# Function to build WASM binding
build_wasm() {
    print_status "Building WASM binding..."
    
    cd wasm/
    
    # Build with wasm-pack
    print_status "Building WASM package..."
    if [ "$RELEASE_MODE" = true ]; then
        wasm-pack build --target web --out-dir pkg --release
    else
        wasm-pack build --target web --out-dir pkg
    fi
    
    # Check bundle size
    if [ -f "pkg/ddex_builder_wasm_bg.wasm" ]; then
        size=$(du -h pkg/ddex_builder_wasm_bg.wasm | cut -f1)
        size_bytes=$(stat -f%z pkg/ddex_builder_wasm_bg.wasm 2>/dev/null || stat -c%s pkg/ddex_builder_wasm_bg.wasm)
        
        # Check if under 500KB
        if [ "$size_bytes" -lt 512000 ]; then
            print_success "WASM binding built successfully (${size}) ✅ Under 500KB target"
        else
            print_warning "WASM binding built (${size}) ⚠️  Over 500KB target"
        fi
    else
        print_error "WASM binding build failed - no .wasm file found"
        return 1
    fi
    
    # Run basic validation if test.html exists
    if [ -f "test.html" ]; then
        print_status "WASM binding ready for browser testing at test.html"
    fi
    
    cd ..
}

# Function to run comprehensive tests
run_comprehensive_tests() {
    if [ "$SKIP_TESTS" = true ]; then
        return 0
    fi
    
    print_status "Running comprehensive tests..."
    
    # Test workspace build
    print_status "Testing workspace build..."
    cd ../../..
    cargo test --package ddex-builder
    cd packages/ddex-builder/bindings
    
    print_success "All tests passed"
}

# Function to display build summary
display_summary() {
    print_status "Build Summary:"
    echo "================================="
    
    if [ "$BUILD_NODE" = true ]; then
        if [ -f "node/ddex-builder.*.node" ]; then
            size=$(du -h node/ddex-builder.*.node | cut -f1)
            echo "✅ Node.js binding: ${size}"
        else
            echo "❌ Node.js binding: Failed"
        fi
    else
        echo "⏭️  Node.js binding: Skipped"
    fi
    
    if [ "$BUILD_PYTHON" = true ]; then
        if [ -d "python/target/wheels" ] && [ "$(ls -A python/target/wheels)" ]; then
            wheel_file=$(ls python/target/wheels/*.whl | head -1)
            size=$(du -h "$wheel_file" | cut -f1)
            echo "✅ Python binding: ${size}"
        elif maturin list 2>/dev/null | grep -q ddex-builder; then
            echo "✅ Python binding: Development mode"
        else
            echo "❌ Python binding: Failed"
        fi
    else
        echo "⏭️  Python binding: Skipped"
    fi
    
    if [ "$BUILD_WASM" = true ]; then
        if [ -f "wasm/pkg/ddex_builder_wasm_bg.wasm" ]; then
            size=$(du -h wasm/pkg/ddex_builder_wasm_bg.wasm | cut -f1)
            echo "✅ WASM binding: ${size}"
        else
            echo "❌ WASM binding: Failed"
        fi
    else
        echo "⏭️  WASM binding: Skipped"
    fi
    
    echo "================================="
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Build DDEX Builder language bindings"
    echo ""
    echo "Options:"
    echo "  -h, --help           Show this help message"
    echo "  -n, --no-node        Skip Node.js binding"
    echo "  -p, --no-python      Skip Python binding"  
    echo "  -w, --no-wasm        Skip WASM binding"
    echo "  -t, --no-tests       Skip running tests"
    echo "  -r, --release        Build in release mode"
    echo "  -c, --clean          Clean build artifacts first"
    echo ""
    echo "Environment variables:"
    echo "  BUILD_NODE=false     Skip Node.js binding"
    echo "  BUILD_PYTHON=false   Skip Python binding"
    echo "  BUILD_WASM=false     Skip WASM binding"
    echo "  SKIP_TESTS=true      Skip running tests"
    echo "  RELEASE_MODE=true    Build in release mode"
    echo ""
    echo "Examples:"
    echo "  $0                   Build all bindings in debug mode"
    echo "  $0 --release         Build all bindings in release mode"
    echo "  $0 --no-python       Build Node.js and WASM only"
    echo "  $0 --clean --release Clean and build in release mode"
}

# Function to clean build artifacts
clean_build() {
    print_status "Cleaning build artifacts..."
    
    # Clean Rust artifacts
    cargo clean
    
    # Clean Node.js
    if [ -d "node" ]; then
        cd node/
        rm -rf node_modules/
        rm -f ddex-builder.*.node
        rm -f index.d.ts
        cd ..
    fi
    
    # Clean Python
    if [ -d "python" ]; then
        cd python/
        rm -rf target/
        rm -rf build/
        rm -rf dist/
        rm -rf *.egg-info/
        cd ..
    fi
    
    # Clean WASM
    if [ -d "wasm" ]; then
        cd wasm/
        rm -rf pkg/
        rm -rf target/
        cd ..
    fi
    
    print_success "Build artifacts cleaned"
}

# Main function
main() {
    local script_dir
    script_dir=$(get_script_dir)
    cd "$script_dir"
    
    print_status "DDEX Builder Bindings Build Script"
    print_status "Building in: $script_dir"
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -n|--no-node)
                BUILD_NODE=false
                shift
                ;;
            -p|--no-python)
                BUILD_PYTHON=false
                shift
                ;;
            -w|--no-wasm)
                BUILD_WASM=false
                shift
                ;;
            -t|--no-tests)
                SKIP_TESTS=true
                shift
                ;;
            -r|--release)
                RELEASE_MODE=true
                shift
                ;;
            -c|--clean)
                clean_build
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    # Start build process
    local start_time
    start_time=$(date +%s)
    
    # Check dependencies
    check_dependencies
    
    # Track build failures
    local failed_builds=()
    
    # Build bindings
    if [ "$BUILD_NODE" = true ]; then
        if ! build_node; then
            failed_builds+=("Node.js")
        fi
    fi
    
    if [ "$BUILD_PYTHON" = true ]; then
        if ! build_python; then
            failed_builds+=("Python")
        fi
    fi
    
    if [ "$BUILD_WASM" = true ]; then
        if ! build_wasm; then
            failed_builds+=("WASM")
        fi
    fi
    
    # Run comprehensive tests
    if ! run_comprehensive_tests; then
        failed_builds+=("Tests")
    fi
    
    # Calculate build time
    local end_time
    end_time=$(date +%s)
    local build_time=$((end_time - start_time))
    
    # Display results
    echo ""
    display_summary
    echo ""
    print_status "Build completed in ${build_time} seconds"
    
    # Report failures
    if [ ${#failed_builds[@]} -gt 0 ]; then
        print_error "Failed builds: ${failed_builds[*]}"
        exit 1
    else
        print_success "All requested bindings built successfully! 🎉"
    fi
}

# Run main function with all arguments
main "$@"