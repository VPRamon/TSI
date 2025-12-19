"""Processing modules for timeline, trends, sky map, and comparison views."""

from tsi.services.processors.compare import (
    calculate_observation_gaps,
)
from tsi.services.processors.sky_map import (
    compute_priority_range,
    get_priority_range,  # Backwards compatibility alias
    get_scheduled_time_range,
    prepare_priority_bins_from_blocks,
)
from tsi.services.processors.timeline import (
    apply_search_filters,
    filter_dark_periods,
    filter_scheduled_data,
    prepare_display_dataframe,
    prepare_scheduled_data,
)
from tsi.services.processors.trends import (
    apply_trends_filters,
    validate_required_columns,
)

__all__ = [
    # timeline
    "prepare_scheduled_data",
    "filter_scheduled_data",
    "filter_dark_periods",
    "prepare_display_dataframe",
    "apply_search_filters",
    # trends
    "validate_required_columns",
    "apply_trends_filters",
    # sky_map (includes priority range)
    "compute_priority_range",
    "get_priority_range",  # Backwards compatibility alias
    "prepare_priority_bins_from_blocks",
    "get_scheduled_time_range",
    # compare
    "calculate_observation_gaps",
]
