"""Custom exception classes for TSI application.

This module defines application-specific exceptions for better error handling.
Since Python handles only the UI layer, we keep the exception hierarchy minimal.

Security Note:
Backend errors contain detailed internal information that should NOT be displayed
directly to end users. Use the `to_user_message()` method or the helper functions
from `tsi.utils.error_display` for consistent, sanitized error display in the UI.

Exception Hierarchy:
    TSIError (base)
    ├── ServerError (all backend/database/service errors)
    ├── DataError (data validation/loading issues)
    └── ConfigurationError (UI configuration issues)

Usage:
    >>> from tsi.exceptions import ServerError
    >>> from tsi.utils.error_display import display_error
    >>>
    >>> try:
    ...     # Backend operation
    ...     store_schedule_db(name, data)
    ... except Exception as e:
    ...     # Raise with detailed info for logs, generic message for users
    ...     raise ServerError(
    ...         "Failed to initialize database connection pool",
    ...         details={"error": str(e)}
    ...     ) from e
    >>>
    >>> # In UI code:
    >>> try:
    ...     some_backend_operation()
    ... except ServerError as e:
    ...     display_error(e)  # Shows generic message to user, logs details
"""


class TSIError(Exception):
    """Base exception for all TSI application errors."""

    def __init__(self, message: str, details: dict | None = None, user_message: str | None = None):
        """
        Initialize TSI error.

        Args:
            message: Detailed error message for logging
            details: Optional dictionary with additional error context
            user_message: Optional user-friendly message to display in UI (hides implementation details)
        """
        super().__init__(message)
        self.message = message
        self.details = details or {}
        self._user_message = user_message

    def __str__(self) -> str:
        """Return string representation with details if available."""
        if self.details:
            details_str = ", ".join(f"{k}={v}" for k, v in self.details.items())
            return f"{self.message} ({details_str})"
        return self.message

    def to_user_message(self) -> str:
        """Return a user-friendly error message, hiding sensitive implementation details.

        This method should be used when displaying errors in the UI to avoid
        exposing database schemas, connection strings, or internal system details.

        Returns:
            User-friendly error message
        """
        if self._user_message:
            return self._user_message
        return self.message


# ===== Configuration Errors =====


class ConfigurationError(TSIError):
    """Raised when application configuration is invalid or missing."""

    pass


# ===== Server/Backend Errors =====


class ServerError(TSIError):
    """Raised when any backend/database/service operation fails.

    This is a catch-all for all backend errors (database connections, queries,
    Rust backend issues, etc.) since Python is just the UI layer.

    Note: Detailed error information is logged but a generic message is shown to users.
    """

    def __init__(self, message: str, details: dict | None = None, user_message: str | None = None):
        if user_message is None:
            user_message = "A server error occurred. Please try again later."
        super().__init__(message, details, user_message)


# ===== Data Errors =====


class DataError(TSIError):
    """Raised when data validation or loading fails in the UI layer."""

    pass


# ===== Timeout and Retry Errors =====


class OperationTimeoutError(TSIError):
    """Raised when an operation exceeds its allowed time limit."""

    def __init__(self, message: str, details: dict | None = None, user_message: str | None = None):
        if user_message is None:
            user_message = "The operation took too long and timed out. Please try again."
        super().__init__(message, details, user_message)


class RetryExhaustedError(TSIError):
    """Raised when retry attempts are exhausted for an operation."""

    def __init__(self, message: str, details: dict | None = None, user_message: str | None = None):
        if user_message is None:
            user_message = "The operation failed after multiple attempts. Please try again later."
        super().__init__(message, details, user_message)


__all__ = [
    "TSIError",
    "ServerError",
    "ConfigurationError",
    "DataError",
    "OperationTimeoutError",
    "RetryExhaustedError",
]
