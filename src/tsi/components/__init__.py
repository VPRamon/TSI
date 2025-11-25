"""Components package initialization."""

from tsi.components.compare_tables import render_comparison_tables
from tsi.components.compare_upload import render_file_upload
from tsi.components.compare_validation import (
    compute_scheduling_changes,
    validate_and_display_discrepancies,
)
from tsi.components.distributions_controls import render_filter_control
from tsi.components.distributions_layout import render_figure_layout
from tsi.components.distributions_stats import render_statistical_summary
from tsi.components.insights_analysis import render_automated_insights, render_correlation_analysis
from tsi.components.insights_controls import render_filter_controls
from tsi.components.insights_metrics import (
    render_key_metrics as render_insights_key_metrics,
)
from tsi.components.insights_metrics import (
    render_priority_analysis,
)
from tsi.components.insights_reports import render_report_downloads
from tsi.components.insights_tables import render_integrity_checks, render_top_observations
from tsi.components.sky_map_controls import render_sidebar_controls, reset_sky_map_controls
from tsi.components.sky_map_stats import render_stats
from tsi.components.timeline_controls import render_search_filters
from tsi.components.timeline_stats import render_download_button, render_key_metrics
from tsi.components.trends_controls import render_sidebar_controls as render_trends_sidebar_controls
from tsi.components.trends_empirical import render_empirical_proportions
from tsi.components.trends_heatmap import render_heatmap_section
from tsi.components.trends_metrics import render_overview_metrics
from tsi.components.trends_model import (
    render_model_information,
    render_model_metrics,
    render_prediction_plot,
)
from tsi.components.trends_smoothed import render_smoothed_trends
from tsi.components.visibility_controls import (
    render_generate_button,
    render_histogram_settings,
)
from tsi.components.visibility_controls import (
    render_sidebar_controls as render_visibility_sidebar_controls,
)
from tsi.components.visibility_stats import render_chart_info, render_dataset_statistics

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
]
