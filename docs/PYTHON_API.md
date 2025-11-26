# TSI Rust Backend - Python API Reference

## Overview

The TSI Rust Backend provides high-performance data processing for telescope scheduling analysis. This document describes the complete Python API.

## Installation

```bash
# Development install (from project root)
maturin develop --release

# Production install (after building wheel)
pip install telescope-scheduling-intelligence
```

## Quick Start

```python
from tsi_rust_api import TSIBackend

# Initialize backend
backend = TSIBackend()

# Load schedule data
df = backend.load_schedule("data/schedule.csv")
print(f"Loaded {len(df)} observations")

# Compute analytics
metrics = backend.compute_metrics(df)
print(f"Scheduled: {metrics['scheduled_count']}/{metrics['total_observations']}")
print(f"Mean priority: {metrics['mean_priority']:.2f}")

# Filter high-priority observations
high_priority = backend.filter_by_priority(df, min_priority=15.0)
print(f"High priority: {len(high_priority)} observations")
```

## API Reference

### TSIBackend Class

High-level interface to the Rust backend.

#### Constructor

```python
backend = TSIBackend(use_pandas: bool = True)
```

**Parameters:**
- `use_pandas`: If True, return pandas DataFrames. If False, return Polars DataFrames.

---

### Data Loading

#### `load_schedule(path, format="auto")`

Load schedule data from file.

**Parameters:**
- `path` (str | Path): Path to CSV or JSON file
- `format` (Literal["auto", "csv", "json"]): File format (auto-detects from extension)

**Returns:** DataFrame with scheduling blocks and derived columns

**Columns in returned DataFrame:**
- `schedulingBlockId`: Unique identifier
- `priority`: Observation priority (0-20)
- `scheduled_flag`: Boolean indicating if scheduled
- `raInDeg`, `decInDeg`: Celestial coordinates
- `requestedDurationSec`: Requested observation duration
- `requested_hours`: Duration in hours
- `priority_bin`: Priority category (Low/Medium/High/Very High)
- `num_visibility_periods`: Count of visibility windows
- `total_visibility_hours`: Total visibility time
- `elevation_range_deg`: Elevation range
- And more...

**Example:**
```python
df = backend.load_schedule("data/schedule.csv")
print(df.columns)
print(df.head())
```

#### `load_schedule_from_string(content, format="json")`

Load schedule from string content.

**Parameters:**
- `content` (str): JSON or CSV string
- `format` (Literal["csv", "json"]): Content format

**Returns:** DataFrame with scheduling blocks

**Example:**
```python
json_str = open("data/schedule.json").read()
df = backend.load_schedule_from_string(json_str, format="json")
```

---

### Preprocessing & Validation

#### `validate_schedule(df)`

Validate schedule data structure and quality.

**Parameters:**
- `df` (DataFrame): DataFrame to validate

**Returns:** Dictionary with keys:
- `is_valid` (bool): Overall validation status
- `warnings` (list[str]): Non-critical issues
- `errors` (list[str]): Critical issues

**Example:**
```python
result = backend.validate_schedule(df)
if not result['is_valid']:
    print("Validation errors:")
    for error in result['errors']:
        print(f"  - {error}")
```

---

### Analytics & Algorithms

#### `compute_metrics(df)`

Compute comprehensive scheduling metrics.

**Parameters:**
- `df` (DataFrame): Scheduling data

**Returns:** Dictionary with metrics:
- `total_observations` (int): Total number of observations
- `scheduled_count` (int): Number scheduled
- `unscheduled_count` (int): Number unscheduled
- `scheduled_percentage` (float): Percentage scheduled
- `mean_priority` (float): Mean priority value
- `median_priority` (float): Median priority
- `mean_duration_hours` (float): Mean duration
- `total_visibility_hours` (float): Total visibility time

**Example:**
```python
metrics = backend.compute_metrics(df)
print(f"Scheduled: {metrics['scheduled_percentage']:.1f}%")
print(f"Mean priority: {metrics['mean_priority']:.2f}")
print(f"Total visibility: {metrics['total_visibility_hours']:.1f} hours")
```

#### `get_top_observations(df, n=10, by="priority")`

Get top N observations sorted by column.

**Parameters:**
- `df` (DataFrame): Scheduling data
- `n` (int): Number of top observations
- `by` (str): Column to sort by

**Returns:** DataFrame with top N rows

**Example:**
```python
top_10 = backend.get_top_observations(df, n=10, by="priority")
print(top_10[['schedulingBlockId', 'priority', 'scheduled_flag']])
```

#### `find_conflicts(df)`

Find scheduling conflicts (overlapping observations).

**Parameters:**
- `df` (DataFrame): Scheduling data

**Returns:** DataFrame with conflicts:
- `observation_id_1`: First observation ID
- `observation_id_2`: Second observation ID
- `conflict_type`: Type of conflict
- `overlap_hours`: Duration of overlap

**Example:**
```python
conflicts = backend.find_conflicts(df)
print(f"Found {len(conflicts)} conflicts")
if len(conflicts) > 0:
    print(conflicts.head())
```

#### `greedy_schedule(df, max_iterations=1000)`

Run greedy scheduling optimization.

**Parameters:**
- `df` (DataFrame): Observations to schedule
- `max_iterations` (int): Maximum optimization iterations

**Returns:** Dictionary with:
- `selected_ids` (list[str]): IDs of selected observations
- `total_duration` (float): Total scheduled duration
- `iterations_run` (int): Number of iterations executed
- `converged` (bool): Whether algorithm converged

**Example:**
```python
result = backend.greedy_schedule(df, max_iterations=500)
print(f"Selected {len(result['selected_ids'])} observations")
print(f"Total duration: {result['total_duration']:.1f} hours")
print(f"Converged: {result['converged']}")
```

---

### Filtering & Transformations

#### `filter_by_priority(df, min_priority=0.0, max_priority=100.0)`

Filter observations by priority range.

**Parameters:**
- `df` (DataFrame): Data to filter
- `min_priority` (float): Minimum priority (inclusive)
- `max_priority` (float): Maximum priority (inclusive)

**Returns:** Filtered DataFrame

**Example:**
```python
high_priority = backend.filter_by_priority(df, min_priority=15.0)
print(f"High priority: {len(high_priority)} / {len(df)}")
```

#### `filter_by_scheduled(df, filter_type="All")`

Filter by scheduling status.

**Parameters:**
- `df` (DataFrame): Data to filter
- `filter_type` (Literal["All", "Scheduled", "Unscheduled"]): Status filter

**Returns:** Filtered DataFrame

**Example:**
```python
scheduled = backend.filter_by_scheduled(df, "Scheduled")
unscheduled = backend.filter_by_scheduled(df, "Unscheduled")
print(f"Scheduled: {len(scheduled)}, Unscheduled: {len(unscheduled)}")
```

#### `filter_dataframe(df, **filters)`

Apply multiple filters simultaneously.

**Parameters:**
- `df` (DataFrame): Data to filter
- `priority_min` (float): Minimum priority
- `priority_max` (float): Maximum priority
- `scheduled_filter` (str): "All", "Scheduled", or "Unscheduled"
- `priority_bins` (list[str] | None): Priority bins to include
- `block_ids` (list[str] | None): Block IDs to include

**Returns:** Filtered DataFrame

**Example:**
```python
filtered = backend.filter_dataframe(
    df,
    priority_min=10.0,
    priority_max=20.0,
    scheduled_filter="Scheduled",
    priority_bins=["High", "Very High"],
    block_ids=["SB001", "SB002", "SB003"]
)
print(f"Filtered: {len(filtered)} observations")
```

#### `remove_duplicates(df, subset=None, keep="first")`

Remove duplicate rows.

**Parameters:**
- `df` (DataFrame): Data to clean
- `subset` (list[str] | None): Columns to check (None = all)
- `keep` (Literal["first", "last", "none"]): Which duplicates to keep

**Returns:** DataFrame with duplicates removed

**Example:**
```python
# Remove complete duplicates
clean_df = backend.remove_duplicates(df)

# Remove duplicates by schedulingBlockId only
unique_blocks = backend.remove_duplicates(df, subset=["schedulingBlockId"])
```

#### `remove_missing_coordinates(df)`

Remove observations with missing RA/Dec.

**Parameters:**
- `df` (DataFrame): Data to clean

**Returns:** DataFrame with valid coordinates only

**Example:**
```python
valid_coords = backend.remove_missing_coordinates(df)
print(f"Valid coordinates: {len(valid_coords)} / {len(df)}")
```

#### `validate_dataframe(df)`

Validate data quality (coordinates, priorities, etc.).

**Parameters:**
- `df` (DataFrame): Data to validate

**Returns:** Tuple of (is_valid: bool, issues: list[str])

**Example:**
```python
is_valid, issues = backend.validate_dataframe(df)
if not is_valid:
    print("Data quality issues:")
    for issue in issues:
        print(f"  - {issue}")
```

---

### Time Conversions

#### `mjd_to_datetime(mjd)` (static method)

Convert Modified Julian Date to ISO datetime string.

**Parameters:**
- `mjd` (float): MJD value

**Returns:** ISO datetime string

**Example:**
```python
dt = TSIBackend.mjd_to_datetime(59580.5)
print(dt)  # '2022-01-01T12:00:00Z'
```

#### `datetime_to_mjd(dt)` (static method)

Convert ISO datetime to MJD.

**Parameters:**
- `dt` (str): ISO datetime string

**Returns:** MJD value (float)

**Example:**
```python
mjd = TSIBackend.datetime_to_mjd('2022-01-01T12:00:00Z')
print(mjd)  # 59580.5
```

---

## Convenience Functions

For quick one-off operations without creating a backend instance:

```python
from tsi_rust_api import load_schedule, compute_metrics, filter_by_priority

# Load data
df = load_schedule("data/schedule.csv")

# Compute metrics
metrics = compute_metrics(df)

# Filter data
high_priority = filter_by_priority(df, min_priority=15.0)
```

---

## Performance Notes

### Zero-Copy Data Transfer

The backend uses Apache Arrow for zero-copy data transfer between Rust and Python:

```python
# This is extremely fast (no data copying)
df = backend.load_schedule("large_file.csv")  # Returns Polars → pandas conversion

# For maximum performance, use Polars directly:
backend = TSIBackend(use_pandas=False)
df_polars = backend.load_schedule("large_file.csv")  # Pure Polars, zero-copy
```

### Benchmarks

Typical performance improvements over pure Python:

| Operation | Python Time | Rust Time | Speedup |
|-----------|-------------|-----------|---------|
| Load CSV (2647 rows) | 0.5-1.0s | 0.1-0.2s | **3-5x** |
| Parse visibility | 40s | 2-4s | **10-20x** |
| Compute metrics | 0.5s | 0.05-0.1s | **5-10x** |
| Filter data | 0.1s | 0.01-0.02s | **5-10x** |

---

## Error Handling

All functions raise Python exceptions with descriptive messages:

```python
try:
    df = backend.load_schedule("nonexistent.csv")
except RuntimeError as e:
    print(f"Error loading file: {e}")

try:
    metrics = backend.compute_metrics(empty_df)
except RuntimeError as e:
    print(f"Error computing metrics: {e}")
```

---

## Type Hints

The API includes full type hints for IDE support:

```python
from typing import Literal
import pandas as pd

backend = TSIBackend(use_pandas=True)

# Type checker knows this returns pd.DataFrame
df: pd.DataFrame = backend.load_schedule("data.csv")

# Type checker knows this returns dict
metrics: dict[str, float] = backend.compute_metrics(df)
```

---

## Migration from Pure Python

### Before (Pure Python):

```python
from core.loaders import schedule_loader
from core.algorithms import analysis

df = schedule_loader.load_schedule("data.csv")
metrics = analysis.compute_metrics(df)
```

### After (Rust Backend):

```python
from tsi_rust_api import TSIBackend

backend = TSIBackend()
df = backend.load_schedule("data.csv")
metrics = backend.compute_metrics(df)
```

**Benefits:**
- ⚡ 5-20x faster
- 🔒 Type-safe operations in Rust
- 📦 Lower memory usage
- 🧵 Automatic parallelization where beneficial

---

## See Also

- [Rust Backend Architecture](../rust_backend/README.md)
- [Performance Benchmarks](../docs/benchmarks.md)
- [Migration Guide](../docs/migration.md)
- [API Examples](../examples/api_examples.py)
