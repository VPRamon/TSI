//! STARS Core integration module
//!
//! This module provides Rust bindings to the STARS Core C++ scheduling library,
//! allowing dynamic modeling of scheduling blocks and running scheduling simulations.
//!
//! The integration uses a C ABI shim (`stars_ffi`) that wraps the C++ library,
//! with safe Rust wrappers on top.

#[cfg(feature = "stars-core")]
pub mod stars;

#[cfg(feature = "stars-core")]
pub use stars::*;

#[cfg(all(test, feature = "stars-core"))]
mod tests;
