# Backend Cleanup Report

## Removed Items

### Legacy Azure backend (code + config + docs)
- Removed Azure repository implementation and placeholders:
  - `backend/src/db/repositories/azure/README.md`
  - `backend/src/db/repositories/azure/analytics.rs`
  - `backend/src/db/repositories/azure/azure-setup.sql`
  - `backend/src/db/repositories/azure/mod.rs`
  - `backend/src/db/repositories/azure/operations.rs`
  - `backend/src/db/repositories/azure/pool.rs`
  - `backend/src/db/repositories/azure/repository.rs`
  - `backend/src/db/repositories/azure/validation.rs`
- Removed Azure config path and legacy feature plumbing:
  - `backend/src/db/config.rs`
  - `backend/repository.azure.toml.example`
- Removed Azure-specific docs:
  - `docs/AZURE_ANALYTICS.md`
  - `docs/AZURE_DATABASE.md`
  - `docs/ARCHITECTURE.md`
  - `docs/ETL_PROCESS.md`
  - `docs/ETL-EXTENDED.md`
  - `docs/BACKEND_MIGRATION.md`

### Unused internal modules
- Removed `algorithms` module:
  - `backend/src/algorithms/analysis.rs`
  - `backend/src/algorithms/conflicts.rs`
  - `backend/src/algorithms/mod.rs`
- Removed `transformations` module:
  - `backend/src/transformations/cleaning.rs`
  - `backend/src/transformations/filtering.rs`
  - `backend/src/transformations/mod.rs`
- Removed unused `siderust` module:
  - `backend/src/siderust/mod.rs`
  - `backend/src/siderust/coordinates/mod.rs`
  - `backend/src/siderust/coordinates/spherical/mod.rs`
  - `backend/src/siderust/coordinates/spherical/direction.rs`

### Dependencies/features pruned
- Removed Cargo feature and deps for the legacy Azure backend:
  - `azure-repo` feature and `tiberius`, `bb8`, `bb8-tiberius`, `reqwest`, `tokio-util` deps
- Removed unused crates:
  - `serde_path_to_error`
  - `once_cell`
- Removed unused dev-dependencies:
  - `criterion`, `proptest`, `tempfile`

## Evidence and Validation Notes

### Azure backend removal
- Legacy/unused signals:
  - `backend/Cargo.toml` labeled `azure-repo` as legacy and it was not enabled by default.
  - Docker builds enable `postgres-repo` only (`Dockerfile`), and runtime uses Postgres (`docker-compose.yml`).
  - Azure repo files were stubs with `todo!` placeholders (removed).
- No Streamlit dependency:
  - Streamlit calls are routed through `backend/src/routes/mod.rs`, which only registers the Postgres/local-backed route functions.

### Algorithms/transformations removal
- No references outside the modules themselves (confirmed via repo-wide `rg`).
- No routes or Python exports reference those modules (`backend/src/routes/mod.rs`, `backend/src/api.rs`).

### Docs/config cleanup
- Docs referencing Azure SQL and removed scripts/modules were deleted or updated:
  - `docs/SETUP.md`, `docs/REPOSITORY_PATTERN.md`, `docs/POSTGRES_SWITCHOVER.md`, `docs/README.md`
- Configuration now matches Postgres/local-only repository selection:
  - `backend/repository.toml`
  - `backend/src/db/repo_config.rs`
  - `src/app_config/settings.py`

## Tests and Checks

### Commands executed
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo build`

### Results
- All Rust tests and clippy checks passed.
- Streamlit run or end-to-end API smoke test was not executed in this cleanup.

## Risk Notes
- Azure SQL backend and its docs/config were removed. Any downstream usage relying on Azure-specific environment variables or scripts will need to switch to Postgres or LocalRepository.
- Postgres-only configuration is now the documented path; verify any deployment automation that referenced Azure docs.
