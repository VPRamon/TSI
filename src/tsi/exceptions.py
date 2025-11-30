"""Custom exception classes for TSI application.

This module defines application-specific exceptions for better error handling
and more informative error messages throughout the codebase.

Exception Hierarchy:
    TSIError (base)
    ├── ConfigurationError
    ├── DatabaseError
    │   ├── DatabaseConnectionError
    │   ├── DatabaseQueryError
    │   └── DatabaseTimeoutError
    ├── BackendError
    │   ├── RustBackendError
    │   └── BackendUnavailableError
    ├── DataError
    │   ├── DataValidationError
    │   ├── DataLoadError
    │   └── SchemaError
    └── OperationError
        ├── OperationTimeoutError
        └── RetryExhaustedError

Usage:
    >>> from tsi.exceptions import DatabaseConnectionError
    >>> try:
    ...     # Database operation
    ...     pass
    ... except Exception as e:
    ...     raise DatabaseConnectionError("Failed to connect to database") from e
"""


class TSIError(Exception):
    """Base exception for all TSI application errors."""

    def __init__(self, message: str, details: dict | None = None):
        """
        Initialize TSI error.
        
        Args:
            message: Human-readable error message
            details: Optional dictionary with additional error context
        """
        super().__init__(message)
        self.message = message
        self.details = details or {}

    def __str__(self) -> str:
        """Return string representation with details if available."""
        if self.details:
            details_str = ", ".join(f"{k}={v}" for k, v in self.details.items())
            return f"{self.message} ({details_str})"
        return self.message


# ===== Configuration Errors =====

class ConfigurationError(TSIError):
    """Raised when application configuration is invalid or missing."""
    pass


# ===== Database Errors =====

class DatabaseError(TSIError):
    """Base exception for database-related errors."""
    pass


class DatabaseConnectionError(DatabaseError):
    """Raised when unable to establish database connection."""
    pass


class DatabaseQueryError(DatabaseError):
    """Raised when a database query fails."""
    pass


class DatabaseTimeoutError(DatabaseError):
    """Raised when a database operation times out."""
    pass


# ===== Backend Errors =====

class BackendError(TSIError):
    """Base exception for backend-related errors."""
    pass


class RustBackendError(BackendError):
    """Raised when Rust backend operation fails."""
    pass


class BackendUnavailableError(BackendError):
    """Raised when the Rust backend is not available or not compiled."""
    pass


# ===== Data Errors =====

class DataError(TSIError):
    """Base exception for data-related errors."""
    pass


class DataValidationError(DataError):
    """Raised when data validation fails."""
    pass


class DataLoadError(DataError):
    """Raised when data loading fails."""
    pass


class SchemaError(DataError):
    """Raised when data schema is invalid or missing required columns."""
    pass


# ===== Operation Errors =====

class OperationError(TSIError):
    """Base exception for operation-related errors."""
    pass


class OperationTimeoutError(OperationError):
    """Raised when an operation times out."""
    pass


class RetryExhaustedError(OperationError):
    """Raised when all retry attempts have been exhausted."""
    pass


__all__ = [
    "TSIError",
    "ConfigurationError",
    "DatabaseError",
    "DatabaseConnectionError",
    "DatabaseQueryError",
    "DatabaseTimeoutError",
    "BackendError",
    "RustBackendError",
    "BackendUnavailableError",
    "DataError",
    "DataValidationError",
    "DataLoadError",
    "SchemaError",
    "OperationError",
    "OperationTimeoutError",
    "RetryExhaustedError",
]
