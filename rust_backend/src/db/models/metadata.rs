//! Schedule metadata and information models.
//!
//! This module contains types for schedule metadata and summary information:
//! - ScheduleMetadata: Lightweight metadata for schedule listings
//! - ScheduleInfo: Extended schedule information with block statistics

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Lightweight metadata about a schedule (for listings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleMetadata {
    pub schedule_id: Option<i64>,
    pub schedule_name: String,
    #[serde(serialize_with = "serialize_datetime", deserialize_with = "deserialize_datetime")]
    pub upload_timestamp: DateTime<Utc>,
    pub checksum: String,
}

impl ScheduleMetadata {
    pub fn upload_timestamp_iso(&self) -> String {
        self.upload_timestamp.to_rfc3339()
    }
}

fn serialize_datetime<S>(dt: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&dt.to_rfc3339())
}

fn deserialize_datetime<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(serde::de::Error::custom)
}
