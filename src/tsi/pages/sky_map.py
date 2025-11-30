"""Sky Map page - celestial coordinate visualization with advanced filtering."""

from __future__ import annotations

import streamlit as st

from tsi import state
from tsi.components.sky_map_controls import render_sidebar_controls
from tsi.components.sky_map_stats import render_stats
from tsi.plots.sky_map import build_figure
from tsi.services.database import get_sky_map_data
from tsi.services.sky_map_filters import filter_blocks


def render() -> None:
    """Render the Sky Map page."""
    st.title("🌌 Sky Map")

    st.markdown(
        """
        Visualize the distribution of targets in celestial coordinates and apply advanced filters
        to understand how they vary by priority and scheduling status.
        """
    )

    schedule_id = state.get_schedule_id()

    if schedule_id is None:
        st.info("Load a schedule from the database to view the Sky Map.")
        return
    
    schedule_id = int(schedule_id)

    try:
        sky_map_data = get_sky_map_data(schedule_id=schedule_id)
    except Exception as exc:
        st.error(f"Failed to load sky map data from the backend: {exc}")
        return

    if not sky_map_data or sky_map_data.total_count == 0:
        st.warning("No scheduling blocks were returned for this schedule.")
        return

    blocks = sky_map_data.blocks
    priority_bins = sky_map_data.priority_bins
    
    # Extract bin labels for filtering
    bin_labels = [bin_info.label for bin_info in priority_bins]

    # Panel lateral izquierdo y sky map derecho
    sidebar_col, map_col = st.columns([1, 3], gap="large")

    with sidebar_col:
        controls = render_sidebar_controls(
            blocks=blocks,
            priority_min=sky_map_data.priority_min,
            priority_max=sky_map_data.priority_max,
            priority_bins=bin_labels,
        )

    with map_col:
        filtered_blocks = filter_blocks(
            blocks,
            priority_range=controls["priority_range"],
            scheduled_filter=controls["scheduled_filter"],
            selected_bins=controls["selected_bins"],
            schedule_window=controls["schedule_window"],
        )

        if not filtered_blocks:
            st.warning("No targets match the selected filters.")
            return

        # Build color palette from priority bins computed in Rust
        category_palette = None
        if controls["color_column"] == "priority_bin":
            category_palette = {
                bin_info.label: bin_info.color
                for bin_info in priority_bins
            }

        fig = build_figure(
            _blocks=filtered_blocks,
            color_by=controls["color_column"],
            size_by="requested_hours",
            flip_ra=controls["flip_ra"],
            category_palette=category_palette,
        )

        st.plotly_chart(fig, width='stretch', key="sky_map_chart")
        st.markdown("---")
        render_stats(filtered_blocks)
