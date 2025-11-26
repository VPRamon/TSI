"""Distribution page layout components."""

from __future__ import annotations

from typing import Any

import streamlit as st


def render_figure_layout(figures: dict[str, Any]) -> None:
    """
    Render all distribution figures in organized layout.

    Args:
        figures: Dictionary of figure names to Plotly figures
    """
    # Priority distribution (full width)
    st.subheader("Priority Distribution")
    st.plotly_chart(figures["priority_hist"], use_container_width=True)

    # Two-column layout for other distributions
    col1, col2 = st.columns(2)

    with col1:
        _render_visibility_figures(figures)

    with col2:
        _render_duration_and_status_figures(figures)

    # Comparison plot (full width)
    st.subheader("Priority Comparison by Scheduling Status")
    st.plotly_chart(figures["priority_violin"], use_container_width=True)


def _render_visibility_figures(figures: dict[str, Any]) -> None:
    """Render visibility-related figures."""
    st.subheader("Visibility Hours")
    st.plotly_chart(figures["visibility_hist"], use_container_width=True)

    st.subheader("Elevation Constraint Range")
    st.plotly_chart(figures["elevation_hist"], use_container_width=True)


def _render_duration_and_status_figures(figures: dict[str, Any]) -> None:
    """Render duration and status figures."""
    st.subheader("Requested Duration")
    st.plotly_chart(figures["duration_hist"], use_container_width=True)

    st.subheader("Scheduling Status")
    st.plotly_chart(figures["scheduled_bar"], use_container_width=True)
