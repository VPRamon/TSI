# TSI Architecture Guide

This document describes the client-server architecture for TSI.

## Architecture Overview

The TSI application uses a modern client-server architecture:

- **Frontend**: React + TypeScript SPA with Plotly.js visualizations
- **Backend**: Rust HTTP server (axum) with REST API
- **Database**: PostgreSQL for production, in-memory for development

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
│ - Business logic services
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
docker compose up --build
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
│   │   ├── http/               # HTTP layer
│   │   │   ├── mod.rs
│   │   │   ├── handlers.rs     # Request handlers
│   │   │   ├── router.rs       # Route definitions
│   │   │   ├── state.rs        # App state
│   │   │   ├── error.rs        # Error handling
│   │   │   └── dto.rs          # DTOs for HTTP API
│   │   ├── api.rs              # Public API types
│   │   ├── db/                 # Repository layer
│   │   ├── services/           # Business logic
│   │   ├── routes/             # Route-specific DTOs
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
├── docker/
│   ├── docker-compose.yml      # Docker Compose config
│   ├── Dockerfile.backend      # Rust backend
│   ├── Dockerfile.frontend     # React frontend
│   └── nginx.conf              # Frontend nginx config
└── archive/                    # Archived deprecated code
    ├── python/                 # Old Streamlit app
    ├── docker/                 # Legacy Docker configs
    ├── tests/                  # Old Python tests
    ├── examples/               # Old Python examples
    └── scripts/                # Deprecated scripts
```

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
