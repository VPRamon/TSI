# TSI Application Setup Guide

Complete guide to run the TSI (Telescope Scheduling Intelligence) application locally with the Rust HTTP backend and React frontend.

## Architecture Overview

- **Backend**: Rust HTTP server (axum) on port 8080
- **Frontend**: React/TypeScript app (Vite) on port 5173 (dev) or nginx on port 3000 (production)
- **Database**: PostgreSQL 16 (production) or in-memory LocalRepository (development)

## Prerequisites

### 1. Development Environment

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
- Rust toolchain (via rustup) + Cargo
- Node.js 18+ and npm
- PostgreSQL 16 (optional, for production mode)

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js dependencies
cd frontend
npm install
```

---

## Quick Start (Development Mode)

### 1. Start the Backend

The backend runs with an in-memory repository by default (no database required):

```bash
# From repository root
./scripts/run_server.sh
```

Or manually:

```bash
cd backend
cargo run --bin tsi-server --features "local-repo,http-server"
```

The API will be available at `http://localhost:8080`.

### 2. Start the Frontend

In a separate terminal:

```bash
# From repository root
./scripts/run_frontend.sh
```

Or manually:

```bash
cd frontend
npm run dev
```

The application will be available at `http://localhost:5173`.

---

## Production Setup with PostgreSQL

### 1. Start PostgreSQL

```bash
./scripts/docker_setup.sh up -d postgres
```

Defaults: user `tsi`, password `tsi`, db `tsi`, port `5432`.

### 2. Configure Environment

Edit `docker/.env`:

```bash
DATABASE_URL=postgres://tsi:tsi@localhost:5432/tsi
BACKEND_PORT=8080
FRONTEND_PORT=3000

# Optional: connection tuning
PG_POOL_MAX=10
PG_POOL_MIN=1
PG_CONN_TIMEOUT_SEC=30
```

### 3. Run with PostgreSQL

```bash
# Backend with PostgreSQL repository
cd backend
cargo run --bin tsi-server --features "postgres-repo,http-server"
```

The Postgres migrations run automatically on first start.

---

## Docker Compose (Full Stack)

### Development

```bash
cd docker
docker-compose up -d
```

This starts:
- PostgreSQL on port 5432
- Rust backend on port 8080
- React frontend (nginx) on port 3000

### Access the Application

- **Frontend**: http://localhost:3000
- **Backend API**: http://localhost:8080

---

## API Endpoints

The backend exposes the following endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/api/landing` | GET | Landing page statistics |
| `/api/timeline` | POST | Timeline visualization data |
| `/api/distribution` | POST | Distribution analysis |
| `/api/trends` | POST | Trend analysis |
| `/api/skymap` | POST | Sky map data |
| `/api/compare` | POST | Schedule comparison |
| `/api/validation` | POST | Schedule validation |
| `/api/visibility` | POST | Visibility calculations |
| `/api/insights` | POST | Schedule insights |

---

## Running Tests

### Backend Tests

```bash
cd backend
cargo test --all-features
```

### Frontend Tests

```bash
cd frontend
npm run lint      # ESLint
npm run typecheck # TypeScript check
```

### Full CI

```bash
./scripts/ci.sh
```

---

## Troubleshooting

### Backend won't start

1. Check if port 8080 is already in use
2. Verify Rust toolchain: `rustc --version`
3. Try clean build: `cd backend && cargo clean && cargo build`

### Frontend can't connect to backend

1. Verify backend is running on port 8080
2. Check CORS settings if using different ports
3. Check browser console for network errors

### Database connection issues

1. Verify PostgreSQL is running: `docker ps`
2. Check DATABASE_URL is correct
3. Test connection: `psql $DATABASE_URL`
