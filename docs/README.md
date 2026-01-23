# TSI Documentation

This directory contains documentation for the Telescope Scheduling Intelligence (TSI) application.

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
| `/api/landing` | Landing page statistics |
| `/api/timeline` | Timeline visualization |
| `/api/distribution` | Distribution analysis |
| `/api/trends` | Trend analysis |
| `/api/skymap` | Sky map data |
| `/api/compare` | Schedule comparison |
| `/api/validation` | Schedule validation |
| `/api/visibility` | Visibility calculations |
| `/api/insights` | Schedule insights |

## Schedule Format

The API accepts schedule data in two formats:

### Astro Format (Recommended)

The astro crate format uses `tasks` and `location` fields. Visibility periods are
computed on-the-fly from target constraints. See [`backend/astro/schemas/schedule.schema.json`](../backend/astro/schemas/schedule.schema.json).

```json
{
  "location": { "lat": 28.7624, "lon": -17.8892, "distance": 6373.396 },
  "period": { "start": 60676.0, "end": 60677.0 },
  "tasks": [
    {
      "type": "observation",
      "id": "1",
      "name": "M31 Observation",
      "target": { "position": { "ra": 10.6847, "dec": 41.2687 }, "time": 2451545.0 },
      "duration_sec": 3600.0,
      "priority": 10
    }
  ]
}
```

### Legacy Format (Deprecated)

The legacy format uses `blocks` and `geographic_location`. See [`backend/docs/schedule.schema.json`](../backend/docs/schedule.schema.json).

**Note**: The `possible_periods` field is deprecated. Visibility periods are now computed
automatically from constraints during ingestion.
