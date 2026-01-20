"""Distribution plotting functionality."""

from typing import Any

import plotly.graph_objects as go
import streamlit as st

from tsi.config import CACHE_TTL, PLOT_HEIGHT
from tsi.plots.plot_theme import (
    SCHEDULED_COLOR,
    UNSCHEDULED_COLOR,
    PlotTheme,
    apply_theme,
    get_histogram_marker,
)


@st.cache_data(show_spinner=False, ttl=CACHE_TTL)
def build_figures(_distribution_data: Any, priority_bins: int = 20) -> dict[str, go.Figure]:
    """
    Build multiple distribution plots from DistributionData.

    Args:
        _distribution_data: DistributionData object from Rust backend (unhashed by Streamlit)
        priority_bins: Number of bins for priority histogram

    Returns:
        Dictionary mapping plot name to Figure object
    """
    figures = {}

    # Extract blocks from the distribution data
    blocks = _distribution_data.blocks

    # 1. Priority distribution
    figures["priority_hist"] = _build_priority_histogram(blocks, _distribution_data, priority_bins)

    # 2. Visibility hours distribution
    figures["visibility_hist"] = _build_visibility_histogram(blocks, _distribution_data)

    # 3. Requested duration distribution
    figures["duration_hist"] = _build_duration_histogram(blocks, _distribution_data)

    # 4. Elevation range distribution
    figures["elevation_hist"] = _build_elevation_histogram(blocks, _distribution_data)

    # 5. Scheduled vs Unscheduled counts
    figures["scheduled_bar"] = _build_scheduled_bar(_distribution_data)

    # 6. Priority by scheduled status (violin/box)
    figures["priority_violin"] = _build_priority_comparison(blocks, _distribution_data)

    return figures


def _build_priority_histogram(blocks: Any, distribution_data: Any, bins: int) -> go.Figure:
    """Build priority distribution histogram with scheduled/unscheduled breakdown."""
    scheduled_count = distribution_data.scheduled_count
    total_count = distribution_data.total_count

    # Separate data by scheduled status
    scheduled_priorities = [b.priority for b in blocks if b.scheduled]
    unscheduled_priorities = [b.priority for b in blocks if not b.scheduled]

    fig = go.Figure()

    # Add unscheduled observations (bottom layer)
    fig.add_trace(
        go.Histogram(
            x=unscheduled_priorities,
            nbinsx=bins,
            **get_histogram_marker(color=UNSCHEDULED_COLOR),
            name="Unscheduled",
        )
    )

    # Add scheduled observations (top layer)
    fig.add_trace(
        go.Histogram(
            x=scheduled_priorities,
            nbinsx=bins,
            **get_histogram_marker(color=SCHEDULED_COLOR),
            name="Scheduled",
        )
    )

    # Apply standard theme
    apply_theme(fig, height=PLOT_HEIGHT - 100, legend_style="horizontal")

    fig.update_layout(
        title=f"Priority Distribution<br><sub>Scheduled: {scheduled_count} of {total_count} ({scheduled_count/total_count*100:.1f}%)</sub>",
        xaxis_title="Priority",
        yaxis_title="Count",
        barmode="stack",
    )

    # Update axes with grid styling
    fig.update_xaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)
    fig.update_yaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)

    return fig


def _build_visibility_histogram(blocks: Any, distribution_data: Any) -> go.Figure:
    """Build total visibility hours distribution with scheduled/unscheduled breakdown."""
    scheduled_count = distribution_data.scheduled_count
    total_count = distribution_data.total_count

    # Separate data by scheduled status
    scheduled_visibility = [b.total_visibility_hours for b in blocks if b.scheduled]
    unscheduled_visibility = [b.total_visibility_hours for b in blocks if not b.scheduled]

    fig = go.Figure()

    # Add unscheduled observations (bottom layer)
    fig.add_trace(
        go.Histogram(
            x=unscheduled_visibility,
            nbinsx=30,
            **get_histogram_marker(color=UNSCHEDULED_COLOR),
            name="Unscheduled",
        )
    )

    # Add scheduled observations (top layer)
    fig.add_trace(
        go.Histogram(
            x=scheduled_visibility,
            nbinsx=30,
            **get_histogram_marker(color=SCHEDULED_COLOR),
            name="Scheduled",
        )
    )

    # Apply standard theme
    apply_theme(fig, height=PLOT_HEIGHT - 100, legend_style="horizontal")

    fig.update_layout(
        title=f"Total Visibility Hours Distribution<br><sub>Scheduled: {scheduled_count} of {total_count} ({scheduled_count/total_count*100:.1f}%)</sub>",
        xaxis_title="Total Visibility Hours",
        yaxis_title="Count",
        barmode="stack",
    )

    # Update axes with grid styling
    fig.update_xaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)
    fig.update_yaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)

    return fig


def _build_duration_histogram(blocks: Any, distribution_data: Any) -> go.Figure:
    """Build requested duration distribution with scheduled/unscheduled breakdown."""
    scheduled_count = distribution_data.scheduled_count
    total_count = distribution_data.total_count

    # Separate data by scheduled status
    scheduled_duration = [b.requested_hours for b in blocks if b.scheduled]
    unscheduled_duration = [b.requested_hours for b in blocks if not b.scheduled]

    fig = go.Figure()

    # Add unscheduled observations (bottom layer)
    fig.add_trace(
        go.Histogram(
            x=unscheduled_duration,
            nbinsx=25,
            **get_histogram_marker(color=UNSCHEDULED_COLOR),
            name="Unscheduled",
        )
    )

    # Add scheduled observations (top layer)
    fig.add_trace(
        go.Histogram(
            x=scheduled_duration,
            nbinsx=25,
            **get_histogram_marker(color=SCHEDULED_COLOR),
            name="Scheduled",
        )
    )

    # Apply standard theme
    apply_theme(fig, height=PLOT_HEIGHT - 100, legend_style="horizontal")

    fig.update_layout(
        title=f"Requested Duration Distribution<br><sub>Scheduled: {scheduled_count} of {total_count} ({scheduled_count/total_count*100:.1f}%)</sub>",
        xaxis_title="Requested Hours",
        yaxis_title="Count",
        barmode="stack",
    )

    # Update axes with grid styling
    fig.update_xaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)
    fig.update_yaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)

    return fig


def _build_elevation_histogram(blocks: Any, distribution_data: Any) -> go.Figure:
    """Build elevation constraint range distribution with scheduled/unscheduled breakdown."""
    scheduled_count = distribution_data.scheduled_count
    total_count = distribution_data.total_count

    # Separate data by scheduled status
    scheduled_elevation = [b.elevation_range_deg for b in blocks if b.scheduled]
    unscheduled_elevation = [b.elevation_range_deg for b in blocks if not b.scheduled]

    fig = go.Figure()

    # Add unscheduled observations (bottom layer)
    fig.add_trace(
        go.Histogram(
            x=unscheduled_elevation,
            nbinsx=20,
            **get_histogram_marker(color=UNSCHEDULED_COLOR),
            name="Unscheduled",
        )
    )

    # Add scheduled observations (top layer)
    fig.add_trace(
        go.Histogram(
            x=scheduled_elevation,
            nbinsx=20,
            **get_histogram_marker(color=SCHEDULED_COLOR),
            name="Scheduled",
        )
    )

    # Apply standard theme
    apply_theme(fig, height=PLOT_HEIGHT - 100, legend_style="horizontal")

    fig.update_layout(
        title=f"Elevation Constraint Range Distribution<br><sub>Scheduled: {scheduled_count} of {total_count} ({scheduled_count/total_count*100:.1f}%)</sub>",
        xaxis_title="Elevation Range (degrees)",
        yaxis_title="Count",
        barmode="stack",
    )

    # Update axes with grid styling
    fig.update_xaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)
    fig.update_yaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)

    return fig


def _build_scheduled_bar(distribution_data: Any) -> go.Figure:
    """Build scheduled vs unscheduled bar chart."""
    scheduled_count = distribution_data.scheduled_count
    unscheduled_count = distribution_data.unscheduled_count

    fig = go.Figure()

    fig.add_trace(
        go.Bar(
            x=["Scheduled"],
            y=[scheduled_count],
            name="Scheduled",
            marker=dict(color=SCHEDULED_COLOR),
            text=[scheduled_count],
            textposition="outside",
            textfont=dict(color=PlotTheme.TEXT_COLOR),
        )
    )

    fig.add_trace(
        go.Bar(
            x=["Unscheduled"],
            y=[unscheduled_count],
            name="Unscheduled",
            marker=dict(color=UNSCHEDULED_COLOR),
            text=[unscheduled_count],
            textposition="outside",
            textfont=dict(color=PlotTheme.TEXT_COLOR),
        )
    )

    # Apply standard theme (no legend for this simple chart)
    apply_theme(fig, height=PLOT_HEIGHT - 100, show_legend=False)

    fig.update_layout(
        title="Scheduled vs Unscheduled Observations",
        xaxis_title="Status",
        yaxis_title="Count",
    )

    # Update axes with grid styling
    fig.update_xaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)
    fig.update_yaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)

    return fig


def _build_priority_comparison(blocks: Any, distribution_data: Any) -> go.Figure:
    """Build violin/box plot comparing priority by scheduled status."""
    fig = go.Figure()

    scheduled_priorities = [b.priority for b in blocks if b.scheduled]
    unscheduled_priorities = [b.priority for b in blocks if not b.scheduled]

    # Add violin plots
    if len(scheduled_priorities) > 0:
        fig.add_trace(
            go.Violin(
                y=scheduled_priorities,
                name="Scheduled",
                box_visible=True,
                meanline_visible=True,
                marker=dict(color=SCHEDULED_COLOR),
                opacity=0.7,
            )
        )

    if len(unscheduled_priorities) > 0:
        fig.add_trace(
            go.Violin(
                y=unscheduled_priorities,
                name="Unscheduled",
                box_visible=True,
                meanline_visible=True,
                marker=dict(color=UNSCHEDULED_COLOR),
                opacity=0.7,
            )
        )

    # Apply standard theme
    apply_theme(fig, height=PLOT_HEIGHT - 100, legend_style="horizontal")

    fig.update_layout(
        title="Priority Distribution by Scheduled Status",
        yaxis_title="Priority",
    )

    # Update axes with grid styling
    fig.update_xaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)
    fig.update_yaxes(showgrid=True, gridcolor=PlotTheme.GRID_COLOR)

    return fig


def build_correlation_heatmap(corr_matrix: Any) -> go.Figure:
    """
    Build correlation heatmap.

    Args:
        corr_matrix: Correlation matrix DataFrame

    Returns:
        Plotly Figure
    """
    if corr_matrix.empty:
        fig = go.Figure()
        fig.update_layout(title="Insufficient data for correlation analysis")
        return fig

    fig = go.Figure(
        data=go.Heatmap(
            z=corr_matrix.values,
            x=corr_matrix.columns,
            y=corr_matrix.index,
            colorscale="RdBu",
            zmid=0,
            text=corr_matrix.values,
            texttemplate="%{text:.2f}",
            textfont={"size": 10, "color": PlotTheme.TEXT_COLOR},
            colorbar=dict(
                title=dict(text="Correlation", font=dict(color=PlotTheme.TEXT_COLOR)),
                tickfont=dict(color=PlotTheme.TEXT_COLOR),
            ),
        )
    )

    # Apply standard theme with custom margin for heatmap
    apply_theme(
        fig,
        height=500,
        margin=dict(l=100, r=60, t=80, b=100),
        show_legend=False,
    )

    fig.update_layout(
        title="Spearman Correlation Heatmap",
        xaxis=dict(tickangle=-45),
    )

    return fig
