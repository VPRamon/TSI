//! Repository trait for algorithm traces.
//!
//! When a schedule is produced by an algorithm that emits a structured
//! trace, TSI stores it as two opaque JSONB blobs plus an `algorithm`
//! discriminator:
//!
//! - `summary` — algorithm configuration and run-level aggregates (taken
//!   from the `Started` and `Summary` events combined).
//! - `iterations` — the ordered list of per-iteration events.
//!
//! Storage is intentionally schemaless on the SQL side so that adding
//! fields on the algorithm side does not require a migration; only the
//! algorithm-specific frontend extension needs to be updated.

use async_trait::async_trait;
use serde_json::Value;

use super::error::RepositoryResult;
use crate::api::{AlgorithmTraceResponse, ScheduleId};

/// Repository operations for algorithm traces.
#[async_trait]
pub trait AlgorithmTraceRepository: Send + Sync {
    /// Persist a parsed trace for a schedule.
    ///
    /// Replaces any existing trace for the same `schedule_id`.
    async fn store_algorithm_trace(
        &self,
        schedule_id: ScheduleId,
        algorithm: &str,
        summary: &Value,
        iterations: &Value,
    ) -> RepositoryResult<()>;

    /// Retrieve the stored trace for a schedule.
    ///
    /// Returns `Ok(None)` when no trace was uploaded for that schedule.
    async fn get_algorithm_trace(
        &self,
        schedule_id: ScheduleId,
    ) -> RepositoryResult<Option<AlgorithmTraceResponse>>;

    /// Return the algorithm name for every schedule that has a stored trace.
    ///
    /// Used to populate `schedule_metadata.algorithm` on list-schedule
    /// responses without fetching full trace blobs.
    async fn list_algorithm_names(&self) -> RepositoryResult<Vec<(ScheduleId, String)>>;
}
