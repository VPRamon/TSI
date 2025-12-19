"""Distribution plotting functionality."""

import plotly.graph_objects as go
import streamlit as st

from tsi.config import CACHE_TTL, PLOT_HEIGHT


@st.cache_data(show_spinner=False, ttl=CACHE_TTL)
def build_figures(_distribution_data, priority_bins: int = 20) -> dict[str, go.Figure]:
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


def _build_priority_histogram(blocks, distribution_data, bins: int) -> go.Figure:
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
            marker=dict(color="#ff7f0e", line=dict(color="white", width=1)),
            name="Unscheduled",
            opacity=0.8,
        )
    )

    # Add scheduled observations (top layer)
    fig.add_trace(
        go.Histogram(
            x=scheduled_priorities,
            nbinsx=bins,
            marker=dict(color="#1f77b4", line=dict(color="white", width=1)),
            name="Scheduled",
            opacity=0.8,
        )
    )

    fig.update_layout(
        title=f"Priority Distribution<br><sub>Planificadas: {scheduled_count} de {total_count} ({scheduled_count/total_count*100:.1f}%)</sub>",
        xaxis_title="Priority",
        yaxis_title="Count",
        barmode="stack",  # Stack the bars
        height=PLOT_HEIGHT - 100,
        margin=dict(l=60, r=60, t=80, b=60),
        showlegend=True,
        legend=dict(orientation="h", yanchor="bottom", y=1.02, xanchor="right", x=1),
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    return fig


def _build_visibility_histogram(blocks, distribution_data) -> go.Figure:
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
            marker=dict(color="#ff7f0e", line=dict(color="white", width=1)),
            name="Unscheduled",
            opacity=0.8,
        )
    )

    # Add scheduled observations (top layer)
    fig.add_trace(
        go.Histogram(
            x=scheduled_visibility,
            nbinsx=30,
            marker=dict(color="#1f77b4", line=dict(color="white", width=1)),
            name="Scheduled",
            opacity=0.8,
        )
    )

    fig.update_layout(
        title=f"Total Visibility Hours Distribution<br><sub>Planificadas: {scheduled_count} de {total_count} ({scheduled_count/total_count*100:.1f}%)</sub>",
        xaxis_title="Total Visibility Hours",
        yaxis_title="Count",
        barmode="stack",
        height=PLOT_HEIGHT - 100,
        margin=dict(l=60, r=60, t=80, b=60),
        showlegend=True,
        legend=dict(orientation="h", yanchor="bottom", y=1.02, xanchor="right", x=1),
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    return fig


def _build_duration_histogram(blocks, distribution_data) -> go.Figure:
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
            marker=dict(color="#ff7f0e", line=dict(color="white", width=1)),
            name="Unscheduled",
            opacity=0.8,
        )
    )

    # Add scheduled observations (top layer)
    fig.add_trace(
        go.Histogram(
            x=scheduled_duration,
            nbinsx=25,
            marker=dict(color="#1f77b4", line=dict(color="white", width=1)),
            name="Scheduled",
            opacity=0.8,
        )
    )

    fig.update_layout(
        title=f"Requested Duration Distribution<br><sub>Planificadas: {scheduled_count} de {total_count} ({scheduled_count/total_count*100:.1f}%)</sub>",
        xaxis_title="Requested Hours",
        yaxis_title="Count",
        barmode="stack",
        height=PLOT_HEIGHT - 100,
        margin=dict(l=60, r=60, t=80, b=60),
        showlegend=True,
        legend=dict(orientation="h", yanchor="bottom", y=1.02, xanchor="right", x=1),
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    return fig


def _build_elevation_histogram(blocks, distribution_data) -> go.Figure:
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
            marker=dict(color="#ff7f0e", line=dict(color="white", width=1)),
            name="Unscheduled",
            opacity=0.8,
        )
    )

    # Add scheduled observations (top layer)
    fig.add_trace(
        go.Histogram(
            x=scheduled_elevation,
            nbinsx=20,
            marker=dict(color="#1f77b4", line=dict(color="white", width=1)),
            name="Scheduled",
            opacity=0.8,
        )
    )

    fig.update_layout(
        title=f"Elevation Constraint Range Distribution<br><sub>Planificadas: {scheduled_count} de {total_count} ({scheduled_count/total_count*100:.1f}%)</sub>",
        xaxis_title="Elevation Range (degrees)",
        yaxis_title="Count",
        barmode="stack",
        height=PLOT_HEIGHT - 100,
        margin=dict(l=60, r=60, t=80, b=60),
        showlegend=True,
        legend=dict(orientation="h", yanchor="bottom", y=1.02, xanchor="right", x=1),
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    return fig


def _build_scheduled_bar(distribution_data) -> go.Figure:
    """Build scheduled vs unscheduled bar chart."""
    scheduled_count = distribution_data.scheduled_count
    unscheduled_count = distribution_data.unscheduled_count

    fig = go.Figure()

    fig.add_trace(
        go.Bar(
            x=["Scheduled"],
            y=[scheduled_count],
            name="Scheduled",
            marker=dict(color="#1f77b4"),
            text=[scheduled_count],
            textposition="outside",
        )
    )

    fig.add_trace(
        go.Bar(
            x=["Unscheduled"],
            y=[unscheduled_count],
            name="Unscheduled",
            marker=dict(color="#ff7f0e"),
            text=[unscheduled_count],
            textposition="outside",
        )
    )

    fig.update_layout(
        title="Scheduled vs Unscheduled Observations",
        xaxis_title="Status",
        yaxis_title="Count",
        height=PLOT_HEIGHT - 100,
        margin=dict(l=60, r=60, t=60, b=60),
        showlegend=False,
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    return fig


def _build_priority_comparison(blocks, distribution_data) -> go.Figure:
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
                marker=dict(color="#1f77b4"),
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
                marker=dict(color="#ff7f0e"),
                opacity=0.7,
            )
        )

    fig.update_layout(
        title="Priority Distribution by Scheduled Status",
        yaxis_title="Priority",
        height=PLOT_HEIGHT - 100,
        margin=dict(l=60, r=60, t=60, b=60),
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    return fig


def build_correlation_heatmap(corr_matrix) -> go.Figure:
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
            textfont={"size": 10},
            colorbar=dict(title="Correlation"),
        )
    )

    fig.update_layout(
        title="Spearman Correlation Heatmap",
        height=500,
        margin=dict(l=100, r=60, t=80, b=100),
        xaxis=dict(tickangle=-45),
    )

    return fig
