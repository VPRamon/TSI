"""Components package initialization."""

from tsi.components.distributions_controls import render_filter_control
from tsi.components.distributions_layout import render_figure_layout
from tsi.components.distributions_stats import render_statistical_summary
from tsi.components.sky_map_controls import render_sidebar_controls, reset_sky_map_controls
from tsi.components.sky_map_stats import render_stats
from tsi.components.visibility_controls import (
    render_generate_button,
    render_histogram_settings,
    render_sidebar_controls as render_visibility_sidebar_controls,
)
from tsi.components.visibility_stats import render_chart_info, render_dataset_statistics
from tsi.components.timeline_controls import render_search_filters
from tsi.components.timeline_stats import render_key_metrics, render_download_button

__all__ = [
    # Sky Map components
    "render_sidebar_controls",
    "reset_sky_map_controls",
    "render_stats",
    # Distributions components
    "render_filter_control",
    "render_figure_layout",
    "render_statistical_summary",
    # Visibility components
    "render_visibility_sidebar_controls",
    "render_histogram_settings",
    "render_generate_button",
    "render_dataset_statistics",
    "render_chart_info",
    # Timeline components
    "render_search_filters",
    "render_key_metrics",
    "render_download_button",
]
