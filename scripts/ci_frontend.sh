#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ci_common.sh
source "${SCRIPT_DIR}/ci_common.sh"

RUN_LINTERS=true
RUN_TESTS=true
TEST_SUBSET=""
SHOW_HELP=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --linters-only)
            RUN_TESTS=false
            shift
            ;;
        --tests-only)
            RUN_LINTERS=false
            shift
            ;;
        --subset)
            TEST_SUBSET="${2:-}"
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

Python/Streamlit (frontend + library) quality gates and tests.

Options:
  --linters-only      Run ruff/black/mypy only
  --tests-only        Run pytest only
  --subset SUBSET     unit|integration|unmarked|all
  --docker            Force Docker execution (DEV_IMAGE_TAG)
  --no-docker         Force native execution
  -h, --help          Show this help message

Notes:
  - E2E tests are run by scripts/ci_e2e.sh
  - Rust/PyO3 bindings tests are run by scripts/ci_backend.sh

EOF
    exit 0
fi

ci_init_docker
ci_show_mode

FAILED=()

if [[ "$RUN_LINTERS" == true ]]; then
    ci_header "Python Quality Gates"

    if ci_run "ruff check src/ tests/"; then
        ci_success "ruff check passed"
    else
        ci_error "ruff check failed"
        FAILED+=("ruff")
    fi

    if ci_run "black --check src/ tests/"; then
        ci_success "black check passed"
    else
        ci_error "black check failed"
        FAILED+=("black")
    fi

    if ci_run "mypy src/"; then
        ci_success "mypy passed"
    else
        ci_error "mypy failed"
        FAILED+=("mypy")
    fi
fi

if [[ "$RUN_TESTS" == true ]]; then
    ci_header "Python Tests"

    if [[ -n "$TEST_SUBSET" && "$TEST_SUBSET" != "all" ]]; then
        SUBSETS=("$TEST_SUBSET")
    else
        SUBSETS=(unit integration unmarked)
    fi

    for subset in "${SUBSETS[@]}"; do
        case "$subset" in
            unit)
                if ci_run "pytest -m unit --cov=src --cov-report=xml --cov-report=html --cov-report=term-missing:skip-covered"; then
                    ci_success "unit tests passed"
                else
                    ci_error "unit tests failed"
                    FAILED+=("pytest-unit")
                fi
                ;;
            integration)
                set +e
                ci_run "pytest -m integration --no-cov"
                PYTEST_EXIT=$?
                set -e
                if [[ $PYTEST_EXIT -eq 0 || $PYTEST_EXIT -eq 5 ]]; then
                    ci_success "integration tests passed"
                else
                    ci_error "integration tests failed"
                    FAILED+=("pytest-integration")
                fi
                ;;
            unmarked)
                set +e
                ci_run "pytest -m 'not unit and not integration and not e2e' --no-cov"
                PYTEST_EXIT=$?
                set -e
                if [[ $PYTEST_EXIT -eq 0 || $PYTEST_EXIT -eq 5 ]]; then
                    ci_success "unmarked tests passed"
                else
                    ci_error "unmarked tests failed"
                    FAILED+=("pytest-unmarked")
                fi
                ;;
            e2e)
                ci_warn "subset 'e2e' belongs to scripts/ci_e2e.sh; skipping"
                ;;
            bindings)
                ci_warn "subset 'bindings' belongs to scripts/ci_backend.sh; skipping"
                ;;
            *)
                ci_error "Unknown subset: $subset"
                FAILED+=("pytest-$subset")
                ;;
        esac
    done
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
