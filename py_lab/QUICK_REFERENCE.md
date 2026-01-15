# py_lab Quick Reference

## CLI Tool (Fastest Way)

### Basic Usage
```bash
# Check feasibility of 50 observations
python3 -m py_lab.cli -s data/schedule.json -n 50

# With possible periods file
python3 -m py_lab.cli \
  -s data/schedule.json \
  -p data/possible_periods.json \
  -n 50

# Filter by priority (≥7)
python3 -m py_lab.cli \
  -s data/schedule.json \
  -n 100 \
  --priority 7

# Save conflict report
python3 -m py_lab.cli \
  -s data/schedule.json \
  -n 50 \
  --output output/
```

## Python API Examples

### Load Schedule
```python
from py_lab import load_schedule
df = load_schedule(schedule_path="sample_schedule.json")
```

### Check Feasibility
```python
from py_lab import ConflictAnalyzer
analyzer = ConflictAnalyzer()
result = analyzer.check_feasibility(df)
print(f"Feasible: {result.feasible}")
```

### Find Conflicts
```python
from py_lab import detect_conflicts
conflicts = detect_conflicts(df, max_iterations=50)
print(f"Conflicting observations: {conflicts.infeasible_tasks}")
```

### Filter by Priority
```python
from py_lab import filter_by_priority
high_priority = filter_by_priority(df, min_priority=8)
```

### Filter Impossible Observations
```python
has_visibility = df['visibility_periods'].apply(
    lambda x: x and len(x) > 0 if isinstance(x, list) else False
)
df_feasible = df[has_visibility & (df['requested_duration'] > 0)]
```

## Common Workflows

### Basic Feasibility Check
```python
from py_lab import load_schedule, ConflictAnalyzer

df = load_schedule()
analyzer = ConflictAnalyzer()
result = analyzer.check_feasibility(df)
```

### Priority-Based Planning
```python
from py_lab import load_schedule, filter_by_priority, ConflictAnalyzer

df = load_schedule()
df_high = filter_by_priority(df, min_priority=8)

analyzer = ConflictAnalyzer()
result = analyzer.check_feasibility(df_high)
```

### Find and Export Conflicts
```python
from py_lab import detect_conflicts, export_to_json

conflicts = detect_conflicts(df)
if conflicts.infeasible_tasks:
    export_to_json(conflicts.details, "conflicts.json")
```

## Module Reference

### Core Modules
- **cp_sat.py**: CP-SAT solver algorithms (Task, can_schedule, find_minimal_infeasible_subset)
- **data_loader.py**: Schedule loading (ScheduleLoader, load_schedule)
- **conflict_analyzer.py**: Conflict detection (ConflictAnalyzer, detect_conflicts)
- **utils.py**: Helpers (filters, time conversion, export)

### Key Classes
- `Task(id, duration, periods)`: CP-SAT task representation
- `ConflictAnalyzer()`: Conflict detection with CP-SAT
- `ConflictResult`: Result object with feasibility and conflicts
- `ScheduleLoader(schedule_path, possible_periods_path)`: Schedule loader

### Essential Functions
- `load_schedule(schedule_path)`: Load observations
- `detect_conflicts(df)`: Find conflicting observations
- `filter_by_priority(df, min_priority, max_priority)`: Priority filter
- `filter_by_sky_region(df, ra_min, ra_max, dec_min, dec_max)`: Sky coordinate filter
- `merge_visibility_periods(periods)`: Merge overlapping periods
- `validate_schedule_dataframe(df)`: Check required columns

## Required Data Format

### JSON Structure
```json
{
  "blocks": [
    {
      "id": 1,
      "priority": 9.0,
      "requested_duration": 1800,
      "visibility_periods": [
        {"start": 60000.1, "stop": 60000.3}
      ]
    }
  ]
}
```

### Required DataFrame Columns
- `id`: Observation identifier
- `priority`: Observation priority
- `requested_duration`: Duration in seconds
- `visibility_periods`: List of time windows (MJD)

## Tips

1. **Start with minimal_example.py** for a complete workflow
2. **Filter early**: Remove impossible observations before CP-SAT
3. **Use backend**: For large datasets, use Rust backend for parsing
4. **Priority groups**: Analyze high-priority observations separately
5. **Adjust max_iterations**: 50 for quick checks, 100-200 for precise MIS

## Run the Example

```bash
cd /workspace/py_lab
python minimal_example.py
```

---
Last Updated: January 15, 2026

## Import Essentials

```python
# Main imports
from py_lab import (
    load_schedule,              # Load schedule JSON
    detect_conflicts,           # Find scheduling conflicts
    analyze_schedule,           # Generate full analysis
    filter_by_priority,         # Filter by priority range
    export_to_json,             # Export results to JSON
    mjd_to_datetime             # Convert MJD to datetime
)
```

## Common Workflows

### 1. Load and Explore
```python
from py_lab import load_schedule

# Load schedule
df, validation = load_schedule("schedule.json")

# Quick stats
print(f"Total: {len(df)}")
print(f"Priority: {df['priority'].min():.1f} - {df['priority'].max():.1f}")
print(f"Columns: {list(df.columns)}")
```

### 2. Filter Data
```python
from py_lab import filter_by_priority, filter_by_sky_region

# High priority only
high_pri = filter_by_priority(df, min_priority=8.0)

# Southern hemisphere
southern = filter_by_sky_region(df, dec_max=0)

# Combine filters
high_southern = filter_by_priority(
    filter_by_sky_region(df, dec_max=0),
    min_priority=8.0
)
```

### 3. Detect Conflicts
```python
from py_lab import detect_conflicts

# Check for conflicts
result = detect_conflicts(df)

if not result.feasible:
    print(f"Found {len(result.infeasible_tasks)} conflicting tasks")
    print(f"Conflict IDs: {result.infeasible_tasks[:5]}")
```

### 4. Generate Analysis
```python
from py_lab import analyze_schedule

# Full analysis report
report = analyze_schedule(df)

print(f"Total: {report['total_observations']}")
print(f"Possible: {report['possible_observations']}")
print(f"Impossible: {report['impossible_observations']}")
```

### 5. Export Results
```python
from py_lab import export_to_json, export_to_csv

# Export filtered data
export_to_csv(high_pri, "output/high_priority.csv")

# Export analysis
export_to_json(report, "output/analysis.json")
```

## Advanced Usage

### Custom Conflict Analysis
```python
from py_lab import ConflictAnalyzer

analyzer = ConflictAnalyzer()

# Check feasibility
feasibility = analyzer.check_feasibility(df)

# Find minimal conflicts
conflicts = analyzer.find_conflicts(df, max_iterations=100)

# Analyze by priority groups
groups = analyzer.analyze_priority_groups(df)
```

### Detailed Analytics
```python
from py_lab import ScheduleAnalytics

analytics = ScheduleAnalytics()

# Filter impossible
possible = analytics.filter_impossible_observations(df)

# Priority analysis
priority_stats = analytics.analyze_priority_distribution(df)

# Spatial analysis
spatial_stats = analytics.analyze_spatial_distribution(df)

# Full report
report = analytics.generate_summary_report(df)
```

### Time Utilities
```python
from py_lab import mjd_to_datetime, datetime_to_mjd, format_mjd_period

# Convert MJD to datetime
dt = mjd_to_datetime(60000.5)
print(dt)  # 2023-02-25 12:00:00+00:00

# Convert back
mjd = datetime_to_mjd(dt)

# Format period
period_str = format_mjd_period(60000.0, 60000.25)
print(period_str)  # "2023-02-25 00:00 to 2023-02-25 06:00 (6.00 hours)"
```

### Visibility Period Utilities
```python
from py_lab import merge_visibility_periods, compute_total_visibility_duration

periods = [
    {'start': 60000.0, 'stop': 60000.1},
    {'start': 60000.09, 'stop': 60000.15},
    {'start': 60000.3, 'stop': 60000.4}
]

# Merge overlapping
merged = merge_visibility_periods(periods)
print(f"Merged {len(periods)} → {len(merged)} periods")

# Total duration
total_days = compute_total_visibility_duration(periods)
print(f"Total: {total_days * 24:.2f} hours")
```

## Data Structure Reference

### Schedule DataFrame Columns
```
id                   - Scheduling block ID (integer)
original_block_id    - User-provided identifier (string)
priority             - Priority value (float, typically 0-10)
target_ra            - Right ascension in degrees (float)
target_dec           - Declination in degrees (float)
constraints          - Dict of observing constraints
min_observation      - Minimum observation time (seconds)
requested_duration   - Requested duration (seconds)
visibility_periods   - List of time windows when observable
scheduled_period     - Actual scheduled time (if scheduled)
```

### Visibility Period Format
```python
# Each period is a dict or tuple:
period = {'start': 60000.25, 'stop': 60000.35}  # MJD
# or
period = (60000.25, 60000.35)
```

### Conflict Result
```python
result = detect_conflicts(df)
# result.feasible          - bool: True if schedulable
# result.infeasible_tasks  - list: IDs of conflicting tasks
# result.message           - str: Human-readable message
# result.details           - dict: Additional information
```

## File Locations

```
data/
├── schedule.json           # Main schedule (2,647 blocks)
├── schedule_test.json      # Test subset
├── possible_periods.json   # Visibility periods
└── dark_periods.json       # Dark time windows

py_lab/
├── schedule_analysis.ipynb # Interactive notebook
└── output/                 # Generated reports
    ├── *.csv
    └── *.json
```

## Performance Tips

### For Large Datasets
```python
# 1. Filter first
high_pri = filter_by_priority(df, min_priority=9.0)
result = detect_conflicts(high_pri)  # Much faster

# 2. Sample for testing
sample = df.sample(1000)
result = detect_conflicts(sample)

# 3. Use limits
result = analyzer.find_conflicts(df, max_iterations=50)
```

### Memory Management
```python
from py_lab.utils import get_dataframe_memory_usage

mem = get_dataframe_memory_usage(df)
print(f"Memory: {mem['total_mb']:.2f} MB")
```

## Common Patterns

### Load → Filter → Analyze → Export
```python
from py_lab import *

# Load
df, _ = load_schedule("schedule.json")

# Filter
high_pri = filter_by_priority(df, min_priority=8.0)
possible = ScheduleAnalytics().filter_impossible_observations(high_pri)

# Analyze
conflicts = detect_conflicts(possible)
report = analyze_schedule(possible)

# Export
export_to_csv(possible, "output/filtered.csv")
export_to_json(report, "output/report.json")
```

### Batch Processing
```python
from pathlib import Path
from py_lab import ScheduleLoader, analyze_schedule, export_to_json

loader = ScheduleLoader()

for schedule_file in loader.list_available_schedules():
    print(f"Processing {schedule_file}...")
    
    df, _ = loader.load_schedule(schedule_file)
    report = analyze_schedule(df)
    
    output = Path("output") / f"{schedule_file.replace('.json', '_report.json')}"
    export_to_json(report, output)
```

## Error Handling

```python
from py_lab import load_schedule, detect_conflicts

try:
    df, validation = load_schedule("schedule.json")
    
    if 'warning' in validation:
        print(f"Warning: {validation['warning']}")
    
    result = detect_conflicts(df)
    
    if not result.feasible:
        # Handle conflicts
        print(f"Conflicts: {result.message}")
        
except FileNotFoundError as e:
    print(f"Schedule file not found: {e}")
except ValueError as e:
    print(f"Invalid schedule data: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

## Help & Documentation

```python
# Get help on any function
help(load_schedule)
help(detect_conflicts)
help(ConflictAnalyzer)

# View module documentation
import py_lab
help(py_lab)
```

## See Also

- [README.md](README.md) - Full documentation
- [schedule_analysis.ipynb](schedule_analysis.ipynb) - Interactive examples
- [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) - Implementation details
- [cp_sat.py](cp_sat.py) - CP-SAT solver documentation
