# CLI Usage Guide

## Overview

The `py_lab.cli` module provides a command-line interface for checking observation schedule feasibility using the CP-SAT constraint programming solver.

## Quick Start

```bash
# Basic usage
python3 -m py_lab.cli --schedule <path> --num-obs <N>

# Full example
python3 -m py_lab.cli \
  --schedule data/schedule.json \
  --possible-periods data/possible_periods.json \
  --num-obs 50
```

## Command-Line Options

| Option | Short | Required | Description |
|--------|-------|----------|-------------|
| `--schedule` | `-s` | ✓ | Path to schedule JSON file |
| `--possible-periods` | `-p` | | Path to possible periods JSON file (optional) |
| `--num-obs` | `-n` | ✓ | Number of observations to analyze |
| `--priority` | | | Minimum priority filter (e.g., 7) |
| `--max-iterations` | | | Max iterations for conflict detection (default: 50) |
| `--output` | `-o` | | Output directory for conflict reports |
| `--quiet` | `-q` | | Quiet mode - minimal output |

## Examples

### Basic Feasibility Check

```bash
python3 -m py_lab.cli -s data/schedule.json -n 50
```

Checks if the first 50 observations can be scheduled without conflicts.

### With Priority Filter

```bash
python3 -m py_lab.cli \
  -s data/schedule.json \
  -p data/possible_periods.json \
  -n 100 \
  --priority 7
```

Filters for high-priority observations (≥7) and checks if 100 of them can be scheduled.

### Save Conflict Report

```bash
python3 -m py_lab.cli \
  -s data/schedule.json \
  -n 50 \
  --output output/
```

If conflicts are found, saves a detailed JSON report to `output/conflict_report.json`.

### Quiet Mode

```bash
python3 -m py_lab.cli -s data/schedule.json -n 50 --quiet
```

Minimal output - useful for scripting. Check exit code:
- `0`: Schedule is feasible
- `1`: Schedule is infeasible (has conflicts)

### Example with Real Data

```bash
python3 -m py_lab.cli \
  -s data/sensitive/est/schedule.json \
  -p data/sensitive/est/possible_periods.json \
  -n 100 \
  --priority 7 \
  --output /workspace/output
```

## Output

### Feasible Schedule

```
======================================================================
CP-SAT Feasibility Checker
======================================================================

[1/5] Loading schedule...
  ✓ Loaded 4864 candidate observations

[2/5] Validating data...
  ✓ Data validated

[3/5] Preprocessing observations...
  ✓ Filtered to 4844 possible observations
    - Zero duration removed: 0
    - No visibility removed: 20
  ✓ Priority filter (≥7): 4246 observations

  Analyzing subset of 100 observations...

[4/5] Checking scheduling feasibility with CP-SAT...
  ✓ FEASIBLE: All 100 observations can be scheduled!

======================================================================
Analysis complete!
======================================================================
```

### Infeasible Schedule with Conflicts

```
======================================================================
CP-SAT Feasibility Checker
======================================================================

[1/5] Loading schedule...
  ✓ Loaded 1000 candidate observations

[2/5] Validating data...
  ✓ Data validated

[3/5] Preprocessing observations...
  ✓ Filtered to 980 possible observations
    - Zero duration removed: 10
    - No visibility removed: 10

  Analyzing subset of 100 observations...

[4/5] Checking scheduling feasibility with CP-SAT...
  ✗ INFEASIBLE: Conflicts detected

[5/5] Finding minimal infeasible subset...
  ✓ Found 12 conflicting observations:
      - 1000001
      - 1000023
      - 1000045
      - 1000067
      - 1000089
      - 1000112
      - 1000134
      - 1000156
      - 1000178
      - 1000201
      ... and 2 more

  Report saved to: output/conflict_report.json

======================================================================
Analysis complete!
======================================================================
```

## Exit Codes

- `0`: Schedule is feasible
- `1`: Schedule is infeasible (conflicts detected)

## Integration with Scripts

```bash
#!/bin/bash

# Run feasibility check
if python3 -m py_lab.cli -s data/schedule.json -n 50 --quiet; then
    echo "Schedule is feasible!"
    # Proceed with scheduling
else
    echo "Conflicts detected - review conflict_report.json"
    exit 1
fi
```

## See Also

- [README.md](README.md) - Full documentation
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - Quick reference for Python API
- [cp_sat.py](cp_sat.py) - Core CP-SAT solver implementation
- [conflict_analyzer.py](conflict_analyzer.py) - Conflict detection logic
