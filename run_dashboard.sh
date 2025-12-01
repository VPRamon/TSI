#!/bin/bash
# Quick start script for the Telescope Scheduling Intelligence Dashboard

set -e  # Exit on error

echo "🔭 Telescope Scheduling Intelligence Dashboard"
echo "=============================================="
echo ""

# Load database credentials if available and export as env vars
if [ -f "scripts/db_credentials.py" ]; then
    echo "🔧 Loading database credentials from scripts/db_credentials.py..."
    eval "$(
        python3 <<'PY'
import importlib.util, pathlib, shlex

path = pathlib.Path("scripts/db_credentials.py")
spec = importlib.util.spec_from_file_location("db_credentials", path)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

def export(name, value):
    print(f"export {name}={shlex.quote(str(value))}")

export("DB_SERVER", getattr(mod, "server", ""))
export("DB_DATABASE", getattr(mod, "database", ""))
export("DB_USERNAME", getattr(mod, "username", ""))
export("DB_PASSWORD", getattr(mod, "password", ""))
export("DB_PORT", 1433)
export("DB_TRUST_CERT", "true")
# Default to AAD password flow when username looks like a UPN
auth_method = "aad_password" if "@" in str(getattr(mod, "username", "")) else "sql_password"
export("DB_AUTH_METHOD", auth_method)
PY
    )"
    echo "   DB_SERVER=$DB_SERVER"
    echo "   DB_DATABASE=$DB_DATABASE"
    echo "   DB_USERNAME=$DB_USERNAME"
    echo "   DB_PORT=$DB_PORT"
    echo "   DB_AUTH_METHOD=$DB_AUTH_METHOD"
    echo ""
fi

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
if [ ! -f "data/schedule.json" ]; then
    echo ""
    echo "⚠️  WARNING: Data file not found!"
    echo "Expected location: data/schedule.json"
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
