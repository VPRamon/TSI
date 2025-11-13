"""Metrics display components."""

import streamlit as st


def render_metrics_row(metrics: dict[str, tuple[str, str | None]]) -> None:
    """
    Render a row of metric cards.

    Args:
        metrics: Dictionary of {column_key: (value, delta)} pairs
    """
    cols = st.columns(len(metrics))

    for col, (label, (value, delta)) in zip(cols, metrics.items()):
        with col:
            st.metric(label=label, value=value, delta=delta)


def render_kpi_cards(
    total_obs: int,
    scheduled: int,
    mean_priority: float,
    total_visibility: float,
) -> None:
    """
    Render key performance indicator cards.

    Args:
        total_obs: Total number of observations
        scheduled: Number of scheduled observations
        mean_priority: Mean priority value
        total_visibility: Total visibility hours
    """
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric(
            label="Total Observations",
            value=f"{total_obs:,}",
        )

    with col2:
        scheduling_rate = (scheduled / total_obs * 100) if total_obs > 0 else 0
        st.metric(
            label="Scheduled",
            value=f"{scheduled:,}",
            delta=f"{scheduling_rate:.1f}%",
        )

    with col3:
        st.metric(
            label="Mean Priority",
            value=f"{mean_priority:.2f}",
        )

    with col4:
        st.metric(
            label="Total Visibility",
            value=f"{total_visibility:,.0f} hrs",
        )


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
