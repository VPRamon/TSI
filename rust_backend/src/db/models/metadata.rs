//! Schedule metadata and information models.
//!
//! This module contains types for schedule metadata and summary information:
//! - ScheduleMetadata: Lightweight metadata for schedule listings
//! - ScheduleInfo: Extended schedule information with block statistics

use chrono::{DateTime, Utc};

/// Lightweight metadata about a schedule (for listings).
#[derive(Debug, Clone)]
pub struct ScheduleMetadata {
    pub schedule_id: Option<i64>,
    pub schedule_name: String,
    pub upload_timestamp: DateTime<Utc>,
    pub checksum: String,
}

impl ScheduleMetadata {
    pub fn upload_timestamp_iso(&self) -> String {
        self.upload_timestamp.to_rfc3339()
    }
}

/// Extended schedule information including stats.
#[derive(Debug, Clone)]
pub struct ScheduleInfo {
    pub metadata: ScheduleMetadata,
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
    pub unscheduled_blocks: usize,
}
