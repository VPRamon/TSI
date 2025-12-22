//! Schedule metadata and information models.
//!
//! This module contains types for schedule metadata and summary information:
//! - ScheduleMetadata: Lightweight metadata for schedule listings
//! - ScheduleInfo: Extended schedule information with block statistics

use chrono::{DateTime, Utc};
use pyo3::prelude::*;

/// Lightweight metadata about a schedule (for listings).
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct ScheduleMetadata {
    #[pyo3(get)]
    pub schedule_id: Option<i64>,
    #[pyo3(get)]
    pub schedule_name: String,
    pub upload_timestamp: DateTime<Utc>,
    #[pyo3(get)]
    pub checksum: String,
}

#[pymethods]
impl ScheduleMetadata {
    pub fn upload_timestamp_iso(&self) -> String {
        self.upload_timestamp.to_rfc3339()
    }

    fn __repr__(&self) -> String {
        format!(
            "ScheduleMetadata(id={:?}, name={}, uploaded={})",
            self.schedule_id,
            self.schedule_name,
            self.upload_timestamp.to_rfc3339(),
        )
    }
}

/// Extended schedule information including stats.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct ScheduleInfo {
    #[pyo3(get)]
    pub metadata: ScheduleMetadata,
    #[pyo3(get)]
    pub total_blocks: usize,
    #[pyo3(get)]
    pub scheduled_blocks: usize,
    #[pyo3(get)]
    pub unscheduled_blocks: usize,
}

#[pymethods]
impl ScheduleInfo {

    fn __repr__(&self) -> String {
        format!(
            "ScheduleInfo(total={}, scheduled={}, unscheduled={})",
            self.total_blocks, self.scheduled_blocks, self.unscheduled_blocks
        )
    }
}
