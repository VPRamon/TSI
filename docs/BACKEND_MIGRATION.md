# Backend Migration Guide

## Overview

The legacy `tsi.backend` package has been removed. All Python backend functionality is now consolidated into a single module: `tsi_rust_api`.

**Migration Date:** January 2026  
**Status:** Complete

## What Changed

### Before (Legacy)
```python
# Old imports (NO LONGER WORK)
from tsi.backend import TSIBackend
from tsi.backend import load_schedule_file
from tsi.backend.analytics import get_top_observations, find_conflicts
from tsi.backend.transformations import filter_by_priority
```

### After (Current)
```python
# New imports (USE THESE)
from tsi_rust_api import TSIBackend
from tsi_rust_api import load_schedule, load_schedule_file
from tsi_rust_api import get_top_observations, find_conflicts
from tsi_rust_api import filter_by_priority

# Or use the services layer (RECOMMENDED)
from tsi.services import BACKEND
from tsi.services.rust_backend import load_schedule_from_any
```

## Migration Steps

### 1. Update Imports

**Old Code:**
```python
from tsi.backend import TSIBackend
from tsi.backend.loaders import load_schedule_file
from tsi.backend.analytics import get_top_observations
```

**New Code:**
```python
from tsi_rust_api import TSIBackend, load_schedule_file, get_top_observations
```

### 2. Use Services Layer (Recommended)

For application code, prefer using the services layer which provides a singleton backend instance:

```python
from tsi.services import BACKEND

# Load data
df = BACKEND.load_schedule("data/schedule.json")

# Filter data
high_priority = BACKEND.filter_by_priority(df, min_priority=15.0)

# Analytics
top_10 = BACKEND.get_top_observations(df, n=10)
conflicts = BACKEND.find_conflicts(df)
```

### 3. File-Like Objects

For handling file uploads (e.g., Streamlit):

```python
from tsi.services.rust_backend import load_schedule_from_any

# Works with file paths
df = load_schedule_from_any("data/schedule.json")

# Works with file-like objects (e.g., uploaded files)
df = load_schedule_from_any(uploaded_file, format="json")
```

## API Reference

### tsi_rust_api Module

The `tsi_rust_api` module is now the single source of truth for all Python backend operations.

#### Classes

- **TSIBackend**: Main backend class providing all operations
  - `load_schedule(path, format="auto")` - Load schedule from file
  - `load_schedule_from_string(content, format="json")` - Load from string
  - `filter_by_priority(df, min_priority, max_priority)` - Filter by priority range
  - `filter_by_scheduled(df, filter_type)` - Filter by scheduling status
  - `filter_dataframe(df, ...)` - Apply multiple filters
  - `get_top_observations(df, n, by)` - Get top N observations
  - `find_conflicts(df)` - Find scheduling conflicts
  - `remove_duplicates(df, subset, keep)` - Remove duplicate rows
  - `validate_dataframe(df)` - Validate data quality
  - `mjd_to_datetime(mjd)` - Convert MJD to datetime
  - `datetime_to_mjd(dt)` - Convert datetime to MJD

#### Functional API

Convenience functions for common operations:

```python
from tsi_rust_api import (
    load_schedule,              # Quick load (returns pandas DataFrame)
    load_schedule_file,         # Load from file with format control
    load_schedule_from_string,  # Load from JSON string
    load_dark_periods,          # Load dark periods data
    filter_by_priority,         # Quick priority filter
    get_top_observations,       # Get top N observations
    find_conflicts,             # Find scheduling conflicts
)
```

### Services Layer

The services layer (`tsi.services`) provides:

- **BACKEND**: Singleton `TSIBackend` instance
- **load_schedule_from_any**: Helper for file paths and file-like objects
- High-level analytics, filtering, and processing functions

```python
from tsi.services import (
    BACKEND,
    load_schedule_rust,
    prepare_dataframe,
    get_filtered_dataframe,
    validate_dataframe,
    compute_correlations,
    find_conflicts,
    get_top_observations,
)
```

## Breaking Changes

### Removed Modules

The following modules have been removed:

- `tsi.backend.core`
- `tsi.backend.loaders`
- `tsi.backend.analytics`
- `tsi.backend.transformations`
- `tsi.backend.utils`

All functionality has been moved to `tsi_rust_api` or the services layer.

### Function Changes

1. **Analytics Functions**: `get_top_observations` and `find_conflicts` now use pandas operations instead of calling Rust functions that no longer exist.

2. **Filter Functions**: All filter functions are now implemented as Python helpers with JSON roundtrips for consistency.

3. **Loading Functions**: Simplified to only support JSON format (CSV support removed).

## Examples

### Complete Example: Before and After

**Before (Legacy):**
```python
from tsi.backend import TSIBackend
from tsi.backend.analytics import get_top_observations

backend = TSIBackend()
df = backend.load_schedule("data/schedule.json")
top_10 = get_top_observations(df, n=10)
```

**After (Current):**
```python
from tsi_rust_api import TSIBackend, get_top_observations

backend = TSIBackend()
df = backend.load_schedule("data/schedule.json")
top_10 = get_top_observations(df, n=10)

# Or using the backend method directly:
top_10 = backend.get_top_observations(df, n=10)
```

**After (Services Layer - RECOMMENDED):**
```python
from tsi.services import BACKEND

df = BACKEND.load_schedule("data/schedule.json")
top_10 = BACKEND.get_top_observations(df, n=10)
```

## Testing

A new comprehensive test suite validates the consolidated API:

```bash
# Run API tests
pytest tests/test_tsi_rust_api.py -v

# Run all tests
pytest tests/ -v
```

## Troubleshooting

### Import Errors

**Error:** `ModuleNotFoundError: No module named 'tsi.backend'`

**Solution:** Update imports to use `tsi_rust_api` or `tsi.services`:
```python
# OLD (broken)
from tsi.backend import TSIBackend

# NEW (working)
from tsi_rust_api import TSIBackend
# OR
from tsi.services import BACKEND
```

### Missing Functions

**Error:** `AttributeError: module 'tsi_rust' has no attribute 'py_...'`

**Solution:** The consolidated API no longer calls these Rust functions directly. Use the higher-level functions in `tsi_rust_api` or `tsi.services`.

## Support

For questions or issues:

1. Check this migration guide
2. Review examples in `/workspace/examples/`
3. Check test suite in `/workspace/tests/test_tsi_rust_api.py`
4. Review the services layer in `/workspace/src/tsi/services/`
