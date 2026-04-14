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
    echo "  --backend-only        Run only backend checks"
    echo "  --frontend-only       Run only frontend checks"
    echo "  --frontend-build      Also run the frontend production build"
    echo "  --backend-coverage    Run backend coverage instead of standard checks"
    echo "  --backend-all         Run backend standard checks and coverage"
    echo "  --help                Show this help message"
    echo ""
}

# Parse arguments
BACKEND_ONLY=false
FRONTEND_ONLY=false
FRONTEND_BUILD=false
BACKEND_MODE="standard"

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --backend-only) BACKEND_ONLY=true ;;
        --frontend-only) FRONTEND_ONLY=true ;;
        --frontend-build) FRONTEND_BUILD=true ;;
        --backend-coverage) BACKEND_MODE="coverage" ;;
        --backend-all) BACKEND_MODE="all" ;;
        --help) show_help; exit 0 ;;
        *) echo "Unknown parameter: $1"; show_help; exit 1 ;;
    esac
    shift
done

# Run Backend CI
if [ "$FRONTEND_ONLY" = false ]; then
    print_header "Running Backend CI"

    "$PROJECT_ROOT/scripts/ci_backend.sh" "$BACKEND_MODE"
    check_result "Backend CI"
fi

# Run Frontend CI
if [ "$BACKEND_ONLY" = false ]; then
    print_header "Running Frontend CI"

    FRONTEND_ARGS=()
    if [ "$FRONTEND_BUILD" = true ]; then
        FRONTEND_ARGS+=(--build)
    else
        FRONTEND_ARGS+=(--quality-gates)
    fi

    "$PROJECT_ROOT/scripts/ci_frontend.sh" "${FRONTEND_ARGS[@]}"
    check_result "Frontend CI"
fi

print_header "All CI checks passed successfully!"
