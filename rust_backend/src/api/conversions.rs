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

impl From<api::ScheduleId> for models::ScheduleId {
    fn from(id: api::ScheduleId) -> Self {
        models::ScheduleId(id.0)
    }
}

impl From<&models::Constraints> for api::Constraints {
    fn from(constraints: &models::Constraints) -> Self {
        api::Constraints {
            min_altitude: Some(constraints.min_alt.value()),
            max_altitude: Some(constraints.max_alt.value()),
            min_azimuth: Some(constraints.min_az.value()),
            max_azimuth: Some(constraints.max_az.value()),
            fixed_time: constraints.fixed_time.as_ref().map(|p| p.start.value()),
        }
    }
}

impl From<&models::SchedulingBlock> for api::SchedulingBlock {
    fn from(block: &models::SchedulingBlock) -> Self {
        api::SchedulingBlock {
            id: block.original_block_id.clone().unwrap_or_else(|| block.id.0.to_string()),
            ra: block.target_ra.value(),
            dec: block.target_dec.value(),
            priority: block.priority,
            scheduled: block.scheduled_period.is_some(),
            scheduled_start: block.scheduled_period.as_ref().map(|p| p.start.value()),
            scheduled_end: block.scheduled_period.as_ref().map(|p| p.stop.value()),
            constraints: Some((&block.constraints).into()),
        }
    }
}

impl From<&models::Schedule> for api::Schedule {
    fn from(schedule: &models::Schedule) -> Self {
        api::Schedule {
            name: schedule.name.clone(),
            blocks: schedule.blocks.iter().map(|b| b.into()).collect(),
            dark_periods: schedule.dark_periods.clone(),
            possible_periods: vec![], // Not stored in this model
        }
    }
}

impl From<&models::ScheduleMetadata> for api::ScheduleMetadata {
    fn from(metadata: &models::ScheduleMetadata) -> Self {
        api::ScheduleMetadata {
            schedule_id: metadata.schedule_id.unwrap_or(0),
            name: metadata.schedule_name.clone(),
            timestamp: metadata.upload_timestamp.to_rfc3339(),
            checksum: metadata.checksum.clone(),
        }
    }
}

impl From<&models::ScheduleInfo> for api::ScheduleInfo {
    fn from(info: &models::ScheduleInfo) -> Self {
        api::ScheduleInfo {
            schedule_id: info.metadata.schedule_id.unwrap_or(0),
            name: info.metadata.schedule_name.clone(),
            timestamp: info.metadata.upload_timestamp.to_rfc3339(),
            total_blocks: info.total_blocks,
            scheduled_blocks: info.scheduled_blocks,
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
            original_block_id: block.original_block_id.clone(),
            priority: block.priority,
            scheduled: block.scheduled,
            visibility_hours: block.total_visibility_hours.value(),
            ra: 0.0, // Not available in InsightsBlock
            dec: 0.0, // Not available in InsightsBlock
        }
    }
}

impl From<&models::AnalyticsMetrics> for api::AnalyticsMetrics {
    fn from(metrics: &models::AnalyticsMetrics) -> Self {
        api::AnalyticsMetrics {
            total_blocks: metrics.total_observations,
            scheduled_count: metrics.scheduled_count,
            mean_priority: metrics.mean_priority,
            mean_visibility: 0.0, // Not directly available, would need total_visibility_hours
        }
    }
}

impl From<&models::CorrelationEntry> for api::CorrelationEntry {
    fn from(entry: &models::CorrelationEntry) -> Self {
        api::CorrelationEntry {
            metric1: entry.variable1.clone(),
            metric2: entry.variable2.clone(),
            correlation: entry.correlation,
        }
    }
}

impl From<&models::ConflictRecord> for api::ConflictRecord {
    fn from(record: &models::ConflictRecord) -> Self {
        api::ConflictRecord {
            block_id_1: record.block_id_1.clone(),
            block_id_2: record.block_id_2.clone(),
            overlap_start: record.start_time_1.value(),
            overlap_end: record.stop_time_1.value(),
        }
    }
}

impl From<&models::TopObservation> for api::TopObservation {
    fn from(obs: &models::TopObservation) -> Self {
        api::TopObservation {
            original_block_id: obs.original_block_id.clone(),
            metric_value: obs.total_visibility_hours.value(),
            priority: obs.priority,
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
            conflicts: data.conflicts.iter().map(|c| c.into()).collect(),
            top_by_priority: data.top_priority.iter().map(|t| t.into()).collect(),
            top_by_visibility: data.top_visibility.iter().map(|t| t.into()).collect(),
        }
    }
}

impl From<&models::TrendsBlock> for api::TrendsBlock {
    fn from(block: &models::TrendsBlock) -> Self {
        api::TrendsBlock {
            original_block_id: block.original_block_id.clone(),
            priority: block.priority,
            scheduled: block.scheduled,
            visibility_hours: block.total_visibility_hours.value(),
        }
    }
}

impl From<&models::EmpiricalRatePoint> for api::EmpiricalRatePoint {
    fn from(point: &models::EmpiricalRatePoint) -> Self {
        api::EmpiricalRatePoint {
            priority: point.mid_value,
            rate: point.scheduled_rate,
            count: point.count,
        }
    }
}

impl From<&models::SmoothedPoint> for api::SmoothedPoint {
    fn from(point: &models::SmoothedPoint) -> Self {
        api::SmoothedPoint {
            priority: point.x,
            rate: point.y_smoothed,
        }
    }
}

impl From<&models::HeatmapBin> for api::HeatmapBin {
    fn from(bin: &models::HeatmapBin) -> Self {
        api::HeatmapBin {
            priority_bin: 0.0, // Not available - using time_mid as substitute
            visibility_bin: bin.visibility_mid.value(),
            count: 0, // Not directly available
            scheduled_count: 0, // Not directly available
        }
    }
}

impl From<&models::TrendsMetrics> for api::TrendsMetrics {
    fn from(metrics: &models::TrendsMetrics) -> Self {
        api::TrendsMetrics {
            overall_rate: metrics.scheduling_rate,
            priority_bins: 0, // Not available in internal model
            visibility_bins: 0, // Not available in internal model
        }
    }
}

impl From<&models::TrendsData> for api::TrendsData {
    fn from(data: &models::TrendsData) -> Self {
        api::TrendsData {
            blocks: data.blocks.iter().map(|b| b.into()).collect(),
            empirical_rates: data.by_priority.iter().map(|r| r.into()).collect(),
            smoothed_trend: data.smoothed_visibility.iter().map(|s| s.into()).collect(),
            heatmap: data.heatmap_bins.iter().map(|h| h.into()).collect(),
            metrics: (&data.metrics).into(),
        }
    }
}

impl From<&models::CompareBlock> for api::CompareBlock {
    fn from(block: &models::CompareBlock) -> Self {
        api::CompareBlock {
            original_block_id: block.scheduling_block_id.clone(),
            priority: block.priority,
            scheduled_a: block.scheduled,
            scheduled_b: false, // Single schedule, no B comparison
            ra: 0.0, // Not available in CompareBlock
            dec: 0.0, // Not available in CompareBlock
        }
    }
}

impl From<&models::CompareStats> for api::CompareStats {
    fn from(stats: &models::CompareStats) -> Self {
        api::CompareStats {
            total_blocks: stats.scheduled_count + stats.unscheduled_count,
            both_scheduled: stats.scheduled_count,
            only_a: 0, // Not available in single-schedule stats
            only_b: 0, // Not available in single-schedule stats
            neither: stats.unscheduled_count,
        }
    }
}

impl From<&models::SchedulingChange> for api::SchedulingChange {
    fn from(change: &models::SchedulingChange) -> Self {
        api::SchedulingChange {
            original_block_id: change.scheduling_block_id.clone(),
            change_type: change.change_type.clone(),
            priority: change.priority,
        }
    }
}

impl From<&models::CompareData> for api::CompareData {
    fn from(data: &models::CompareData) -> Self {
        api::CompareData {
            blocks: data.current_blocks.iter().map(|b| b.into()).collect(),
            stats: (&data.current_stats).into(),
            changes: data.scheduling_changes.iter().map(|c| c.into()).collect(),
        }
    }
}

// =========================================================
// =========================================================
// Phase 2 Analytics Types (Now directly in api::types)
// =========================================================
// Note: ScheduleSummary, PriorityRate, VisibilityBin, HeatmapBinData, 
// VisibilityTimeMetadata, and VisibilityTimeBin are now defined directly
// in api::types and do not need conversions as they are the source of truth.

// =========================================================
// Visibility Map Types
// =========================================================

impl From<&models::VisibilityBlockSummary> for api::VisibilityBlockSummary {
    fn from(block: &models::VisibilityBlockSummary) -> Self {
        api::VisibilityBlockSummary {
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
