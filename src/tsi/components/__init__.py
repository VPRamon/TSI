"""Components package initialization.

This module re-exports commonly used component functions from the
per-page subpackages so downstream imports like
`from tsi.components.compare_tables import ...` can be migrated to the
new layout incrementally.
"""

# Re-export selected symbols from subpackages
from .compare.compare_tables import render_comparison_tables
from .compare.compare_upload import render_file_upload
from .compare.compare_validation import (
    compute_scheduling_changes,
    validate_and_display_discrepancies,
)
from .distributions.distributions_controls import render_filter_control
from .distributions.distributions_layout import render_figure_layout
from .distributions.distributions_stats import render_statistical_summary
from .insights.insights_analysis import render_automated_insights, render_correlation_analysis
from .insights.insights_controls import render_filter_controls
from .insights.insights_metrics import (
    render_key_metrics as render_insights_key_metrics,
)
from .insights.insights_metrics import (
    render_priority_analysis,
)
from .insights.insights_reports import render_report_downloads
from .insights.insights_tables import render_integrity_checks, render_top_observations
from .landing.landing_backend import render_backend_section
from .landing.landing_upload import render_upload_section
from .shared.filters import (
    render_exclude_impossible_checkbox,
    render_exclude_zero_visibility_checkbox,
    render_status_filter,
)
from .sky_map.sky_map_controls import render_sidebar_controls, reset_sky_map_controls
from .sky_map.sky_map_stats import render_stats
from .timeline.timeline_controls import render_search_filters
from .timeline.timeline_stats import render_download_button, render_key_metrics
from .trends.trends_controls import render_sidebar_controls as render_trends_sidebar_controls
from .trends.trends_empirical import render_empirical_proportions
from .trends.trends_heatmap import render_heatmap_section
from .trends.trends_metrics import render_overview_metrics
from .trends.trends_model import (
    render_model_information,
    render_model_metrics,
    render_prediction_plot,
)
from .trends.trends_smoothed import render_smoothed_trends
from .visibility.visibility_controls import (
    render_generate_button,
    render_histogram_settings,
)
from .visibility.visibility_controls import (
    render_sidebar_controls as render_visibility_sidebar_controls,
)
from .visibility.visibility_stats import render_chart_info, render_dataset_statistics

__all__ = [
    # Shared components
    "render_exclude_impossible_checkbox",
    "render_exclude_zero_visibility_checkbox",
    "render_status_filter",
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
    # Insights components
    "render_filter_controls",
    "render_insights_key_metrics",
    "render_priority_analysis",
    "render_automated_insights",
    "render_correlation_analysis",
    "render_top_observations",
    "render_integrity_checks",
    "render_report_downloads",
    # Trends components
    "render_trends_sidebar_controls",
    "render_overview_metrics",
    "render_empirical_proportions",
    "render_smoothed_trends",
    "render_heatmap_section",
    "render_model_metrics",
    "render_prediction_plot",
    "render_model_information",
    # Compare components
    "render_file_upload",
    "validate_and_display_discrepancies",
    "compute_scheduling_changes",
    "render_comparison_tables",
    # Landing components
    "render_backend_section",
    "render_upload_section",
]
