"""Scheduled timeline statistics and display components."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi.services.utils.time import format_datetime_utc


def render_key_metrics(blocks: list, unique_months: list[str]) -> None:
    """
    Render key metrics about scheduled observations.

    Args:
        blocks: List of ScheduleTimelineBlock objects
        unique_months: List of unique month labels
    """
    from tsi.services.utils.time import mjd_to_datetime

    st.markdown("---")
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Scheduled blocks", f"{len(blocks):,}")

    with col2:
        total_hours = sum(
            (block.scheduled_stop_mjd - block.scheduled_start_mjd) * 24.0 for block in blocks
        )
        st.metric("Total hours", f"{total_hours:,.1f}")

    with col3:
        if blocks:
            avg_duration = total_hours / len(blocks)
            st.metric("Average duration", f"{avg_duration:.2f} h")
        else:
            st.metric("Average duration", "N/A")

    with col4:
        st.metric("Months covered", f"{len(unique_months)}")

    # Date range info
    if blocks:
        min_mjd = min(block.scheduled_start_mjd for block in blocks)
        max_mjd = max(block.scheduled_stop_mjd for block in blocks)
        min_date = mjd_to_datetime(min_mjd)
        max_date = mjd_to_datetime(max_mjd)
        st.caption(
            f"**Time range:** {format_datetime_utc(min_date)} → {format_datetime_utc(max_date)}"
        )


def render_download_button(display_df: pd.DataFrame) -> None:
    """
    Render CSV download button.

    Args:
        display_df: DataFrame to download
    """
    csv = display_df.to_csv(index=False).encode("utf-8")
    st.download_button(
        label="📥 Download table as CSV",
        data=csv,
        file_name=f"scheduled_timeline_{pd.Timestamp.now().strftime('%Y%m%d_%H%M%S')}.csv",
        mime="text/csv",
        key="timeline_download_csv",
    )
