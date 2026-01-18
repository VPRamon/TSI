# Docker Setup

This setup runs the TSI application with Docker Compose:
- **PostgreSQL** database
- **Rust backend** (axum HTTP server)
- **React frontend** (served by nginx)

## Prerequisites

- Docker + Docker Compose v2 (`docker compose version`)

## Quickstart

```bash
cd docker
docker compose up --build
```

Services:
- Frontend: http://localhost:3000
- Backend API: http://localhost:8080
- PostgreSQL: localhost:5432

## Environment Variables

Copy `.env.example` to `.env` to customize:

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGRES_USER` | `tsi` | Database user |
| `POSTGRES_PASSWORD` | `tsi` | Database password |
| `POSTGRES_DB` | `tsi` | Database name |
| `POSTGRES_PORT` | `5432` | Database port |
| `BACKEND_PORT` | `8080` | Backend API port |
| `FRONTEND_PORT` | `3000` | Frontend port |
| `RUST_LOG` | `info` | Log level |

## Common Commands

```bash
# Start in background
docker compose up -d --build

# View logs
docker compose logs -f backend
docker compose logs -f frontend

# Stop
docker compose down

# Stop and delete database volume (DANGER: deletes data)
docker compose down -v

# Rebuild a single service
docker compose build backend
docker compose up -d backend
```

Note: the Nginx proxy defaults to a 1MB client body limit. If your schedule JSON files exceed 1MB, increase the limit in [docker/nginx.conf](nginx.conf) (for example `client_max_body_size 50M;`) and then restart the frontend service.

To restart the frontend after changing the config:

```bash
cd docker
docker compose up -d --build frontend
# or
docker compose restart frontend
```

## Connecting to PostgreSQL

```bash
# Via docker
docker compose exec postgres psql -U tsi -d tsi

# From host (if port is exposed)
psql "postgres://tsi:tsi@localhost:5432/tsi"
```

## Production Deployment

For production, update the `.env` file with secure credentials and consider:
- Using Docker secrets for sensitive values
- Adding SSL/TLS termination (nginx or load balancer)
- Setting `RUST_LOG=warn` or `error`
- Configuring proper health check intervals
