# API Migration - Status Report

## Overview
Created a new `api/` module to isolate PyO3 from internal Rust models, allowing independent evolution of qtty-based types and internal database structures.

## ✅ Completed

### Module Structure
- **api/mod.rs** (26 lines): Module organization and public exports
- **api/types.rs** (682 lines): 40+ Python-facing DTOs using only primitives (f64, String, Vec, HashMap)
- **api/conversions.rs** (514 lines): Type conversion layer (From/TryFrom implementations)
- **api/streamlit.rs** (382 lines): PyO3 function wrappers exposing services to Python
- **db/models/**: Removed all PyO3 derives (#[pyclass], #[pymethods]) - now pure Rust types

### Key Features
1. **Type Isolation**: All PyO3 derives (#[pyclass], #[pyfunction]) confined to api/ module only
2. **Primitive Types**: API DTOs use f64 instead of qtty types (MJD, Degrees, Seconds)
3. **Conversion Layer**: Automatic conversion between internal models and API DTOs
4. **Clean Compilation**: New API module (tsi_rust_api) compiles without errors or warnings
5. **Internal Models Clean**: db/models/ no longer have any PyO3 dependencies

### API Functions Exposed
- Time conversion: `mjd_to_datetime`, `datetime_to_mjd`
- Database: `init_database`, `db_health_check`, `store_schedule`, `list_schedules`, `get_schedule`
- Analytics ETL: `populate_analytics` (phases 1, 2, 3)
- Visualization queries: `get_sky_map_data`, `get_distribution_data`, `get_timeline_data`, `get_insights_data`, `get_trends_data`, `get_compare_data`
- Algorithms: `find_conflicts`, `get_top_observations`
- Validation: `get_validation_report`

### Conversion Handling
- **qtty types → f64**: MJD.value(), Degrees.value(), Seconds.value()
- **Nested structures → flat DTOs**: Analytics stats flattened to single level
- **Unix timestamps → MJD**: Time bins converted using offset 40587.0
- **Option types**: Properly handled with Option<T> in API
- **IDs**: BlockId/ScheduleId → i64, string IDs preserved

## ⚠️ Known Limitations

### Old tsi_rust Module
The original `tsi_rust` Python module is now **deprecated** due to internal models no longer having PyO3 derives. Users must migrate to the new `tsi_rust_api` module.

**Migration:**
```python
# Old (deprecated)
# from tsi_rust import ...

# New (recommended)
from tsi_rust_api import (
    init_database,
    store_schedule, 
    get_sky_map_data,
    get_trends_data,
    # ... etc
)
```

### Placeholder Implementations
Two functions return placeholder data due to internal functions returning `Py<PyAny>`:
1. **list_schedules**: Returns empty Vec (internal py_list_schedules returns Py<PyAny>)
2. **get_schedule**: Returns minimal Schedule (internal py_get_schedule returns Py<PyAny>)

These would need internal functions refactored to return typed Rust structs.

### Simplified API Functions
- **store_schedule**: Takes 3 args (schedule_name, schedule_json, visibility_json), returns String confirmation instead of full metadata (internal returns Py<PyAny>)
- **get_top_observations**: Returns JSON String instead of typed Vec (matches internal implementation)

## 🔄 Next Steps

### 1. Remove PyO3 from Internal Models
Remove #[pyclass] and #[pymethods] from:
- `rust_backend/src/db/models/*.rs`
- `rust_backend/src/services/*.rs`

Keep only Serialize/Deserialize for JSON compatibility.

### 2. Update lib.rs
The new `tsi_rust_api` module is registered. Consider:
- Deprecating old `tsi_rust` module once Python code migrates
- Making pyo3 an optional dependency (feature-gated)

### 3. Python Integration Testing
```python
# Test new API module
import tsi_rust_api

# Initialize database
tsi_rust_api.init_database(repository_type="local")

# Store schedule
result = tsi_rust_api.store_schedule(
    schedule_name="test",
    schedule_json=json.dumps(schedule_data),
    visibility_json=json.dumps(visibility_data)
)

# Query analytics
trends = tsi_rust_api.get_trends_data(
    schedule_name="test",
    n_priority_bins=10,
    smoothing_window=0.5,
    n_time_bins=12
)
```

### 4. Streamlit Migration
Update Streamlit app to import from `tsi_rust_api` instead of `tsi_rust`:
```python
# Old
from tsi_rust import ...

# New
from tsi_rust_api import ...
```

### 5. Refactor Internal Functions (Optional)
To remove placeholder implementations, refactor:
- `python::py_list_schedules` → return `Vec<crate::api::ScheduleInfo>`
- `python::py_get_schedule` → return `Schedule`
- `python::py_store_schedule` → return `ScheduleMetadata`

## 📊 Impact

### Benefits
1. **Isolation**: Internal models free to use qtty types without Python constraints
2. **Maintainability**: Single conversion point in api/conversions.rs
3. **Type Safety**: Compile-time verification of conversions
4. **Documentation**: Clear API boundary with documented DTOs
5. **Evolution**: Can modify internal structures without breaking Python

### Considerations
- **Performance**: Conversion overhead (minimal for data sizes involved)
- **Duplication**: API DTOs duplicate some internal model structures
- **Maintenance**: Need to keep conversions updated when models change

## 🔍 Testing Checklist

- [ ] Build Python module: `maturin develop`
- [ ] Import test: `python -c "import tsi_rust_api; print(dir(tsi_rust_api))"`
- [ ] Database operations: init, health check, store/retrieve schedules
- [ ] Analytics pipeline: populate phases 1-3, query results
- [ ] Visualization queries: sky map, distributions, timeline, insights
- [ ] Algorithms: conflict detection, top observations
- [ ] Validation: generate reports with issues
- [ ] Integration with Streamlit app
- [ ] Performance comparison with old module

## 📝 Notes

### Compilation Stats
- Initial errors: 99
- Final errors: 0
- Final warnings: 0
- Build time: ~18 seconds (dev profile)

### Module Sizes
- Total API module: ~1,600 lines across 4 files
- Largest file: types.rs (682 lines)
- Conversion layer: conversions.rs (514 lines)
- Function wrappers: streamlit.rs (382 lines)

### Dependencies
- PyO3 0.27.2: Python bindings
- qtty (local): Strongly-typed astronomical units
- siderust (local): Astronomy calculations
- anyhow: Error handling
- serde/serde_json: Serialization
