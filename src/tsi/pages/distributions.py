"""Distributions page - statistical visualizations."""

import streamlit as st

from tsi import state
from tsi.components.distributions_controls import render_filter_control
from tsi.components.distributions_layout import render_figure_layout
from tsi.components.distributions_stats import render_statistical_summary
from tsi.plots.distributions import build_figures
from tsi.services.distributions_filters import filter_impossible_observations


def render() -> None:
    """Render the Distributions page."""
    # Title and filter controls in same row
    col1, col2 = st.columns([3, 1])

    with col1:
        st.title("📊 Distributions")

    df = state.get_prepared_data()

    if df is None:
        st.warning("No data loaded. Please return to the landing page.")
        return

    st.markdown(
        """
        Statistical analysis and distribution visualizations of key scheduling parameters.
        """
    )

    # Render filter control
    with col2:
        filter_mode, filter_supported = render_filter_control(df)

    # Apply filtering
    filtered_df = filter_impossible_observations(df, filter_mode)

    if filtered_df.empty:
        st.warning("⚠️ No observations available with the selected filter.")
        return

    # Build all distribution figures
    priority_bins = 20  # Default bins value
    with st.spinner("Generating plots..."):
        figures = build_figures(filtered_df, priority_bins=priority_bins)

    # Display figures in organized layout
    render_figure_layout(figures)

    # Display statistical summary
    render_statistical_summary(filtered_df)