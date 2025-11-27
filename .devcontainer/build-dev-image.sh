#!/usr/bin/env bash
set -euo pipefail

# Builds the devcontainer image ahead of time so VS Code can reuse it without
# invoking the Dev Containers CLI build step (which fails in this environment).

IMAGE_NAME="${DEVCONTAINER_IMAGE:-tsi-devcontainer:dev}"
DEBIAN_VERSION="${DEBIAN_VERSION:-12}"
RUST_VERSION="${RUST_VERSION:-nightly}"

if ! command -v docker >/dev/null 2>&1; then
  echo "[devcontainer] docker CLI not available on host." >&2
  exit 1
fi

if docker image inspect "${IMAGE_NAME}" >/dev/null 2>&1; then
  if [[ "${DEVCONTAINER_FORCE_REBUILD:-0}" != "1" ]]; then
    echo "[devcontainer] Image '${IMAGE_NAME}' already present; skipping rebuild."
    exit 0
  else
    echo "[devcontainer] Rebuilding '${IMAGE_NAME}' because DEVCONTAINER_FORCE_REBUILD=1."
  fi
else
  echo "[devcontainer] Building devcontainer image '${IMAGE_NAME}'."
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

docker build \
  --target dev \
  --build-arg DEBIAN_VERSION="${DEBIAN_VERSION}" \
  --build-arg RUST_VERSION="${RUST_VERSION}" \
  -t "${IMAGE_NAME}" \
  -f "${WORKSPACE_DIR}/Dockerfile" \
  "${WORKSPACE_DIR}"
