pub mod analysis;
pub mod conflicts;
pub mod optimization;

pub use analysis::{compute_correlations, compute_metrics, get_top_observations, AnalyticsSnapshot};
pub use conflicts::{find_conflicts, suggest_candidate_positions, CandidatePlacement, SchedulingConflict};
pub use optimization::{greedy_schedule, Constraint, Observation, OptimizationResult};
