#!/bin/bash
# Run the TSI HTTP server in development mode

set -e

cd "$(dirname "$0")/../backend"

# Default to local repository if DATABASE_URL is not set
if [ -z "$DATABASE_URL" ]; then
    echo "Running with local (in-memory) repository"
    cargo run --bin tsi-server --features "local-repo,http-server" "$@"
else
    echo "Running with PostgreSQL repository"
    cargo run --bin tsi-server --features "postgres-repo,http-server" "$@"
fi
