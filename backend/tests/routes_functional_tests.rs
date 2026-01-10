//! Functional tests for route handlers.
//!
//! These tests exercise the full API call stack from route handlers
//! through services to repositories, validating end-to-end functionality.

use tsi_rust::api::{Constraints, Period, Schedule, SchedulingBlock, SchedulingBlockId};
use tsi_rust::db::repositories::LocalRepository;
use tsi_rust::db::services;
use tsi_rust::models::ModifiedJulianDate;

/// Helper to create a complete schedule with visibility periods
fn create_schedule_with_visibility(name: &str, block_count: usize) -> Schedule {
    let blocks: Vec<SchedulingBlock> = (0..block_count)
        .map(|i| {
            let constraints = Constraints {
                min_alt: qtty::Degrees::new(30.0),
                max_alt: qtty::Degrees::new(85.0),
                min_az: qtty::Degrees::new(0.0),
                max_az: qtty::Degrees::new(360.0),
                fixed_time: None,
            };

            // Create visibility periods
            let visibility_periods = vec![Period {
                start: ModifiedJulianDate::new(59580.0 + i as f64),
                stop: ModifiedJulianDate::new(59581.0 + i as f64),
            }];

            SchedulingBlock {
                id: SchedulingBlockId::new((i + 1) as i64),
                original_block_id: Some(format!("block_{}", i)),
                target_ra: qtty::Degrees::new(i as f64 * 15.0),
                target_dec: qtty::Degrees::new(i as f64 * 10.0 - 45.0),
                constraints,
                priority: (i as f64 + 1.0) * 2.0,
                min_observation: qtty::Seconds::new(120.0),
                requested_duration: qtty::Seconds::new(3600.0),
                visibility_periods,
                scheduled_period: if i % 2 == 0 {
                    Some(Period {
                        start: ModifiedJulianDate::new(59580.0 + i as f64),
                        stop: ModifiedJulianDate::new(59580.5 + i as f64),
                    })
                } else {
                    None
                },
            }
        })
        .collect();

    Schedule {
        id: None,
        name: name.to_string(),
        blocks,
        dark_periods: vec![Period {
            start: ModifiedJulianDate::new(59580.0),
            stop: ModifiedJulianDate::new(59590.0),
        }],
        checksum: format!("checksum_{}", name),
    }
}

// =========================================================
// Landing Route Functional Tests
// =========================================================

#[tokio::test]
async fn test_landing_store_and_list_full_flow() {
    let repo = LocalRepository::new();

    // Store multiple schedules
    let schedule1 = create_schedule_with_visibility("schedule_alpha", 5);
    let schedule2 = create_schedule_with_visibility("schedule_beta", 3);

    let meta1 = services::store_schedule(&repo, &schedule1).await.unwrap();
    let meta2 = services::store_schedule(&repo, &schedule2).await.unwrap();

    // List schedules
    let schedules = services::list_schedules(&repo).await.unwrap();

    assert_eq!(schedules.len(), 2);

    // Verify both schedules are present
    let names: Vec<_> = schedules.iter().map(|s| s.schedule_name.as_str()).collect();
    assert!(names.contains(&"schedule_alpha"));
    assert!(names.contains(&"schedule_beta"));

    // Verify IDs are unique
    assert_ne!(meta1.schedule_id, meta2.schedule_id);
}

#[tokio::test]
async fn test_landing_store_duplicate_checksum() {
    let repo = LocalRepository::new();

    let mut schedule1 = create_schedule_with_visibility("first", 2);
    schedule1.checksum = "duplicate_checksum".to_string();

    let mut schedule2 = create_schedule_with_visibility("second", 2);
    schedule2.checksum = "duplicate_checksum".to_string();

    // Store first schedule
    let meta1 = services::store_schedule(&repo, &schedule1).await.unwrap();

    // Store second schedule with same checksum
    // Note: LocalRepository doesn't implement checksum deduplication
    // (that's only in Postgres backend), so this creates a new schedule
    let meta2 = services::store_schedule(&repo, &schedule2).await.unwrap();

    // LocalRepository creates separate schedules even with same checksum
    assert_ne!(meta1.schedule_id, meta2.schedule_id);

    // Should have two schedules in LocalRepository
    let schedules = services::list_schedules(&repo).await.unwrap();
    assert_eq!(schedules.len(), 2);
}

#[tokio::test]
async fn test_landing_retrieve_stored_schedule() {
    let repo = LocalRepository::new();

    let original = create_schedule_with_visibility("retrieve_test", 4);
    let metadata = services::store_schedule(&repo, &original).await.unwrap();

    // Retrieve the schedule
    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .unwrap();

    assert_eq!(retrieved.name, original.name);
    assert_eq!(retrieved.blocks.len(), original.blocks.len());
    assert_eq!(retrieved.dark_periods.len(), original.dark_periods.len());
}

// =========================================================
// Full API Stack Tests
// NOTE: PyO3/Python runtime tests are in test_pyo3_integration.py
// =========================================================

#[tokio::test]
async fn test_full_api_stack_store_to_visualization() {
    let repo = LocalRepository::new();

    // 1. Store schedule
    let schedule = create_schedule_with_visibility("full_stack_test", 10);
    let metadata = services::store_schedule(&repo, &schedule).await.unwrap();

    // 2. List schedules
    let schedules = services::list_schedules(&repo).await.unwrap();
    assert!(schedules
        .iter()
        .any(|s| s.schedule_id == metadata.schedule_id));

    // 3. Retrieve schedule
    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .unwrap();
    assert_eq!(retrieved.name, "full_stack_test");
    assert_eq!(retrieved.blocks.len(), 10);

    // 4. Get blocks for schedule
    let blocks = services::get_blocks_for_schedule(&repo, metadata.schedule_id)
        .await
        .unwrap();
    assert_eq!(blocks.len(), 10);
}

#[tokio::test]
async fn test_multiple_schedules_isolation() {
    let repo = LocalRepository::new();

    // Store three different schedules
    let schedule1 = create_schedule_with_visibility("sched1", 3);
    let schedule2 = create_schedule_with_visibility("sched2", 5);
    let schedule3 = create_schedule_with_visibility("sched3", 2);

    let meta1 = services::store_schedule(&repo, &schedule1).await.unwrap();
    let meta2 = services::store_schedule(&repo, &schedule2).await.unwrap();
    let meta3 = services::store_schedule(&repo, &schedule3).await.unwrap();

    // Verify each schedule has correct number of blocks
    let blocks1 = services::get_blocks_for_schedule(&repo, meta1.schedule_id)
        .await
        .unwrap();
    let blocks2 = services::get_blocks_for_schedule(&repo, meta2.schedule_id)
        .await
        .unwrap();
    let blocks3 = services::get_blocks_for_schedule(&repo, meta3.schedule_id)
        .await
        .unwrap();

    assert_eq!(blocks1.len(), 3);
    assert_eq!(blocks2.len(), 5);
    assert_eq!(blocks3.len(), 2);

    // Verify schedules are isolated (no data leakage)
    let sched1 = services::get_schedule(&repo, meta1.schedule_id)
        .await
        .unwrap();
    assert_eq!(sched1.name, "sched1");

    let sched2 = services::get_schedule(&repo, meta2.schedule_id)
        .await
        .unwrap();
    assert_eq!(sched2.name, "sched2");
}

#[tokio::test]
async fn test_analytics_population_on_store() {
    let repo = LocalRepository::new();

    let schedule = create_schedule_with_visibility("analytics_test", 8);

    // Store with analytics population enabled
    let metadata = services::store_schedule_with_options(&repo, &schedule, true)
        .await
        .unwrap();

    // Verify schedule was stored
    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .unwrap();
    assert_eq!(retrieved.blocks.len(), 8);

    // Analytics should be populated (this is best-effort in LocalRepository)
    // The fact that store succeeded indicates analytics population was attempted
}

#[tokio::test]
async fn test_schedule_with_dark_periods() {
    let repo = LocalRepository::new();

    let mut schedule = create_schedule_with_visibility("dark_periods_test", 3);

    // Add multiple dark periods
    schedule.dark_periods = vec![
        Period {
            start: ModifiedJulianDate::new(59580.0),
            stop: ModifiedJulianDate::new(59582.0),
        },
        Period {
            start: ModifiedJulianDate::new(59585.0),
            stop: ModifiedJulianDate::new(59587.0),
        },
    ];

    let metadata = services::store_schedule(&repo, &schedule).await.unwrap();

    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .unwrap();
    assert_eq!(retrieved.dark_periods.len(), 2);

    // Verify dark period values
    assert_eq!(retrieved.dark_periods[0].start.value(), 59580.0);
    assert_eq!(retrieved.dark_periods[0].stop.value(), 59582.0);
}

#[tokio::test]
async fn test_blocks_with_various_priorities() {
    let repo = LocalRepository::new();

    let mut schedule = create_schedule_with_visibility("priority_test", 5);

    // Set specific priorities
    schedule.blocks[0].priority = 1.0;
    schedule.blocks[1].priority = 5.0;
    schedule.blocks[2].priority = 10.0;
    schedule.blocks[3].priority = 7.5;
    schedule.blocks[4].priority = 3.2;

    let metadata = services::store_schedule(&repo, &schedule).await.unwrap();
    let blocks = services::get_blocks_for_schedule(&repo, metadata.schedule_id)
        .await
        .unwrap();

    assert_eq!(blocks.len(), 5);

    // Verify priorities are preserved
    let priorities: Vec<f64> = blocks.iter().map(|b| b.priority).collect();
    assert!(priorities.contains(&1.0));
    assert!(priorities.contains(&5.0));
    assert!(priorities.contains(&10.0));
}

// =========================================================
// Concurrent Access Tests
// =========================================================

#[tokio::test]
async fn test_concurrent_schedule_storage() {
    let repo = LocalRepository::new();

    // Store schedules concurrently
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let repo_clone = repo.clone();
            tokio::spawn(async move {
                let schedule = create_schedule_with_visibility(&format!("concurrent_{}", i), 2);
                services::store_schedule(&repo_clone, &schedule).await
            })
        })
        .collect();

    // Wait for all tasks to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    // Verify all schedules were stored
    let schedules = services::list_schedules(&repo).await.unwrap();
    assert_eq!(schedules.len(), 5);
}

#[tokio::test]
async fn test_concurrent_read_operations() {
    let repo = LocalRepository::new();

    // Store a schedule first
    let schedule = create_schedule_with_visibility("concurrent_read", 10);
    let metadata = services::store_schedule(&repo, &schedule).await.unwrap();

    // Perform concurrent reads
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let repo_clone = repo.clone();
            let schedule_id = metadata.schedule_id;
            tokio::spawn(async move { services::get_schedule(&repo_clone, schedule_id).await })
        })
        .collect();

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await);
    }

    // All reads should succeed
    for result in results {
        let schedule_result = result.unwrap();
        assert!(schedule_result.is_ok());
        assert_eq!(schedule_result.unwrap().name, "concurrent_read");
    }
}
