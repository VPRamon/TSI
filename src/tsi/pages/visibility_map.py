"""Visibility and schedule timeline page."""

import streamlit as st

from tsi import state
from tsi.components.visibility.visibility_controls import (
    render_generate_button,
    render_histogram_settings,
    render_sidebar_controls,
)
from tsi.components.visibility.visibility_stats import render_chart_info, render_dataset_statistics
from tsi.components.visibility.visibility_map_figure import render_visibility_map_figure
from tsi.services.database import get_visibility_map_data
from tsi.services.visibility_processing import (
    compute_effective_priority_range,
    filter_visibility_blocks,
    get_all_block_ids,
)


def render() -> None:
    """Render the Visibility Map."""
    st.title("📅 Visibility Map")

    st.markdown(
        """
        Histogram showing the number of targets visible over the observation period.
        """
    )

    # Check for schedule ID first - this is now required
    schedule_id = state.get_schedule_id()

    if schedule_id is None:
        st.error(
            "❌ No schedule loaded from database. "
            "The Visibility Map requires a schedule to be loaded from the database. "
            "Please go to the landing page and load a schedule."
        )
        return

    schedule_id = int(schedule_id)

    try:
        visibility_data = get_visibility_map_data(schedule_id=schedule_id)
    except Exception as exc:
        st.error(f"Failed to load visibility data from the backend: {exc}")
        return

    if visibility_data.total_count == 0:
        st.warning("No scheduling blocks were returned for this schedule.")
        return

    # Calculate priority range and get block IDs
    priority_min, priority_max = visibility_data.priority_min, visibility_data.priority_max
    all_block_ids = get_all_block_ids(visibility_data.blocks)

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

    # Filter data to get count of blocks matching filters
    filtered_blocks = filter_visibility_blocks(
        visibility_data.blocks,
        priority_range=effective_priority_range,
        block_ids=settings["selected_block_ids"] if settings["selected_block_ids"] else None,
    )

    # Show statistics FIRST - immediate feedback while histogram loads
    render_dataset_statistics(visibility_data.blocks, filtered_blocks)

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
                "🔄 Building visibility histogram using Rust backend... This is much faster!"
            ):
                # Call the modular figure component
                try:
                    fig = render_visibility_map_figure(
                        schedule_id=schedule_id,
                        settings=settings,
                        effective_priority_range=effective_priority_range,
                        total_blocks=len(filtered_blocks),
                    )
                except Exception as e:
                    st.error(f"Failed to generate histogram: {e}")
                    st.exception(e)

