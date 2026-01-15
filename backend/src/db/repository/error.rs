//! Error types for repository operations.
//!
//! This module provides comprehensive error handling for all repository operations
//! with structured context for debugging and monitoring.

use std::fmt;

/// Result type for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Structured context for repository errors.
///
/// Provides additional information about where and why an error occurred.
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    /// The operation being performed (e.g., "store_schedule", "fetch_blocks")
    pub operation: Option<String>,
    /// The entity type involved (e.g., "schedule", "block", "analytics")
    pub entity: Option<String>,
    /// The entity ID if applicable
    pub entity_id: Option<String>,
    /// Additional details about the error
    pub details: Option<String>,
    /// Whether this error is retryable
    pub retryable: bool,
}

impl ErrorContext {
    /// Create a new error context with an operation name.
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: Some(operation.into()),
            ..Default::default()
        }
    }

    /// Set the entity type.
    pub fn with_entity(mut self, entity: impl Into<String>) -> Self {
        self.entity = Some(entity.into());
        self
    }

    /// Set the entity ID.
    pub fn with_entity_id(mut self, id: impl ToString) -> Self {
        self.entity_id = Some(id.to_string());
        self
    }

    /// Set additional details.
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Mark this error as retryable.
    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(ref op) = self.operation {
            parts.push(format!("operation={}", op));
        }
        if let Some(ref entity) = self.entity {
            parts.push(format!("entity={}", entity));
        }
        if let Some(ref id) = self.entity_id {
            parts.push(format!("id={}", id));
        }
        if let Some(ref details) = self.details {
            parts.push(format!("details={}", details));
        }
        if self.retryable {
            parts.push("retryable=true".to_string());
        }
        write!(f, "[{}]", parts.join(", "))
    }
}

/// Error type for repository operations
#[derive(Debug, thiserror::Error)]
#[allow(clippy::result_large_err)]
pub enum RepositoryError {
    /// Connection pool or database connection errors.
    /// These are typically transient and may be retried.
    #[error("Connection error: {message} {context}")]
    ConnectionError {
        message: String,
        context: ErrorContext,
    },

    /// SQL query execution errors.
    #[error("Query error: {message} {context}")]
    QueryError {
        message: String,
        context: ErrorContext,
    },

    /// Requested entity was not found.
    #[error("Not found: {message} {context}")]
    NotFound {
        message: String,
        context: ErrorContext,
    },

    /// Data validation failed before or after database operation.
    #[error("Data validation error: {message} {context}")]
    ValidationError {
        message: String,
        context: ErrorContext,
    },

    /// Configuration or initialization error.
    #[error("Configuration error: {message} {context}")]
    ConfigurationError {
        message: String,
        context: ErrorContext,
    },

    /// Internal/unexpected errors.
    #[error("Internal error: {message} {context}")]
    InternalError {
        message: String,
        context: ErrorContext,
    },

    /// Transaction error (commit/rollback failed).
    #[error("Transaction error: {message} {context}")]
    TransactionError {
        message: String,
        context: ErrorContext,
    },

    /// Timeout waiting for connection or query.
    #[error("Timeout error: {message} {context}")]
    TimeoutError {
        message: String,
        context: ErrorContext,
    },
}

impl RepositoryError {
    /// Create a connection error with context.
    pub fn connection(message: impl Into<String>) -> Self {
        Self::ConnectionError {
            message: message.into(),
            context: ErrorContext::default().retryable(),
        }
    }

    /// Create a connection error with full context.
    pub fn connection_with_context(message: impl Into<String>, context: ErrorContext) -> Self {
        Self::ConnectionError {
            message: message.into(),
            context: context.retryable(),
        }
    }

    /// Create a query error.
    pub fn query(message: impl Into<String>) -> Self {
        Self::QueryError {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a query error with context.
    pub fn query_with_context(message: impl Into<String>, context: ErrorContext) -> Self {
        Self::QueryError {
            message: message.into(),
            context,
        }
    }

    /// Create a not found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a not found error with context.
    pub fn not_found_with_context(message: impl Into<String>, context: ErrorContext) -> Self {
        Self::NotFound {
            message: message.into(),
            context,
        }
    }

    /// Create a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a validation error with context.
    pub fn validation_with_context(message: impl Into<String>, context: ErrorContext) -> Self {
        Self::ValidationError {
            message: message.into(),
            context,
        }
    }

    /// Create a configuration error.
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::ConfigurationError {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a configuration error with context.
    pub fn configuration_with_context(message: impl Into<String>, context: ErrorContext) -> Self {
        Self::ConfigurationError {
            message: message.into(),
            context,
        }
    }

    /// Create an internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create an internal error with context.
    pub fn internal_with_context(message: impl Into<String>, context: ErrorContext) -> Self {
        Self::InternalError {
            message: message.into(),
            context,
        }
    }

    /// Create a transaction error.
    pub fn transaction(message: impl Into<String>) -> Self {
        Self::TransactionError {
            message: message.into(),
            context: ErrorContext::default(),
        }
    }

    /// Create a timeout error.
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::TimeoutError {
            message: message.into(),
            context: ErrorContext::default().retryable(),
        }
    }

    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::ConnectionError { context, .. } => context.retryable,
            Self::TimeoutError { context, .. } => context.retryable,
            Self::QueryError { context, .. } => context.retryable,
            Self::TransactionError { context, .. } => context.retryable,
            _ => false,
        }
    }

    /// Get the error context.
    pub fn context(&self) -> &ErrorContext {
        match self {
            Self::ConnectionError { context, .. } => context,
            Self::QueryError { context, .. } => context,
            Self::NotFound { context, .. } => context,
            Self::ValidationError { context, .. } => context,
            Self::ConfigurationError { context, .. } => context,
            Self::InternalError { context, .. } => context,
            Self::TransactionError { context, .. } => context,
            Self::TimeoutError { context, .. } => context,
        }
    }

    /// Add or update the operation in the error context.
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        match &mut self {
            Self::ConnectionError { context, .. }
            | Self::QueryError { context, .. }
            | Self::NotFound { context, .. }
            | Self::ValidationError { context, .. }
            | Self::ConfigurationError { context, .. }
            | Self::InternalError { context, .. }
            | Self::TransactionError { context, .. }
            | Self::TimeoutError { context, .. } => {
                context.operation = Some(operation.into());
            }
        }
        self
    }
}

// Backward compatibility: allow creating errors from strings
impl From<String> for RepositoryError {
    fn from(s: String) -> Self {
        RepositoryError::internal(s)
    }
}

impl From<&str> for RepositoryError {
    fn from(s: &str) -> Self {
        RepositoryError::internal(s.to_string())
    }
}

// Backward compatibility constructors for the old enum-style API.
// These allow existing code to continue working without changes.
impl RepositoryError {
    /// Legacy constructor for ConnectionError variant.
    #[allow(non_snake_case)]
    pub fn ConnectionError(message: String) -> Self {
        Self::connection(message)
    }

    /// Legacy constructor for QueryError variant.
    #[allow(non_snake_case)]
    pub fn QueryError(message: String) -> Self {
        Self::query(message)
    }

    /// Legacy constructor for NotFound variant.
    #[allow(non_snake_case)]
    pub fn NotFound(message: String) -> Self {
        Self::not_found(message)
    }

    /// Legacy constructor for ValidationError variant.
    #[allow(non_snake_case)]
    pub fn ValidationError(message: String) -> Self {
        Self::validation(message)
    }

    /// Legacy constructor for ConfigurationError variant.
    #[allow(non_snake_case)]
    pub fn ConfigurationError(message: String) -> Self {
        Self::configuration(message)
    }

    /// Legacy constructor for InternalError variant.
    #[allow(non_snake_case)]
    pub fn InternalError(message: String) -> Self {
        Self::internal(message)
    }
}

#[cfg(feature = "postgres-repo")]
impl From<diesel::result::Error> for RepositoryError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => RepositoryError::not_found("Record not found"),
            diesel::result::Error::DatabaseError(kind, info) => {
                let message = info.message().to_string();
                let context =
                    ErrorContext::default().with_details(format!("db_error_kind={:?}", kind));

                // Some database errors are retryable (deadlocks, serialization failures)
                let is_retryable = matches!(
                    kind,
                    diesel::result::DatabaseErrorKind::SerializationFailure
                );

                let context = if is_retryable {
                    context.retryable()
                } else {
                    context
                };

                RepositoryError::QueryError { message, context }
            }
            diesel::result::Error::QueryBuilderError(e) => {
                RepositoryError::query(format!("Query builder error: {}", e))
            }
            diesel::result::Error::DeserializationError(e) => {
                RepositoryError::internal(format!("Deserialization error: {}", e))
            }
            diesel::result::Error::SerializationError(e) => {
                RepositoryError::internal(format!("Serialization error: {}", e))
            }
            other => RepositoryError::query(other.to_string()),
        }
    }
}

#[cfg(feature = "postgres-repo")]
impl From<diesel::r2d2::PoolError> for RepositoryError {
    fn from(err: diesel::r2d2::PoolError) -> Self {
        RepositoryError::connection_with_context(
            err.to_string(),
            ErrorContext::default()
                .with_details("pool_error")
                .retryable(),
        )
    }
}
