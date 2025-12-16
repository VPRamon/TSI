//! Error types for repository operations.

/// Result type for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Error type for repository operations
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Data validation error: {0}")]
    ValidationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<String> for RepositoryError {
    fn from(s: String) -> Self {
        RepositoryError::InternalError(s)
    }
}

impl From<&str> for RepositoryError {
    fn from(s: &str) -> Self {
        RepositoryError::InternalError(s.to_string())
    }
}
