"""Application configuration and constants."""

from pathlib import Path

from app_config import get_settings

# Project paths
_settings = get_settings()
ASSETS_DIR = Path(__file__).parent / "assets"

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

# UI Configuration
PAGES = [
    "Sky Map",
    "Distributions",
    "Visibility Map",
    "Schedule",
    "Insights",
    "Trends",
    "Compare",
]

# Plot defaults
PLOT_HEIGHT = 600
PLOT_MARGIN = dict(l=80, r=80, t=80, b=80)

# Analytics
CORRELATION_COLUMNS = [
    "priority",
    "requested_hours",
    "elevation_range_deg",
    "total_visibility_hours",
]
