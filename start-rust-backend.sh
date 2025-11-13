#!/bin/bash

# Start the Rust backend using Docker Compose
echo "🚀 Starting TSI Rust Backend..."
docker compose up --build rust-backend
