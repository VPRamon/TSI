# Repository Pattern (Postgres + Local)

The Rust backend uses a repository abstraction so the application can switch between:

1. **PostgresRepository** (production, Diesel-backed)
2. **LocalRepository** (in-memory, tests/dev)

## Quick Selection

Use the environment variable:

```bash
export REPOSITORY_TYPE=postgres  # or "local"
```

If `REPOSITORY_TYPE` is unset but `DATABASE_URL`/`PG_DATABASE_URL` is set, Postgres is selected automatically; otherwise Local is used.

## Postgres Configuration

Environment variables (preferred):

```bash
DATABASE_URL=postgres://user:pass@host:5432/dbname
PG_POOL_MAX=10
PG_POOL_MIN=1
PG_CONN_TIMEOUT_SEC=30
PG_IDLE_TIMEOUT_SEC=600
PG_MAX_RETRIES=3
PG_RETRY_DELAY_MS=100
```

Or use `backend/repository.toml`:

```toml
[repository]
type = "postgres"

[postgres]
database_url = "postgres://user:pass@host:5432/dbname"
max_connections = 10
min_connections = 1
connect_timeout = 30
idle_timeout = 600
max_retries = 3
retry_delay_ms = 100
```

## Usage Examples

```rust
use tsi_rust::db::{RepositoryFactory, RepositoryType, PostgresConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PostgresConfig::from_env()?;
    let repo = RepositoryFactory::create(RepositoryType::Postgres, Some(&config)).await?;
    println!("healthy? {}", repo.health_check().await?);
    Ok(())
}
```

```rust
use tsi_rust::db::{RepositoryFactory, RepositoryType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo = RepositoryFactory::create(RepositoryType::Local, None).await?;
    println!("healthy? {}", repo.health_check().await?);
    Ok(())
}
```

## Notes

- Migrations run automatically when the Postgres repository initializes.
- The Local repository is ephemeral per process; data is not persisted.
