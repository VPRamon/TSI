"""Timeline (Gantt-style) plotting for visibility and scheduling."""

from datetime import datetime, timedelta

import numpy as np
import pandas as pd
import plotly.graph_objects as go
import streamlit as st

from core.time import format_datetime_utc
from tsi.services.visibility_cache import parse_subset_lazy


@st.cache_data(show_spinner=False, ttl=3600)
def build_timeline(
    df: pd.DataFrame,
    max_blocks: int = 50,
    show_visibility: bool = True,
    show_scheduled: bool = True,
    show_fixed: bool = True,
    date_range: tuple[datetime | None, datetime | None] = (None, None),
) -> go.Figure:
    """
    Build Gantt-style timeline visualization.

    Args:
        df: Prepared DataFrame with parsed time periods
        max_blocks: Maximum number of scheduling blocks to display
        show_visibility: Show visibility windows
        show_scheduled: Show scheduled periods
        show_fixed: Show fixed time constraints
        date_range: Optional (start, end) datetime filter

    Returns:
        Plotly Figure object
    """
    # Limit to top N by priority for performance
    # Use parse_subset_lazy to only parse visibility for the rows we'll display
    top_indices = df.nlargest(max_blocks, "priority").index
    display_df = parse_subset_lazy(df, df.index.isin(top_indices))

    if len(display_df) == 0:
        fig = go.Figure()
        fig.update_layout(title="No data to display")
        return fig

    fig = go.Figure()

    # Track y-axis positions
    y_positions = {}
    y_idx = 0

    for idx, row in display_df.iterrows():
        block_id = row["schedulingBlockId"]
        y_positions[block_id] = y_idx

        # 1. Visibility windows (light blue bars)
        if show_visibility and row["visibility_periods_parsed"]:
            for vis_start, vis_stop in row["visibility_periods_parsed"]:
                # Apply date range filter if specified
                if date_range[0] and vis_stop < date_range[0]:
                    continue
                if date_range[1] and vis_start > date_range[1]:
                    continue

                fig.add_trace(
                    go.Scatter(
                        x=[vis_start, vis_stop, vis_stop, vis_start, vis_start],
                        y=[
                            y_idx - 0.3,
                            y_idx - 0.3,
                            y_idx + 0.3,
                            y_idx + 0.3,
                            y_idx - 0.3,
                        ],
                        fill="toself",
                        fillcolor="rgba(173, 216, 230, 0.4)",
                        line=dict(color="rgba(100, 150, 200, 0.6)", width=1),
                        hovertemplate=(
                            f"<b>Block {block_id}</b><br>"
                            f"Visibility Window<br>"
                            f"Start: {format_datetime_utc(vis_start)}<br>"
                            f"End: {format_datetime_utc(vis_stop)}<br>"
                            "<extra></extra>"
                        ),
                        showlegend=False,
                        name="Visibility",
                    )
                )

        # 2. Fixed time constraints (dashed red boxes)
        if show_fixed and not pd.isna(row["fixed_start_dt"]) and not pd.isna(row["fixed_stop_dt"]):
            fixed_start = row["fixed_start_dt"]
            fixed_stop = row["fixed_stop_dt"]

            if date_range[0] is None or (fixed_stop >= date_range[0]):
                if date_range[1] is None or (fixed_start <= date_range[1]):
                    fig.add_trace(
                        go.Scatter(
                            x=[
                                fixed_start,
                                fixed_stop,
                                fixed_stop,
                                fixed_start,
                                fixed_start,
                            ],
                            y=[
                                y_idx - 0.35,
                                y_idx - 0.35,
                                y_idx + 0.35,
                                y_idx + 0.35,
                                y_idx - 0.35,
                            ],
                            mode="lines",
                            line=dict(color="red", width=2, dash="dash"),
                            hovertemplate=(
                                f"<b>Block {block_id}</b><br>"
                                f"Fixed Constraint<br>"
                                f"Start: {format_datetime_utc(fixed_start)}<br>"
                                f"End: {format_datetime_utc(fixed_stop)}<br>"
                                "<extra></extra>"
                            ),
                            showlegend=False,
                            name="Fixed",
                        )
                    )

        # 3. Scheduled period (solid green bar)
        if show_scheduled and row["scheduled_flag"]:
            sched_start = row["scheduled_start_dt"]
            sched_stop = row["scheduled_stop_dt"]

            if not pd.isna(sched_start) and not pd.isna(sched_stop):
                if date_range[0] is None or (sched_stop >= date_range[0]):
                    if date_range[1] is None or (sched_start <= date_range[1]):
                        fig.add_trace(
                            go.Scatter(
                                x=[
                                    sched_start,
                                    sched_stop,
                                    sched_stop,
                                    sched_start,
                                    sched_start,
                                ],
                                y=[
                                    y_idx - 0.25,
                                    y_idx - 0.25,
                                    y_idx + 0.25,
                                    y_idx + 0.25,
                                    y_idx - 0.25,
                                ],
                                fill="toself",
                                fillcolor="rgba(50, 200, 50, 0.7)",
                                line=dict(color="green", width=2),
                                hovertemplate=(
                                    f"<b>Block {block_id}</b><br>"
                                    f"Scheduled Period<br>"
                                    f"Start: {format_datetime_utc(sched_start)}<br>"
                                    f"End: {format_datetime_utc(sched_stop)}<br>"
                                    f"Priority: {row['priority']:.1f}<br>"
                                    "<extra></extra>"
                                ),
                                showlegend=False,
                                name="Scheduled",
                            )
                        )

        y_idx += 1

    # Create legend entries
    if show_visibility:
        fig.add_trace(
            go.Scatter(
                x=[None],
                y=[None],
                mode="markers",
                marker=dict(size=10, color="rgba(173, 216, 230, 0.6)"),
                name="Visibility Windows",
            )
        )

    if show_fixed:
        fig.add_trace(
            go.Scatter(
                x=[None],
                y=[None],
                mode="lines",
                line=dict(color="red", width=2, dash="dash"),
                name="Fixed Constraints",
            )
        )

    if show_scheduled:
        fig.add_trace(
            go.Scatter(
                x=[None],
                y=[None],
                mode="markers",
                marker=dict(size=10, color="rgba(50, 200, 50, 0.7)"),
                name="Scheduled Periods",
            )
        )

    # Update layout
    y_labels = [f"Block {bid}" for bid in y_positions.keys()]

    fig.update_layout(
        title=f"Visibility & Schedule Timeline (Top {len(display_df)} by Priority)",
        xaxis=dict(
            title="Time (UTC)",
            showgrid=True,
            gridcolor="rgba(100, 100, 100, 0.3)",
        ),
        yaxis=dict(
            title="Scheduling Block",
            tickmode="array",
            tickvals=list(range(len(y_labels))),
            ticktext=y_labels,
            showgrid=True,
            gridcolor="rgba(100, 100, 100, 0.3)",
        ),
        height=max(600, len(display_df) * 40),
        margin=dict(l=120, r=80, t=80, b=80),
        hovermode="closest",
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
        legend=dict(
            orientation="h",
            yanchor="bottom",
            y=1.02,
            xanchor="center",
            x=0.5,
        ),
    )

    return fig


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
    
    # Vectorized approach: convert periods to arrays for faster processing
    # Build arrays of all period starts and stops
    period_starts = np.array([pd.Timestamp(start).value for start, _ in all_periods], dtype=np.int64)
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
        
        # Count unique rows (not periods) - track which row each period belongs to
        # We need to build a row index array
        row_indices = []
        for row_idx, periods in enumerate(df_with_vis["visibility_periods_parsed"]):
            if periods:
                row_indices.extend([row_idx] * len(periods))
        
        row_indices = np.array(row_indices, dtype=np.int32)
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
