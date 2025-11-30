"""Distribution page statistics components."""

from __future__ import annotations

import streamlit as st


def render_statistical_summary(distribution_data) -> None:
    """
    Render statistical summary for the distribution data.

    Args:
        distribution_data: DistributionData object with pre-computed statistics
    """
    st.divider()
    st.subheader("Statistical Summary")

    col1, col2, col3 = st.columns(3)

    with col1:
        _render_priority_stats(distribution_data.priority_stats)

    with col2:
        _render_visibility_stats(distribution_data.visibility_stats)

    with col3:
        _render_requested_hours_stats(distribution_data.requested_hours_stats)


def _render_priority_stats(stats) -> None:
    """Render priority statistics."""
    st.markdown("**Priority**")
    st.write(f"Mean: {stats.mean:.2f}")
    st.write(f"Median: {stats.median:.2f}")
    st.write(f"Std Dev: {stats.std_dev:.2f}")


def _render_visibility_stats(stats) -> None:
    """Render visibility hours statistics."""
    st.markdown("**Visibility Hours**")
    st.write(f"Mean: {stats.mean:.1f}")
    st.write(f"Median: {stats.median:.1f}")
    st.write(f"Total: {stats.sum:,.0f}")


def _render_requested_hours_stats(stats) -> None:
    """Render requested hours statistics."""
    st.markdown("**Requested Hours**")
    st.write(f"Mean: {stats.mean:.2f}")
    st.write(f"Median: {stats.median:.2f}")
    st.write(f"Total: {stats.sum:,.1f}")
