# Repository Pattern Implementation

This document provides comprehensive documentation for the Repository pattern implementation in the TSI Rust backend.

## Overview

The Repository pattern has been implemented to decouple database operations from business logic, making the codebase more testable, maintainable, and flexible for future migrations.

## Architecture

### Components

1. **`ScheduleRepository` Trait** (`src/db/repository.rs`)
   - Defines the interface for all database operations
   - Async methods returning `RepositoryResult<T>`
   - Thread-safe (`Send + Sync`)

2. **`AzureRepository`** (`src/db/repositories/azure.rs`)
   - Production implementation using Azure SQL Server
   - Wraps existing database operations
   - Requires initialized connection pool

3. **`TestRepository`** (`src/db/repositories/test.rs`)
   - In-memory mock implementation
   - Fast, isolated, deterministic tests
   - Helper methods for populating test data

4. **`RepositoryFactory`** (`src/db/factory.rs`)
   - Factory pattern for creating repositories
   - Handles initialization and configuration
   - Supports environment-based configuration

5. **`RepositoryBuilder`** (`src/db/factory.rs`)
   - Fluent API for repository configuration
   - Chain methods to configure settings
   - Build pattern for clean initialization

## Usage Examples

### Basic Usage with Azure (Production)

```rust
use tsi_rust::db::{DbConfig, RepositoryFactory, ScheduleRepository};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment
    let config = DbConfig::from_env()?;
    
    // Create Azure repository
    let repo = RepositoryFactory::create_azure(&config).await?;
    
    // Use the repository
    let schedules = repo.list_schedules().await?;
    println!("Found {} schedules", schedules.len());
    
    Ok(())
}
```

### Using the Builder Pattern

```rust
use tsi_rust::db::{RepositoryBuilder, RepositoryType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build from environment variables
    let repo = RepositoryBuilder::new()
        .from_env()?
        .build()
        .await?;
    
    // Or specify manually
    let repo = RepositoryBuilder::new()
        .repository_type(RepositoryType::Test)
        .build()
        .await?;
    
    Ok(())
}
```

### Unit Testing with Test Repository

```rust
use tsi_rust::db::{repositories::TestRepository, ScheduleRepository};
use tsi_rust::api::Schedule;

#[tokio::test]
async fn test_schedule_operations() {
    // Create test repository
    let repo = TestRepository::new();
    
    // Create test schedule
    let schedule = Schedule {
        name: "Test Schedule".to_string(),
        blocks: vec![],
        dark_periods: vec![],
        checksum: "test123".to_string(),
    };
    
    // Store schedule
    let metadata = repo.store_schedule(&schedule).await.unwrap();
    let schedule_id = metadata.schedule_id.unwrap();
    
    // Retrieve schedule
    let retrieved = repo.get_schedule(schedule_id).await.unwrap();
    assert_eq!(retrieved.name, schedule.name);
    
    // List all schedules
    let schedules = repo.list_schedules().await.unwrap();
    assert_eq!(schedules.len(), 1);
}
```

### Advanced: Dependency Injection

```rust
use std::sync::Arc;
use tsi_rust::db::{ScheduleRepository, RepositoryFactory, RepositoryType};

// Service that depends on repository
struct ScheduleService {
    repo: Arc<dyn ScheduleRepository>,
}

impl ScheduleService {
    pub fn new(repo: Arc<dyn ScheduleRepository>) -> Self {
        Self { repo }
    }
    
    pub async fn get_schedule_count(&self) -> Result<usize, String> {
        let schedules = self.repo.list_schedules()
            .await
            .map_err(|e| e.to_string())?;
        Ok(schedules.len())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create repository (Azure for production)
    let repo = RepositoryFactory::from_env().await?;
    
    // Inject into service
    let service = ScheduleService::new(repo);
    
    let count = service.get_schedule_count().await?;
    println!("Total schedules: {}", count);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_service_with_mock() {
        // Use test repository for testing
        let repo = RepositoryFactory::create_test();
        let service = ScheduleService::new(repo);
        
        // Test without hitting real database
        let count = service.get_schedule_count().await.unwrap();
        assert_eq!(count, 0);
    }
}
```

### Environment Configuration

Set the `REPOSITORY_TYPE` environment variable to choose the implementation:

```bash
# Use Azure (default)
export REPOSITORY_TYPE=azure

# Use test repository
export REPOSITORY_TYPE=test
```

Azure configuration requires these environment variables:
```bash
export DB_SERVER=your-server.database.windows.net
export DB_DATABASE=your-database
export DB_USERNAME=your-username
export DB_PASSWORD=your-password
# ... other DB configuration
```

## Migration Guide

### Migrating Existing Code

#### Before (Direct Database Access)
```rust
use tsi_rust::db::operations;

async fn get_schedules() -> Result<Vec<crate::api::ScheduleInfo>, String> {
    operations::list_schedules().await
}
```

#### After (Repository Pattern)
```rust
use tsi_rust::db::{ScheduleRepository, RepositoryFactory};

async fn get_schedules(repo: &dyn ScheduleRepository) -> Result<Vec<crate::api::ScheduleInfo>, String> {
    repo.list_schedules()
        .await
        .map_err(|e| e.to_string())
}

// In main or initialization
let repo = RepositoryFactory::from_env().await?;
let schedules = get_schedules(&*repo).await?;
```

### Backward Compatibility

The existing direct access functions remain available for gradual migration:

```rust
// Old way (still works)
use tsi_rust::db::operations;
let schedules = operations::list_schedules().await?;

// New way (recommended)
use tsi_rust::db::{RepositoryFactory, ScheduleRepository};
let repo = RepositoryFactory::from_env().await?;
let schedules = repo.list_schedules().await?;
```

## Testing Strategies

### Unit Tests with Mock Repository

```rust
#[tokio::test]
async fn test_with_mock_data() {
    let repo = TestRepository::new();
    
    // Pre-populate test data
    let schedule = Schedule { /* ... */ };
    let schedule_id = repo.add_test_schedule(schedule);
    
    // Add dark periods
    repo.add_dark_periods(schedule_id, vec![
        (60000.0, 60001.0),
        (60002.0, 60003.0),
    ]);
    
    // Test your logic
    let periods = repo.fetch_dark_periods(schedule_id).await.unwrap();
    assert_eq!(periods.len(), 2);
}
```

### Integration Tests with Azure

```rust
#[tokio::test]
#[ignore] // Only run with --ignored flag
async fn test_azure_integration() {
    let config = DbConfig::from_env().unwrap();
    let repo = RepositoryFactory::create_azure(&config).await.unwrap();
    
    // Test real database operations
    assert!(repo.health_check().await.unwrap());
}
```

## Error Handling

The repository pattern uses a unified error type:

```rust
use tsi_rust::db::{RepositoryError, RepositoryResult};

async fn example(repo: &dyn ScheduleRepository) -> RepositoryResult<()> {
    match repo.get_schedule(123).await {
        Ok(schedule) => println!("Found: {}", schedule.name),
        Err(RepositoryError::NotFound(msg)) => println!("Not found: {}", msg),
        Err(RepositoryError::ConnectionError(msg)) => println!("Connection error: {}", msg),
        Err(e) => println!("Other error: {}", e),
    }
    Ok(())
}
```

## Best Practices

1. **Dependency Injection**: Pass repository as parameter rather than creating it in functions
2. **Use Arc<dyn Repository>**: For shared ownership across threads
3. **Mock for Tests**: Always use `TestRepository` for unit tests
4. **Environment Config**: Use environment variables for production configuration
5. **Error Handling**: Match on specific error types when handling failures
6. **Builder Pattern**: Use `RepositoryBuilder` for complex initialization

## Performance Considerations

- **Azure Repository**: Uses connection pooling (15 connections max)
- **Test Repository**: In-memory operations, no I/O overhead
- **Thread Safety**: All implementations are `Send + Sync`
- **Connection Reuse**: Pool reuses connections, no reconnection overhead

## Future Extensions

To add a new database backend:

1. Create a new file in `src/db/repositories/`
2. Implement the `ScheduleRepository` trait
3. Add to `RepositoryType` enum in `factory.rs`
4. Update factory to create your implementation

Example:
```rust
// src/db/repositories/postgres.rs
pub struct PostgresRepository { /* ... */ }

#[async_trait]
impl ScheduleRepository for PostgresRepository {
    // Implement all trait methods
}

// Add to factory.rs
enum RepositoryType {
    Azure,
    Postgres,  // New!
    Test,
}
```

## Troubleshooting

### Connection Pool Not Initialized
```
Error: "Database pool not initialized"
```
**Solution**: Call `pool::init_pool()` or use `RepositoryFactory::create_azure()` which handles initialization.

### Azure Authentication Failures
```
Error: "Login failed"
```
**Solution**: Verify environment variables are set correctly and Azure firewall allows your IP.

### Type Inference Issues
```
Error: "cannot infer type"
```
**Solution**: Explicitly type the repository: `let repo: Arc<dyn ScheduleRepository> = ...`

## API Reference

See inline documentation in:
- `src/db/repository.rs` - Trait definition
- `src/db/repositories/azure.rs` - Azure implementation
- `src/db/repositories/test.rs` - Test implementation
- `src/db/factory.rs` - Factory and builder

## Examples

Complete working examples can be found in:
- `examples/repository_usage.rs` (basic usage)
- `tests/repository_integration_tests.rs` (integration tests)
- `src/db/repositories/test.rs` (unit test examples)
