//! STARS Context - holds instrument, execution period, and configuration

use crate::error::{Error, Result};
use crate::types::ExecutionPeriod;
use stars_core_sys as ffi;
use std::ffi::CString;
use std::ptr;

/// STARS scheduling context
///
/// Holds the instrument configuration, execution period, and other settings
/// needed for scheduling operations.
///
/// # Example
///
/// ```rust,ignore
/// use stars_core::Context;
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
        let mut out_json: *mut std::os::raw::c_char = ptr::null_mut();

        let result =
            unsafe { ffi::stars_context_get_execution_period(self.handle, &mut out_json) };

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
            let period: ExecutionPeriod = serde_json::from_str(&json_str)?;
            Ok(period)
        } else {
            Err(Error::from_ffi_result(&result))
        }
    }

    /// Get the raw FFI handle (for internal use)
    pub(crate) fn handle(&self) -> ffi::StarsContextHandle {
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
