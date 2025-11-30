"""Distributions page - statistical visualizations."""

import streamlit as st

from tsi import state
from tsi.components.distributions.distributions_controls import render_filter_control
from tsi.components.distributions.distributions_layout import render_figure_layout
from tsi.components.distributions.distributions_stats import render_statistical_summary
from tsi.plots.distributions import build_figures
from tsi.services.database import get_distribution_data


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

    if schedule_id is None:
        st.info("Load a schedule from the database to view distributions.")
        return

    schedule_id = int(schedule_id)

    # Render filter control (simplified - just the impossible filter toggle)
    with col2:
        filter_impossible = st.checkbox(
            "Exclude Impossible",
            value=False,
            help="Exclude observations with zero visibility hours"
        )

    try:
        with st.spinner("Loading distribution data..."):
            distribution_data = get_distribution_data(
                schedule_id=schedule_id,
                filter_impossible=filter_impossible
            )
    except Exception as exc:
        st.error(f"Failed to load distribution data from the backend: {exc}")
        return

    if distribution_data.total_count == 0:
        st.warning("⚠️ No observations available with the selected filter.")
        return

    # Build all distribution figures
    priority_bins = 20  # Default bins value
    with st.spinner("Generating plots..."):
        figures = build_figures(distribution_data, priority_bins=priority_bins)

    # Display figures in organized layout
    render_figure_layout(figures)

    # Display statistical summary with pre-computed stats
    render_statistical_summary(distribution_data)
