"""Scheduled timeline control components."""

from __future__ import annotations

import streamlit as st


def render_search_filters(filtered_df) -> dict:
    """
    Render search and filter controls for the observation table.

    Args:
        filtered_df: Filtered DataFrame containing observations

    Returns:
        Dictionary with filter values
    """
    col_search1, col_search2, col_search3 = st.columns(3)

    with col_search1:
        search_id = st.text_input(
            "🔍 Search by ID",
            key="timeline_search_id",
            placeholder="e.g., SB001",
        )

    with col_search2:
        search_month = st.selectbox(
            "📅 Month",
            options=["All"] + sorted(filtered_df["scheduled_month_label"].unique().tolist()),
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
        "search_id": search_id,
        "search_month": search_month,
        "min_priority_filter": min_priority_filter,
    }
