"""Metrics display components."""

import streamlit as st


def render_comparison_metrics(
    label1: str,
    value1: float,
    label2: str,
    value2: float,
    metric_name: str = "Value",
) -> None:
    """
    Render side-by-side comparison metrics.

    Args:
        label1: First label
        value1: First value
        label2: Second label
        value2: Second value
        metric_name: Name of the metric being compared
    """
    col1, col2 = st.columns(2)

    with col1:
        st.metric(
            label=f"{label1} {metric_name}",
            value=f"{value1:.2f}",
        )

    with col2:
        delta = value2 - value1
        st.metric(
            label=f"{label2} {metric_name}",
            value=f"{value2:.2f}",
            delta=f"{delta:+.2f}",
        )
