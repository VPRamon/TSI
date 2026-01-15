#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ci_common.sh
source "${SCRIPT_DIR}/ci_common.sh"

SHOW_HELP=false

while [[ $# -gt 0 ]]; do
    case "$1" in
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
Usage: scripts/ci_e2e.sh [OPTIONS]

End-to-end checks (pytest -m e2e).

Options:
  --docker            Force Docker execution (DEV_IMAGE_TAG)
  --no-docker         Force native execution
  -h, --help          Show this help message

EOF
    exit 0
fi

ci_init_docker
ci_show_mode

FAILED=()

ci_header "E2E Tests"
if ci_run "pytest -m e2e --no-cov"; then
    ci_success "e2e tests passed"
else
    ci_error "e2e tests failed"
    FAILED+=("pytest-e2e")
fi

ci_header "Summary"
if [[ ${#FAILED[@]} -eq 0 ]]; then
    ci_success "E2E checks passed"
    exit 0
else
    ci_error "E2E checks failed"
    exit 1
fi
