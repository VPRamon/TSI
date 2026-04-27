//! Generic algorithm trace API types.
//!
//! Some scheduling algorithms emit a structured trace alongside the schedule
//! they produce (a configuration / "Started" event, one event per iteration,
//! and a summary event).  TSI persists those traces verbatim — without
//! interpreting any algorithm-specific fields — so that algorithm-specific
//! frontend extensions can replay or visualise them.
//!
//! The schema is deliberately permissive: `summary` and each `iteration`
//! are opaque JSON objects.  Adding or removing fields on the producer
//! side requires no backend change.
//!
//! HTTP endpoint: `GET /v1/schedules/{id}/algorithm_trace`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::ScheduleId;

/// One iteration event from an algorithm trace.
///
/// The full event body is preserved as-is; consumers parse fields they
/// understand and ignore the rest.
pub type AlgorithmTraceIteration = Value;

/// Run-level summary combining the algorithm configuration recorded in the
/// `Started` event and the aggregates from the `Summary` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmTraceSummary {
    /// Algorithm identifier, e.g. `"est"`.
    #[serde(default)]
    pub algorithm: String,
    /// Frozen algorithm configuration as reported by the `Started` event.
    #[serde(default)]
    pub algorithm_config: Value,
    /// Catch-all for forward-compatible additions (run-level aggregates,
    /// total wall time, best score, ...).
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Response for `GET /v1/schedules/{id}/algorithm_trace`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmTraceResponse {
    pub schedule_id: ScheduleId,
    pub summary: AlgorithmTraceSummary,
    pub iterations: Vec<AlgorithmTraceIteration>,
}

/// Compact schedule-level metadata derived from the trace `Started` event.
///
/// Returned inline by some endpoints that already aggregate per-schedule data
/// so the frontend can group or facet by `algorithm` / `algorithm_config`
/// without an extra round-trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleMetadata {
    pub algorithm: String,
    #[serde(default)]
    pub algorithm_config: Value,
}
