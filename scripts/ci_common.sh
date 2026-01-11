#!/bin/bash
set -euo pipefail

# Shared helpers for CI scripts.

# Colors for output
CI_RED='\033[0;31m'
CI_GREEN='\033[0;32m'
CI_YELLOW='\033[1;33m'
CI_BLUE='\033[0;34m'
CI_NC='\033[0m'

ci_header() {
    echo -e "\n${CI_BLUE}========================================${CI_NC}"
    echo -e "${CI_BLUE}$1${CI_NC}"
    echo -e "${CI_BLUE}========================================${CI_NC}\n"
}

ci_success() {
    echo -e "${CI_GREEN}✓ $1${CI_NC}"
}

ci_error() {
    echo -e "${CI_RED}✗ $1${CI_NC}"
}

ci_warn() {
    echo -e "${CI_YELLOW}⚠ $1${CI_NC}"
}

ci_repo_root() {
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    (cd "${script_dir}/.." && pwd)
}

# Initializes docker execution mode.
# Uses env:
#   DEV_IMAGE_TAG (default: tsi-dev:ci)
#   DOCKER_MODE (auto|always|never, default: auto)
# Can be overridden per-script by setting USE_DOCKER=true/false before calling.
ci_init_docker() {
    : "${DEV_IMAGE_TAG:=tsi-dev:ci}"
    : "${DOCKER_MODE:=auto}"

    if [[ "${USE_DOCKER:-}" == "true" || "${USE_DOCKER:-}" == "false" ]]; then
        return 0
    fi

    USE_DOCKER=false

    if [[ "$DOCKER_MODE" == "always" ]]; then
        USE_DOCKER=true
    elif [[ "$DOCKER_MODE" == "never" ]]; then
        USE_DOCKER=false
    elif [[ -f /.dockerenv ]] || [[ -f /run/.containerenv ]]; then
        # Already inside a container
        USE_DOCKER=false
    else
        # Outside container, check if Docker image exists
        if command -v docker &>/dev/null && docker image inspect "$DEV_IMAGE_TAG" &>/dev/null; then
            USE_DOCKER=true
        else
            ci_warn "Docker image $DEV_IMAGE_TAG not found, running natively"
            USE_DOCKER=false
        fi
    fi
}

ci_show_mode() {
    if [[ "${USE_DOCKER:-false}" == true ]]; then
        ci_header "Running in Docker mode (image: ${DEV_IMAGE_TAG})"
    else
        ci_header "Running in native mode"
    fi
}

# Run a command in Docker (if enabled) or natively.
# Always runs from repo root.
ci_run() {
    local cmd="$1"
    local root
    root="$(ci_repo_root)"

    if [[ "${USE_DOCKER:-false}" == true ]]; then
        docker run --rm \
            --env CI=1 \
            --env HOME=/tmp \
            --env CARGO_HOME=/tmp/cargo \
            --user "$(id -u):$(id -g)" \
            -v "${root}":/workspace \
            -w /workspace \
            "${DEV_IMAGE_TAG}" \
            bash -lc "${cmd}"
    else
        (cd "${root}" && bash -lc "${cmd}")
    fi
}
