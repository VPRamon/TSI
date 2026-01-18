#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PROJECT_ROOT

print_header() {
    echo -e "\n${GREEN}=== $1 ===${NC}\n"
}

check_result() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ $1 passed${NC}"
    else
        echo -e "${RED}✗ $1 failed${NC}"
        exit 1
    fi
}

show_help() {
    echo "Usage: ./ci.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --quick               Skip long-running tests"
    echo "  --fix                 Auto-fix formatting and linting issues where possible"
    echo "  --backend-only        Run only backend checks"
    echo "  --frontend-only       Run only frontend checks"
    echo "  --help                Show this help message"
    echo ""
}

# Parse arguments
QUICK=false
FIX=false
BACKEND_ONLY=false
FRONTEND_ONLY=false

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --quick) QUICK=true ;;
        --fix) FIX=true ;;
        --backend-only) BACKEND_ONLY=true ;;
        --frontend-only) FRONTEND_ONLY=true ;;
        --help) show_help; exit 0 ;;
        *) echo "Unknown parameter: $1"; show_help; exit 1 ;;
    esac
    shift
done

# Run Backend CI
if [ "$FRONTEND_ONLY" = false ]; then
    print_header "Running Backend CI"
    
    BACKEND_ARGS=""
    if [ "$FIX" = true ]; then
        BACKEND_ARGS="$BACKEND_ARGS --fix"
    fi
    
    # We don't have a direct equivalent for --quick in the rust script yet, 
    # but we can pass it if we update ci_backend.sh to handle it.
    # For now, let's just run it.
    
    "$PROJECT_ROOT/scripts/ci_backend.sh" $BACKEND_ARGS
    check_result "Backend CI"
fi

# Run Frontend CI
if [ "$BACKEND_ONLY" = false ]; then
    print_header "Running Frontend CI"
    
    FRONTEND_ARGS=""
    if [ "$FIX" = true ]; then
        FRONTEND_ARGS="$FRONTEND_ARGS --fix"
    fi
    
    "$PROJECT_ROOT/scripts/ci_frontend.sh" $FRONTEND_ARGS
    check_result "Frontend CI"
fi

print_header "All CI checks passed successfully!"
