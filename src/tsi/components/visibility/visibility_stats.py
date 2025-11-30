"""Visibility schedule statistics components."""

from __future__ import annotations

import pandas as pd
import streamlit as st


def render_dataset_statistics(df: pd.DataFrame, filtered_df: pd.DataFrame) -> None:
    """
    Render dataset statistics metrics.

    Args:
        df: Full DataFrame
        filtered_df: Filtered DataFrame
    """
    st.subheader("📊 Dataset Statistics")

    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Total Blocks", f"{len(df):,}")

    with col2:
        st.metric("Filtered Blocks", f"{len(filtered_df):,}")

    with col3:
        scheduled_count = filtered_df["scheduled_flag"].sum()
        st.metric("Scheduled", f"{int(scheduled_count):,}")

    with col4:
        avg_vis_periods = filtered_df["num_visibility_periods"].mean()
        if pd.notna(avg_vis_periods):
            st.metric("Avg Visibility Periods", f"{avg_vis_periods:.1f}")
        else:
            st.metric("Avg Visibility Periods", "N/A")


def render_chart_info() -> None:
    """Render information panel explaining the chart."""
    st.info(
        """
        **How to read this chart:**
        - **X-axis**: Time period of observations (UTC)
        - **Y-axis**: Number of blocks/targets that are visible at that time
        - **Color**: Intensity indicates density of visible targets (darker = more targets)

        This histogram aggregates all visibility windows into time bins, showing when
        the telescope has the most observation opportunities.
        """
    )
