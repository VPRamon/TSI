# TSI Application Setup Guide

Complete guide to deploy the TSI (Telescope Schedule Inspector) application from scratch.

## Prerequisites

### 1. Azure SQL Database

Create an Azure SQL Database:

- **Azure Portal**: Create resource → SQL Database
- **Tier**: Standard S0 or higher (S3 recommended for production)
- **Collation**: `SQL_Latin1_General_CP1_CI_AS`

Note your connection details:
- Server name: `<your-server>.database.windows.net`
- Database name: `db-schedules` (or your preferred name)
- Admin username and password

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

## Step 1: Database Setup

### Run the Setup Script

Connect to your Azure SQL Database and run the complete setup script:

#### Using Azure Data Studio

1. Connect to your server
2. Open `scripts/azure-setup-complete.sql`
3. Click Run (or press F5)

#### Using sqlcmd

```bash
sqlcmd -S <server>.database.windows.net \
       -d <database> \
       -U <username> \
       -P '<password>' \
       -i scripts/azure-setup-complete.sql
```

#### Using Azure Portal Query Editor

1. Navigate to your database in Azure Portal
2. Click "Query editor" in the left menu
3. Login with your credentials
4. Copy/paste the contents of `scripts/azure-setup-complete.sql`
5. Click Run

### Verify Setup

After running the script, you should see:

```
✅ DATABASE SETUP COMPLETE

Tables:
  - dbo.schedules
  - dbo.targets
  - dbo.altitude_constraints
  - dbo.azimuth_constraints
  - dbo.constraints
  - dbo.scheduling_blocks
  - dbo.schedule_scheduling_blocks
  - dbo.visibility_periods
  - dbo.schedule_dark_periods
  - analytics.schedule_blocks_analytics
```

---

## Step 2: Application Configuration

### Create Environment File

Create a `.env` file in the project root:

```bash
# Option A: Full connection string
DATABASE_URL="mssql://<user>:<pass>@<server>.database.windows.net/<database>"

# Option B: Individual components
DB_SERVER=<your-server>.database.windows.net
DB_DATABASE=db-schedules
DB_USERNAME=<your-username>
DB_PASSWORD=<your-password>

# Auth and app settings
USE_AAD_AUTH=false
USE_ANALYTICS_TABLE=true
ENABLE_RUST_BACKEND=true

# Optional: Connection tuning
DATABASE_CONNECTION_TIMEOUT=30
DATABASE_MAX_RETRIES=3
```

Alternative: you can also keep credentials in `scripts/db_credentials.py`. The `./run_dashboard.sh` script will read it and export compatible environment variables automatically:

```python
# scripts/db_credentials.py
server = "<your-server>.database.windows.net"
database = "db-schedules"
username = "<user>@<tenant>"  # or SQL user
password = "<password>"
```

### Configure Azure Firewall

Allow connections from your IP:

1. Azure Portal → Your SQL Server → Networking
2. Add your client IP address
3. Click Save

Or use Azure CLI:

```bash
az sql server firewall-rule create \
    --resource-group <your-rg> \
    --server <your-server> \
    --name AllowMyIP \
    --start-ip-address <your-ip> \
    --end-ip-address <your-ip>
```

---

## Step 3: Build the Application

### Build Rust Backend

```bash
./build_rust.sh --release
```

This compiles the high-performance Rust backend and installs it as a Python module.

### Verify Build

```bash
python -c "import tsi_rust; print('✅ Rust backend loaded')"
python -c "from app_config import get_settings; print('✅ Config loaded')"
```

---

## Step 4: Run the Application

### Development Mode

```bash
streamlit run src/tsi/app.py
```

Or use the provided script:

```bash
./run_dashboard.sh
```

The application will be available at `http://localhost:8501`

### Production Mode

For production deployment, use a process manager and reverse proxy:

```bash
# Example with gunicorn-compatible ASGI
streamlit run src/tsi/app.py --server.port 8501 --server.address 0.0.0.0
```

---

## Step 5: Upload Your First Schedule

1. Open the application in your browser
2. Navigate to "Upload Schedule" page
3. Select a JSON schedule file
4. Click Upload

The application will:
1. Parse and validate the schedule
2. Store it in the normalized tables
3. Automatically populate the analytics table

---

## Database Schema Reference

### Base Tables (dbo schema)

| Table | Description |
|-------|-------------|
| `schedules` | Schedule metadata (name, upload time, checksum) |
| `targets` | Observation targets (RA, Dec, proper motion) |
| `altitude_constraints` | Altitude limits for observations |
| `azimuth_constraints` | Azimuth limits for observations |
| `constraints` | Combined constraint references |
| `scheduling_blocks` | Individual observation blocks |
| `schedule_scheduling_blocks` | Block assignments per schedule |
| `visibility_periods` | When targets are visible |
| `schedule_dark_periods` | Dark time windows per schedule |

### Analytics Table (analytics schema)

| Table | Description |
|-------|-------------|
| `schedule_blocks_analytics` | Denormalized, pre-computed data for fast queries |

The analytics table is automatically populated when schedules are uploaded.

---

## Troubleshooting

### Connection Errors

**"Login failed for user"**
- Check firewall rules in Azure Portal
- Verify username/password in `.env`
- Ensure database name is correct

**"Cannot open server"**
- Check server name format: `<server>.database.windows.net`
- Verify port 1433 is not blocked

### Application Errors

**"Rust backend not found"**
- Rebuild: `cd rust_backend && maturin develop --release`
- Check Python version matches (3.11+)

**"No analytics data"**
- Analytics are populated on schedule upload
- For existing schedules, run: `EXEC analytics.sp_populate_schedule_analytics @schedule_id`

### Performance Issues

**Slow queries**
- Check analytics table is populated
- Verify indexes exist (run setup script again if needed)
- Monitor DTU usage in Azure Portal

---

## Application User Permissions

For security, create a dedicated application user with limited permissions:

```sql
-- Create user
CREATE USER [app_user] WITH PASSWORD = 'StrongPassword123!';

-- Grant necessary permissions
GRANT SELECT, INSERT, UPDATE, DELETE ON SCHEMA::dbo TO [app_user];
GRANT SELECT, INSERT, UPDATE, DELETE ON SCHEMA::analytics TO [app_user];
GRANT EXECUTE ON SCHEMA::analytics TO [app_user];
```

Update `.env` to use this user for the application.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    TSI Application                       │
├─────────────────────────────────────────────────────────┤
│  Streamlit UI (Python)                                   │
│    └── Pages: Sky Map, Distributions, Trends, etc.      │
├─────────────────────────────────────────────────────────┤
│  Services Layer (Python)                                 │
│    └── backend_client.py: Backend schedule operations   │
├─────────────────────────────────────────────────────────┤
│  Rust Backend (tsi_rust)                                │
│    └── High-performance data processing                 │
│    └── Astronomical calculations                        │
│    └── Database queries via Tiberius                    │
├─────────────────────────────────────────────────────────┤
│  Azure SQL Database                                      │
│    └── dbo schema: Normalized tables                    │
│    └── analytics schema: Pre-computed analytics         │
└─────────────────────────────────────────────────────────┘
```

---

## Support

- **Issues**: https://github.com/VPRamon/TSI/issues
- **Documentation**: See `docs/` folder for detailed guides

---

## Quick Reference

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | No | - | Full connection string (overrides individual parts) |
| `DB_SERVER` | Cond. | - | SQL Server hostname (used if no `DATABASE_URL`) |
| `DB_DATABASE` | Cond. | - | Database name (used if no `DATABASE_URL`) |
| `DB_USERNAME` | Cond. | - | Username (used if no `DATABASE_URL`) |
| `DB_PASSWORD` | Cond. | - | Password (used if no `DATABASE_URL`) |
| `USE_AAD_AUTH` | No | `false` | Use Azure AD (sets trusted connection) |
| `USE_ANALYTICS_TABLE` | No | `true` | Use pre-computed analytics tables |
| `ENABLE_RUST_BACKEND` | No | `true` | Enable Rust-backed DB operations |
| `DATABASE_CONNECTION_TIMEOUT` | No | `30` | Connection timeout (seconds) |
| `DATABASE_MAX_RETRIES` | No | `3` | Retries for transient DB errors |

### Useful Commands

```bash
# Start application
streamlit run src/tsi/app.py

# Build Rust backend
./build_rust.sh --release

# Run tests
pytest tests/ -v

# Run Rust tests
(cd rust_backend && cargo test)

# Check database connection
python -c "from tsi.services import db_health_check; print(db_health_check())"
```

### SQL Maintenance

```sql
-- Repopulate analytics for a schedule
EXEC analytics.sp_populate_schedule_analytics @schedule_id = 123;

-- Delete a schedule and its analytics
DELETE FROM dbo.schedules WHERE schedule_id = 123;
-- (CASCADE will clean up related tables)

-- Check analytics status
SELECT schedule_id, COUNT(*) as block_count
FROM analytics.schedule_blocks_analytics
GROUP BY schedule_id;
```
