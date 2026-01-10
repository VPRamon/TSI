#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print section headers
print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Variables
DEV_IMAGE_TAG="${DEV_IMAGE_TAG:-tsi-dev:ci}"
DOCKER_MODE="${DOCKER_MODE:-auto}"
USE_DOCKER=false

# Detect if we should use Docker
if [[ "$DOCKER_MODE" == "always" ]]; then
    USE_DOCKER=true
elif [[ "$DOCKER_MODE" == "never" ]]; then
    USE_DOCKER=false
elif [[ -f /.dockerenv ]] || [[ -f /run/.containerenv ]]; then
    # Already inside a container
    USE_DOCKER=false
else
    # Outside container, check if Docker image exists
    if docker image inspect "$DEV_IMAGE_TAG" &>/dev/null; then
        USE_DOCKER=true
    else
        print_warning "Docker image $DEV_IMAGE_TAG not found, running natively"
        USE_DOCKER=false
    fi
fi

# Function to run command in Docker or natively
run_cmd() {
    local cmd="$1"
    if [[ "$USE_DOCKER" == true ]]; then
        docker run --rm \
            --env CI=1 \
            -v "$(pwd)":/workspace \
            -w /workspace \
            "$DEV_IMAGE_TAG" \
            bash -lc "$cmd"
    else
        bash -lc "$cmd"
    fi
}

# Parse command line arguments
RUN_LINTERS=true
RUN_PYTHON_TESTS=true
RUN_RUST=true
TEST_SUBSET=""
SHOW_HELP=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --linters-only)
            RUN_PYTHON_TESTS=false
            RUN_RUST=false
            shift
            ;;
        --tests-only)
            RUN_LINTERS=false
            RUN_RUST=false
            shift
            ;;
        --rust-only)
            RUN_LINTERS=false
            RUN_PYTHON_TESTS=false
            shift
            ;;
        --subset)
            TEST_SUBSET="$2"
            shift 2
            ;;
        --docker)
            USE_DOCKER=true
            shift
            ;;
        --no-docker)
            USE_DOCKER=false
            shift
            ;;
        --help|-h)
            SHOW_HELP=true
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            SHOW_HELP=true
            shift
            ;;
    esac
done

if [[ "$SHOW_HELP" == true ]]; then
    cat << EOF
Usage: $0 [OPTIONS]

Mirror CI execution locally for testing and linting.

OPTIONS:
    --linters-only      Run only Python linters and type checks
    --tests-only        Run only Python tests
    --rust-only         Run only Rust checks
    --subset SUBSET     Run specific test subset (unit|integration|e2e|unmarked|bindings)
    --docker            Force Docker execution
    --no-docker         Force native execution
    -h, --help          Show this help message

EXAMPLES:
    $0                                  # Run everything
    $0 --linters-only                   # Run only linters
    $0 --subset unit                    # Run only unit tests
    $0 --no-docker --subset integration # Run integration tests natively

ENVIRONMENT VARIABLES:
    DEV_IMAGE_TAG       Docker image tag (default: tsi-dev:ci)
    DOCKER_MODE         Docker mode: auto|always|never (default: auto)

EOF
    exit 0
fi

# Show execution mode
if [[ "$USE_DOCKER" == true ]]; then
    print_header "Running in Docker mode (image: $DEV_IMAGE_TAG)"
else
    print_header "Running in native mode"
fi

# Exit status tracking
FAILED_STEPS=()

# ============================================
# Python Quality Gates
# ============================================
if [[ "$RUN_LINTERS" == true ]]; then
    print_header "Python Quality Gates"
    
    # Ruff check
    echo "Running ruff check..."
    if run_cmd "ruff check src/ tests/"; then
        print_success "Ruff check passed"
    else
        print_error "Ruff check failed"
        FAILED_STEPS+=("ruff")
    fi
    
    # Black check
    echo "Running black check..."
    if run_cmd "black --check src/ tests/"; then
        print_success "Black formatting check passed"
    else
        print_error "Black formatting check failed"
        FAILED_STEPS+=("black")
    fi
    
    # MyPy type checking
    echo "Running mypy..."
    if run_cmd "mypy src/"; then
        print_success "MyPy type check passed"
    else
        print_error "MyPy type check failed"
        FAILED_STEPS+=("mypy")
    fi
fi

# ============================================
# Python Tests
# ============================================
if [[ "$RUN_PYTHON_TESTS" == true ]]; then
    print_header "Python Tests"
    
    # Determine which subsets to run
    if [[ -n "$TEST_SUBSET" ]]; then
        SUBSETS=("$TEST_SUBSET")
    else
        SUBSETS=(unit integration e2e unmarked bindings)
    fi
    
    for subset in "${SUBSETS[@]}"; do
        echo "Running $subset tests..."
        
        case "$subset" in
            unit)
                if run_cmd "pytest -m unit --cov=src --cov-report=xml --cov-report=html --cov-report=term-missing:skip-covered"; then
                    print_success "Unit tests passed"
                else
                    print_error "Unit tests failed"
                    FAILED_STEPS+=("pytest-unit")
                fi
                ;;
            integration)
                # Run integration tests; exit code 5 means no tests collected which is OK
                set +e
                run_cmd "pytest -m integration --no-cov"
                PYTEST_EXIT=$?
                set -e
                if [[ $PYTEST_EXIT -eq 0 || $PYTEST_EXIT -eq 5 ]]; then
                    print_success "Integration tests passed"
                else
                    print_error "Integration tests failed"
                    FAILED_STEPS+=("pytest-integration")
                fi
                ;;
            e2e)
                if run_cmd "pytest -m e2e --no-cov"; then
                    print_success "E2E tests passed"
                else
                    print_error "E2E tests failed"
                    FAILED_STEPS+=("pytest-e2e")
                fi
                ;;
            unmarked)
                if run_cmd "pytest -m 'not unit and not integration and not e2e' --no-cov"; then
                    print_success "Unmarked tests passed"
                else
                    print_error "Unmarked tests failed"
                    FAILED_STEPS+=("pytest-unmarked")
                fi
                ;;
            bindings)
                # Run Python bindings tests; exit code 5 means no tests collected which is OK
                set +e
                run_cmd "pytest backend/tests --no-cov"
                PYTEST_EXIT=$?
                set -e
                if [[ $PYTEST_EXIT -eq 0 || $PYTEST_EXIT -eq 5 ]]; then
                    print_success "Bindings tests passed"
                else
                    print_error "Bindings tests failed"
                    FAILED_STEPS+=("pytest-bindings")
                fi
                ;;
            *)
                print_error "Unknown test subset: $subset"
                FAILED_STEPS+=("pytest-$subset")
                ;;
        esac
    done
fi

# ============================================
# Rust Checks
# ============================================
if [[ "$RUN_RUST" == true ]]; then
    print_header "Rust Checks"
    
    # Cargo fmt
    echo "Running cargo fmt..."
    if run_cmd "cd backend && cargo fmt --all --check"; then
        print_success "Cargo fmt check passed"
    else
        print_error "Cargo fmt check failed"
        FAILED_STEPS+=("cargo-fmt")
    fi
    
    # Cargo clippy
    echo "Running cargo clippy..."
    if run_cmd "cd backend && cargo clippy --all-targets --all-features -- -D warnings"; then
        print_success "Cargo clippy passed"
    else
        print_error "Cargo clippy failed"
        FAILED_STEPS+=("cargo-clippy")
    fi
    
    # Cargo test
    echo "Running cargo test..."
    if run_cmd "cd backend && cargo test --no-default-features --features local-repo"; then
        print_success "Cargo tests passed"
    else
        print_error "Cargo tests failed"
        FAILED_STEPS+=("cargo-test")
    fi
fi

# ============================================
# Summary
# ============================================
print_header "Summary"

if [[ ${#FAILED_STEPS[@]} -eq 0 ]]; then
    print_success "All checks passed! ✨"
    exit 0
else
    print_error "The following checks failed:"
    for step in "${FAILED_STEPS[@]}"; do
        echo "  - $step"
    done
    exit 1
fi
