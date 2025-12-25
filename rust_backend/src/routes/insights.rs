use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use crate::api::types as api;
use crate::db::models;

// =========================================================
// Insights types + route
// =========================================================

/// Block data for insights analysis.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsBlock {
    pub scheduling_block_id: i64,
    pub original_block_id: String,
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub elevation_range_deg: f64,
    pub scheduled: bool,
    pub scheduled_start_mjd: Option<f64>,
    pub scheduled_stop_mjd: Option<f64>,
}

/// Analytics metrics.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMetrics {
    pub total_observations: usize,
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub scheduling_rate: f64,
    pub mean_priority: f64,
    pub median_priority: f64,
    pub mean_priority_scheduled: f64,
    pub mean_priority_unscheduled: f64,
    pub total_visibility_hours: f64,
    pub mean_requested_hours: f64,
}

/// Correlation entry.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationEntry {
    pub variable1: String,
    pub variable2: String,
    pub correlation: f64,
}

/// Conflict record.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub block_id_1: String,
    pub block_id_2: String,
    pub start_time_1: f64,
    pub stop_time_1: f64,
    pub start_time_2: f64,
    pub stop_time_2: f64,
    pub overlap_hours: f64,
}

/// Top observation entry.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopObservation {
    pub scheduling_block_id: i64,
    pub original_block_id: String,
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub scheduled: bool,
}

/// Complete insights dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsData {
    pub blocks: Vec<InsightsBlock>,
    pub metrics: AnalyticsMetrics,
    pub correlations: Vec<CorrelationEntry>,
    pub top_priority: Vec<TopObservation>,
    pub top_visibility: Vec<TopObservation>,
    pub conflicts: Vec<ConflictRecord>,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub impossible_count: usize,
}

/// Route function name constant for insights
pub const GET_INSIGHTS_DATA: &str = "get_insights_data";

/// Get insights analysis data (wraps service call)
#[pyfunction]
pub fn get_insights_data(schedule_id: i64) -> PyResult<api::InsightsData> {
    let data = crate::services::py_get_insights_data(schedule_id)?;
    Ok((&data).into())
}

/// Register insights functions, classes and constants.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_insights_data, m)?)?;
    m.add_class::<InsightsBlock>()?;
    m.add_class::<AnalyticsMetrics>()?;
    m.add_class::<CorrelationEntry>()?;
    m.add_class::<ConflictRecord>()?;
    m.add_class::<TopObservation>()?;
    m.add_class::<InsightsData>()?;
    m.add("GET_INSIGHTS_DATA", GET_INSIGHTS_DATA)?;
    Ok(())
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
