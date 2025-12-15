//! Integration tests for repository implementations.

use std::sync::Arc;
use tsi_rust::db::{
    models::Schedule, repositories::TestRepository, RepositoryError, ScheduleRepository,
};

#[tokio::test]
async fn test_repository_health_check() {
    let repo: Arc<dyn ScheduleRepository> = Arc::new(TestRepository::new());
    let result = repo.health_check().await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_store_and_retrieve_schedule() {
    let repo = TestRepository::new();

    let schedule = Schedule {
        id: None,
        name: "Integration Test Schedule".to_string(),
        blocks: vec![],
        dark_periods: vec![],
        checksum: "integration_test_123".to_string(),
    };

    // Store the schedule
    let metadata = repo.store_schedule(&schedule).await.unwrap();
    assert!(metadata.schedule_id.is_some());
    assert_eq!(metadata.schedule_name, schedule.name);
    assert_eq!(metadata.checksum, schedule.checksum);

    // Retrieve by ID
    let schedule_id = metadata.schedule_id.unwrap();
    let retrieved = repo.get_schedule(schedule_id).await.unwrap();
    assert_eq!(retrieved.name, schedule.name);
    assert_eq!(retrieved.checksum, schedule.checksum);
}

#[tokio::test]
async fn test_list_schedules() {
    let repo = TestRepository::new();

    // Initially empty
    let schedules = repo.list_schedules().await.unwrap();
    assert_eq!(schedules.len(), 0);

    // Add schedules
    for i in 1..=3 {
        let schedule = Schedule {
        id: None,
            name: format!("Schedule {}", i),
            blocks: vec![],
            dark_periods: vec![],
            checksum: format!("checksum_{}", i),
        };
        repo.store_schedule(&schedule).await.unwrap();
    }

    // Verify list
    let schedules = repo.list_schedules().await.unwrap();
    assert_eq!(schedules.len(), 3);
}

#[tokio::test]
async fn test_not_found_error() {
    let repo = TestRepository::new();

    let result = repo.get_schedule(99999).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RepositoryError::NotFound(_)));
}

#[tokio::test]
async fn test_analytics_lifecycle() {
    let repo = TestRepository::new();

    // Create schedule
    let schedule = Schedule {
        id: None,
        name: "Analytics Test".to_string(),
        blocks: vec![],
        dark_periods: vec![],
        checksum: "analytics_test".to_string(),
    };

    let metadata = repo.store_schedule(&schedule).await.unwrap();
    let schedule_id = metadata.schedule_id.unwrap();

    // Initially no analytics
    assert!(!repo.has_analytics_data(schedule_id).await.unwrap());

    // Populate analytics
    let rows = repo.populate_schedule_analytics(schedule_id).await.unwrap();
    assert!(rows > 0);
    assert!(repo.has_analytics_data(schedule_id).await.unwrap());

    // Delete analytics
    let deleted = repo.delete_schedule_analytics(schedule_id).await.unwrap();
    assert!(deleted > 0);
    assert!(!repo.has_analytics_data(schedule_id).await.unwrap());
}

#[tokio::test]
async fn test_summary_analytics_lifecycle() {
    let repo = TestRepository::new();

    let schedule = Schedule {
        id: None,
        name: "Summary Test".to_string(),
        blocks: vec![],
        dark_periods: vec![],
        checksum: "summary_test".to_string(),
    };

    let metadata = repo.store_schedule(&schedule).await.unwrap();
    let schedule_id = metadata.schedule_id.unwrap();

    // Initially no summary
    assert!(!repo.has_summary_analytics(schedule_id).await.unwrap());
    assert!(repo.fetch_schedule_summary(schedule_id).await.unwrap().is_none());

    // Populate summary
    repo.populate_summary_analytics(schedule_id, 10).await.unwrap();
    assert!(repo.has_summary_analytics(schedule_id).await.unwrap());
    
    let summary = repo.fetch_schedule_summary(schedule_id).await.unwrap();
    assert!(summary.is_some());
    assert_eq!(summary.unwrap().schedule_id, schedule_id);

    // Delete summary
    repo.delete_summary_analytics(schedule_id).await.unwrap();
    assert!(!repo.has_summary_analytics(schedule_id).await.unwrap());
}

#[tokio::test]
async fn test_validation_lifecycle() {
    let repo = TestRepository::new();

    let schedule = Schedule {
        id: None,
        name: "Validation Test".to_string(),
        blocks: vec![],
        dark_periods: vec![],
        checksum: "validation_test".to_string(),
    };

    let metadata = repo.store_schedule(&schedule).await.unwrap();
    let schedule_id = metadata.schedule_id.unwrap();

    // Initially no validation results
    assert!(!repo.has_validation_results(schedule_id).await.unwrap());

    // The validation results require ValidationResult structs from services module
    // For now, just test the has/delete methods
    
    // Delete (should return 0 as nothing exists)
    let deleted = repo.delete_validation_results(schedule_id).await.unwrap();
    assert_eq!(deleted, 0);
}

#[tokio::test]
async fn test_concurrent_access() {
    use tokio::task::JoinSet;

    let repo = Arc::new(TestRepository::new());
    let mut set = JoinSet::new();

    // Spawn multiple tasks accessing the repository concurrently
    for i in 0..10 {
        let repo_clone = repo.clone();
        set.spawn(async move {
            let schedule = Schedule {
        id: None,
                name: format!("Concurrent Schedule {}", i),
                blocks: vec![],
                dark_periods: vec![],
                checksum: format!("concurrent_{}", i),
            };
            repo_clone.store_schedule(&schedule).await
        });
    }

    // Wait for all tasks
    let mut count = 0;
    while let Some(result) = set.join_next().await {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
        count += 1;
    }

    assert_eq!(count, 10);

    // Verify all schedules were stored
    let schedules = repo.list_schedules().await.unwrap();
    assert_eq!(schedules.len(), 10);
}

#[tokio::test]
async fn test_helper_methods() {
    let repo = TestRepository::new();

    // Test helper methods
    assert_eq!(repo.schedule_count(), 0);
    assert!(!repo.has_schedule(1));

    // Add schedule using helper
    let schedule = Schedule {
        id: None,
        name: "Helper Test".to_string(),
        blocks: vec![],
        dark_periods: vec![],
        checksum: "helper".to_string(),
    };
    let schedule_id = repo.store_schedule(&schedule).await.unwrap().schedule_id.unwrap();

    assert_eq!(repo.schedule_count(), 1);
    assert!(repo.has_schedule(schedule_id));

    // Clear repository
    repo.clear();
    assert_eq!(repo.schedule_count(), 0);
}

#[tokio::test]
async fn test_connection_unhealthy() {
    let repo = TestRepository::new();

    // Set unhealthy
    repo.set_healthy(false);
    assert!(!repo.health_check().await.unwrap());

    // Try to store (should fail)
    let schedule = Schedule {
        id: None,
        name: "Should Fail".to_string(),
        blocks: vec![],
        dark_periods: vec![],
        checksum: "fail".to_string(),
    };
    
    let result = repo.store_schedule(&schedule).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RepositoryError::ConnectionError(_)));
}
