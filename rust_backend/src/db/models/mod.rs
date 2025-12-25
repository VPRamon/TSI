//! Domain models for the scheduling system.
//!
//! This module is organized into several submodules:
//!
//! - [`schedule`]: Core schedule types (Schedule, SchedulingBlock, Period, Constraints)
//! - [`metadata`]: Schedule metadata and info types (ScheduleMetadata, crate::api_tmp::ScheduleInfo)
//! - [`analytics`]: Analytics and visualization types (LightweightBlock, DistributionData, SkyMapData)
//! - [`visualization`]: Visualization domain models (visibility, timeline, insights, trends, comparison)

pub mod metadata;
pub mod schedule;
pub mod visualization;

// Re-export all public types for convenience
pub use metadata::{ScheduleMetadata};
pub use visualization::{
    AnalyticsMetrics, BlockHistogramData, CompareBlock, CompareData, CompareStats, ConflictRecord,
    CorrelationEntry, EmpiricalRatePoint, HeatmapBin, InsightsBlock, InsightsData,
    ScheduleTimelineBlock, SchedulingChange, SmoothedPoint, TopObservation,
    TrendsBlock, TrendsData, TrendsMetrics, VisibilityBin,
};
pub use schedule::{
    Constraints, ConstraintsId, Period, Schedule, ScheduleId, SchedulingBlock, SchedulingBlockId,
    TargetId,
};
