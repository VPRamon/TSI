//! Custom error types for the TSI backend
//! 
//! This module provides a unified error handling approach using thiserror
//! for domain-specific errors that can be easily converted to HTTP responses.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Response structure for API errors
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Main error type for the backend
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("Dataset not found: {0}")]
    DatasetNotFound(String),

    #[error("Comparison dataset not found")]
    ComparisonDatasetNotFound,

    #[error("Invalid CSV format: {0}")]
    InvalidCsvFormat(String),

    #[error("Invalid JSON format: {0}")]
    InvalidJsonFormat(String),

    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    #[error("Invalid column: {0}")]
    InvalidColumn(String),

    #[error("State lock error: {0}")]
    StateLockError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("File upload error: {0}")]
    FileUploadError(String),

    #[error("File too large: {size} bytes exceeds limit of {limit} bytes")]
    FileTooLarge { size: usize, limit: usize },

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Polars error: {0}")]
    PolarsError(String),

    #[error("Anyhow error: {0}")]
    AnyhowError(String),
}

// Implement From traits for common error types
impl From<std::io::Error> for BackendError {
    fn from(err: std::io::Error) -> Self {
        BackendError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for BackendError {
    fn from(err: serde_json::Error) -> Self {
        BackendError::SerializationError(err.to_string())
    }
}

impl From<polars::error::PolarsError> for BackendError {
    fn from(err: polars::error::PolarsError) -> Self {
        BackendError::PolarsError(err.to_string())
    }
}

impl From<anyhow::Error> for BackendError {
    fn from(err: anyhow::Error) -> Self {
        BackendError::AnyhowError(err.to_string())
    }
}

// Convert BackendError into HTTP responses
impl IntoResponse for BackendError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match &self {
            BackendError::DatasetNotFound(msg) => {
                (StatusCode::NOT_FOUND, "No dataset loaded".to_string(), Some(msg.clone()))
            }
            BackendError::ComparisonDatasetNotFound => {
                (StatusCode::NOT_FOUND, "No comparison dataset loaded".to_string(), None)
            }
            BackendError::InvalidCsvFormat(msg) => {
                (StatusCode::BAD_REQUEST, "Invalid CSV format".to_string(), Some(msg.clone()))
            }
            BackendError::InvalidJsonFormat(msg) => {
                (StatusCode::BAD_REQUEST, "Invalid JSON format".to_string(), Some(msg.clone()))
            }
            BackendError::MissingRequiredField(field) => {
                (StatusCode::BAD_REQUEST, "Missing required field".to_string(), Some(field.clone()))
            }
            BackendError::InvalidColumn(col) => {
                (StatusCode::BAD_REQUEST, "Invalid column".to_string(), Some(col.clone()))
            }
            BackendError::StateLockError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "State lock error".to_string(), Some(msg.clone()))
            }
            BackendError::SerializationError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Serialization error".to_string(), Some(msg.clone()))
            }
            BackendError::FileUploadError(msg) => {
                (StatusCode::BAD_REQUEST, "File upload error".to_string(), Some(msg.clone()))
            }
            BackendError::FileTooLarge { size, limit } => {
                (
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "File too large".to_string(),
                    Some(format!("{} bytes exceeds limit of {} bytes", size, limit)),
                )
            }
            BackendError::InvalidParameter(msg) => {
                (StatusCode::BAD_REQUEST, "Invalid parameter".to_string(), Some(msg.clone()))
            }
            BackendError::IoError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "IO error".to_string(), Some(msg.clone()))
            }
            BackendError::PolarsError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Data processing error".to_string(), Some(msg.clone()))
            }
            BackendError::AnyhowError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string(), Some(msg.clone()))
            }
        };

        let body = Json(ErrorResponse {
            error: error_message,
            details,
        });

        (status, body).into_response()
    }
}

/// Result type alias using BackendError
pub type Result<T> = std::result::Result<T, BackendError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BackendError::DatasetNotFound("test.csv".to_string());
        assert_eq!(err.to_string(), "Dataset not found: test.csv");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let backend_err: BackendError = io_err.into();
        assert!(matches!(backend_err, BackendError::IoError(_)));
    }

    #[test]
    fn test_file_too_large_error() {
        let err = BackendError::FileTooLarge {
            size: 150_000_000,
            limit: 100_000_000,
        };
        assert!(err.to_string().contains("150000000 bytes"));
        assert!(err.to_string().contains("100000000 bytes"));
    }
}
