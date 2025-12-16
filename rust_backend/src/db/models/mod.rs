//! Domain models for the scheduling system.
//! 
//! This module is organized into several submodules:
//! 
//! - [`schedule`]: Core schedule types (Schedule, SchedulingBlock, Period, Constraints)
//! - [`metadata`]: Schedule metadata and info types (ScheduleMetadata, ScheduleInfo)
//! - [`analytics`]: Analytics and visualization types (LightweightBlock, DistributionData, SkyMapData)
//! - [`python`]: PyO3 wrapper types for Python interop (visibility, timeline, insights, trends, comparison)

pub mod analytics;
pub mod metadata;
pub mod python;
pub mod schedule;

// Re-export all public types for convenience
pub use analytics::{
    DistributionBlock, DistributionData, DistributionStats, LightweightBlock, PriorityBinInfo,
    SkyMapData,
};
pub use metadata::{ScheduleInfo, ScheduleMetadata};
pub use python::{
    AnalyticsMetrics, BlockHistogramData, CompareBlock, CompareData, CompareStats,
    ConflictRecord, CorrelationEntry, EmpiricalRatePoint, HeatmapBin, InsightsBlock,
    InsightsData, ScheduleTimelineBlock, ScheduleTimelineData, SchedulingChange, SmoothedPoint,
    TopObservation, TrendsBlock, TrendsData, TrendsMetrics, VisibilityBin, VisibilityBlockSummary,
    VisibilityMapData,
};
pub use schedule::{
    Constraints, ConstraintsId, Period, Schedule, ScheduleId, SchedulingBlock, SchedulingBlockId,
    TargetId,
};
