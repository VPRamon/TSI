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
    sleep 1
    echo "Backend started."
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
