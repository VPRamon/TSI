# Telescope Scheduling Intelligence (TSI)

Analyze and visualize astronomical scheduling outputs with an interactive web application.

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)
![TypeScript](https://img.shields.io/badge/typescript-5.0%2B-blue.svg)
![React](https://img.shields.io/badge/react-18%2B-61dafb.svg)
![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)

## Architecture

TSI uses a modern client-server architecture:

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

## Features

- **Sky Map**: RA/Dec visualization with priority coloring and status filtering
- **Distributions**: Histograms for priority, visibility, duration, and elevation
- **Visibility Map**: Constraint-based visibility window visualization
- **Timeline**: Month-by-month scheduled observation view with dark period overlays
- **Insights**: Scheduling rates, correlations, and analytics
- **Trends**: Time evolution of scheduling metrics
- **Compare**: Side-by-side schedule comparison
- **Validation**: Schedule integrity and constraint validation reports

## Project Structure

```
TSI/
├── backend/              # Rust HTTP server (axum)
│   ├── src/
│   │   ├── bin/server.rs   # Entry point
│   │   ├── http/           # HTTP handlers & routing
│   │   ├── services/       # Business logic
│   │   ├── db/             # Repository layer
│   │   └── models/         # Domain models
│   └── Cargo.toml
├── frontend/             # React SPA
│   ├── src/
│   │   ├── pages/          # Page components
│   │   ├── components/     # Reusable UI components
│   │   ├── api/            # API client
│   │   └── hooks/          # React Query hooks
│   └── package.json
├── docker/               # Docker configuration
│   ├── docker-compose.yml
│   ├── Dockerfile.backend
│   └── Dockerfile.frontend
├── data/                 # Sample datasets
└── docs/                 # Documentation
```

## Quick Start

### Prerequisites

- Rust 1.75+ with `cargo`
- Node.js 20+ with `npm`
- Docker and Docker Compose (optional)

### Development

**Backend:**
```bash
cd backend
cargo run --bin tsi-server
```
Server starts at http://localhost:8080

**Frontend:**
```bash
cd frontend
npm install
npm run dev
```
Frontend starts at http://localhost:3000

### Docker Compose

```bash
cd docker
docker compose up --build
```

This starts:
- PostgreSQL on port 5432
- Backend API on port 8080
- Frontend on port 3000

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| GET | `/v1/schedules` | List all schedules |
| POST | `/v1/schedules` | Create a new schedule |
| GET | `/v1/schedules/{id}/sky-map` | Sky map data |
| GET | `/v1/schedules/{id}/distributions` | Distribution statistics |
| GET | `/v1/schedules/{id}/visibility-map` | Visibility map |
| GET | `/v1/schedules/{id}/timeline` | Timeline data |
| GET | `/v1/schedules/{id}/insights` | Analytics insights |
| GET | `/v1/schedules/{id}/trends` | Scheduling trends |
| GET | `/v1/schedules/{id}/validation-report` | Validation report |
| GET | `/v1/schedules/{id}/compare/{other}` | Compare schedules |

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `8080` | Server port |
| `DATABASE_URL` | - | PostgreSQL connection string |
| `RUST_LOG` | `info` | Log level |

### Feature Flags (Cargo)

| Feature | Description |
|---------|-------------|
| `local-repo` | In-memory repository (development) |
| `postgres-repo` | PostgreSQL repository (production) |
| `http-server` | HTTP server with axum |

## Testing

**Backend:**
```bash
cd backend
cargo test
```

**Frontend:**
```bash
cd frontend
npm run typecheck
npm run lint
```

## Documentation

- [Architecture Guide](docs/NEW_ARCHITECTURE.md)
- [Repository Pattern](docs/REPOSITORY_PATTERN.md)
- [Docker Setup](docs/SETUP.md)
- [PostgreSQL Design](docs/POSTGRES_ETL_DB_DESIGN.md)

## License

AGPL-3.0 — see [LICENSE](LICENSE) for details.

---

Built with Rust, React, TypeScript, Plotly.js, and modern web tooling.
