"""Scheduled timeline statistics and display components."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi.services.time_utils import format_datetime_utc


def render_dark_period_summary(dark_periods: list[tuple[float, float]]) -> None:
    """
    Render dark period summary information.

    Args:
        dark_periods: List of (start_mjd, stop_mjd) tuples for dark periods
    """
    if not dark_periods:
        return
    
    from tsi.services.time_utils import mjd_to_datetime
    
    st.markdown("---")
    st.subheader("ℹ️ Observable periods information")

    dark_count = len(dark_periods)
    total_dark_hours = sum((stop - start) * 24.0 for start, stop in dark_periods)
    min_dark = min(start for start, _ in dark_periods)
    max_dark = max(stop for _, stop in dark_periods)

    st.caption(
        f"Detected {dark_count:,} dark/nocturnal OBSERVABLE periods (total {total_dark_hours:,.1f} h). "
        f"The chart shows DAYTIME periods (non-observable) in light yellow."
        f" Time range: {format_datetime_utc(mjd_to_datetime(min_dark))} → {format_datetime_utc(mjd_to_datetime(max_dark))}."
    )

    # Convert to display format
    display_data = []
    for start_mjd, stop_mjd in dark_periods:
        start_dt = mjd_to_datetime(start_mjd)
        stop_dt = mjd_to_datetime(stop_mjd)
        duration_hours = (stop_mjd - start_mjd) * 24.0
        month_label = start_dt.strftime("%Y-%m")
        display_data.append({
            "Start": start_dt.strftime("%Y-%m-%d %H:%M"),
            "End": stop_dt.strftime("%Y-%m-%d %H:%M"),
            "Duration (h)": duration_hours,
            "Month": month_label,
        })
    
    dark_display = pd.DataFrame(display_data)

    st.dataframe(
        dark_display,
        use_container_width=True,
        hide_index=True,
        height=min(300, 60 + 24 * min(len(dark_display), 8)),
    )


def render_key_metrics(blocks: list, unique_months: list[str]) -> None:
    """
    Render key metrics about scheduled observations.

    Args:
        blocks: List of ScheduleTimelineBlock objects
        unique_months: List of unique month labels
    """
    from tsi.services.time_utils import mjd_to_datetime
    
    st.markdown("---")
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Scheduled blocks", f"{len(blocks):,}")

    with col2:
        total_hours = sum((block.scheduled_stop_mjd - block.scheduled_start_mjd) * 24.0 for block in blocks)
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
        st.caption(f"**Time range:** {format_datetime_utc(min_date)} → {format_datetime_utc(max_date)}")


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
