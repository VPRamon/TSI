"""Plots package initialization."""

from tsi.plots.compare_plots import (
    create_changes_plot,
    create_priority_distribution_plot,
    create_scheduling_status_plot,
    create_time_distribution_plot,
)
from tsi.plots.plot_theme import (
    COMPARISON_SCHEDULE_COLOR,
    CURRENT_SCHEDULE_COLOR,
    NEGATIVE_CHANGE_COLOR,
    POSITIVE_CHANGE_COLOR,
    SCHEDULED_COLOR,
    UNSCHEDULED_COLOR,
    PlotTheme,
    apply_theme,
    get_axis_config,
    get_bar_marker,
    get_box_marker,
    get_colorscale_config,
    get_histogram_marker,
    get_horizontal_legend,
    get_scatter_marker,
    get_standard_layout,
    get_vertical_legend,
)

__all__ = [
    # Compare plots
    "create_priority_distribution_plot",
    "create_scheduling_status_plot",
    "create_changes_plot",
    "create_time_distribution_plot",
    # Theme and styling
    "PlotTheme",
    "apply_theme",
    "get_standard_layout",
    "get_horizontal_legend",
    "get_vertical_legend",
    "get_axis_config",
    "get_histogram_marker",
    "get_bar_marker",
    "get_scatter_marker",
    "get_box_marker",
    "get_colorscale_config",
    # Color constants
    "SCHEDULED_COLOR",
    "UNSCHEDULED_COLOR",
    "CURRENT_SCHEDULE_COLOR",
    "COMPARISON_SCHEDULE_COLOR",
    "POSITIVE_CHANGE_COLOR",
    "NEGATIVE_CHANGE_COLOR",
]
