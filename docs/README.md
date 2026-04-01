# TSI Documentation

This directory contains documentation for the Telescope Scheduling Intelligence (TSI) schedule analysis application.

## Architecture

TSI uses a modern React + Rust architecture:

- **Frontend**: React/TypeScript SPA with Vite, React Query, and Plotly.js
- **Backend**: Rust HTTP server using axum, with diesel for PostgreSQL
- **Database**: PostgreSQL 16 (production) or in-memory LocalRepository (development)

## Quick Links

- **Setup Guide**: [SETUP.md](./SETUP.md) - Complete installation and running instructions
- **New Architecture**: [NEW_ARCHITECTURE.md](./NEW_ARCHITECTURE.md) - Architecture overview
- **Repository Pattern**: [REPOSITORY_PATTERN.md](./REPOSITORY_PATTERN.md) - Data access patterns
- **Database Design**: [POSTGRES_ETL_DB_DESIGN.md](./POSTGRES_ETL_DB_DESIGN.md) - PostgreSQL schema

## Quick Start

```bash
# Start backend (with in-memory repository)
./scripts/run_server.sh

# Start frontend (in separate terminal)
./scripts/run_frontend.sh
```

Or use Docker Compose:

```bash
cd docker
docker-compose up -d
```

## Directory Structure

```
TSI/
├── backend/           # Rust HTTP server
│   ├── src/          # Source code
│   ├── tests/        # Integration tests
│   └── Cargo.toml    # Rust dependencies
├── frontend/         # React/TypeScript app
│   ├── src/          # Source code
│   └── package.json  # Node dependencies
├── docker/           # Docker configuration
├── scripts/          # Build and run scripts
└── docs/             # This directory
```

## Running Tests

```bash
# Backend tests
cd backend && cargo test --all-features

# Frontend checks
cd frontend && npm run lint && npm run typecheck

# Full CI
./scripts/ci.sh
```

## API Reference

See the backend source in `backend/src/routes/` for endpoint definitions:

| Endpoint | Description |
|----------|-------------|
| `/health` | Health check |
| `/v1/schedules` | List or import schedules |
| `/v1/schedules/{id}/sky-map` | Sky map data |
| `/v1/schedules/{id}/distributions` | Distribution analysis |
| `/v1/schedules/{id}/visibility-map` | Visibility calculations |
| `/v1/schedules/{id}/timeline` | Timeline visualization |
| `/v1/schedules/{id}/insights` | Schedule insights |
| `/v1/schedules/{id}/trends` | Trend analysis |
| `/v1/schedules/{id}/validation-report` | Schedule validation |
| `/v1/schedules/{id}/compare/{other_id}` | Schedule comparison |
