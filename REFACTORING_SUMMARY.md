# Configuration and Error Handling Refactoring - Summary

## Overview

This refactoring introduces centralized configuration management and robust error handling to the Telescope Scheduling Intelligence (TSI) project. The changes maintain backward compatibility while providing a more maintainable and production-ready codebase.

## 1. Centralized Configuration System

### Design: `app_config/settings.py`

Created a comprehensive `Settings` class using `pydantic-settings` with the following categories:

#### Database Configuration
- `database_url`: Full database connection string
- `db_server`, `db_database`, `db_username`, `db_password`: Individual DB components
- `use_aad_auth`: Azure AD authentication flag
- `database_connection_timeout`: Connection timeout (default: 30s)
- `database_max_retries`: Max retry attempts (default: 3)

#### Data Paths
- `data_root`: Base directory for data files (default: `data`)
- `sample_dataset`: Default sample schedule path (default: `data/schedule.csv`)
- `artifacts_dir`: ML model artifacts directory

#### UI Configuration
- `app_title`: Application title (default: "Telescope Scheduling Intelligence")
- `app_icon`: Application icon emoji (default: "🔭")
- `layout`: Streamlit layout mode (default: "wide")
- `initial_sidebar_state`: Sidebar state (default: "collapsed")
- `pages`: List of available pages

#### Performance Settings
- `cache_ttl`: Cache time-to-live in seconds (default: 3600)
- `max_workers`: Maximum worker threads/processes (default: 4)
- `enable_rust_backend`: Enable Rust backend (default: True)

#### Plot Defaults
- `plot_height`: Default plot height (default: 600px)
- `plot_margin_*`: Plot margins (left, right, top, bottom)

#### Feature Flags
- `enable_database`: Enable database features (default: True)
- `enable_file_upload`: Enable file upload (default: True)
- `enable_comparison`: Enable comparison feature (default: True)

### Key Features

1. **Environment Variable Support**: All settings can be overridden via environment variables
2. **`.env` File Support**: Supports loading from `.env` file
3. **Validation**: Built-in validation for types and constraints
4. **Smart Defaults**: Sensible defaults that match current behavior
5. **Database URL Construction**: Can build connection string from components
6. **Singleton Pattern**: Settings are cached via `@lru_cache` for performance

### Helper Methods

- `get_database_url()`: Returns full connection URL or constructs from components
- `get_plot_margin()`: Returns plot margins as dictionary
- `validate_database_config()`: Validates database configuration completeness

## 2. Custom Exception Hierarchy

### Design: `tsi/exceptions.py`

Created a comprehensive exception hierarchy:

```
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
```

### Benefits

1. **Clear Error Types**: Specific exception types for different failure modes
2. **Error Context**: All exceptions support optional `details` dictionary
3. **Better Logging**: Enhanced string representation with context
4. **Targeted Handling**: Can catch specific error types for different recovery strategies

## 3. Error Handling Utilities

### Design: `tsi/error_handling.py`

Implemented robust error handling utilities:

#### `with_retry` Decorator
- Automatic retry logic with exponential backoff
- Configurable max attempts, backoff factor, delays
- Transient error detection
- Selective retry based on exception types
- Comprehensive logging

Example:
```python
@with_retry(max_attempts=3, backoff_factor=1.5)
def fetch_data():
    # Operation that might fail transiently
    pass
```

#### `is_transient_error` Function
- Detects transient errors worth retrying
- Pattern matching on error messages
- Recognizes network, timeout, connection issues

#### `safe_execute` Function
- Execute operations with default fallback values
- Optional error logging
- Useful for non-critical operations

#### `ErrorContext` Context Manager
- Structured error handling
- Automatic logging
- Optional error suppression
- Default value support

#### `log_error` Helper
- Consistent error logging
- Optional traceback inclusion
- Context enrichment

## 4. Migration of Hardcoded Configuration

### Database Settings
- **Before**: `DATABASE_URL` mentioned in landing.py error message
- **After**: 
  - Centralized in `Settings` class
  - Better validation and construction
  - Clear error messages with configuration hints

### Data Paths
- **Before**: Hardcoded `Path("data")` in multiple locations
- **After**: 
  - `settings.data_root` used throughout
  - Consistent path handling

### UI Configuration
- **Before**: Hardcoded in `theme.py`
  ```python
  page_title="Telescope Scheduling Intelligence"
  page_icon="🔭"
  ```
- **After**:
  ```python
  page_title=settings.app_title
  page_icon=settings.app_icon
  ```

### Performance Settings
- **Before**: Hardcoded cache TTL
  ```python
  @st.cache_data(ttl=3600)
  ```
- **After**:
  ```python
  from tsi.config import CACHE_TTL
  @st.cache_data(ttl=CACHE_TTL)
  ```
- Migrated in:
  - `src/tsi/plots/distributions.py`
  - `src/tsi/plots/sky_map.py`
  - `src/tsi/plots/timeline.py`

### Updated Modules
- `src/tsi/config.py`: Now imports from centralized settings
- `src/tsi/theme.py`: Uses configuration for page config
- `src/tsi/pages/landing.py`: Better error handling with config validation

## 5. Enhanced Error Handling in Database Operations

### `src/tsi/services/database.py`

#### Improvements
1. **Retry Logic**: Database operations auto-retry on transient errors
2. **Custom Exceptions**: Uses `DatabaseConnectionError`, `DatabaseQueryError`
3. **Better Logging**: Contextual logging for all operations
4. **Configuration Integration**: Uses settings for retry configuration

#### Updated Functions
- `init_database()`: Retries on connection failures, logs success
- `db_health_check()`: Proper exception types, retry logic
- `store_schedule_db()`: Retries, detailed error context
- `list_schedules_db()`: Consistent error handling

#### Error Handling Pattern
```python
@with_retry(max_attempts=3, backoff_factor=1.5)
def database_operation():
    try:
        result = _rust_call("operation")
        logger.info("Operation successful")
        return result
    except Exception as e:
        raise DatabaseQueryError(
            "Operation failed",
            details={"context": "value"}
        ) from e
```

### `src/tsi/services/data/loaders.py`

#### Improvements
1. **Custom Exceptions**: `DataLoadError`, `SchemaError`
2. **Retry Logic**: File operations retry on transient errors
3. **Better Error Messages**: Include file path, format, columns

#### Updated Functions
- `load_schedule_rust()`: Retries with detailed error context
- `_load_csv_core()`: Better schema validation error messages

### `src/tsi_rust_api.py`

#### Improvements
1. **Import Error Handling**: Uses `BackendUnavailableError` with install instructions
2. **Clearer Messages**: Suggests `maturin develop --release` command

## 6. Comprehensive Test Suite

### Configuration Tests (`tests/test_config.py`)
- 31 tests covering all configuration aspects
- Tests for defaults, environment variables, validation
- Database URL construction tests
- Path conversion tests
- Constraint validation tests
- **Result**: ✅ All 31 tests pass

### Error Handling Tests (`tests/test_error_handling.py`)
- 33 tests covering exception hierarchy and utilities
- Tests for exception types and inheritance
- Transient error detection tests
- Retry decorator tests (including exponential backoff)
- Safe execution tests
- Error context manager tests
- **Result**: ✅ All 33 tests pass

## 7. Behavior Preservation

### Guarantee of Backward Compatibility

1. **Default Values**: All configuration defaults match previous hardcoded values
2. **No Breaking Changes**: Existing code continues to work
3. **Environment Variables**: Same `DATABASE_URL` variable supported
4. **Error Propagation**: Errors still propagate, just with better context
5. **Performance**: No degradation (retry logic only on failures)

### What Changed (Internally)
- Error types are more specific
- Logging is more comprehensive
- Configuration is centralized
- Retry logic is automatic

### What Stayed the Same (Externally)
- User-visible behavior identical
- Same default values
- Same environment variables
- Same functionality

## 8. Key Improvements Summary

### Maintainability
- ✅ Single source of truth for configuration
- ✅ Clear separation of concerns
- ✅ Well-documented exception types
- ✅ Consistent error handling patterns

### Robustness
- ✅ Automatic retry on transient failures
- ✅ Better error messages for debugging
- ✅ Configuration validation
- ✅ Graceful degradation

### Observability
- ✅ Comprehensive logging
- ✅ Error context preservation
- ✅ Retry attempt tracking
- ✅ Configuration status logging

### Developer Experience
- ✅ Type hints throughout
- ✅ Comprehensive docstrings
- ✅ Clear error messages
- ✅ Easy configuration via env vars

## 9. Usage Examples

### Configuration

```python
# Using default configuration
from app_config import get_settings

settings = get_settings()
print(settings.app_title)
print(settings.cache_ttl)
```

```bash
# Override via environment variables
export DATABASE_URL="mssql://user:pass@server/db"
export CACHE_TTL=7200
export APP_TITLE="My Custom Title"
```

```bash
# Or use .env file
cat > .env << EOF
DATABASE_URL=mssql://user:pass@server/db
CACHE_TTL=7200
APP_TITLE=My Custom Title
EOF
```

### Error Handling

```python
# Using retry decorator
from tsi.error_handling import with_retry

@with_retry(max_attempts=3)
def unreliable_operation():
    # Automatically retries on transient errors
    return fetch_data()

# Using custom exceptions
from tsi.exceptions import DatabaseConnectionError

try:
    init_database()
except DatabaseConnectionError as e:
    logger.error(f"Database connection failed: {e.message}")
    # Handle gracefully

# Using safe_execute
from tsi.error_handling import safe_execute

result = safe_execute(
    optional_operation,
    default=[],
    error_context="fetching metadata"
)
```

## 10. Files Modified

### Created
- `src/tsi/exceptions.py` - Custom exception hierarchy
- `src/tsi/error_handling.py` - Error handling utilities
- `tests/test_config.py` - Configuration tests (31 tests)
- `tests/test_error_handling.py` - Error handling tests (33 tests)

### Modified
- `src/app_config/settings.py` - Enhanced with comprehensive configuration
- `src/tsi/config.py` - Now imports from centralized settings
- `src/tsi/theme.py` - Uses configuration values
- `src/tsi/pages/landing.py` - Better error handling
- `src/tsi/services/database.py` - Retry logic and custom exceptions
- `src/tsi/services/data/loaders.py` - Enhanced error handling
- `src/tsi_rust_api.py` - Better import error handling
- `src/tsi/plots/distributions.py` - Uses CACHE_TTL from config
- `src/tsi/plots/sky_map.py` - Uses CACHE_TTL from config
- `src/tsi/plots/timeline.py` - Uses CACHE_TTL from config

## 11. Future Enhancements

Potential future improvements (not implemented in this phase):

1. **Configuration UI**: Admin panel for runtime configuration changes
2. **Circuit Breaker**: Advanced failure detection and recovery
3. **Metrics**: Prometheus/StatsD integration for monitoring
4. **Configuration Profiles**: Development, staging, production profiles
5. **Secret Management**: Integration with AWS Secrets Manager or Azure Key Vault
6. **Health Checks**: Comprehensive health check endpoints
7. **Audit Logging**: Track configuration changes

## 12. Migration Guide for Developers

### Using Configuration Values

```python
# Old way
cache_ttl = 3600
data_path = Path("data")

# New way
from tsi.config import CACHE_TTL, DATA_ROOT
cache_ttl = CACHE_TTL
data_path = DATA_ROOT
```

### Error Handling

```python
# Old way
try:
    result = database.query()
except Exception as e:
    logger.error(f"Query failed: {e}")
    raise

# New way
from tsi.exceptions import DatabaseQueryError
from tsi.error_handling import with_retry

@with_retry(max_attempts=3)
def query_database():
    try:
        result = database.query()
        return result
    except Exception as e:
        raise DatabaseQueryError(
            "Failed to execute query",
            details={"query": "SELECT ..."}
        ) from e
```

### Adding New Configuration

```python
# In app_config/settings.py
class Settings(BaseSettings):
    # Add new field
    new_feature_enabled: bool = Field(
        default=False,
        description="Enable new feature"
    )

# Use it
from app_config import get_settings
settings = get_settings()
if settings.new_feature_enabled:
    # Feature code
    pass
```

## Conclusion

This refactoring successfully centralizes configuration and improves error handling throughout the TSI application while maintaining complete backward compatibility. The changes provide a solid foundation for production deployment with better maintainability, observability, and robustness.

All tests pass, configuration is validated, and the system is ready for deployment.
