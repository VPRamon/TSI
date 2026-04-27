//! Application state for the HTTP server.

use crate::db::repository::FullRepository;
use crate::http::extensions::BackendExtensions;
use crate::services::job_tracker::JobTracker;
use crate::services::{default_schedule_import_adapter, ScheduleImportAdapter};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Number of recent bulk-import requests whose timing/outcome we keep in
/// memory for the `/v1/_health/db` diagnostics endpoint.
const BULK_IMPORT_LATENCY_RING_SIZE: usize = 64;

/// One entry in the bulk-import latency ring buffer.
#[derive(Debug, Clone)]
pub struct BulkImportSample {
    /// Total wall-clock duration of the bulk-import handler call.
    pub duration_ms: u64,
    /// Number of items received in the request.
    pub items: usize,
    /// Number of `created` entries returned.
    pub created: usize,
    /// Number of `rejected` entries returned.
    pub rejected: usize,
    /// Server-side concurrency that handled the parallel fan-out.
    pub concurrency: usize,
    /// Environment id the request targeted.
    pub environment_id: i64,
    /// Wall-clock time the sample was recorded (unix millis).
    pub recorded_at_unix_ms: u64,
}

/// Bounded ring buffer of recent bulk-import samples. Cheap to clone (Arc).
#[derive(Debug, Clone, Default)]
pub struct BulkImportLatencyRing {
    inner: Arc<Mutex<VecDeque<BulkImportSample>>>,
}

impl BulkImportLatencyRing {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(
                BULK_IMPORT_LATENCY_RING_SIZE,
            ))),
        }
    }

    /// Push a new sample, evicting the oldest if the ring is full. Lock
    /// poisoning is intentionally swallowed: diagnostics must never break
    /// a real request.
    pub fn push(&self, sample: BulkImportSample) {
        if let Ok(mut q) = self.inner.lock() {
            if q.len() == BULK_IMPORT_LATENCY_RING_SIZE {
                q.pop_front();
            }
            q.push_back(sample);
        }
    }

    /// Snapshot the current contents (oldest → newest).
    pub fn snapshot(&self) -> Vec<BulkImportSample> {
        self.inner
            .lock()
            .map(|q| q.iter().cloned().collect())
            .unwrap_or_default()
    }
}

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Repository instance for database operations
    pub repository: Arc<dyn FullRepository>,
    /// Active import adapter for uploaded payloads
    pub import_adapter: Arc<dyn ScheduleImportAdapter>,
    /// Job tracker for async schedule processing
    pub job_tracker: JobTracker,
    /// Maximum number of bulk-import items processed concurrently within a
    /// single request. Bounded so a large batch can't exhaust the database
    /// connection pool. Read once from the `BULK_IMPORT_CONCURRENCY`
    /// environment variable at startup; defaults to 2 (matches the
    /// default `PG_POOL_MAX=8` divided by the 3-connections-per-item
    /// peak demand of the import pipeline).
    pub bulk_import_concurrency: usize,
    /// Recent bulk-import requests captured for the `/v1/_health/db`
    /// diagnostics endpoint. Cheap to clone — the underlying buffer is
    /// shared via `Arc<Mutex<_>>`.
    pub bulk_import_latencies: BulkImportLatencyRing,
    /// Integrator-supplied extension registry. The router clones this
    /// during construction to mount any extra routes; handlers may
    /// also consult it (e.g. to look up algorithm trace validators).
    pub extensions: Arc<BackendExtensions>,
}

impl AppState {
    /// Create a new application state with the given repository.
    pub fn new(repository: Arc<dyn FullRepository>) -> Self {
        Self::with_import_adapter(repository, default_schedule_import_adapter())
    }

    /// Create application state with an explicitly provided import adapter.
    pub fn with_import_adapter(
        repository: Arc<dyn FullRepository>,
        import_adapter: Arc<dyn ScheduleImportAdapter>,
    ) -> Self {
        Self {
            repository,
            import_adapter,
            job_tracker: JobTracker::new(),
            bulk_import_concurrency: bulk_import_concurrency_from_env(),
            bulk_import_latencies: BulkImportLatencyRing::new(),
            extensions: Arc::new(BackendExtensions::default()),
        }
    }

    /// Builder-style setter for the integrator extension registry.
    /// Replaces any previously-attached extensions.
    pub fn with_extensions(mut self, extensions: BackendExtensions) -> Self {
        self.extensions = Arc::new(extensions);
        self
    }
}

fn bulk_import_concurrency_from_env() -> usize {
    std::env::var("BULK_IMPORT_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or_else(default_bulk_import_concurrency)
}

/// Default fan-out used by the bulk-import handler when
/// `BULK_IMPORT_CONCURRENCY` is unset.
///
/// Conservative default of 2 keeps peak DB-pool demand at 6
/// connections (= 2 × 3-conn-per-item peak), which fits inside the
/// default `PG_POOL_MAX=8` while leaving headroom for HTTP request
/// traffic. Operators with larger pools can raise both knobs together
/// (`PG_POOL_MAX` and `BULK_IMPORT_CONCURRENCY`); the documented
/// invariant is `BULK_IMPORT_CONCURRENCY ≤ PG_POOL_MAX / 3`.
fn default_bulk_import_concurrency() -> usize {
    2
}
