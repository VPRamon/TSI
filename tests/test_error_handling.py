"""
Tests for custom exceptions and error handling utilities.
"""

import time
from unittest.mock import Mock, patch

import pytest

from tsi.exceptions import (
    TSIError,
    ConfigurationError,
    ServerError,
    ServerError,
    ServerError,
    ServerError,
    ServerError,
    ServerError,
    ServerError,
    DataError,
    DataValidationError,
    ServerError,
    SchemaError,
    OperationError,
    OperationTimeoutError,
    RetryExhaustedError,
)
from tsi.error_handling import (
    is_transient_error,
    with_retry,
    log_error,
    safe_execute,
    ErrorContext,
)


class TestExceptionHierarchy:
    """Test custom exception classes and hierarchy."""

    def test_tsi_error_base(self):
        """Test TSIError base exception."""
        error = TSIError("Test error", details={"key": "value"})
        assert str(error) == "Test error (key=value)"
        assert error.message == "Test error"
        assert error.details == {"key": "value"}

    def test_tsi_error_no_details(self):
        """Test TSIError without details."""
        error = TSIError("Test error")
        assert str(error) == "Test error"
        assert error.details == {}

    def test_configuration_error(self):
        """Test ConfigurationError."""
        error = ConfigurationError("Invalid config")
        assert isinstance(error, TSIError)
        assert str(error) == "Invalid config"

    def test_database_error_hierarchy(self):
        """Test database error hierarchy."""
        conn_error = ServerError("Connection failed")
        query_error = ServerError("Query failed")
        timeout_error = ServerError("Timeout")

        assert isinstance(conn_error, ServerError)
        assert isinstance(conn_error, TSIError)
        assert isinstance(query_error, ServerError)
        assert isinstance(timeout_error, ServerError)

    def test_backend_error_hierarchy(self):
        """Test backend error hierarchy."""
        rust_error = ServerError("Rust error")
        unavailable_error = ServerError("Backend unavailable")

        assert isinstance(rust_error, ServerError)
        assert isinstance(unavailable_error, ServerError)

    def test_data_error_hierarchy(self):
        """Test data error hierarchy."""
        validation_error = DataValidationError("Validation failed")
        load_error = ServerError("Load failed")
        schema_error = SchemaError("Schema invalid")

        assert isinstance(validation_error, DataError)
        assert isinstance(load_error, DataError)
        assert isinstance(schema_error, DataError)

    def test_operation_error_hierarchy(self):
        """Test operation error hierarchy."""
        timeout_error = OperationTimeoutError("Operation timed out")
        retry_error = RetryExhaustedError("Retries exhausted")

        assert isinstance(timeout_error, OperationError)
        assert isinstance(retry_error, OperationError)


class TestIsTransientError:
    """Test transient error detection."""

    def test_detects_database_connection_error(self):
        """Test that database connection errors are transient."""
        error = ServerError("Connection failed")
        assert is_transient_error(error) is True

    def test_detects_database_timeout_error(self):
        """Test that database timeout errors are transient."""
        error = ServerError("Query timed out")
        assert is_transient_error(error) is True

    def test_detects_operation_timeout_error(self):
        """Test that operation timeout errors are transient."""
        error = OperationTimeoutError("Operation timed out")
        assert is_transient_error(error) is True

    def test_detects_connection_in_message(self):
        """Test detection based on error message patterns."""
        error = Exception("Connection reset by peer")
        assert is_transient_error(error) is True

    def test_detects_timeout_in_message(self):
        """Test detection of timeout in message."""
        error = Exception("Operation timeout exceeded")
        assert is_transient_error(error) is True

    def test_non_transient_error(self):
        """Test that non-transient errors are not detected."""
        error = DataValidationError("Invalid data")
        assert is_transient_error(error) is False

    def test_generic_error_non_transient(self):
        """Test that generic errors without transient patterns are not transient."""
        error = Exception("Some other error")
        assert is_transient_error(error) is False


class TestWithRetryDecorator:
    """Test retry decorator functionality."""

    def test_successful_first_attempt(self):
        """Test that successful operations don't retry."""
        mock_func = Mock(return_value="success")
        decorated = with_retry(max_attempts=3)(mock_func)

        result = decorated()

        assert result == "success"
        assert mock_func.call_count == 1

    def test_retries_on_transient_error(self):
        """Test that transient errors trigger retries."""
        mock_func = Mock(side_effect=[
            ServerError("Connection failed"),
            ServerError("Connection failed"),
            "success"
        ])
        decorated = with_retry(max_attempts=3, initial_delay=0.01)(mock_func)

        result = decorated()

        assert result == "success"
        assert mock_func.call_count == 3

    def test_exhausts_retries(self):
        """Test that retry exhaustion raises the last error."""
        error = ServerError("Connection failed")
        mock_func = Mock(side_effect=error)
        decorated = with_retry(max_attempts=3, initial_delay=0.01)(mock_func)

        with pytest.raises(ServerError):
            decorated()

        assert mock_func.call_count == 3

    def test_non_transient_error_no_retry(self):
        """Test that non-transient errors don't trigger retries."""
        error = DataValidationError("Invalid data")
        mock_func = Mock(side_effect=error)
        decorated = with_retry(max_attempts=3)(mock_func)

        with pytest.raises(DataValidationError):
            decorated()

        assert mock_func.call_count == 1

    def test_retry_with_specific_exceptions(self):
        """Test retry_on parameter for specific exception types."""
        mock_func = Mock(side_effect=[
            ValueError("Error 1"),
            ValueError("Error 2"),
            "success"
        ])
        decorated = with_retry(
            max_attempts=3,
            retry_on=(ValueError,),
            initial_delay=0.01
        )(mock_func)

        result = decorated()

        assert result == "success"
        assert mock_func.call_count == 3

    def test_exponential_backoff(self):
        """Test that retry delays increase exponentially."""
        mock_func = Mock(side_effect=[
            ServerError("Error"),
            ServerError("Error"),
            "success"
        ])
        
        start_time = time.time()
        decorated = with_retry(
            max_attempts=3,
            initial_delay=0.1,
            backoff_factor=2.0
        )(mock_func)
        result = decorated()
        elapsed = time.time() - start_time

        assert result == "success"
        # Should wait ~0.1s + ~0.2s = ~0.3s total
        assert elapsed >= 0.3


class TestSafeExecute:
    """Test safe_execute utility function."""

    def test_successful_execution(self):
        """Test successful function execution."""
        mock_func = Mock(return_value="success")
        result = safe_execute(mock_func, arg1="value")

        assert result == "success"
        mock_func.assert_called_once_with(arg1="value")

    def test_returns_default_on_error(self):
        """Test that default value is returned on error."""
        mock_func = Mock(side_effect=Exception("Error"))
        result = safe_execute(mock_func, default="default_value", log_errors=False)

        assert result == "default_value"

    def test_returns_none_default(self):
        """Test that None is returned as default if not specified."""
        mock_func = Mock(side_effect=Exception("Error"))
        result = safe_execute(mock_func, log_errors=False)

        assert result is None

    def test_logs_errors_by_default(self):
        """Test that errors are logged by default."""
        mock_func = Mock(side_effect=Exception("Test error"))

        with patch('tsi.error_handling.log_error') as mock_log:
            safe_execute(mock_func, error_context="test operation")
            mock_log.assert_called_once()

    def test_suppresses_logging_when_disabled(self):
        """Test that logging can be disabled."""
        mock_func = Mock(side_effect=Exception("Test error"))

        with patch('tsi.error_handling.log_error') as mock_log:
            safe_execute(mock_func, log_errors=False)
            mock_log.assert_not_called()


class TestLogError:
    """Test log_error utility function."""

    def test_logs_error_without_traceback(self):
        """Test basic error logging."""
        error = ValueError("Test error")

        with patch('tsi.error_handling.logger') as mock_logger:
            log_error(error, "test operation", include_traceback=False)
            mock_logger.error.assert_called_once()

    def test_logs_error_with_traceback(self):
        """Test error logging with traceback."""
        error = ValueError("Test error")

        with patch('tsi.error_handling.logger') as mock_logger:
            log_error(error, "test operation", include_traceback=True)
            mock_logger.exception.assert_called_once()

    def test_includes_extra_context(self):
        """Test that extra context is included in log message."""
        error = ValueError("Test error")
        extra = {"key1": "value1", "key2": "value2"}

        with patch('tsi.error_handling.logger') as mock_logger:
            log_error(error, "test operation", extra=extra)
            
            call_args = mock_logger.error.call_args[0][0]
            assert "key1=value1" in call_args
            assert "key2=value2" in call_args


class TestErrorContext:
    """Test ErrorContext context manager."""

    def test_successful_operation_no_error(self):
        """Test context manager with successful operation."""
        with ErrorContext("test operation") as ctx:
            result = "success"

        assert ctx.error is None
        assert ctx.get_value("success") == "success"

    def test_captures_and_reraises_error(self):
        """Test that errors are captured and re-raised by default."""
        with pytest.raises(ValueError):
            with ErrorContext("test operation") as ctx:
                raise ValueError("Test error")

        assert ctx.error is not None
        assert isinstance(ctx.error, ValueError)

    def test_suppresses_error_when_configured(self):
        """Test that errors can be suppressed."""
        with ErrorContext("test operation", reraise=False) as ctx:
            raise ValueError("Test error")

        assert ctx.error is not None
        assert isinstance(ctx.error, ValueError)

    def test_returns_default_value_on_error(self):
        """Test default value is returned on error."""
        with ErrorContext("test operation", reraise=False, default_value="default") as ctx:
            raise ValueError("Test error")

        assert ctx.get_value("success") == "default"

    def test_logs_error(self):
        """Test that errors are logged."""
        with patch('tsi.error_handling.log_error') as mock_log:
            try:
                with ErrorContext("test operation") as ctx:
                    raise ValueError("Test error")
            except ValueError:
                pass

            mock_log.assert_called_once()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
