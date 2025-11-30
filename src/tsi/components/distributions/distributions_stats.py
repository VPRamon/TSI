"""Distribution page statistics components."""

from __future__ import annotations

import pandas as pd
import streamlit as st


def render_statistical_summary(df: pd.DataFrame) -> None:
    """
    Render statistical summary for the filtered dataset.

    Args:
        df: Filtered DataFrame to display statistics for
    """
    st.divider()
    st.subheader("Statistical Summary")

    col1, col2, col3 = st.columns(3)

    with col1:
        _render_priority_stats(df)

    with col2:
        _render_visibility_stats(df)

    with col3:
        _render_requested_hours_stats(df)


def _render_priority_stats(df: pd.DataFrame) -> None:
    """Render priority statistics."""
    st.markdown("**Priority**")
    st.write(f"Mean: {df['priority'].mean():.2f}")
    st.write(f"Median: {df['priority'].median():.2f}")
    st.write(f"Std Dev: {df['priority'].std():.2f}")


def _render_visibility_stats(df: pd.DataFrame) -> None:
    """Render visibility hours statistics."""
    st.markdown("**Visibility Hours**")
    st.write(f"Mean: {df['total_visibility_hours'].mean():.1f}")
    st.write(f"Median: {df['total_visibility_hours'].median():.1f}")
    st.write(f"Total: {df['total_visibility_hours'].sum():,.0f}")


def _render_requested_hours_stats(df: pd.DataFrame) -> None:
    """Render requested hours statistics."""
    st.markdown("**Requested Hours**")
    st.write(f"Mean: {df['requested_hours'].mean():.2f}")
    st.write(f"Median: {df['requested_hours'].median():.2f}")
    st.write(f"Total: {df['requested_hours'].sum():,.1f}")
