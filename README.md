# Telescope Scheduling Intelligence# Telescope Scheduling Intelligence



Modern web application for analyzing and visualizing astronomical scheduling outputs with a high-performance Rust backend and Vue.js frontend.Analyze and visualize astronomical scheduling outputs with an interactive Streamlit app, a reusable preprocessing library, examples, and notebooks. https://telescope-scheduling-intelligence.streamlit.app



![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)![Python](https://img.shields.io/badge/python-3.10%2B-blue.svg)

![Vue](https://img.shields.io/badge/vue-3.0%2B-green.svg)![Streamlit](https://img.shields.io/badge/streamlit-1.31%2B-ff4b4b.svg)

![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)



## Architecture## What’s inside



TSI is built with a modern microservices architecture:- Streamlit dashboard with pages for Sky Map, Distributions, Visibility Map, Scheduled Timeline, Insights, Trends, and Compare

- JSON/CSV loaders and preprocessing pipeline (fast, consistent, validated)

- **Backend**: Rust + Axum + Polars (high-performance REST API)- Optional Dark Periods overlay to distinguish observable vs non-observable time windows

- **Frontend**: Vue 3 + TypeScript + Vite (reactive SPA)- Scripts and examples for batch preprocessing and data exploration

- **Data**: JSON/CSV schedule files with preprocessing pipeline- Tests (unit + e2e) and a Docker image for reproducible runs

- **Deployment**: Docker + Docker Compose

## Repository layout (key files)

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for detailed architecture diagrams and data flow.

```

## Quick Start.

├── data/

### Using Docker Compose (Recommended)│   ├── schedule.csv              # Sample preprocessed dataset (used by the app)

│   ├── schedule.json             # Raw schedule (example)

Start both backend and frontend with a single command:│   ├── possible_periods.json     # Optional visibility periods

│   └── dark_periods.json         # Optional dark periods (auto-detected by the app)

```bash├── examples/

docker-compose up --build│   ├── example_data_loading.py   # Loaders usage

```│   └── example_preprocessing.py  # Preprocessor usage

├── notebooks/

Access the application:│   ├── eda.ipynb

- **Frontend**: http://localhost:5173│   └── scheduling_trends.ipynb

- **Backend API**: http://localhost:8081├── scripts/

- **Health Check**: http://localhost:8081/health│   ├── preprocess_schedules.py   # CLI: JSON → CSV (single/batch)

│   └── train_model.py            # Modeling pipeline entrypoint

See [`DOCKER_QUICKSTART.md`](DOCKER_QUICKSTART.md) for more Docker commands.├── src/

│   ├── core/

### Manual Setup│   │   ├── loaders/              # JSON/CSV/data-dir loaders

│   │   └── preprocessing/        # SchedulePreprocessor + helpers

#### Backend (Rust)│   └── tsi/

│       ├── app.py                # Streamlit entrypoint

```bash│       ├── routing.py, state.py, theme.py

cd backend│       ├── pages/                # Sky Map, Distributions, Visibility Map, Schedule, Insights, Trends, Compare

cargo run│       ├── plots/, components/, services/

```│       └── assets/styles.css

├── run_dashboard.sh              # Local launcher (venv + streamlit)

Server runs at http://127.0.0.1:8080. See [`backend/README.md`](backend/README.md) for details.├── streamlit_app.py              # Streamlit Cloud entry (imports tsi.app.main)

├── Dockerfile

#### Frontend (Vue)├── pyproject.toml

├── requirements.txt

```bash└── tests/

cd frontend    ├── core/, e2e/, manual/

npm install    └── benchmarks/

npm run dev```

```

## Quickstart

Frontend runs at http://localhost:5173. See [`frontend/README.md`](frontend/README.md) for details.

Prereqs: Python 3.10+ and pip.

## Repository Structure

```bash

```# Install

.pip install -r requirements.txt

├── backend/              # Rust API server (Axum + Polars)

│   ├── src/# Run the Streamlit app

│   │   ├── main.rs      # Server entrypointstreamlit run src/tsi/app.py

│   │   ├── routes.rs    # HTTP handlers

│   │   ├── compute.rs   # Analytics logic# Or use the helper

│   │   └── models/      # Data models./run_dashboard.sh

│   ├── tests/           # Integration tests```

│   └── benches/         # Performance benchmarks

│The app opens at http://localhost:8501. On the landing page you can:

├── frontend/            # Vue 3 + TypeScript SPA- Upload a preprocessed CSV (fastest), or

│   ├── src/- Upload a raw schedule.json (+ optional possible_periods.json). JSON is processed in‑memory, or

│   │   ├── App.vue      # Main application- Load the bundled sample dataset at `data/schedule.csv`.

│   │   ├── components/  # Vue components

│   │   └── pages/       # Page viewsDark periods: if `data/dark_periods.json` exists, it is auto‑loaded; you can also upload it later on the landing page. The Scheduled Timeline page then shades nighttime (observable) vs daytime (non‑observable) periods.

│   └── package.json

│## Preprocess JSON → CSV (recommended for performance)

├── data/                # Sample datasets

│   ├── schedule.csvThe dashboard can process JSON directly, but for repeat analysis and faster loads prefer CSV precomputation using the CLI.

│   ├── schedule.json

│   └── dark_periods.json```bash

│# Single file

├── src/                 # Shared Python utilitiespython scripts/preprocess_schedules.py \

│   ├── core/           # Core preprocessing library  --schedule data/schedule.json \

│   │   ├── loaders/    # Data loaders  --output data/schedule.csv

│   │   └── preprocessing/

│   └── adapters/       # Data adapters# With visibility/possible periods

│python scripts/preprocess_schedules.py \

├── tests/              # Python core library tests  --schedule data/schedule.json \

│   └── core/  --visibility data/possible_periods.json \

│  --output data/schedule.csv \

├── docs/               # Documentation  --verbose

│   ├── API.md

│   └── *.md# Batch directory

│python scripts/preprocess_schedules.py \

├── migration-doc/      # Migration documentation  --batch-dir data/schedules \

│   └── PHASE_*.md  --output-dir data/preprocessed

│

├── old/               # Legacy Streamlit application# Batch with custom patterns

│   └── ...            # (archived)python scripts/preprocess_schedules.py \

│  --batch-dir data/schedules \

├── docker-compose.yml  --pattern "schedule_*.json" \

├── ARCHITECTURE.md  --visibility-pattern "possible_periods*.json" \

└── README.md  --output-dir data/preprocessed

``````



## FeaturesExamples: see `examples/example_data_loading.py` and `examples/example_preprocessing.py`.



### Current Capabilities## Data schema (CSV expected by the app)



- **Health Monitoring**: System health checks via REST APIRequired columns (from `src/tsi/config.py`):

- **Data Analysis**: Statistical computations (mean, std dev) on datasets- schedulingBlockId, priority, minObservationTimeInSec, requestedDurationSec

- **Real-time Updates**: Server-Sent Events (SSE) for progress tracking- fixedStartTime, fixedStopTime, decInDeg, raInDeg

- **Modern UI**: Responsive Vue 3 interface- minAzimuthAngleInDeg, maxAzimuthAngleInDeg, minElevationAngleInDeg, maxElevationAngleInDeg

- **Performance**: Rust backend with Polars for fast data processing- scheduled_period.start, scheduled_period.stop

- visibility, num_visibility_periods, total_visibility_hours, priority_bin

### API Endpoints- scheduled_flag, requested_hours, elevation_range_deg



- `GET /health` - Health checkNotes

- `POST /api/v1/compute` - Compute statistics on input data- Times are MJD in the raw JSON; the app converts to UTC timestamps for display.

- `GET /api/v1/progress` - SSE stream for progress updates- The `visibility` column is a list of (start, stop) MJD pairs; when stored in CSV it’s stringified.



## Development## Dashboard pages



### Backend Testing- Sky Map: RA/Dec scatter with color by priority or status, size by requested hours, priority and time filters

- Distributions: histograms and summary distributions (priority, visibility hours, requested duration, elevation range)

```bash- Visibility Map: visualize visibility windows and constraints

cd backend- Scheduled Timeline: month‑by‑month view of scheduled observations; optional dark/daytime overlays; CSV export

cargo test              # Run all tests- Insights: scheduling rates, correlations, integrity checks, and top lists

cargo test -- --ignored # Run integration tests- Trends: time evolution of scheduling metrics

cargo bench            # Run benchmarks- Compare: load a second CSV to compare two schedules side‑by‑side

```

## Configuration

### Frontend Testing

Runtime settings are managed via `pydantic-settings` in `src/app_config/settings.py` and can be overridden with environment variables or a `.env` file at repo root.

```bash

cd frontendKey variables

npm test               # Run tests- DATA_ROOT: base data directory (default: data)

npm run build         # Production build- SAMPLE_DATASET: path to the sample CSV (default: data/schedule.csv)

```- CACHE_TTL_SECONDS: cache TTL for loaders (default: 600)



### Python Core LibraryExample `.env`

```

The `src/core/` directory contains shared Python utilities for data preprocessing:SAMPLE_DATASET=data/schedule.csv

DATA_ROOT=data

```bashCACHE_TTL_SECONDS=900

# Run core library tests```

pytest tests/core/

## Run tests and quality gates

# Install development dependencies

pip install -e ".[dev]"```bash

```# Unit + e2e tests

pytest

## Data Format

# Optional dev tools (install with: pip install -e ".[dev]")

Schedule data can be provided as:ruff check src/ tests/

- **JSON**: Raw schedule format (see `data/schedule.json`)black --check src/ tests/

- **CSV**: Preprocessed format for faster loadingmypy src/

```

Required columns include:

- `schedulingBlockId`, `priority`, `requestedDurationSec`## Docker

- `fixedStartTime`, `fixedStopTime`

- `raInDeg`, `decInDeg````bash

- `minElevationAngleInDeg`, `maxElevationAngleInDeg`# Build

docker build -t tsi-app .

## Configuration

# Run dashboard (http://localhost:8501)

### Backend Environment Variablesdocker run --rm -p 8501:8501 tsi-app



- `RUST_LOG`: Log level (info, debug, trace)# Run tests inside the same image

- `RUST_BACKTRACE`: Enable backtraces (0, 1, full)docker run --rm tsi-app pytest

```

### Frontend Configuration

The image defaults to launching the dashboard; overriding the command lets you reuse it for CI.

Edit `frontend/vite.config.ts` to configure API proxy and build settings.

## Development notes

## Migration from Streamlit

- Add pages under `src/tsi/pages/` and register them in `src/tsi/routing.py`.

This project has been migrated from a Python/Streamlit application to a modern Rust/Vue stack. The legacy Streamlit application and all related files are preserved in the `/old` directory for reference.- Plots live in `src/tsi/plots/`, components in `src/tsi/components/`, services in `src/tsi/services/`.

- JSON/CSV parsing and preprocessing are under `src/core/`.

Migration documentation can be found in:

- [`migration-doc/RUST_VUE_MIGRATION_PLAN.md`](migration-doc/RUST_VUE_MIGRATION_PLAN.md)## License

- [`migration-doc/PHASE_*.md`](migration-doc/) - Detailed phase-by-phase migration logs

AGPL-3.0 — see `LICENSE` for details.

## Documentation

---

- [`ARCHITECTURE.md`](ARCHITECTURE.md) - System architecture and design

- [`DOCKER_QUICKSTART.md`](DOCKER_QUICKSTART.md) - Docker deployment guideBuilt with Streamlit, Plotly, pandas, and modern Python tooling.

- [`docs/API.md`](docs/API.md) - API documentation
- [`backend/README.md`](backend/README.md) - Backend development guide
- [`frontend/README.md`](frontend/README.md) - Frontend development guide

## License

AGPL-3.0 — see [`LICENSE`](LICENSE) for details.

---

**Built with**: Rust, Axum, Polars, Vue 3, TypeScript, Docker
