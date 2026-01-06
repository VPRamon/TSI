"""
Services package for TSI application.

This package provides high-level services organized into logical sub-packages:

- `data`: Data loading, preparation, and analytics
- `filters`: Filtering modules for observations and sky map blocks
- `processors`: Processing modules for timeline, trends, sky map, and comparison views
- `utils`: Utility modules for time conversion, visibility, and reporting

Core modules at the root level:
- `rust_backend`: Shared Rust backend instance
- `backend_client`: High-level client for Rust backend operations
"""

# ============================================================================
# Core Backend Client & Data
# ============================================================================
from tsi.services.data import (
    AnalyticsSnapshot,
    compute_correlations,
    find_conflicts,
    generate_insights,
    get_filtered_dataframe,
    get_top_observations,
    load_schedule_rust,
    prepare_dataframe,
    validate_dataframe,
)
from tsi.services.backend_client import (
    ScheduleSummary,
    fetch_dark_periods,
    fetch_possible_periods,
    get_compare_data,
    get_distribution_data,
    get_insights_data,
    get_schedule_time_range,
    get_schedule_timeline_data,
    get_sky_map_data,
    get_trends_data,
    get_validation_report,
    get_visibility_histogram,
    get_visibility_map_data,
    list_schedules,
    upload_schedule,
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
from tsi.services.rust_backend import BACKEND

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

# Time conversions now use Rust backend (8x faster)
from tsi_rust_api import load_dark_periods

__all__ = [
    # Core Backend Client
    "BACKEND",
    "upload_schedule",
    "list_schedules",
    "ScheduleSummary",
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
    "prepare_dataframe",
    "get_filtered_dataframe",
    "validate_dataframe",
    "load_schedule_rust",
    "load_dark_periods",
    "AnalyticsSnapshot",
    "compute_correlations",
    "get_top_observations",
    "find_conflicts",
    "generate_insights",
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
