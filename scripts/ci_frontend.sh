#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
FRONTEND_DIR="${ROOT_DIR}/frontend"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

ci_header() { echo -e "\n${BLUE}=== $1 ===${NC}"; }
ci_success() { echo -e "${GREEN}✓ $1${NC}"; }
ci_error() { echo -e "${RED}✗ $1${NC}"; }
ci_warn() { echo -e "${YELLOW}! $1${NC}"; }

RUN_LINTERS=true
RUN_TYPECHECK=true
RUN_BUILD=false
SHOW_HELP=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --linters-only)
            RUN_TYPECHECK=false
            RUN_BUILD=false
            shift
            ;;
        --typecheck-only)
            RUN_LINTERS=false
            RUN_BUILD=false
            shift
            ;;
        --build)
            RUN_BUILD=true
            shift
            ;;
        -h|--help)
            SHOW_HELP=true
            shift
            ;;
        *)
            ci_error "Unknown option: $1"
            SHOW_HELP=true
            shift
            ;;
    esac
done

if [[ "$SHOW_HELP" == true ]]; then
    cat << EOF
Usage: scripts/ci_frontend.sh [OPTIONS]

React/TypeScript frontend quality gates and build checks.

Options:
  --linters-only      Run ESLint only
  --typecheck-only    Run TypeScript check only
  --build             Also run production build
  -h, --help          Show this help message

EOF
    exit 0
fi

cd "$FRONTEND_DIR"

# Check if node_modules exists
if [[ ! -d "node_modules" ]]; then
    ci_header "Installing dependencies"
    npm ci
fi

FAILED=()

if [[ "$RUN_LINTERS" == true ]]; then
    ci_header "ESLint"
    if npm run lint 2>/dev/null; then
        ci_success "ESLint passed"
    else
        ci_error "ESLint failed"
        FAILED+=("eslint")
    fi
fi

if [[ "$RUN_TYPECHECK" == true ]]; then
    ci_header "TypeScript Check"
    if npm run typecheck 2>/dev/null; then
        ci_success "TypeScript check passed"
    else
        ci_error "TypeScript check failed"
        FAILED+=("typecheck")
    fi
fi

if [[ "$RUN_BUILD" == true ]]; then
    ci_header "Production Build"
    if npm run build; then
        ci_success "Build succeeded"
    else
        ci_error "Build failed"
        FAILED+=("build")
    fi
fi

ci_header "Summary"
if [[ ${#FAILED[@]} -eq 0 ]]; then
    ci_success "Frontend checks passed"
    exit 0
else
    ci_error "Frontend checks failed:"
    for step in "${FAILED[@]}"; do
        echo "  - ${step}"
    done
    exit 1
fi
