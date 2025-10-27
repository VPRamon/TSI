# Telescope Scheduling Intelligence

A production-grade, multi-module Streamlit application for analyzing and visualizing telescope scheduling data.

![Python](https://img.shields.io/badge/python-3.10+-blue.svg)
![Streamlit](https://img.shields.io/badge/streamlit-1.31+-red.svg)
![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)

## Features

- 🌌 **Sky Map**: Interactive celestial coordinate visualization with RA/Dec plotting
- 📊 **Distributions**: Comprehensive statistical analysis of scheduling parameters
- 📅 **Visibility & Schedule Timeline**: Gantt-style visualization of observation windows
- 💡 **Insights & Conclusions**: Automated analytics with correlation analysis and integrity checks
- 🧠 **Predictive Model (CLI only)**: Run offline scripts to analyze unscheduled blocks with ML
- �📥 **Flexible Data Loading**: Upload custom CSV or use sample dataset
- 🎨 **Professional UI**: Clean, responsive design with persistent navigation

## Architecture

This application follows best practices with:

- **Modular design**: Clear separation of concerns across services, plots, pages, and components
- **Type safety**: Pydantic schemas and type hints throughout
- **Performance**: Strategic caching with `@st.cache_data` decorators
- **Testability**: Comprehensive pytest suite
- **Code quality**: Configured with ruff, black, and mypy

## Project Structure

```
telescope_scheduling_intelligence/
├── pyproject.toml
├── requirements.txt
├── README.md
├── .streamlit/
│   └── config.toml
├── data/
│   ├── schedule.csv                  # Sample preprocessed data
│   ├── schedule.json                 # Raw schedule JSON
│   ├── possible_periods.json         # Visibility/possible periods data
│   └── dark_periods.json             # Dark time periods data
│
├── src/
│   ├── core/
│   │   ├── loaders/                  # Unified data loading
│   │   │   ├── __init__.py
│   │   │   └── schedule_loader.py
│   │   ├── preprocessing/            # Data preprocessing
│   │   │   └── schedule_preprocessor.py
│   │   └── ...
│   └── tsi/
│       ├── app.py              # Main entry point
│       ├── state.py            # Session state management
│       ├── theme.py            # Theming and CSS
│       ├── routing.py          # Navigation logic
│       ├── config.py           # Configuration constants
│       ├── models/
│       │   └── schemas.py      # Data models
│       ├── services/
│       │   ├── loaders.py      # Data loading
│       │   ├── time_utils.py   # Time conversion utilities
│       │   ├── analytics.py    # Statistical analysis
│       │   └── report.py       # Report generation
│       ├── plots/
│       │   ├── sky_map.py
│       │   ├── distributions.py
│       │   └── timeline.py
│       ├── pages/
│       │   ├── landing.py
│       │   ├── sky_map.py
│       │   ├── distributions.py
│       │   ├── visibility_schedule.py
│       │   └── insights.py
│       ├── components/
│       │   ├── toolbar.py
│       │   ├── data_preview.py
│       │   └── metrics.py
│       └── assets/
│           └── styles.css
├── tests/
│   ├── test_time_utils.py
│   ├── test_loaders.py
│   └── test_analytics.py
└── docs/
    └── index.md
```

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd bootcamp

# Install dependencies
pip install -r requirements.txt

# For development
pip install -e ".[dev]"
```

## Usage

### Preprocess Data (Optional)

If you have raw JSON schedule files, you can preprocess them into CSV format:

```bash
# Process a single schedule JSON file
python preprocess_schedules.py \
  --schedule data/schedule.json \
  --output data/schedule.csv

# Process with visibility/possible periods data
python preprocess_schedules.py \
  --schedule data/schedule.json \
  --visibility data/possible_periods.json \
  --output data/schedule.csv \
  --verbose

# Batch process multiple schedule files in a directory
python preprocess_schedules.py \
  --batch-dir data/schedules \
  --output-dir data/preprocessed

# Batch process with custom patterns
python preprocess_schedules.py \
  --batch-dir data/schedules \
  --pattern "schedule_*.json" \
  --visibility-pattern "possible_periods*.json" \
  --output-dir data/preprocessed
```

For more details, see [PREPROCESS_SCHEDULES_README.md](PREPROCESS_SCHEDULES_README.md).

### Run the application

```bash
streamlit run src/tsi/app.py
```

Or use the convenience script:

```bash
./run_dashboard.sh
```

The dashboard will open at `http://localhost:8501`.

### Run with Docker

You can build a container image that contains all runtime and development dependencies for the
Streamlit dashboard and the test suite.

```bash
# Build the image
docker build -t tsi-app .

# Run the Streamlit application (http://localhost:8501)
docker run --rm -p 8501:8501 tsi-app

# Run the test suite inside the container
docker run --rm tsi-app pytest
```

The default container command launches the dashboard, so specifying an alternate command (such as
`pytest`) lets you reuse the same image for CI or local test execution.

### Data Loading Options

The application supports three methods of loading data:

1. **Upload CSV**: Upload a preprocessed CSV file (recommended for performance)
2. **Upload JSON**: Upload raw `schedule.json` (and optionally `possible_periods.json`) - processed automatically in-memory
3. **Use Sample Data**: Load the included sample dataset

**For Notebooks**: Notebooks use preprocessed CSV files (e.g., `data/schedule.csv`). The data loading logic is centralized in `src/core/loaders/`.

See [DATA_LOADING_ARCHITECTURE.md](doc/DATA_LOADING_ARCHITECTURE.md) for detailed documentation on the unified data loading system.

### Run tests

```bash
pytest
```

### Code quality

```bash
# Format code
black src/ tests/

# Lint
ruff check src/ tests/

# Type check
mypy src/
```

## Data Schema

The application expects CSV files with the following columns:

- `schedulingBlockId`: Unique identifier
- `priority`: Observation priority (0-10)
- `minObservationTimeInSec`: Minimum observation time
- `requestedDurationSec`: Requested duration
- `fixedStartTime`: Optional fixed start constraint (MJD)
- `fixedStopTime`: Optional fixed stop constraint (MJD)
- `decInDeg`: Declination in degrees
- `raInDeg`: Right Ascension in degrees
- `minAzimuthAngleInDeg`: Minimum azimuth constraint
- `maxAzimuthAngleInDeg`: Maximum azimuth constraint
- `minElevationAngleInDeg`: Minimum elevation constraint
- `maxElevationAngleInDeg`: Maximum elevation constraint
- `scheduled_period.start`: Scheduled start time (MJD)
- `scheduled_period.stop`: Scheduled stop time (MJD)
- `visibility`: List of visibility windows (stringified tuples of MJD values)
- `num_visibility_periods`: Count of visibility windows
- `total_visibility_hours`: Total visibility time in hours
- `priority_bin`: Priority category

**Note**: All time values use Modified Julian Date (MJD) format and are converted to UTC internally.

## Key Features

### Landing Page
- Upload custom CSV files
- Use pre-loaded sample dataset
- Data validation and preview

### Sky Map
- Interactive RA/Dec scatter plot
- Color by priority or scheduling status
- Size by requested observation hours
- Flip RA axis for astronomical convention
- Filterable by priority and scheduling status

### Distributions
- Priority histogram with adjustable bins
- Visibility hours distribution
- Requested duration analysis
- Elevation constraint range
- Scheduled vs unscheduled counts
- Violin plots for comparative analysis

### Visibility & Schedule Timeline
- Gantt-style timeline visualization
- Visibility windows
- Scheduled periods overlay
- Fixed time constraints
- Zoomable date range
- Multi-layer filtering

### Insights
- Automated scheduling rate calculation
- Priority statistics and correlations
- Top observations by priority and visibility
- Integrity checks for scheduling conflicts
- Downloadable analytical reports

### Predictive Model (CLI Only)

The predictive model and SHAP-based explainability tooling remain available for offline use, but the Streamlit prediction page
has been removed. To work with the model:

1. Train or refresh the model artifacts:
   ```bash
   python scripts/train_model.py
   ```

2. Run the scripted demo for a full CLI walkthrough:
   ```bash
   python demo_unscheduled_analysis.py
   ```

3. Validate the artifacts and inference pipeline with the test harness:
   ```bash
   python test_unscheduled_analysis.py
   ```

Example inputs are available in `src/tsi/modeling/artifacts/`, including `example_unscheduled_block.csv` and
`example_unscheduled_block.json`.

## Development

### Adding a new page

1. Create module in `src/tsi/pages/`
2. Register in `src/tsi/routing.py`
3. Add navigation item

### Adding a new plot

1. Create module in `src/tsi/plots/`
2. Implement `build_figure()` function
3. Use in appropriate page module

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

GNU Affero General Public License v3.0 - see LICENSE file for details

## Support

For issues, questions, or contributions, please open an issue on GitHub.

---

**Built with ❤️ using Streamlit, Plotly, and modern Python best practices**
