# Telescope Scheduling Intelligence

Analyze and visualize astronomical scheduling outputs with an interactive Streamlit app, a reusable preprocessing library, examples, and notebooks. https://telescope-scheduling-intelligence.streamlit.app

![Python](https://img.shields.io/badge/python-3.10%2B-blue.svg)
![Streamlit](https://img.shields.io/badge/streamlit-1.31%2B-ff4b4b.svg)
![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)

## What’s inside

- Streamlit dashboard with pages for Sky Map, Distributions, Visibility Map, Scheduled Timeline, Insights, Trends, and Compare
- JSON/CSV loaders and preprocessing pipeline (fast, consistent, validated)
- Optional Dark Periods overlay to distinguish observable vs non-observable time windows
- Scripts and examples for batch preprocessing and data exploration
- Tests (unit + e2e) and a Docker image for reproducible runs

## Repository layout (key files)

```
.
├── data/
│   ├── schedule.csv              # Sample preprocessed dataset (used by the app)
│   ├── schedule.json             # Raw schedule (example)
│   ├── possible_periods.json     # Optional visibility periods
│   └── dark_periods.json         # Optional dark periods (auto-detected by the app)
├── examples/
│   ├── example_data_loading.py   # Loaders usage
│   └── example_preprocessing.py  # Preprocessor usage
├── notebooks/
│   ├── eda.ipynb
│   └── scheduling_trends.ipynb
├── scripts/
│   ├── preprocess_schedules.py   # CLI: JSON → CSV (single/batch)
│   └── train_model.py            # Modeling pipeline entrypoint
├── src/
│   ├── core/
│   │   ├── loaders/              # JSON/CSV/data-dir loaders
│   │   └── preprocessing/        # SchedulePreprocessor + helpers
│   └── tsi/
│       ├── app.py                # Streamlit entrypoint
│       ├── routing.py, state.py, theme.py
│       ├── pages/                # Sky Map, Distributions, Visibility Map, Schedule, Insights, Trends, Compare
│       ├── plots/, components/, services/
│       └── assets/styles.css
├── run_dashboard.sh              # Local launcher (venv + streamlit)
├── streamlit_app.py              # Streamlit Cloud entry (imports tsi.app.main)
├── Dockerfile
├── pyproject.toml
├── requirements.txt
└── tests/
    ├── core/, e2e/, manual/
    └── benchmarks/
```

## Quickstart

Prereqs: Python 3.10+ and pip.

```bash
# Install
pip install -r requirements.txt

# Run the Streamlit app
streamlit run src/tsi/app.py

# Or use the helper
./run_dashboard.sh
```

The app opens at http://localhost:8501. On the landing page you can:
- Upload a preprocessed CSV (fastest), or
- Upload a raw schedule.json (+ optional possible_periods.json). JSON is processed in‑memory, or
- Load the bundled sample dataset at `data/schedule.csv`.

Dark periods: if `data/dark_periods.json` exists, it is auto‑loaded; you can also upload it later on the landing page. The Scheduled Timeline page then shades nighttime (observable) vs daytime (non‑observable) periods.

## Preprocess JSON → CSV (recommended for performance)

The dashboard can process JSON directly, but for repeat analysis and faster loads prefer CSV precomputation using the CLI.

```bash
# Single file
python scripts/preprocess_schedules.py \
  --schedule data/schedule.json \
  --output data/schedule.csv

# With visibility/possible periods
python scripts/preprocess_schedules.py \
  --schedule data/schedule.json \
  --visibility data/possible_periods.json \
  --output data/schedule.csv \
  --verbose

# Batch directory
python scripts/preprocess_schedules.py \
  --batch-dir data/schedules \
  --output-dir data/preprocessed

# Batch with custom patterns
python scripts/preprocess_schedules.py \
  --batch-dir data/schedules \
  --pattern "schedule_*.json" \
  --visibility-pattern "possible_periods*.json" \
  --output-dir data/preprocessed
```

Examples: see `examples/example_data_loading.py` and `examples/example_preprocessing.py`.

## Data schema (CSV expected by the app)

Required columns (from `src/tsi/config.py`):
- schedulingBlockId, priority, minObservationTimeInSec, requestedDurationSec
- fixedStartTime, fixedStopTime, decInDeg, raInDeg
- minAzimuthAngleInDeg, maxAzimuthAngleInDeg, minElevationAngleInDeg, maxElevationAngleInDeg
- scheduled_period.start, scheduled_period.stop
- visibility, num_visibility_periods, total_visibility_hours, priority_bin
- scheduled_flag, requested_hours, elevation_range_deg

Notes
- Times are MJD in the raw JSON; the app converts to UTC timestamps for display.
- The `visibility` column is a list of (start, stop) MJD pairs; when stored in CSV it’s stringified.

## Dashboard pages

- Sky Map: RA/Dec scatter with color by priority or status, size by requested hours, priority and time filters
- Distributions: histograms and summary distributions (priority, visibility hours, requested duration, elevation range)
- Visibility Map: visualize visibility windows and constraints
- Scheduled Timeline: month‑by‑month view of scheduled observations; optional dark/daytime overlays; CSV export
- Insights: scheduling rates, correlations, integrity checks, and top lists
- Trends: time evolution of scheduling metrics
- Compare: load a second CSV to compare two schedules side‑by‑side

## Configuration

Runtime settings are managed via `pydantic-settings` in `src/app_config/settings.py` and can be overridden with environment variables or a `.env` file at repo root.

Key variables
- DATA_ROOT: base data directory (default: data)
- SAMPLE_DATASET: path to the sample CSV (default: data/schedule.csv)
- CACHE_TTL_SECONDS: cache TTL for loaders (default: 600)

Example `.env`
```
SAMPLE_DATASET=data/schedule.csv
DATA_ROOT=data
CACHE_TTL_SECONDS=900
```

## Run tests and quality gates

```bash
# Unit + e2e tests
pytest

# Optional dev tools (install with: pip install -e ".[dev]")
ruff check src/ tests/
black --check src/ tests/
mypy src/
```

## Docker

```bash
# Build
docker build -t tsi-app .

# Run dashboard (http://localhost:8501)
docker run --rm -p 8501:8501 tsi-app

# Run tests inside the same image
docker run --rm tsi-app pytest
```

The image defaults to launching the dashboard; overriding the command lets you reuse it for CI.

## Development notes

- Add pages under `src/tsi/pages/` and register them in `src/tsi/routing.py`.
- Plots live in `src/tsi/plots/`, components in `src/tsi/components/`, services in `src/tsi/services/`.
- JSON/CSV parsing and preprocessing are under `src/core/`.

## License

AGPL-3.0 — see `LICENSE` for details.

---

Built with Streamlit, Plotly, pandas, and modern Python tooling.
