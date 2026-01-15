//! Type definitions for stars-core

use serde::{Deserialize, Serialize};
use stars_core_sys as ffi;

/// Scheduling algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerType {
    /// Accumulative scheduling algorithm
    #[default]
    Accumulative,
    /// Hybrid accumulative scheduling algorithm
    HybridAccumulative,
}

impl From<SchedulerType> for ffi::StarsSchedulerType {
    fn from(t: SchedulerType) -> Self {
        match t {
            SchedulerType::Accumulative => ffi::StarsSchedulerType::STARS_SCHEDULER_ACCUMULATIVE,
            SchedulerType::HybridAccumulative => {
                ffi::StarsSchedulerType::STARS_SCHEDULER_HYBRID_ACCUMULATIVE
            }
        }
    }
}

impl From<ffi::StarsSchedulerType> for SchedulerType {
    fn from(t: ffi::StarsSchedulerType) -> Self {
        match t {
            ffi::StarsSchedulerType::STARS_SCHEDULER_ACCUMULATIVE => SchedulerType::Accumulative,
            ffi::StarsSchedulerType::STARS_SCHEDULER_HYBRID_ACCUMULATIVE => {
                SchedulerType::HybridAccumulative
            }
        }
    }
}

/// Parameters for the scheduling algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingParams {
    /// Algorithm to use
    #[serde(default)]
    pub algorithm: SchedulerType,

    /// Maximum number of iterations (0 = default)
    #[serde(default)]
    pub max_iterations: u32,

    /// Time limit in seconds (0 = no limit)
    #[serde(default)]
    pub time_limit_seconds: f64,

    /// Random seed (-1 = random)
    #[serde(default = "default_seed")]
    pub seed: i32,
}

fn default_seed() -> i32 {
    -1
}

impl Default for SchedulingParams {
    fn default() -> Self {
        Self {
            algorithm: SchedulerType::default(),
            max_iterations: 0,
            time_limit_seconds: 0.0,
            seed: -1,
        }
    }
}

impl From<SchedulingParams> for ffi::StarsSchedulingParams {
    fn from(p: SchedulingParams) -> Self {
        ffi::StarsSchedulingParams {
            algorithm: p.algorithm.into(),
            max_iterations: p.max_iterations,
            time_limit_seconds: p.time_limit_seconds,
            seed: p.seed,
        }
    }
}

impl From<ffi::StarsSchedulingParams> for SchedulingParams {
    fn from(p: ffi::StarsSchedulingParams) -> Self {
        Self {
            algorithm: p.algorithm.into(),
            max_iterations: p.max_iterations,
            time_limit_seconds: p.time_limit_seconds,
            seed: p.seed,
        }
    }
}

/// Time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    /// Start time (ISO 8601)
    pub begin: String,
    /// End time (ISO 8601)
    pub end: String,
}

/// Execution period with additional info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPeriod {
    /// Start time (ISO 8601)
    pub begin: String,
    /// End time (ISO 8601)
    pub end: String,
    /// Duration in days
    #[serde(default)]
    pub duration_days: f64,
}

/// Scheduled unit (task assignment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledUnit {
    /// Task identifier
    pub task_id: String,
    /// Task name
    pub task_name: String,
    /// Scheduled start time
    pub begin: String,
    /// Scheduled end time
    pub end: String,
}

/// Unscheduled block info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnscheduledBlock {
    /// Block identifier
    pub id: String,
    /// Block name
    pub name: String,
}

/// Schedule statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleStats {
    /// Number of scheduled blocks
    pub scheduled_count: usize,
    /// Number of unscheduled blocks
    pub unscheduled_count: usize,
    /// Total number of blocks
    pub total_blocks: usize,
    /// Scheduling rate (0.0 to 1.0)
    pub scheduling_rate: f64,
    /// Fitness score
    pub fitness: f64,
}

/// Schedule result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResult {
    /// Scheduled units
    pub units: Vec<ScheduledUnit>,
    /// Fitness score
    pub fitness: f64,
    /// Number of scheduled blocks
    pub scheduled_count: usize,
    /// Unscheduled blocks
    pub unscheduled: Vec<UnscheduledBlock>,
    /// Number of unscheduled blocks
    pub unscheduled_count: usize,
}

/// Possible periods for a single block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPossiblePeriods {
    /// Block identifier
    pub block_id: String,
    /// Block name
    pub block_name: String,
    /// List of possible observation periods
    pub periods: Vec<Period>,
}
