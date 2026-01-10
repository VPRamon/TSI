# Docker Compose (Postgres + Streamlit)

This setup runs the TSI Streamlit app plus a Postgres database, and builds the Rust backend with the `postgres-repo` feature enabled.

## Prerequisites

- Docker + Docker Compose v2 (`docker compose version`)

## Quickstart

```bash
cp .env.example .env
docker compose -f docker/docker-compose.yml up --build
```

- Streamlit UI: `http://localhost:8501` (or `$TSI_PORT`)
- Postgres: `localhost:5432` (or `$POSTGRES_PORT`)

## Common commands

```bash
# Start (in background)
docker compose -f docker/docker-compose.yml up -d --build

# Logs
docker compose -f docker/docker-compose.yml logs -f app

# Stop
docker compose -f docker/docker-compose.yml down

# Stop + delete DB volume (DANGER: deletes persisted data)
docker compose -f docker/docker-compose.yml down -v
```

## Connecting to Postgres

```bash
docker compose -f docker/docker-compose.yml exec postgres psql -U "$POSTGRES_USER" -d "$POSTGRES_DB"
```

From your host (if you published the port):

```bash
psql "postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:$POSTGRES_PORT/$POSTGRES_DB"
```

## Notes

- The Rust backend reads `DATABASE_URL` and will create/update tables via Diesel migrations automatically when it first initializes.
- App data files are mounted from `data` (repo root) into the container at `/app/data` (see `docker/docker-compose.yml`).
