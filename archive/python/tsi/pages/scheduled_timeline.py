"""Scheduled Timeline - Monthly view of scheduled observations."""

import streamlit as st

from tsi import state
from tsi.components.timeline.timeline_controls import render_search_filters
from tsi.components.timeline.timeline_figure import render_timeline_figure
from tsi.components.timeline.timeline_observable_periods import render_observable_periods_info
from tsi.components.timeline.timeline_observation_details import render_observation_details_table
from tsi.components.timeline.timeline_summary import render_timeline_summary
from tsi.services import (
    get_schedule_timeline_data,
)


def render() -> None:
    """Render the Scheduled Timeline page."""
    st.title("🗂️ Scheduled Timeline")

    st.markdown(
        """
        Monthly chronological view of scheduled observations. Each row represents a month
        and the horizontal axis shows the days (1-31) of that specific month.
        Colors represent the priority of each block (darker = lower priority).
        """
    )

    schedule_ref = state.get_schedule_ref()
    timeline_data = get_schedule_timeline_data(schedule_ref)

    if timeline_data.total_count == 0:
        st.warning("There are no scheduled observations with valid dates.")
        return

    # Build and display the monthly timeline figure
    priority_range = (timeline_data.priority_min, timeline_data.priority_max)
    render_timeline_figure(
        blocks=timeline_data.blocks,
        priority_range=priority_range,
        dark_periods=timeline_data.dark_periods,
    )

    # Show observable periods information if available
    if timeline_data.dark_periods:
        render_observable_periods_info(timeline_data.dark_periods)

    # Render observation details table with filters
    filters = render_search_filters(timeline_data.blocks)
    render_observation_details_table(timeline_data.blocks, filters)

    # Display summary metrics
    render_timeline_summary(timeline_data.blocks, timeline_data.unique_months)
