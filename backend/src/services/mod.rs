//! Service layer for business logic and orchestration.
//!
//! This module contains the service layer that sits between the database operations
//! and the Python bindings. Services orchestrate database calls and implement
//! business logic and data processing.

pub mod compare;
#[cfg(test)]
#[path = "compare_tests.rs"]
mod compare_tests;

pub mod distributions;

pub mod insights;
#[cfg(test)]
#[path = "insights_tests.rs"]
mod insights_tests;

pub mod sky_map;
#[cfg(test)]
#[path = "sky_map_tests.rs"]
mod sky_map_tests;

pub mod timeline;

pub mod trends;
#[cfg(test)]
#[path = "trends_tests.rs"]
mod trends_tests;

pub mod validation;
pub mod validation_report;
pub mod visibility;

pub use compare::py_get_compare_data;
pub use distributions::{py_get_distribution_data, py_get_distribution_data_analytics};
pub use insights::py_get_insights_data;
pub use sky_map::{py_get_sky_map_data, py_get_sky_map_data_analytics};
pub use timeline::py_get_schedule_timeline_data;
pub use trends::py_get_trends_data;
pub use validation_report::py_get_validation_report;
pub use visibility::compute_visibility_histogram_rust;
