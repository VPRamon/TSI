"""Services package initialization."""

from tsi.services.rust_backend import BACKEND
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
    load_schedule_rust,
    prepare_dataframe,
    validate_dataframe,
)
from tsi.services.database import (
    db_health_check,
    fetch_dark_periods_db,
    fetch_possible_periods_db,
    fetch_schedule_db,
    init_database,
    list_schedules_db,
    store_schedule_db,
    get_visibility_map_data,
)

# Time conversions now use Rust backend (8x faster)
from tsi_rust_api import load_dark_periods
from tsi.services.time_utils import (
    datetime_to_mjd,
    format_datetime_utc,
    get_time_range,
    mjd_to_datetime,
    parse_optional_mjd,
    parse_visibility_periods,
)
from tsi.services.sky_map_filters import (
    build_palette,
    filter_blocks,
)
from tsi.services.database import (
    get_distribution_data,
    get_sky_map_data,
    get_visibility_map_data,
)
from tsi.services.sky_map_blocks import (
    get_priority_range_from_blocks,
    prepare_priority_bins_from_blocks,
    get_scheduled_time_range,
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
    filter_visibility_blocks,
    get_all_block_ids,
)

__all__ = [
    "BACKEND",
    # loaders
    "load_csv",
    "prepare_dataframe",
    "get_filtered_dataframe",
    "validate_dataframe",
    "load_schedule_rust",
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
    "filter_blocks",
    "build_palette",
    "get_sky_map_data",
    "get_distribution_data",
    # sky_map_blocks
    "get_priority_range_from_blocks",
    "prepare_priority_bins_from_blocks",
    "get_scheduled_time_range",
    # impossible_filters (consolidated)
    "filter_impossible_observations",
    "compute_impossible_mask",
    "check_filter_support",
    "apply_insights_filter",
    # visibility_processing
    "compute_effective_priority_range",
    "filter_visibility_blocks",
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
    # database
    "init_database",
    "db_health_check",
    "store_schedule_db",
    "fetch_schedule_db",
    "list_schedules_db",
    "fetch_dark_periods_db",
    "fetch_possible_periods_db",
    "get_visibility_map_data",
]
