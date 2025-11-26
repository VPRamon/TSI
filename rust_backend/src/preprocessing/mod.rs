//! Schedule preprocessing pipeline and validation.
//!
//! This module orchestrates the complete preprocessing workflow for telescope schedules,
//! including validation, enrichment with visibility data, and data quality checks.
//!
//! # Components
//!
//! - [`validator`]: Schedule validation with error and warning reporting
//! - [`enricher`]: Enrich schedules with visibility period data
//! - [`pipeline`]: Complete preprocessing pipeline coordinating validation and enrichment
//!
//! # Example
//!
//! ```ignore
//! use tsi_rust::preprocessing::preprocess_schedule;
//! use std::path::Path;
//!
//! let result = preprocess_schedule(
//!     Path::new(\"schedule.json\"),
//!     Some(Path::new(\"visibility.json\")),
//!     true  // validate
//! ).expect(\"Preprocessing failed\");
//!
//! println!(\"Loaded {} blocks\", result.dataframe.height());
//! if !result.validation.is_valid {
//!     eprintln!(\"Validation errors: {:?}\", result.validation.errors);
//! }
//! ```

pub mod enricher;
pub mod pipeline;
pub mod validator;

pub use enricher::ScheduleEnricher;
pub use pipeline::{preprocess_schedule, PreprocessConfig, PreprocessPipeline, PreprocessResult};
pub use validator::{ScheduleValidator, ValidationResult, ValidationStats};
