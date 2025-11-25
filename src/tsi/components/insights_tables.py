"""Insights page table display components."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi.components.data_preview import render_conflicts_table, render_data_preview


def render_top_observations(
    top_priority: pd.DataFrame,
    top_visibility: pd.DataFrame,
) -> None:
    """
    Display top observations in tabbed layout.
    
    Args:
        top_priority: Top observations by priority
        top_visibility: Top observations by visibility hours
    """
    st.header("🏆 Top Observations")

    tab1, tab2 = st.tabs(["By Priority", "By Visibility Hours"])

    with tab1:
        if not top_priority.empty:
            render_data_preview(
                top_priority,
                max_rows=10,
                title="Top 10 by Priority",
            )
        else:
            st.info("No data available")

    with tab2:
        if not top_visibility.empty:
            render_data_preview(
                top_visibility,
                max_rows=10,
                title="Top 10 by Total Visibility Hours",
            )
        else:
            st.info("No data available")


def render_integrity_checks(conflicts: pd.DataFrame) -> None:
    """
    Display scheduling integrity checks and conflict information.
    
    Args:
        conflicts: DataFrame with detected conflicts
    """
    st.header("🔎 Scheduling Integrity")

    render_conflicts_table(conflicts)

    if not conflicts.empty:
        st.warning(
            """
            **Conflict Types:**
            - **Outside visibility**: Scheduled period doesn't fall within any visibility window
            - **Before/after fixed constraints**: Scheduled outside the fixed start/stop times
            """
        )
