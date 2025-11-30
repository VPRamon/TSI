# ETL Integration Plan

## Overview

This document describes the ETL (Extract, Transform, Load) architecture for the TSI scheduling analytics system. The goal is to pre-compute and denormalize data into an analytics table (`schedule_blocks_analytics`) to accelerate query performance for Streamlit dashboard pages.

## Phase 1: Analytics Table for Block-Level Data (IMPLEMENTED)

### Goals

1. **Reduce Query Complexity**: Eliminate complex multi-table joins on every page load
2. **Pre-compute Derived Fields**: Calculate visibility hours, priority bins, and other metrics once during upload
3. **Improve Response Times**: Direct reads from a single denormalized table instead of 5+ table joins
4. **Enable Future Optimizations**: Foundation for caching, incremental updates, and materialized views

### Current Architecture (Before ETL)

The current system performs these joins on every page load:

```sql
FROM dbo.schedule_scheduling_blocks ssb
JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
JOIN dbo.targets t ON sb.target_id = t.target_id
LEFT JOIN dbo.constraints c ON sb.constraints_id = c.constraints_id
LEFT JOIN dbo.altitude_constraints ac ON c.altitude_constraints_id = ac.altitude_constraints_id
```

Additionally, it parses JSON for visibility periods (`visibility_periods_json`) to compute total visibility hours on every request.

### Target Architecture (After ETL)

A single denormalized analytics table that is populated during schedule upload:

```sql
SELECT * FROM analytics.schedule_blocks_analytics
WHERE schedule_id = @schedule_id
```

## Analytics Table Schema

### Table: `analytics.schedule_blocks_analytics`

Located in: `docs/sql/001_create_analytics_table.sql`

```sql
CREATE TABLE analytics.schedule_blocks_analytics (
    -- Identity
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    
    -- Foreign Keys
    schedule_id BIGINT NOT NULL,
    scheduling_block_id BIGINT NOT NULL,
    
    -- Target Information (denormalized from targets table)
    target_ra_deg FLOAT NOT NULL,
    target_dec_deg FLOAT NOT NULL,
    
    -- Block Core Fields (from scheduling_blocks)
    priority FLOAT NOT NULL,
    priority_bucket TINYINT NOT NULL,  -- Pre-computed: 1=Low, 2=Medium-Low, 3=Medium-High, 4=High
    requested_duration_sec INT NOT NULL,
    min_observation_sec INT NOT NULL,
    
    -- Computed columns
    requested_hours AS (CAST(requested_duration_sec AS FLOAT) / 3600.0) PERSISTED,
    elevation_range_deg AS (COALESCE(max_altitude_deg, 90.0) - COALESCE(min_altitude_deg, 0.0)) PERSISTED,
    
    -- Constraints (denormalized from constraints/altitude_constraints)
    min_altitude_deg FLOAT NULL,
    max_altitude_deg FLOAT NULL,
    min_azimuth_deg FLOAT NULL,
    max_azimuth_deg FLOAT NULL,
    
    -- Scheduling Status (from schedule_scheduling_blocks)
    is_scheduled BIT NOT NULL DEFAULT 0,
    scheduled_start_mjd FLOAT NULL,
    scheduled_stop_mjd FLOAT NULL,
    scheduled_duration_sec AS (...) PERSISTED,
    
    -- Pre-computed Visibility Metrics (extracted from visibility_periods_json)
    total_visibility_hours FLOAT NOT NULL DEFAULT 0.0,
    visibility_period_count INT NOT NULL DEFAULT 0,
    is_impossible AS (CASE WHEN total_visibility_hours = 0 THEN 1 ELSE 0 END) PERSISTED,
    
    -- Metadata
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE()
);
```

### Priority Bucket Calculation

Priority buckets are computed as quartiles based on the schedule's priority range:

```
bucket = 1 + floor(3 * (priority - min_priority) / (max_priority - min_priority))
```

- **Bucket 1**: Low priority (bottom 25%)
- **Bucket 2**: Medium-Low priority (25-50%)
- **Bucket 3**: Medium-High priority (50-75%)
- **Bucket 4**: High priority (top 25%)

## ETL Implementation

### Trigger Point

The ETL runs automatically after `store_schedule()` in `rust_backend/src/db/operations.rs`:

```
[Schedule Upload] → [store_schedule()] → [populate_schedule_analytics()] → [Success]
```

### ETL Logic (Rust)

Located in: `rust_backend/src/db/analytics.rs`

Key functions:
- `populate_schedule_analytics(schedule_id)` - Main ETL function
- `fetch_analytics_blocks_for_sky_map(schedule_id)` - Query for Sky Map page
- `fetch_analytics_blocks_for_distribution(schedule_id)` - Query for Distributions page
- `has_analytics_data(schedule_id)` - Check if analytics exist
- `delete_schedule_analytics(schedule_id)` - Clean up analytics

The function:
1. Deletes any existing analytics rows for the schedule (idempotent)
2. Computes priority range for bucket calculation
3. Queries normalized tables with all necessary joins
4. Parses visibility JSON to compute total hours
5. Computes priority buckets based on min/max
6. Bulk inserts into `analytics.schedule_blocks_analytics`

### Python API

Located in: `rust_backend/src/python/database.rs`

```python
import tsi_rust

# Manually populate analytics
rows = tsi_rust.py_populate_analytics(schedule_id=42)

# Check if analytics exist
has_data = tsi_rust.py_has_analytics_data(schedule_id=42)

# Delete analytics
deleted = tsi_rust.py_delete_analytics(schedule_id=42)
```

### Feature Flag

A configuration flag controls the analytics table usage:

```python
# app_config/settings.py
use_analytics_table: bool = Field(
    default=True,
    description="Use pre-computed analytics table for improved query performance",
)
```

## Page Migration Status

### Phase 1 Targets (IMPLEMENTED)

| Page | Status | Notes |
|------|--------|-------|
| Sky Map | ✅ Migrated | Uses `fetch_analytics_blocks_for_sky_map()` with automatic fallback |
| Distributions | ✅ Migrated | Uses `fetch_analytics_blocks_for_distribution()` with automatic fallback |

### Phase 2 Targets (Planned)

| Page | Status | Notes |
|------|--------|-------|
| Insights | 🔄 Planned | Complex, needs additional derived fields |
| Trends | 🔄 Planned | Similar to Distributions |
| Timeline | 🔄 Planned | Needs scheduled_start/stop_mjd |
| Visibility Map | 🔄 Planned | Low priority |
| Compare | 🔄 Planned | Needs two-schedule support |

### Automatic Fallback

Both Sky Map and Distributions services automatically fall back to legacy joins if:
- Analytics table doesn't exist
- Analytics data is empty for the schedule
- Any error occurs querying analytics

This ensures the application works even if the analytics table hasn't been created yet.

## SQL Migration

The migration script is located at: `docs/sql/001_create_analytics_table.sql`

Run order:
1. Create the `analytics` schema
2. Create the `schedule_blocks_analytics` table
3. Create indexes for common query patterns
4. Create stored procedures for maintenance (optional)

### Running the Migration

```bash
# Using sqlcmd
sqlcmd -S your-server.database.windows.net -d your-database -U your-user -P your-password -i docs/sql/001_create_analytics_table.sql

# Or using Azure Data Studio / SSMS
# Open and execute docs/sql/001_create_analytics_table.sql
```

### Backfilling Existing Schedules

After creating the table, populate analytics for existing schedules:

```python
import tsi_rust

# Get all schedules
schedules = tsi_rust.py_list_schedules()

# Populate analytics for each
for schedule in schedules:
    schedule_id = schedule["schedule_id"]
    rows = tsi_rust.py_populate_analytics(schedule_id)
    print(f"Schedule {schedule_id}: {rows} analytics rows")
```

## Testing

### Unit Tests

Located in: `tests/test_analytics_etl.py`

Tests cover:
- Priority bucket computation
- Visibility periods JSON parsing
- Data consistency between analytics and legacy paths
- Field value calculations

Run tests:
```bash
pytest tests/test_analytics_etl.py -v
```

### Rust Tests

Located in: `rust_backend/src/db/analytics.rs`

Run tests:
```bash
cd rust_backend
cargo test analytics
```

## Performance Expectations

### Before (Legacy Joins)
- ~5 table joins per query
- JSON parsing for every row
- Repeated computation of derived fields

### After (Analytics Table)
- Single table read
- Pre-computed fields
- Indexed queries for common filters

Expected improvement: **2-5x faster** for typical page loads.

## Future Enhancements (Phase 3+)

1. **Incremental Updates**: Update only changed blocks instead of full refresh
2. **Additional Derived Fields**: Pre-compute correlations, conflicts
3. **Caching Layer**: Redis/in-memory cache for hot schedules
4. **Materialized Views**: Database-level aggregations for common queries

---

## Phase 2: Summary Analytics Tables (IMPLEMENTED)

### Goals

1. **Pre-compute Schedule-Level Metrics**: Overall statistics used by Insights/Trends pages
2. **Pre-compute Priority-Level Statistics**: Per-priority scheduling rates for Trends charts
3. **Pre-compute Histogram Bins**: Visibility and time-based rate distributions
4. **Reduce Runtime Computation**: Eliminate expensive aggregations on every page load

### New Tables

Phase 2 introduces four new tables in the `analytics` schema:

#### Table: `analytics.schedule_summary_analytics`

Overall metrics for each schedule:

```sql
CREATE TABLE analytics.schedule_summary_analytics (
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_id BIGINT NOT NULL,
    
    -- Block counts
    total_blocks INT NOT NULL,
    scheduled_blocks INT NOT NULL,
    unscheduled_blocks INT NOT NULL,
    impossible_blocks INT NOT NULL,
    scheduling_rate FLOAT NOT NULL,
    
    -- Priority statistics
    priority_min FLOAT,
    priority_max FLOAT,
    priority_mean FLOAT,
    priority_median FLOAT,
    priority_scheduled_mean FLOAT,
    priority_unscheduled_mean FLOAT,
    
    -- Visibility statistics
    visibility_total_hours FLOAT NOT NULL,
    visibility_mean_hours FLOAT,
    visibility_min_hours FLOAT,
    visibility_max_hours FLOAT,
    
    -- Time statistics
    requested_total_hours FLOAT NOT NULL,
    requested_mean_hours FLOAT,
    scheduled_total_hours FLOAT NOT NULL,
    
    -- Correlations (Spearman)
    corr_priority_visibility FLOAT,
    corr_priority_requested FLOAT,
    corr_visibility_requested FLOAT,
    
    -- Conflict stats
    conflict_count INT NOT NULL DEFAULT 0,
    
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE()
);
```

#### Table: `analytics.schedule_priority_rates`

Per-priority scheduling statistics:

```sql
CREATE TABLE analytics.schedule_priority_rates (
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_id BIGINT NOT NULL,
    priority_value INT NOT NULL,
    total_count INT NOT NULL,
    scheduled_count INT NOT NULL,
    scheduling_rate FLOAT NOT NULL,
    visibility_mean_hours FLOAT,
    requested_mean_hours FLOAT,
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE()
);
```

#### Table: `analytics.schedule_visibility_bins`

Visibility-based rate histogram:

```sql
CREATE TABLE analytics.schedule_visibility_bins (
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_id BIGINT NOT NULL,
    bin_index INT NOT NULL,
    bin_min_hours FLOAT NOT NULL,
    bin_max_hours FLOAT NOT NULL,
    bin_mid_hours FLOAT NOT NULL,
    total_count INT NOT NULL,
    scheduled_count INT NOT NULL,
    scheduling_rate FLOAT NOT NULL,
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE()
);
```

#### Table: `analytics.schedule_heatmap_bins`

2D heatmap for visibility vs. requested time:

```sql
CREATE TABLE analytics.schedule_heatmap_bins (
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_id BIGINT NOT NULL,
    visibility_mid_hours FLOAT NOT NULL,
    time_mid_hours FLOAT NOT NULL,
    total_count INT NOT NULL,
    scheduled_count INT NOT NULL,
    scheduling_rate FLOAT NOT NULL,
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE()
);
```

### SQL Migration

Located at: `docs/sql/002_create_summary_tables.sql`

Run after Phase 1 migration:
```bash
sqlcmd -S your-server.database.windows.net -d your-database -U your-user -P your-password -i docs/sql/002_create_summary_tables.sql
```

### ETL Implementation

The summary analytics are populated automatically after block-level analytics:

```
[Schedule Upload] 
  → [store_schedule()] 
  → [populate_schedule_analytics()]      # Phase 1
  → [populate_summary_analytics()]       # Phase 2
  → [Success]
```

#### Rust Functions (analytics.rs)

- `populate_summary_analytics(schedule_id, n_bins)` - Main Phase 2 ETL
- `fetch_schedule_summary(schedule_id)` - Get summary metrics
- `fetch_priority_rates(schedule_id)` - Get per-priority rates
- `fetch_visibility_bins(schedule_id)` - Get visibility histogram
- `fetch_heatmap_bins(schedule_id)` - Get 2D heatmap bins
- `has_summary_analytics(schedule_id)` - Check if summary exists
- `delete_summary_analytics(schedule_id)` - Clean up summary data

### Python API

```python
import tsi_rust

# Manually populate summary analytics (uses 10 bins by default)
tsi_rust.py_populate_summary_analytics(schedule_id=42)
tsi_rust.py_populate_summary_analytics(schedule_id=42, n_bins=15)

# Check if summary analytics exist
has_data = tsi_rust.py_has_summary_analytics(schedule_id=42)

# Fetch summary metrics
summary = tsi_rust.py_get_schedule_summary(schedule_id=42)
print(f"Scheduling rate: {summary.scheduling_rate:.2%}")
print(f"Total blocks: {summary.total_blocks}")

# Fetch priority rates
rates = tsi_rust.py_get_priority_rates(schedule_id=42)
for rate in rates:
    print(f"Priority {rate.priority_value}: {rate.scheduling_rate:.2%} ({rate.total_count} blocks)")

# Fetch visibility bins
bins = tsi_rust.py_get_visibility_bins(schedule_id=42)
for b in bins:
    print(f"[{b.bin_min_hours:.1f}-{b.bin_max_hours:.1f}h]: {b.scheduling_rate:.2%}")

# Fetch heatmap bins
heatmap = tsi_rust.py_get_heatmap_bins(schedule_id=42)
for h in heatmap:
    print(f"Vis={h.visibility_mid_hours:.1f}h, Time={h.time_mid_hours:.1f}h: {h.scheduling_rate:.2%}")

# Delete summary analytics
tsi_rust.py_delete_summary_analytics(schedule_id=42)
```

### Backfilling Existing Schedules

After running both migrations, backfill all schedules:

```python
import tsi_rust

schedules = tsi_rust.py_list_schedules()

for schedule in schedules:
    schedule_id = schedule["schedule_id"]
    
    # Phase 1: Block-level analytics (if not already done)
    if not tsi_rust.py_has_analytics_data(schedule_id):
        rows = tsi_rust.py_populate_analytics(schedule_id)
        print(f"Schedule {schedule_id}: {rows} block analytics rows")
    
    # Phase 2: Summary analytics
    if not tsi_rust.py_has_summary_analytics(schedule_id):
        tsi_rust.py_populate_summary_analytics(schedule_id)
        print(f"Schedule {schedule_id}: Summary analytics populated")
```

### Phase 2 Files Changed

#### New Files
- `docs/sql/002_create_summary_tables.sql` - SQL migration for Phase 2 tables

#### Modified Files
- `rust_backend/src/db/analytics.rs` - Added summary analytics functions and structs
- `rust_backend/src/db/mod.rs` - Export new functions and types
- `rust_backend/src/db/operations.rs` - Call summary analytics after upload
- `rust_backend/src/python/database.rs` - Python bindings for summary functions
- `rust_backend/src/lib.rs` - Register new Python functions and classes
- `docs/ETL_INTEGRATION_PLAN.md` - This documentation

---

## Rollback Plan

If issues are discovered:

1. The system automatically falls back to legacy joins if analytics fails
2. To force legacy path: delete analytics data or don't create the table
3. Analytics table data can be deleted without affecting operational data
4. No changes needed to application code for rollback

## Files Changed

### New Files
- `docs/ETL_INTEGRATION_PLAN.md` - This document
- `docs/sql/001_create_analytics_table.sql` - SQL migration script
- `rust_backend/src/db/analytics.rs` - ETL implementation
- `tests/test_analytics_etl.py` - Python tests

### Modified Files
- `rust_backend/src/db/mod.rs` - Export analytics module
- `rust_backend/src/db/operations.rs` - Call analytics after schedule upload
- `rust_backend/src/services/sky_map.rs` - Use analytics with fallback
- `rust_backend/src/services/distributions.rs` - Use analytics with fallback
- `rust_backend/src/python/database.rs` - Python bindings for analytics
- `rust_backend/src/lib.rs` - Register analytics Python functions
- `src/app_config/settings.py` - Add `use_analytics_table` flag
