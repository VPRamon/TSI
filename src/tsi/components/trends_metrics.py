"""Scheduling trends page metrics display components."""

from __future__ import annotations

import pandas as pd
import streamlit as st


def render_overview_metrics(df: pd.DataFrame) -> None:
    """
    Display overview metrics for scheduling trends analysis.
    
    Args:
        df: Filtered DataFrame
    """
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Total observations", f"{len(df):,}")

    with col2:
        n_scheduled = int(df["scheduled_flag"].sum())
        st.metric("Scheduled", f"{n_scheduled:,}")

    with col3:
        rate_scheduled = df["scheduled_flag"].mean() * 100
        st.metric("% Scheduled", f"{rate_scheduled:.1f}%")

    with col4:
        zero_vis = (df["total_visibility_hours"] == 0).sum()
        st.metric("Visibility = 0", f"{zero_vis:,}")
