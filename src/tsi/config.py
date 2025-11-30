"""Application configuration and constants.

This module provides access to centralized configuration and defines
application-wide constants. Configuration values come from app_config.settings
which supports environment variables and .env files.
"""

from pathlib import Path

from app_config import get_settings

# Get centralized settings
_settings = get_settings()

# Project paths
ASSETS_DIR = Path(__file__).parent / "assets"

# Data paths (from configuration)
DATA_ROOT = _settings.data_root
SAMPLE_DATASET = _settings.sample_dataset

# UI Configuration (from configuration)
APP_TITLE = _settings.app_title
APP_ICON = _settings.app_icon
PAGES = _settings.pages

# Plot defaults (from configuration)
PLOT_HEIGHT = _settings.plot_height
PLOT_MARGIN = _settings.get_plot_margin()

# Performance settings (from configuration)
CACHE_TTL = _settings.cache_ttl
MAX_WORKERS = _settings.max_workers

# Data schema - Pre-processed CSV required columns
REQUIRED_COLUMNS = [
    # Base columns from JSON
    "schedulingBlockId",
    "priority",
    "minObservationTimeInSec",
    "requestedDurationSec",
    "fixedStartTime",
    "fixedStopTime",
    "decInDeg",
    "raInDeg",
    "minAzimuthAngleInDeg",
    "maxAzimuthAngleInDeg",
    "minElevationAngleInDeg",
    "maxElevationAngleInDeg",
    "scheduled_period.start",
    "scheduled_period.stop",
    "visibility",
    "num_visibility_periods",
    "total_visibility_hours",
    "priority_bin",
    # Derived columns (required from pre-processing)
    "scheduled_flag",
    "requested_hours",
    "elevation_range_deg",
]

# MJD time conversion constants
MJD_UNIX_EPOCH = 40587.0  # Unix epoch (1970-01-01) in MJD
SECONDS_PER_DAY = 86400.0

# Analytics
CORRELATION_COLUMNS = [
    "priority",
    "requested_hours",
    "elevation_range_deg",
    "total_visibility_hours",
]
