"""Calendar heatmap data processing service.

Handles all compute-heavy operations for calendar heatmap visualization:
- Time bin generation
- Occupancy calculation
- Overlap detection
- Conflict identification
"""

from __future__ import annotations

from typing import Literal, cast

import numpy as np
import pandas as pd

XUnit = Literal["hours", "days", "weeks", "months"]
YUnit = Literal["days", "weeks", "months"]


def _to_timedelta(unit: XUnit | YUnit) -> pd.Timedelta:
    if unit == "hours":
        return pd.Timedelta(minutes=15)
    if unit == "days":
        return pd.Timedelta(days=1)
    if unit == "weeks":
        return pd.Timedelta(weeks=1)
    return pd.Timedelta(days=1)


def compute_calendar_bins(
    df: pd.DataFrame,
    *,
    x_unit: XUnit = "hours",
    y_unit: YUnit = "days",
    range_start: pd.Timestamp | None = None,
    range_end: pd.Timestamp | None = None,
    instrument_col: str | None = None,
    pending_duration: pd.Timedelta | None = None,
) -> pd.DataFrame:
    """Compute occupancy bins for calendar heatmap.

    Args:
        df: DataFrame with scheduled_start_dt and scheduled_stop_dt columns
        x_unit: X-axis time unit (hours, days, weeks, months)
        y_unit: Y-axis time unit (days, weeks, months)
        range_start: Start of time range (defaults to min scheduled_start_dt)
        range_end: End of time range (defaults to max scheduled_stop_dt)
        instrument_col: Column name for instrument (enables conflict detection)
        pending_duration: If set, marks bins suitable for this duration

    Returns:
        DataFrame with columns: y_start, y_label, x_start, x_stop, duration_s,
        occupied_s, occupancy, overlaps (list), conflict (bool), suitable_for_pending (bool)
    """
    scheduled = df[df.get("scheduled_flag", False)].copy()

    if range_start is None:
        if not scheduled.empty:
            range_start = scheduled["scheduled_start_dt"].min()
        else:
            range_start = pd.Timestamp.now(tz="UTC") - pd.Timedelta(days=7)
    if range_end is None:
        if not scheduled.empty:
            range_end = scheduled["scheduled_stop_dt"].max()
        else:
            range_end = pd.Timestamp.now(tz="UTC") + pd.Timedelta(days=7)

    range_start = pd.to_datetime(range_start, utc=True)
    range_end = pd.to_datetime(range_end, utc=True)

    y_starts = _generate_y_bins(range_start, range_end, y_unit)
    x_delta = _to_timedelta(x_unit)

    rows = []

    for y0 in y_starts:
        y1 = _compute_y_end(y0, y_unit)
        row_start = max(y0, range_start)
        row_end = min(y1, range_end)
        if row_start >= row_end:
            continue

        x0 = row_start
        while x0 < row_end:
            x1 = min(x0 + x_delta, row_end)
            duration = (x1 - x0).total_seconds()

            overlaps = []
            occ_seconds = 0.0
            for idx, r in scheduled.iterrows():
                s = r.get("scheduled_start_dt")
                e = r.get("scheduled_stop_dt")
                if s is None or e is None:
                    continue
                overlap_start = max(s, x0)
                overlap_end = min(e, x1)
                if overlap_end > overlap_start:
                    overlap = (overlap_end - overlap_start).total_seconds()
                    occ_seconds += overlap
                    overlaps.append(r.get("schedulingBlockId", str(idx)))

            occupancy_fraction = min(1.0, occ_seconds / duration) if duration > 0 else 0.0

            conflict = False
            if instrument_col and not scheduled.empty:
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

    if pending_duration and pending_duration.total_seconds() > 0 and not bins_df.empty:
        bins_df["suitable_for_pending"] = _compute_suitable_bins(
            bins_df, pending_duration.total_seconds()
        )
    else:
        bins_df["suitable_for_pending"] = False

    return bins_df


def _generate_y_bins(
    range_start: pd.Timestamp, range_end: pd.Timestamp, y_unit: YUnit
) -> pd.DatetimeIndex:
    """Generate Y-axis bin start times."""
    if y_unit == "months":
        y_starts_base = pd.date_range(
            start=range_start.normalize(), end=range_end, freq="MS", tz="UTC"
        )
        if y_starts_base[-1] < range_end:
            return cast(
                pd.DatetimeIndex,
                y_starts_base.union(
                    pd.DatetimeIndex([y_starts_base[-1] + pd.offsets.MonthBegin()])
                ),
            )
        return y_starts_base
    elif y_unit == "weeks":
        return pd.date_range(start=range_start.normalize(), end=range_end, freq="W-MON", tz="UTC")
    else:
        return pd.date_range(start=range_start.normalize(), end=range_end, freq="D", tz="UTC")


def _compute_y_end(y0: pd.Timestamp, y_unit: YUnit) -> pd.Timestamp:
    """Compute end time for a Y-axis bin."""
    if y_unit == "months":
        return y0 + pd.offsets.MonthBegin(1)
    elif y_unit == "weeks":
        return y0 + pd.Timedelta(weeks=1)
    else:
        return y0 + pd.Timedelta(days=1)


def _compute_suitable_bins(bins_df: pd.DataFrame, needed_seconds: float) -> pd.Series:
    """Compute which bins are suitable for a pending observation duration."""
    x_labels = sorted(bins_df["x_start"].unique())
    y_labels = bins_df["y_label"].unique()

    occupancy_matrix = np.zeros((len(y_labels), len(x_labels)), dtype=float)
    duration_matrix = np.zeros((len(y_labels), len(x_labels)), dtype=float)

    x_index = {x: i for i, x in enumerate(x_labels)}
    y_index = {y: i for i, y in enumerate(y_labels)}

    for _, row in bins_df.iterrows():
        xi = x_index[row["x_start"]]
        yi = y_index[row["y_label"]]
        occupancy_matrix[yi, xi] = row["occupancy"]
        duration_matrix[yi, xi] = row["duration_s"]

    free_matrix = (1.0 - occupancy_matrix) * duration_matrix

    suitable_coords = set()
    for yi in range(free_matrix.shape[0]):
        seq_sum = 0.0
        seq_start = 0
        for xi in range(free_matrix.shape[1]):
            val = free_matrix[yi, xi]
            if val > 0:
                seq_sum += val
            else:
                seq_sum = 0.0
                seq_start = xi + 1
            if seq_sum >= needed_seconds:
                for xj in range(seq_start, xi + 1):
                    suitable_coords.add((yi, xj))
                seq_sum = 0.0
                seq_start = xi + 1

    suitable = []
    for _, row in bins_df.iterrows():
        xi = x_index[row["x_start"]]
        yi = y_index[row["y_label"]]
        suitable.append((yi, xi) in suitable_coords)

    return pd.Series(suitable, index=bins_df.index)
