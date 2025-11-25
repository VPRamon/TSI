//! Data transformation and cleaning utilities.
//!
//! This module provides operations for cleaning, filtering, and transforming
//! schedule DataFrames, including duplicate removal, missing data handling,
//! and flexible filtering operations.
//!
//! # Modules
//!
//! - [`cleaning`]: Remove duplicates, handle missing data, validate schemas
//! - [`filtering`]: Filter DataFrames by various criteria
//!
//! # Example
//!
//! ```no_run
//! use tsi_rust::transformations::{remove_duplicates, filter_by_scheduled};
//! use polars::prelude::*;
//!
//! # fn example(df: DataFrame) -> Result<(), PolarsError> {
//! // Clean duplicates
//! let cleaned = remove_duplicates(&df, \"schedulingBlockId\")?;
//!
//! // Filter to scheduled observations only
//! let scheduled = filter_by_scheduled(&cleaned, true)?;
//! # Ok(())
//! # }
//! ```

pub mod cleaning;
pub mod filtering;

pub use cleaning::{
    remove_duplicates, remove_missing_coordinates, impute_missing, validate_schema,
};
pub use filtering::{
    filter_by_column, filter_by_range, filter_by_scheduled, filter_dataframe, validate_dataframe,
};
