# TSI New Architecture - Quick Start Guide

This document describes how to run the new client-server architecture for TSI.

## Architecture Overview

```
┌──────────────────────────┐
│ Web Frontend (React+TS) │  ← Port 3000
│ - Plotly.js visualizations
│ - React Query for data fetching
│ - Tailwind CSS styling
└─────────────▲──────────┘
              │ HTTP/JSON
              ▼
┌──────────────────────────┐
│ Rust Backend (axum)     │  ← Port 8080
│ - REST API endpoints
│ - Reuses existing services
│ - PostgreSQL/Local repo
└─────────────▲──────────┘
              │
              ▼
┌──────────────────────────┐
│ PostgreSQL Database      │  ← Port 5432
└──────────────────────────┘
```

## Development Setup

### Prerequisites

- Rust 1.75+ with `cargo`
- Node.js 20+ with `npm`
- PostgreSQL 16+ (optional, can use local in-memory repo)
- Docker and Docker Compose (optional, for containerized setup)

### Running the Backend (Development)

```bash
# Navigate to backend directory
cd backend

# With local (in-memory) repository
cargo run --bin tsi-server --features "local-repo,http-server"

# With PostgreSQL repository
export DATABASE_URL="postgres://tsi:tsi@localhost:5432/tsi"
cargo run --bin tsi-server --features "postgres-repo,http-server"
```

The server will start on `http://localhost:8080`.

### Running the Frontend (Development)

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
npm install

# Start development server (with hot reload)
npm run dev
```

The frontend will start on `http://localhost:3000` and proxy API requests to the backend.

### Running with Docker Compose

```bash
# Start all services (backend, frontend, PostgreSQL)
cd docker
docker compose -f docker-compose.new.yml up --build

# Start with legacy Streamlit app as well
docker compose -f docker-compose.new.yml --profile legacy up --build
```

## API Endpoints

### Health Check
- `GET /health` - Service health status

### Schedules
- `GET /v1/schedules` - List all schedules
- `POST /v1/schedules` - Create a new schedule

### Visualizations
- `GET /v1/schedules/{id}/sky-map` - Sky map data
- `GET /v1/schedules/{id}/distributions` - Distribution statistics
- `GET /v1/schedules/{id}/visibility-map` - Visibility map
- `GET /v1/schedules/{id}/timeline` - Timeline data
- `GET /v1/schedules/{id}/insights` - Analytics insights
- `GET /v1/schedules/{id}/trends` - Scheduling trends
- `GET /v1/schedules/{id}/validation-report` - Validation report
- `GET /v1/schedules/{id}/compare/{other_id}` - Compare two schedules

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `8080` | Server port |
| `DATABASE_URL` | - | PostgreSQL connection string |
| `RUST_LOG` | `info` | Log level |

### Feature Flags

| Feature | Description |
|---------|-------------|
| `local-repo` | In-memory repository (development) |
| `postgres-repo` | PostgreSQL repository (production) |
| `http-server` | HTTP server with axum |

## Project Structure

```
TSI/
├── backend/
│   ├── src/
│   │   ├── bin/
│   │   │   └── server.rs       # HTTP server entry point
│   │   ├── http/               # HTTP layer (new)
│   │   │   ├── mod.rs
│   │   │   ├── handlers.rs     # Request handlers
│   │   │   ├── router.rs       # Route definitions
│   │   │   ├── state.rs        # App state
│   │   │   ├── error.rs        # Error handling
│   │   │   └── dto.rs          # DTOs for HTTP API
│   │   ├── db/                 # Repository layer
│   │   ├── services/           # Business logic
│   │   ├── routes/             # Python bindings (legacy)
│   │   └── models/             # Domain models
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   ├── api/                # API client
│   │   ├── components/         # React components
│   │   ├── hooks/              # React Query hooks
│   │   ├── pages/              # Page components
│   │   ├── store/              # Zustand store
│   │   ├── App.tsx
│   │   └── main.tsx
│   ├── package.json
│   └── vite.config.ts
└── docker/
    ├── docker-compose.new.yml  # New architecture
    ├── Dockerfile.backend      # Rust backend
    ├── Dockerfile.frontend     # React frontend
    └── nginx.conf              # Frontend nginx config
```

## Migration from Streamlit

The new architecture runs alongside the existing Streamlit app during the migration period:

1. The Rust backend exposes the same functionality via HTTP that was previously available through Python bindings
2. The React frontend provides the same visualizations as Streamlit pages
3. Both can run simultaneously during transition
4. Once migration is complete, the Python/Streamlit code can be removed

## Testing

### Backend Tests

```bash
cd backend
cargo test --features "local-repo,http-server"
```

### Frontend Type Check

```bash
cd frontend
npm run type-check
```
