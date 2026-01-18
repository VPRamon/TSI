#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOGDIR="$ROOT/logs"
mkdir -p "$LOGDIR"

# Default behavior: restart both backend and frontend in background
TARGET="all"
FOREGROUND=0

usage() {
  cat <<EOF
Usage: $0 [--backend] [--frontend] [--foreground]

Options:
  --backend     Restart only the backend
  --frontend    Restart only the frontend
  --foreground  Run the selected component(s) in foreground (only valid for a single target)

If neither --backend nor --frontend is provided, both are restarted.
Logs are written to $LOGDIR/backend.log and $LOGDIR/frontend.log when running as daemons.
EOF
  exit 1
}

# parse args
while [[ $# -gt 0 ]]; do
  case "$1" in
    --backend)
      TARGET="backend"; shift;;
    --frontend)
      TARGET="frontend"; shift;;
    --foreground)
      FOREGROUND=1; shift;;
    -h|--help)
      usage;;
    *)
      echo "Unknown arg: $1"; usage;;
  esac
done

# helper to stop by a grep pattern
stop_by_pattern() {
  local pattern="$1"
  if pgrep -f "$pattern" >/dev/null; then
    echo "Found processes for pattern '$pattern', stopping..."
    pids=$(pgrep -f "$pattern" | tr '\n' ' ')
    echo "$pids" | xargs -r kill
    # wait up to 10s
    for i in {1..10}; do
      if ! pgrep -f "$pattern" >/dev/null; then
        break
      fi
      sleep 1
    done
    if pgrep -f "$pattern" >/dev/null; then
      echo "Processes did not exit, forcing..."
      echo "$pids" | xargs -r kill -9
    fi
  else
    echo "No processes found for pattern '$pattern'."
  fi
}

restart_backend() {
  echo "--- Backend ---"
  stop_by_pattern 'tsi-server'
  if [[ $FOREGROUND -eq 1 ]]; then
    echo "Starting backend in foreground (logs to stdout)..."
    (cd "$ROOT" && exec ./scripts/run_server.sh)
  else
    echo "Starting backend in background (logs: $LOGDIR/backend.log)..."
    nohup bash -c "cd '$ROOT' && ./scripts/run_server.sh" > "$LOGDIR/backend.log" 2>&1 &
    echo "Backend started (pid: $!). Waiting for readiness..."
    wait_for_backend || echo "Warning: backend did not report healthy within timeout. Check $LOGDIR/backend.log"
  fi
}

restart_frontend() {
  echo "--- Frontend ---"
  stop_by_pattern 'npm run dev|vite|Vite' 
  if [[ $FOREGROUND -eq 1 ]]; then
    echo "Starting frontend in foreground (logs to stdout)..."
    (cd "$ROOT" && exec ./scripts/run_frontend.sh)
  else
    echo "Starting frontend in background (logs: $LOGDIR/frontend.log)..."
    nohup bash -c "cd '$ROOT' && ./scripts/run_frontend.sh" > "$LOGDIR/frontend.log" 2>&1 &
    sleep 1
    echo "Frontend started."
  fi
}

# Wait for backend to become healthy (poll /health). Respects BACKEND_HOST/BACKEND_PORT env vars.
wait_for_backend() {
  local host=${BACKEND_HOST:-127.0.0.1}
  # If the backend binds 0.0.0.0, check localhost instead
  if [[ "$host" == "0.0.0.0" ]]; then host=127.0.0.1; fi
  local port=${BACKEND_PORT:-8080}
  local url="http://${host}:${port}/health"
  local timeout=${BACKEND_WAIT_TIMEOUT:-30}

  echo "Waiting for backend health at $url (timeout ${timeout}s) ..."
  for i in $(seq 1 $timeout); do
    if command -v curl >/dev/null 2>&1; then
      if curl -fs "$url" >/dev/null 2>&1; then
        echo "Backend healthy"
        return 0
      fi
    else
      # Fallback: try TCP connect
      if (echo > /dev/tcp/${host}/${port}) >/dev/null 2>&1; then
        echo "Backend accepting connections on ${host}:${port}"
        return 0
      fi
    fi
    sleep 1
  done
  return 1
}

# validate foreground usage
if [[ $FOREGROUND -eq 1 && "$TARGET" == "all" ]]; then
  echo "--foreground may only be used when restarting a single target (--backend or --frontend)."
  exit 2
fi

case "$TARGET" in
  backend)
    restart_backend;;
  frontend)
    restart_frontend;;
  all)
    restart_backend
    restart_frontend;;
  *)
    usage;;
esac

exit 0
