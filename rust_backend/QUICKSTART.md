# Quick Start: Repository Configuration

## For Developers (Local Development)

**Default setup - no configuration needed!**

The repository is already configured to use the local in-memory storage by default via the `repository.toml` file.

```bash
# Just build and run
cd rust_backend
cargo build
cargo test
```

## For Production (Azure SQL Server)

### Option 1: Configuration File (Recommended)

1. Copy the example file:
```bash
cd rust_backend
cp repository.azure.toml.example repository.toml
```

2. Edit `repository.toml`:
```toml
[repository]
type = "azure"

[database]
server = "your-server.database.windows.net"
database = "your-database"
auth_method = "azure_identity"  # or "sql" for SQL auth
username = "user@domain.com"    # Your Azure AD email or SQL user
password = "your-password"
```

3. Build and run:
```bash
cargo build
cargo test
```

### Option 2: Environment Variables

```bash
# Set repository type
export REPOSITORY_TYPE=azure

# Set database connection
export DB_SERVER=your-server.database.windows.net
export DB_DATABASE=your-database
export DB_USERNAME=user@domain.com
export DB_PASSWORD=your-password
export DB_AUTH_METHOD=aad_password  # or sql_password

# Build and run
cargo build
```

## Quick Commands

```bash
# Switch to local repository
echo 'type = "local"' > repository.toml

# Switch to Azure repository
export REPOSITORY_TYPE=azure

# Run tests
cargo test

# Run integration tests
cargo test --test repository_integration_tests

# Build library
cargo build --lib
```

## Verify Configuration

```bash
# Check current config file
cat repository.toml

# Check environment variables
echo $REPOSITORY_TYPE
echo $DB_SERVER
```

## Common Issues

### "No repository.toml found"
Create the file in the `rust_backend/` directory.

### Azure connection fails
- Verify firewall rules
- Check credentials
- Ensure server hostname is correct

### Want to switch back to local?
Edit `repository.toml` and set `type = "local"`

## More Information

- Full guide: [REPOSITORY_CONFIG.md](REPOSITORY_CONFIG.md)
- Examples: [examples/repository_config_example.py](../examples/repository_config_example.py)
- Changes summary: [REFACTORING_SUMMARY.md](REFACTORING_SUMMARY.md)
