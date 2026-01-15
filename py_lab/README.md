# py_lab - CP-SAT Planning Laboratory

A minimal toolkit for astronomers and scientists to experiment with constraint programming for observation scheduling. This lab focuses on loading candidate observations, preprocessing impossible observations, and using Google OR-Tools CP-SAT solver to detect conflicts and optimize schedules.

## Overview

`py_lab` is a **planning-focused** laboratory for pre-scheduling analysis:

- **Data Loading**: Load candidate observations from JSON files (uses Rust backend when available)
- **Preprocessing**: Filter impossible observations based on visibility and duration constraints
- **CP-SAT Solver**: Use constraint programming to check feasibility and find conflicts
- **Minimal Infeasible Subsets**: Identify which observations conflict with each other
- **Utilities**: Essential helpers for time conversion, filtering, and data export

**Note**: This lab is designed for **planning sessions** (before scheduling is executed). It does not track or analyze scheduled vs. unscheduled observations. For full scheduling analytics, use the Rust backend directly.

## Directory Structure

```
py_lab/
├── __init__.py              # Package exports
├── cp_sat.py                # CP-SAT constraint solver (core algorithm)
├── data_loader.py           # Schedule JSON loading (thin backend wrapper)
├── conflict_analyzer.py     # Conflict detection using CP-SAT
├── utils.py                 # Time conversion, filtering, export utilities
├── cli.py                   # Command-line tool for feasibility checking
├── README.md                # This file
└── archive/                 # Archived analytics, notebooks, and outputs
    ├── analytics.py         # (archived) Heavy analytics - use backend instead
    ├── schedule_analysis.ipynb  # (archived) Interactive notebook
    └── output/              # (archived) Generated reports
```

## Installation

## Installation

### Prerequisites

1. **Python 3.11+** with pip
2. **Rust backend** (optional, for faster JSON parsing):
   ```bash
   cd backend
   cargo build --release
   ```

### Dependencies

Install required Python packages:

```bash
pip install pandas ortools
```

Key dependencies:
- `pandas` - DataFrame operations
- `ortools` - Google OR-Tools CP-SAT solver
- `tsi_rust` / `tsi_rust_api` - (optional) Rust backend for fast parsing

## Quick Start

### Using the CLI Tool

The easiest way to check schedule feasibility:

```bash
# Basic usage
python3 -m py_lab.cli \
  --schedule data/schedule.json \
  --possible-periods data/possible_periods.json \
  --num-obs 50

# With priority filter
python3 -m py_lab.cli \
  -s data/schedule.json \
  -p data/possible_periods.json \
  -n 100 \
  --priority 7

# Save conflict report to file
python3 -m py_lab.cli \
  -s data/schedule.json \
  -p data/possible_periods.json \
  -n 50 \
  --output output/

# Quiet mode (minimal output)
python3 -m py_lab.cli -s data/schedule.json -n 50 --quiet
```

**CLI Options:**
- `-s, --schedule`: Path to schedule JSON file (required)
- `-p, --possible-periods`: Path to possible periods JSON file (optional)
- `-n, --num-obs`: Number of observations to analyze (required)
- `--priority`: Minimum priority filter (e.g., 7 for high priority only)
- `--max-iterations`: Maximum iterations for conflict detection (default: 50)
- `-o, --output`: Output directory for conflict reports
- `-q, --quiet`: Quiet mode - minimal output

The CLI tool will:
1. Load and validate the schedule
2. Filter out impossible observations (no visibility, zero duration)
3. Apply optional priority filters
4. Check feasibility using CP-SAT
5. If infeasible, find minimal conflicting subset
6. Optionally save conflict report to JSON

### Python API Example

```python
from py_lab import (
    load_schedule,
    filter_by_priority,
    ConflictAnalyzer,
    validate_schedule_dataframe
)

# Load candidate observations
df = load_schedule(schedule_path="data/schedule.json")
print(f"Loaded {len(df)} candidate observations")

# Validate and filter
is_valid, missing = validate_schedule_dataframe(df)

# Filter impossible observations (no visibility or zero duration)
has_visibility = df['visibility_periods'].apply(
    lambda x: x is not None and len(x) > 0 if isinstance(x, list) else False
)
df_feasible = df[has_visibility & (df['requested_duration'] > 0)]

# Optional: filter by priority
df_high = filter_by_priority(df_feasible, min_priority=7)

# Check scheduling feasibility with CP-SAT
analyzer = ConflictAnalyzer()
result = analyzer.check_feasibility(df_high)

if not result.feasible:
    # Find minimal infeasible subset
    conflicts = analyzer.find_conflicts(df_high, max_iterations=50)
    print(f"Conflicts: {conflicts.infeasible_tasks}")
```

## Data Requirements

### Schedule JSON Format

Candidate observations must be in JSON format with the following structure:

```json
{
  "name": "schedule_name",
  "blocks": [
    {
      "id": 1000001,
      "original_block_id": "obs-001",
      "priority": 8.5,
      "target_ra": 150.0,
      "target_dec": -60.0,
      "requested_duration": 1800,
      "visibility_periods": [
        {"start_mjd": 61771.2, "stop_mjd": 61771.4},
        {"start_mjd": 61772.1, "stop_mjd": 61772.3}
      ]
    }
  ]
}
```

**Required fields** for CP-SAT analysis:
- `id`: Unique observation identifier
- `priority`: Observation priority (higher = more important)
- `requested_duration`: Duration in seconds
- `visibility_periods`: List of time windows when observation is possible (MJD format)

**Optional fields**:
- `target_ra`, `target_dec`: Sky coordinates (for spatial filtering)
- `original_block_id`: Original identifier from proposal system

### Data Directory

Place schedule files in the `data/` directory:

```
data/
├── schedule.json           # Main schedule file
├── schedule_test.json      # Test subset
└── possible_periods.json   # (optional) Separate visibility periods
```

## Module Reference

### data_loader

Load candidate observations from JSON files:

```python
from py_lab import ScheduleLoader, load_schedule

# Quick load
df = load_schedule(schedule_path="data/schedule.json")

# Advanced usage
loader = ScheduleLoader(
    schedule_path="data/schedule.json",
    possible_periods_path="data/possible_periods.json"
)
df = loader.load_schedule(validate=True)
available = loader.list_available_schedules()
```

**Backend Integration**: `load_schedule` uses the Rust backend (`tsi_rust` / `tsi_rust_api`) when available for faster parsing, otherwise falls back to Python JSON parsing.

### conflict_analyzer

Detect scheduling conflicts using CP-SAT constraint programming:

```python
from py_lab import ConflictAnalyzer, detect_conflicts

# Quick conflict detection
result = detect_conflicts(df)

# Advanced analysis
analyzer = ConflictAnalyzer()

# Check feasibility
feasibility = analyzer.check_feasibility(df)

# Find minimal infeasible subset
conflicts = analyzer.find_conflicts(df, max_iterations=100)

# Analyze by priority groups
priority_analysis = analyzer.analyze_priority_groups(df)
```

### cp_sat (Core Solver)

Low-level CP-SAT solver for custom constraints:

```python
from py_lab.cp_sat import Task, can_schedule, find_minimal_infeasible_subset

# Define tasks manually
tasks = [
    Task(id="obs1", duration=1800, periods=[(0, 10000), (20000, 30000)]),
Essential utilities for data manipulation and filtering:

```python
from py_lab import utils

# Time conversion
dt = utils.mjd_to_datetime(60000.5)
mjd = utils.datetime_to_mjd(dt)
period_str = utils.format_mjd_period(60000.0, 60000.5)

# Visibility period helpers
merged = utils.merge_visibility_periods(periods)
duration = utils.compute_total_visibility_duration(periods)
filtered = utils.filter_visibility_by_period(periods, start=60000.0, stop=61000.0)

# DataFrame filtering
high_priority = utils.filter_by_priority(df, min_priority=8)
northern_sky = utils.filter_by_sky_region(df, dec_min=0)

# Export
utils.export_to_json(df, "output/results.json")
utils.export_to_csv(df, "output/results.csv")

# Validation
is_valid, missing_cols = utils.validate_schedule_dataframe(df)
```

## Common Workflows

### 1. Basic Feasibility Check

```python
from py_lab import load_schedule, ConflictAnalyzer

df = load_schedule()
analyzer = ConflictAnalyzer()
result = analyzer.check_feasibility(df)

print(f"Feasible: {result.feasible}")
print(f"Message: {result.message}")
```

### 2. Find Conflicting Observations

```python
from py_lab import load_schedule, detect_conflicts

df = load_schedule()
conflicts = detect_conflicts(df, max_iterations=100)

if conflicts.infeasible_tasks:
    print(f"Found {len(conflicts.infeasible_tasks)} conflicting observations:")
    for obs_id in conflicts.infeasible_tasks:
        print(f"  - {obs_id}")
```

### 3. Priority-Based Analysis

```python
from py_lab import load_schedule, filter_by_priority, ConflictAnalyzer

df = load_schedule()

# Analyze high-priority observations separately
df_high = filter_by_priority(df, min_priority=8)

analyzer = ConflictAnalyzer()
result = analyzer.check_feasibility(df_high)

print(f"High priority: {len(df_high)} obs, feasible: {result.feasible}")
```

### 4. Custom Preprocessing

```python
from py_lab import load_schedule, merge_visibility_periods, ConflictAnalyzer

df = load_schedule()

# Filter impossible observations
def preprocess(df):
    # Remove observations with no visibility
    has_vis = df['visibility_periods'].apply(
        lambda x: x and len(x) > 0 if isinstance(x, list) else False
    )
    df = df[has_vis]
    
    # Remove zero duration
    df = df[df['requested_duration'] > 0]
    
    # Merge overlapping visibility periods
    df['visibility_periods'] = df['visibility_periods'].apply(
        lambda periods: merge_visibility_periods(periods)
    )
    
    return df

df_preprocessed = preprocess(df)

analyzer = ConflictAnalyzer()
result = analyzer.check_feasibility(df_preprocessed)
```

## Performance Tips

1. **Use the Rust backend**: Build and install `tsi_rust` for 10-100x faster JSON parsing
2. **Filter early**: Remove impossible observations before CP-SAT analysis
3. **Limit subsets**: For large datasets (>1000 obs), analyze priority groups separately
4. **Adjust max_iterations**: Lower values for quick checks, higher (100-200) for precise MIS

## Archived Components

The following components have been archived to `archive/` as they duplicate backend functionality:

- `analytics.py`: Heavy statistical analysis (use backend analytics API instead)
- `schedule_analysis.ipynb`: Interactive notebook (superseded by `minimal_example.py`)
- `output/`: Generated reports (create new outputs as needed)

For full analytics capabilities, use the Rust backend directly:
```python
import tsi_rust_api as tsi
trends = tsi.get_trends_data(schedule_id)
distribution = tsi.get_distribution_data(schedule_id)
```

## Contributing

This is a minimal experimental lab. For production scheduling:
- Use the Rust backend (`/workspace/backend`) for performance
- Use backend analytics for comprehensive reporting
- Refer to `BACKEND_INTEGRATION_ANALYSIS.md` for migration details

## License

See [LICENSE](../LICENSE) in the repository root.
mis = find_minimal_infeasible_subset(tasks, max_iterations=50)
```

### utils

Helper utilities for data manipulation:

```python
from py_lab import utils

# Time conversion
dt = utils.mjd_to_datetime(60000.5)
mjd = utils.datetime_to_mjd(dt)
period_str = utils.format_mjd_period(60000.0, 60000.25)

# Visibility periods
merged = utils.merge_visibility_periods(periods)
duration = utils.compute_total_visibility_duration(periods)

# Filtering
high_priority = utils.filter_by_priority(df, min_priority=7.0)
northern = utils.filter_by_sky_region(df, dec_min=0)
scheduled = utils.filter_scheduled_only(df)

# Export
utils.export_to_json(data, "output/report.json")
utils.export_to_csv(df, "output/observations.csv")
```

## Examples

### Example 1: Load and Filter High Priority

```python
from py_lab import load_schedule, filter_by_priority

# Load schedule
df, _ = load_schedule("schedule.json")

# Filter high priority observations
high_priority = filter_by_priority(df, min_priority=8.0)
print(f"Found {len(high_priority)} high-priority observations")
```

### Example 2: Detect Conflicts in Priority Group

```python
from py_lab import load_schedule, filter_by_priority, detect_conflicts

# Load and filter
df, _ = load_schedule("schedule.json")
priority_9 = filter_by_priority(df, min_priority=9.0, max_priority=10.0)

# Detect conflicts
result = detect_conflicts(priority_9)
if not result.feasible:
    print(f"Conflicts: {result.infeasible_tasks}")
```

### Example 3: Generate Full Report

```python
from py_lab import load_schedule, analyze_schedule, export_to_json

# Load and analyze
df, _ = load_schedule("schedule.json")
report = analyze_schedule(df)

# Export report
export_to_json(report, "output/full_report.json")
print(f"Total: {report['total_observations']}")
print(f"Possible: {report['possible_observations']}")
```

### Example 4: Batch Processing

```python
from pathlib import Path
from py_lab import ScheduleLoader, ScheduleAnalytics, export_to_json

loader = ScheduleLoader()
analytics = ScheduleAnalytics()

for schedule_file in loader.list_available_schedules():
    print(f"Processing {schedule_file}...")
    
    df, _ = loader.load_schedule(schedule_file)
    report = analytics.generate_summary_report(df)
    
    output_name = Path(schedule_file).stem + "_report.json"
    export_to_json(report, f"output/{output_name}")
```

## Testing Modules

Each module includes a `__main__` block for testing:

```bash
# Test data loader
python3 py_lab/data_loader.py

# Test conflict analyzer
python3 py_lab/conflict_analyzer.py

# Test analytics
python3 py_lab/analytics.py

# Test utilities
python3 py_lab/utils.py
```

## Performance Considerations

### Large Datasets

For schedules with >100k observations:

1. **Use filters first**: Filter by priority/region before conflict analysis
2. **Sample for testing**: Use `df.sample(1000)` or `df.head(1000)` for initial tests
3. **Adjust CP-SAT parameters**: Increase `max_iterations` for better conflict detection
4. **Use Rust backend**: Always use `tsi_rust` parser (10-100x faster than pure Python)

### Memory Usage

Check memory usage:

```python
from py_lab.utils import get_dataframe_memory_usage

mem_info = get_dataframe_memory_usage(df)
print(f"Memory: {mem_info['total_mb']:.2f} MB")
```

## Troubleshooting

### ImportError: No module named 'tsi_rust'

Build the Rust backend:

```bash
cd backend
cargo build --release
```

Then ensure Python can find it (may need to set `LD_LIBRARY_PATH` or install as package).

### CP-SAT solver timeout

For large problem sets, the solver may take time or timeout:

```python
# Use smaller subsets
small_df = df.head(100)
result = detect_conflicts(small_df)

# Or filter by priority first
high_priority = filter_by_priority(df, min_priority=9.0)
result = detect_conflicts(high_priority)
```

### Missing visibility_periods

Some blocks may have empty or missing visibility periods:

```python
from py_lab.analytics import ScheduleAnalytics

analytics = ScheduleAnalytics()
possible_df = analytics.filter_impossible_observations(df)
# Now only blocks with visibility remain
```

## Contributing

When adding new features:

1. Add functionality to appropriate module (data_loader, conflict_analyzer, analytics, utils)
2. Include docstrings with examples
3. Add test cases in `__main__` block
4. Update this README with new examples

## References

- **CP-SAT Solver**: [Google OR-Tools Documentation](https://developers.google.com/optimization/cp/cp_solver)
- **Schedule Schema**: See `backend/docs/schedule.schema.json`
- **Backend Repository Pattern**: See `docs/REPOSITORY_PATTERN.md`

## License

See LICENSE file in the root directory.
