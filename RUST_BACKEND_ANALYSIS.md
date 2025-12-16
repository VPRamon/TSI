# Rust Backend Complete Module and Function Analysis

**Generated:** December 16, 2025  
**Purpose:** Comprehensive documentation of every module and function in `rust_backend/`

---

## Table of Contents

1. [Overview](#overview)
2. [Module Structure](#module-structure)
3. [Core Modules](#core-modules)
   - [lib.rs](#librs)
   - [algorithms](#algorithms-module)
   - [db](#db-module)
   - [parsing](#parsing-module)
   - [python](#python-module)
   - [services](#services-module)
   - [transformations](#transformations-module)
4. [Optimization Recommendations](#optimization-recommendations)
5. [Summary Statistics](#summary-statistics)

---

## Overview

The Rust backend provides high-performance telescope scheduling analysis with Python bindings via PyO3. It handles:
- **JSON parsing** of observation schedules and visibility data
- **Database operations** for Azure PostgreSQL via repository pattern
- **Analytics computation** including metrics, conflicts, and optimization
- **Data transformation** and validation
- **Dashboard services** for visualization components

**Primary Users:**
- Python TSI application (`src/tsi/`)
- Dashboard components needing fast analytics
- ETL pipeline for schedule preprocessing

---

## Module Structure

```
rust_backend/
├── src/
│   ├── lib.rs                    # Module entry point & Python bindings registration
│   ├── algorithms/               # Analytics & optimization
│   │   ├── mod.rs
│   │   ├── analysis.rs          # Metrics, correlations, top observations
│   │   ├── conflicts.rs         # Conflict detection
│   │   └── optimization.rs      # Greedy scheduling
│   ├── db/                       # Database layer (Repository pattern)
│   │   ├── mod.rs
│   │   ├── checksum.rs          # SHA256 checksums
│   │   ├── config.rs            # Database configuration
│   │   ├── factory.rs           # Repository factory
│   │   ├── models.rs            # Domain models (Schedule, SchedulingBlock, etc.)
│   │   ├── repository.rs        # Repository trait definition
│   │   ├── services.rs          # High-level service layer
│   │   └── repositories/        # Concrete implementations
│   │       ├── azure/           # Azure PostgreSQL implementation
│   │       └── test.rs          # In-memory mock
│   ├── parsing/                  # JSON parsing
│   │   ├── mod.rs
│   │   └── json_parser.rs       # Schedule & visibility JSON parsing
│   ├── python/                   # PyO3 bindings
│   │   ├── mod.rs
│   │   ├── algorithms.rs        # Bindings for analytics functions
│   │   ├── database.rs          # Bindings for database operations
│   │   ├── time_bindings.rs     # MJD ↔ datetime conversion
│   │   └── transformations.rs  # Bindings for data cleaning/filtering
│   ├── services/                 # Business logic for dashboard
│   │   ├── mod.rs
│   │   ├── compare.rs           # Schedule comparison
│   │   ├── distributions.rs     # Distribution charts
│   │   ├── insights.rs          # Insights computation
│   │   ├── sky_map.rs           # Sky map data
│   │   ├── timeline.rs          # Timeline visualization
│   │   ├── trends.rs            # Trends analysis
│   │   ├── validation.rs        # Validation rules
│   │   ├── validation_report.rs # Validation reporting
│   │   └── visibility.rs        # Visibility histograms
│   └── transformations/          # Data processing utilities
│       ├── mod.rs
│       ├── cleaning.rs          # Duplicate removal, imputation
│       └── filtering.rs         # DataFrame filtering operations
└── tests/                        # Integration tests
```

---

## Core Modules

### lib.rs

**Purpose:** Main entry point for the Rust crate. Defines the Python module `tsi_rust` and registers all exported functions.

**Functions:**
- `tsi_rust(m: &Bound<'_, PyModule>)` - PyO3 module initializer, registers all Python-facing functions

**Used By:**
- Python import statement: `import tsi_rust`
- maturin build system

**Optimization Opportunities:**
- ✅ **Already optimal** - Clean module organization
- Consider splitting registration into sub-modules if it grows beyond ~100 functions

---

## algorithms Module

### algorithms/mod.rs

**Purpose:** Module orchestrator for analytics and optimization algorithms.

**Exports:**
- `analysis::{compute_metrics, compute_correlations, get_top_observations, AnalyticsSnapshot}`
- `conflicts::{find_conflicts, suggest_candidate_positions, SchedulingConflict, CandidatePlacement}`
- `optimization::{greedy_schedule, Constraint, Observation, OptimizationResult}`

**Used By:**
- `python/algorithms.rs` for Python bindings
- Internal analytics pipeline

**Optimization Opportunities:**
- ✅ Well-organized, no changes needed

---

### algorithms/analysis.rs

#### `compute_metrics(df: &DataFrame) -> Result<AnalyticsSnapshot, PolarsError>`

**What it does:**
- Computes dataset-level summary statistics: total observations, scheduled count, scheduling rate, mean/median priority, total visibility hours

**Purpose:**
- Provide high-level metrics for dashboard summary cards
- Quick assessment of schedule quality

**Used By:**
- `python/algorithms.rs::py_compute_metrics()` → Python `tsi_rust.compute_metrics()`
- Dashboard homepage, schedule comparison views

**Optimization Opportunities:**
- ⚠️ **Potential duplication** - Check if database `populate_summary_analytics` provides same metrics
- Consider caching results for repeated calls on same data
- **Recommendation:** Keep for ad-hoc DataFrame analysis, use DB-backed summaries for persisted schedules

---

#### `compute_correlations(df: &DataFrame, columns: &[String]) -> Result<DataFrame, PolarsError>`

**What it does:**
- **Currently returns empty DataFrame** (placeholder)
- Intended to compute Spearman correlation matrix

**Purpose:**
- Identify relationships between priority, visibility, duration, etc.

**Used By:**
- Not actively used (returns empty)

**Optimization Opportunities:**
- 🔴 **Remove or implement**
  - If correlation analysis is not needed, remove this function
  - If needed, implement using `ndarray-stats` or call Python's `scipy.stats.spearmanr` via PyO3
- **Recommendation:** **Remove** unless user stories require correlation heatmaps

---

#### `get_top_observations(df: &DataFrame, by: &str, n: usize) -> Result<DataFrame, PolarsError>`

**What it does:**
- Returns top N rows sorted by specified column (e.g., highest priority)

**Purpose:**
- Quickly identify high-priority or problematic observations

**Used By:**
- `python/algorithms.rs::py_get_top_observations()` → `tsi_rust.get_top_observations()`
- Debugging, data exploration notebooks

**Optimization Opportunities:**
- ✅ Efficient Polars operations
- Could add error handling for invalid column names
- **Recommendation:** Keep as-is, useful utility

---

### algorithms/conflicts.rs

#### `find_conflicts(df: &DataFrame) -> Result<Vec<SchedulingConflict>, PolarsError>`

**What it does:**
- Detects scheduling conflicts:
  - Observations scheduled outside visibility windows
  - Violations of fixed start/stop times

**Purpose:**
- Validate schedules, identify problematic assignments

**Used By:**
- `python/algorithms.rs::py_find_conflicts()` → `tsi_rust.find_conflicts()`
- Validation reports, schedule review dashboards

**Optimization Opportunities:**
- ⚠️ **Incomplete implementation** - Currently only checks fixed time constraints (lines 95-119), does not validate against visibility periods
- 🔴 **High priority fix:** Implement actual visibility window validation (parse `visibility_periods_parsed` column and check overlap)
- **Recommendation:** Complete the implementation or document limitations clearly

---

#### `suggest_candidate_positions(df: &DataFrame, row_index: usize) -> Result<Vec<CandidatePlacement>, PolarsError>`

**What it does:**
- **Currently returns empty vector** (placeholder)
- Intended to suggest alternative time slots for unscheduled observations

**Purpose:**
- Help schedulers find valid positions for failed observations

**Used By:**
- Not actively used (placeholder)

**Optimization Opportunities:**
- 🔴 **Remove or implement**
  - Real implementation would need:
    1. Parse visibility windows
    2. Check overlaps with scheduled blocks
    3. Generate candidate start times
    4. Validate constraints
- **Recommendation:** **Remove** unless actively developing scheduling suggestions feature

---

### algorithms/optimization.rs

#### `greedy_schedule(observations: &[Observation], constraints: &[Box<dyn Constraint>], max_iterations: usize) -> OptimizationResult`

**What it does:**
- Greedy scheduling algorithm: iteratively selects highest-priority observation that satisfies all constraints

**Purpose:**
- Baseline scheduling optimization
- Research/comparison with other algorithms

**Used By:**
- `python/algorithms.rs::py_greedy_schedule()` → `tsi_rust.greedy_schedule()`
- Optimization experiments, schedule generation

**Optimization Opportunities:**
- ⚠️ **Limited utility without constraints** - Currently only works with empty constraint list (baseline case)
- Need to implement concrete `Constraint` types (visibility overlap, time window, elevation, etc.)
- **Recommendation:** 
  - If optimization is not a core feature: **Keep as simple baseline**
  - If production scheduling needed: Expand constraint system or integrate existing solver (OR-Tools, HiGHS)

---

#### `greedy_schedule_parallel(observations: &[Observation], constraints: &[Box<dyn Constraint>], max_iterations: usize) -> OptimizationResult`

**What it does:**
- **Disabled** - Returns empty result with `converged=false`
- Intended for parallel greedy optimization using Rayon

**Purpose:**
- Speed up optimization for large datasets

**Used By:**
- Not used (disabled)

**Optimization Opportunities:**
- 🔴 **Remove**
  - Commented-out Rayon code suggests this was experimental
  - Parallel greedy is often not faster than sequential due to synchronization overhead
- **Recommendation:** **Remove** to reduce maintenance burden

---

## db Module

### db/mod.rs

**Purpose:** Database module orchestrator. Defines repository pattern architecture.

**Documentation Quality:** ✅ Excellent - includes ASCII architecture diagram

**Used By:**
- All Python database operations
- ETL pipeline
- Dashboard data fetching

**Optimization Opportunities:**
- ✅ Well-designed, no structural changes needed
- Architecture documentation should be referenced in onboarding docs

---

### db/checksum.rs

#### `calculate_checksum(content: &str) -> String`

**What it does:**
- Computes SHA256 hash of string content (hex-encoded)

**Purpose:**
- Detect duplicate schedule uploads
- Track data changes

**Used By:**
- `parsing/json_parser.rs::compute_schedule_checksum()`
- `db/services.rs::store_schedule()` for deduplication

**Optimization Opportunities:**
- ✅ Efficient, standard SHA256 implementation
- **Recommendation:** Keep as-is

---

### db/config.rs

**Purpose:** Database configuration management (connection strings, credentials)

**Key Types:**
- `DbConfig` - Holds connection parameters

**Used By:**
- `db/factory.rs` to create repository instances
- Application initialization

**Optimization Opportunities:**
- Ensure secrets are loaded from environment variables, not hardcoded
- ✅ Likely already handled correctly (check `from_env()` method)

---

### db/factory.rs

**Purpose:** Factory pattern for creating repository instances (Azure, Test, etc.)

**Key Functions:**
- `RepositoryFactory::create(repo_type: RepositoryType, config: Option<&DbConfig>)` - Creates repository instance

**Used By:**
- Application startup
- Python binding initialization
- Test setup

**Optimization Opportunities:**
- ✅ Good design
- Consider adding connection pooling configuration options
- **Recommendation:** Document connection pool sizing for production deployments

---

### db/models.rs

**Purpose:** Core domain models (2000+ lines). Defines:
- `Schedule`, `SchedulingBlock`, `Period`, `Constraints`
- `ScheduleInfo`, `ScheduleMetadata`
- Analytics models: `LightweightBlock`, `DistributionBlock`, `CompareBlock`, etc.
- PyO3 wrapper classes for Python exposure

**Used By:**
- Entire application - central data structures

**Optimization Opportunities:**
- ⚠️ **Large file (2000+ lines)** - Consider splitting into:
  - `models/schedule.rs` - Schedule, SchedulingBlock
  - `models/analytics.rs` - LightweightBlock, DistributionBlock, etc.
  - `models/metadata.rs` - ScheduleInfo, ScheduleMetadata
  - `models/python.rs` - PyO3 wrappers
- ✅ Otherwise well-structured with good documentation
- **Recommendation:** Modularize for maintainability if adding more model types

---

### db/repository.rs

**Purpose:** Defines `ScheduleRepository` trait (interface for all database operations)

**Key Methods (503 lines total):**
- `health_check()` - Connection health
- `store_schedule()`, `get_schedule()`, `list_schedules()` - Schedule CRUD
- `get_blocks_for_schedule()`, `get_scheduling_block()` - Block retrieval
- `populate_schedule_analytics()`, `fetch_analytics_blocks_for_*()` - Analytics operations
- `populate_summary_analytics()`, `fetch_schedule_summary()` - Summary statistics
- `populate_visibility_time_bins()`, `fetch_visibility_metadata()` - Visibility data

**Used By:**
- `db/services.rs` - Business logic layer
- `db/repositories/azure/` - Concrete Azure implementation
- `db/repositories/test.rs` - Mock for testing

**Optimization Opportunities:**
- ⚠️ **Large trait (30+ methods)** - Consider splitting into smaller traits:
  - `ScheduleRepository` (core CRUD)
  - `AnalyticsRepository` (analytics operations)
  - `ValidationRepository` (validation data)
- **Recommendation:** Evaluate if all methods are used; archive unused ones

---

### db/services.rs

**Purpose:** High-level service layer with business logic (400+ lines)

**Key Functions:**
- `store_schedule()` - Orchestrates schedule storage + analytics population
- `store_schedule_with_options()` - Allows skipping expensive analytics (time bins)
- `list_schedules()`, `get_schedule()` - Pass-through to repository
- `health_check()` - Database health verification

**Used By:**
- `python/database.rs` - All Python database operations
- Application controllers

**Optimization Opportunities:**
- ✅ Well-designed orchestration layer
- **Good practice:** Separates business logic from database implementation
- **Recommendation:** Keep as-is, excellent example of service layer pattern

---

### db/repositories/azure/

**Purpose:** Azure PostgreSQL implementation of `ScheduleRepository` trait

**Structure:**
- Multiple files implementing different aspects (operations, analytics, validation, etc.)
- Connection pooling via `pool.rs`

**Used By:**
- Production application
- ETL pipeline

**Optimization Opportunities:**
- Review SQL query performance (add indexes, optimize joins)
- Consider materialized views for expensive analytics queries
- **Recommendation:** Run `EXPLAIN ANALYZE` on slow queries during profiling

---

### db/repositories/test.rs

**Purpose:** In-memory mock implementation for unit testing

**Used By:**
- Unit tests
- Local development without database

**Optimization Opportunities:**
- ✅ Useful for testing
- **Recommendation:** Ensure all repository methods are implemented (check for `unimplemented!()` placeholders)

---

## parsing Module

### parsing/json_parser.rs

**Purpose:** Parse observation schedules from JSON format (471 lines)

**Key Functions:**

#### `parse_schedule_json_str(json_schedule_json: &str, possible_periods_json: Option<&str>, dark_periods_json: &str) -> Result<Schedule>`

**What it does:**
- Parses schedule JSON string into `Schedule` domain model
- Handles scheduling blocks, dark periods, visibility periods

**Purpose:**
- ETL ingestion from JSON files
- Schedule upload endpoint

**Used By:**
- `python/database.rs::py_store_schedule()`
- File-based ETL scripts

**Optimization Opportunities:**
- ✅ Robust error handling with `anyhow::Context`
- Consider streaming parser for very large files (>100MB)
- **Recommendation:** Add benchmarks for typical file sizes

---

#### `parse_schedule_json(schedule_json_path: &Path, possible_periods_json_path: Option<&Path>, dark_periods_json_path: &Path) -> Result<Schedule>`

**What it does:**
- File-based wrapper for `parse_schedule_json_str()`

**Purpose:**
- Convenience function for file input

**Used By:**
- Command-line tools
- Batch processing scripts

**Optimization Opportunities:**
- ✅ Good separation of file I/O and parsing logic
- **Recommendation:** Keep for ergonomics

---

#### Helper functions (parse_dark_periods_from_str, parse_possible_periods_from_str, parse_scheduling_blocks_from_str, etc.)

**What they do:**
- Parse specific JSON structures (dark periods, visibility windows, constraints, coordinates)

**Purpose:**
- Modular parsing logic, reusable components

**Optimization Opportunities:**
- ✅ Well-factored
- Consider adding JSON schema validation before parsing
- **Recommendation:** Document expected JSON structure in examples/

---

## python Module

### python/mod.rs

**Purpose:** Module orchestrator for Python bindings

**Exports all sub-modules:**
- `algorithms` - Analytics functions
- `database` - DB operations
- `time_bindings` - MJD ↔ datetime
- `transformations` - Data cleaning/filtering

**Used By:**
- `lib.rs::tsi_rust()` module registration

**Optimization Opportunities:**
- ✅ Clean organization
- **Recommendation:** Keep as-is

---

### python/algorithms.rs

**Purpose:** Python bindings for analytics algorithms (250+ lines)

**Key Functions:**

#### `py_compute_metrics(df: PyDataFrame) -> PyResult<PyAnalyticsSnapshot>`

**What it does:**
- Python wrapper for `algorithms::analysis::compute_metrics()`

**Used By:**
- Python: `tsi_rust.compute_metrics(df)`
- Dashboard analytics views

**Optimization Opportunities:**
- ✅ Efficient DataFrame passing via `pyo3-polars`
- **Recommendation:** Keep as-is

---

#### `py_get_top_observations(df: PyDataFrame, by: &str, n: usize) -> PyResult<PyDataFrame>`

**What it does:**
- Python wrapper for `algorithms::analysis::get_top_observations()`

**Used By:**
- Python: `tsi_rust.get_top_observations(df, "priority", 10)`
- Data exploration, debugging

**Optimization Opportunities:**
- ✅ Simple, efficient
- **Recommendation:** Keep

---

#### `py_find_conflicts(df: PyDataFrame) -> PyResult<Vec<PySchedulingConflict>>`

**What it does:**
- Python wrapper for `algorithms::conflicts::find_conflicts()`

**Used By:**
- Python: `tsi_rust.find_conflicts(df)`
- Validation reports

**Optimization Opportunities:**
- ⚠️ **Depends on incomplete underlying function** (see `algorithms/conflicts.rs::find_conflicts`)
- **Recommendation:** Fix or document limitations

---

#### `py_greedy_schedule(priorities: Vec<f64>, max_iterations: usize) -> PyResult<PyOptimizationResult>`

**What it does:**
- Python wrapper for `algorithms::optimization::greedy_schedule()`

**Used By:**
- Python: `result = tsi_rust.greedy_schedule(priorities, max_iter=1000)`
- Optimization experiments

**Optimization Opportunities:**
- ⚠️ Limited utility without constraint system
- **Recommendation:** Expand or mark as experimental

---

### python/database.rs

**Purpose:** Python bindings for all database operations (900+ lines)

**Key Functions (40+ Python-facing functions):**

#### `py_init_database() -> PyResult<()>`

**What it does:**
- Initializes global database connection pool

**Used By:**
- Python: `tsi_rust.py_init_database()` at application startup

**Optimization Opportunities:**
- ✅ Critical for connection reuse
- **Recommendation:** Document required environment variables

---

#### `py_store_schedule(schedule_json: &str, possible_periods_json: Option<&str>, dark_periods_json: &str, name: String, populate_analytics: bool, skip_time_bins: bool) -> PyResult<PyScheduleMetadata>`

**What it does:**
- Parse JSON → store in database → populate analytics

**Used By:**
- Python: `metadata = tsi_rust.py_store_schedule(...)`
- Schedule upload API endpoint

**Optimization Opportunities:**
- ⚠️ **Long function** - Consider extracting parsing and storage logic
- ✅ Good control over analytics computation (skip_time_bins flag)
- **Recommendation:** Keep current design, works well in production

---

#### `py_list_schedules() -> PyResult<PyObject>`

**What it does:**
- Returns list of all schedules with metadata

**Used By:**
- Python: `schedules = tsi_rust.py_list_schedules()`
- Dashboard schedule selector

**Optimization Opportunities:**
- ✅ Efficient
- Consider adding pagination for large deployments (1000+ schedules)
- **Recommendation:** Add pagination if needed in future

---

#### Analytics functions (py_populate_analytics, py_has_analytics_data, py_delete_analytics, etc.)

**What they do:**
- Manage pre-computed analytics tables

**Purpose:**
- Speed up dashboard queries by denormalizing data

**Used By:**
- Python dashboard views (sky map, distributions, trends, etc.)

**Optimization Opportunities:**
- ✅ Essential for performance
- Consider background job system for analytics population on large uploads
- **Recommendation:** Document analytics lifecycle (when to recompute)

---

#### Visibility functions (py_populate_visibility_time_bins, py_get_visibility_histogram, etc.)

**What they do:**
- Compute and retrieve visibility histograms and time bins

**Purpose:**
- Visualization of observation opportunities over time

**Used By:**
- Python dashboard: visibility charts, timeline views

**Optimization Opportunities:**
- ⚠️ **Known slow operation** (minutes for large schedules) - documented in code
- Consider incremental computation or caching strategies
- **Recommendation:** Acceptable for current use; optimize if it becomes bottleneck

---

### python/time_bindings.rs

**Purpose:** Time conversion utilities

**Key Functions:**

#### `mjd_to_datetime(py: Python, mjd: f64) -> PyResult<Py<PyAny>>`

**What it does:**
- Converts Modified Julian Date to Python datetime

**Used By:**
- Python: `dt = tsi_rust.mjd_to_datetime(59000.0)`
- Timeline visualizations, human-readable dates

**Optimization Opportunities:**
- ✅ Efficient using `chrono` and `siderust`
- **Recommendation:** Keep as-is

---

#### `datetime_to_mjd(dt: &Bound<'_, PyAny>) -> PyResult<f64>`

**What it does:**
- Converts Python datetime to Modified Julian Date

**Used By:**
- Python: `mjd = tsi_rust.datetime_to_mjd(datetime_obj)`
- Input validation, time range queries

**Optimization Opportunities:**
- ✅ Efficient
- **Recommendation:** Keep

---

### python/transformations.rs

**Purpose:** Python bindings for data cleaning and filtering

**Key Functions:**

#### `py_remove_duplicates(df: PyDataFrame, subset: Option<Vec<String>>, keep: &str) -> PyResult<PyDataFrame>`

**What it does:**
- Remove duplicate rows from DataFrame

**Used By:**
- Python: `clean_df = tsi_rust.py_remove_duplicates(df, ["schedulingBlockId"], "first")`
- ETL preprocessing

**Optimization Opportunities:**
- ✅ Efficient Polars operations
- **Recommendation:** Keep

---

#### `py_filter_dataframe(df: PyDataFrame, priority_min: f64, priority_max: f64, scheduled_filter: &str, priority_bins: Option<Vec<String>>, block_ids: Option<Vec<String>>) -> PyResult<PyDataFrame>`

**What it does:**
- Multi-criteria DataFrame filtering (priority, schedule status, bins, IDs)

**Used By:**
- Python: `filtered = tsi_rust.py_filter_dataframe(df, 0, 10, "Scheduled", None, None)`
- Dashboard filters, interactive data exploration

**Optimization Opportunities:**
- ✅ Efficient
- Consider adding more filter criteria if needed (coordinates, duration, etc.)
- **Recommendation:** Keep, extend as needed

---

## services Module

### services/mod.rs

**Purpose:** Service layer orchestrator for business logic

**Exports:**
- `compare::py_get_compare_data`
- `distributions::{py_get_distribution_data, py_get_distribution_data_analytics}`
- `insights::py_get_insights_data`
- `sky_map::{py_get_sky_map_data, py_get_sky_map_data_analytics}`
- `timeline::py_get_schedule_timeline_data`
- `trends::py_get_trends_data`
- `validation_report::{py_get_validation_report, PyValidationIssue, PyValidationReportData}`

**Used By:**
- Python dashboard components

**Optimization Opportunities:**
- ✅ Well-organized
- **Recommendation:** Keep structure, good separation of concerns

---

### services/compare.rs

#### `compute_compare_data(current_blocks: Vec<CompareBlock>, comparison_blocks: Vec<CompareBlock>, current_name: String, comparison_name: String) -> Result<CompareData, String>`

**What it does:**
- Compares two schedules: identifies common blocks, differences, scheduling changes
- Computes statistics for both schedules

**Purpose:**
- Schedule comparison dashboard
- Track changes between iterations

**Used By:**
- Python: `data = tsi_rust.py_get_compare_data(schedule_id_1, schedule_id_2, name1, name2)`
- Dashboard comparison view

**Optimization Opportunities:**
- ✅ Efficient HashSet operations for set differences
- Consider caching comparison results for frequently compared schedules
- **Recommendation:** Keep as-is, works well

---

### services/distributions.rs

#### `compute_distribution_data(blocks: Vec<DistributionBlock>) -> Result<DistributionData, String>`

**What it does:**
- Computes distribution data for priority, duration, visibility histograms
- Handles binning and statistical aggregation

**Purpose:**
- Dashboard distribution charts (priority distribution, duration distribution, etc.)

**Used By:**
- Python: `data = tsi_rust.py_get_distribution_data(schedule_id)`
- Dashboard distribution views

**Optimization Opportunities:**
- ✅ Efficient aggregation logic
- **Recommendation:** Keep as-is

---

### services/insights.rs

#### `compute_insights_data(blocks: Vec<InsightsBlock>) -> Result<InsightsData, String>`

**What it does:**
- Generates scheduling insights:
  - High-priority unscheduled blocks
  - Blocks with limited visibility
  - Scheduling inefficiencies
  - Recommendations

**Purpose:**
- Dashboard insights panel
- Actionable scheduling improvements

**Used By:**
- Python: `insights = tsi_rust.py_get_insights_data(schedule_id)`
- Dashboard insights view

**Optimization Opportunities:**
- ✅ Good business logic encapsulation
- Consider making insight rules configurable (thresholds, priorities)
- **Recommendation:** Keep, excellent feature for user guidance

---

### services/sky_map.rs

#### `compute_sky_map_data(blocks: Vec<LightweightBlock>) -> Result<SkyMapData, String>`

**What it does:**
- Prepares data for celestial coordinate sky map visualization
- Groups blocks by scheduled status

**Purpose:**
- Dashboard sky map view (RA/Dec scatter plot)

**Used By:**
- Python: `data = tsi_rust.py_get_sky_map_data(schedule_id)`
- Dashboard sky map component

**Optimization Opportunities:**
- ✅ Lightweight data structure
- Consider adding coordinate range filtering for zoomed views
- **Recommendation:** Keep as-is

---

### services/timeline.rs

#### `compute_schedule_timeline_data(blocks: Vec<TimelineBlock>, dark_periods: Vec<Period>) -> Result<ScheduleTimelineData, String>`

**What it does:**
- Prepares timeline visualization data (Gantt chart)
- Includes scheduled blocks and dark periods

**Purpose:**
- Dashboard timeline view
- Visualize schedule occupancy over time

**Used By:**
- Python: `data = tsi_rust.py_get_schedule_timeline_data(schedule_id)`
- Dashboard timeline component

**Optimization Opportunities:**
- ✅ Efficient data preparation
- Consider adding time range filtering for large schedules
- **Recommendation:** Keep as-is

---

### services/trends.rs

#### `compute_trends_data(blocks: Vec<TrendsBlock>) -> Result<TrendsData, String>`

**What it does:**
- Analyzes trends over time (if multiple schedule versions exist)
- Computes metrics evolution

**Purpose:**
- Dashboard trends view
- Track scheduling performance over iterations

**Used By:**
- Python: `data = tsi_rust.py_get_trends_data(schedule_ids)`
- Dashboard trends component

**Optimization Opportunities:**
- ✅ Good aggregation logic
- Consider adding forecasting/prediction features
- **Recommendation:** Keep as-is, extend if predictive analytics needed

---

### services/validation.rs

#### `validate_block(block: &BlockForValidation) -> Vec<ValidationResult>`

**What it does:**
- Validates a single scheduling block against rules:
  - **CRITICAL:** Zero visibility, insufficient visibility
  - **HIGH:** Negative priority, invalid durations, out-of-range coordinates
  - **MEDIUM/LOW:** Warnings for suboptimal configurations

**Purpose:**
- ETL validation stage
- Filter out impossible-to-schedule blocks
- Provide detailed error reports

**Used By:**
- Database validation table population
- Python validation report generation

**Optimization Opportunities:**
- ✅ Comprehensive rule set
- ✅ Well-categorized by criticality
- Consider making rules extensible (plugin system)
- **Recommendation:** Keep as-is, excellent validation framework

---

#### `validate_blocks(blocks: &[BlockForValidation]) -> Vec<ValidationResult>`

**What it does:**
- Batch validation wrapper for multiple blocks

**Used By:**
- ETL pipeline
- Bulk validation operations

**Optimization Opportunities:**
- ✅ Efficient batch processing
- Consider parallelizing with Rayon for large batches (10,000+ blocks)
- **Recommendation:** Add parallelization if validation becomes bottleneck

---

### services/validation_report.rs

#### `py_get_validation_report(schedule_id: i64) -> PyResult<PyValidationReportData>`

**What it does:**
- Fetches validation results from database
- Aggregates by category and criticality
- Prepares report for display

**Purpose:**
- Dashboard validation report view
- Show validation issues to users

**Used By:**
- Python: `report = tsi_rust.py_get_validation_report(schedule_id)`
- Dashboard validation page

**Optimization Opportunities:**
- ✅ Efficient aggregation
- **Recommendation:** Keep as-is

---

### services/visibility.rs

#### `compute_visibility_histogram_rust(metadata: Vec<VisibilityTimeMetadata>) -> PyResult<Vec<VisibilityHistogramPoint>>`

**What it does:**
- Computes visibility histogram over time
- Aggregates scheduled vs available visibility

**Purpose:**
- Dashboard visibility histogram chart

**Used By:**
- Python: `histogram = tsi_rust.py_get_visibility_histogram(...)`
- Dashboard visibility analytics

**Optimization Opportunities:**
- ✅ Efficient binning
- **Recommendation:** Keep as-is

---

## transformations Module

### transformations/cleaning.rs

**Purpose:** Data cleaning utilities for Polars DataFrames

**Key Functions:**

#### `remove_duplicates(df: &DataFrame, subset: Option<Vec<String>>, keep: &str) -> PolarsResult<DataFrame>`

**What it does:**
- Removes duplicate rows based on subset of columns
- Supports "first", "last", "none" keep strategies

**Used By:**
- ETL preprocessing
- Data quality pipeline

**Optimization Opportunities:**
- ✅ Efficient Polars implementation
- **Recommendation:** Keep

---

#### `remove_missing_coordinates(df: &DataFrame) -> PolarsResult<DataFrame>`

**What it does:**
- Filters out rows with null RA or Dec

**Used By:**
- ETL preprocessing

**Optimization Opportunities:**
- ✅ Simple, efficient
- **Recommendation:** Keep

---

#### `impute_missing(series: &Series, strategy: &str, fill_value: Option<f64>) -> PolarsResult<Series>`

**What it does:**
- Imputes missing values using mean, median, or constant strategy

**Used By:**
- Data cleaning pipelines

**Optimization Opportunities:**
- ⚠️ **Median strategy uses mean** (bug on line 58) - should be `FillNullStrategy::Median` if available
- **Recommendation:** Fix median implementation or document limitation

---

#### `validate_schema(df: &DataFrame, required_columns: Vec<String>, expected_dtypes: Option<Vec<(String, DataType)>>) -> PolarsResult<(bool, Vec<String>)>`

**What it does:**
- Validates DataFrame has required columns and correct types

**Used By:**
- ETL validation
- Input validation

**Optimization Opportunities:**
- ✅ Essential validation
- **Recommendation:** Keep, consider adding to ETL error handling

---

### transformations/filtering.rs

**Purpose:** DataFrame filtering utilities

**Key Functions:**

#### `filter_by_column(df: &DataFrame, column: &str, value: &str) -> PolarsResult<DataFrame>`

**What it does:**
- Filters DataFrame by exact string match on column

**Used By:**
- Dashboard filters

**Optimization Opportunities:**
- ✅ Simple, works well
- **Recommendation:** Keep

---

#### `filter_by_range(df: &DataFrame, column: &str, min_value: f64, max_value: f64) -> PolarsResult<DataFrame>`

**What it does:**
- Filters DataFrame by numeric range [min, max]

**Used By:**
- Dashboard priority filters, date range filters

**Optimization Opportunities:**
- ✅ Efficient
- **Recommendation:** Keep

---

#### `filter_by_scheduled(df: &DataFrame, filter_type: &str) -> PolarsResult<DataFrame>`

**What it does:**
- Filters by schedule status: "All", "Scheduled", "Unscheduled"

**Used By:**
- Dashboard schedule status filters

**Optimization Opportunities:**
- ✅ Clean implementation
- **Recommendation:** Keep

---

#### `filter_dataframe(df: &DataFrame, priority_min: f64, priority_max: f64, scheduled_filter: &str, priority_bins: Option<Vec<String>>, block_ids: Option<Vec<String>>) -> PolarsResult<DataFrame>`

**What it does:**
- Combines multiple filters (priority range, schedule status, bins, IDs)

**Used By:**
- Dashboard advanced filtering

**Optimization Opportunities:**
- ✅ Flexible, composable filters
- **Recommendation:** Keep as-is

---

#### `validate_dataframe(df: &DataFrame) -> (bool, Vec<String>)`

**What it does:**
- Validates DataFrame data quality:
  - Missing IDs
  - Invalid priority
  - Out-of-range coordinates (RA: [0, 360), Dec: [-90, 90])

**Used By:**
- ETL validation
- Input sanity checks

**Optimization Opportunities:**
- ✅ Essential data quality checks
- **Recommendation:** Keep, ensure used in all data ingestion paths

---

## Optimization Recommendations

### High Priority

1. **🔴 Fix `algorithms/conflicts.rs::find_conflicts()`**
   - Currently incomplete: only checks fixed time constraints
   - Missing: visibility window validation
   - **Impact:** Validation reports may miss conflicts
   - **Recommendation:** Complete implementation or document as "fixed-time-only"

2. **🔴 Remove placeholder functions**
   - `algorithms/analysis.rs::compute_correlations()` - returns empty
   - `algorithms/conflicts.rs::suggest_candidate_positions()` - returns empty
   - `algorithms/optimization.rs::greedy_schedule_parallel()` - disabled
   - **Impact:** Dead code, confusing API
   - **Recommendation:** Remove or mark as `#[doc(hidden)]` experimental

3. **🔴 Fix `transformations/cleaning.rs::impute_missing()` median bug**
   - Line 58: median strategy uses `FillNullStrategy::Mean`
   - **Impact:** Incorrect imputation
   - **Recommendation:** Use correct median strategy or document limitation

### Medium Priority

4. **⚠️ Split large files**
   - `db/models.rs` (2000+ lines) → split into submodules
   - `python/database.rs` (900+ lines) → consider grouping by feature
   - **Impact:** Maintainability, code navigation
   - **Recommendation:** Refactor when adding new model types

5. **⚠️ Add pagination to `py_list_schedules()`**
   - Current: returns all schedules
   - **Impact:** Performance degradation at scale (1000+ schedules)
   - **Recommendation:** Add pagination parameters when needed

6. **⚠️ Profile and optimize analytics queries**
   - Review SQL performance in `db/repositories/azure/`
   - **Impact:** Dashboard load times
   - **Recommendation:** Run `EXPLAIN ANALYZE`, add indexes, consider materialized views

### Low Priority (Future Enhancements)

7. **💡 Extend constraint system in optimization**
   - Current: greedy scheduler works but lacks constraint types
   - **Impact:** Limited optimization capabilities
   - **Recommendation:** Implement if optimization becomes core feature

8. **💡 Add caching for comparison results**
   - `services/compare.rs` could cache frequently compared schedules
   - **Impact:** Minor speedup for repeated comparisons
   - **Recommendation:** Add if users repeatedly compare same pairs

9. **💡 Make validation rules configurable**
   - `services/validation.rs` has hardcoded thresholds
   - **Impact:** Flexibility for different observatories
   - **Recommendation:** Add configuration system if multi-tenancy needed

10. **💡 Parallelize validation for large batches**
    - `services/validation.rs::validate_blocks()` is sequential
    - **Impact:** Speedup for 10,000+ block schedules
    - **Recommendation:** Use Rayon if validation becomes bottleneck

---

## Summary Statistics

### Code Volume
- **Total Rust files:** ~40 source files
- **Total lines (estimated):** ~12,000 lines
- **Largest files:**
  - `db/models.rs`: ~2000 lines
  - `python/database.rs`: ~900 lines
  - `parsing/json_parser.rs`: ~471 lines
  - `services/trends.rs`: ~430 lines

### Module Breakdown
- **algorithms:** 3 files, ~700 lines - Analytics and optimization
- **db:** 8+ files, ~4000+ lines - Database layer (largest module)
- **parsing:** 1 file, ~471 lines - JSON parsing
- **python:** 4 files, ~1500+ lines - Python bindings
- **services:** 9 files, ~2000+ lines - Business logic
- **transformations:** 2 files, ~300 lines - Data processing utilities

### Function Count
- **Public functions exported to Python:** ~60 functions
- **Internal functions:** ~150+ functions

### Functionality Coverage

✅ **Well-Implemented:**
- JSON parsing (robust error handling)
- Database operations (repository pattern, connection pooling)
- Analytics computation (metrics, distributions, insights)
- Data validation (comprehensive rule set)
- Time conversions (MJD ↔ datetime)
- DataFrame transformations (cleaning, filtering)

⚠️ **Partially Implemented:**
- Conflict detection (only fixed time constraints)
- Optimization (limited constraint system)

🔴 **Placeholder/Incomplete:**
- Correlation analysis (returns empty)
- Candidate position suggestions (returns empty)
- Parallel greedy scheduling (disabled)

### Usage Patterns

**Most-Used Modules:**
1. `db/` - All database operations (schedule CRUD, analytics)
2. `python/database.rs` - Primary interface for Python app
3. `services/` - Dashboard data preparation
4. `parsing/json_parser.rs` - ETL ingestion

**Rarely-Used Modules:**
- `algorithms/optimization.rs` - Experimental, not production
- `algorithms/conflicts.rs::suggest_candidate_positions()` - Placeholder

### Performance Characteristics

**Fast Operations (<100ms):**
- Metrics computation (`compute_metrics`)
- DataFrame filtering
- Coordinate transformations
- Checksum calculation

**Medium Operations (100ms - 1s):**
- Schedule parsing (~1000 blocks)
- Analytics population (block-level)
- Validation (batch of 1000 blocks)

**Slow Operations (>1s):**
- Visibility time bins (~minutes for large schedules) - **documented as known limitation**
- Large schedule uploads (>5000 blocks)

---

## Conclusion

The Rust backend is **well-architected** with good separation of concerns (repository pattern, service layer, Python bindings). The code quality is high with comprehensive error handling and documentation.

### Strengths
1. ✅ Clean module structure
2. ✅ Efficient Polars DataFrame operations
3. ✅ Robust JSON parsing with error context
4. ✅ Comprehensive validation framework
5. ✅ Good use of async/await for database operations
6. ✅ Excellent Python integration via PyO3

### Areas for Improvement
1. 🔴 Remove dead code (placeholder functions)
2. 🔴 Fix bugs (conflict detection, median imputation)
3. ⚠️ Modularize large files for maintainability
4. ⚠️ Document known performance limitations
5. 💡 Extend features (optimization constraints, configurable validation)

### Who Uses What
- **Python TSI application:** Everything (primary consumer)
- **Dashboard:** `services/*` functions heavily
- **ETL pipeline:** `parsing/*`, `db/services.rs`, `validation`
- **Data analysts:** `algorithms/analysis.rs`, `transformations/*`
- **Rarely used:** Optimization, conflict suggestions

### Removal Candidates
- `algorithms/analysis.rs::compute_correlations()` - if not planned
- `algorithms/conflicts.rs::suggest_candidate_positions()` - if not planned
- `algorithms/optimization.rs::greedy_schedule_parallel()` - disabled
- Evaluate if optimization module is needed at all (if not production feature)

---

**Document Version:** 1.0  
**Last Updated:** December 16, 2025  
**Maintainer:** TSI Development Team
