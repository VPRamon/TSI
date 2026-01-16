//! Safe Rust API for STARS Core scheduling library
//!
//! This module provides complete Rust bindings to the STARS Core C++ scheduling library,
//! including raw FFI declarations and safe wrappers.
//!
//! # Architecture
//!
//! - `ffi` submodule: Raw, unsafe C bindings
//! - Safe wrapper types: `Context`, `Blocks`, `PossiblePeriods`, `Schedule`
//! - High-level functions: `run_full_pipeline`, `compute_possible_periods`, `run_scheduler`
//!
//! # Example
//!
//! ```rust,ignore
//! use tsi_rust::scheduler::stars::{
//!     Context, Blocks, Schedule, SchedulingParams, SchedulerType,
//!     compute_possible_periods, run_scheduler,
//! };
//!
//! // Load from schedule file
//! let ctx = Context::from_file("data/schedule.json")?;
//! let blocks = Blocks::from_file("data/schedule.json")?;
//!
//! // Compute visibility windows
//! let periods = compute_possible_periods(&ctx, &blocks)?;
//!
//! // Run scheduling
//! let params = SchedulingParams {
//!     algorithm: SchedulerType::HybridAccumulative,
//!     ..Default::default()
//! };
//! let schedule = run_scheduler(&ctx, &blocks, Some(&periods), params)?;
//!
//! // Get results
//! let stats = schedule.stats()?;
//! println!("Scheduled: {}/{}", stats.scheduled_count, stats.total_blocks);
//! ```

use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double};
use std::ptr;
use thiserror::Error;

// ============================================================================
// FFI Module - Raw C bindings
// ============================================================================

/// Raw FFI bindings to the STARS Core C shim library.
///
/// # Safety
///
/// All functions in this module are unsafe. Users must ensure:
/// - Pointers passed to functions are valid and properly aligned
/// - Strings are null-terminated UTF-8
/// - Handles are not used after being destroyed
/// - Memory allocated by `stars_*` functions is freed with corresponding `stars_free_*` functions
pub mod ffi {
    use super::*;

    // ========================================================================
    // Error Codes
    // ========================================================================

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

    // ========================================================================
    // Opaque Handle Types
    // ========================================================================

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

    // ========================================================================
    // Scheduling Parameters
    // ========================================================================

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

    // ========================================================================
    // FFI Function Declarations
    // ========================================================================

    #[link(name = "stars_ffi")]
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
}

// ============================================================================
// Error Types
// ============================================================================

/// Result type for stars operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the STARS Core library
#[derive(Error, Debug)]
pub enum Error {
    /// Null pointer encountered
    #[error("Null pointer error")]
    NullPointer,

    /// Invalid JSON input or output
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Invalid handle
    #[error("Invalid handle: {0}")]
    InvalidHandle(String),

    /// Scheduling algorithm failed
    #[error("Scheduling failed: {0}")]
    SchedulingFailed(String),

    /// Prescheduler failed
    #[error("Prescheduler failed: {0}")]
    PreschedulerFailed(String),

    /// I/O error (file operations)
    #[error("I/O error: {0}")]
    Io(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Unknown error
    #[error("Unknown error: {0}")]
    Unknown(String),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl Error {
    /// Create an Error from an FFI result
    fn from_ffi_result(result: &ffi::StarsResult) -> Self {
        let message = if result.error_message.is_null() {
            String::new()
        } else {
            unsafe {
                CStr::from_ptr(result.error_message)
                    .to_string_lossy()
                    .into_owned()
            }
        };

        match result.code {
            ffi::StarsErrorCode::STARS_OK => {
                // This shouldn't happen, but handle it gracefully
                Error::Unknown("Unexpected OK status treated as error".into())
            }
            ffi::StarsErrorCode::STARS_ERROR_NULL_POINTER => Error::NullPointer,
            ffi::StarsErrorCode::STARS_ERROR_INVALID_JSON => Error::InvalidJson(message),
            ffi::StarsErrorCode::STARS_ERROR_SERIALIZATION => Error::Serialization(message),
            ffi::StarsErrorCode::STARS_ERROR_DESERIALIZATION => Error::Deserialization(message),
            ffi::StarsErrorCode::STARS_ERROR_INVALID_HANDLE => Error::InvalidHandle(message),
            ffi::StarsErrorCode::STARS_ERROR_SCHEDULING_FAILED => Error::SchedulingFailed(message),
            ffi::StarsErrorCode::STARS_ERROR_PRESCHEDULER_FAILED => {
                Error::PreschedulerFailed(message)
            }
            ffi::StarsErrorCode::STARS_ERROR_IO => Error::Io(message),
            ffi::StarsErrorCode::STARS_ERROR_UNKNOWN => Error::Unknown(message),
        }
    }
}

// ============================================================================
// Type Definitions
// ============================================================================

/// Scheduling algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerType {
    /// Accumulative scheduling algorithm
    #[default]
    Accumulative,
    /// Hybrid accumulative scheduling algorithm
    HybridAccumulative,
}

impl From<SchedulerType> for ffi::StarsSchedulerType {
    fn from(t: SchedulerType) -> Self {
        match t {
            SchedulerType::Accumulative => ffi::StarsSchedulerType::STARS_SCHEDULER_ACCUMULATIVE,
            SchedulerType::HybridAccumulative => {
                ffi::StarsSchedulerType::STARS_SCHEDULER_HYBRID_ACCUMULATIVE
            }
        }
    }
}

impl From<ffi::StarsSchedulerType> for SchedulerType {
    fn from(t: ffi::StarsSchedulerType) -> Self {
        match t {
            ffi::StarsSchedulerType::STARS_SCHEDULER_ACCUMULATIVE => SchedulerType::Accumulative,
            ffi::StarsSchedulerType::STARS_SCHEDULER_HYBRID_ACCUMULATIVE => {
                SchedulerType::HybridAccumulative
            }
        }
    }
}

/// Parameters for the scheduling algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingParams {
    /// Algorithm to use
    #[serde(default)]
    pub algorithm: SchedulerType,

    /// Maximum number of iterations (0 = default)
    #[serde(default)]
    pub max_iterations: u32,

    /// Time limit in seconds (0 = no limit)
    #[serde(default)]
    pub time_limit_seconds: f64,

    /// Random seed (-1 = random)
    #[serde(default = "default_seed")]
    pub seed: i32,
}

fn default_seed() -> i32 {
    -1
}

impl Default for SchedulingParams {
    fn default() -> Self {
        Self {
            algorithm: SchedulerType::default(),
            max_iterations: 0,
            time_limit_seconds: 0.0,
            seed: -1,
        }
    }
}

impl From<SchedulingParams> for ffi::StarsSchedulingParams {
    fn from(p: SchedulingParams) -> Self {
        ffi::StarsSchedulingParams {
            algorithm: p.algorithm.into(),
            max_iterations: p.max_iterations,
            time_limit_seconds: p.time_limit_seconds,
            seed: p.seed,
        }
    }
}

impl From<ffi::StarsSchedulingParams> for SchedulingParams {
    fn from(p: ffi::StarsSchedulingParams) -> Self {
        Self {
            algorithm: p.algorithm.into(),
            max_iterations: p.max_iterations,
            time_limit_seconds: p.time_limit_seconds,
            seed: p.seed,
        }
    }
}

/// Time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    /// Start time (ISO 8601)
    pub begin: String,
    /// End time (ISO 8601)
    pub end: String,
}

/// Execution period with additional info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPeriod {
    /// Start time (ISO 8601)
    pub begin: String,
    /// End time (ISO 8601)
    pub end: String,
    /// Duration in days
    #[serde(default)]
    pub duration_days: f64,
}

/// Scheduled unit (task assignment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledUnit {
    /// Task identifier
    pub task_id: String,
    /// Task name
    pub task_name: String,
    /// Scheduled start time
    pub begin: String,
    /// Scheduled end time
    pub end: String,
}

/// Unscheduled block info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnscheduledBlock {
    /// Block identifier
    pub id: String,
    /// Block name
    pub name: String,
}

/// Schedule statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleStats {
    /// Number of scheduled blocks
    pub scheduled_count: usize,
    /// Number of unscheduled blocks
    pub unscheduled_count: usize,
    /// Total number of blocks
    pub total_blocks: usize,
    /// Scheduling rate (0.0 to 1.0)
    pub scheduling_rate: f64,
    /// Fitness score
    pub fitness: f64,
}

/// Schedule result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleResult {
    /// Scheduled units
    pub units: Vec<ScheduledUnit>,
    /// Fitness score
    pub fitness: f64,
    /// Number of scheduled blocks
    pub scheduled_count: usize,
    /// Unscheduled blocks
    pub unscheduled: Vec<UnscheduledBlock>,
    /// Number of unscheduled blocks
    pub unscheduled_count: usize,
}

/// Possible periods for a single block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPossiblePeriods {
    /// Block identifier
    pub block_id: String,
    /// Block name
    pub block_name: String,
    /// List of possible observation periods
    pub periods: Vec<Period>,
}

// ============================================================================
// Safe Wrapper Types
// ============================================================================

/// STARS scheduling context
///
/// Holds the instrument configuration, execution period, and other settings
/// needed for scheduling operations.
///
/// # Example
///
/// ```rust,ignore
/// use tsi_rust::scheduler::stars::Context;
///
/// // Create from JSON configuration
/// let config = r#"{
///     "instrument": { ... },
///     "executionPeriod": { "begin": "2024-01-01T00:00:00", "end": "2024-12-31T23:59:59" }
/// }"#;
/// let ctx = Context::from_json(config)?;
///
/// // Or load from file
/// let ctx = Context::from_file("schedule.json")?;
/// ```
pub struct Context {
    handle: ffi::StarsContextHandle,
}

impl Context {
    /// Create a new context from JSON configuration
    ///
    /// The JSON should contain:
    /// - `instrument`: Instrument configuration object
    /// - `executionPeriod`: Object with `begin` and `end` ISO datetime strings
    /// - `observatory`: Optional observatory name string
    pub fn from_json(config_json: &str) -> Result<Self> {
        let c_config = CString::new(config_json)
            .map_err(|_| Error::InvalidInput("Config contains null bytes".into()))?;
        let mut handle: ffi::StarsContextHandle = ptr::null_mut();

        let result = unsafe { ffi::stars_context_create(c_config.as_ptr(), &mut handle) };

        if result.is_ok() {
            Ok(Self { handle })
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Create a new context from a schedule JSON file
    ///
    /// The file should contain the same structure as `from_json`.
    pub fn from_file(file_path: &str) -> Result<Self> {
        let c_path = CString::new(file_path)
            .map_err(|_| Error::InvalidInput("Path contains null bytes".into()))?;
        let mut handle: ffi::StarsContextHandle = ptr::null_mut();

        let result = unsafe { ffi::stars_context_create_from_file(c_path.as_ptr(), &mut handle) };

        if result.is_ok() {
            Ok(Self { handle })
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Get the execution period
    pub fn execution_period(&self) -> Result<ExecutionPeriod> {
        let mut out_json: *mut c_char = ptr::null_mut();

        let result =
            unsafe { ffi::stars_context_get_execution_period(self.handle, &mut out_json) };

        if result.is_ok() {
            let json_str = unsafe {
                if out_json.is_null() {
                    return Err(Error::NullPointer);
                }
                let s = CStr::from_ptr(out_json).to_string_lossy().into_owned();
                ffi::stars_free_string(out_json);
                s
            };
            let period: ExecutionPeriod = serde_json::from_str(&json_str)?;
            Ok(period)
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Get the raw FFI handle (for internal use)
    fn handle(&self) -> ffi::StarsContextHandle {
        self.handle
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::stars_context_destroy(self.handle);
            }
        }
    }
}

// Context owns its handle and can be sent between threads
unsafe impl Send for Context {}

/// Collection of scheduling blocks
///
/// Represents a set of scheduling blocks (tasks, sequences, etc.) that can be
/// scheduled by the STARS scheduling algorithm.
///
/// # Example
///
/// ```rust,ignore
/// use tsi_rust::scheduler::stars::Blocks;
///
/// // Load from JSON string
/// let json = r#"{ "schedulingBlocks": [...] }"#;
/// let blocks = Blocks::from_json(json)?;
///
/// // Or load from file
/// let blocks = Blocks::from_file("schedule.json")?;
///
/// // Get count
/// println!("Loaded {} blocks", blocks.len()?);
/// ```
pub struct Blocks {
    handle: ffi::StarsBlocksHandle,
}

impl Blocks {
    /// Load scheduling blocks from a JSON string
    ///
    /// The JSON can be either:
    /// - An array of scheduling block objects
    /// - An object with a `schedulingBlocks` key containing the array
    pub fn from_json(json: &str) -> Result<Self> {
        let c_json =
            CString::new(json).map_err(|_| Error::InvalidInput("JSON contains null bytes".into()))?;
        let mut handle: ffi::StarsBlocksHandle = ptr::null_mut();

        let result = unsafe { ffi::stars_blocks_load_json(c_json.as_ptr(), &mut handle) };

        if result.is_ok() {
            Ok(Self { handle })
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Load scheduling blocks from a schedule JSON file
    pub fn from_file(file_path: &str) -> Result<Self> {
        let c_path = CString::new(file_path)
            .map_err(|_| Error::InvalidInput("Path contains null bytes".into()))?;
        let mut handle: ffi::StarsBlocksHandle = ptr::null_mut();

        let result = unsafe { ffi::stars_blocks_load_file(c_path.as_ptr(), &mut handle) };

        if result.is_ok() {
            Ok(Self { handle })
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Serialize blocks to JSON string
    pub fn to_json(&self) -> Result<String> {
        let mut out_json: *mut c_char = ptr::null_mut();

        let result = unsafe { ffi::stars_blocks_to_json(self.handle, &mut out_json) };

        if result.is_ok() {
            let json_str = unsafe {
                if out_json.is_null() {
                    return Err(Error::NullPointer);
                }
                let s = CStr::from_ptr(out_json).to_string_lossy().into_owned();
                ffi::stars_free_string(out_json);
                s
            };
            Ok(json_str)
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Get the number of blocks in the collection
    pub fn len(&self) -> Result<usize> {
        let mut count: usize = 0;

        let result = unsafe { ffi::stars_blocks_count(self.handle, &mut count) };

        if result.is_ok() {
            Ok(count)
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    /// Get a single block by index as JSON
    pub fn get(&self, index: usize) -> Result<String> {
        let mut out_json: *mut c_char = ptr::null_mut();

        let result = unsafe { ffi::stars_blocks_get_at(self.handle, index, &mut out_json) };

        if result.is_ok() {
            let json_str = unsafe {
                if out_json.is_null() {
                    return Err(Error::NullPointer);
                }
                let s = CStr::from_ptr(out_json).to_string_lossy().into_owned();
                ffi::stars_free_string(out_json);
                s
            };
            Ok(json_str)
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Get a single block by index as a typed value
    pub fn get_as<T: serde::de::DeserializeOwned>(&self, index: usize) -> Result<T> {
        let json = self.get(index)?;
        let value: T = serde_json::from_str(&json)?;
        Ok(value)
    }

    /// Iterate over blocks as JSON strings
    pub fn iter(&self) -> BlocksIter<'_> {
        BlocksIter {
            blocks: self,
            index: 0,
            len: self.len().unwrap_or(0),
        }
    }

    /// Get the raw FFI handle (for internal use)
    fn handle(&self) -> ffi::StarsBlocksHandle {
        self.handle
    }
}

impl Drop for Blocks {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::stars_blocks_destroy(self.handle);
            }
        }
    }
}

// Blocks owns its handle and can be sent between threads
unsafe impl Send for Blocks {}

/// Iterator over scheduling blocks
pub struct BlocksIter<'a> {
    blocks: &'a Blocks,
    index: usize,
    len: usize,
}

impl<'a> Iterator for BlocksIter<'a> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            None
        } else {
            let result = self.blocks.get(self.index);
            self.index += 1;
            Some(result)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for BlocksIter<'a> {}

/// Computed possible observation periods
///
/// Contains the time periods during which each scheduling block can be observed,
/// taking into account instrument constraints, astronomical conditions, and time windows.
///
/// # Example
///
/// ```rust,ignore
/// use tsi_rust::scheduler::stars::{Context, Blocks, PossiblePeriods};
///
/// let ctx = Context::from_file("schedule.json")?;
/// let blocks = Blocks::from_file("schedule.json")?;
///
/// // Compute possible periods
/// let periods = PossiblePeriods::compute(&ctx, &blocks)?;
///
/// // Export as JSON
/// let json = periods.to_json()?;
/// ```
pub struct PossiblePeriods {
    handle: ffi::StarsPossiblePeriodsHandle,
}

impl PossiblePeriods {
    /// Compute possible periods for scheduling blocks
    ///
    /// This runs the STARS prescheduler which computes when each task can be observed
    /// given instrument constraints, astronomical conditions, and time windows.
    fn compute(ctx: &Context, blocks: &Blocks) -> Result<Self> {
        let mut handle: ffi::StarsPossiblePeriodsHandle = ptr::null_mut();

        let result = unsafe {
            ffi::stars_compute_possible_periods(ctx.handle(), blocks.handle(), &mut handle)
        };

        if result.is_ok() {
            Ok(Self { handle })
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Export possible periods to JSON string
    pub fn to_json(&self) -> Result<String> {
        let mut out_json: *mut c_char = ptr::null_mut();

        let result = unsafe { ffi::stars_possible_periods_to_json(self.handle, &mut out_json) };

        if result.is_ok() {
            let json_str = unsafe {
                if out_json.is_null() {
                    return Err(Error::NullPointer);
                }
                let s = CStr::from_ptr(out_json).to_string_lossy().into_owned();
                ffi::stars_free_string(out_json);
                s
            };
            Ok(json_str)
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Export possible periods as typed values
    pub fn to_vec(&self) -> Result<Vec<BlockPossiblePeriods>> {
        let json = self.to_json()?;
        let periods: Vec<BlockPossiblePeriods> = serde_json::from_str(&json)?;
        Ok(periods)
    }

    /// Get the raw FFI handle (for internal use)
    fn handle(&self) -> ffi::StarsPossiblePeriodsHandle {
        self.handle
    }
}

impl Drop for PossiblePeriods {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::stars_possible_periods_destroy(self.handle);
            }
        }
    }
}

// PossiblePeriods owns its handle and can be sent between threads
unsafe impl Send for PossiblePeriods {}

/// Schedule result from the scheduling algorithm
///
/// Contains the scheduled units, unscheduled blocks, and fitness score.
///
/// # Example
///
/// ```rust,ignore
/// use tsi_rust::scheduler::stars::{Context, Blocks, Schedule, SchedulingParams};
///
/// let ctx = Context::from_file("schedule.json")?;
/// let blocks = Blocks::from_file("schedule.json")?;
/// let params = SchedulingParams::default();
///
/// let schedule = Schedule::run(&ctx, &blocks, None, params)?;
///
/// // Get statistics
/// let stats = schedule.stats()?;
/// println!("Scheduled {}/{} blocks", stats.scheduled_count, stats.total_blocks);
/// println!("Fitness: {}", stats.fitness);
/// ```
pub struct Schedule {
    handle: ffi::StarsScheduleHandle,
}

impl Schedule {
    /// Run the scheduling algorithm
    ///
    /// # Arguments
    ///
    /// * `ctx` - STARS context with instrument and execution period
    /// * `blocks` - Collection of scheduling blocks
    /// * `periods` - Optional pre-computed possible periods (if None, computed internally)
    /// * `params` - Scheduling parameters
    fn run(
        ctx: &Context,
        blocks: &Blocks,
        periods: Option<&PossiblePeriods>,
        params: SchedulingParams,
    ) -> Result<Self> {
        let mut handle: ffi::StarsScheduleHandle = ptr::null_mut();
        let periods_handle = periods.map(|p| p.handle()).unwrap_or(ptr::null_mut());
        let ffi_params = params.into();

        let result = unsafe {
            ffi::stars_run_scheduler(
                ctx.handle(),
                blocks.handle(),
                periods_handle,
                ffi_params,
                &mut handle,
            )
        };

        if result.is_ok() {
            Ok(Self { handle })
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Export schedule to JSON string
    pub fn to_json(&self) -> Result<String> {
        let mut out_json: *mut c_char = ptr::null_mut();

        let result = unsafe { ffi::stars_schedule_to_json(self.handle, &mut out_json) };

        if result.is_ok() {
            let json_str = unsafe {
                if out_json.is_null() {
                    return Err(Error::NullPointer);
                }
                let s = CStr::from_ptr(out_json).to_string_lossy().into_owned();
                ffi::stars_free_string(out_json);
                s
            };
            Ok(json_str)
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Export schedule as typed result
    pub fn to_result(&self) -> Result<ScheduleResult> {
        let json = self.to_json()?;
        let result: ScheduleResult = serde_json::from_str(&json)?;
        Ok(result)
    }

    /// Get schedule statistics
    pub fn stats(&self) -> Result<ScheduleStats> {
        let mut out_json: *mut c_char = ptr::null_mut();

        let result = unsafe { ffi::stars_schedule_get_stats(self.handle, &mut out_json) };

        if result.is_ok() {
            let json_str = unsafe {
                if out_json.is_null() {
                    return Err(Error::NullPointer);
                }
                let s = CStr::from_ptr(out_json).to_string_lossy().into_owned();
                ffi::stars_free_string(out_json);
                s
            };
            let stats: ScheduleStats = serde_json::from_str(&json_str)?;
            Ok(stats)
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }
}

impl Drop for Schedule {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::stars_schedule_destroy(self.handle);
            }
        }
    }
}

// Schedule owns its handle and can be sent between threads
unsafe impl Send for Schedule {}

// ============================================================================
// High-Level Functions
// ============================================================================

/// Get the version of the STARS FFI library
pub fn ffi_version() -> String {
    unsafe {
        CStr::from_ptr(ffi::stars_ffi_version())
            .to_str()
            .unwrap_or("unknown")
            .to_string()
    }
}

/// Get the version of the underlying STARS Core library
pub fn core_version() -> String {
    unsafe {
        CStr::from_ptr(ffi::stars_core_version())
            .to_str()
            .unwrap_or("unknown")
            .to_string()
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
            Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
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
    let c_input =
        CString::new(input_json).map_err(|_| Error::InvalidInput("Input contains null bytes".into()))?;
    let ffi_params = params.into();
    let mut out_json: *mut c_char = ptr::null_mut();

    let result = unsafe { ffi::stars_run_full_pipeline(c_input.as_ptr(), ffi_params, &mut out_json) };

    if result.is_ok() {
        let json_str = unsafe {
            if out_json.is_null() {
                return Err(Error::NullPointer);
            }
            let s = CStr::from_ptr(out_json).to_string_lossy().into_owned();
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
    let c_path =
        CString::new(file_path).map_err(|_| Error::InvalidInput("Path contains null bytes".into()))?;
    let ffi_params = params.into();
    let mut out_json: *mut c_char = ptr::null_mut();

    let result =
        unsafe { ffi::stars_run_pipeline_from_file(c_path.as_ptr(), ffi_params, &mut out_json) };

    if result.is_ok() {
        let json_str = unsafe {
            if out_json.is_null() {
                return Err(Error::NullPointer);
            }
            let s = CStr::from_ptr(out_json).to_string_lossy().into_owned();
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

// ============================================================================
// TSI-specific Extensions
// ============================================================================

/// Extended scheduling parameters with TSI-specific options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TsiSchedulingParams {
    /// Base STARS Core parameters
    #[serde(flatten)]
    pub base: SchedulingParams,

    /// Whether to store results in the database
    #[serde(default)]
    pub store_results: bool,

    /// Optional schedule name for storage
    #[serde(default)]
    pub schedule_name: Option<String>,
}

impl Default for TsiSchedulingParams {
    fn default() -> Self {
        Self {
            base: SchedulingParams::default(),
            store_results: false,
            schedule_name: None,
        }
    }
}

impl From<TsiSchedulingParams> for SchedulingParams {
    fn from(p: TsiSchedulingParams) -> Self {
        p.base
    }
}

/// Run scheduling from a JSON configuration string
///
/// This is a convenience function that:
/// 1. Parses the configuration and blocks from JSON
/// 2. Runs the prescheduler to compute possible periods
/// 3. Runs the scheduling algorithm
/// 4. Returns the results as JSON
///
/// # Arguments
///
/// * `config_json` - JSON string containing instrument config, execution period, and blocks
/// * `params` - Scheduling parameters
///
/// # Returns
///
/// JSON string with scheduling results
pub fn schedule_from_json(config_json: &str, params: SchedulingParams) -> anyhow::Result<String> {
    run_full_pipeline(config_json, params).map_err(|e| anyhow::anyhow!("Scheduling failed: {}", e))
}

/// Run scheduling from a file path
///
/// # Arguments
///
/// * `file_path` - Path to JSON file with configuration and blocks
/// * `params` - Scheduling parameters
///
/// # Returns
///
/// JSON string with scheduling results
pub fn schedule_from_file(file_path: &str, params: SchedulingParams) -> anyhow::Result<String> {
    run_pipeline_from_file(file_path, params)
        .map_err(|e| anyhow::anyhow!("Scheduling failed: {}", e))
}

/// Run scheduling with full control over the process
///
/// This function provides fine-grained control over the scheduling pipeline:
/// 1. Create context from configuration
/// 2. Load blocks
/// 3. Optionally compute possible periods
/// 4. Run scheduler
/// 5. Return typed results
///
/// # Arguments
///
/// * `config_json` - JSON string with instrument and execution period
/// * `blocks_json` - JSON string with scheduling blocks
/// * `params` - Scheduling parameters
///
/// # Returns
///
/// Schedule statistics
pub fn schedule_with_control(
    config_json: &str,
    blocks_json: &str,
    params: SchedulingParams,
) -> anyhow::Result<ScheduleStats> {
    let ctx =
        Context::from_json(config_json).map_err(|e| anyhow::anyhow!("Invalid config: {}", e))?;

    let blocks =
        Blocks::from_json(blocks_json).map_err(|e| anyhow::anyhow!("Invalid blocks: {}", e))?;

    // Compute possible periods
    let periods = compute_possible_periods(&ctx, &blocks)
        .map_err(|e| anyhow::anyhow!("Prescheduler failed: {}", e))?;

    // Run scheduler
    let schedule = run_scheduler(&ctx, &blocks, Some(&periods), params)
        .map_err(|e| anyhow::anyhow!("Scheduler failed: {}", e))?;

    schedule
        .stats()
        .map_err(|e| anyhow::anyhow!("Failed to get stats: {}", e))
}
