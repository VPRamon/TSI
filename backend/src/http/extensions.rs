//! Backend extension contract (v1).
//!
//! TSI is intentionally algorithm-agnostic: the core router and
//! services know nothing about EST, HAP or any other scheduler.
//! Integrators (e.g. the PhD/EST adapter under `webapp/scripts/`) plug
//! algorithm-specific HTTP surface in through the [`BackendExtensions`]
//! registry built before calling [`super::router::create_router`].
//!
//! # Contract
//!
//! Extensions may contribute:
//! - **Extra routes** mounted under `/v1` alongside the built-in
//!   handlers. Route paths must NOT collide with TSI built-ins; the
//!   integrator owns the path prefix it chose (e.g. `/v1/est/...`).
//! - **Algorithm trace validators** invoked when a schedule is
//!   uploaded with an `algorithm_trace_jsonl` payload. Validators are
//!   keyed by the trace's `algorithm` field; the first matching
//!   validator gets the parsed summary and may reject the upload.
//!
//! Extensions may NOT mutate the core repository contract or
//! intercept built-in handlers. If you need that level of integration
//! you should fork TSI rather than register an extension.
//!
//! # Versioning
//!
//! Backwards-incompatible changes to this surface bump
//! [`EXTENSION_CONTRACT_VERSION`]. Integrators should assert against it
//! at startup so their build fails loudly when the contract moves.

use std::sync::Arc;

use axum::Router;

use super::state::AppState;

/// Current version of the backend extension contract.
///
/// Bump on any breaking change to the public surface of this module
/// or to the [`AlgorithmTraceValidator`] trait.
pub const EXTENSION_CONTRACT_VERSION: u32 = 1;

/// Validator invoked at upload time for traces tagged with a given
/// `algorithm` value. Implementations should return `Ok(())` if the
/// summary payload is structurally valid for the algorithm they own,
/// or a human-readable error string that will be surfaced to the user
/// via a 400 response.
pub trait AlgorithmTraceValidator: Send + Sync + 'static {
    /// The algorithm identifier this validator owns (e.g. `"est"`).
    /// Matches the `algorithm` field embedded in a trace summary.
    fn algorithm(&self) -> &'static str;

    /// Validate the parsed summary; returns the textual error to send
    /// back to the client when the payload is malformed.
    fn validate_summary(&self, summary: &serde_json::Value) -> Result<(), String>;
}

/// Registry of integrator-supplied extensions.
///
/// Construct via [`BackendExtensions::builder`] in the integrator's
/// binary, then pass to [`super::router::create_router_with_extensions`].
/// `Default` is intentionally empty — TSI ships zero algorithm-specific
/// routes.
#[derive(Default, Clone)]
pub struct BackendExtensions {
    pub(crate) extra_routes: Option<Router<AppState>>,
    pub(crate) trace_validators: Vec<Arc<dyn AlgorithmTraceValidator>>,
}

impl BackendExtensions {
    /// Start building a new extensions registry.
    pub fn builder() -> BackendExtensionsBuilder {
        BackendExtensionsBuilder::default()
    }

    /// Look up the validator that owns the given algorithm name, if any.
    pub fn trace_validator_for(
        &self,
        algorithm: &str,
    ) -> Option<&Arc<dyn AlgorithmTraceValidator>> {
        self.trace_validators
            .iter()
            .find(|v| v.algorithm() == algorithm)
    }

    /// Borrow the integrator's extra routes for mounting into the main
    /// router.
    pub(crate) fn take_extra_routes(&mut self) -> Option<Router<AppState>> {
        self.extra_routes.take()
    }
}

#[derive(Default)]
pub struct BackendExtensionsBuilder {
    extra_routes: Option<Router<AppState>>,
    trace_validators: Vec<Arc<dyn AlgorithmTraceValidator>>,
}

impl BackendExtensionsBuilder {
    /// Mount additional axum routes alongside the built-in `/v1` API.
    /// Successive calls merge using axum's [`Router::merge`].
    pub fn with_routes(mut self, routes: Router<AppState>) -> Self {
        self.extra_routes = Some(match self.extra_routes {
            Some(existing) => existing.merge(routes),
            None => routes,
        });
        self
    }

    /// Register an algorithm trace validator. Only one validator per
    /// algorithm is supported; a second registration replaces the first.
    pub fn with_trace_validator<V: AlgorithmTraceValidator>(mut self, validator: V) -> Self {
        let validator: Arc<dyn AlgorithmTraceValidator> = Arc::new(validator);
        let algo = validator.algorithm();
        self.trace_validators
            .retain(|existing| existing.algorithm() != algo);
        self.trace_validators.push(validator);
        self
    }

    pub fn build(self) -> BackendExtensions {
        BackendExtensions {
            extra_routes: self.extra_routes,
            trace_validators: self.trace_validators,
        }
    }
}
