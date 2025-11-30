"""Scheduled timeline control components."""

from __future__ import annotations

import streamlit as st


def render_search_filters(blocks: list) -> dict:
    """
    Render search and filter controls for the observation table.

    Args:
        blocks: List of ScheduleTimelineBlock objects

    Returns:
        Dictionary with filter values
    """
    from tsi.services.time_utils import mjd_to_datetime
    
    # Extract unique months from blocks
    unique_months = set()
    for block in blocks:
        month_label = mjd_to_datetime(block.scheduled_start_mjd).strftime("%Y-%m")
        unique_months.add(month_label)
    
    col_search1, col_search2, col_search3 = st.columns(3)

    with col_search1:
        search_id = st.text_input(
            "🔍 Search by ID",
            key="timeline_search_id",
            placeholder="e.g., 12345",
        )

    with col_search2:
        search_month = st.selectbox(
            "📅 Month",
            options=["All"] + sorted(unique_months),
            key="timeline_search_month",
        )

    with col_search3:
        min_priority_filter = st.number_input(
            "⭐ Minimum priority",
            min_value=0.0,
            max_value=10.0,
            value=0.0,
            step=0.5,
            key="timeline_min_priority_filter",
        )

    return {
        "search_id": search_id if search_id else None,
        "search_month": search_month if search_month != "All" else None,
        "min_priority_filter": min_priority_filter if min_priority_filter > 0 else None,
    }
