"""Distributions page - statistical visualizations."""

import streamlit as st

from tsi import state
from tsi.components.distributions.distributions_layout import render_figure_layout
from tsi.components.distributions.distributions_stats import render_statistical_summary
from tsi.plots.distributions import build_figures
from tsi.services import database as db
from tsi.utils.error_display import display_backend_error


def render() -> None:
    """Render the Distributions page."""
    # Title and filter controls in same row
    col1, col2 = st.columns([3, 1])

    with col1:
        st.title("📊 Distributions")

    st.markdown(
        """
        Statistical analysis and distribution visualizations of key scheduling parameters.
        """
    )

    schedule_id = state.get_schedule_id()

    try:
        with st.spinner("Loading distribution data..."):
            distribution_data = db.get_distribution_data(
                schedule_id=schedule_id,
            )
    except Exception as exc:
        display_backend_error(exc)
        return

    # Build all distribution figures
    priority_bins = 20  # Default bins value
    with st.spinner("Generating plots..."):
        figures = build_figures(distribution_data, priority_bins=priority_bins)

    # Display figures in organized layout
    render_figure_layout(figures)

    # Display statistical summary with pre-computed stats
    render_statistical_summary(distribution_data)
