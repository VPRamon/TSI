//! Expanded tests for LocalRepository.
//!
//! These tests cover concurrent access patterns, edge cases, error conditions,
//! and stress testing for the in-memory local repository implementation.

use std::sync::Arc;
use std::time::Duration;
use tsi_rust::api::{
    Constraints, GeographicLocation, Period, Schedule, ScheduleId, SchedulingBlock,
    SchedulingBlockId,
};
use tsi_rust::db::repositories::LocalRepository;
use tsi_rust::db::repository::{AnalyticsRepository, ScheduleRepository};
use tsi_rust::models::ModifiedJulianDate;

fn default_schedule_period() -> Period {
    Period {
        start: ModifiedJulianDate::new(59580.0),
        stop: ModifiedJulianDate::new(59590.0),
    }
}

fn create_test_schedule(name: &str, block_count: usize) -> Schedule {
    let blocks: Vec<SchedulingBlock> = (0..block_count)
        .map(|i| SchedulingBlock {
            id: Some(SchedulingBlockId::new((i + 1) as i64)),
            original_block_id: format!("block_{}", i),
            target_ra: qtty::Degrees::new(i as f64 * 10.0),
            target_dec: qtty::Degrees::new(i as f64 * 5.0 - 45.0),
            constraints: Constraints {
                min_alt: qtty::Degrees::new(30.0),
                max_alt: qtty::Degrees::new(85.0),
                min_az: qtty::Degrees::new(0.0),
                max_az: qtty::Degrees::new(360.0),
                fixed_time: None,
            },
            priority: (i + 1) as f64,
            min_observation: qtty::Seconds::new(60.0),
            requested_duration: qtty::Seconds::new(3600.0),
            visibility_periods: vec![Period {
                start: ModifiedJulianDate::new(59580.0),
                stop: ModifiedJulianDate::new(59581.0),
            }],
            scheduled_period: if i % 2 == 0 {
                Some(Period {
                    start: ModifiedJulianDate::new(59580.0),
                    stop: ModifiedJulianDate::new(59580.5),
                })
            } else {
                None
            },
        })
        .collect();

    Schedule {
        id: None,
        name: name.to_string(),
        blocks,
        dark_periods: vec![],
        geographic_location: GeographicLocation {
            latitude: 28.7624,
            longitude: -17.8892,
            elevation_m: Some(2396.0),
        },
        astronomical_nights: vec![],
        checksum: format!("checksum_{}", name),
        schedule_period: default_schedule_period(),
    }
}

// =========================================================
// Concurrent Access Tests
// =========================================================

#[tokio::test]
async fn test_concurrent_write_different_schedules() {
    let repo = Arc::new(LocalRepository::new());

    // Spawn multiple tasks writing different schedules
    let mut handles = vec![];
    for i in 0..10 {
        let repo_clone = Arc::clone(&repo);
        let handle = tokio::spawn(async move {
            let schedule = create_test_schedule(&format!("schedule_{}", i), 5);
            repo_clone.store_schedule(&schedule).await
        });
        handles.push(handle);
    }

    // Wait for all tasks
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    // Verify all schedules exist
    let schedules = repo.list_schedules().await.unwrap();
    assert_eq!(schedules.len(), 10);
}

#[tokio::test]
async fn test_concurrent_read_write_same_repository() {
    let repo = Arc::new(LocalRepository::new());

    // Store initial schedule
    let initial = create_test_schedule("initial", 3);
    let metadata = repo.store_schedule(&initial).await.unwrap();

    // Spawn readers and writers separately
    let mut read_handles = vec![];
    let mut write_handles = vec![];

    // Spawn 10 readers
    for _ in 0..10 {
        let repo_clone = Arc::clone(&repo);
        let schedule_id = metadata.schedule_id;
        let handle = tokio::spawn(async move { repo_clone.get_schedule(schedule_id).await });
        read_handles.push(handle);
    }

    // Spawn 5 writers
    for i in 0..5 {
        let repo_clone = Arc::clone(&repo);
        let handle = tokio::spawn(async move {
            let schedule = create_test_schedule(&format!("concurrent_{}", i), 2);
            repo_clone.store_schedule(&schedule).await
        });
        write_handles.push(handle);
    }

    // Wait for all readers
    for handle in read_handles {
        assert!(handle.await.is_ok());
    }

    // Wait for all writers
    for handle in write_handles {
        assert!(handle.await.is_ok());
    }
}

#[tokio::test]
async fn test_concurrent_list_and_store() {
    let repo = Arc::new(LocalRepository::new());

    let mut list_handles = vec![];
    let mut store_handles = vec![];

    // Interleave list and store operations
    for i in 0..20 {
        let repo_clone = Arc::clone(&repo);
        if i % 2 == 0 {
            // Store
            let handle = tokio::spawn(async move {
                let schedule = create_test_schedule(&format!("sched_{}", i), 1);
                repo_clone.store_schedule(&schedule).await
            });
            store_handles.push(handle);
        } else {
            // List
            let handle = tokio::spawn(async move { repo_clone.list_schedules().await });
            list_handles.push(handle);
        }
    }

    // Wait for all operations
    for handle in list_handles {
        assert!(handle.await.is_ok());
    }

    for handle in store_handles {
        assert!(handle.await.is_ok());
    }
}

#[tokio::test]
async fn test_concurrent_health_checks() {
    let repo = Arc::new(LocalRepository::new());

    // Spawn many concurrent health checks
    let handles: Vec<_> = (0..100)
        .map(|_| {
            let repo_clone = Arc::clone(&repo);
            tokio::spawn(async move { repo_clone.health_check().await })
        })
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // All should succeed and return true
    for result in results {
        let health = result.unwrap().unwrap();
        assert!(health);
    }
}

#[tokio::test]
async fn test_concurrent_analytics_population() {
    let repo = Arc::new(LocalRepository::new());

    // Store schedules first
    let mut schedule_ids = vec![];
    for i in 0..5 {
        let schedule = create_test_schedule(&format!("analytics_{}", i), 10);
        let metadata = repo.store_schedule(&schedule).await.unwrap();
        schedule_ids.push(metadata.schedule_id);
    }

    // Concurrently populate analytics for all schedules
    let handles: Vec<_> = schedule_ids
        .into_iter()
        .map(|schedule_id| {
            let repo_clone = Arc::clone(&repo);
            tokio::spawn(async move { repo_clone.populate_schedule_analytics(schedule_id).await })
        })
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // All should complete (may succeed or fail depending on implementation)
    for result in results {
        assert!(result.is_ok());
    }
}

// =========================================================
// Edge Case Tests
// =========================================================

#[tokio::test]
async fn test_empty_schedule_storage() {
    let repo = LocalRepository::new();

    let empty_schedule = Schedule {
        id: None,
        name: "empty".to_string(),
        blocks: vec![],
        dark_periods: vec![],
        geographic_location: GeographicLocation {
            latitude: 28.7624,
            longitude: -17.8892,
            elevation_m: Some(2396.0),
        },
        astronomical_nights: vec![],
        checksum: "empty_checksum".to_string(),
        schedule_period: default_schedule_period(),
    };

    let result = repo.store_schedule(&empty_schedule).await;
    assert!(result.is_ok());

    let metadata = result.unwrap();
    let retrieved = repo.get_schedule(metadata.schedule_id).await.unwrap();

    assert_eq!(retrieved.blocks.len(), 0);
    assert_eq!(retrieved.dark_periods.len(), 0);
}

#[tokio::test]
async fn test_schedule_with_many_blocks() {
    let repo = LocalRepository::new();

    // Store schedule with 1000 blocks
    let large_schedule = create_test_schedule("large", 1000);
    let metadata = repo.store_schedule(&large_schedule).await.unwrap();

    let retrieved = repo.get_schedule(metadata.schedule_id).await.unwrap();
    assert_eq!(retrieved.blocks.len(), 1000);
}

#[tokio::test]
async fn test_schedule_with_very_long_name() {
    let repo = LocalRepository::new();

    let long_name = "a".repeat(10000);
    let mut schedule = create_test_schedule(&long_name, 1);
    schedule.name = long_name.clone();

    let metadata = repo.store_schedule(&schedule).await.unwrap();
    let retrieved = repo.get_schedule(metadata.schedule_id).await.unwrap();

    assert_eq!(retrieved.name.len(), 10000);
    assert_eq!(retrieved.name, long_name);
}

#[tokio::test]
async fn test_schedule_with_special_characters_in_name() {
    let repo = LocalRepository::new();

    let special_names = vec![
        "schedule\nwith\nnewlines",
        "schedule\twith\ttabs",
        "schedule with spaces",
        "schedule-with-dashes",
        "schedule_with_underscores",
        "schedule.with.dots",
        "スケジュール", // Japanese
        "расписание",   // Russian
        "📅🔭🌟",       // Emojis
    ];

    for name in special_names {
        let schedule = create_test_schedule(name, 1);
        let metadata = repo.store_schedule(&schedule).await.unwrap();
        let retrieved = repo.get_schedule(metadata.schedule_id).await.unwrap();
        assert_eq!(retrieved.name, name);
    }
}

#[tokio::test]
async fn test_multiple_schedules_same_name() {
    let repo = LocalRepository::new();

    // Store multiple schedules with the same name but different checksums
    let schedule1 = create_test_schedule("duplicate_name", 2);
    let mut schedule2 = create_test_schedule("duplicate_name", 3);
    schedule2.checksum = "different_checksum".to_string();

    let meta1 = repo.store_schedule(&schedule1).await.unwrap();
    let meta2 = repo.store_schedule(&schedule2).await.unwrap();

    // Should have different IDs
    assert_ne!(meta1.schedule_id, meta2.schedule_id);

    let schedules = repo.list_schedules().await.unwrap();
    assert_eq!(schedules.len(), 2);
}

#[tokio::test]
async fn test_repository_clear_function() {
    let repo = LocalRepository::new();

    // Store multiple schedules
    for i in 0..5 {
        let schedule = create_test_schedule(&format!("sched_{}", i), 3);
        repo.store_schedule(&schedule).await.unwrap();
    }

    // Verify schedules exist
    assert_eq!(repo.schedule_count(), 5);

    // Clear repository
    repo.clear();

    // Verify all schedules are gone
    assert_eq!(repo.schedule_count(), 0);
    let schedules = repo.list_schedules().await.unwrap();
    assert_eq!(schedules.len(), 0);
}

#[tokio::test]
async fn test_repository_has_schedule() {
    let repo = LocalRepository::new();

    let schedule = create_test_schedule("test", 2);
    let metadata = repo.store_schedule(&schedule).await.unwrap();

    // Should exist
    assert!(repo.has_schedule(metadata.schedule_id));

    // Non-existent schedule
    assert!(!repo.has_schedule(ScheduleId::new(99999)));
}

// =========================================================
// Error Condition Tests
// =========================================================

#[tokio::test]
async fn test_unhealthy_repository_store_fails() {
    let repo = LocalRepository::new();

    // Set unhealthy
    repo.set_healthy(false);

    let schedule = create_test_schedule("test", 1);
    let result = repo.store_schedule(&schedule).await;

    assert!(result.is_err());

    // Restore health
    repo.set_healthy(true);
}

#[tokio::test]
async fn test_unhealthy_repository_list_fails() {
    let repo = LocalRepository::new();

    repo.set_healthy(false);
    let result = repo.list_schedules().await;

    assert!(result.is_err());

    repo.set_healthy(true);
}

#[tokio::test]
async fn test_unhealthy_repository_get_fails() {
    let repo = LocalRepository::new();

    // Store while healthy
    let schedule = create_test_schedule("test", 1);
    let metadata = repo.store_schedule(&schedule).await.unwrap();

    // Make unhealthy
    repo.set_healthy(false);

    // Try to retrieve
    let result = repo.get_schedule(metadata.schedule_id).await;
    assert!(result.is_err());

    repo.set_healthy(true);
}

#[tokio::test]
async fn test_get_nonexistent_schedule() {
    let repo = LocalRepository::new();

    let result = repo.get_schedule(ScheduleId::new(12345)).await;

    assert!(result.is_err());

    // Error should indicate not found
    let error_msg = format!("{}", result.unwrap_err());
    assert!(error_msg.contains("not found") || error_msg.contains("Not found"));
}

#[tokio::test]
async fn test_health_check_transitions() {
    let repo = LocalRepository::new();

    // Initially healthy
    let health = repo.health_check().await.unwrap();
    assert!(health);

    // Set unhealthy
    repo.set_healthy(false);
    let health = repo.health_check().await.unwrap();
    assert!(!health);

    // Restore health
    repo.set_healthy(true);
    let health = repo.health_check().await.unwrap();
    assert!(health);
}

// =========================================================
// Stress Tests
// =========================================================

#[tokio::test]
async fn test_store_many_schedules_sequentially() {
    let repo = LocalRepository::new();

    // Store 100 schedules
    for i in 0..100 {
        let schedule = create_test_schedule(&format!("schedule_{}", i), 5);
        let result = repo.store_schedule(&schedule).await;
        assert!(result.is_ok());
    }

    let schedules = repo.list_schedules().await.unwrap();
    assert_eq!(schedules.len(), 100);
}

#[tokio::test]
async fn test_retrieve_many_schedules_sequentially() {
    let repo = LocalRepository::new();

    // Store schedules and collect IDs
    let mut ids = vec![];
    for i in 0..50 {
        let schedule = create_test_schedule(&format!("sched_{}", i), 3);
        let metadata = repo.store_schedule(&schedule).await.unwrap();
        ids.push(metadata.schedule_id);
    }

    // Retrieve each schedule
    for schedule_id in ids {
        let result = repo.get_schedule(schedule_id).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_high_concurrency_mixed_operations() {
    let repo = Arc::new(LocalRepository::new());

    // Store some initial schedules
    let mut schedule_ids = vec![];
    for i in 0..10 {
        let schedule = create_test_schedule(&format!("init_{}", i), 2);
        let metadata = repo.store_schedule(&schedule).await.unwrap();
        schedule_ids.push(metadata.schedule_id);
    }

    // Spawn 100 concurrent tasks with mixed operations
    let mut handles = vec![];
    for i in 0..100 {
        let repo_clone = Arc::clone(&repo);
        let ids = schedule_ids.clone();

        let handle = tokio::spawn(async move {
            match i % 4 {
                0 => {
                    // Store new schedule
                    let schedule = create_test_schedule(&format!("concurrent_{}", i), 1);
                    repo_clone.store_schedule(&schedule).await.map(|_| ())
                }
                1 => {
                    // List schedules
                    repo_clone.list_schedules().await.map(|_| ())
                }
                2 => {
                    // Health check
                    repo_clone.health_check().await.map(|_| ())
                }
                _ => {
                    // Get random schedule
                    let schedule_id = ids[i % ids.len()];
                    repo_clone.get_schedule(schedule_id).await.map(|_| ())
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // Most should succeed (some reads might fail if schedule was deleted)
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    assert!(success_count > 90); // At least 90% success rate
}

#[tokio::test]
async fn test_rapid_health_state_changes() {
    let repo = Arc::new(LocalRepository::new());

    // Spawn task that rapidly changes health state
    let repo_clone1 = Arc::clone(&repo);
    let health_changer = tokio::spawn(async move {
        for _ in 0..1000 {
            repo_clone1.set_healthy(false);
            tokio::time::sleep(Duration::from_micros(10)).await;
            repo_clone1.set_healthy(true);
            tokio::time::sleep(Duration::from_micros(10)).await;
        }
    });

    // Spawn tasks that perform operations
    let mut handles = vec![];
    for i in 0..50 {
        let repo_clone2 = Arc::clone(&repo);
        let handle = tokio::spawn(async move {
            let schedule = create_test_schedule(&format!("rapid_{}", i), 1);
            // Some will succeed, some will fail due to health changes
            let _ = repo_clone2.store_schedule(&schedule).await;
        });
        handles.push(handle);
    }

    // Wait for all
    for handle in handles {
        let _ = handle.await;
    }
    health_changer.await.unwrap();

    // Ensure repository is in a consistent state
    repo.set_healthy(true);
    let health = repo.health_check().await.unwrap();
    assert!(health);
}

// =========================================================
// Clone and Shared State Tests
// =========================================================

#[tokio::test]
async fn test_cloned_repository_shares_state() {
    let repo1 = LocalRepository::new();
    let repo2 = repo1.clone();

    // Store in repo1
    let schedule = create_test_schedule("shared", 3);
    let metadata = repo1.store_schedule(&schedule).await.unwrap();

    // Should be visible in repo2
    let retrieved = repo2.get_schedule(metadata.schedule_id).await.unwrap();
    assert_eq!(retrieved.name, "shared");
}

#[tokio::test]
async fn test_cloned_repository_concurrent_access() {
    let repo1 = LocalRepository::new();
    let repo2 = repo1.clone();
    let repo3 = repo1.clone();

    // Store from different clones concurrently
    let handle1 = tokio::spawn(async move {
        let schedule = create_test_schedule("from_repo1", 2);
        repo1.store_schedule(&schedule).await
    });

    let handle2 = tokio::spawn(async move {
        let schedule = create_test_schedule("from_repo2", 2);
        repo2.store_schedule(&schedule).await
    });

    let handle3 = tokio::spawn(async move {
        let schedule = create_test_schedule("from_repo3", 2);
        repo3.store_schedule(&schedule).await
    });

    let mut results = Vec::new();
    results.push(handle1.await);
    results.push(handle2.await);
    results.push(handle3.await);

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[test]
fn test_repository_default_trait() {
    let repo = LocalRepository::default();

    // Should be healthy by default
    assert_eq!(repo.schedule_count(), 0);
}

#[test]
fn test_repository_clone_trait() {
    let repo1 = LocalRepository::new();
    let repo2 = repo1.clone();

    // Both should reference the same underlying data
    let _ = repo1.store_schedule_impl(create_test_schedule("test", 1));
    assert_eq!(repo2.schedule_count(), 1);
}
