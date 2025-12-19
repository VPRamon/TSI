"""Visibility schedule statistics components."""

from __future__ import annotations

from collections.abc import Sequence
from typing import Any

import streamlit as st


def render_dataset_statistics(all_blocks: Sequence[Any], filtered_blocks: Sequence[Any]) -> None:
    """
    Render dataset statistics metrics.

    Args:
        all_blocks: All blocks returned by the backend
        filtered_blocks: Blocks after applying page filters
    """
    st.subheader("📊 Dataset Statistics")

    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Total Blocks", f"{len(all_blocks):,}")

    with col2:
        st.metric("Filtered Blocks", f"{len(filtered_blocks):,}")

    with col3:
        scheduled_count = sum(1 for block in filtered_blocks if getattr(block, "scheduled", False))
        st.metric("Scheduled", f"{int(scheduled_count):,}")

    with col4:
        if filtered_blocks:
            total_periods = sum(
                float(getattr(block, "num_visibility_periods", 0) or 0) for block in filtered_blocks
            )
            avg_vis_periods = total_periods / len(filtered_blocks)
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
