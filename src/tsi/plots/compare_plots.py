"""Compare schedules page plotting functions."""

from __future__ import annotations

import pandas as pd
import plotly.graph_objects as go
from plotly.subplots import make_subplots

from tsi.config import PLOT_HEIGHT


def create_priority_distribution_plot(
    current_scheduled: pd.DataFrame,
    comparison_scheduled: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> go.Figure:
    """
    Create overlaid histogram of priority distributions.

    Args:
        current_scheduled: Current schedule's scheduled observations
        comparison_scheduled: Comparison schedule's scheduled observations
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule

    Returns:
        Plotly Figure
    """
    fig = go.Figure()

    # Determine which dataset has fewer items to plot it on top
    current_count = len(current_scheduled)
    comparison_count = len(comparison_scheduled)

    trace_current = go.Histogram(
        x=current_scheduled["priority"],
        name=current_name,
        opacity=1.0,
        marker=dict(color="#1f77b4", line=dict(color="#0d5a9e", width=2)),
        nbinsx=30,
    )

    trace_comparison = go.Histogram(
        x=comparison_scheduled["priority"],
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
    current_common: pd.DataFrame,
    comparison_common: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> go.Figure:
    """
    Create grouped bar chart of scheduling status.

    Args:
        current_common: Current schedule with common blocks
        comparison_common: Comparison schedule with common blocks
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule

    Returns:
        Plotly Figure
    """
    current_scheduled = (current_common["scheduled_flag"] == 1).sum()
    current_unscheduled = (current_common["scheduled_flag"] == 0).sum()

    comp_scheduled = (comparison_common["scheduled_flag"] == 1).sum()
    comp_unscheduled = (comparison_common["scheduled_flag"] == 0).sum()

    fig = go.Figure()

    fig.add_trace(
        go.Bar(
            name=current_name,
            x=["Scheduled", "Unscheduled"],
            y=[current_scheduled, current_unscheduled],
            marker=dict(
                color="#1f77b4",
                line=dict(color="#0d5a9e", width=2),
                pattern=dict(shape="/", bgcolor="#0d5a9e", fgcolor="#1f77b4", solidity=0.3),
            ),
            text=[f"{current_scheduled:,}", f"{current_unscheduled:,}"],
            textposition="auto",
            textfont=dict(color="white", size=12, family="Arial Black"),
            opacity=1.0,
        )
    )

    fig.add_trace(
        go.Bar(
            name=comparison_name,
            x=["Scheduled", "Unscheduled"],
            y=[comp_scheduled, comp_unscheduled],
            marker=dict(
                color="#ff7f0e",
                line=dict(color="#cc6600", width=2),
                pattern=dict(shape="\\", bgcolor="#cc6600", fgcolor="#ff7f0e", solidity=0.3),
            ),
            text=[f"{comp_scheduled:,}", f"{comp_unscheduled:,}"],
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
    newly_scheduled: pd.DataFrame,
    newly_unscheduled: pd.DataFrame,
) -> go.Figure:
    """
    Create visualization of scheduling changes.

    Args:
        newly_scheduled: Newly scheduled blocks
        newly_unscheduled: Newly unscheduled blocks

    Returns:
        Plotly Figure with subplots
    """
    fig = make_subplots(
        rows=1,
        cols=2,
        subplot_titles=("Newly Scheduled Blocks", "Newly Unscheduled Blocks"),
        specs=[[{"type": "histogram"}, {"type": "histogram"}]],
    )

    fig.add_trace(
        go.Histogram(
            x=newly_scheduled["priority_current"] if len(newly_scheduled) > 0 else [],
            name="Newly Scheduled",
            marker=dict(color="#2ca02c", line=dict(color="#1a7a1a", width=2)),
            nbinsx=20,
            showlegend=False,
            opacity=1.0,
        ),
        row=1,
        col=1,
    )

    fig.add_trace(
        go.Histogram(
            x=newly_unscheduled["priority_current"] if len(newly_unscheduled) > 0 else [],
            name="Newly Unscheduled",
            marker=dict(color="#d62728", line=dict(color="#8b1a1a", width=2)),
            nbinsx=20,
            showlegend=False,
            opacity=1.0,
        ),
        row=1,
        col=2,
    )

    fig.update_xaxes(title_text="Priority", row=1, col=1)
    fig.update_xaxes(title_text="Priority", row=1, col=2)
    fig.update_yaxes(title_text="Count", row=1, col=1)
    fig.update_yaxes(title_text="Count", row=1, col=2)

    fig.update_annotations(font=dict(size=14, color="white"))

    fig.update_layout(
        height=PLOT_HEIGHT - 100,
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    return fig


def create_time_distribution_plot(
    current_scheduled: pd.DataFrame,
    comparison_scheduled: pd.DataFrame,
    current_name: str,
    comparison_name: str,
) -> go.Figure:
    """
    Create box plot comparison of requested time distributions.

    Args:
        current_scheduled: Current schedule's scheduled observations
        comparison_scheduled: Comparison schedule's scheduled observations
        current_name: Name of current schedule
        comparison_name: Name of comparison schedule

    Returns:
        Plotly Figure
    """
    fig = go.Figure()

    # Determine which dataset has fewer items to plot it on top
    current_count = len(current_scheduled)
    comparison_count = len(comparison_scheduled)

    trace_current = go.Box(
        y=current_scheduled["requested_hours"],
        name=current_name,
        marker=dict(color="#1f77b4", line=dict(color="#0d5a9e", width=2)),
        fillcolor="#1f77b4",
        line=dict(color="#0d5a9e", width=2),
        boxmean="sd",
        opacity=1.0,
    )

    trace_comparison = go.Box(
        y=comparison_scheduled["requested_hours"],
        name=comparison_name,
        marker=dict(color="#ff7f0e", line=dict(color="#cc6600", width=2)),
        fillcolor="#ff7f0e",
        line=dict(color="#cc6600", width=2),
        boxmean="sd",
        opacity=1.0,
    )

    # Add larger dataset first, then smaller on top
    if current_count >= comparison_count:
        fig.add_trace(trace_current)
        fig.add_trace(trace_comparison)
    else:
        fig.add_trace(trace_comparison)
        fig.add_trace(trace_current)

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
