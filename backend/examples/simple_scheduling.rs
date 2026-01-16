//! Example demonstrating STARS Core scheduling simulation
//!
//! This example shows how to use the STARS Core library to:
//! 1. Define an instrument and observation period
//! 2. Create scheduling blocks (observation tasks)
//! 3. Compute possible observation periods (prescheduler)
//! 4. Run the scheduling algorithm
//! 5. Analyze results
//!
//! To run this example:
//! ```bash
//! # First, build the native library
//! cd backend
//! cargo build -p stars-core-sys --features build-native --release
//!
//! # Then run the example
//! cargo run --example simple_scheduling --features stars-core --release
//! ```

#[cfg(feature = "stars-core")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::json;
    use tsi_rust::scheduler::stars::{
        compute_possible_periods, run_scheduler, Blocks, Context, SchedulerType,
        SchedulingParams,
    };

    println!("=== STARS Core Scheduling Simulation ===\n");

    // Step 1: Define instrument configuration
    println!("1. Creating instrument configuration...");
    let config = json!({
        "instrument": {
            "name": "Gran Telescopio Canarias (GTC)",
            "location": {
                "name": "Roque de los Muchachos Observatory",
                "latitude": 28.7606,
                "longitude": -17.8810,
                "altitude": 2396.0
            },
            "capabilities": {
                "min_elevation": 20.0,
                "max_elevation": 85.0
            }
        },
        "executionPeriod": {
            "begin": "2024-03-15T20:00:00",
            "end": "2024-03-16T06:00:00"
        },
        "observatory": "Roque de los Muchachos Observatory"
    });

    let ctx = Context::from_json(&config.to_string())?;
    let period = ctx.execution_period()?;
    println!(
        "   Execution period: {} to {} ({:.1} hours)\n",
        period.begin,
        period.end,
        period.duration_days * 24.0
    );

    // Step 2: Define scheduling blocks (observation tasks)
    println!("2. Creating observation tasks...");
    let blocks_config = json!({
        "schedulingBlocks": [
            {
                "stars::scheduling_blocks::ObservationTask": {
                    "name": "M51 (Whirlpool Galaxy)",
                    "priority": 1.0,
                    "duration": {
                        "days": 0,
                        "hours": 1,
                        "minutes": 30,
                        "seconds": 0
                    },
                    "targetCoordinates": {
                        "ra": "13:29:52.7",
                        "dec": "+47:11:43"
                    }
                }
            },
            {
                "stars::scheduling_blocks::ObservationTask": {
                    "name": "M81 (Bode's Galaxy)",
                    "priority": 0.9,
                    "duration": {
                        "days": 0,
                        "hours": 2,
                        "minutes": 0,
                        "seconds": 0
                    },
                    "targetCoordinates": {
                        "ra": "09:55:33.2",
                        "dec": "+69:03:55"
                    }
                }
            },
            {
                "stars::scheduling_blocks::ObservationTask": {
                    "name": "NGC 2237 (Rosette Nebula)",
                    "priority": 0.7,
                    "duration": {
                        "days": 0,
                        "hours": 1,
                        "minutes": 0,
                        "seconds": 0
                    },
                    "targetCoordinates": {
                        "ra": "06:32:18",
                        "dec": "+04:59:00"
                    }
                }
            },
            {
                "stars::scheduling_blocks::ObservationTask": {
                    "name": "NGC 6543 (Cat's Eye Nebula)",
                    "priority": 0.8,
                    "duration": {
                        "days": 0,
                        "hours": 0,
                        "minutes": 45,
                        "seconds": 0
                    },
                    "targetCoordinates": {
                        "ra": "17:58:33.4",
                        "dec": "+66:37:59"
                    }
                }
            },
            {
                "stars::scheduling_blocks::ObservationTask": {
                    "name": "M42 (Orion Nebula)",
                    "priority": 0.6,
                    "duration": {
                        "days": 0,
                        "hours": 1,
                        "minutes": 15,
                        "seconds": 0
                    },
                    "targetCoordinates": {
                        "ra": "05:35:17.3",
                        "dec": "-05:23:28"
                    }
                }
            }
        ]
    });

    let blocks = Blocks::from_json(&blocks_config.to_string())?;
    let block_count = blocks.len()?;
    println!("   Loaded {} observation tasks\n", block_count);

    // List all blocks
    println!("   Tasks:");
    for (i, block_result) in blocks.iter().enumerate() {
        let block_json = block_result?;
        let block: serde_json::Value = serde_json::from_str(&block_json)?;
        if let Some(task) = block.get("stars::scheduling_blocks::ObservationTask") {
            let name = task["name"].as_str().unwrap_or("Unknown");
            let priority = task["priority"].as_f64().unwrap_or(0.0);
            println!("     {}. {} (priority: {:.1})", i + 1, name, priority);
        }
    }
    println!();

    // Step 3: Run prescheduler to compute possible observation periods
    println!("3. Computing possible observation periods (prescheduler)...");
    let periods = compute_possible_periods(&ctx, &blocks)?;
    let periods_data = periods.to_vec()?;

    for period_info in &periods_data {
        println!(
            "   {} has {} possible period(s)",
            period_info.block_name,
            period_info.periods.len()
        );
    }
    println!();

    // Step 4: Run scheduling algorithm
    println!("4. Running scheduling algorithm...");
    let params = SchedulingParams {
        algorithm: SchedulerType::HybridAccumulative,
        max_iterations: 5000,
        time_limit_seconds: 30.0,
        seed: 12345,
    };

    println!("   Algorithm: {:?}", params.algorithm);
    println!("   Max iterations: {}", params.max_iterations);
    println!("   Time limit: {:.1}s", params.time_limit_seconds);
    println!();

    let schedule = run_scheduler(&ctx, &blocks, Some(&periods), params)?;

    // Step 5: Analyze results
    println!("5. Results:\n");
    let stats = schedule.stats()?;

    println!("   Summary:");
    println!("   --------");
    println!("   Total observation tasks: {}", stats.total_blocks);
    println!("   Scheduled: {}", stats.scheduled_count);
    println!("   Unscheduled: {}", stats.unscheduled_count);
    println!(
        "   Scheduling rate: {:.1}%",
        stats.scheduling_rate * 100.0
    );
    println!("   Fitness score: {:.4}", stats.fitness);
    println!();

    // Show scheduled observations
    let result = schedule.to_result()?;

    if !result.units.is_empty() {
        println!("   Scheduled Observations:");
        println!("   ----------------------");
        for unit in &result.units {
            println!(
                "   • {} ({} to {})",
                unit.task_name, unit.begin, unit.end
            );
        }
        println!();
    }

    if !result.unscheduled.is_empty() {
        println!("   Unscheduled Tasks:");
        println!("   -----------------");
        for block in &result.unscheduled {
            println!("   • {}", block.name);
        }
        println!();
    }

    println!("=== Simulation Complete ===");

    Ok(())
}

#[cfg(not(feature = "stars-core"))]
fn main() {
    eprintln!("This example requires the 'stars-core' feature to be enabled.");
    eprintln!("Build the native library first, then run:");
    eprintln!();
    eprintln!("  cargo run --example simple_scheduling --features stars-core");
    eprintln!();
    std::process::exit(1);
}
