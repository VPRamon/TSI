//! Error types for stars-core

use stars_core_sys as ffi;
use thiserror::Error;

/// Result type for stars-core operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using stars-core
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
    pub(crate) fn from_ffi_result(result: &ffi::StarsResult) -> Self {
        let message = if result.error_message.is_null() {
            String::new()
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(result.error_message)
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

    /// Create an Error from the last FFI error
    pub(crate) fn from_last_error(default_msg: &str) -> Self {
        let msg = crate::last_error().unwrap_or_else(|| default_msg.to_string());
        Error::Unknown(msg)
    }
}
