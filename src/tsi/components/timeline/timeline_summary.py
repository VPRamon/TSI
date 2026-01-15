"""Timeline summary metrics component."""

from __future__ import annotations

import streamlit as st

from tsi.services.utils.time import format_datetime_utc, mjd_to_datetime


def render_timeline_summary(blocks: list, unique_months: list[str]) -> None:
    """
    Render key metrics summary for scheduled timeline.

    Displays overview statistics including total blocks, hours, average duration,
    and months covered.

    Args:
        blocks: List of ScheduleTimelineBlock objects
        unique_months: List of unique month labels (e.g., ["2024-01", "2024-02"])
    """
    st.markdown("---")

    # Calculate metrics
    total_blocks = len(blocks)
    total_hours = sum(
        (float(block.scheduled_stop_mjd) - float(block.scheduled_start_mjd)) * 24.0
        for block in blocks
    )
    avg_duration = total_hours / total_blocks if total_blocks > 0 else 0
    months_covered = len(unique_months)

    # Display metrics in columns
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Scheduled blocks", f"{total_blocks:,}")

    with col2:
        st.metric("Total hours", f"{total_hours:,.1f}")

    with col3:
        st.metric("Average duration", f"{avg_duration:.2f} h" if total_blocks > 0 else "N/A")

    with col4:
        st.metric("Months covered", f"{months_covered}")

    # Display date range
    if blocks:
        min_mjd = min(float(block.scheduled_start_mjd) for block in blocks)
        max_mjd = max(float(block.scheduled_stop_mjd) for block in blocks)
        min_date = mjd_to_datetime(min_mjd)
        max_date = mjd_to_datetime(max_mjd)
        st.caption(
            f"**Time range:** {format_datetime_utc(min_date)} → {format_datetime_utc(max_date)}"
        )
