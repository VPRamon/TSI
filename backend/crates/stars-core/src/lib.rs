//! # stars-core
//!
//! Safe, ergonomic Rust bindings to the STARS Core scheduling library.
//!
//! This crate provides a high-level API for modeling scheduling blocks and running
//! scheduling simulations using the STARS Core C++ library.
//!
//! ## Features
//!
//! - Load scheduling blocks from JSON
//! - Compute possible observation periods (prescheduler)
//! - Run scheduling algorithms (accumulative, hybrid)
//! - Export results as JSON
//! - Full pipeline convenience function
//!
//! ## Example
//!
//! ```rust,ignore
//! use stars_core::{Context, Blocks, SchedulingParams, run_scheduler};
//!
//! // Load configuration and blocks from JSON
//! let ctx = Context::from_file("schedule.json")?;
//! let blocks = Blocks::from_file("schedule.json")?;
//!
//! // Run scheduling
//! let params = SchedulingParams::default();
//! let schedule = run_scheduler(&ctx, &blocks, None, params)?;
//!
//! // Get results
//! let stats = schedule.stats()?;
//! println!("Scheduled: {}/{}", stats.scheduled_count, stats.total_blocks);
//! ```

mod context;
mod blocks;
mod periods;
mod schedule;
mod error;
mod types;

pub use context::Context;
pub use blocks::Blocks;
pub use periods::PossiblePeriods;
pub use schedule::Schedule;
pub use error::{Error, Result};
pub use types::*;

use stars_core_sys as ffi;

/// Get the version of the STARS FFI library
pub fn ffi_version() -> &'static str {
    unsafe {
        std::ffi::CStr::from_ptr(ffi::stars_ffi_version())
            .to_str()
            .unwrap_or("unknown")
    }
}

/// Get the version of the underlying STARS Core library
pub fn core_version() -> &'static str {
    unsafe {
        std::ffi::CStr::from_ptr(ffi::stars_core_version())
            .to_str()
            .unwrap_or("unknown")
    }
}

/// Clear the last error in the FFI layer
pub fn clear_error() {
    unsafe {
        ffi::stars_clear_error();
    }
}

/// Get the last error message from the FFI layer
pub fn last_error() -> Option<String> {
    unsafe {
        let ptr = ffi::stars_get_last_error();
        if ptr.is_null() {
            None
        } else {
            Some(
                std::ffi::CStr::from_ptr(ptr)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }
}

/// Run the full scheduling pipeline
///
/// This is a convenience function that loads configuration and blocks from JSON,
/// runs the prescheduler and scheduler, and returns the results.
///
/// # Arguments
///
/// * `input_json` - JSON string containing configuration and scheduling blocks
/// * `params` - Scheduling parameters
///
/// # Returns
///
/// JSON string with scheduling results
pub fn run_full_pipeline(input_json: &str, params: SchedulingParams) -> Result<String> {
    use std::ffi::CString;
    use std::ptr;

    let c_input = CString::new(input_json).map_err(|_| Error::InvalidInput("Input contains null bytes".into()))?;
    let ffi_params = params.into();
    let mut out_json: *mut std::os::raw::c_char = ptr::null_mut();

    let result = unsafe {
        ffi::stars_run_full_pipeline(c_input.as_ptr(), ffi_params, &mut out_json)
    };

    if result.is_ok() {
        let json_str = unsafe {
            if out_json.is_null() {
                return Err(Error::NullPointer);
            }
            let s = std::ffi::CStr::from_ptr(out_json)
                .to_string_lossy()
                .into_owned();
            ffi::stars_free_string(out_json);
            s
        };
        Ok(json_str)
    } else {
        Err(Error::from_ffi_result(&result))
    }
}

/// Run the full scheduling pipeline from a file
///
/// # Arguments
///
/// * `file_path` - Path to JSON file containing configuration and scheduling blocks
/// * `params` - Scheduling parameters
///
/// # Returns
///
/// JSON string with scheduling results
pub fn run_pipeline_from_file(file_path: &str, params: SchedulingParams) -> Result<String> {
    use std::ffi::CString;
    use std::ptr;

    let c_path = CString::new(file_path).map_err(|_| Error::InvalidInput("Path contains null bytes".into()))?;
    let ffi_params = params.into();
    let mut out_json: *mut std::os::raw::c_char = ptr::null_mut();

    let result = unsafe {
        ffi::stars_run_pipeline_from_file(c_path.as_ptr(), ffi_params, &mut out_json)
    };

    if result.is_ok() {
        let json_str = unsafe {
            if out_json.is_null() {
                return Err(Error::NullPointer);
            }
            let s = std::ffi::CStr::from_ptr(out_json)
                .to_string_lossy()
                .into_owned();
            ffi::stars_free_string(out_json);
            s
        };
        Ok(json_str)
    } else {
        Err(Error::from_ffi_result(&result))
    }
}

/// Compute possible periods for scheduling blocks
///
/// # Arguments
///
/// * `ctx` - STARS context with instrument and execution period
/// * `blocks` - Collection of scheduling blocks
///
/// # Returns
///
/// Computed possible periods for each block
pub fn compute_possible_periods(ctx: &Context, blocks: &Blocks) -> Result<PossiblePeriods> {
    PossiblePeriods::compute(ctx, blocks)
}

/// Run the scheduling algorithm
///
/// # Arguments
///
/// * `ctx` - STARS context with instrument and execution period
/// * `blocks` - Collection of scheduling blocks
/// * `periods` - Optional pre-computed possible periods
/// * `params` - Scheduling parameters
///
/// # Returns
///
/// Schedule result
pub fn run_scheduler(
    ctx: &Context,
    blocks: &Blocks,
    periods: Option<&PossiblePeriods>,
    params: SchedulingParams,
) -> Result<Schedule> {
    Schedule::run(ctx, blocks, periods, params)
}
