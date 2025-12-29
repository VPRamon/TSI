"""Visibility and schedule timeline page."""

import streamlit as st

from tsi import state
from tsi.components.visibility.visibility_controls import (
    render_histogram_settings,
    render_sidebar_controls,
)
from tsi.components.visibility.visibility_map_figure import render_visibility_map_figure
from tsi.components.visibility.visibility_stats import render_dataset_statistics
from tsi.services import backend_client
from tsi.services.utils.visibility_processing import (
    filter_visibility_blocks,
)


def render() -> None:
    """Render the Visibility Map."""
    st.title("📅 Visibility Map")

    st.markdown(
        """
        Histogram showing the number of targets observable over the observation period.
        """
    )

    # Check for schedule reference first - this is now required
    schedule_ref = state.get_schedule_ref()

    visibility_data = backend_client.get_visibility_map_data(schedule_ref)

    if visibility_data.total_count == 0:
        st.warning("No scheduling blocks were returned for this schedule.")
        return

    # Calculate priority range and get block IDs
    priority_range = (visibility_data.priority_min, visibility_data.priority_max)
    all_block_ids = sorted(block.original_block_id for block in visibility_data.blocks)

    # Main-panel histogram settings
    settings = render_histogram_settings(priority_range, all_block_ids)

    # Filter data to get count of blocks matching filters
    filtered_blocks = filter_visibility_blocks(
        visibility_data.blocks,
        priority_range=priority_range,
        block_ids=settings["selected_block_ids"] if settings["selected_block_ids"] else None,
    )

    # Create placeholder for the histogram and build it immediately on page load
    histogram_container = st.container()

    with histogram_container:
        with st.spinner(
            "🔄 Building visibility histogram using Rust backend... This is much faster!"
        ):
            # Call the modular figure component
            try:
                render_visibility_map_figure(
                    schedule_ref=schedule_ref,
                    settings=settings,
                    effective_priority_range=priority_range,
                    total_blocks=len(filtered_blocks),
                )
            except Exception as e:
                st.error(f"Failed to generate histogram: {e}")
                st.exception(e)

    st.divider()

    render_dataset_statistics(visibility_data.blocks, filtered_blocks)
