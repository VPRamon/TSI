#!/bin/bash

# Script to upload schedule and possible periods to Azure SQL Database
# This script sets up environment variables and runs the Rust upload tool

set -e

# Database configuration
export DB_SERVER="${DB_SERVER:-tsi-upgrade.database.windows.net}"
export DB_DATABASE="${DB_DATABASE:-db-schedules}"
export DB_USERNAME="${DB_USERNAME:-ramon.valles@bootcamp-upgrade.com}"

# Check if password is set
if [ -z "$DB_PASSWORD" ]; then
    echo "Error: DB_PASSWORD environment variable must be set"
    echo "Usage: DB_PASSWORD='your-password' ./upload_schedule.sh [schedule.json] [possible_periods.json]"
    exit 1
fi

# File paths (optional arguments)
SCHEDULE_FILE="${1:-/workspace/data/schedule.json}"
PERIODS_FILE="${2:-/workspace/data/possible_periods.json}"

# Check if files exist
if [ ! -f "$SCHEDULE_FILE" ]; then
    echo "Error: Schedule file not found: $SCHEDULE_FILE"
    exit 1
fi

if [ ! -f "$PERIODS_FILE" ]; then
    echo "Error: Possible periods file not found: $PERIODS_FILE"
    exit 1
fi

# Build the Rust binary if it doesn't exist or is outdated
BINARY_PATH="/workspace/target/release/upload_schedule"
if [ ! -f "$BINARY_PATH" ] || [ "rust_backend/src/bin/upload_schedule.rs" -nt "$BINARY_PATH" ]; then
    echo "Building upload_schedule binary..."
    cargo build --manifest-path /workspace/rust_backend/Cargo.toml --bin upload_schedule --release
fi

# Run the upload tool
echo "Starting upload..."
"$BINARY_PATH" "$SCHEDULE_FILE" "$PERIODS_FILE"
