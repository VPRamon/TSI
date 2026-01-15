//! # stars-core-sys
//!
//! Low-level FFI bindings to the STARS Core scheduling library via the stars_ffi C shim.
//!
//! This crate provides raw, unsafe bindings to the C API. For a safe, ergonomic API,
//! use the `stars-core` crate instead.
//!
//! ## Safety
//!
//! All functions in this crate are unsafe. Users must ensure:
//! - Pointers passed to functions are valid and properly aligned
//! - Strings are null-terminated UTF-8
//! - Handles are not used after being destroyed
//! - Memory allocated by `stars_*` functions is freed with corresponding `stars_free_*` functions
//!
//! ## Example
//!
//! ```rust,ignore
//! use stars_core_sys::*;
//! use std::ffi::{CStr, CString};
//! use std::ptr;
//!
//! unsafe {
//!     let version = CStr::from_ptr(stars_ffi_version());
//!     println!("STARS FFI version: {}", version.to_str().unwrap());
//! }
//! ```

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_double, c_int, c_void};

// ============================================================================
// Error Codes
// ============================================================================

/// Error codes returned by FFI functions
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StarsErrorCode {
    STARS_OK = 0,
    STARS_ERROR_NULL_POINTER = 1,
    STARS_ERROR_INVALID_JSON = 2,
    STARS_ERROR_SERIALIZATION = 3,
    STARS_ERROR_DESERIALIZATION = 4,
    STARS_ERROR_INVALID_HANDLE = 5,
    STARS_ERROR_SCHEDULING_FAILED = 6,
    STARS_ERROR_PRESCHEDULER_FAILED = 7,
    STARS_ERROR_IO = 8,
    STARS_ERROR_UNKNOWN = 99,
}

/// Result structure containing error code and optional message
#[repr(C)]
#[derive(Debug)]
pub struct StarsResult {
    pub code: StarsErrorCode,
    pub error_message: *mut c_char,
}

// ============================================================================
// Opaque Handle Types
// ============================================================================

/// Opaque handle to a STARS context
#[repr(C)]
pub struct StarsContext {
    _private: [u8; 0],
}

/// Opaque handle to a collection of scheduling blocks
#[repr(C)]
pub struct StarsBlocksCollection {
    _private: [u8; 0],
}

/// Opaque handle to computed possible periods
#[repr(C)]
pub struct StarsPossiblePeriods {
    _private: [u8; 0],
}

/// Opaque handle to a schedule result
#[repr(C)]
pub struct StarsSchedule {
    _private: [u8; 0],
}

pub type StarsContextHandle = *mut StarsContext;
pub type StarsBlocksHandle = *mut StarsBlocksCollection;
pub type StarsPossiblePeriodsHandle = *mut StarsPossiblePeriods;
pub type StarsScheduleHandle = *mut StarsSchedule;

// ============================================================================
// Scheduling Parameters
// ============================================================================

/// Scheduling algorithm type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StarsSchedulerType {
    STARS_SCHEDULER_ACCUMULATIVE = 0,
    STARS_SCHEDULER_HYBRID_ACCUMULATIVE = 1,
}

/// Scheduling parameters
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StarsSchedulingParams {
    pub algorithm: StarsSchedulerType,
    pub max_iterations: u32,
    pub time_limit_seconds: c_double,
    pub seed: i32,
}

impl Default for StarsSchedulingParams {
    fn default() -> Self {
        Self {
            algorithm: StarsSchedulerType::STARS_SCHEDULER_ACCUMULATIVE,
            max_iterations: 0,
            time_limit_seconds: 0.0,
            seed: -1,
        }
    }
}

// ============================================================================
// FFI Function Declarations
// ============================================================================

extern "C" {
    // Error Handling
    pub fn stars_get_last_error() -> *const c_char;
    pub fn stars_clear_error();

    // Memory Management
    pub fn stars_free_string(str: *mut c_char);
    pub fn stars_free_result(result: *mut StarsResult);

    // Context Management
    pub fn stars_context_create(
        config_json: *const c_char,
        out_handle: *mut StarsContextHandle,
    ) -> StarsResult;

    pub fn stars_context_create_from_file(
        file_path: *const c_char,
        out_handle: *mut StarsContextHandle,
    ) -> StarsResult;

    pub fn stars_context_destroy(handle: StarsContextHandle);

    pub fn stars_context_get_execution_period(
        handle: StarsContextHandle,
        out_json: *mut *mut c_char,
    ) -> StarsResult;

    // Scheduling Blocks Management
    pub fn stars_blocks_load_json(
        json: *const c_char,
        out_handle: *mut StarsBlocksHandle,
    ) -> StarsResult;

    pub fn stars_blocks_load_file(
        file_path: *const c_char,
        out_handle: *mut StarsBlocksHandle,
    ) -> StarsResult;

    pub fn stars_blocks_to_json(
        handle: StarsBlocksHandle,
        out_json: *mut *mut c_char,
    ) -> StarsResult;

    pub fn stars_blocks_count(handle: StarsBlocksHandle, out_count: *mut usize) -> StarsResult;

    pub fn stars_blocks_get_at(
        handle: StarsBlocksHandle,
        index: usize,
        out_json: *mut *mut c_char,
    ) -> StarsResult;

    pub fn stars_blocks_destroy(handle: StarsBlocksHandle);

    // Prescheduler
    pub fn stars_compute_possible_periods(
        ctx: StarsContextHandle,
        blocks: StarsBlocksHandle,
        out_handle: *mut StarsPossiblePeriodsHandle,
    ) -> StarsResult;

    pub fn stars_possible_periods_to_json(
        handle: StarsPossiblePeriodsHandle,
        out_json: *mut *mut c_char,
    ) -> StarsResult;

    pub fn stars_possible_periods_destroy(handle: StarsPossiblePeriodsHandle);

    // Scheduling Algorithm
    pub fn stars_scheduling_params_default() -> StarsSchedulingParams;

    pub fn stars_run_scheduler(
        ctx: StarsContextHandle,
        blocks: StarsBlocksHandle,
        possible_periods: StarsPossiblePeriodsHandle,
        params: StarsSchedulingParams,
        out_handle: *mut StarsScheduleHandle,
    ) -> StarsResult;

    pub fn stars_schedule_to_json(
        handle: StarsScheduleHandle,
        out_json: *mut *mut c_char,
    ) -> StarsResult;

    pub fn stars_schedule_get_stats(
        handle: StarsScheduleHandle,
        out_json: *mut *mut c_char,
    ) -> StarsResult;

    pub fn stars_schedule_destroy(handle: StarsScheduleHandle);

    // Full Pipeline
    pub fn stars_run_full_pipeline(
        input_json: *const c_char,
        params: StarsSchedulingParams,
        out_result_json: *mut *mut c_char,
    ) -> StarsResult;

    pub fn stars_run_pipeline_from_file(
        input_file_path: *const c_char,
        params: StarsSchedulingParams,
        out_result_json: *mut *mut c_char,
    ) -> StarsResult;

    // Version Info
    pub fn stars_ffi_version() -> *const c_char;
    pub fn stars_core_version() -> *const c_char;
}

// ============================================================================
// Helper Functions
// ============================================================================

impl StarsResult {
    /// Check if the result indicates success
    #[inline]
    pub fn is_ok(&self) -> bool {
        self.code == StarsErrorCode::STARS_OK
    }

    /// Check if the result indicates an error
    #[inline]
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params = StarsSchedulingParams::default();
        assert_eq!(params.algorithm, StarsSchedulerType::STARS_SCHEDULER_ACCUMULATIVE);
        assert_eq!(params.max_iterations, 0);
        assert_eq!(params.seed, -1);
    }

    #[test]
    fn test_error_code_values() {
        assert_eq!(StarsErrorCode::STARS_OK as i32, 0);
        assert_eq!(StarsErrorCode::STARS_ERROR_NULL_POINTER as i32, 1);
        assert_eq!(StarsErrorCode::STARS_ERROR_UNKNOWN as i32, 99);
    }
}
