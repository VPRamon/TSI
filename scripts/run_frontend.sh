#!/bin/bash
# Run the TSI frontend in development mode

set -e

cd "$(dirname "$0")/../frontend"

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install
fi

# Start development server
# Allow overriding host/port for containerized environments
DEV_HOST=${DEV_HOST:-0.0.0.0}
DEV_PORT_ARG=""
if [ -n "${DEV_PORT:-}" ]; then
    DEV_PORT_ARG=" --port ${DEV_PORT}"
fi

# Pass host and optional port through to Vite
npm run dev -- --host "$DEV_HOST"$DEV_PORT_ARG
