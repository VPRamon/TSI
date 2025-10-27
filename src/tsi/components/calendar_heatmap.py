"""Calendar heatmap component for schedule inspection.

Provides a reusable function to build a configurable time-grid heatmap
showing occupancy, partial fills and conflicts. Uses Plotly for rendering
and returns both the figure and the underlying bin dataframe for further
interaction (simulations, zooming, etc.).
"""

from __future__ import annotations

from typing import Literal

import numpy as np
import pandas as pd
import plotly.graph_objects as go

XUnit = Literal["hours", "days", "weeks", "months"]
YUnit = Literal["days", "weeks", "months"]


def _to_timedelta(unit: XUnit | YUnit) -> pd.Timedelta:
    if unit == "hours":
        return pd.Timedelta(minutes=15)
    if unit == "days":
        return pd.Timedelta(days=1)
    if unit == "weeks":
        return pd.Timedelta(weeks=1)
    # months have variable length; we will treat as 1 day placeholder when
    # computing deltas but create bins based on month boundaries where needed
    return pd.Timedelta(days=1)


def build_calendar_heatmap(
    df: pd.DataFrame,
    *,
    x_unit: XUnit = "hours",
    y_unit: YUnit = "days",
    range_start: pd.Timestamp | None = None,
    range_end: pd.Timestamp | None = None,
    instrument_col: str | None = None,
    pending_duration: pd.Timedelta | None = None,
) -> tuple[go.Figure, pd.DataFrame]:
    """Build a heatmap figure and return the occupancy bins.

    df is expected to contain `scheduled_start_dt` and `scheduled_stop_dt`
    timestamps (UTC aware) produced by the preparation pipeline.

    Returns (figure, bins_df) where bins_df contains one row per cell with
    columns: y_start, x_start, x_stop, duration, occupied_seconds,
    occupancy_fraction, overlapping_ids (list), conflict (bool).
    """

    # Defensive copy
    scheduled = df[df.get("scheduled_flag", False)].copy()

    if range_start is None:
        # try scheduled times, otherwise full dataset visibility
        if not scheduled.empty:
            range_start = scheduled["scheduled_start_dt"].min()
        else:
            range_start = pd.Timestamp.now(tz="UTC") - pd.Timedelta(days=7)
    if range_end is None:
        if not scheduled.empty:
            range_end = scheduled["scheduled_stop_dt"].max()
        else:
            range_end = pd.Timestamp.now(tz="UTC") + pd.Timedelta(days=7)

    # Ensure timestamps
    range_start = pd.to_datetime(range_start, utc=True)
    range_end = pd.to_datetime(range_end, utc=True)

    # Build Y bins (rows)
    if y_unit == "months":
        y_starts_base = pd.date_range(
            start=range_start.normalize(), end=range_end, freq="MS", tz="UTC"
        )
        # ensure last month included
        if y_starts_base[-1] < range_end:
            y_starts: pd.DatetimeIndex = y_starts_base.union(pd.DatetimeIndex([y_starts_base[-1] + pd.offsets.MonthBegin()]))  # type: ignore[assignment]
        else:
            y_starts = y_starts_base
    elif y_unit == "weeks":
        y_starts = pd.date_range(
            start=range_start.normalize(), end=range_end, freq="W-MON", tz="UTC"
        )
    else:  # days
        y_starts = pd.date_range(start=range_start.normalize(), end=range_end, freq="D", tz="UTC")

    # X bin delta
    x_delta = _to_timedelta(x_unit)

    rows = []

    for y0 in y_starts:
        # Compute y-row end
        if y_unit == "months":
            y1 = y0 + pd.offsets.MonthBegin(1)
        elif y_unit == "weeks":
            y1 = y0 + pd.Timedelta(weeks=1)
        else:
            y1 = y0 + pd.Timedelta(days=1)

        row_start = max(y0, range_start)
        row_end = min(y1, range_end)
        if row_start >= row_end:
            continue

        # build x bins inside this row between row_start and row_end
        x0 = row_start
        while x0 < row_end:
            x1 = x0 + x_delta
            if x1 > row_end:
                x1 = row_end

            duration = (x1 - x0).total_seconds()

            # find scheduled observations overlapping this bin
            overlaps = []
            occ_seconds = 0.0
            for idx, r in scheduled.iterrows():
                s = r.get("scheduled_start_dt")
                e = r.get("scheduled_stop_dt")
                if s is None or e is None:
                    continue
                # overlap
                overlap_start = max(s, x0)
                overlap_end = min(e, x1)
                if overlap_end > overlap_start:
                    overlap = (overlap_end - overlap_start).total_seconds()
                    occ_seconds += overlap
                    overlaps.append(r.get("schedulingBlockId", str(idx)))

            occupancy_fraction = min(1.0, occ_seconds / duration) if duration > 0 else 0.0

            # Conflict detection: if same instrument has >1 observation overlap
            conflict = False
            if instrument_col and not scheduled.empty:
                # count overlaps per instrument
                hits: dict[str, int] = {}
                for idx, r in scheduled.iterrows():
                    s = r.get("scheduled_start_dt")
                    e = r.get("scheduled_stop_dt")
                    if s is None or e is None:
                        continue
                    overlap_start = max(s, x0)
                    overlap_end = min(e, x1)
                    if overlap_end > overlap_start:
                        inst = r.get(instrument_col)
                        if inst is not None:
                            hits[str(inst)] = hits.get(str(inst), 0) + 1
                # conflict if any instrument has more than one
                conflict = any(cnt > 1 for cnt in hits.values())

            rows.append(
                {
                    "y_start": y0,
                    "y_label": (
                        y0.strftime("%Y-%m-%d") if y_unit != "months" else y0.strftime("%Y-%m")
                    ),
                    "x_start": x0,
                    "x_stop": x1,
                    "duration_s": duration,
                    "occupied_s": occ_seconds,
                    "occupancy": occupancy_fraction,
                    "overlaps": overlaps,
                    "conflict": conflict,
                }
            )

            x0 = x1

    bins_df = pd.DataFrame(rows)
    if bins_df.empty:
        # return empty figure
        fig = go.Figure()
        fig.update_layout(title="Calendar heatmap (no data)")
        return fig, bins_df

    # Pivot to matrix for plotting: rows are y_label, columns are x_start
    # make labels for x columns
    x_labels = sorted(bins_df["x_start"].unique())
    y_labels = [lbl for lbl in bins_df["y_label"].unique()]

    matrix = np.zeros((len(y_labels), len(x_labels)), dtype=float)
    hover_text = [["" for _ in x_labels] for _ in y_labels]
    conflict_mask = np.zeros_like(matrix, dtype=bool)

    x_index = {x: i for i, x in enumerate(x_labels)}
    y_index = {y: i for i, y in enumerate(y_labels)}

    for _, row in bins_df.iterrows():
        xi = x_index[row["x_start"]]
        yi = y_index[row["y_label"]]
        matrix[yi, xi] = row["occupancy"]
        conflict_mask[yi, xi] = bool(row["conflict"])
        hover_text[yi][xi] = (
            f"{row['x_start'].strftime('%Y-%m-%d %H:%M')} - {row['x_stop'].strftime('%Y-%m-%d %H:%M')}<br>"
            f"Occupancy: {row['occupancy']*100:.1f}%<br>"
            f"Tasks: {', '.join(map(str, row['overlaps'])) if row['overlaps'] else 'None'}"
        )

    # color scale: perceptually-uniform-ish blue->gray->black
    colorscale = [
        [0.0, "#e8f4ff"],
        [0.5, "#c0c8d6"],
        [1.0, "#0b0b0b"],
    ]

    fig = go.Figure(
        data=go.Heatmap(
            z=matrix,
            x=[x.strftime("%Y-%m-%d %H:%M") for x in x_labels],
            y=y_labels,
            text=hover_text,
            hoverinfo="text",
            colorscale=colorscale,
            zmin=0,
            zmax=1,
            colorbar=dict(title="Occupancy"),
        )
    )

    # Add red borders for conflict cells
    for yi, ylab in enumerate(y_labels):
        for xi, xlab in enumerate(x_labels):
            if conflict_mask[yi, xi]:
                # draw rectangle via scatter trace
                x0 = x_labels[xi]
                x1 = x0 + pd.Timedelta(
                    seconds=bins_df[bins_df["x_start"] == x0]["duration_s"].iloc[0]
                )
                fig.add_shape(
                    type="rect",
                    x0=x0.strftime("%Y-%m-%d %H:%M"),
                    x1=x1.strftime("%Y-%m-%d %H:%M"),
                    y0=yi - 0.5,
                    y1=yi + 0.5,
                    xref="x",
                    yref="y",
                    line=dict(color="red", width=2),
                    fillcolor="rgba(0,0,0,0)",
                )

    fig.update_layout(
        title="Matriz temporal (heatmap)",
        xaxis=dict(tickangle=-45),
        yaxis=dict(autorange="reversed"),
        margin=dict(l=120, r=20, t=60, b=120),
        height=400 + len(y_labels) * 20,
    )

    # Highlight bins suitable for pending_duration if provided
    if pending_duration and pending_duration.total_seconds() > 0:
        needed = pending_duration.total_seconds()
        # For each row search for contiguous free capacity
        free_mask = (1.0 - matrix) * np.array(
            [bins_df[bins_df["x_start"] == x]["duration_s"].iloc[0] for x in x_labels]
        )
        # free_mask: rows x cols -> free seconds
        # find sequences per row where sum >= needed
        highlight_coords = []
        for yi in range(free_mask.shape[0]):
            seq_sum = 0.0
            seq_start = 0
            for xi in range(free_mask.shape[1]):
                val = free_mask[yi, xi]
                if val > 0:
                    seq_sum += val
                else:
                    seq_sum = 0.0
                    seq_start = xi + 1
                if seq_sum >= needed:
                    # mark sequence from seq_start..xi
                    for xj in range(seq_start, xi + 1):
                        highlight_coords.append((yi, xj))
                    # reset to find other sequences
                    seq_sum = 0.0
                    seq_start = xi + 1

        # draw faint green overlays
        for yi, xi in highlight_coords:
            x0 = x_labels[xi]
            dur = bins_df[bins_df["x_start"] == x0]["duration_s"].iloc[0]
            x1 = x0 + pd.Timedelta(seconds=dur)
            fig.add_shape(
                type="rect",
                x0=x0.strftime("%Y-%m-%d %H:%M"),
                x1=x1.strftime("%Y-%m-%d %H:%M"),
                y0=yi - 0.5,
                y1=yi + 0.5,
                xref="x",
                yref="y",
                line=dict(color="rgba(0,0,0,0)", width=0),
                fillcolor="rgba(0,200,0,0.2)",
            )

    return fig, bins_df
