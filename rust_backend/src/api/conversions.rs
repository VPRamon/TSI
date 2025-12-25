//! Type conversions between internal models and API DTOs.
//!
//! This module provides conversion traits to transform internal Rust types
//! (which use qtty types like MJD, Degrees, etc.) into Python-compatible DTOs
//! (which use only primitives like f64, String, etc.).
//!
//! ## Conversion Strategy
//!
//! - `From<InternalType> for ApiType`: Infallible conversion to API types
//! - `TryFrom<ApiType> for InternalType`: Fallible conversion from API types
//! - qtty types → f64 primitives (MJD::value(), Degrees::value())
//! - Strongly-typed IDs → i64 or String
//! - Option types preserved where semantically equivalent

//! Re-export conversions implemented under `routes::conversions` so that
//! existing callers referencing `crate::api::conversions` keep working.

pub use crate::routes::conversions::*;
