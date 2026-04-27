//! Service layer for business logic and orchestration.
//!
//! This module contains the service layer that sits between the database
//! operations and the HTTP handlers. Services orchestrate database calls and
//! implement business logic and data processing.

pub mod altaz;
pub mod astronomical_night;
pub mod compare;
pub mod distributions;
pub mod environment_preschedule;
pub mod environment_structure;
pub mod fragmentation;
pub mod import_adapter;

pub mod insights;

pub mod sky_map;

pub mod timeline;

pub mod trends;

pub mod validation;

// Async job processing
pub mod job_tracker;
pub mod schedule_processor;

// Backend visibility fallback computation
pub mod visibility;

// KPI summary for Workspace verdict / delta / evolution UIs
pub mod schedule_kpis;

pub use altaz::compute_alt_az_data;
pub use environment_preschedule::{
    apply_to_schedule, compute_env_preschedule, EnvPreschedulePayload,
};
pub use environment_structure::{
    compute_blocks_hash, matches as match_structure, structure_from_schedule, StructureMismatch,
};
pub use import_adapter::{
    default_schedule_import_adapter, NativeScheduleImportAdapter, ScheduleImportAdapter,
};
pub use visibility::compute_visibility_histogram_rust;
