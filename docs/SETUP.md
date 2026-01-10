# TSI Application Setup Guide

Complete guide to run the TSI (Telescope Scheduling Intelligence) application locally with the Rust backend.

## Prerequisites

### 1. Postgres Database

Use Docker Compose (recommended):

```bash
docker compose -f docker/docker-compose.yml up -d postgres
```

Defaults: user `tsi`, password `tsi`, db `tsi`, port `5432`.

### 2. Development Environment

#### Option A: VS Code Dev Container (Recommended)

1. Install [Docker Desktop](https://www.docker.com/products/docker-desktop/)
2. Install [VS Code](https://code.visualstudio.com/) with the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
3. Clone the repository and open in container:

```bash
git clone https://github.com/VPRamon/TSI.git
cd TSI
code .
# When prompted, click "Reopen in Container"
```

#### Option B: Local Setup

Requirements:
- Python 3.11+
- Rust toolchain (via rustup) + Cargo
- Maturin (installed automatically by the build script if missing)

```bash
# Create and activate a virtual environment (Linux)
python3 -m venv venv
source venv/bin/activate

# Install Python dependencies
pip install -r requirements.txt

# Build the Rust backend (from repo root)
./build_rust.sh --release
```

---

## Step 1: Database Configuration

Create a `.env` file in the project root:

```bash
DATABASE_URL=postgres://tsi:tsi@localhost:5432/tsi
REPOSITORY_TYPE=postgres

# Optional: connection tuning
PG_POOL_MAX=10
PG_POOL_MIN=1
PG_CONN_TIMEOUT_SEC=30
PG_IDLE_TIMEOUT_SEC=600
PG_MAX_RETRIES=3
PG_RETRY_DELAY_MS=100
```

The Postgres migrations run automatically the first time the repository initializes.

---

## Step 2: Build the Application

```bash
./build_rust.sh --release
```

Verify the Rust module loads:

```bash
python -c "import tsi_rust; print('✅ Rust backend loaded')"
python -c "from app_config import get_settings; print('✅ Config loaded')"
```

---

## Step 3: Run the Application

```bash
streamlit run src/tsi/app.py
```

Or use the helper script:

```bash
./run_dashboard.sh
```

The application will be available at `http://localhost:8501`.

---

## Step 4: Upload Your First Schedule

1. Open the application in your browser
2. Navigate to "Upload Schedule" page
3. Select a JSON schedule file
4. Click Upload

The application will:
1. Parse and validate the schedule
2. Store it in the Postgres-backed repository
3. Populate analytics used by the visualization pages
