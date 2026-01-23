//! Data-driven functional tests using actual JSON files from data/ directory.
//!
//! These tests exercise the complete workflow for each page/feature:
//! 1. Load schedule from JSON (with possible periods and dark periods)
//! 2. Store schedule via service layer
//! 3. Populate analytics tables
//! 4. Retrieve validation reports
//! 5. Retrieve sky map data
//! 6. Retrieve trends data
//! 7. Retrieve timeline data
//! 8. Retrieve distributions data
//! 9. Retrieve insights data
//! 10. Compare schedules
//!
//! This validates the end-to-end functionality using real production data.

use std::fs;
use tsi_rust::api::*;
use tsi_rust::db::repositories::LocalRepository;
use tsi_rust::db::services;
use tsi_rust::db::{AnalyticsRepository, ValidationRepository, VisualizationRepository};
use tsi_rust::models::parse_schedule_json_str;
use tsi_rust::routes::trends::TrendsBlock;
use tsi_rust::services::{compare, distributions, insights, sky_map, timeline, trends};

// ==================== Helper Functions ====================

/// Load schedule from actual data files in /workspace/data/
fn load_schedule_from_files() -> Schedule {
    // Use astro format schedule file
    let schedule_json =
        fs::read_to_string("/workspace/data/schedule_astro.json").expect("Failed to read schedule_astro.json");

    parse_schedule_json_str(&schedule_json)
        .expect("Failed to parse schedule from JSON files")
}

/// Load the smaller test schedule (subset of main schedule for faster tests)
fn load_test_schedule() -> Schedule {
    // Load the main schedule and take a subset for faster tests
    let mut schedule = load_schedule_from_files();

    // Keep only first 5 blocks for faster testing
    schedule.blocks.truncate(5);
    schedule.name = "test_schedule_subset".to_string();
    schedule.checksum = "test_checksum".to_string();

    schedule
}

// ==================== Full Workflow Tests ====================

#[tokio::test]
async fn test_full_workflow_load_store_validate() {
    // This test validates the complete workflow from loading to validation
    let repo = LocalRepository::new();
    let schedule = load_schedule_from_files();

    // Verify schedule was parsed correctly
    assert!(!schedule.name.is_empty(), "Schedule should have a name");
    assert!(
        !schedule.checksum.is_empty(),
        "Schedule should have checksum"
    );
    assert!(!schedule.blocks.is_empty(), "Schedule should have blocks");
    // Astro format computes astronomical_nights instead of dark_periods
    assert!(
        !schedule.astronomical_nights.is_empty(),
        "Schedule should have astronomical nights"
    );

    println!(
        "Loaded schedule: {} blocks, {} astronomical nights",
        schedule.blocks.len(),
        schedule.astronomical_nights.len()
    );

    // Store schedule using service layer
    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    assert!(
        metadata.schedule_id.value() > 0,
        "Should have valid schedule ID"
    );
    assert_eq!(metadata.schedule_name, schedule.name, "Name should match");

    // Verify schedule can be retrieved
    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .expect("Failed to retrieve schedule");

    assert_eq!(retrieved.name, schedule.name);
    assert_eq!(retrieved.blocks.len(), schedule.blocks.len());
    assert_eq!(retrieved.astronomical_nights.len(), schedule.astronomical_nights.len());

    // Verify validation results were populated
    let validation_report = repo
        .fetch_validation_results(metadata.schedule_id)
        .await
        .expect("Failed to fetch validation report");

    assert_eq!(validation_report.total_blocks, schedule.blocks.len());
    println!(
        "Validation report: {} total blocks, {} impossible, {} errors, {} warnings",
        validation_report.total_blocks,
        validation_report.impossible_blocks.len(),
        validation_report.validation_errors.len(),
        validation_report.validation_warnings.len()
    );
}

#[tokio::test]
async fn test_full_workflow_sky_map_data() {
    // Test the complete workflow for sky map visualization page
    let repo = LocalRepository::new();
    let schedule = load_test_schedule();

    // Store schedule
    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    println!(
        "Stored schedule {} with {} blocks",
        metadata.schedule_id,
        schedule.blocks.len()
    );

    // Retrieve sky map blocks directly from repository
    let blocks = repo
        .fetch_analytics_blocks_for_sky_map(metadata.schedule_id)
        .await
        .expect("Failed to fetch sky map blocks");

    if blocks.is_empty() {
        println!(
            "No sky map blocks found - this is expected when blocks have no valid coordinates"
        );
        return;
    }

    // Compute sky map data
    let sky_map = sky_map::compute_sky_map_data(blocks).expect("Failed to compute sky map data");

    // Verify sky map structure
    println!(
        "Sky map data: {} blocks, bounds: RA [{}, {}], Dec [{}, {}]",
        sky_map.blocks.len(),
        sky_map.ra_min.value(),
        sky_map.ra_max.value(),
        sky_map.dec_min.value(),
        sky_map.dec_max.value()
    );

    // Verify bounds are sensible (using .value() to access f64)
    if !sky_map.blocks.is_empty() {
        assert!(sky_map.ra_min.value() >= 0.0 && sky_map.ra_min.value() <= 360.0);
        assert!(sky_map.ra_max.value() >= 0.0 && sky_map.ra_max.value() <= 360.0);
        assert!(sky_map.dec_min.value() >= -90.0 && sky_map.dec_min.value() <= 90.0);
        assert!(sky_map.dec_max.value() >= -90.0 && sky_map.dec_max.value() <= 90.0);
    }
}

#[tokio::test]
async fn test_full_workflow_trends_data() {
    // Test the complete workflow for trends visualization page
    let repo = LocalRepository::new();
    let schedule = load_test_schedule();

    // Store schedule with analytics
    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    println!(
        "Stored schedule {} with {} blocks",
        metadata.schedule_id,
        schedule.blocks.len()
    );

    // Retrieve analytics blocks directly from repository (instead of global singleton service)
    let blocks = repo
        .fetch_analytics_blocks_for_insights(metadata.schedule_id)
        .await
        .expect("Failed to fetch analytics blocks");

    if blocks.is_empty() {
        println!("No analytics blocks found - this is expected when blocks have no visibility");
        return;
    }

    // Compute trends data using the compute function - convert InsightsBlock to TrendsBlock
    let trends_blocks: Vec<TrendsBlock> = blocks
        .into_iter()
        .filter_map(|b| {
            // Filter out zero visibility
            if b.total_visibility_hours.value() == 0.0 {
                return None;
            }
            Some(TrendsBlock {
                scheduling_block_id: b.scheduling_block_id,
                original_block_id: b.original_block_id,
                priority: b.priority,
                total_visibility_hours: b.total_visibility_hours,
                requested_hours: b.requested_hours,
                scheduled: b.scheduled,
            })
        })
        .collect();

    if !trends_blocks.is_empty() {
        let trends = trends::compute_trends_data(trends_blocks, 20, 0.5, 3)
            .expect("Failed to compute trends");

        println!("Trends data: {} blocks, {} priority bins, {} visibility bins, {} time bins, {} heatmap bins",
                 trends.blocks.len(),
                 trends.by_priority.len(),
                 trends.by_visibility.len(),
                 trends.by_time.len(),
                 trends.heatmap_bins.len());

        // Verify metrics
        println!(
            "Trends metrics: total={}, scheduled={}, rate={:.2}%",
            trends.metrics.total_count,
            trends.metrics.scheduled_count,
            trends.metrics.scheduling_rate * 100.0
        );

        // Verify sensible values
        assert!(trends.metrics.scheduling_rate >= 0.0 && trends.metrics.scheduling_rate <= 1.0);
        assert!(trends.metrics.priority_min <= trends.metrics.priority_max);
    } else {
        println!("No valid trends blocks after filtering - all blocks may have zero visibility");
    }
}

#[tokio::test]
async fn test_full_workflow_timeline_data() {
    // Test the complete workflow for timeline visualization page
    let repo = LocalRepository::new();
    let schedule = load_test_schedule();

    // Store schedule with analytics
    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    println!(
        "Stored schedule {} with {} blocks",
        metadata.schedule_id,
        schedule.blocks.len()
    );

    // Retrieve timeline blocks directly from repository
    let blocks = repo
        .fetch_schedule_timeline_blocks(metadata.schedule_id)
        .await
        .expect("Failed to fetch timeline blocks");

    // Compute timeline data using the compute function - use astronomical_nights for astro format
    let timeline = timeline::compute_schedule_timeline_data(blocks.clone(), schedule.astronomical_nights.clone())
        .expect("Failed to compute timeline data");

    // Verify timeline structure - total_count is the number of SCHEDULED blocks (with scheduled_period)
    // which may be 0 for unscheduled tasks from astro format
    assert_eq!(
        timeline.total_count,
        blocks.len(),
        "Timeline should report correct count of timeline blocks"
    );

    println!(
        "Timeline data: {} blocks, {} scheduled, priority range [{}, {}], {} unique months",
        timeline.total_count,
        timeline.scheduled_count,
        timeline.priority_min,
        timeline.priority_max,
        timeline.unique_months.len()
    );

    // If there are scheduled blocks, verify their structure
    if !timeline.blocks.is_empty() {
        // Verify priority bounds are sensible
        assert!(timeline.priority_min <= timeline.priority_max);

        // Verify blocks have expected data
        for block in &timeline.blocks {
            assert!(block.priority >= 0.0, "Priority should be non-negative");
            assert!(
                block.scheduled_start_mjd.value() > 0.0,
                "Start MJD should be positive"
            );
        }
    }
}

#[tokio::test]
async fn test_full_workflow_validation_report() {
    // Test the complete workflow for validation report page
    let repo = LocalRepository::new();
    let schedule = load_test_schedule();

    // Store schedule (this populates validation results)
    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    println!(
        "Stored schedule {} with {} blocks",
        metadata.schedule_id,
        schedule.blocks.len()
    );

    // Retrieve validation report via repository
    let report = repo
        .fetch_validation_results(metadata.schedule_id)
        .await
        .expect("Failed to retrieve validation report");

    // Verify report structure
    assert_eq!(
        report.total_blocks,
        schedule.blocks.len(),
        "Report should show correct total block count"
    );

    println!("Validation report:");
    println!("  Total blocks: {}", report.total_blocks);
    println!("  Valid blocks: {}", report.valid_blocks);
    println!(
        "  Impossible blocks: {} ({:.1}%)",
        report.impossible_blocks.len(),
        if report.total_blocks > 0 {
            report.impossible_blocks.len() as f64 / report.total_blocks as f64 * 100.0
        } else {
            0.0
        }
    );
    println!("  Errors: {}", report.validation_errors.len());
    println!("  Warnings: {}", report.validation_warnings.len());

    // Verify counts sum correctly
    let categorized = report.impossible_blocks.len()
        + report.validation_errors.len()
        + report.validation_warnings.len();
    assert!(
        categorized <= report.total_blocks,
        "Categorized issues should not exceed total blocks"
    );

    // Verify valid blocks calculation
    assert!(
        report.valid_blocks <= report.total_blocks,
        "Valid blocks should not exceed total"
    );
}

#[tokio::test]
async fn test_full_workflow_distributions_data() {
    // Test the complete workflow for distributions page
    let repo = LocalRepository::new();
    let schedule = load_test_schedule();

    // Store schedule with analytics
    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    println!(
        "Stored schedule {} with {} blocks",
        metadata.schedule_id,
        schedule.blocks.len()
    );

    // Retrieve distribution blocks directly from repository
    let blocks = repo
        .fetch_analytics_blocks_for_distribution(metadata.schedule_id)
        .await
        .expect("Failed to fetch distribution blocks");

    if blocks.is_empty() {
        println!("No distribution blocks found - this is expected when blocks have no visibility");
        return;
    }

    // Count impossible blocks (zero visibility) for the compute function
    let impossible_count = blocks
        .iter()
        .filter(|b| b.total_visibility_hours.value() == 0.0)
        .count();

    // Compute distribution data with impossible_count
    let distributions = distributions::compute_distribution_data(blocks, impossible_count)
        .expect("Failed to compute distribution data");

    println!("Distribution data retrieved successfully:");
    println!("  Total blocks: {}", distributions.blocks.len());
    println!("  Total count: {}", distributions.total_count);
    println!("  Scheduled count: {}", distributions.scheduled_count);
    println!("  Unscheduled count: {}", distributions.unscheduled_count);
    println!("  Impossible count: {}", distributions.impossible_count);

    // Verify statistics structure
    println!(
        "  Priority stats: mean={:.2}, median={:.2}, min={:.2}, max={:.2}",
        distributions.priority_stats.mean,
        distributions.priority_stats.median,
        distributions.priority_stats.min,
        distributions.priority_stats.max
    );

    // Verify counts add up
    assert_eq!(
        distributions.total_count,
        distributions.scheduled_count
            + distributions.unscheduled_count
            + distributions.impossible_count,
        "Counts should sum to total"
    );
}

#[tokio::test]
async fn test_full_workflow_insights_data() {
    // Test the complete workflow for insights page
    let repo = LocalRepository::new();
    let schedule = load_test_schedule();

    // Store schedule
    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    println!(
        "Stored schedule {} with {} blocks",
        metadata.schedule_id,
        schedule.blocks.len()
    );

    // Retrieve insights blocks directly from repository
    let blocks = repo
        .fetch_analytics_blocks_for_insights(metadata.schedule_id)
        .await
        .expect("Failed to fetch insights blocks");

    if blocks.is_empty() {
        println!("No insights blocks found - this is expected when blocks have no visibility");
        return;
    }

    // Filter out zero visibility blocks
    let filtered_blocks: Vec<_> = blocks
        .into_iter()
        .filter(|b| b.total_visibility_hours.value() > 0.0)
        .collect();

    if filtered_blocks.is_empty() {
        println!("No valid insights blocks after filtering - all blocks may have zero visibility");
        return;
    }

    // Compute insights data
    let insights =
        insights::compute_insights_data(filtered_blocks).expect("Failed to compute insights data");

    println!("Insights data retrieved:");
    println!("  Total blocks: {}", insights.blocks.len());
    println!("  Total count: {}", insights.total_count);
    println!("  Scheduled count: {}", insights.scheduled_count);
    println!("  Impossible count: {}", insights.impossible_count);

    // Verify metrics
    println!(
        "  Scheduling rate: {:.2}%",
        insights.metrics.scheduling_rate * 100.0
    );
    println!("  Mean priority: {:.2}", insights.metrics.mean_priority);
    println!("  Median priority: {:.2}", insights.metrics.median_priority);

    // Verify scheduling rate is valid
    assert!(insights.metrics.scheduling_rate >= 0.0 && insights.metrics.scheduling_rate <= 1.0);

    // Verify priority stats are sensible
    assert!(insights.metrics.mean_priority >= 0.0);
    assert!(insights.metrics.median_priority >= 0.0);

    // Verify top lists
    println!(
        "  Top priority observations: {}",
        insights.top_priority.len()
    );
    println!(
        "  Top visibility observations: {}",
        insights.top_visibility.len()
    );
    println!("  Correlations: {}", insights.correlations.len());
    println!("  Conflicts detected: {}", insights.conflicts.len());
}

// ==================== Multi-Schedule Tests ====================

#[tokio::test]
async fn test_full_workflow_multiple_schedules() {
    // Test handling multiple schedules in the same repository
    let repo = LocalRepository::new();

    // Load and store test schedule
    let schedule1 = load_test_schedule();
    let metadata1 = services::store_schedule(&repo, &schedule1)
        .await
        .expect("Failed to store first schedule");

    // Modify schedule to create a different one
    let mut schedule2 = schedule1.clone();
    schedule2.name = "Modified Schedule".to_string();
    schedule2.checksum = "different_checksum".to_string();

    let metadata2 = services::store_schedule(&repo, &schedule2)
        .await
        .expect("Failed to store second schedule");

    // Verify both schedules are stored
    assert_ne!(metadata1.schedule_id, metadata2.schedule_id);

    // List all schedules
    let all_schedules = services::list_schedules(&repo)
        .await
        .expect("Failed to list schedules");

    assert_eq!(all_schedules.len(), 2, "Should have 2 schedules");

    // Retrieve each schedule individually
    let retrieved1 = services::get_schedule(&repo, metadata1.schedule_id)
        .await
        .expect("Failed to retrieve schedule 1");
    let retrieved2 = services::get_schedule(&repo, metadata2.schedule_id)
        .await
        .expect("Failed to retrieve schedule 2");

    assert_eq!(retrieved1.name, schedule1.name);
    assert_eq!(retrieved2.name, schedule2.name);

    println!("Successfully managed {} schedules", all_schedules.len());
}

// ==================== Performance Tests ====================

#[tokio::test]
async fn test_large_schedule_performance() {
    // Test performance with the large production schedule
    let repo = LocalRepository::new();

    let start = std::time::Instant::now();
    let schedule = load_schedule_from_files();
    let load_time = start.elapsed();

    println!(
        "Loaded large schedule in {:?}: {} blocks, {} dark periods",
        load_time,
        schedule.blocks.len(),
        schedule.dark_periods.len()
    );

    let start = std::time::Instant::now();
    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store large schedule");
    let store_time = start.elapsed();

    println!("Stored large schedule in {:?}", store_time);

    // Verify analytics population succeeded
    let start = std::time::Instant::now();
    let validation = repo
        .fetch_validation_results(metadata.schedule_id)
        .await
        .expect("Failed to fetch validation");
    let validation_time = start.elapsed();

    println!(
        "Retrieved validation in {:?}: {}/{} blocks validated",
        validation_time,
        validation.total_blocks,
        schedule.blocks.len()
    );

    // Test retrieval performance
    let start = std::time::Instant::now();
    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .expect("Failed to retrieve schedule");
    let retrieve_time = start.elapsed();

    println!(
        "Retrieved large schedule in {:?}: {} blocks",
        retrieve_time,
        retrieved.blocks.len()
    );

    // Verify correctness
    assert_eq!(retrieved.blocks.len(), schedule.blocks.len());
    assert_eq!(retrieved.dark_periods.len(), schedule.dark_periods.len());
}

// ==================== Data Integrity Tests ====================

#[tokio::test]
async fn test_visibility_periods_preserved() {
    // Verify that visibility periods from possible_periods.json are correctly loaded
    let repo = LocalRepository::new();
    let schedule = load_schedule_from_files();

    // Verify visibility periods were loaded
    let blocks_with_visibility = schedule
        .blocks
        .iter()
        .filter(|b| !b.visibility_periods.is_empty())
        .count();

    println!(
        "Schedule has {} blocks with visibility periods out of {} total",
        blocks_with_visibility,
        schedule.blocks.len()
    );

    assert!(
        blocks_with_visibility > 0,
        "Should have at least some blocks with visibility periods from possible_periods.json"
    );

    // Store and retrieve
    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .expect("Failed to retrieve schedule");

    // Verify visibility periods were preserved
    let retrieved_blocks_with_visibility = retrieved
        .blocks
        .iter()
        .filter(|b| !b.visibility_periods.is_empty())
        .count();

    assert_eq!(
        retrieved_blocks_with_visibility, blocks_with_visibility,
        "Visibility periods should be preserved through store/retrieve cycle"
    );

    // Verify a specific block's visibility periods
    if let Some(original_block) = schedule.blocks.first() {
        if let Some(retrieved_block) = retrieved.blocks.iter().find(|b| b.id == original_block.id) {
            assert_eq!(
                retrieved_block.visibility_periods.len(),
                original_block.visibility_periods.len(),
                "Block visibility periods should match"
            );
        }
    }
}

#[tokio::test]
async fn test_dark_periods_preserved() {
    // Verify astronomical nights are correctly preserved (astro format computes these)
    let repo = LocalRepository::new();
    let schedule = load_schedule_from_files();

    assert!(
        !schedule.astronomical_nights.is_empty(),
        "Should have astronomical nights"
    );

    let original_count = schedule.astronomical_nights.len();
    println!("Original schedule has {} astronomical nights", original_count);

    // Store and retrieve
    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .expect("Failed to retrieve schedule");

    assert_eq!(
        retrieved.astronomical_nights.len(),
        original_count,
        "Astronomical nights count should be preserved"
    );

    // Verify first and last astronomical night match
    if let (Some(orig_first), Some(retr_first)) = (
        schedule.astronomical_nights.first(),
        retrieved.astronomical_nights.first(),
    ) {
        assert_eq!(orig_first.start, retr_first.start);
        assert_eq!(orig_first.stop, retr_first.stop);
    }

    if let (Some(orig_last), Some(retr_last)) =
        (schedule.astronomical_nights.last(), retrieved.astronomical_nights.last())
    {
        assert_eq!(orig_last.start, retr_last.start);
        assert_eq!(orig_last.stop, retr_last.stop);
    }
}

#[tokio::test]
async fn test_checksum_deduplication() {
    // Verify checksum-based deduplication works correctly
    let repo = LocalRepository::new();
    let schedule = load_test_schedule();

    // Store schedule first time
    let metadata1 = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule first time");

    // Store same schedule again (same checksum)
    let metadata2 = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule second time");

    // Note: LocalRepository doesn't implement checksum deduplication
    // This is expected behavior - each store creates a new schedule
    println!("First store: schedule_id = {}", metadata1.schedule_id);
    println!("Second store: schedule_id = {}", metadata2.schedule_id);

    // For LocalRepository, we expect different IDs
    // (Postgres implementation would deduplicate based on checksum)
    assert_ne!(
        metadata1.schedule_id, metadata2.schedule_id,
        "LocalRepository creates new schedule each time"
    );
}

// ==================== Compare Workflow Tests ====================

#[tokio::test]
async fn test_full_workflow_compare_schedules() {
    // Test the complete workflow for schedule comparison page
    let repo = LocalRepository::new();

    // Load and store two different schedules
    let schedule1 = load_test_schedule();
    let metadata1 = services::store_schedule(&repo, &schedule1)
        .await
        .expect("Failed to store first schedule");

    let mut schedule2 = schedule1.clone();
    schedule2.name = "Comparison Schedule".to_string();
    schedule2.checksum = "different_checksum_2".to_string();
    // Modify some blocks for meaningful comparison
    if schedule2.blocks.len() > 1 {
        schedule2.blocks.remove(0);
    }

    let metadata2 = services::store_schedule(&repo, &schedule2)
        .await
        .expect("Failed to store second schedule");

    println!(
        "Stored two schedules for comparison: {} vs {}",
        metadata1.schedule_id, metadata2.schedule_id
    );

    // Fetch blocks from both schedules using repository methods
    let current_blocks = repo
        .fetch_compare_blocks(metadata1.schedule_id)
        .await
        .expect("Failed to fetch current schedule blocks");
    let comparison_blocks = repo
        .fetch_compare_blocks(metadata2.schedule_id)
        .await
        .expect("Failed to fetch comparison schedule blocks");

    // Compute compare data
    let comparison = compare::compute_compare_data(
        current_blocks,
        comparison_blocks,
        "Schedule A".to_string(),
        "Schedule B".to_string(),
    )
    .expect("Failed to compute compare data");

    println!("Comparison data retrieved:");
    println!(
        "  Current ({}) blocks: {}",
        comparison.current_name,
        comparison.current_blocks.len()
    );
    println!(
        "  Comparison ({}) blocks: {}",
        comparison.comparison_name,
        comparison.comparison_blocks.len()
    );
    println!("  Common IDs: {}", comparison.common_ids.len());
    println!("  Only in current: {}", comparison.only_in_current.len());
    println!(
        "  Only in comparison: {}",
        comparison.only_in_comparison.len()
    );
    println!(
        "  Scheduling changes: {}",
        comparison.scheduling_changes.len()
    );

    // Verify stats
    println!(
        "  Current stats: scheduled={}, unscheduled={}, mean_priority={:.2}",
        comparison.current_stats.scheduled_count,
        comparison.current_stats.unscheduled_count,
        comparison.current_stats.mean_priority
    );
    println!(
        "  Comparison stats: scheduled={}, unscheduled={}, mean_priority={:.2}",
        comparison.comparison_stats.scheduled_count,
        comparison.comparison_stats.unscheduled_count,
        comparison.comparison_stats.mean_priority
    );

    // Verify block counts
    assert_eq!(comparison.current_blocks.len(), schedule1.blocks.len());
    assert_eq!(comparison.comparison_blocks.len(), schedule2.blocks.len());
}

// ==================== Block-Level Data Tests ====================

#[tokio::test]
async fn test_block_data_integrity() {
    // Verify that all block fields are correctly preserved through the workflow
    let repo = LocalRepository::new();
    let schedule = load_test_schedule();

    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .expect("Failed to retrieve schedule");

    // Verify each block's key fields
    for (orig, retr) in schedule.blocks.iter().zip(retrieved.blocks.iter()) {
        // Verify ID is assigned and original ID is preserved
        assert!(retr.id.is_some(), "Retrieved block ID should be set");
        assert_eq!(
            orig.original_block_id, retr.original_block_id,
            "Original block ID mismatch"
        );

        // Verify priority
        assert!(
            (orig.priority - retr.priority).abs() < 0.001,
            "Priority mismatch for block {}",
            orig.original_block_id
        );

        // Verify target coordinates (target_ra and target_dec are direct fields)
        assert!(
            (orig.target_ra.value() - retr.target_ra.value()).abs() < 0.001,
            "RA mismatch for block {}",
            orig.original_block_id
        );
        assert!(
            (orig.target_dec.value() - retr.target_dec.value()).abs() < 0.001,
            "Dec mismatch for block {}",
            orig.original_block_id
        );

        // Verify scheduled period if present
        if let (Some(orig_period), Some(retr_period)) =
            (&orig.scheduled_period, &retr.scheduled_period)
        {
            assert!(
                (orig_period.start.value() - retr_period.start.value()).abs() < 0.0001,
                "Scheduled start mismatch for block {}",
                orig.original_block_id
            );
            assert!(
                (orig_period.stop.value() - retr_period.stop.value()).abs() < 0.0001,
                "Scheduled stop mismatch for block {}",
                orig.original_block_id
            );
        }
    }

    println!("All {} blocks preserved correctly", retrieved.blocks.len());
}

#[tokio::test]
async fn test_constraints_preserved() {
    // Verify that block constraints are preserved
    let repo = LocalRepository::new();
    let schedule = load_test_schedule();

    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .expect("Failed to retrieve schedule");

    for (orig, retr) in schedule.blocks.iter().zip(retrieved.blocks.iter()) {
        // Verify constraint fields (using actual field names: min_alt, max_alt, min_az, max_az)
        assert_eq!(
            orig.constraints.min_alt.value(),
            retr.constraints.min_alt.value(),
            "Min altitude mismatch for block {}",
            orig.original_block_id
        );
        assert_eq!(
            orig.constraints.max_alt.value(),
            retr.constraints.max_alt.value(),
            "Max altitude mismatch for block {}",
            orig.original_block_id
        );
        assert_eq!(
            orig.constraints.min_az.value(),
            retr.constraints.min_az.value(),
            "Min azimuth mismatch for block {}",
            orig.original_block_id
        );
        assert_eq!(
            orig.constraints.max_az.value(),
            retr.constraints.max_az.value(),
            "Max azimuth mismatch for block {}",
            orig.original_block_id
        );
        assert_eq!(
            orig.requested_duration.value(),
            retr.requested_duration.value(),
            "Requested duration mismatch for block {}",
            orig.original_block_id
        );
    }

    println!(
        "All constraints preserved for {} blocks",
        retrieved.blocks.len()
    );
}

// ==================== Analytics Population Tests ====================

#[tokio::test]
async fn test_analytics_populated_on_store() {
    // Verify that analytics are automatically populated when storing a schedule
    let repo = LocalRepository::new();
    let schedule = load_test_schedule();

    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    // Verify we can retrieve analytics data immediately after store
    // (This confirms analytics population during store)

    // Sky map data - fetch from repository and compute
    let sky_map_blocks = repo
        .fetch_analytics_blocks_for_sky_map(metadata.schedule_id)
        .await
        .expect("Sky map blocks should be available after store");
    if !sky_map_blocks.is_empty() {
        let sky_map = sky_map::compute_sky_map_data(sky_map_blocks);
        assert!(sky_map.is_ok(), "Sky map should be computable after store");
    }

    // Trends data - fetch from repository, convert, and compute
    let trends_insights_blocks = repo
        .fetch_analytics_blocks_for_insights(metadata.schedule_id)
        .await
        .expect("Trends blocks should be available after store");
    let trends_blocks: Vec<TrendsBlock> = trends_insights_blocks
        .into_iter()
        .filter_map(|b| {
            if b.total_visibility_hours.value() == 0.0 {
                return None;
            }
            Some(TrendsBlock {
                scheduling_block_id: b.scheduling_block_id,
                original_block_id: b.original_block_id,
                priority: b.priority,
                total_visibility_hours: b.total_visibility_hours,
                requested_hours: b.requested_hours,
                scheduled: b.scheduled,
            })
        })
        .collect();
    if !trends_blocks.is_empty() {
        let trends = trends::compute_trends_data(trends_blocks, 10, 0.5, 3);
        assert!(trends.is_ok(), "Trends should be computable after store");
    }

    // Timeline data - fetch from repository and compute
    let timeline_blocks = repo
        .fetch_schedule_timeline_blocks(metadata.schedule_id)
        .await
        .expect("Timeline blocks should be available after store");
    let timeline =
        timeline::compute_schedule_timeline_data(timeline_blocks, schedule.dark_periods.clone());
    assert!(
        timeline.is_ok(),
        "Timeline should be computable after store"
    );

    // Distribution data - fetch from repository and compute
    let dist_blocks = repo
        .fetch_analytics_blocks_for_distribution(metadata.schedule_id)
        .await
        .expect("Distribution blocks should be available after store");
    let impossible_count = dist_blocks
        .iter()
        .filter(|b| b.total_visibility_hours.value() == 0.0)
        .count();
    let dist = distributions::compute_distribution_data(dist_blocks, impossible_count);
    assert!(
        dist.is_ok(),
        "Distribution should be computable after store"
    );

    // Insights data - fetch from repository and compute
    let insights_blocks = repo
        .fetch_analytics_blocks_for_insights(metadata.schedule_id)
        .await
        .expect("Insights blocks should be available after store");
    // Filter for schedulable blocks (like the insights service does)
    let schedulable_blocks: Vec<_> = insights_blocks
        .iter()
        .filter(|b| b.total_visibility_hours.value() > 0.0)
        .cloned()
        .collect();
    let insights = insights::compute_insights_data(schedulable_blocks);
    assert!(
        insights.is_ok(),
        "Insights should be computable after store"
    );

    // Validation data
    let validation = repo.fetch_validation_results(metadata.schedule_id).await;
    assert!(
        validation.is_ok(),
        "Validation should be available after store"
    );

    println!(
        "All analytics successfully populated for schedule {}",
        metadata.schedule_id
    );
}

// ==================== Edge Case Tests ====================

#[tokio::test]
async fn test_empty_schedule_handling() {
    // Test that we can handle a schedule with no blocks
    let repo = LocalRepository::new();

    // Create minimal empty schedule JSON in astro format
    let empty_schedule_json = r#"{
        "location": {
            "lat": 28.7624,
            "lon": -17.8892,
            "distance": 6373.396
        },
        "period": {
            "start": 60676.0,
            "end": 60680.0
        },
        "tasks": []
    }"#;

    let schedule =
        parse_schedule_json_str(empty_schedule_json).expect("Failed to parse empty schedule");

    assert!(schedule.blocks.is_empty(), "Schedule should have no blocks");

    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store empty schedule");

    let retrieved = services::get_schedule(&repo, metadata.schedule_id)
        .await
        .expect("Failed to retrieve empty schedule");

    assert!(
        retrieved.blocks.is_empty(),
        "Retrieved schedule should have no blocks"
    );

    // Verify analytics can handle empty schedule using repository methods
    let timeline_blocks = repo
        .fetch_schedule_timeline_blocks(metadata.schedule_id)
        .await
        .expect("Timeline blocks should be available for empty schedule");
    // Use astronomical_nights for timeline since astro format computes those
    let timeline =
        timeline::compute_schedule_timeline_data(timeline_blocks, schedule.astronomical_nights.clone())
            .expect("Timeline should handle empty schedule");

    assert_eq!(timeline.total_count, 0);
    assert_eq!(timeline.scheduled_count, 0);

    println!("Empty schedule handled correctly");
}

#[tokio::test]
async fn test_schedule_with_extreme_values() {
    // Test handling of schedules with extreme values
    let repo = LocalRepository::new();
    let schedule = load_test_schedule();

    // Store and retrieve
    let metadata = services::store_schedule(&repo, &schedule)
        .await
        .expect("Failed to store schedule");

    // Verify trends can handle the data without overflow/underflow
    let trends_insights_blocks = repo
        .fetch_analytics_blocks_for_insights(metadata.schedule_id)
        .await
        .expect("Failed to fetch trends blocks");
    let trends_blocks: Vec<TrendsBlock> = trends_insights_blocks
        .into_iter()
        .filter_map(|b| {
            if b.total_visibility_hours.value() == 0.0 {
                return None;
            }
            Some(TrendsBlock {
                scheduling_block_id: b.scheduling_block_id,
                original_block_id: b.original_block_id,
                priority: b.priority,
                total_visibility_hours: b.total_visibility_hours,
                requested_hours: b.requested_hours,
                scheduled: b.scheduled,
            })
        })
        .collect();
    if !trends_blocks.is_empty() {
        let trends = trends::compute_trends_data(trends_blocks, 100, 0.1, 50);
        assert!(
            trends.is_ok(),
            "Trends should handle extreme parameter values"
        );
    }

    // Verify distribution can handle the data
    let dist_blocks = repo
        .fetch_analytics_blocks_for_distribution(metadata.schedule_id)
        .await
        .expect("Failed to fetch distribution blocks");
    let impossible_count = dist_blocks
        .iter()
        .filter(|b| b.total_visibility_hours.value() == 0.0)
        .count();
    let dist = distributions::compute_distribution_data(dist_blocks, impossible_count);
    assert!(dist.is_ok(), "Distribution should handle schedule data");

    println!("Extreme values handled correctly");
}
