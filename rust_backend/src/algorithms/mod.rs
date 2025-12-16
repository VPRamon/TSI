//! Scheduling algorithms and analytics.
//!
//! This module provides analytical computations, conflict detection, and optimization
//! algorithms for telescope observation scheduling.
//!
//! # Components
//!
//! - [`analysis`]: Dataset-level analytics and correlation analysis
//! - [`conflicts`]: Scheduling conflict detection and resolution suggestions
//! - [`optimization`]: Greedy scheduling optimization algorithms
//!
//! # Example
//!
//! ```ignore
//! use tsi_rust::algorithms::compute_metrics;
//! use polars::prelude::*;
//!
//! # fn example(df: &DataFrame) -> Result<(), PolarsError> {
//! let metrics = compute_metrics(&df)?;
//! println!(\"Scheduling rate: {:.2}%\", metrics.scheduling_rate * 100.0);
//! # Ok(())
//! # }
//! ```

pub mod analysis;
pub mod conflicts;
pub mod optimization;

pub use analysis::{
    get_top_observations, AnalyticsSnapshot,
};
pub use conflicts::{
    find_conflicts, suggest_candidate_positions, CandidatePlacement, SchedulingConflict,
};
pub use optimization::{greedy_schedule, Constraint, Observation, OptimizationResult};
