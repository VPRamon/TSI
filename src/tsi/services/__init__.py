"""Services package initialization."""

from core.time import (
    format_datetime_utc,
    get_time_range,
    parse_optional_mjd,
)
# Time conversions now use Rust backend (8x faster)
from tsi.services.rust_compat import (
    datetime_to_mjd_rust as datetime_to_mjd,
    mjd_to_datetime_rust as mjd_to_datetime,
    parse_visibility_periods_rust as parse_visibility_periods,
    load_dark_periods_rust as load_dark_periods,
)
from tsi.services.analytics import (
    compute_correlations,
    compute_distribution_stats,
    compute_metrics,
    find_conflicts,
    generate_insights,
    get_top_observations,
)
from tsi.services.loaders import (
    get_filtered_dataframe,
    load_csv,
    prepare_dataframe,
    validate_dataframe,
)
from tsi.services.sky_map_filters import (
    build_palette,
    filter_dataframe,
    get_priority_range,
    prepare_priority_bins,
    to_utc_timestamp,
)
from tsi.services.distributions_filters import (
    check_filter_support,
    compute_impossible_mask,
    filter_impossible_observations,
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
    "compute_metrics",
    "compute_correlations",
    "get_top_observations",
    "find_conflicts",
    "compute_distribution_stats",
    "generate_insights",
    # sky_map_filters
    "filter_dataframe",
    "prepare_priority_bins",
    "get_priority_range",
    "build_palette",
    "to_utc_timestamp",
    # distributions_filters
    "filter_impossible_observations",
    "compute_impossible_mask",
    "check_filter_support",
]
