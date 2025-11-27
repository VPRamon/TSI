"""Scheduled timeline statistics and display components."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi.services.rust_compat import format_datetime_utc_rust as format_datetime_utc


def render_dark_period_summary(dark_periods_df: pd.DataFrame) -> None:
    """
    Render dark period summary information.

    Args:
        dark_periods_df: DataFrame containing dark period information
    """
    st.markdown("---")
    st.subheader("ℹ️ Observable periods information")

    dark_count = len(dark_periods_df)
    total_dark_hours = dark_periods_df["duration_hours"].sum()
    min_dark = dark_periods_df["start_dt"].min()
    max_dark = dark_periods_df["stop_dt"].max()

    st.caption(
        f"Detected {dark_count:,} dark/nocturnal OBSERVABLE periods (total {total_dark_hours:,.1f} h). "
        f"The chart shows DAYTIME periods (non-observable) in light yellow."
        f" Time range: {format_datetime_utc(min_dark)} → {format_datetime_utc(max_dark)}."
    )

    dark_display = dark_periods_df.copy()
    dark_display["Start"] = dark_display["start_dt"].dt.strftime("%Y-%m-%d %H:%M")
    dark_display["End"] = dark_display["stop_dt"].dt.strftime("%Y-%m-%d %H:%M")
    dark_display = dark_display[["Start", "End", "duration_hours", "months"]]
    dark_display = dark_display.rename(
        columns={"duration_hours": "Duration (h)", "months": "Months"}
    )

    st.dataframe(
        dark_display,
        width="stretch",
        hide_index=True,
        height=min(300, 60 + 24 * min(len(dark_display), 8)),
    )


def render_key_metrics(filtered_df: pd.DataFrame) -> None:
    """
    Render key metrics about scheduled observations.

    Args:
        filtered_df: Filtered DataFrame containing observations
    """
    st.markdown("---")
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Scheduled blocks", f"{len(filtered_df):,}")

    with col2:
        total_hours = filtered_df["duration_hours"].sum()
        st.metric("Total hours", f"{total_hours:,.1f}")

    with col3:
        avg_duration = filtered_df["duration_hours"].mean()
        st.metric("Average duration", f"{avg_duration:.2f} h")

    with col4:
        num_months = filtered_df["scheduled_month_label"].nunique()
        st.metric("Months covered", f"{num_months}")

    # Date range info
    min_date = filtered_df["scheduled_start_dt"].min()
    max_date = filtered_df["scheduled_stop_dt"].max()
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
