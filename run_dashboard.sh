#!/bin/bash
# Quick start script for the Telescope Scheduling Intelligence Dashboard

set -e  # Exit on error

echo "🔭 Telescope Scheduling Intelligence Dashboard"
echo "=============================================="
echo ""

# Check if virtual environment exists
if [ ! -d "venv" ]; then
    echo "📦 Creating virtual environment..."
    if ! python3 -m venv venv 2>/dev/null; then
        echo ""
        echo "❌ Failed to create virtual environment!"
        echo "Please install python3-venv first:"
        echo "   sudo apt install python3-venv python3-full"
        exit 1
    fi
    echo "✅ Virtual environment created successfully"
fi

# Activate virtual environment
echo "🔌 Activating virtual environment..."
source venv/bin/activate

# Install/upgrade dependencies
echo "📥 Installing dependencies..."
pip install -q -r requirements.txt

# Check data file
if [ ! -f "data/schedule.csv" ]; then
    echo ""
    echo "⚠️  WARNING: Data file not found!"
    echo "Expected location: data/schedule.csv"
    echo ""
    echo "Please ensure the data file exists before continuing."
    read -p "Press Enter to continue anyway or Ctrl+C to exit..."
fi

# Launch dashboard
echo ""
echo "🚀 Launching dashboard..."
echo "   Dashboard will open at: http://localhost:8501"
echo "   Press Ctrl+C to stop the server"
echo ""

export PYTHONPATH="$(pwd)/src:$PYTHONPATH"
streamlit run src/tsi/app.py
