"""Scheduled Timeline - Monthly view of scheduled observations."""

import streamlit as st

from tsi import state
from tsi.components.timeline_controls import render_search_filters
from tsi.components.timeline_stats import (
    render_dark_period_summary,
    render_download_button,
    render_key_metrics,
)
from tsi.plots.timeline_monthly import build_monthly_timeline
from tsi.services.timeline_processing import (
    apply_search_filters,
    filter_dark_periods,
    filter_scheduled_data,
    get_priority_range,
    prepare_display_dataframe,
    prepare_scheduled_data,
)


def render() -> None:
    """Render the Scheduled Timeline page."""
    st.title("🗂️ Scheduled Timeline")

    st.markdown(
        """
        Monthly chronological view of scheduled observations. Each row represents a month
        and the horizontal axis shows the days (1-31) of that specific month, avoiding visual
        overflow. Colors represent the priority of each block (darker = lower priority).
        """
    )

    df = state.get_prepared_data()
    dark_periods_df = state.get_dark_periods()

    if df is None:
        st.warning("No data loaded. Please return to the landing page.")
        return

    # Prepare scheduled data
    scheduled_df = prepare_scheduled_data(df)

    if scheduled_df is None:
        st.warning("There are no scheduled observations with valid dates.")
        return

    # Calculate priority range
    priority_min, priority_max = get_priority_range(scheduled_df)

    # Get unique months
    all_months = sorted(scheduled_df["scheduled_month_label"].unique())

    # Apply filters (using defaults for now)
    priority_range = (priority_min, priority_max)
    selected_months = all_months

    filtered_df = filter_scheduled_data(
        scheduled_df,
        priority_range=priority_range,
        selected_months=selected_months,
    )

    if len(filtered_df) == 0:
        st.warning("There are no scheduled observations for the selected criteria.")
        return

    # Filter dark periods
    filtered_dark_periods = filter_dark_periods(dark_periods_df, selected_months)

    # Build and display the monthly timeline figure
    fig = build_monthly_timeline(filtered_df, priority_range, filtered_dark_periods)

    config = {
        "displayModeBar": True,
        "displaylogo": False,
        "modeBarButtonsToRemove": ["lasso2d", "select2d"],
        "scrollZoom": True,
    }
    st.plotly_chart(fig, use_container_width=True, config=config)

    # Show dark period summary if available
    if filtered_dark_periods is not None:
        render_dark_period_summary(filtered_dark_periods)

    # Observation details table
    st.markdown("---")
    st.subheader("📊 Observation Details")

    # Prepare display DataFrame
    display_df = prepare_display_dataframe(filtered_df)

    # Add search/filter capability
    filters = render_search_filters(filtered_df)

    # Apply search filters
    filtered_display = apply_search_filters(
        display_df,
        filters["search_id"],
        filters["search_month"],
        filters["min_priority_filter"],
    )

    # Display count
    st.caption(f"Showing {len(filtered_display):,} of {len(display_df):,} observations")

    # Display the table with formatting
    st.dataframe(
        filtered_display.style.format(
            {
                "Priority": "{:.2f}",
                "Duration (h)": "{:.2f}",
                "RA (°)": "{:.2f}",
                "Dec (°)": "{:.2f}",
                "Requested (h)": "{:.2f}",
                "Total Visibility (h)": "{:.2f}",
            },
            na_rep="-",
        ),
        width="stretch",
        height=400,
        hide_index=True,
    )

    # Download button
    render_download_button(filtered_display)

    # Display key metrics
    render_key_metrics(filtered_df)
