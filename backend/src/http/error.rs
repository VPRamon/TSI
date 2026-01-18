//! HTTP error handling and response types.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

/// API error response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Application error type for HTTP handlers.
#[derive(Debug)]
pub enum AppError {
    /// Resource not found
    NotFound(String),
    /// Invalid request (validation error)
    BadRequest(String),
    /// Internal server error
    Internal(String),
    /// Repository error
    Repository(crate::db::repository::RepositoryError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ApiError::new("NOT_FOUND", msg),
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                ApiError::new("BAD_REQUEST", msg),
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::new("INTERNAL_ERROR", msg),
            ),
            AppError::Repository(e) => {
                let msg = e.to_string();
                // Check if it's a "not found" type error
                if msg.to_lowercase().contains("not found") {
                    (StatusCode::NOT_FOUND, ApiError::new("NOT_FOUND", msg))
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, ApiError::new("REPOSITORY_ERROR", msg))
                }
            }
        };

        (status, Json(error)).into_response()
    }
}

impl From<crate::db::repository::RepositoryError> for AppError {
    fn from(err: crate::db::repository::RepositoryError) -> Self {
        AppError::Repository(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}
