"""Error handling utilities and retry logic.

This module provides utilities for robust error handling, including:
- Retry decorators with exponential backoff
- Context managers for error handling
- Transient error detection
- Logging helpers

Example:
    >>> from tsi.error_handling import with_retry, is_transient_error
    >>> 
    >>> @with_retry(max_attempts=3, backoff_factor=1.5)
    >>> def fetch_data():
    ...     # Operation that might fail transiently
    ...     pass
"""

import functools
import logging
import time
from typing import Any, Callable, TypeVar, ParamSpec

from tsi.exceptions import (
    DatabaseTimeoutError,
    DatabaseConnectionError,
    RetryExhaustedError,
    OperationTimeoutError,
)

logger = logging.getLogger(__name__)

# Type variables for generic decorators
P = ParamSpec('P')
R = TypeVar('R')


def is_transient_error(error: Exception) -> bool:
    """
    Determine if an error is transient and worth retrying.
    
    Transient errors include:
    - Connection errors
    - Timeout errors
    - Temporary network issues
    
    Args:
        error: Exception to check
        
    Returns:
        True if error is transient, False otherwise
    """
    # Check custom exception types
    if isinstance(error, (DatabaseConnectionError, DatabaseTimeoutError, OperationTimeoutError)):
        return True
    
    # Check error message for known transient patterns
    error_msg = str(error).lower()
    transient_patterns = [
        "timeout",
        "connection",
        "network",
        "temporary",
        "unavailable",
        "too many connections",
        "connection reset",
        "broken pipe",
        "connection refused",
        "no route to host",
    ]
    
    return any(pattern in error_msg for pattern in transient_patterns)


def with_retry(
    max_attempts: int = 3,
    backoff_factor: float = 1.5,
    initial_delay: float = 0.5,
    max_delay: float = 10.0,
    retry_on: tuple[type[Exception], ...] | None = None,
    log_attempts: bool = True,
) -> Callable[[Callable[P, R]], Callable[P, R]]:
    """
    Decorator to retry a function on transient errors with exponential backoff.
    
    Args:
        max_attempts: Maximum number of attempts (including initial)
        backoff_factor: Multiplier for delay between attempts
        initial_delay: Initial delay in seconds
        max_delay: Maximum delay between attempts in seconds
        retry_on: Tuple of exception types to retry on (None = all transient errors)
        log_attempts: Whether to log retry attempts
        
    Returns:
        Decorated function with retry logic
        
    Example:
        >>> @with_retry(max_attempts=3, backoff_factor=2.0)
        >>> def fetch_schedule():
        ...     return database.fetch_schedule(1)
    """
    def decorator(func: Callable[P, R]) -> Callable[P, R]:
        @functools.wraps(func)
        def wrapper(*args: P.args, **kwargs: P.kwargs) -> R:
            delay = initial_delay
            last_exception = None
            
            for attempt in range(1, max_attempts + 1):
                try:
                    return func(*args, **kwargs)
                except Exception as e:
                    last_exception = e
                    
                    # Check if we should retry this error
                    should_retry = (
                        retry_on is None and is_transient_error(e)
                    ) or (
                        retry_on is not None and isinstance(e, retry_on)
                    )
                    
                    if not should_retry or attempt >= max_attempts:
                        # Don't retry or exhausted attempts
                        if log_attempts and attempt > 1:
                            func_name = getattr(func, '__name__', 'unknown')
                            logger.warning(
                                f"{func_name} failed after {attempt} attempt(s): {e}"
                            )
                        raise
                    
                    # Log retry attempt
                    if log_attempts:
                        func_name = getattr(func, '__name__', 'unknown')
                        logger.info(
                            f"{func_name} attempt {attempt}/{max_attempts} failed: {e}. "
                            f"Retrying in {delay:.1f}s..."
                        )
                    
                    # Wait before retrying
                    time.sleep(delay)
                    
                    # Exponential backoff with cap
                    delay = min(delay * backoff_factor, max_delay)
            
            # Should never reach here, but just in case
            func_name = getattr(func, '__name__', 'unknown')
            raise RetryExhaustedError(
                f"All {max_attempts} retry attempts exhausted",
                details={"function": func_name, "last_error": str(last_exception)}
            ) from last_exception
        
        return wrapper
    return decorator


def log_error(
    error: Exception,
    context: str,
    include_traceback: bool = False,
    extra: dict[str, Any] | None = None,
) -> None:
    """
    Log an error with context and optional details.
    
    Args:
        error: Exception to log
        context: Context description (e.g., "database query", "file load")
        include_traceback: Whether to include full traceback
        extra: Additional context to include in log
    """
    error_type = type(error).__name__
    message = f"{context}: {error_type} - {error}"
    
    if extra:
        extra_str = ", ".join(f"{k}={v}" for k, v in extra.items())
        message = f"{message} ({extra_str})"
    
    if include_traceback:
        logger.exception(message)
    else:
        logger.error(message)


def safe_execute(
    func: Callable[..., R],
    *args: Any,
    default: R | None = None,
    error_context: str = "",
    log_errors: bool = True,
    **kwargs: Any,
) -> R | None:
    """
    Execute a function safely, returning a default value on error.
    
    Useful for optional operations where failure shouldn't crash the application.
    
    Args:
        func: Function to execute
        *args: Positional arguments for func
        default: Default value to return on error
        error_context: Context description for logging
        log_errors: Whether to log errors
        **kwargs: Keyword arguments for func
        
    Returns:
        Function result or default value on error
        
    Example:
        >>> result = safe_execute(
        ...     fetch_optional_data,
        ...     schedule_id=1,
        ...     default=[],
        ...     error_context="fetching optional metadata"
        ... )
    """
    try:
        return func(*args, **kwargs)
    except Exception as e:
        if log_errors:
            func_name = getattr(func, '__name__', 'unknown')
            context = error_context or f"executing {func_name}"
            log_error(e, context)
        return default


class ErrorContext:
    """
    Context manager for handling errors with automatic logging.
    
    Example:
        >>> with ErrorContext("database query", reraise=True):
        ...     result = database.query(...)
    """
    
    def __init__(
        self,
        context: str,
        reraise: bool = True,
        log_traceback: bool = False,
        default_value: Any = None,
    ):
        """
        Initialize error context.
        
        Args:
            context: Description of the operation
            reraise: Whether to re-raise exceptions
            log_traceback: Whether to log full traceback
            default_value: Value to return on error if not reraising
        """
        self.context = context
        self.reraise = reraise
        self.log_traceback = log_traceback
        self.default_value = default_value
        self.error: Exception | None = None
    
    def __enter__(self):
        """Enter context manager."""
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Exit context manager and handle any exception."""
        if exc_val is not None:
            self.error = exc_val
            log_error(exc_val, self.context, include_traceback=self.log_traceback)
            
            if not self.reraise:
                # Suppress exception and return False to indicate no error
                return True
        
        return False
    
    def get_value(self, success_value: Any = None) -> Any:
        """
        Get the value to use based on whether an error occurred.
        
        Args:
            success_value: Value to return if no error
            
        Returns:
            success_value if no error, default_value otherwise
        """
        if self.error is not None:
            return self.default_value
        return success_value


__all__ = [
    "is_transient_error",
    "with_retry",
    "log_error",
    "safe_execute",
    "ErrorContext",
]
