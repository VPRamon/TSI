//! Integration tests for STARS Core scheduling
//!
//! These tests demonstrate how to use the STARS Core library for scheduling simulations.
//! They require the native library to be built.
//!
//! First-time build from bundled sources:
//!   cargo test --features build-native
//!
//! If you already have a system-installed `stars_ffi` (see `STARS_FFI_LIB_DIR`), you can use:
//!   cargo test --features stars-core

#![cfg(all(test, feature = "stars-core"))]

use serde_json::json;

#[test]
fn test_simple_scheduling_pipeline() {
    use crate::scheduler::stars::{
        compute_possible_periods, run_scheduler, Blocks, Context, SchedulerType,
        SchedulingParams,
    };

    // Create a minimal configuration with instrument and execution period
    let config_json = json!({
        "instrument": {
            "name": "TestInstrument",
            "location": {
                "latitude": 28.76,
                "longitude": -17.88,
                "altitude": 2400.0
            }
        },
        "executionPeriod": {
            "begin": "2024-01-15T00:00:00",
            "end": "2024-01-15T23:59:59"
        },
        "observatory": "Test Observatory"
    })
    .to_string();

    // Create scheduling blocks (observation tasks)
    // Note: RA/Dec are in degrees (RA: 0-360, Dec: -90 to +90)
    let blocks_json = json!({
        "schedulingBlocks": [
            {
                "stars::scheduling_blocks::ObservationTask": {
                    "name": "Target-1",
                    "priority": 1.0,
                    "duration": {
                        "hours": 1,
                        "minutes": 0,
                        "seconds": 0
                    },
                    "targetCoordinates": {
                        "ra": 157.5,   // 10h 30m = 157.5 degrees
                        "dec": 45.0    // +45 degrees
                    }
                }
            },
            {
                "stars::scheduling_blocks::ObservationTask": {
                    "name": "Target-2",
                    "priority": 0.8,
                    "duration": {
                        "hours": 2,
                        "minutes": 0,
                        "seconds": 0
                    },
                    "targetCoordinates": {
                        "ra": 215.0,   // 14h 20m = 215 degrees
                        "dec": -20.0   // -20 degrees
                    }
                }
            }
        ]
    })
    .to_string();

    // Step 1: Create context from configuration
    let ctx = Context::from_json(&config_json).expect("Failed to create context");

    // Verify we can get execution period
    let period = ctx.execution_period().expect("Failed to get execution period");
    // Note: The format depends on FFI implementation (may have Z suffix)
    assert!(period.begin.starts_with("2024-01-15"), "Begin should start with correct date");
    assert!(period.end.starts_with("2024-01-15"), "End should start with correct date");

    // Step 2: Load scheduling blocks
    let blocks = Blocks::from_json(&blocks_json).expect("Failed to load blocks");
    let block_count = blocks.len().expect("Failed to get block count");
    assert_eq!(block_count, 2, "Should have loaded 2 scheduling blocks");

    // Step 3: Compute possible observation periods (prescheduler)
    let periods =
        compute_possible_periods(&ctx, &blocks).expect("Failed to compute possible periods");

    // Verify periods can be exported
    let periods_json = periods.to_json().expect("Failed to export periods");
    assert!(!periods_json.is_empty(), "Periods JSON should not be empty");

    // Step 4: Run scheduling algorithm
    let params = SchedulingParams {
        algorithm: SchedulerType::Accumulative,
        max_iterations: 1000,
        time_limit_seconds: 10.0,
        seed: 42,
    };

    let schedule = run_scheduler(&ctx, &blocks, Some(&periods), params)
        .expect("Failed to run scheduler");

    // Step 5: Get results and verify
    let stats = schedule.stats().expect("Failed to get statistics");

    println!("Scheduling Results:");
    println!("  Total blocks: {}", stats.total_blocks);
    println!("  Scheduled: {}", stats.scheduled_count);
    println!("  Unscheduled: {}", stats.unscheduled_count);
    println!("  Scheduling rate: {:.1}%", stats.scheduling_rate * 100.0);
    println!("  Fitness: {:.4}", stats.fitness);

    assert_eq!(stats.total_blocks, 2);
    assert!(
        stats.scheduling_rate >= 0.0 && stats.scheduling_rate <= 1.0,
        "Scheduling rate should be between 0 and 1"
    );

    // Export schedule as JSON
    let schedule_json = schedule.to_json().expect("Failed to export schedule");
    assert!(!schedule_json.is_empty(), "Schedule JSON should not be empty");

    // Parse and verify structure
    let result = schedule.to_result().expect("Failed to get typed result");
    assert_eq!(result.scheduled_count + result.unscheduled_count, 2);
}

#[test]
fn test_full_pipeline_convenience() {
    use crate::scheduler::stars::{run_full_pipeline, SchedulerType, SchedulingParams};

    // Combined configuration and blocks in one JSON
    let input_json = json!({
        "instrument": {
            "name": "TestInstrument",
            "location": {
                "latitude": 28.76,
                "longitude": -17.88,
                "altitude": 2400.0
            }
        },
        "executionPeriod": {
            "begin": "2024-01-15T00:00:00",
            "end": "2024-01-15T23:59:59"
        },
        "schedulingBlocks": [
            {
                "stars::scheduling_blocks::ObservationTask": {
                    "name": "Target-1",
                    "priority": 1.0,
                    "duration": {
                        "days": 0,
                        "hours": 1,
                        "minutes": 0,
                        "seconds": 0
                    },
                    "targetCoordinates": {
                        "ra": "10:30:00",
                        "dec": "+45:00:00"
                    }
                }
            }
        ]
    })
    .to_string();

    let params = SchedulingParams {
        algorithm: SchedulerType::HybridAccumulative,
        ..Default::default()
    };

    // Run entire pipeline in one call
    let result_json = run_full_pipeline(&input_json, params).expect("Pipeline failed");

    println!("Pipeline result: {}", result_json);
    assert!(!result_json.is_empty());

    // Verify result structure
    let result: serde_json::Value =
        serde_json::from_str(&result_json).expect("Invalid JSON result");
    assert!(result.get("units").is_some());
    assert!(result.get("fitness").is_some());
    assert!(result.get("scheduled_count").is_some());
    assert!(result.get("unscheduled_count").is_some());
}

#[test]
fn test_blocks_iteration() {
    use crate::scheduler::stars::Blocks;

    let blocks_json = json!({
        "schedulingBlocks": [
            {
                "stars::scheduling_blocks::ObservationTask": {
                    "name": "Target-A",
                    "priority": 1.0,
                    "duration": { "days": 0, "hours": 1, "minutes": 0, "seconds": 0 },
                    "targetCoordinates": { "ra": "10:30:00", "dec": "+45:00:00" }
                }
            },
            {
                "stars::scheduling_blocks::ObservationTask": {
                    "name": "Target-B",
                    "priority": 0.5,
                    "duration": { "days": 0, "hours": 2, "minutes": 0, "seconds": 0 },
                    "targetCoordinates": { "ra": "14:20:00", "dec": "-20:00:00" }
                }
            },
            {
                "stars::scheduling_blocks::ObservationTask": {
                    "name": "Target-C",
                    "priority": 0.7,
                    "duration": { "days": 0, "hours": 1, "minutes": 30, "seconds": 0 },
                    "targetCoordinates": { "ra": "08:15:00", "dec": "+30:00:00" }
                }
            }
        ]
    })
    .to_string();

    let blocks = Blocks::from_json(&blocks_json).expect("Failed to load blocks");

    // Test direct access
    assert_eq!(blocks.len().unwrap(), 3);
    assert!(!blocks.is_empty().unwrap());

    let first = blocks.get(0).expect("Failed to get first block");
    assert!(first.contains("Target-A"));

    // Test iteration
    let mut count = 0;
    for (i, block_result) in blocks.iter().enumerate() {
        let block_json = block_result.expect("Failed to get block");
        println!("Block {}: {}", i, block_json);
        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn test_error_handling() {
    use crate::scheduler::stars::{Blocks, Context};

    // Invalid JSON
    let result = Context::from_json("not valid json");
    assert!(result.is_err());

    // Missing required fields
    let incomplete_json = json!({ "instrument": {} }).to_string();
    let result = Context::from_json(&incomplete_json);
    assert!(result.is_err());

    // Invalid blocks
    let bad_blocks = json!({ "schedulingBlocks": "not an array" }).to_string();
    let result = Blocks::from_json(&bad_blocks);
    assert!(result.is_err());
}

#[test]
fn test_scheduling_params_defaults() {
    use crate::scheduler::stars::{SchedulerType, SchedulingParams};

    let params = SchedulingParams::default();
    assert_eq!(params.algorithm, SchedulerType::Accumulative);
    assert_eq!(params.max_iterations, 0);
    assert_eq!(params.time_limit_seconds, 0.0);
    assert_eq!(params.seed, -1);
}

#[test]
fn test_version_info() {
    use crate::scheduler::stars::{core_version, ffi_version};

    let ffi_ver = ffi_version();
    let core_ver = core_version();

    println!("FFI version: {}", ffi_ver);
    println!("Core version: {}", core_ver);

    assert!(!ffi_ver.is_empty());
    assert!(!core_ver.is_empty());
}

/// Example demonstrating how to build scheduling configuration programmatically
#[test]
fn test_programmatic_config_building() {
    use crate::scheduler::stars::Context;

    // Build configuration step by step
    let config = json!({
        "instrument": {
            "name": "GTC",
            "location": {
                "latitude": 28.7606,
                "longitude": -17.8810,
                "altitude": 2396.0
            },
            "capabilities": {
                "min_elevation": 15.0,
                "max_elevation": 89.0
            }
        },
        "executionPeriod": {
            "begin": "2024-03-01T00:00:00",
            "end": "2024-03-31T23:59:59"
        },
        "observatory": "Roque de los Muchachos Observatory"
    });

    let ctx = Context::from_json(&config.to_string()).expect("Failed to create context");

    let period = ctx.execution_period().expect("Failed to get period");
    assert!(period.duration_days > 0.0);
    assert!(period.duration_days <= 31.0);

    println!("Execution period: {} days", period.duration_days);
}
