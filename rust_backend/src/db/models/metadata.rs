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
    pub schedule_id: Option<i64>,
    pub schedule_name: String,
    pub upload_timestamp: DateTime<Utc>,
    pub checksum: String,
}

#[pymethods]
impl ScheduleMetadata {
    #[getter]
    pub fn schedule_id(&self) -> Option<i64> {
        self.schedule_id
    }

    #[getter]
    pub fn schedule_name(&self) -> String {
        self.schedule_name.clone()
    }

    pub fn upload_timestamp_iso(&self) -> String {
        self.upload_timestamp.to_rfc3339()
    }

    #[getter]
    pub fn checksum(&self) -> String {
        self.checksum.clone()
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
    pub metadata: ScheduleMetadata,
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
    pub unscheduled_blocks: usize,
}

#[pymethods]
impl ScheduleInfo {
    #[getter]
    pub fn metadata(&self) -> ScheduleMetadata {
        self.metadata.clone()
    }

    #[getter]
    pub fn total_blocks(&self) -> usize {
        self.total_blocks
    }

    #[getter]
    pub fn scheduled_blocks(&self) -> usize {
        self.scheduled_blocks
    }

    #[getter]
    pub fn unscheduled_blocks(&self) -> usize {
        self.unscheduled_blocks
    }

    fn __repr__(&self) -> String {
        format!(
            "ScheduleInfo(total={}, scheduled={}, unscheduled={})",
            self.total_blocks, self.scheduled_blocks, self.unscheduled_blocks
        )
    }
}
