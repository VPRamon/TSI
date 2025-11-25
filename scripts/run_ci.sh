#!/usr/bin/env bash

# Local replica of the CI workflow defined in .github/workflows/ci.yml.
# Runs Python quality gates, the matrix of pytest subsets, and Rust checks.

set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1 && pwd)"
cd "$REPO_ROOT"

PYTHON_BIN="${PYTHON_BIN:-python3}"
CARGO_BIN="${CARGO_BIN:-cargo}"

command -v "$PYTHON_BIN" >/dev/null || {
  echo "error: $PYTHON_BIN not found in PATH" >&2
  exit 1
}

command -v "$CARGO_BIN" >/dev/null || {
  echo "error: $CARGO_BIN not found in PATH" >&2
  exit 1
}

log() {
  echo
  echo "==> $*"
  echo
}

run_python_install() {
  log "Installing Python dependencies"
  "$PYTHON_BIN" -m pip install --upgrade pip
  "$PYTHON_BIN" -m pip install -e ".[dev]"
}

run_python_quality() {
  log "Running Python linters and type checks"
  ruff check src/ tests/
  black --check src/ tests/
  mypy src/
}

run_pytest_subset() {
  local subset="$1"
  log "Running pytest subset: $subset"
  case "$subset" in
    unit)
      pytest -m unit --cov=src --cov-report=xml --cov-report=html --cov-report=term-missing:skip-covered
      ;;
    integration)
      pytest -m integration --no-cov
      ;;
    e2e)
      pytest -m e2e --no-cov
      ;;
    unmarked)
      pytest -m "not unit and not integration and not e2e" --no-cov
      ;;
    bindings)
      pytest rust_backend/tests --no-cov
      ;;
    *)
      echo "error: unknown pytest subset '$subset'" >&2
      exit 1
      ;;
  esac
}

run_rust_checks() {
  log "Running Rust format, lint, and tests"
  "$CARGO_BIN" fmt --all --check
  "$CARGO_BIN" clippy --all-targets --all-features -- -D warnings
  "$CARGO_BIN" test --all-features
}

main() {
  log "Starting local CI run"
  run_python_install
  run_python_quality

  for subset in unit integration e2e unmarked bindings; do
    run_pytest_subset "$subset"
  done

  run_rust_checks
  log "Local CI run completed successfully"
}

main "$@"
