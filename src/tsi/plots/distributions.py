"""Distribution plotting functionality."""

import pandas as pd
import plotly.graph_objects as go

from tsi.config import PLOT_HEIGHT


def build_figures(df: pd.DataFrame, priority_bins: int = 20) -> dict[str, go.Figure]:
    """
    Build multiple distribution plots.

    Args:
        df: Prepared DataFrame
        priority_bins: Number of bins for priority histogram

    Returns:
        Dictionary mapping plot name to Figure object
    """
    figures = {}

    # 1. Priority distribution
    figures["priority_hist"] = _build_priority_histogram(df, priority_bins)

    # 2. Visibility hours distribution
    figures["visibility_hist"] = _build_visibility_histogram(df)

    # 3. Requested duration distribution
    figures["duration_hist"] = _build_duration_histogram(df)

    # 4. Elevation range distribution
    figures["elevation_hist"] = _build_elevation_histogram(df)

    # 5. Scheduled vs Unscheduled counts
    figures["scheduled_bar"] = _build_scheduled_bar(df)

    # 6. Priority by scheduled status (violin/box)
    figures["priority_violin"] = _build_priority_comparison(df)

    return figures


def _build_priority_histogram(df: pd.DataFrame, bins: int) -> go.Figure:
    """Build priority distribution histogram with scheduled/unscheduled breakdown."""
    # Calculate scheduled observations
    scheduled_count = df["scheduled_flag"].sum()
    total_count = len(df)

    # Separate data by scheduled status
    scheduled_df = df[df["scheduled_flag"]]
    unscheduled_df = df[~df["scheduled_flag"]]

    fig = go.Figure()

    # Add unscheduled observations (bottom layer)
    fig.add_trace(
        go.Histogram(
            x=unscheduled_df["priority"],
            nbinsx=bins,
            marker=dict(color="#ff7f0e", line=dict(color="white", width=1)),
            name="Unscheduled",
            opacity=0.8,
        )
    )

    # Add scheduled observations (top layer)
    fig.add_trace(
        go.Histogram(
            x=scheduled_df["priority"],
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


def _build_visibility_histogram(df: pd.DataFrame) -> go.Figure:
    """Build total visibility hours distribution with scheduled/unscheduled breakdown."""
    # Calculate scheduled observations
    scheduled_count = df["scheduled_flag"].sum()
    total_count = len(df)

    # Separate data by scheduled status
    scheduled_df = df[df["scheduled_flag"]]
    unscheduled_df = df[~df["scheduled_flag"]]

    fig = go.Figure()

    # Add unscheduled observations (bottom layer)
    fig.add_trace(
        go.Histogram(
            x=unscheduled_df["total_visibility_hours"],
            nbinsx=30,
            marker=dict(color="#ff7f0e", line=dict(color="white", width=1)),
            name="Unscheduled",
            opacity=0.8,
        )
    )

    # Add scheduled observations (top layer)
    fig.add_trace(
        go.Histogram(
            x=scheduled_df["total_visibility_hours"],
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


def _build_duration_histogram(df: pd.DataFrame) -> go.Figure:
    """Build requested duration distribution with scheduled/unscheduled breakdown."""
    # Calculate scheduled observations
    scheduled_count = df["scheduled_flag"].sum()
    total_count = len(df)

    # Separate data by scheduled status
    scheduled_df = df[df["scheduled_flag"]]
    unscheduled_df = df[~df["scheduled_flag"]]

    fig = go.Figure()

    # Add unscheduled observations (bottom layer)
    fig.add_trace(
        go.Histogram(
            x=unscheduled_df["requested_hours"],
            nbinsx=25,
            marker=dict(color="#ff7f0e", line=dict(color="white", width=1)),
            name="Unscheduled",
            opacity=0.8,
        )
    )

    # Add scheduled observations (top layer)
    fig.add_trace(
        go.Histogram(
            x=scheduled_df["requested_hours"],
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


def _build_elevation_histogram(df: pd.DataFrame) -> go.Figure:
    """Build elevation constraint range distribution with scheduled/unscheduled breakdown."""
    # Calculate scheduled observations
    scheduled_count = df["scheduled_flag"].sum()
    total_count = len(df)

    # Separate data by scheduled status
    scheduled_df = df[df["scheduled_flag"]]
    unscheduled_df = df[~df["scheduled_flag"]]

    fig = go.Figure()

    # Add unscheduled observations (bottom layer)
    fig.add_trace(
        go.Histogram(
            x=unscheduled_df["elevation_range_deg"],
            nbinsx=20,
            marker=dict(color="#ff7f0e", line=dict(color="white", width=1)),
            name="Unscheduled",
            opacity=0.8,
        )
    )

    # Add scheduled observations (top layer)
    fig.add_trace(
        go.Histogram(
            x=scheduled_df["elevation_range_deg"],
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


def _build_scheduled_bar(df: pd.DataFrame) -> go.Figure:
    """Build scheduled vs unscheduled bar chart."""
    scheduled_counts = df["scheduled_flag"].value_counts()

    fig = go.Figure()

    colors = {True: "#1f77b4", False: "#ff7f0e"}
    labels = {True: "Scheduled", False: "Unscheduled"}

    for status in [True, False]:
        if status in scheduled_counts.index:
            fig.add_trace(
                go.Bar(
                    x=[labels[status]],
                    y=[scheduled_counts[status]],
                    name=labels[status],
                    marker=dict(color=colors[status]),
                    text=[scheduled_counts[status]],
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


def _build_priority_comparison(df: pd.DataFrame) -> go.Figure:
    """Build violin/box plot comparing priority by scheduled status."""
    fig = go.Figure()

    scheduled_df = df[df["scheduled_flag"]]
    unscheduled_df = df[~df["scheduled_flag"]]

    # Add violin plots
    if len(scheduled_df) > 0:
        fig.add_trace(
            go.Violin(
                y=scheduled_df["priority"],
                name="Scheduled",
                box_visible=True,
                meanline_visible=True,
                marker=dict(color="#1f77b4"),
                opacity=0.7,
            )
        )

    if len(unscheduled_df) > 0:
        fig.add_trace(
            go.Violin(
                y=unscheduled_df["priority"],
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


def build_correlation_heatmap(corr_matrix: pd.DataFrame) -> go.Figure:
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
