# Streamlit Cleanup Report

## Step 0 - Entrypoints, Routing, Config, Backend Integration

### Entrypoints / Run Commands
- `streamlit run src/tsi/app.py` (README.md, docs/SETUP.md, scripts/run_dashboard.sh)
- Dockerfile runs: `streamlit run src/tsi/app.py --server.address=0.0.0.0 --server.port=8501`

### App Structure / Navigation
- Entrypoint: `src/tsi/app.py`
- Custom navigation in `src/tsi/routing.py` (button-based navigation, not `pages/` directory)
- Pages list from Settings (`src/app_config/settings.py`):
  - Validation
  - Sky Map
  - Distributions
  - Visibility Map
  - Schedule
  - Insights
  - Trends
  - Compare
- Page modules live in `src/tsi/pages/`:
  - `landing.py` (shown when no data)
  - `validation_report.py`
  - `sky_map.py`
  - `distributions.py`
  - `visibility_map.py`
  - `scheduled_timeline.py`
  - `insights.py`
  - `scheduling_trends.py`
  - `compare_schedules.py`

### Configuration Sources
- `.streamlit/config.toml` (Streamlit UI/server defaults)
- `.env` / environment variables (via `Settings` in `src/app_config/settings.py`)
  - `DATA_ROOT`, `CACHE_TTL`, `MAX_WORKERS`, backend env vars like `DATABASE_URL` / `PG_*`

### Backend Integration Points (Rust backend via PyO3)
- Backend access via `tsi.services` -> `src/tsi/services/backend_service.py`
- UI endpoints used by pages:
  - Landing: `upload_schedule`, `list_schedules` (POST_SCHEDULE, LIST_SCHEDULES)
  - Validation: `get_validation_report` (GET_VALIDATION_REPORT)
  - Sky Map: `get_sky_map_data` (GET_SKY_MAP_DATA)
  - Distributions: `get_distribution_data` (GET_DISTRIBUTION_DATA)
  - Visibility Map: `get_visibility_map_data`, `get_visibility_histogram`, `get_schedule_time_range` (GET_VISIBILITY_MAP_DATA, GET_VISIBILITY_HISTOGRAM, GET_SCHEDULE_TIME_RANGE, py_get_visibility_histogram_analytics)
  - Schedule Timeline: `get_schedule_timeline_data` (GET_SCHEDULE_TIMELINE_DATA)
  - Insights: `get_insights_data` (GET_INSIGHTS_DATA)
  - Trends: `get_trends_data` (GET_TRENDS_DATA)
  - Compare: `get_compare_data` (GET_COMPARE_DATA)
  - Supporting fetchers: `py_fetch_dark_periods`, `py_fetch_possible_periods`

## Step 1-3 - Removed Items (Evidence + Rationale)

### UI / Components
- Removed re-export shims that are unused:
  - `src/tsi/components/__init__.py` (no `from tsi.components import ...` imports in repo)
  - `src/tsi/components/shared/__init__.py` (no `from tsi.components.shared import ...` imports)
- Removed unused filter helpers from `src/tsi/components/shared/filters.py`:
  - `render_exclude_impossible_checkbox`
  - `render_exclude_zero_visibility_checkbox`
  - Evidence: no references in `src/` (only defined, never imported)

### Services / Utilities
- Removed unused backend facade:
  - `src/tsi/services/backend_client.py`
  - Evidence: no imports in `src/`; only tests referenced it, now updated to use `tsi.services.upload_schedule`
- Removed unused visibility cache + data preparation pipeline:
  - `src/tsi/services/utils/visibility_cache.py`
  - `src/tsi/services/data/preparation.py`
  - Evidence: no imports in `src/` after removal; only referenced by deleted tests/benchmarks/manual scripts

### Legacy Session State
- Removed legacy `schedule_id` migration in `src/tsi/state.py`
  - Evidence: only reference to the old key was the migration block; no other usage in UI flows

### Tests / Benchmarks Tied to Removed Data Prep
- Removed files only exercising the deleted data preparation pipeline:
  - `tests/tsi/services/test_loaders.py`
  - `tests/manual/test_streamlit_json_loading.py`
  - `tests/benchmarks/benchmark_visibility_strategies.py`
  - `tests/benchmarks/benchmark_loading.py`
  - `tests/core/test_transformations.py`
  - `tests/core/test_preparation_comprehensive.py`
- Updated `tests/test_upload_performance.py` to call `tsi.services.upload_schedule`
- Updated `tests/test_config.py` to drop expectations for removed settings
- Updated `tests/test_analytics_fix.py` skip reason to reflect new API name

### Config Cleanup
- Removed unused `Settings` fields in `src/app_config/settings.py`:
  - `artifacts_dir`, `enable_file_upload`, `enable_comparison`, `use_analytics_table`
  - Evidence: no references in `src/` or tests (post-cleanup)

### Dependency Cleanup
- Removed unused deps from `requirements.base.txt`:
  - `altair`, `tabulate`, `shap`, `pyyaml`, `joblib`, `matplotlib`, `seaborn`
  - Evidence: no imports in `src/` or tests; only notebook references for matplotlib/seaborn
- Removed `tabulate` from `pyproject.toml` runtime deps (not imported anywhere)

## Validation

Commands executed:
- `python -m compileall .`
- `ruff check .` (not available: `ruff` binary missing)
- `pytest` (pass)
- `timeout 8s streamlit run src/tsi/app.py --server.headless true --server.port 8501`
  - App started successfully and was stopped by timeout (no interactive smoke clicks in CLI)

## Notes
- Streamlit UI flows remain intact: routing still uses `PAGES` + `route_to_page` and all referenced pages/components are unchanged.
- Backend usage remains intact: all UI-facing service functions still call the Rust backend as before.
