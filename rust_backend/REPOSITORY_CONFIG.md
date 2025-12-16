# Repository Configuration Guide

This guide explains how to configure and select which repository system to use in the TSI Rust backend.

## Overview

The TSI backend supports two repository implementations:

1. **Local Repository** (`local`): In-memory storage for testing and local development
2. **Azure Repository** (`azure`): Azure SQL Server for production use

## Configuration Methods

You can configure the repository system in three ways (in order of precedence):

### 1. Configuration File (Recommended)

Create a `repository.toml` file in one of these locations:
- Current directory: `./repository.toml`
- Rust backend directory: `./rust_backend/repository.toml`
- Parent directory: `../repository.toml`

**Example: Local Repository Configuration**

```toml
[repository]
type = "local"
```

**Example: Azure Repository Configuration**

```toml
[repository]
type = "azure"

[database]
server = "myserver.database.windows.net"
database = "mydatabase"
auth_method = "azure_identity"  # or "sql"

# Only required if auth_method = "sql"
username = "myuser"
password = "mypassword"

[connection_pool]
max_connections = 20
min_connections = 2
connect_timeout = 30
```

### 2. Environment Variables

Set the `REPOSITORY_TYPE` environment variable:

```bash
# Use local repository
export REPOSITORY_TYPE=local

# Use Azure repository (requires additional DB env vars)
export REPOSITORY_TYPE=azure
export DB_SERVER=myserver.database.windows.net
export DB_NAME=mydatabase
export DB_AUTH_METHOD=azure_identity
```

### 3. Programmatic Configuration

In your Rust code:

```rust
use tsi_rust::db::{RepositoryFactory, RepositoryType, DbConfig};

// Create local repository
let repo = RepositoryFactory::create_local();

// Create Azure repository
let config = DbConfig::from_env()?;
let repo = RepositoryFactory::create_azure(&config).await?;

// Create from config file
let repo = RepositoryFactory::from_config_file("repository.toml").await?;

// Create from default config location
let repo = RepositoryFactory::from_default_config().await?;

// Use builder pattern
let repo = RepositoryBuilder::new()
    .from_config_file("repository.toml")?
    .build()
    .await?;
```

## Configuration File Reference

### `[repository]` Section

- `type`: Repository type to use
  - `"local"`: In-memory local repository
  - `"azure"`: Azure SQL Server

### `[database]` Section (Azure only)

- `server`: Azure SQL Server hostname (e.g., "myserver.database.windows.net")
- `database`: Database name
- `auth_method`: Authentication method
  - `"azure_identity"`: Azure Active Directory (recommended)
  - `"sql"`: SQL Server authentication
- `username`: SQL auth username (required if `auth_method = "sql"`)
- `password`: SQL auth password (required if `auth_method = "sql"`)

### `[connection_pool]` Section (Optional)

- `max_connections`: Maximum connection pool size (default: 10)
- `min_connections`: Minimum connection pool size (default: 1)
- `connect_timeout`: Connection timeout in seconds (default: 30)

## Example Configurations

### Development (Local)

```toml
[repository]
type = "local"
```

### Production (Azure with Azure Identity)

```toml
[repository]
type = "azure"

[database]
server = "prod-server.database.windows.net"
database = "production_db"
auth_method = "azure_identity"

[connection_pool]
max_connections = 50
min_connections = 5
connect_timeout = 30
```

### Testing (Azure with SQL Auth)

```toml
[repository]
type = "azure"

[database]
server = "test-server.database.windows.net"
database = "test_db"
auth_method = "sql"
username = "test_user"
password = "test_password"

[connection_pool]
max_connections = 10
min_connections = 1
connect_timeout = 15
```

## Python Bindings

When using the Python bindings, the repository will automatically be configured based on:

1. `repository.toml` file if present
2. Environment variables (`REPOSITORY_TYPE`, etc.)
3. Default to Azure if no configuration found

```python
from tsi_rust import create_repository

# Uses configuration from repository.toml or environment
repo = create_repository()
```

## Security Best Practices

1. **Never commit credentials** to version control
   - Add `repository.toml` to `.gitignore` if it contains secrets
   - Use example files like `repository.azure.toml.example` for templates

2. **Prefer Azure Identity** over SQL authentication
   - More secure (no passwords in config)
   - Easier credential rotation
   - Better audit logging

3. **Use environment variables** for CI/CD
   - Set `REPOSITORY_TYPE` in CI environment
   - Keep secrets in secure vaults

4. **Separate configs** for different environments
   - `repository.dev.toml`
   - `repository.staging.toml`
   - `repository.prod.toml`

## Migration Guide

### Migrating from TestRepository to LocalRepository

The `TestRepository` has been renamed to `LocalRepository` to better reflect its use as a local in-memory storage option for both testing and development.

**Old code:**
```rust
use tsi_rust::db::repositories::TestRepository;

let repo = TestRepository::new();
```

**New code:**
```rust
use tsi_rust::db::repositories::LocalRepository;

let repo = LocalRepository::new();
```

**Environment variable:**
- Old: `REPOSITORY_TYPE=test`
- New: `REPOSITORY_TYPE=local`

## Troubleshooting

### "No repository.toml found"

Create a `repository.toml` file in the working directory or use environment variables.

### "Azure repository requires DbConfig"

Ensure `[database]` section is properly configured when using `type = "azure"`.

### "SQL auth requires username"

When using `auth_method = "sql"`, both `username` and `password` must be set.

### Connection failures

- Check `server` hostname is correct
- Verify firewall rules allow your IP
- Ensure credentials are valid
- Try increasing `connect_timeout`

## Additional Resources

- [Azure SQL Documentation](https://docs.microsoft.com/en-us/azure/azure-sql/)
- [Repository Pattern Documentation](../docs/REPOSITORY_PATTERN.md)
- [Database Setup Guide](../docs/SETUP.md)
