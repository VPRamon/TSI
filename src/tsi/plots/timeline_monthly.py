"""Scheduled timeline plotting functions."""

from __future__ import annotations

from typing import cast

import pandas as pd
import plotly.graph_objects as go

from tsi.services.rust_compat import format_datetime_utc_rust as format_datetime_utc


def build_monthly_timeline(
    df: pd.DataFrame,
    priority_range: tuple[float, float],
    dark_periods: pd.DataFrame | None = None,
) -> go.Figure:
    """
    Build a Plotly figure showing scheduled observations grouped by month.

    Each month is represented as a horizontal row, with observations displayed
    as bars within that month's time range (days 1-31). This prevents overflow
    by constraining each row to its month's temporal boundaries.

    Args:
        df: DataFrame with scheduled observations and monthly grouping columns
        priority_range: (min, max) priority values for color normalization
        dark_periods: Optional DataFrame with dark period windows to overlay

    Returns:
        Plotly Figure object
    """
    # Get sorted months for Y-axis ordering
    scheduled_months = set(df["scheduled_month_label"].unique())
    dark_months: set[str] = set()
    if dark_periods is not None and "months" in dark_periods.columns:
        for month_list in dark_periods["months"]:
            if isinstance(month_list, list):
                dark_months.update(month_list)

    ordered_months = sorted(scheduled_months | dark_months)
    num_months = len(ordered_months)

    # Create figure
    fig = go.Figure()

    # Normalize priority for color mapping
    priority_min, priority_max = priority_range
    if priority_max <= priority_min:
        priority_max = priority_min + 1.0

    # Create a mapping from month_label to numeric position
    month_to_position = {month: idx for idx, month in enumerate(ordered_months)}

    # Add both dark and light period overlays
    if dark_periods is not None and not dark_periods.empty:
        # Add light periods (inverted from dark periods)
        light_segments = _invert_to_light_periods(dark_periods)
        _add_light_period_traces(fig, light_segments, month_to_position)

        # Add dark periods (observable times)
        dark_segments = _split_dark_periods_by_month(dark_periods)
        _add_dark_period_traces(fig, dark_segments, month_to_position)

    # Add observation traces
    _add_observation_traces(fig, df, month_to_position, priority_min, priority_max)

    # Add colorbar legend
    _add_colorbar(fig, priority_min, priority_max)

    # Calculate dynamic height based on number of months
    height = max(600, num_months * 80)

    # Update layout
    fig.update_layout(
        title=f"Scheduled Timeline by Month ({len(df):,} observations)",
        xaxis=dict(
            title="Day of month",
            showgrid=True,
            gridcolor="rgba(100, 100, 100, 0.3)",
            range=[0.5, 31.5],
            tickmode="linear",
            tick0=1,
            dtick=1,
            tickformat="d",
            rangemode="normal",
            autorange=False,
            minallowed=0.5,
            maxallowed=31.5,
        ),
        yaxis=dict(
            title="Scheduled month",
            tickmode="array",
            tickvals=list(range(num_months)),
            ticktext=ordered_months,
            showgrid=True,
            gridcolor="rgba(100, 100, 100, 0.3)",
            range=[-0.5, num_months - 0.5],
        ),
        height=height,
        margin=dict(l=100, r=120, t=80, b=80),
        hovermode="closest",
        dragmode="pan",
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    # Enforce absolute min/max boundaries
    fig.update_xaxes(
        range=[0.5, 31.5],
        minallowed=0.5,
        maxallowed=31.5,
    )

    fig.update_yaxes(fixedrange=False)

    return fig


def _add_light_period_traces(
    fig: go.Figure,
    light_segments: list[tuple[str, pd.Timestamp, pd.Timestamp]],
    month_to_position: dict,
) -> None:
    """Add light (daytime) period traces to figure."""
    light_traces_added = 0
    for month_label, seg_start, seg_stop in light_segments:
        if month_label not in month_to_position:
            continue

        y_pos = month_to_position[month_label]
        y_bottom = y_pos - 0.04
        y_top = y_pos + 0.04

        x_start = _datetime_to_day_fraction(seg_start)
        x_stop = _datetime_to_day_fraction(seg_stop)
        if x_stop <= x_start:
            continue

        duration_hours = (seg_stop - seg_start).total_seconds() / 3600.0

        fig.add_trace(
            go.Scatter(
                name="Daytime periods (non-observable)",
                x=[x_start, x_stop, x_stop, x_start, x_start],
                y=[y_bottom, y_bottom, y_top, y_top, y_bottom],
                fill="toself",
                fillcolor="rgba(255, 230, 180, 0.15)",
                line=dict(color="rgba(200, 180, 140, 0.3)", width=1.5),
                hovertemplate=(
                    "<b>☀️ Daytime period (non-observable)</b><br><br>"
                    f"<b>Month:</b> {month_label}<br>"
                    f"<b>Start:</b> {format_datetime_utc(seg_start)}<br>"
                    f"<b>End:</b> {format_datetime_utc(seg_stop)}<br>"
                    f"<b>Duration:</b> {duration_hours:.2f} hours"
                    "<extra></extra>"
                ),
                legendgroup="light_periods",
                showlegend=light_traces_added == 0,
                mode="lines",
            )
        )
        light_traces_added += 1


def _add_dark_period_traces(
    fig: go.Figure,
    dark_segments: list[tuple[str, pd.Timestamp, pd.Timestamp]],
    month_to_position: dict,
) -> None:
    """Add dark (nighttime) period traces to figure."""
    dark_traces_added = 0
    for month_label, seg_start, seg_stop in dark_segments:
        if month_label not in month_to_position:
            continue

        y_pos = month_to_position[month_label]
        y_bottom = y_pos - 0.04
        y_top = y_pos + 0.04

        x_start = _datetime_to_day_fraction(seg_start)
        x_stop = _datetime_to_day_fraction(seg_stop)
        if x_stop <= x_start:
            continue

        duration_hours = (seg_stop - seg_start).total_seconds() / 3600.0

        fig.add_trace(
            go.Scatter(
                name="Nighttime periods (observable)",
                x=[x_start, x_stop, x_stop, x_start, x_start],
                y=[y_bottom, y_bottom, y_top, y_top, y_bottom],
                fill="toself",
                fillcolor="rgba(80, 100, 140, 0.15)",
                line=dict(color="rgba(100, 120, 160, 0.3)", width=1.5),
                hovertemplate=(
                    "<b>🌙 Nighttime period (observable)</b><br><br>"
                    f"<b>Month:</b> {month_label}<br>"
                    f"<b>Start:</b> {format_datetime_utc(seg_start)}<br>"
                    f"<b>End:</b> {format_datetime_utc(seg_stop)}<br>"
                    f"<b>Duration:</b> {duration_hours:.2f} hours"
                    "<extra></extra>"
                ),
                legendgroup="dark_periods",
                showlegend=dark_traces_added == 0,
                mode="lines",
            )
        )
        dark_traces_added += 1


def _add_observation_traces(
    fig: go.Figure,
    df: pd.DataFrame,
    month_to_position: dict,
    priority_min: float,
    priority_max: float,
) -> None:
    """Add observation traces to figure."""
    for _, row in df.iterrows():
        block_id = row["schedulingBlockId"]
        priority = row["priority"]
        start_dt = row["scheduled_start_dt"]
        stop_dt = row["scheduled_stop_dt"]
        duration_hours = row["duration_hours"]
        month_label = row["scheduled_month_label"]

        # Normalize priority to [0, 1] for colorscale
        normalized_priority = (priority - priority_min) / (priority_max - priority_min)

        # Check if observation spans multiple months
        start_month = start_dt.tz_localize(None).to_period("M")
        stop_month = stop_dt.tz_localize(None).to_period("M")

        if start_month != stop_month:
            _add_multi_month_observation(
                fig,
                row,
                start_dt,
                stop_dt,
                start_month,
                stop_month,
                month_to_position,
                normalized_priority,
                block_id,
                priority,
                duration_hours,
            )
        else:
            _add_single_month_observation(
                fig,
                row,
                start_dt,
                stop_dt,
                month_to_position,
                normalized_priority,
                block_id,
                priority,
                duration_hours,
                month_label,
            )


def _add_multi_month_observation(
    fig: go.Figure,
    row: pd.Series,
    start_dt: pd.Timestamp,
    stop_dt: pd.Timestamp,
    start_month: pd.Period,
    stop_month: pd.Period,
    month_to_position: dict[str, int],
    normalized_priority: float,
    block_id: str,
    priority: float,
    duration_hours: float,
) -> None:
    """Add observation that spans multiple months."""
    # Part 1: From start to end of start month
    end_of_start_month = start_dt.tz_localize(None).to_period("M").to_timestamp(how="end")

    start_month_label = start_dt.strftime("%Y-%m")
    if start_month_label in month_to_position:
        y_pos = month_to_position[start_month_label]
        y_bottom = y_pos - 0.4
        y_top = y_pos + 0.4

        start_day = start_dt.day + start_dt.hour / 24.0 + start_dt.minute / 1440.0
        end_day = end_of_start_month.day + 23.0 / 24.0 + 59.0 / 1440.0

        fig.add_trace(
            go.Scatter(
                name=f"Block {block_id} (part 1)",
                x=[start_day, end_day, end_day, start_day, start_day],
                y=[y_bottom, y_bottom, y_top, y_top, y_bottom],
                fill="toself",
                fillcolor=f"rgba({_viridis_color(normalized_priority)}, 0.85)",
                line=dict(
                    color=f"rgba({_viridis_color(normalized_priority)}, 1.0)",
                    width=1,
                ),
                hovertemplate=(
                    f"<b>📡 Block {block_id}</b> (crosses months)<br><br>"
                    f"<b>Month:</b> {start_month_label}<br>"
                    f"<b>Days:</b> {start_dt.day} → {end_of_start_month.day}<br>"
                    f"<b>Priority:</b> {priority:.2f}<br>"
                    f"<b>Total duration:</b> {duration_hours:.2f} hours<br><br>"
                    f"<b>Start:</b> {format_datetime_utc(start_dt)}<br>"
                    f"<b>Total end:</b> {format_datetime_utc(stop_dt)}<br>"
                    "<extra></extra>"
                ),
                showlegend=False,
                mode="lines",
                customdata=[[block_id, priority, duration_hours, start_month_label]],
            )
        )

    # Part 2: From start of stop month to actual stop
    stop_month_label = stop_dt.strftime("%Y-%m")
    if stop_month_label in month_to_position:
        y_pos = month_to_position[stop_month_label]
        y_bottom = y_pos - 0.4
        y_top = y_pos + 0.4

        start_day = 1.0
        stop_day = stop_dt.day + stop_dt.hour / 24.0 + stop_dt.minute / 1440.0

        fig.add_trace(
            go.Scatter(
                name=f"Block {block_id} (part 2)",
                x=[start_day, stop_day, stop_day, start_day, start_day],
                y=[y_bottom, y_bottom, y_top, y_top, y_bottom],
                fill="toself",
                fillcolor=f"rgba({_viridis_color(normalized_priority)}, 0.85)",
                line=dict(
                    color=f"rgba({_viridis_color(normalized_priority)}, 1.0)",
                    width=1,
                ),
                hovertemplate=(
                    f"<b>📡 Block {block_id}</b> (crosses months)<br><br>"
                    f"<b>Month:</b> {stop_month_label}<br>"
                    f"<b>Days:</b> 1 → {stop_dt.day}<br>"
                    f"<b>Priority:</b> {priority:.2f}<br>"
                    f"<b>Total duration:</b> {duration_hours:.2f} hours<br><br>"
                    f"<b>Total start:</b> {format_datetime_utc(start_dt)}<br>"
                    f"<b>End:</b> {format_datetime_utc(stop_dt)}<br>"
                    "<extra></extra>"
                ),
                showlegend=False,
                mode="lines",
                customdata=[[block_id, priority, duration_hours, stop_month_label]],
            )
        )


def _add_single_month_observation(
    fig: go.Figure,
    row: pd.Series,
    start_dt: pd.Timestamp,
    stop_dt: pd.Timestamp,
    month_to_position: dict,
    normalized_priority: float,
    block_id: str,
    priority: float,
    duration_hours: float,
    month_label: str,
) -> None:
    """Add observation within a single month."""
    y_pos = month_to_position[month_label]
    y_bottom = y_pos - 0.4
    y_top = y_pos + 0.4

    start_day = start_dt.day + start_dt.hour / 24.0 + start_dt.minute / 1440.0
    stop_day = stop_dt.day + stop_dt.hour / 24.0 + stop_dt.minute / 1440.0

    fig.add_trace(
        go.Scatter(
            name=f"Block {block_id}",
            x=[start_day, stop_day, stop_day, start_day, start_day],
            y=[y_bottom, y_bottom, y_top, y_top, y_bottom],
            fill="toself",
            fillcolor=f"rgba({_viridis_color(normalized_priority)}, 0.85)",
            line=dict(
                color=f"rgba({_viridis_color(normalized_priority)}, 1.0)",
                width=1,
            ),
            hovertemplate=(
                f"<b>📡 Block {block_id}</b><br><br>"
                f"<b>Month:</b> {month_label}<br>"
                f"<b>Days:</b> {start_dt.day} → {stop_dt.day}<br>"
                f"<b>Priority:</b> {priority:.2f}<br>"
                f"<b>Duration:</b> {duration_hours:.2f} hours<br><br>"
                f"<b>Start:</b> {format_datetime_utc(start_dt)}<br>"
                f"<b>End:</b> {format_datetime_utc(stop_dt)}<br>"
                "<extra></extra>"
            ),
            showlegend=False,
            mode="lines",
            customdata=[[block_id, priority, duration_hours, month_label]],
        )
    )


def _add_colorbar(fig: go.Figure, priority_min: float, priority_max: float) -> None:
    """Add dummy trace for colorbar legend."""
    fig.add_trace(
        go.Scatter(
            x=[None],
            y=[None],
            mode="markers",
            marker=dict(
                colorscale="Viridis",
                cmin=priority_min,
                cmax=priority_max,
                colorbar=dict(
                    title="Priority",
                    thickness=15,
                    len=0.7,
                ),
                showscale=True,
            ),
            hoverinfo="skip",
            showlegend=False,
        )
    )


def _viridis_color(normalized_value: float) -> str:
    """
    Convert a normalized value [0, 1] to an RGB string using Viridis colorscale approximation.

    Args:
        normalized_value: Value between 0 and 1

    Returns:
        RGB values as comma-separated string (e.g., "68, 1, 84")
    """
    # Simplified Viridis colorscale (5 key points)
    viridis_colors = [
        (0.0, (68, 1, 84)),
        (0.25, (59, 82, 139)),
        (0.5, (33, 145, 140)),
        (0.75, (94, 201, 98)),
        (1.0, (253, 231, 37)),
    ]

    # Clamp value to [0, 1]
    normalized_value = max(0.0, min(1.0, normalized_value))

    # Find the two colors to interpolate between
    for i in range(len(viridis_colors) - 1):
        pos1, color1 = viridis_colors[i]
        pos2, color2 = viridis_colors[i + 1]

        if pos1 <= normalized_value <= pos2:
            # Linear interpolation
            t = (normalized_value - pos1) / (pos2 - pos1) if pos2 != pos1 else 0
            r = int(color1[0] + t * (color2[0] - color1[0]))
            g = int(color1[1] + t * (color2[1] - color1[1]))
            b = int(color1[2] + t * (color2[2] - color1[2]))
            return f"{r}, {g}, {b}"

    # Fallback
    return "68, 1, 84"


def _split_dark_periods_by_month(
    dark_df: pd.DataFrame,
) -> list[tuple[str, pd.Timestamp, pd.Timestamp]]:
    """Split each dark period by month boundaries for plotting."""
    segments: list[tuple[str, pd.Timestamp, pd.Timestamp]] = []
    for _, row in dark_df.iterrows():
        start_dt = row.get("start_dt")
        stop_dt = row.get("stop_dt")

        if start_dt is None or stop_dt is None:
            continue
        if pd.isna(start_dt) or pd.isna(stop_dt):
            continue

        for month_label, seg_start, seg_stop in _iter_month_segments(start_dt, stop_dt):
            if seg_stop <= seg_start:
                continue
            segments.append((month_label, seg_start, seg_stop))

    return segments


def _invert_to_light_periods(
    dark_df: pd.DataFrame,
) -> list[tuple[str, pd.Timestamp, pd.Timestamp]]:
    """
    Invert dark (observable) periods to obtain light (non-observable) periods.

    Returns:
        List of tuples: (month_label, start, stop) for each light period
    """
    if dark_df.empty:
        return []

    # Sort by start time
    dark_df_sorted = dark_df.sort_values("start_dt").reset_index(drop=True)

    light_segments: list[tuple[str, pd.Timestamp, pd.Timestamp]] = []

    # For each month, find the gaps between dark periods
    all_months = set()
    for month_list in dark_df_sorted["months"]:
        all_months.update(month_list)

    for month_label in sorted(all_months):
        # Get all dark periods for this month
        month_dark = dark_df_sorted[
            dark_df_sorted["months"].apply(lambda x: month_label in x)
        ].copy()

        if month_dark.empty:
            continue

        # Get month boundaries
        year, month = map(int, month_label.split("-"))
        month_start = pd.Timestamp(year=year, month=month, day=1, hour=0, minute=0, tz="UTC")

        # Compute the first instant of the next month
        if month == 12:
            month_end = pd.Timestamp(year=year + 1, month=1, day=1, hour=0, minute=0, tz="UTC")
        else:
            month_end = pd.Timestamp(year=year, month=month + 1, day=1, hour=0, minute=0, tz="UTC")

        # Collect all dark periods of the month in order
        dark_periods_in_month = []
        for _, row in month_dark.iterrows():
            start = max(row["start_dt"], month_start)
            stop = min(row["stop_dt"], month_end)
            if start < stop:
                dark_periods_in_month.append((start, stop))

        # Sort by start time
        dark_periods_in_month.sort(key=lambda x: x[0])

        if not dark_periods_in_month:
            continue

        # Create light periods between dark periods
        # 1. Gap from the start of the month to the first dark period
        first_dark_start = dark_periods_in_month[0][0]
        if month_start < first_dark_start:
            light_segments.append((month_label, month_start, first_dark_start))

        # 2. Gaps between consecutive dark periods
        for i in range(len(dark_periods_in_month) - 1):
            current_dark_end = dark_periods_in_month[i][1]
            next_dark_start = dark_periods_in_month[i + 1][0]

            if current_dark_end < next_dark_start:
                light_segments.append((month_label, current_dark_end, next_dark_start))

        # 3. Gap from the last dark period to the end of the month
        last_dark_end = dark_periods_in_month[-1][1]
        if last_dark_end < month_end:
            light_segments.append((month_label, last_dark_end, month_end))

    return light_segments


def _iter_month_segments(
    start_dt: pd.Timestamp, stop_dt: pd.Timestamp
) -> list[tuple[str, pd.Timestamp, pd.Timestamp]]:
    """Yield (month, start, stop) segments clipped to month boundaries."""
    segments: list[tuple[str, pd.Timestamp, pd.Timestamp]] = []
    current_start = start_dt

    while current_start < stop_dt:
        month_label = current_start.strftime("%Y-%m")
        next_month_start = _start_of_next_month(current_start)
        month_end = min(stop_dt, next_month_start - pd.Timedelta(seconds=1))

        segments.append((month_label, current_start, month_end))

        if month_end >= stop_dt:
            break

        current_start = next_month_start

    return segments


def _start_of_next_month(dt: pd.Timestamp) -> pd.Timestamp:
    """Return the first instant of the month following ``dt`` preserving timezone."""
    tz = dt.tzinfo or "UTC"
    year = dt.year
    month = dt.month

    if month == 12:
        return pd.Timestamp(year=year + 1, month=1, day=1, tz=tz)

    return pd.Timestamp(year=year, month=month + 1, day=1, tz=tz)


def _datetime_to_day_fraction(dt: pd.Timestamp) -> float:
    """Convert a timestamp into day-of-month with fractional component."""
    base = float(dt.day)
    base += dt.hour / 24.0
    base += dt.minute / 1440.0
    base += dt.second / 86_400.0
    base += dt.microsecond / 86_400_000_000.0
    return cast(float, base)
