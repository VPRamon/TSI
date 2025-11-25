"""Timeline (Gantt-style) plotting for visibility and scheduling."""

from datetime import timedelta

import numpy as np
import pandas as pd
import plotly.graph_objects as go
import streamlit as st


@st.cache_data(show_spinner=False, ttl=3600)
def build_visibility_histogram(
    df: pd.DataFrame,
    num_bins: int | None = 50,
    bin_duration_minutes: float | None = None,
) -> go.Figure:
    """
    Build histogram showing number of visible targets over time.

    Args:
        df: Prepared DataFrame with parsed visibility periods
        num_bins: Number of time bins for the histogram (ignored if bin_duration_minutes is set)
        bin_duration_minutes: Optional duration for each bin (in minutes)

    Returns:
        Plotly Figure object with histogram
    """
    if len(df) == 0:
        fig = go.Figure()
        fig.update_layout(title="No data to display")
        return fig

    # Ensure visibility is parsed (with caching) - ONLY for rows in df
    from tsi.services.visibility_cache import ensure_visibility_parsed

    df_with_vis = ensure_visibility_parsed(df)

    # Collect all visibility periods
    all_periods = []
    for periods in df_with_vis["visibility_periods_parsed"]:
        if periods:
            all_periods.extend(periods)

    if not all_periods:
        fig = go.Figure()
        fig.update_layout(title="No visibility periods found in data")
        return fig

    # Find overall time range
    min_time = min(start for start, _ in all_periods)
    max_time = max(stop for _, stop in all_periods)

    # Handle zero-length ranges by padding one minute
    time_range = max_time - min_time
    if time_range == timedelta(0):
        time_range = timedelta(minutes=1)
        max_time = min_time + time_range

    # Determine bin duration/quantity
    if bin_duration_minutes is not None and bin_duration_minutes > 0:
        bin_duration = timedelta(minutes=bin_duration_minutes)
        num_bins = max(1, int(np.ceil(time_range / bin_duration)))
    else:
        if not num_bins:
            num_bins = 50
        num_bins = max(1, int(num_bins))
        bin_duration = time_range / num_bins

    # Initialize bins
    bin_edges = [min_time + i * bin_duration for i in range(num_bins + 1)]

    # Build a mapping of period index to row index
    # This ensures we count each block only once per bin, even if it has multiple visibility periods
    row_indices_list: list[int] = []
    for row_idx, periods in enumerate(df_with_vis["visibility_periods_parsed"]):
        if periods:
            row_indices_list.extend([row_idx] * len(periods))

    row_indices = np.array(row_indices_list, dtype=np.int32)

    # Vectorized approach: convert periods to arrays for faster processing
    # Build arrays of all period starts and stops
    period_starts = np.array(
        [pd.Timestamp(start).value for start, _ in all_periods], dtype=np.int64
    )
    period_stops = np.array([pd.Timestamp(stop).value for _, stop in all_periods], dtype=np.int64)

    # Convert bin edges to numpy array
    bin_edge_values = np.array([pd.Timestamp(edge).value for edge in bin_edges], dtype=np.int64)

    # Count overlaps using vectorized operations
    bin_counts = []
    for i in range(num_bins):
        bin_start = bin_edge_values[i]
        bin_end = bin_edge_values[i + 1]

        # A period overlaps if: period_start < bin_end AND period_stop > bin_start
        overlaps = (period_starts < bin_end) & (period_stops > bin_start)

        # Count unique blocks (rows) that have at least one overlapping period in this bin
        overlapping_rows = np.unique(row_indices[overlaps])
        bin_counts.append(len(overlapping_rows))

    # Calculate bin centers correctly (timedelta arithmetic)
    bin_centers = [bin_edges[i] + (bin_edges[i + 1] - bin_edges[i]) / 2 for i in range(num_bins)]

    # Create the histogram
    fig = go.Figure()

    fig.add_trace(
        go.Bar(
            x=bin_centers,
            y=bin_counts,
            name="Visible Targets",
            marker=dict(
                color=bin_counts,
                colorscale="Viridis",
                colorbar=dict(title="Number of<br>Visible Blocks"),
                line=dict(color="rgba(255, 255, 255, 0.3)", width=0.5),
            ),
            hovertemplate=(
                "<b>%{y} visible blocks</b><br>" "Time: %{x|%Y-%m-%d %H:%M}<br>" "<extra></extra>"
            ),
        )
    )

    # Update layout
    # Human-readable bin duration for title
    duration_minutes = bin_duration.total_seconds() / 60
    if duration_minutes >= 24 * 60:
        duration_label = f"{duration_minutes / (24 * 60):.1f} day(s)"
    elif duration_minutes >= 60:
        duration_label = f"{duration_minutes / 60:.1f} hour(s)"
    else:
        duration_label = f"{duration_minutes:.1f} minute(s)"

    fig.update_layout(
        title=(
            "Target Visibility Over Time "
            f"({len(df):,} total blocks, {num_bins} bins, ~{duration_label} per bin)"
        ),
        xaxis=dict(
            title="Observation Period (UTC)",
            showgrid=True,
            gridcolor="rgba(100, 100, 100, 0.3)",
        ),
        yaxis=dict(
            title="Number of Visible Blocks",
            showgrid=True,
            gridcolor="rgba(100, 100, 100, 0.3)",
        ),
        height=600,
        margin=dict(l=80, r=80, t=100, b=80),
        hovermode="x unified",
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
        showlegend=False,
    )

    return fig
