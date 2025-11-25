#!/bin/bash
# Build script for TSI Rust Backend
# Usage: ./build_rust.sh [--dev|--release|--test]

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if running from correct directory
if [ ! -f "Cargo.toml" ] && [ ! -f "rust_backend/Cargo.toml" ]; then
    print_error "Must run from project root or rust_backend directory"
    exit 1
fi

# Move to project root if in rust_backend
if [ -f "rust_backend/Cargo.toml" ]; then
    cd ..
fi

# Parse arguments
MODE="release"
RUN_TESTS=false

for arg in "$@"; do
    case $arg in
        --dev)
            MODE="dev"
            ;;
        --release)
            MODE="release"
            ;;
        --test)
            RUN_TESTS=true
            ;;
        --help|-h)
            echo "TSI Rust Backend Build Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dev        Build in development mode (faster compile, slower runtime)"
            echo "  --release    Build in release mode (default, slower compile, faster runtime)"
            echo "  --test       Run tests after building"
            echo "  --help, -h   Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $arg"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

print_info "Starting TSI Rust Backend build..."
echo ""

# Check prerequisites
print_info "Checking prerequisites..."

# Check for Rust
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed"
    echo ""
    echo "Install Rust from: https://rustup.rs/"
    echo "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
print_success "Rust found: $(rustc --version)"

# Check for Python
if ! command -v python3 &> /dev/null; then
    print_error "Python 3 is not installed"
    exit 1
fi
PYTHON_VERSION=$(python3 --version)
print_success "Python found: $PYTHON_VERSION"

# Check for Python development headers
if ! python3 -c "import sysconfig" 2>/dev/null; then
    print_warning "Python development headers may not be installed"
    echo "On Ubuntu/Debian: sudo apt-get install python3-dev"
    echo "On Fedora/RHEL: sudo dnf install python3-devel"
    echo "On macOS: brew install python"
fi

# Check for maturin
if ! command -v maturin &> /dev/null; then
    print_warning "Maturin is not installed"
    echo "Installing maturin..."
    pip install maturin
    if [ $? -eq 0 ]; then
        print_success "Maturin installed successfully"
    else
        print_error "Failed to install maturin"
        exit 1
    fi
else
    print_success "Maturin found: $(maturin --version)"
fi

echo ""

# Run tests if requested
if [ "$RUN_TESTS" = true ]; then
    print_info "Running Rust tests..."
    cd rust_backend
    cargo test --release 2>&1 | grep -E "(test result:|running|PASSED|FAILED)" || true
    cd ..
    echo ""
fi

# Build the Rust backend
print_info "Building Rust backend in $MODE mode..."
echo ""

if [ "$MODE" = "release" ]; then
    maturin develop --release
else
    maturin develop
fi

BUILD_EXIT_CODE=$?

echo ""

if [ $BUILD_EXIT_CODE -eq 0 ]; then
    print_success "Build completed successfully!"
    echo ""
    print_info "Testing the build..."
    
    # Test import
    if python3 -c "import tsi_rust; print(f'✅ tsi_rust module imported successfully')" 2>/dev/null; then
        print_success "Rust backend is ready to use!"
        echo ""
        echo "You can now run the dashboard with:"
        echo "  ./run_dashboard.sh"
    else
        print_warning "Build succeeded but import failed"
        echo "You may need to activate your Python virtual environment"
    fi
else
    print_error "Build failed with exit code $BUILD_EXIT_CODE"
    echo ""
    print_info "Common issues:"
    echo "  1. Missing Python development headers"
    echo "     Ubuntu/Debian: sudo apt-get install python3-dev"
    echo "     Fedora/RHEL: sudo dnf install python3-devel"
    echo "  2. Wrong Python version - PyO3 requires Python 3.7+"
    echo "  3. Missing system libraries - check error messages above"
    exit 1
fi
