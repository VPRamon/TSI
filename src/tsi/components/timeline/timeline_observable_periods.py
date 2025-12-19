"""Observable periods information component for scheduled timeline."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi.services.utils.time import format_datetime_utc, mjd_to_datetime


def render_observable_periods_info(dark_periods: list[tuple[float, float]]) -> None:
    """
    Render observable periods information section.

    Displays a summary and detailed table of dark/nocturnal observable periods
    with statistics and time ranges.

    Args:
        dark_periods: List of (start_mjd, stop_mjd) tuples for dark periods
    """
    if not dark_periods:
        return

    st.markdown("---")
    st.subheader("ℹ️ Observable periods information")

    # Calculate summary statistics
    dark_count = len(dark_periods)
    total_dark_hours = sum((stop - start) * 24.0 for start, stop in dark_periods)
    min_dark = min(start for start, _ in dark_periods)
    max_dark = max(stop for _, stop in dark_periods)

    # Display summary caption
    st.caption(
        f"Detected {dark_count:,} dark/nocturnal OBSERVABLE periods (total {total_dark_hours:,.1f} h). "
        f"The chart shows DAYTIME periods (non-observable) in light yellow. "
        f"Time range: {format_datetime_utc(mjd_to_datetime(min_dark))} → {format_datetime_utc(mjd_to_datetime(max_dark))}."
    )

    # Convert to display format
    display_data = []
    for start_mjd, stop_mjd in dark_periods:
        start_dt = mjd_to_datetime(start_mjd)
        stop_dt = mjd_to_datetime(stop_mjd)
        duration_hours = (stop_mjd - start_mjd) * 24.0
        month_label = start_dt.strftime("%Y-%m")
        display_data.append(
            {
                "Start": start_dt.strftime("%Y-%m-%d %H:%M"),
                "End": stop_dt.strftime("%Y-%m-%d %H:%M"),
                "Duration (h)": duration_hours,
                "Month": month_label,
            }
        )

    dark_display = pd.DataFrame(display_data)

    # Display the table
    st.dataframe(
        dark_display,
        width="stretch",
        hide_index=True,
        height=min(300, 60 + 24 * min(len(dark_display), 8)),
    )
