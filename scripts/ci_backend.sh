#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ci_common.sh
source "${SCRIPT_DIR}/ci_common.sh"

MODE="standard"  # standard|coverage|all
RUN_BINDINGS=true
RUN_RUST=true

SHOW_HELP=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        standard|coverage|all)
            MODE="$1"
            shift
            ;;
        --coverage)
            MODE="coverage"
            shift
            ;;
        --all)
            MODE="all"
            shift
            ;;
        --no-bindings)
            RUN_BINDINGS=false
            shift
            ;;
        --bindings-only)
            RUN_RUST=false
            shift
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

if [[ "${SHOW_HELP}" == true ]]; then
    cat << EOF
Usage: scripts/ci_backend.sh [standard|coverage|all] [OPTIONS]

Rust backend CI checks (plus optional Python bindings tests).

Modes:
  standard            cargo fmt/check/clippy/test (+ doc tests)
  coverage            rust coverage via cargo-llvm-cov (nightly)
  all                 run standard + coverage

Options:
  --no-bindings       Skip Python bindings tests (pytest backend/tests)
    --bindings-only     Run only Python bindings tests (skip Rust)
  --docker            Force Docker execution (DEV_IMAGE_TAG)
  --no-docker         Force native execution
  -h, --help          Show this help message

Env:
  DEV_IMAGE_TAG       Docker image tag (default: tsi-dev:ci)
  DOCKER_MODE         auto|always|never (default: auto)

EOF
    exit 0
fi

ci_init_docker
ci_show_mode

FAILED=()

run_standard() {
    if [[ "$RUN_RUST" == true ]]; then
        ci_header "Rust Backend Checks"

        if ci_run "cd backend && cargo fmt --all --check"; then
            ci_success "cargo fmt passed"
        else
            ci_error "cargo fmt failed"
            FAILED+=("cargo-fmt")
        fi

        # Match the most complete flavor used in root CI (all features).
        if ci_run "cd backend && cargo check --all-targets --all-features"; then
            ci_success "cargo check passed"
        else
            ci_error "cargo check failed"
            FAILED+=("cargo-check")
        fi

        if ci_run "cd backend && cargo clippy --all-targets --all-features -- -D warnings"; then
            ci_success "cargo clippy passed"
        else
            ci_error "cargo clippy failed"
            FAILED+=("cargo-clippy")
        fi

        if ci_run "cd backend && cargo test --all-targets --all-features"; then
            ci_success "cargo test passed"
        else
            ci_error "cargo test failed"
            FAILED+=("cargo-test")
        fi

        if ci_run "cd backend && cargo test --doc --all-features"; then
            ci_success "cargo doc tests passed"
        else
            ci_error "cargo doc tests failed"
            FAILED+=("cargo-doc-test")
        fi
    fi

    if [[ "$RUN_BINDINGS" == true ]]; then
        ci_header "Python Bindings Tests"
        set +e
        ci_run "pytest backend/tests --no-cov"
        PYTEST_EXIT=$?
        set -e
        if [[ $PYTEST_EXIT -eq 0 || $PYTEST_EXIT -eq 5 ]]; then
            ci_success "bindings tests passed"
        else
            ci_error "bindings tests failed"
            FAILED+=("pytest-bindings")
        fi
    fi
}

run_coverage() {
    ci_header "Rust Coverage"

    # Check prerequisites
    set +e
    ci_run "command -v cargo-llvm-cov &>/dev/null && cargo +nightly llvm-cov --version &>/dev/null"
    HAS_LLVMCOV=$?
    set -e

    if [[ $HAS_LLVMCOV -ne 0 ]]; then
        ci_error "cargo-llvm-cov is not installed or nightly toolchain is missing"
        echo ""
        echo "To install cargo-llvm-cov:"
        echo "  cargo install cargo-llvm-cov"
        echo "To install nightly + llvm-tools-preview:"
        echo "  rustup toolchain install nightly --component llvm-tools-preview"
        FAILED+=("cargo-llvm-cov-missing")
        return 0
    fi

    ci_success "cargo-llvm-cov is available"

    if ci_run "cd backend && cargo +nightly llvm-cov clean --workspace"; then
        ci_success "coverage data cleaned"
    else
        ci_error "failed to clean coverage data"
        FAILED+=("cargo-coverage-clean")
    fi

    # Match backend/ci.sh behavior, ignoring the qtty subcrate.
    if ci_run "cd backend && cargo +nightly llvm-cov --workspace --all-features --doctests --no-report --ignore-filename-regex='qtty/' -- --test-threads=1"; then
        ci_success "coverage data collected"
    else
        ci_error "failed to collect coverage data"
        FAILED+=("cargo-coverage-collect")
    fi

    if ci_run "cd backend && cargo +nightly llvm-cov report --cobertura --output-path coverage.xml --ignore-filename-regex='qtty/'"; then
        ci_success "backend/coverage.xml generated"
    else
        ci_error "failed to generate Cobertura report"
        FAILED+=("cargo-coverage-xml")
    fi

    if ci_run "cd backend && cargo +nightly llvm-cov report --html --output-dir coverage_html --ignore-filename-regex='qtty/'"; then
        ci_success "backend/coverage_html generated"
    else
        ci_error "failed to generate HTML report"
        FAILED+=("cargo-coverage-html")
    fi

    if ci_run "cd backend && cargo +nightly llvm-cov --workspace --all-features --doctests --no-run --fail-under-lines 90 --ignore-filename-regex='qtty/' -- --test-threads=1"; then
        ci_success "coverage threshold met (≥90% lines)"
    else
        ci_error "coverage threshold not met"
        FAILED+=("cargo-coverage-threshold")
    fi
}

case "$MODE" in
    standard)
        run_standard
        ;;
    coverage)
        run_coverage
        ;;
    all)
        run_standard
        run_coverage
        ;;
esac

ci_header "Summary"
if [[ ${#FAILED[@]} -eq 0 ]]; then
    ci_success "Backend checks passed"
    exit 0
else
    ci_error "Backend checks failed:"
    for step in "${FAILED[@]}"; do
        echo "  - ${step}"
    done
    exit 1
fi
