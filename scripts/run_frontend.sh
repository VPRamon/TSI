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
npm run dev
