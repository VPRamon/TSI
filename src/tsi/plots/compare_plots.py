"""Compare schedules page plotting functions."""

from __future__ import annotations

import plotly.graph_objects as go

from tsi.config import PLOT_HEIGHT
from tsi.plots.plot_theme import (
    COMPARISON_SCHEDULE_COLOR,
    CURRENT_SCHEDULE_COLOR,
    NEGATIVE_CHANGE_COLOR,
    POSITIVE_CHANGE_COLOR,
    PlotTheme,
    apply_theme,
    get_bar_marker,
    get_box_marker,
    get_histogram_marker,
)


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
        **get_histogram_marker(color=CURRENT_SCHEDULE_COLOR, opacity=1.0),
        nbinsx=30,
    )

    trace_comparison = go.Histogram(
        x=comparison_priorities,
        name=comparison_name,
        **get_histogram_marker(color=COMPARISON_SCHEDULE_COLOR, opacity=1.0),
        nbinsx=30,
    )

    # Add larger dataset first, then smaller on top
    if current_count >= comparison_count:
        fig.add_trace(trace_current)
        fig.add_trace(trace_comparison)
    else:
        fig.add_trace(trace_comparison)
        fig.add_trace(trace_current)

    # Apply standard theme
    apply_theme(fig, height=PLOT_HEIGHT - 150, legend_style="horizontal")

    fig.update_layout(
        barmode="overlay",
        xaxis_title="Priority",
        yaxis_title="Count",
    )

    # Update axes with grid styling
    fig.update_xaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)
    fig.update_yaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)

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

    current_marker = get_bar_marker(color=CURRENT_SCHEDULE_COLOR)
    current_marker["pattern"] = dict(
        shape="/",
        bgcolor=PlotTheme.PRIMARY_BLUE_DARK,
        fgcolor=CURRENT_SCHEDULE_COLOR,
        solidity=0.3,
    )

    fig.add_trace(
        go.Bar(
            name=current_name,
            x=["Scheduled", "Unscheduled"],
            y=[current_scheduled_count, current_unscheduled_count],
            marker=current_marker,
            text=[f"{current_scheduled_count:,}", f"{current_unscheduled_count:,}"],
            textposition="auto",
            textfont=dict(
                color=PlotTheme.TEXT_COLOR,
                size=PlotTheme.FONT_SIZE,
                family=PlotTheme.FONT_FAMILY,
            ),
            opacity=1.0,
        )
    )

    comparison_marker = get_bar_marker(color=COMPARISON_SCHEDULE_COLOR)
    comparison_marker["pattern"] = dict(
        shape="\\",
        bgcolor=PlotTheme.PRIMARY_ORANGE_DARK,
        fgcolor=COMPARISON_SCHEDULE_COLOR,
        solidity=0.3,
    )

    fig.add_trace(
        go.Bar(
            name=comparison_name,
            x=["Scheduled", "Unscheduled"],
            y=[comp_scheduled_count, comp_unscheduled_count],
            marker=comparison_marker,
            text=[f"{comp_scheduled_count:,}", f"{comp_unscheduled_count:,}"],
            textposition="auto",
            textfont=dict(
                color=PlotTheme.TEXT_COLOR,
                size=PlotTheme.FONT_SIZE,
                family=PlotTheme.FONT_FAMILY,
            ),
            opacity=1.0,
        )
    )

    # Apply standard theme
    apply_theme(fig, height=PLOT_HEIGHT - 150, legend_style="horizontal")

    fig.update_layout(
        barmode="group",
        yaxis_title="Number of Blocks",
    )

    # Update axes with grid styling
    fig.update_xaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)
    fig.update_yaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)

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
                color=[POSITIVE_CHANGE_COLOR, NEGATIVE_CHANGE_COLOR],
                line=dict(
                    color=[PlotTheme.SUCCESS_GREEN_DARK, PlotTheme.ERROR_RED_DARK],
                    width=2,
                ),
            ),
            text=[f"{newly_scheduled_count:,}", f"{newly_unscheduled_count:,}"],
            textposition="auto",
            textfont=dict(
                color=PlotTheme.TEXT_COLOR,
                size=14,
                family=PlotTheme.FONT_FAMILY,
            ),
            opacity=1.0,
        )
    )

    # Apply standard theme (no legend for this chart)
    apply_theme(fig, height=PLOT_HEIGHT - 200, show_legend=False)

    fig.update_layout(
        yaxis_title="Number of Blocks",
    )

    # Update axes with grid styling
    fig.update_xaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)
    fig.update_yaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)

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

    Returns:
        Plotly Figure
    """
    fig = go.Figure()

    current_box_style = get_box_marker(color=CURRENT_SCHEDULE_COLOR)
    comparison_box_style = get_box_marker(color=COMPARISON_SCHEDULE_COLOR)

    trace_current = go.Box(
        y=current_times,
        name=current_name,
        marker=current_box_style["marker"],
        fillcolor=current_box_style["fillcolor"],
        line=current_box_style["line"],
        boxmean="sd",
        opacity=1.0,
    )

    trace_comparison = go.Box(
        y=comparison_times,
        name=comparison_name,
        marker=comparison_box_style["marker"],
        fillcolor=comparison_box_style["fillcolor"],
        line=comparison_box_style["line"],
        boxmean="sd",
        opacity=1.0,
    )

    fig.add_trace(trace_current)
    fig.add_trace(trace_comparison)

    # Apply standard theme
    apply_theme(fig, height=PLOT_HEIGHT - 200, legend_style="horizontal")

    fig.update_layout(
        yaxis_title="Requested Hours",
    )

    # Update axes with grid styling
    fig.update_xaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)
    fig.update_yaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)

    return fig
