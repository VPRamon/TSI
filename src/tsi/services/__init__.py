"""
Services package for TSI application.

This package provides high-level services organized into logical sub-packages:

- `data`: Analytics helpers
- `filters`: Filtering modules for observations and sky map blocks
- `processors`: Processing modules for timeline, trends, sky map, and comparison views
- `utils`: Utility modules for time conversion, visibility, and reporting

Core modules at the root level:
- `backend_service`: Unified backend service facade combining remote and local operations
"""

# ============================================================================
# Core Backend Service & Data
# ============================================================================
from tsi.services.backend_service import (
    BackendService,
    ScheduleSummary,
    backend,
)
from tsi.services.data import (
    AnalyticsSnapshot,
    generate_insights,
    generate_correlation_insights,
)

# ============================================================================
# Filters: Impossible Observations & Sky Map
# ============================================================================
from tsi.services.filters import (
    apply_insights_filter,
    check_filter_support,
    compute_impossible_mask,
    filter_blocks,
    filter_impossible_observations,
)

# ============================================================================
# Processors: Timeline, Trends, Sky Map, Compare
# ============================================================================
from tsi.services.processors import (
    apply_search_filters,
    apply_trends_filters,
    calculate_observation_gaps,
    compute_priority_range,
    filter_dark_periods,
    filter_scheduled_data,
    get_priority_range,  # Backwards compatibility alias for compute_priority_range
    get_scheduled_time_range,
    prepare_display_dataframe,
    prepare_priority_bins_from_blocks,
    prepare_scheduled_data,
    validate_required_columns,
)

# ============================================================================
# Utils: Time, Visibility, Reporting
# ============================================================================
from tsi.services.utils import (
    datetime_to_mjd,
    filter_visibility_blocks,
    format_datetime_utc,
    get_all_block_ids,
    get_time_range,
    mjd_to_datetime,
    parse_optional_mjd,
    parse_visibility_periods,
)

# ============================================================================
# Backward Compatibility Wrappers
# ============================================================================
# These functions delegate to the unified backend service

def upload_schedule(
    schedule_name: str,
    schedule_json: str,
    visibility_json: str | None = None,
) -> ScheduleSummary:
    """Upload and store a schedule via the backend."""
    return backend.upload_schedule(schedule_name, schedule_json, visibility_json)


def list_schedules() -> list[ScheduleSummary]:
    """List available schedules using the backend."""
    return backend.list_schedules()


def get_sky_map_data(schedule_ref):
    """Get complete sky map data with computed bins and metadata."""
    return backend.get_sky_map_data(schedule_ref)


def get_visibility_map_data(schedule_ref):
    """Fetch visibility map metadata and block summaries from the backend."""
    return backend.get_visibility_map_data(schedule_ref)


def get_distribution_data(schedule_ref):
    """Get complete distribution data with computed statistics."""
    return backend.get_distribution_data(schedule_ref)


def get_schedule_timeline_data(schedule_ref):
    """Get complete schedule timeline data with computed statistics and metadata."""
    return backend.get_schedule_timeline_data(schedule_ref)


def get_insights_data(schedule_ref):
    """Get complete insights data with computed analytics and metadata."""
    return backend.get_insights_data(schedule_ref)


def get_trends_data(schedule_ref, n_bins: int = 10, bandwidth: float = 0.3, n_smooth_points: int = 100):
    """Get complete trends data with computed statistics and smoothed curves."""
    return backend.get_trends_data(schedule_ref, n_bins, bandwidth, n_smooth_points)


def get_compare_data(schedule_a_ref, schedule_b_ref):
    """Get comparison data between two schedules."""
    return backend.get_compare_data(schedule_a_ref, schedule_b_ref)


def get_validation_report(schedule_ref):
    """Get validation report for a schedule."""
    return backend.get_validation_report(schedule_ref)


def fetch_dark_periods(schedule_ref):
    """Fetch dark periods for a schedule (with global fallback)."""
    return backend.fetch_dark_periods(schedule_ref)


def fetch_possible_periods(schedule_ref):
    """Fetch possible/visibility periods for a schedule."""
    return backend.fetch_possible_periods(schedule_ref)


def get_visibility_histogram(
    schedule_ref,
    start,
    end,
    bin_duration_minutes: int,
    priority_range=None,
    block_ids=None,
):
    """Compute visibility histogram from the backend."""
    return backend.get_visibility_histogram(
        schedule_ref, start, end, bin_duration_minutes, priority_range, block_ids
    )


def get_schedule_time_range(schedule_ref):
    """Get the time range (min/max timestamps) for a schedule's visibility periods."""
    return backend.get_schedule_time_range(schedule_ref)

__all__ = [
    # Core Backend Service
    "backend",
    "BackendService",
    "ScheduleSummary",
    # Backend client compatibility functions
    "upload_schedule",
    "list_schedules",
    "fetch_dark_periods",
    "fetch_possible_periods",
    "get_visibility_map_data",
    "get_distribution_data",
    "get_sky_map_data",
    "get_schedule_time_range",
    "get_visibility_histogram",
    "get_schedule_timeline_data",
    "get_insights_data",
    "get_trends_data",
    "get_compare_data",
    "get_validation_report",
    # Data
    "backend",
    "BackendService",
    "ScheduleSummary",
    "upload_schedule",
    "list_schedules",
    "fetch_dark_periods",
    "fetch_possible_periods",
    "get_visibility_map_data",
    "get_distribution_data",
    "get_sky_map_data",
    "get_schedule_time_range",
    "get_visibility_histogram",
    "get_schedule_timeline_data",
    "get_insights_data",
    "get_trends_data",
    "get_compare_data",
    "get_validation_report",
    "AnalyticsSnapshot",
    "generate_insights",
    "generate_correlation_insights",
    # Filters
    "filter_impossible_observations",
    "compute_impossible_mask",
    "check_filter_support",
    "apply_insights_filter",
    "filter_blocks",
    # Processors
    "prepare_scheduled_data",
    "filter_scheduled_data",
    "filter_dark_periods",
    "prepare_display_dataframe",
    "apply_search_filters",
    "validate_required_columns",
    "apply_trends_filters",
    "compute_priority_range",
    "get_priority_range",  # Backwards compatibility alias
    "prepare_priority_bins_from_blocks",
    "get_scheduled_time_range",
    "calculate_observation_gaps",
    # Utils
    "mjd_to_datetime",
    "datetime_to_mjd",
    "parse_visibility_periods",
    "parse_optional_mjd",
    "get_time_range",
    "format_datetime_utc",
    "filter_visibility_blocks",
    "get_all_block_ids",
]
