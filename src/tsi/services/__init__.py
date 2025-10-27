"""Services package initialization."""

from core.time import (
    datetime_to_mjd,
    format_datetime_utc,
    get_time_range,
    mjd_to_datetime,
    parse_optional_mjd,
    parse_visibility_periods,
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

__all__ = [
    # loaders
    "load_csv",
    "prepare_dataframe",
    "get_filtered_dataframe",
    "validate_dataframe",
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
]
