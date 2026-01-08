//! Example demonstrating repository pattern usage.
//!
//! This example shows how to use the repository pattern for database operations.

use std::sync::Arc;
use tsi_rust::db::{
    models::Schedule, RepositoryBuilder, RepositoryError, RepositoryFactory, RepositoryType,
    ScheduleRepository,
};

/// Example 1: Basic usage with automatic configuration
async fn example_basic_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 1: Basic Usage ===");

    // Create repository from environment
    let repo = RepositoryFactory::from_env().await?;

    // Check connection health
    let is_healthy = repo.health_check().await?;
    println!("Database connection healthy: {}", is_healthy);

    // List all schedules
    let schedules = repo.list_schedules().await?;
    println!("Found {} schedules", schedules.len());

    for schedule_info in schedules.iter().take(5) {
        println!(
            "  - {} (ID: {}, blocks: {})",
            schedule_info.schedule_name, schedule_info.schedule_id, schedule_info.num_blocks
        );
    }

    Ok(())
}

/// Example 2: Using the builder pattern
async fn example_builder_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 2: Builder Pattern ===");

    // Build with explicit configuration
    let repo = RepositoryBuilder::new()
        .repository_type(RepositoryType::Test)
        .build()
        .await?;

    println!("Created test repository");
    println!("Health check: {}", repo.health_check().await?);

    Ok(())
}

/// Example 3: Dependency injection pattern
struct ScheduleService {
    repo: Arc<dyn ScheduleRepository>,
}

impl ScheduleService {
    pub fn new(repo: Arc<dyn ScheduleRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_schedule_summary(&self, schedule_id: i64) -> Result<String, RepositoryError> {
        let schedule = self.repo.get_schedule(schedule_id).await?;

        Ok(format!(
            "Schedule '{}' has {} blocks and {} dark periods",
            schedule.name,
            schedule.blocks.len(),
            schedule.dark_periods.len()
        ))
    }

    pub async fn get_total_blocks(&self) -> Result<usize, RepositoryError> {
        let schedules = self.repo.list_schedules().await?;
        Ok(schedules.iter().map(|s| s.num_blocks as usize).sum())
    }
}

async fn example_dependency_injection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 3: Dependency Injection ===");

    // Create repository
    let repo = RepositoryFactory::create_test();

    // Inject into service
    let service = ScheduleService::new(repo);

    // Use service
    let total_blocks = service.get_total_blocks().await?;
    println!("Total blocks across all schedules: {}", total_blocks);

    Ok(())
}

/// Example 4: Error handling
async fn example_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 4: Error Handling ===");

    let repo = RepositoryFactory::create_test();

    // Try to get non-existent schedule
    match repo.get_schedule(99999).await {
        Ok(schedule) => println!("Found schedule: {}", schedule.name),
        Err(RepositoryError::NotFound(msg)) => {
            println!("Expected error - Schedule not found: {}", msg);
        }
        Err(e) => println!("Unexpected error: {}", e),
    }

    Ok(())
}

/// Example 5: Switching implementations
async fn example_switching_implementations() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Example 5: Switching Implementations ===");

    // Function that works with any repository
    async fn count_schedules(repo: &dyn ScheduleRepository) -> Result<usize, RepositoryError> {
        let schedules = repo.list_schedules().await?;
        Ok(schedules.len())
    }

    // Use test repository
    let test_repo = RepositoryFactory::create_test();
    let test_count = count_schedules(&*test_repo).await?;
    println!("Test repository schedule count: {}", test_count);

    // Same function works with Azure repository (if configured)
    // let azure_repo = RepositoryFactory::from_env().await?;
    // let azure_count = count_schedules(&*azure_repo).await?;
    // println!("Azure repository schedule count: {}", azure_count);

    Ok(())
}

/// Example 6: Using test repository for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use tsi_rust::db::repositories::TestRepository;

    #[tokio::test]
    async fn test_schedule_storage() {
        let repo = TestRepository::new();

        // Create test schedule
        let schedule = Schedule {
            name: "Test Schedule".to_string(),
            blocks: vec![],
            dark_periods: vec![],
            checksum: "abc123".to_string(),
        };

        // Store schedule
        let metadata = repo.store_schedule(&schedule).await.unwrap();
        assert!(metadata.schedule_id.is_some());

        // Retrieve schedule
        let schedule_id = metadata.schedule_id.unwrap();
        let retrieved = repo.get_schedule(schedule_id).await.unwrap();
        assert_eq!(retrieved.name, schedule.name);

        // Verify it appears in list
        let schedules = repo.list_schedules().await.unwrap();
        assert_eq!(schedules.len(), 1);
    }

    #[tokio::test]
    async fn test_schedule_service() {
        let repo = RepositoryFactory::create_test();
        let service = ScheduleService::new(repo);

        let total = service.get_total_blocks().await.unwrap();
        assert_eq!(total, 0); // Empty repository
    }

    #[tokio::test]
    async fn test_not_found_error() {
        let repo = TestRepository::new();

        let result = repo.get_schedule(999).await;
        assert!(matches!(result, Err(RepositoryError::NotFound { .. })));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Repository Pattern Examples\n");

    // Run examples that don't require Azure
    example_builder_pattern().await?;
    example_dependency_injection().await?;
    example_error_handling().await?;
    example_switching_implementations().await?;

    // Uncomment to run Azure examples (requires configuration)
    // example_basic_usage().await?;

    println!("\n✓ All examples completed successfully!");
    Ok(())
}
