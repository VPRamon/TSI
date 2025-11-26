"""Visibility and schedule timeline page."""

import streamlit as st

from tsi import state
from tsi.components.visibility_controls import (
    render_generate_button,
    render_histogram_settings,
    render_sidebar_controls,
)
from tsi.components.visibility_stats import render_chart_info, render_dataset_statistics
from tsi.plots.timeline import build_visibility_histogram
from tsi.services.loaders import get_filtered_dataframe
from tsi.services.visibility_processing import (
    compute_effective_priority_range,
    get_all_block_ids,
    get_priority_range,
)


def render() -> None:
    """Render the Visibility Map & Schedule page."""
    st.title("📅 Visibility Map & Schedule")

    st.markdown(
        """
        Histogram showing the number of targets visible over the observation period.
        This view is optimized for analyzing large datasets with thousands of blocks.
        """
    )

    df = state.get_prepared_data()

    if df is None:
        st.warning("No data loaded. Please return to the landing page.")
        return

    # Calculate priority range and get block IDs
    priority_min, priority_max = get_priority_range(df)
    all_block_ids = get_all_block_ids(df)

    # Sidebar controls
    with st.sidebar:
        priority_range = render_sidebar_controls(priority_min, priority_max)

    # Main-panel histogram settings
    settings = render_histogram_settings(priority_min, priority_max, all_block_ids)

    # Compute effective priority range
    effective_priority_range = compute_effective_priority_range(
        priority_range,
        settings["priority_filter_range"],
    )

    # Filter data BEFORE parsing visibility - major performance improvement
    filtered_df = get_filtered_dataframe(
        df,
        priority_range=effective_priority_range,
        scheduled_filter="All",
        block_ids=settings["selected_block_ids"] if settings["selected_block_ids"] else None,
    )

    # Show statistics FIRST - immediate feedback while histogram loads
    render_dataset_statistics(df, filtered_df)

    st.divider()

    # Information panel BEFORE heavy computation
    render_chart_info()

    # Create placeholder for the histogram
    histogram_container = st.container()

    # Generate button
    should_generate = render_generate_button()

    # Only build histogram if button was clicked or if we have a cached result
    if should_generate:
        # Mark that we've generated it at least once
        st.session_state["visibility_histogram_generated"] = True

        with histogram_container:
            with st.spinner(
                "🔄 Building visibility histogram... This may take 10-30 seconds for large datasets."
            ):
                fig = build_visibility_histogram(
                    df=filtered_df,
                    num_bins=settings["num_bins"],
                    bin_duration_minutes=settings["bin_duration_minutes"],
                )

            st.plotly_chart(fig, use_container_width=True)
    else:
        with histogram_container:
            st.info(
                "👆 Click 'Generate Histogram' above to build the visualization. This prevents automatic computation on page load."
            )
