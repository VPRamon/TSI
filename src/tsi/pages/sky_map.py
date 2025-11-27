"""Sky Map page - celestial coordinate visualization with advanced filtering."""

from __future__ import annotations

import streamlit as st

from tsi import state
from tsi.components.sky_map_controls import render_sidebar_controls
from tsi.components.sky_map_stats import render_stats
from tsi.plots.sky_map import build_figure
from tsi.services import get_priority_range
from tsi.services.sky_map_filters import (
    build_palette,
    filter_dataframe,
    prepare_priority_bins,
)


def render() -> None:
    """Render the Sky Map page."""
    st.title("🌌 Sky Map")

    st.markdown(
        """
        Visualize the distribution of targets in celestial coordinates and apply advanced filters
        to understand how they vary by priority and scheduling status.
        """
    )

    df = state.get_prepared_data()

    # Prepare data
    priority_min, priority_max = get_priority_range(df)
    df, priority_bins = prepare_priority_bins(df)

    if not priority_bins:
        st.warning("No original priority bins available in the dataset.")
        return

    # Panel lateral izquierdo y sky map derecho
    sidebar_col, map_col = st.columns([1, 3], gap="large")

    with sidebar_col:
        controls = render_sidebar_controls(
            df=df,
            priority_min=priority_min,
            priority_max=priority_max,
            priority_bins=priority_bins,
        )

    with map_col:
        filtered_df = filter_dataframe(
            df,
            priority_range=controls["priority_range"],
            scheduled_filter=controls["scheduled_filter"],
            selected_bins=controls["selected_bins"],
            schedule_window=controls["schedule_window"],
        )

        if filtered_df.empty:
            st.warning("No targets match the selected filters.")
            return

        category_palette = None
        if controls["color_column"] == "priority_bin":
            category_palette = build_palette(priority_bins)

        fig = build_figure(
            df=filtered_df,
            color_by=controls["color_column"],
            size_by="requested_hours",
            flip_ra=controls["flip_ra"],
            category_palette=category_palette,
        )

        st.plotly_chart(fig, use_container_width=True, key="sky_map_chart")

        st.markdown("---")
        render_stats(filtered_df)
