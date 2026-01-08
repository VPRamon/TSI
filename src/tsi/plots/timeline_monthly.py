"""Scheduled timeline plotting functions."""

from __future__ import annotations

import calendar
from collections.abc import Iterable
from dataclasses import dataclass
from typing import Any

import pandas as pd
import plotly.graph_objects as go

from tsi.services.utils.time import format_datetime_utc, mjd_to_datetime


@dataclass(frozen=True)
class MonthSegment:
    """Time window clipped to a calendar month."""

    month_label: str
    start: pd.Timestamp
    stop: pd.Timestamp

    @property
    def duration_hours(self) -> float:
        return (self.stop - self.start).total_seconds() / 3600.0


def build_monthly_timeline(
    blocks: list,
    priority_range: tuple[float, float],
    dark_periods: list[tuple[float, float]] | None = None,
) -> go.Figure:
    """Build a Plotly figure showing scheduled observations grouped by month."""
    priority_min, priority_max = _normalize_priority_range(priority_range)

    dark_segments = _build_dark_segments(dark_periods or [])
    ordered_months = sorted(_collect_months(blocks, dark_segments))
    month_to_position = {month: idx for idx, month in enumerate(ordered_months)}

    fig = go.Figure()

    if dark_segments:
        light_segments = _invert_dark_segments(dark_segments, ordered_months)
        _add_period_bands(
            fig,
            light_segments,
            month_to_position,
            name="Daytime periods (non-observable)",
            fillcolor="rgba(255, 230, 180, 0.15)",
            linecolor="rgba(200, 180, 140, 0.3)",
            hover_label="☀️ Daytime period (non-observable)",
            legendgroup="light_periods",
        )
        _add_period_bands(
            fig,
            dark_segments,
            month_to_position,
            name="Nighttime periods (observable)",
            fillcolor="rgba(80, 100, 140, 0.15)",
            linecolor="rgba(100, 120, 160, 0.3)",
            hover_label="🌙 Nighttime period (observable)",
            legendgroup="dark_periods",
        )

    _add_observations(fig, blocks, month_to_position, priority_min, priority_max)
    _add_colorbar(fig, priority_min, priority_max)

    num_months = len(ordered_months)
    height = max(600, num_months * 80)

    fig.update_layout(
        title=f"Scheduled Timeline by Month ({len(blocks):,} observations)",
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
            range=[-0.5, num_months - 0.5] if num_months else [-0.5, 0.5],
        ),
        height=height,
        margin=dict(l=100, r=120, t=80, b=80),
        hovermode="closest",
        dragmode="pan",
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
    )

    fig.update_xaxes(
        range=[0.5, 31.5],
        minallowed=0.5,
        maxallowed=31.5,
    )
    fig.update_yaxes(fixedrange=False)

    return fig


def _normalize_priority_range(priority_range: tuple[float, float]) -> tuple[float, float]:
    priority_min, priority_max = priority_range
    if priority_max <= priority_min:
        priority_max = priority_min + 1.0
    return priority_min, priority_max


def _collect_months(blocks: list, dark_segments: Iterable[MonthSegment]) -> set[str]:
    months = {segment.month_label for segment in dark_segments}
    for block in blocks:
        start_dt = mjd_to_datetime(float(block.scheduled_start_mjd))
        stop_dt = mjd_to_datetime(float(block.scheduled_stop_mjd))
        months.update(_iter_month_labels(start_dt, stop_dt))
    return months


def _build_dark_segments(dark_periods: list[tuple[float, float]]) -> list[MonthSegment]:
    segments: list[MonthSegment] = []
    for start_mjd, stop_mjd in dark_periods:
        start_dt = mjd_to_datetime(float(start_mjd))
        stop_dt = mjd_to_datetime(float(stop_mjd))
        if stop_dt <= start_dt:
            continue
        segments.extend(_split_by_month(start_dt, stop_dt))
    return segments


def _invert_dark_segments(
    dark_segments: Iterable[MonthSegment],
    ordered_months: list[str],
) -> list[MonthSegment]:
    """Return light (non-observable) segments that fill the gaps between dark ones."""
    segments_by_month: dict[str, list[MonthSegment]] = {}
    for segment in dark_segments:
        segments_by_month.setdefault(segment.month_label, []).append(segment)

    light_segments: list[MonthSegment] = []
    for month_label in ordered_months:
        month_segments = sorted(segments_by_month.get(month_label, []), key=lambda seg: seg.start)
        month_start, month_end = _month_range(
            month_label, month_segments[0].start.tzinfo if month_segments else "UTC"
        )

        if not month_segments:
            light_segments.append(MonthSegment(month_label, month_start, month_end))
            continue

        current = month_start
        for segment in month_segments:
            if current < segment.start:
                light_segments.append(
                    MonthSegment(
                        month_label,
                        current,
                        segment.start - pd.Timedelta(microseconds=1),
                    )
                )
            current = segment.stop + pd.Timedelta(microseconds=1)

        if current <= month_end:
            light_segments.append(MonthSegment(month_label, current, month_end))

    return light_segments


def _add_period_bands(
    fig: go.Figure,
    segments: Iterable[MonthSegment],
    month_to_position: dict[str, int],
    *,
    name: str,
    fillcolor: str,
    linecolor: str,
    hover_label: str,
    legendgroup: str,
) -> None:
    show_legend = True
    for segment in segments:
        y_pos = month_to_position.get(segment.month_label)
        if y_pos is None:
            continue

        x_start = _datetime_to_day_fraction(segment.start)
        x_stop = _datetime_to_day_fraction(segment.stop)
        if x_stop <= x_start:
            continue

        y_bottom = y_pos - 0.04
        y_top = y_pos + 0.04

        fig.add_trace(
            go.Scatter(
                name=name,
                x=[x_start, x_stop, x_stop, x_start, x_start],
                y=[y_bottom, y_bottom, y_top, y_top, y_bottom],
                fill="toself",
                fillcolor=fillcolor,
                line=dict(color=linecolor, width=1.5),
                hovertemplate=(
                    f"<b>{hover_label}</b><br><br>"
                    f"<b>Month:</b> {segment.month_label}<br>"
                    f"<b>Start:</b> {format_datetime_utc(segment.start)}<br>"
                    f"<b>End:</b> {format_datetime_utc(segment.stop)}<br>"
                    f"<b>Duration:</b> {segment.duration_hours:.2f} hours"
                    "<extra></extra>"
                ),
                legendgroup=legendgroup,
                showlegend=show_legend,
                mode="lines",
            )
        )
        show_legend = False


def _add_observations(
    fig: go.Figure,
    blocks: list,
    month_to_position: dict[str, int],
    priority_min: float,
    priority_max: float,
) -> None:
    priority_span = max(priority_max - priority_min, 1e-9)
    for block in blocks:
        start_mjd = float(block.scheduled_start_mjd)
        stop_mjd = float(block.scheduled_stop_mjd)
        priority = float(block.priority)
        start_dt = mjd_to_datetime(start_mjd)
        stop_dt = mjd_to_datetime(stop_mjd)
        if stop_dt <= start_dt:
            continue

        normalized_priority = (priority - priority_min) / priority_span
        duration_hours = (stop_mjd - start_mjd) * 24.0
        segments = _split_by_month(start_dt, stop_dt)

        for idx, segment in enumerate(segments, start=1):
            _add_observation_segment(
                fig,
                block,
                segment,
                month_to_position,
                normalized_priority,
                priority,
                duration_hours,
                start_dt,
                stop_dt,
                idx,
                len(segments),
            )


def _add_observation_segment(
    fig: go.Figure,
    block: object,
    segment: MonthSegment,
    month_to_position: dict[str, int],
    normalized_priority: float,
    priority: float,
    duration_hours: float,
    start_dt: pd.Timestamp,
    stop_dt: pd.Timestamp,
    part_index: int,
    part_count: int,
) -> None:
    y_pos = month_to_position.get(segment.month_label)
    if y_pos is None:
        return

    x_start = _datetime_to_day_fraction(segment.start)
    x_stop = _datetime_to_day_fraction(segment.stop)
    if x_stop <= x_start:
        return

    y_bottom = y_pos - 0.4
    y_top = y_pos + 0.4
    block_id = getattr(block, "scheduling_block_id", getattr(block, "schedulingBlockId", "unknown"))
    part_suffix = "" if part_count == 1 else f" (part {part_index}/{part_count})"

    ra_deg = getattr(block, "ra_deg", None)
    dec_deg = getattr(block, "dec_deg", None)
    requested_hours = getattr(block, "requested_hours", None)
    visibility_hours = getattr(block, "total_visibility_hours", None)
    visibility_periods = getattr(block, "num_visibility_periods", None)

    visibility_text = ""
    if visibility_hours is not None and visibility_periods is not None:
        visibility_text = (
            f"<b>Visibility:</b> {visibility_hours:.2f}h ({visibility_periods} periods)<br>"
        )

    coord_text = ""
    if ra_deg is not None and dec_deg is not None:
        coord_text = f"<b>RA:</b> {ra_deg:.2f}°<br><b>Dec:</b> {dec_deg:.2f}°<br>"

    requested_text = ""
    if requested_hours is not None:
        requested_text = f"<b>Requested:</b> {requested_hours:.2f}h<br>"

    fig.add_trace(
        go.Scatter(
            name=f"Block {block_id}{part_suffix}",
            x=[x_start, x_stop, x_stop, x_start, x_start],
            y=[y_bottom, y_bottom, y_top, y_top, y_bottom],
            fill="toself",
            fillcolor=f"rgba({_viridis_color(normalized_priority)}, 0.85)",
            line=dict(
                color=f"rgba({_viridis_color(normalized_priority)}, 1.0)",
                width=1,
            ),
            hovertemplate=(
                f"<b>📡 Block {block_id}</b>{part_suffix}<br><br>"
                f"<b>Month:</b> {segment.month_label}<br>"
                f"<b>Days:</b> {segment.start.day} → {segment.stop.day}<br>"
                f"<b>Priority:</b> {priority:.2f}<br>"
                f"<b>Total duration:</b> {duration_hours:.2f} hours<br><br>"
                f"<b>Start:</b> {format_datetime_utc(start_dt)}<br>"
                f"<b>End:</b> {format_datetime_utc(stop_dt)}<br>"
                f"{requested_text}"
                f"{visibility_text}"
                f"{coord_text}"
                "<extra></extra>"
            ),
            showlegend=False,
            mode="lines",
        )
    )


def _split_by_month(start_dt: pd.Timestamp, stop_dt: pd.Timestamp) -> list[MonthSegment]:
    """Split a window into month-clipped segments."""
    if stop_dt <= start_dt:
        return []

    start_dt = _ensure_utc(start_dt)
    stop_dt = _ensure_utc(stop_dt)

    segments: list[MonthSegment] = []
    current_start = start_dt
    while current_start < stop_dt:
        month_label = current_start.strftime("%Y-%m")
        month_start, month_end = _month_range(month_label, current_start.tzinfo)
        segment_stop = min(stop_dt, month_end)
        segments.append(MonthSegment(month_label, current_start, segment_stop))
        current_start = segment_stop + pd.Timedelta(microseconds=1)

    return segments


def _iter_month_labels(start_dt: pd.Timestamp, stop_dt: pd.Timestamp) -> set[str]:
    if stop_dt < start_dt:
        return {start_dt.strftime("%Y-%m")}

    labels: set[str] = set()
    current = start_dt.replace(day=1)
    while current <= stop_dt:
        labels.add(current.strftime("%Y-%m"))
        current = _start_of_next_month(current)
    return labels


def _month_range(
    month_label: str, tz: Any
) -> tuple[pd.Timestamp, pd.Timestamp]:
    year, month = map(int, month_label.split("-"))
    last_day = calendar.monthrange(year, month)[1]
    month_start = pd.Timestamp(year=year, month=month, day=1, tz=tz)
    month_end = pd.Timestamp(
        year=year,
        month=month,
        day=last_day,
        hour=23,
        minute=59,
        second=59,
        microsecond=999999,
        tz=tz,
    )
    return month_start, month_end


def _start_of_next_month(dt: pd.Timestamp) -> pd.Timestamp:
    tz = dt.tzinfo
    year, month = dt.year, dt.month
    if month == 12:
        return pd.Timestamp(year=year + 1, month=1, day=1, tz=tz)
    return pd.Timestamp(year=year, month=month + 1, day=1, tz=tz)


def _ensure_utc(dt: pd.Timestamp) -> pd.Timestamp:
    if dt.tzinfo is None:
        return dt.tz_localize("UTC")
    return dt.tz_convert("UTC")


def _datetime_to_day_fraction(dt: pd.Timestamp) -> float:
    base = float(dt.day)
    base += dt.hour / 24.0
    base += dt.minute / 1440.0
    base += dt.second / 86_400.0
    base += dt.microsecond / 86_400_000_000.0
    return base


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
    viridis_colors = [
        (0.0, (68, 1, 84)),
        (0.25, (59, 82, 139)),
        (0.5, (33, 145, 140)),
        (0.75, (94, 201, 98)),
        (1.0, (253, 231, 37)),
    ]

    normalized_value = max(0.0, min(1.0, normalized_value))

    for i in range(len(viridis_colors) - 1):
        pos1, color1 = viridis_colors[i]
        pos2, color2 = viridis_colors[i + 1]

        if pos1 <= normalized_value <= pos2:
            t = (normalized_value - pos1) / (pos2 - pos1) if pos2 != pos1 else 0
            r = int(color1[0] + t * (color2[0] - color1[0]))
            g = int(color1[1] + t * (color2[1] - color1[1]))
            b = int(color1[2] + t * (color2[2] - color1[2]))
            return f"{r}, {g}, {b}"

    return "68, 1, 84"
