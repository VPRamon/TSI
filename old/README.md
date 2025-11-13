# Legacy Streamlit Application Archive

This directory contains the archived Streamlit-based version of the Telescope Scheduling Intelligence application, which has been superseded by the modern Rust + Vue.js architecture.

## Contents

### Application Files
- **`streamlit-app/`** - Main Streamlit application (`src/tsi/`)
  - `app.py` - Application entrypoint
  - `pages/` - Dashboard pages (Sky Map, Distributions, Visibility Map, etc.)
  - `components/` - Reusable UI components
  - `plots/` - Plotly visualization modules
  - `services/` - Business logic services
  - `routing.py`, `state.py`, `theme.py` - Core app infrastructure

- **`app/`** - Alternative app directory structure

- **`.streamlit/`** - Streamlit configuration
  - `config.toml` - Theme and server settings

### Configuration & Build
- **`Dockerfile`** - Python/Streamlit Docker image
- **`pyproject.toml`** - Python project metadata
- **`requirements.txt`** - Python dependencies
- **`.dockerignore`** - Docker build exclusions
- **`run_dashboard.sh`** - Local development launcher script

### Code & Scripts
- **`examples/`** - Usage examples for data loading and preprocessing
  - `example_data_loading.py`
  - `example_preprocessing.py`

- **`scripts/`** - Batch processing scripts
  - `preprocess_schedules.py` - JSON to CSV conversion
  - `train_model.py` - ML model training pipeline

### Tests
- **`tests-tsi/`** - Streamlit app unit tests
- **`tests-e2e/`** - End-to-end UI tests
- **`tests-manual/`** - Manual test scripts
- **`tests-benchmarks/`** - Performance benchmarks

### Documentation
- **`README-streamlit.md`** - Original Streamlit README

## Why Was It Replaced?

The application was migrated from Streamlit to Rust + Vue for:

1. **Performance**: Rust backend with Polars provides 10-100x faster data processing
2. **Scalability**: Microservices architecture scales horizontally
3. **Modern Stack**: Vue 3 + TypeScript provides better developer experience
4. **Production Ready**: Better suited for deployment, monitoring, and maintenance

## Migration Documentation

See the `/migration-doc` directory in the project root for detailed migration documentation:
- `RUST_VUE_MIGRATION_PLAN.md` - Overall migration strategy
- `PHASE_*.md` - Phase-by-phase implementation logs

## Running the Legacy Application

If you need to run the legacy Streamlit version:

```bash
# From the project root
cd old

# Install dependencies
pip install -r requirements.txt

# Run the dashboard
streamlit run streamlit-app/app.py
```

Or use the helper script:
```bash
./run_dashboard.sh
```

## Historical Context

This version was:
- **Created**: Early 2024
- **Active**: Through October 2024
- **Archived**: November 2024
- **Final Version**: See git history for commit details

The Streamlit version successfully demonstrated:
- Interactive astronomical data visualization
- CSV/JSON data processing pipelines
- Multiple analytical views (Sky Map, Trends, Distributions, etc.)
- Dark period overlays for observability windows

All of this functionality is being ported to the new architecture with improved performance and maintainability.

---

**Note**: This archive is preserved for reference and comparison purposes. Active development occurs in the modern Rust/Vue implementation at the project root.
