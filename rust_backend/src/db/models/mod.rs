//! Domain models for the scheduling system.
//!
//! This module is organized into several submodules:
//!
//! - [`schedule`]: Core schedule types (Schedule, SchedulingBlock, Period, Constraints)
//! - [`metadata`]: Schedule metadata and info types (use `crate::api::ScheduleInfo`)
//! - [`analytics`]: Analytics and visualization types (LightweightBlock, DistributionData, SkyMapData)
//! - [`visualization`]: Visualization domain models (visibility, timeline, insights, trends, comparison)

pub mod schedule;
pub mod visualization;

// Re-export all public types for convenience
// `ScheduleMetadata` removed; use `crate::api::ScheduleInfo` instead when
// returning lightweight schedule listings from repositories.
pub use visualization::{
    AnalyticsMetrics, BlockHistogramData, CompareBlock, CompareData, CompareStats, ConflictRecord,
    CorrelationEntry, EmpiricalRatePoint, HeatmapBin, InsightsBlock, InsightsData,
    ScheduleTimelineBlock, SchedulingChange, SmoothedPoint, TopObservation,
    TrendsBlock, TrendsData, TrendsMetrics, VisibilityBin,
};
pub use schedule::{
    Constraints, Schedule, SchedulingBlock,
};
