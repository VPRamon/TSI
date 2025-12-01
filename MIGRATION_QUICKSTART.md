# Database Migration Quick Reference

## Configuration

### Set Data Source

**Option 1: Environment Variable**
```bash
export DATA_SOURCE=legacy  # or etl
./run_dashboard.sh
```

**Option 2: .env File**
```ini
# .env file in project root
DATA_SOURCE=legacy
```

**Option 3: Direct Python**
```python
import os
os.environ['DATA_SOURCE'] = 'etl'

from app_config import get_settings
settings = get_settings()
print(settings.data_source)  # 'etl'
```

## Using the Unified API

### Recommended Approach (Automatic Routing)

```python
from tsi.services.database import (
    get_sky_map_data,
    get_distribution_data,
)

# These automatically route based on DATA_SOURCE config
sky_data = get_sky_map_data(schedule_id=123)
dist_data = get_distribution_data(schedule_id=123, filter_impossible=False)
```

### Explicit Path Control

```python
from tsi.services.data_source import (
    get_sky_map_data_legacy,
    get_sky_map_data_etl,
    get_sky_map_data_unified,
)

# Force legacy path (for testing/comparison)
sky_data_legacy = get_sky_map_data_legacy(schedule_id=123)

# Force ETL path (will error if analytics not available)
try:
    sky_data_etl = get_sky_map_data_etl(schedule_id=123)
except DatabaseQueryError:
    print("ETL data not available, falling back")
    sky_data_etl = get_sky_map_data_legacy(schedule_id=123)

# Use configured path (recommended)
sky_data = get_sky_map_data_unified(schedule_id=123)
```

## Testing Both Paths

### Compare Outputs
```python
from tsi.services.data_source import (
    get_distribution_data_legacy,
    get_distribution_data_etl,
)

# Get data from both paths
legacy_data = get_distribution_data_legacy(schedule_id=123, filter_impossible=False)
etl_data = get_distribution_data_etl(schedule_id=123, filter_impossible=False)

# Compare
print(f"Legacy: {legacy_data.total_count} blocks")
print(f"ETL: {etl_data.total_count} blocks")
print(f"Match: {legacy_data.total_count == etl_data.total_count}")
```

### Run Tests with Different Configurations
```bash
# Test with legacy
DATA_SOURCE=legacy pytest tests/ -v

# Test with ETL
DATA_SOURCE=etl pytest tests/ -v

# Compare performance
time DATA_SOURCE=legacy pytest tests/
time DATA_SOURCE=etl pytest tests/
```

## Troubleshooting

### Check Current Configuration
```python
from app_config import get_settings
settings = get_settings()
print(f"Current data source: {settings.data_source}")
```

### Check if ETL Data Available
```python
import tsi_rust

# Check if analytics exist for a schedule
has_analytics = tsi_rust.py_has_analytics_data(schedule_id=123)
print(f"Analytics available: {has_analytics}")

if not has_analytics:
    # Populate analytics
    rows = tsi_rust.py_populate_analytics(schedule_id=123)
    print(f"Populated {rows} analytics rows")
```

### View Logs
```python
import logging

# Enable debug logging
logging.basicConfig(level=logging.DEBUG)

# Now calls will log which path they use
from tsi.services.data_source import get_sky_map_data_unified
data = get_sky_map_data_unified(schedule_id=123)
# Look for log: "get_sky_map_data_unified: Using data_source=..."
```

### Handle ETL Path Errors
```python
from tsi.services.data_source import get_sky_map_data_etl
from tsi.exceptions import DatabaseQueryError

try:
    data = get_sky_map_data_etl(schedule_id=123)
except DatabaseQueryError as e:
    print(f"ETL path failed: {e}")
    # Either fall back or investigate why analytics missing
    if "No analytics data available" in str(e):
        print("Analytics table is empty for this schedule")
        print("Run populate_analytics() or use legacy path")
```

## Performance Monitoring

### Timing Comparison
```python
import time
from tsi.services.data_source import (
    get_sky_map_data_legacy,
    get_sky_map_data_etl,
)

# Time legacy path
start = time.time()
data_legacy = get_sky_map_data_legacy(schedule_id=123)
legacy_time = time.time() - start

# Time ETL path
start = time.time()
data_etl = get_sky_map_data_etl(schedule_id=123)
etl_time = time.time() - start

print(f"Legacy: {legacy_time:.3f}s")
print(f"ETL: {etl_time:.3f}s")
print(f"Speedup: {legacy_time / etl_time:.1f}x")
```

## Migration Checklist

### Before Switching to ETL
- [ ] Verify analytics tables exist in database
- [ ] Run migration script: `docs/sql/001_create_analytics_table.sql`
- [ ] Populate analytics for existing schedules
- [ ] Test ETL path in staging environment
- [ ] Compare outputs between legacy and ETL
- [ ] Benchmark performance improvements
- [ ] Review logs for any errors

### Switching to ETL
- [ ] Set `DATA_SOURCE=etl` in configuration
- [ ] Restart application
- [ ] Monitor logs for "Using data_source=etl"
- [ ] Verify all pages load correctly
- [ ] Check for any error messages
- [ ] Monitor performance metrics

### Rolling Back
- [ ] Set `DATA_SOURCE=legacy` in configuration
- [ ] Restart application
- [ ] Verify application works normally
- [ ] Investigate ETL issues offline

## Available Functions

### Sky Map
- `get_sky_map_data_unified(schedule_id)` - Auto-routing
- `get_sky_map_data_legacy(schedule_id)` - Force legacy
- `get_sky_map_data_etl(schedule_id)` - Force ETL

### Distributions
- `get_distribution_data_unified(schedule_id, filter_impossible=False)` - Auto-routing
- `get_distribution_data_legacy(schedule_id, filter_impossible=False)` - Force legacy
- `get_distribution_data_etl(schedule_id, filter_impossible=False)` - Force ETL

### Timeline
- `get_schedule_timeline_data_unified(schedule_id)` - Both paths same for now

### Insights
- `get_insights_data_unified(schedule_id, filter_impossible=False)` - Both paths same for now

### Trends
- `get_trends_data_unified(schedule_id, filter_impossible=False, n_bins=10, bandwidth=0.3, n_smooth_points=100)` - Both paths same for now

### Compare
- `get_compare_data_unified(current_schedule_id, comparison_schedule_id, current_name, comparison_name)` - Both paths same for now

### Visibility Map
- `get_visibility_map_data_unified(schedule_id)` - Both paths same for now

## Support

For issues or questions:
1. Check `DATABASE_MIGRATION_SUMMARY.md` for detailed documentation
2. Review logs for configuration and routing information
3. Verify analytics tables exist if using ETL path
4. Use test script: `python test_migration.py`
5. Compare legacy and ETL outputs to identify discrepancies
