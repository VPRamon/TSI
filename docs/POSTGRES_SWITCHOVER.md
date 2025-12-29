# Switching Repositories: Local ↔ Postgres

This project can run against either the in-memory `LocalRepository` (default for tests/dev) or the Diesel-backed Postgres repository. Use the steps below to move between them.

## Choose via env var (preferred)
- `REPOSITORY_TYPE=postgres` → use Postgres (requires connection info).
- `REPOSITORY_TYPE=local`   → force in-memory.
- If `REPOSITORY_TYPE` is unset but `DATABASE_URL`/`PG_DATABASE_URL` is set, Postgres is auto-selected; otherwise Local is used.

## Postgres setup
1) Start a database (local example):
   ```bash
   docker compose up -d postgres
   ```
   Defaults: user `tsi`, password `tsi`, db `tsi`, port `5432`.
2) Export connection info:
   ```bash
   export REPOSITORY_TYPE=postgres
   export DATABASE_URL=postgres://tsi:tsi@localhost:5432/tsi
   export PG_POOL_MAX=10  # optional
   ```
3) Run the app/tests. Migrations run automatically when the repo initializes.

## Local (in-memory) setup
1) Clear repo choice:
   ```bash
   unset DATABASE_URL PG_DATABASE_URL
   export REPOSITORY_TYPE=local
   ```
2) Run the app/tests. Data lives only in memory per process.

## Using repository.toml instead
1) Edit `rust_backend/repository.toml`:
   ```toml
   [repository]
   type = "postgres" # or "local"

   [postgres]
   database_url = "postgres://user:pass@host:5432/dbname"
   max_connections = 10
   ```
   (Leave `[database]` for Azure untouched.)
2) Start the repo with:
   ```bash
   export REPOSITORY_TYPE=postgres  # optional; file is used if env var absent
   ```

## Quick verification
- Health check (requires repo init):
  ```bash
  python - <<'PY'
  import tsi_rust
  from tsi_rust import db
  db.init_repository()
  print("healthy?", db.health_check(db.get_repository()[0]))
  PY
  ```
- Logs should show migrations running on first Postgres use. If connection fails, recheck `DATABASE_URL` and that the container/DB is reachable.
