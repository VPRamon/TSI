#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PASSTHRU_DOCKER_ARGS=()

RUN_BACKEND=true
RUN_FRONTEND=true
RUN_E2E=true

BACKEND_MODE="standard" # standard|coverage|all
FRONTEND_LINTERS_ONLY=false
FRONTEND_TESTS_ONLY=false
FRONTEND_SUBSET=""

SHOW_HELP=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --backend)
            RUN_BACKEND=true
            RUN_FRONTEND=false
            RUN_E2E=false
            shift
            ;;
        --frontend)
            RUN_BACKEND=false
            RUN_FRONTEND=true
            RUN_E2E=false
            shift
            ;;
        --e2e)
            RUN_BACKEND=false
            RUN_FRONTEND=false
            RUN_E2E=true
            shift
            ;;
        --all)
            RUN_BACKEND=true
            RUN_FRONTEND=true
            RUN_E2E=true
            shift
            ;;
        --rust-coverage|--coverage)
            BACKEND_MODE="coverage"
            shift
            ;;
        --linters-only)
            FRONTEND_LINTERS_ONLY=true
            FRONTEND_TESTS_ONLY=false
            shift
            ;;
        --tests-only)
            FRONTEND_TESTS_ONLY=true
            FRONTEND_LINTERS_ONLY=false
            shift
            ;;
        --subset)
            FRONTEND_SUBSET="${2:-}"
            shift 2
            ;;
        --docker|--no-docker)
            # Pass through to sub-scripts.
            PASSTHRU_DOCKER_ARGS+=("$1")
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

Run CI checks locally, split into backend (Rust), frontend (Python/Streamlit), and e2e.

Selection:
  --all               Run backend + frontend + e2e (default)
  --backend           Run backend only
  --frontend          Run frontend only
  --e2e               Run e2e only

Backend options:
  --rust-coverage     Run Rust coverage (nightly + cargo-llvm-cov)

Frontend options:
  --linters-only      Run ruff/black/mypy only
  --tests-only        Run pytest only
  --subset SUBSET     unit|integration|unmarked|all

Execution:
  --docker            Force Docker execution
  --no-docker         Force native execution

Env:
  DEV_IMAGE_TAG       Docker image tag (default: tsi-dev:ci)
  DOCKER_MODE         auto|always|never (default: auto)

EOF
    exit 0
fi

FAILED=()

if [[ "$RUN_FRONTEND" == true ]]; then
    FRONTEND_ARGS=()
    if [[ "$FRONTEND_LINTERS_ONLY" == true ]]; then
        FRONTEND_ARGS+=("--linters-only")
    fi
    if [[ "$FRONTEND_TESTS_ONLY" == true ]]; then
        FRONTEND_ARGS+=("--tests-only")
    fi
    if [[ -n "$FRONTEND_SUBSET" ]]; then
        FRONTEND_ARGS+=("--subset" "$FRONTEND_SUBSET")
    fi

    if ! "${SCRIPT_DIR}/ci_frontend.sh" "${PASSTHRU_DOCKER_ARGS[@]}" "${FRONTEND_ARGS[@]}"; then
        FAILED+=("frontend")
    fi
fi

if [[ "$RUN_BACKEND" == true ]]; then
    BACKEND_ARGS=()
    if [[ "$BACKEND_MODE" == "coverage" ]]; then
        BACKEND_ARGS+=("coverage")
    elif [[ "$BACKEND_MODE" == "all" ]]; then
        BACKEND_ARGS+=("all")
    fi

    if ! "${SCRIPT_DIR}/ci_backend.sh" "${PASSTHRU_DOCKER_ARGS[@]}" "${BACKEND_ARGS[@]}"; then
        FAILED+=("backend")
    fi
fi

if [[ "$RUN_E2E" == true ]]; then
    if ! "${SCRIPT_DIR}/ci_e2e.sh" "${PASSTHRU_DOCKER_ARGS[@]}"; then
        FAILED+=("e2e")
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
