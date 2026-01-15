#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
ENV_FILE="$ROOT_DIR/docker/.env"
COMPOSE_FILE="$ROOT_DIR/docker/docker-compose.yml"

if [ "$#" -eq 0 ]; then
  set -- up --build
fi

if [ ! -f "$ENV_FILE" ]; then
  echo "Missing $ENV_FILE. Create it before running." >&2
  exit 1
fi

docker compose --env-file "$ENV_FILE" -f "$COMPOSE_FILE" "$@"
