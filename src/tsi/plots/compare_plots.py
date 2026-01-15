"""Compare schedules page plotting functions."""

from __future__ import annotations

import plotly.graph_objects as go

from tsi.config import PLOT_HEIGHT


def create_priority_distribution_plot(
    current_priorities: list[float],
    comparison_priorities: list[float],
    current_name: str,
    comparison_name: str,
) -> go.Figure:
    """
    Create overlaid histogram of priority distributions.

    Args:
        current_priorities: List of priority values from current schedule
        comparison_priorities: List of priority values from comparison schedule
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule

    Returns:
        Plotly Figure
    """
    fig = go.Figure()

    current_count = len(current_priorities)
    comparison_count = len(comparison_priorities)

    trace_current = go.Histogram(
        x=current_priorities,
        name=current_name,
        opacity=1.0,
        marker=dict(color="#1f77b4", line=dict(color="#0d5a9e", width=2)),
        nbinsx=30,
    )

    trace_comparison = go.Histogram(
        x=comparison_priorities,
        name=comparison_name,
        opacity=1.0,
        marker=dict(color="#ff7f0e", line=dict(color="#cc6600", width=2)),
        nbinsx=30,
    )

    # Add larger dataset first, then smaller on top
    if current_count >= comparison_count:
        fig.add_trace(trace_current)
        fig.add_trace(trace_comparison)
    else:
        fig.add_trace(trace_comparison)
        fig.add_trace(trace_current)

    fig.update_layout(
        barmode="overlay",
        xaxis_title="Priority",
        yaxis_title="Count",
        height=450,
        legend=dict(
            orientation="h",
            yanchor="bottom",
            y=1.02,
            xanchor="right",
            x=1,
            font=dict(size=12, color="white"),
            bgcolor="rgba(0, 0, 0, 0.5)",
            bordercolor="white",
            borderwidth=1,
        ),
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    return fig


def create_scheduling_status_plot(
    current_scheduled_count: int,
    current_unscheduled_count: int,
    comp_scheduled_count: int,
    comp_unscheduled_count: int,
    current_name: str,
    comparison_name: str,
) -> go.Figure:
    """
    Create grouped bar chart of scheduling status.

    Args:
        current_scheduled_count: Number of scheduled blocks in current schedule
        current_unscheduled_count: Number of unscheduled blocks in current schedule
        comp_scheduled_count: Number of scheduled blocks in comparison schedule
        comp_unscheduled_count: Number of unscheduled blocks in comparison schedule
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule

    Returns:
        Plotly Figure
    """
    fig = go.Figure()

    fig.add_trace(
        go.Bar(
            name=current_name,
            x=["Scheduled", "Unscheduled"],
            y=[current_scheduled_count, current_unscheduled_count],
            marker=dict(
                color="#1f77b4",
                line=dict(color="#0d5a9e", width=2),
                pattern=dict(shape="/", bgcolor="#0d5a9e", fgcolor="#1f77b4", solidity=0.3),
            ),
            text=[f"{current_scheduled_count:,}", f"{current_unscheduled_count:,}"],
            textposition="auto",
            textfont=dict(color="white", size=12, family="Arial Black"),
            opacity=1.0,
        )
    )

    fig.add_trace(
        go.Bar(
            name=comparison_name,
            x=["Scheduled", "Unscheduled"],
            y=[comp_scheduled_count, comp_unscheduled_count],
            marker=dict(
                color="#ff7f0e",
                line=dict(color="#cc6600", width=2),
                pattern=dict(shape="\\", bgcolor="#cc6600", fgcolor="#ff7f0e", solidity=0.3),
            ),
            text=[f"{comp_scheduled_count:,}", f"{comp_unscheduled_count:,}"],
            textposition="auto",
            textfont=dict(color="white", size=12, family="Arial Black"),
            opacity=1.0,
        )
    )

    fig.update_layout(
        barmode="group",
        yaxis_title="Number of Blocks",
        height=450,
        legend=dict(
            orientation="h",
            yanchor="bottom",
            y=1.02,
            xanchor="right",
            x=1,
            font=dict(size=12, color="white"),
            bgcolor="rgba(0, 0, 0, 0.5)",
            bordercolor="white",
            borderwidth=1,
        ),
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    return fig


def create_changes_plot(
    newly_scheduled_count: int,
    newly_unscheduled_count: int,
) -> go.Figure:
    """
    Create visualization of scheduling changes.

    Args:
        newly_scheduled_count: Number of newly scheduled blocks
        newly_unscheduled_count: Number of newly unscheduled blocks

    Returns:
        Plotly Figure with bar chart
    """
    fig = go.Figure()

    fig.add_trace(
        go.Bar(
            x=["Newly Scheduled", "Newly Unscheduled"],
            y=[newly_scheduled_count, newly_unscheduled_count],
            marker=dict(
                color=["#2ca02c", "#d62728"],
                line=dict(color=["#1a7a1a", "#8b1a1a"], width=2),
            ),
            text=[f"{newly_scheduled_count:,}", f"{newly_unscheduled_count:,}"],
            textposition="auto",
            textfont=dict(color="white", size=14, family="Arial Black"),
            opacity=1.0,
        )
    )

    fig.update_layout(
        yaxis_title="Number of Blocks",
        height=PLOT_HEIGHT - 100,
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
        showlegend=False,
    )

    return fig


def create_time_distribution_plot(
    current_times: list[float],
    comparison_times: list[float],
    current_name: str,
    comparison_name: str,
) -> go.Figure:
    """
    Create box plot comparison of requested time distributions.

    Args:
        current_times: List of requested hours from current schedule
        comparison_times: List of requested hours from comparison schedule
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule
        current_scheduled: Current schedule's scheduled observations
        comparison_scheduled: Comparison schedule's scheduled observations
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule

    Returns:
        Plotly Figure
    """
    fig = go.Figure()

    trace_current = go.Box(
        y=current_times,
        name=current_name,
        marker=dict(color="#1f77b4", line=dict(color="#0d5a9e", width=2)),
        fillcolor="#1f77b4",
        line=dict(color="#0d5a9e", width=2),
        boxmean="sd",
        opacity=1.0,
    )

    trace_comparison = go.Box(
        y=comparison_times,
        name=comparison_name,
        marker=dict(color="#ff7f0e", line=dict(color="#cc6600", width=2)),
        fillcolor="#ff7f0e",
        line=dict(color="#cc6600", width=2),
        boxmean="sd",
        opacity=1.0,
    )

    fig.add_trace(trace_current)
    fig.add_trace(trace_comparison)

    fig.update_layout(
        yaxis_title="Requested Hours",
        height=PLOT_HEIGHT - 150,
        showlegend=True,
        legend=dict(
            orientation="h",
            yanchor="bottom",
            y=1.02,
            xanchor="right",
            x=1,
            font=dict(size=12, color="white"),
            bgcolor="rgba(0, 0, 0, 0.5)",
            bordercolor="white",
            borderwidth=1,
        ),
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    return fig
