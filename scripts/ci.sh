#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

RUN_BACKEND=true
RUN_FRONTEND=true

BACKEND_MODE="standard" # standard|coverage
FRONTEND_LINTERS_ONLY=false
FRONTEND_TYPECHECK_ONLY=false

SHOW_HELP=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --backend)
            RUN_BACKEND=true
            RUN_FRONTEND=false
            shift
            ;;
        --frontend)
            RUN_BACKEND=false
            RUN_FRONTEND=true
            shift
            ;;
        --all)
            RUN_BACKEND=true
            RUN_FRONTEND=true
            shift
            ;;
        --rust-coverage|--coverage)
            BACKEND_MODE="coverage"
            shift
            ;;
        --linters-only)
            FRONTEND_LINTERS_ONLY=true
            FRONTEND_TYPECHECK_ONLY=false
            shift
            ;;
        --typecheck-only)
            FRONTEND_TYPECHECK_ONLY=true
            FRONTEND_LINTERS_ONLY=false
            shift
            ;;
        -h|--help)
            SHOW_HELP=true
            shift
            ;;
        *)
            echo "Unknown option: $1" >&2
            SHOW_HELP=true
            shift
            ;;
    esac
done

if [[ "$SHOW_HELP" == true ]]; then
    cat << EOF
Usage: scripts/ci.sh [OPTIONS]

Run CI checks locally for backend (Rust) and frontend (React/TypeScript).

Selection:
  --all               Run backend + frontend (default)
  --backend           Run backend only
  --frontend          Run frontend only

Backend options:
  --rust-coverage     Run Rust coverage (nightly + cargo-llvm-cov)

Frontend options:
  --linters-only      Run ESLint only
  --typecheck-only    Run TypeScript check only

EOF
    exit 0
fi

FAILED=()

if [[ "$RUN_FRONTEND" == true ]]; then
    FRONTEND_ARGS=()
    if [[ "$FRONTEND_LINTERS_ONLY" == true ]]; then
        FRONTEND_ARGS+=("--linters-only")
    fi
    if [[ "$FRONTEND_TYPECHECK_ONLY" == true ]]; then
        FRONTEND_ARGS+=("--typecheck-only")
    fi

    if ! "${SCRIPT_DIR}/ci_frontend.sh" "${FRONTEND_ARGS[@]+"${FRONTEND_ARGS[@]}"}"; then
        FAILED+=("frontend")
    fi
fi

if [[ "$RUN_BACKEND" == true ]]; then
    BACKEND_ARGS=()
    if [[ "$BACKEND_MODE" == "coverage" ]]; then
        BACKEND_ARGS+=("coverage")
    fi

    if ! "${SCRIPT_DIR}/ci_backend.sh" "${BACKEND_ARGS[@]+"${BACKEND_ARGS[@]}"}"; then
        FAILED+=("backend")
    fi
fi

echo ""
if [[ ${#FAILED[@]} -eq 0 ]]; then
    echo "All CI checks passed"
    exit 0
else
    echo "The following groups failed:" >&2
    for group in "${FAILED[@]}"; do
        echo "  - ${group}" >&2
    done
    exit 1
fi
