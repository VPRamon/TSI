//! Type conversions between internal models and API DTOs.
//!
//! This module provides conversion traits to transform internal Rust types
//! (which use qtty types like MJD, Degrees, etc.) into Python-compatible DTOs
//! (which use only primitives like f64, String, etc.).
//!
//! ## Conversion Strategy
//!
//! - `From<InternalType> for ApiType`: Infallible conversion to API types
//! - `TryFrom<ApiType> for InternalType`: Fallible conversion from API types
//! - qtty types → f64 primitives (MJD::value(), Degrees::value())
//! - Strongly-typed IDs → i64 or String
//! - Option types preserved where semantically equivalent

use crate::api::types as api;
use crate::db::models;

// =========================================================
// Core Schedule Types - Internal to API
// =========================================================

impl From<models::ScheduleId> for api::ScheduleId {
    fn from(id: models::ScheduleId) -> Self {
        api::ScheduleId(id.0)
    }
}

impl From<models::TargetId> for api::TargetId {
    fn from(id: models::TargetId) -> Self {
        api::TargetId(id.0)
    }
}

impl From<models::ConstraintsId> for api::ConstraintsId {
    fn from(id: models::ConstraintsId) -> Self {
        api::ConstraintsId(id.0)
    }
}

impl From<models::SchedulingBlockId> for api::SchedulingBlockId {
    fn from(id: models::SchedulingBlockId) -> Self {
        api::SchedulingBlockId(id.0)
    }
}

impl From<api::ScheduleId> for models::ScheduleId {
    fn from(id: api::ScheduleId) -> Self {
        models::ScheduleId(id.0)
    }
}

impl From<api::TargetId> for models::TargetId {
    fn from(id: api::TargetId) -> Self {
        models::TargetId(id.0)
    }
}

impl From<api::ConstraintsId> for models::ConstraintsId {
    fn from(id: api::ConstraintsId) -> Self {
        models::ConstraintsId(id.0)
    }
}

impl From<api::SchedulingBlockId> for models::SchedulingBlockId {
    fn from(id: api::SchedulingBlockId) -> Self {
        models::SchedulingBlockId(id.0)
    }
}

impl From<&models::Period> for api::Period {
    fn from(period: &models::Period) -> Self {
        api::Period {
            start: period.start.value(),
            stop: period.stop.value(),
        }
    }
}

impl From<&models::Constraints> for api::Constraints {
    fn from(constraints: &models::Constraints) -> Self {
        api::Constraints {
            min_alt: constraints.min_alt.value(),
            max_alt: constraints.max_alt.value(),
            min_az: constraints.min_az.value(),
            max_az: constraints.max_az.value(),
            fixed_time: constraints.fixed_time.as_ref().map(|p| p.into()),
        }
    }
}

impl From<&models::SchedulingBlock> for api::SchedulingBlock {
    fn from(block: &models::SchedulingBlock) -> Self {
        api::SchedulingBlock {
            id: block.id.0,
            original_block_id: block.original_block_id.clone(),
            target_ra: block.target_ra.value(),
            target_dec: block.target_dec.value(),
            constraints: (&block.constraints).into(),
            priority: block.priority,
            min_observation: block.min_observation.value(),
            requested_duration: block.requested_duration.value(),
            visibility_periods: block.visibility_periods.iter().map(|p| p.into()).collect(),
            scheduled_period: block.scheduled_period.as_ref().map(|p| p.into()),
        }
    }
}

impl From<&models::Schedule> for api::Schedule {
    fn from(schedule: &models::Schedule) -> Self {
        api::Schedule {
            id: schedule.id.map(|id| id.0),
            name: schedule.name.clone(),
            checksum: schedule.checksum.clone(),
            dark_periods: schedule
                .dark_periods
                .iter()
                .map(|p| p.into())
                .collect(),
            blocks: schedule.blocks.iter().map(|b| b.into()).collect(),
        }
    }
}

impl From<&models::ScheduleMetadata> for api::ScheduleMetadata {
    fn from(metadata: &models::ScheduleMetadata) -> Self {
        api::ScheduleMetadata {
            schedule_id: metadata.schedule_id,
            schedule_name: metadata.schedule_name.clone(),
            upload_timestamp: metadata.upload_timestamp.to_rfc3339(),
            checksum: metadata.checksum.clone(),
        }
    }
}

// =========================================================
// Analytics Types - Internal to API
// =========================================================

impl From<&crate::api::LightweightBlock> for api::LightweightBlock {
    fn from(block: &crate::api::LightweightBlock) -> Self {
        api::LightweightBlock {
            original_block_id: block.original_block_id.clone(),
            priority: block.priority,
            priority_bin: block.priority_bin.clone(),
            requested_duration_seconds: block.requested_duration_seconds,
            target_ra_deg: block.target_ra_deg,
            target_dec_deg: block.target_dec_deg,
            scheduled_period: block.scheduled_period.clone()
        }
    }
}

impl From<&models::ScheduleTimelineBlock> for api::ScheduleTimelineBlock {
    fn from(block: &models::ScheduleTimelineBlock) -> Self {
        api::ScheduleTimelineBlock {
            scheduling_block_id: block.scheduling_block_id,
            original_block_id: block.original_block_id.clone(),
            priority: block.priority,
            scheduled_start_mjd: block.scheduled_start_mjd.value(),
            scheduled_stop_mjd: block.scheduled_stop_mjd.value(),
            ra_deg: block.ra_deg.value(),
            dec_deg: block.dec_deg.value(),
            requested_hours: block.requested_hours.value(),
            total_visibility_hours: block.total_visibility_hours.value(),
            num_visibility_periods: block.num_visibility_periods,
        }
    }
}

impl From<&models::InsightsBlock> for api::InsightsBlock {
    fn from(block: &models::InsightsBlock) -> Self {
        api::InsightsBlock {
            scheduling_block_id: block.scheduling_block_id,
            original_block_id: block.original_block_id.clone(),
            priority: block.priority,
            total_visibility_hours: block.total_visibility_hours.value(),
            requested_hours: block.requested_hours.value(),
            elevation_range_deg: block.elevation_range_deg.value(),
            scheduled: block.scheduled,
            scheduled_start_mjd: block.scheduled_start_mjd.map(|v| v.value()),
            scheduled_stop_mjd: block.scheduled_stop_mjd.map(|v| v.value()),
        }
    }
}

impl From<&models::AnalyticsMetrics> for api::AnalyticsMetrics {
    fn from(metrics: &models::AnalyticsMetrics) -> Self {
        api::AnalyticsMetrics {
            total_observations: metrics.total_observations,
            scheduled_count: metrics.scheduled_count,
            unscheduled_count: metrics.unscheduled_count,
            scheduling_rate: metrics.scheduling_rate,
            mean_priority: metrics.mean_priority,
            median_priority: metrics.median_priority,
            mean_priority_scheduled: metrics.mean_priority_scheduled,
            mean_priority_unscheduled: metrics.mean_priority_unscheduled,
            total_visibility_hours: metrics.total_visibility_hours.value(),
            mean_requested_hours: metrics.mean_requested_hours.value(),
        }
    }
}

impl From<&models::CorrelationEntry> for api::CorrelationEntry {
    fn from(entry: &models::CorrelationEntry) -> Self {
        api::CorrelationEntry {
            variable1: entry.variable1.clone(),
            variable2: entry.variable2.clone(),
            correlation: entry.correlation,
        }
    }
}

impl From<&models::ConflictRecord> for api::ConflictRecord {
    fn from(record: &models::ConflictRecord) -> Self {
        api::ConflictRecord {
            block_id_1: record.block_id_1.clone(),
            block_id_2: record.block_id_2.clone(),
            start_time_1: record.start_time_1.value(),
            stop_time_1: record.stop_time_1.value(),
            start_time_2: record.start_time_2.value(),
            stop_time_2: record.stop_time_2.value(),
            overlap_hours: record.overlap_hours.value(),
        }
    }
}

impl From<&models::TopObservation> for api::TopObservation {
    fn from(obs: &models::TopObservation) -> Self {
        api::TopObservation {
            scheduling_block_id: obs.scheduling_block_id,
            original_block_id: obs.original_block_id.clone(),
            priority: obs.priority,
            total_visibility_hours: obs.total_visibility_hours.value(),
            requested_hours: obs.requested_hours.value(),
            scheduled: obs.scheduled,
        }
    }
}

impl From<&models::InsightsData> for api::InsightsData {
    fn from(data: &models::InsightsData) -> Self {
        api::InsightsData {
            blocks: data.blocks.iter().map(|b| b.into()).collect(),
            metrics: (&data.metrics).into(),
            correlations: data.correlations.iter().map(|c| c.into()).collect(),
            top_priority: data.top_priority.iter().map(|t| t.into()).collect(),
            top_visibility: data.top_visibility.iter().map(|t| t.into()).collect(),
            conflicts: data.conflicts.iter().map(|c| c.into()).collect(),
            total_count: data.total_count,
            scheduled_count: data.scheduled_count,
            impossible_count: data.impossible_count,
        }
    }
}

impl From<&models::TrendsBlock> for api::TrendsBlock {
    fn from(block: &models::TrendsBlock) -> Self {
        api::TrendsBlock {
            scheduling_block_id: block.scheduling_block_id,
            original_block_id: block.original_block_id.clone(),
            priority: block.priority,
            total_visibility_hours: block.total_visibility_hours.value(),
            requested_hours: block.requested_hours.value(),
            scheduled: block.scheduled,
        }
    }
}

impl From<&models::EmpiricalRatePoint> for api::EmpiricalRatePoint {
    fn from(point: &models::EmpiricalRatePoint) -> Self {
        api::EmpiricalRatePoint {
            bin_label: point.bin_label.clone(),
            mid_value: point.mid_value,
            scheduled_rate: point.scheduled_rate,
            count: point.count,
        }
    }
}

impl From<&models::SmoothedPoint> for api::SmoothedPoint {
    fn from(point: &models::SmoothedPoint) -> Self {
        api::SmoothedPoint {
            x: point.x,
            y_smoothed: point.y_smoothed,
            n_samples: point.n_samples,
        }
    }
}

impl From<&models::HeatmapBin> for api::HeatmapBin {
    fn from(bin: &models::HeatmapBin) -> Self {
        api::HeatmapBin {
            visibility_mid: bin.visibility_mid.value(),
            time_mid: bin.time_mid.value(),
            scheduled_rate: bin.scheduled_rate,
            count: bin.count,
        }
    }
}

impl From<&models::TrendsMetrics> for api::TrendsMetrics {
    fn from(metrics: &models::TrendsMetrics) -> Self {
        api::TrendsMetrics {
            total_count: metrics.total_count,
            scheduled_count: metrics.scheduled_count,
            scheduling_rate: metrics.scheduling_rate,
            zero_visibility_count: metrics.zero_visibility_count,
            priority_min: metrics.priority_min,
            priority_max: metrics.priority_max,
            priority_mean: metrics.priority_mean,
            visibility_min: metrics.visibility_min.value(),
            visibility_max: metrics.visibility_max.value(),
            visibility_mean: metrics.visibility_mean.value(),
            time_min: metrics.time_min.value(),
            time_max: metrics.time_max.value(),
            time_mean: metrics.time_mean.value(),
        }
    }
}

impl From<&models::TrendsData> for api::TrendsData {
    fn from(data: &models::TrendsData) -> Self {
        api::TrendsData {
            blocks: data.blocks.iter().map(|b| b.into()).collect(),
            metrics: (&data.metrics).into(),
            by_priority: data.by_priority.iter().map(|r| r.into()).collect(),
            by_visibility: data.by_visibility.iter().map(|r| r.into()).collect(),
            by_time: data.by_time.iter().map(|r| r.into()).collect(),
            smoothed_visibility: data.smoothed_visibility.iter().map(|s| s.into()).collect(),
            smoothed_time: data.smoothed_time.iter().map(|s| s.into()).collect(),
            heatmap_bins: data.heatmap_bins.iter().map(|h| h.into()).collect(),
            priority_values: data.priority_values.clone(),
        }
    }
}

impl From<&models::CompareBlock> for api::CompareBlock {
    fn from(block: &models::CompareBlock) -> Self {
        api::CompareBlock {
            scheduling_block_id: block.scheduling_block_id.clone(),
            priority: block.priority,
            scheduled: block.scheduled,
            requested_hours: block.requested_hours.value(),
        }
    }
}

impl From<&models::CompareStats> for api::CompareStats {
    fn from(stats: &models::CompareStats) -> Self {
        api::CompareStats {
            scheduled_count: stats.scheduled_count,
            unscheduled_count: stats.unscheduled_count,
            total_priority: stats.total_priority,
            mean_priority: stats.mean_priority,
            median_priority: stats.median_priority,
            total_hours: stats.total_hours.value(),
        }
    }
}

impl From<&models::SchedulingChange> for api::SchedulingChange {
    fn from(change: &models::SchedulingChange) -> Self {
        api::SchedulingChange {
            scheduling_block_id: change.scheduling_block_id.clone(),
            priority: change.priority,
            change_type: change.change_type.clone(),
        }
    }
}

impl From<&models::CompareData> for api::CompareData {
    fn from(data: &models::CompareData) -> Self {
        api::CompareData {
            current_blocks: data.current_blocks.iter().map(|b| b.into()).collect(),
            comparison_blocks: data.comparison_blocks.iter().map(|b| b.into()).collect(),
            current_stats: (&data.current_stats).into(),
            comparison_stats: (&data.comparison_stats).into(),
            common_ids: data.common_ids.clone(),
            only_in_current: data.only_in_current.clone(),
            only_in_comparison: data.only_in_comparison.clone(),
            scheduling_changes: data.scheduling_changes.iter().map(|c| c.into()).collect(),
            current_name: data.current_name.clone(),
            comparison_name: data.comparison_name.clone(),
        }
    }
}

// =========================================================
// =========================================================
// Phase 2 Analytics Types (Now directly in api::types)
// =========================================================
// Note: ScheduleSummary, PriorityRate, VisibilityBinData, HeatmapBinData,
// VisibilityTimeMetadata, and VisibilityTimeBin are now defined directly
// in api::types and do not need conversions as they are the source of truth.

// =========================================================
// Visibility Map Types
// =========================================================

impl From<&models::VisibilityBlockSummary> for api::VisibilityBlockSummary {
    fn from(block: &models::VisibilityBlockSummary) -> Self {
        api::VisibilityBlockSummary {
            scheduling_block_id: block.scheduling_block_id,
            original_block_id: block.original_block_id.clone(),
            priority: block.priority,
            num_visibility_periods: block.num_visibility_periods,
            scheduled: block.scheduled,
        }
    }
}

impl From<&models::VisibilityMapData> for api::VisibilityMapData {
    fn from(data: &models::VisibilityMapData) -> Self {
        api::VisibilityMapData {
            blocks: data.blocks.iter().map(|b| b.into()).collect(),
            priority_min: data.priority_min,
            priority_max: data.priority_max,
            total_count: data.total_count,
            scheduled_count: data.scheduled_count,
        }
    }
}

impl From<&models::VisibilityBin> for api::VisibilityBin {
    fn from(bin: &models::VisibilityBin) -> Self {
        api::VisibilityBin {
            bin_start_unix: bin.bin_start_unix,
            bin_end_unix: bin.bin_end_unix,
            visible_count: bin.visible_count,
        }
    }
}

impl From<&models::BlockHistogramData> for api::BlockHistogramData {
    fn from(row: &models::BlockHistogramData) -> Self {
        api::BlockHistogramData {
            scheduling_block_id: row.scheduling_block_id,
            priority: row.priority,
            visibility_periods: row
                .visibility_periods
                .as_ref()
                .map(|periods| periods.iter().map(|p| p.into()).collect()),
        }
    }
}


// =========================================================
// Algorithm Result Types
// =========================================================

impl From<&crate::algorithms::SchedulingConflict> for api::SchedulingConflict {
    fn from(conflict: &crate::algorithms::SchedulingConflict) -> Self {
        api::SchedulingConflict {
            block_id_1: conflict.scheduling_block_id.clone(),
            block_id_2: String::new(),
            overlap_start: 0.0,
            overlap_end: 0.0,
            overlap_duration_hours: 0.0,
        }
    }
}

// =========================================================
// Helper functions for collections
// =========================================================

/// Convert a vector of internal models to API types.
pub fn convert_vec<T, U>(items: &[T]) -> Vec<U>
where
    U: for<'a> From<&'a T>,
{
    items.iter().map(|item| item.into()).collect()
}

/// Convert Option<InternalType> to Option<ApiType>.
pub fn convert_option<T, U>(item: &Option<T>) -> Option<U>
where
    U: for<'a> From<&'a T>,
{
    item.as_ref().map(|i| i.into())
}
