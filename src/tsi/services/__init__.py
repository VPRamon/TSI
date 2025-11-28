"""Services package initialization."""

from tsi.services.analytics import (
    AnalyticsSnapshot,
    compute_correlations,
    compute_metrics,
    find_conflicts,
    generate_insights,
    get_top_observations,
)
from tsi.services.compare_processing import calculate_observation_gaps

# Consolidated impossible observation filtering
from tsi.services.impossible_filters import (
    apply_insights_filter,
    check_filter_support,
    compute_impossible_mask,
    filter_impossible_observations,
)
from tsi.services.loaders import (
    get_filtered_dataframe,
    load_csv,
    prepare_dataframe,
    validate_dataframe,
)

# Time conversions now use Rust backend (8x faster)
from tsi.services.rust_compat import (
    datetime_to_mjd_rust as datetime_to_mjd,
    format_datetime_utc_rust as format_datetime_utc,
    get_time_range_rust as get_time_range,
    load_dark_periods_rust as load_dark_periods,
    mjd_to_datetime_rust as mjd_to_datetime,
    parse_optional_mjd_rust as parse_optional_mjd,
    parse_visibility_periods_rust as parse_visibility_periods,
)
from tsi.services.sky_map_filters import (
    build_palette,
    filter_dataframe,
    prepare_priority_bins,
    to_utc_timestamp,
)
from tsi.services.timeline_processing import (
    apply_search_filters,
    filter_dark_periods,
    filter_scheduled_data,
    prepare_display_dataframe,
    prepare_scheduled_data,
)
from tsi.services.trends_processing import (
    apply_trends_filters,
    validate_required_columns,
)
from tsi.services.priority_range import get_priority_range
from tsi.services.visibility_processing import (
    compute_effective_priority_range,
    get_all_block_ids,
)

__all__ = [
    # loaders
    "load_csv",
    "prepare_dataframe",
    "get_filtered_dataframe",
    "validate_dataframe",
    "load_dark_periods",
    # time_utils
    "mjd_to_datetime",
    "datetime_to_mjd",
    "parse_visibility_periods",
    "parse_optional_mjd",
    "get_time_range",
    "format_datetime_utc",
    # analytics
    "AnalyticsSnapshot",
    "compute_metrics",
    "compute_correlations",
    "get_top_observations",
    "find_conflicts",
    "generate_insights",
    # priority
    "get_priority_range",
    # sky_map_filters
    "filter_dataframe",
    "prepare_priority_bins",
    "build_palette",
    "to_utc_timestamp",
    # impossible_filters (consolidated)
    "filter_impossible_observations",
    "compute_impossible_mask",
    "check_filter_support",
    "apply_insights_filter",
    # visibility_processing
    "compute_effective_priority_range",
    "get_all_block_ids",
    # timeline_processing
    "prepare_scheduled_data",
    "filter_scheduled_data",
    "filter_dark_periods",
    "prepare_display_dataframe",
    "apply_search_filters",
    # trends_processing
    "validate_required_columns",
    "apply_trends_filters",
    # compare_processing
    "calculate_observation_gaps",
]
