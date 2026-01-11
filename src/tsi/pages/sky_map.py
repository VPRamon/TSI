"""Sky Map page - celestial coordinate visualization with advanced filtering."""

from __future__ import annotations

import streamlit as st

from tsi import state
from tsi.components.sky_map.sky_map_controls import render_sidebar_controls
from tsi.components.sky_map.sky_map_figure import render_sky_map_figure
from tsi.components.sky_map.sky_map_stats import render_stats
from tsi.services import get_sky_map_data
from tsi.services.filters.sky_map import filter_blocks


def render() -> None:
    """Render the Sky Map page."""
    st.title("🌌 Sky Map")

    st.markdown(
        """
        Visualize the distribution of targets in celestial coordinates and apply advanced filters
        to understand how they vary by priority and scheduling status.
        """
    )

    schedule_ref = state.get_schedule_ref()

    try:
        sky_map_data = get_sky_map_data(schedule_ref)
    except Exception as exc:
        st.error(f"Failed to load sky map data from the backend: {exc}")
        return

    if len(sky_map_data.blocks) == 0:
        st.warning("No scheduling blocks were returned for this schedule.")
        return

    # Panel lateral izquierdo y sky map derecho
    sidebar_col, map_col = st.columns([1, 3], gap="large")

    with sidebar_col:
        controls = render_sidebar_controls(
            blocks=sky_map_data.blocks,
            priority_min=sky_map_data.priority_min,
            priority_max=sky_map_data.priority_max,
            priority_bins=[bin_info.label for bin_info in sky_map_data.priority_bins],
        )

    with map_col:
        filtered_blocks = filter_blocks(
            sky_map_data.blocks,
            priority_range=controls["priority_range"],
            scheduled_filter=controls["scheduled_filter"],
            selected_bins=controls["selected_bins"],
            schedule_window=controls["schedule_window"],
        )

        render_sky_map_figure(
            blocks=filtered_blocks,
            controls=controls,
            priority_bins=sky_map_data.priority_bins,
        )

        st.markdown("---")
        render_stats(filtered_blocks)
