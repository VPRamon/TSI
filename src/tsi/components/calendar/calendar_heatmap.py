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

from tsi.services.calendar_processing import compute_calendar_bins

XUnit = Literal["hours", "days", "weeks", "months"]
YUnit = Literal["days", "weeks", "months"]


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
    bins_df = compute_calendar_bins(
        df,
        x_unit=x_unit,
        y_unit=y_unit,
        range_start=range_start,
        range_end=range_end,
        instrument_col=instrument_col,
        pending_duration=pending_duration,
    )

    if bins_df.empty:
        fig = go.Figure()
        fig.update_layout(title="Calendar heatmap (no data)")
        return fig, bins_df

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

    for yi, ylab in enumerate(y_labels):
        for xi, xlab in enumerate(x_labels):
            if conflict_mask[yi, xi]:
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

    if "suitable_for_pending" in bins_df.columns:
        for _, row in bins_df[bins_df["suitable_for_pending"]].iterrows():
            xi = x_index[row["x_start"]]
            yi = y_index[row["y_label"]]
            x0 = row["x_start"]
            x1 = x0 + pd.Timedelta(seconds=row["duration_s"])
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
