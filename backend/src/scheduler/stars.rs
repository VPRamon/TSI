//! Safe Rust API for STARS Core scheduling library
//!
//! This module re-exports and extends the `stars-core` crate for use within
//! the TSI backend.
//!
//! # Example
//!
//! ```rust,ignore
//! use tsi_rust::scheduler::stars::{
//!     Context, Blocks, Schedule, SchedulingParams, SchedulerType,
//!     compute_possible_periods, run_scheduler,
//! };
//!
//! // Load from schedule file
//! let ctx = Context::from_file("data/schedule.json")?;
//! let blocks = Blocks::from_file("data/schedule.json")?;
//!
//! // Compute visibility windows
//! let periods = compute_possible_periods(&ctx, &blocks)?;
//!
//! // Run scheduling
//! let params = SchedulingParams {
//!     algorithm: SchedulerType::HybridAccumulative,
//!     ..Default::default()
//! };
//! let schedule = run_scheduler(&ctx, &blocks, Some(&periods), params)?;
//!
//! // Get results
//! let stats = schedule.stats()?;
//! println!("Scheduled: {}/{}", stats.scheduled_count, stats.total_blocks);
//! ```

// Re-export everything from stars-core
pub use stars_core::*;

use crate::models::Schedule as TsiSchedule;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Extended scheduling parameters with TSI-specific options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsiSchedulingParams {
    /// Base STARS Core parameters
    #[serde(flatten)]
    pub base: SchedulingParams,

    /// Whether to store results in the database
    #[serde(default)]
    pub store_results: bool,

    /// Optional schedule name for storage
    #[serde(default)]
    pub schedule_name: Option<String>,
}

impl Default for TsiSchedulingParams {
    fn default() -> Self {
        Self {
            base: SchedulingParams::default(),
            store_results: false,
            schedule_name: None,
        }
    }
}

impl From<TsiSchedulingParams> for SchedulingParams {
    fn from(p: TsiSchedulingParams) -> Self {
        p.base
    }
}

/// Run scheduling from a JSON configuration string
///
/// This is a convenience function that:
/// 1. Parses the configuration and blocks from JSON
/// 2. Runs the prescheduler to compute possible periods
/// 3. Runs the scheduling algorithm
/// 4. Returns the results as JSON
///
/// # Arguments
///
/// * `config_json` - JSON string containing instrument config, execution period, and blocks
/// * `params` - Scheduling parameters
///
/// # Returns
///
/// JSON string with scheduling results
pub fn schedule_from_json(config_json: &str, params: SchedulingParams) -> Result<String> {
    run_full_pipeline(config_json, params).map_err(|e| anyhow::anyhow!("Scheduling failed: {}", e))
}

/// Run scheduling from a file path
///
/// # Arguments
///
/// * `file_path` - Path to JSON file with configuration and blocks
/// * `params` - Scheduling parameters
///
/// # Returns
///
/// JSON string with scheduling results
pub fn schedule_from_file(file_path: &str, params: SchedulingParams) -> Result<String> {
    run_pipeline_from_file(file_path, params)
        .map_err(|e| anyhow::anyhow!("Scheduling failed: {}", e))
}

/// Run scheduling with full control over the process
///
/// This function provides fine-grained control over the scheduling pipeline:
/// 1. Create context from configuration
/// 2. Load blocks
/// 3. Optionally compute possible periods
/// 4. Run scheduler
/// 5. Return typed results
///
/// # Arguments
///
/// * `config_json` - JSON string with instrument and execution period
/// * `blocks_json` - JSON string with scheduling blocks
/// * `params` - Scheduling parameters
///
/// # Returns
///
/// Schedule statistics
pub fn schedule_with_control(
    config_json: &str,
    blocks_json: &str,
    params: SchedulingParams,
) -> Result<ScheduleStats> {
    let ctx =
        Context::from_json(config_json).map_err(|e| anyhow::anyhow!("Invalid config: {}", e))?;

    let blocks =
        Blocks::from_json(blocks_json).map_err(|e| anyhow::anyhow!("Invalid blocks: {}", e))?;

    // Compute possible periods
    let periods = compute_possible_periods(&ctx, &blocks)
        .map_err(|e| anyhow::anyhow!("Prescheduler failed: {}", e))?;

    // Run scheduler
    let schedule = run_scheduler(&ctx, &blocks, Some(&periods), params)
        .map_err(|e| anyhow::anyhow!("Scheduler failed: {}", e))?;

    schedule
        .stats()
        .map_err(|e| anyhow::anyhow!("Failed to get stats: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = TsiSchedulingParams::default();
        assert!(!params.store_results);
        assert!(params.schedule_name.is_none());
        assert_eq!(params.base.algorithm, SchedulerType::Accumulative);
    }

    #[test]
    fn test_versions() {
        // These will fail if the library isn't linked, but that's expected in unit tests
        // The important thing is the API compiles correctly
        let _ = ffi_version();
        let _ = core_version();
    }
}
