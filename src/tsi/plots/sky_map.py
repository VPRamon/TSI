"""Sky map plotting functionality."""

from collections.abc import Sequence

import pandas as pd
import plotly.graph_objects as go

from tsi.config import PLOT_HEIGHT, PLOT_MARGIN


def build_figure(
    df: pd.DataFrame,
    color_by: str = "priority_bin",
    size_by: str = "requested_hours",
    flip_ra: bool = True,
    category_palette: dict | None = None,
) -> go.Figure:
    """
    Build an interactive sky map scatter plot with RA and Dec.

    Args:
        df: Prepared DataFrame with celestial coordinates
        color_by: Column to use for color coding
        size_by: Column to use for marker size
        flip_ra: If True, reverse RA axis (astronomy convention)
        category_palette: Optional dict mapping category -> color

    Returns:
        Plotly Figure object
    """
    if len(df) == 0:
        # Return empty figure
        fig = go.Figure()
        fig.update_layout(
            title="No data matching filters",
            xaxis_title="Right Ascension (deg)",
            yaxis_title="Declination (deg)",
        )
        return fig

    # Prepare hover text
    hover_template = (
        "<b>ID:</b> %{customdata[0]}<br>"
        "<b>Priority:</b> %{customdata[1]:.2f}<br>"
        "<b>Requested Hours:</b> %{customdata[2]:.2f}<br>"
        "<b>RA:</b> %{x:.2f}°<br>"
        "<b>Dec:</b> %{y:.2f}°<br>"
        "<b>Scheduled:</b> %{customdata[3]}<br>"
        "<extra></extra>"
    )

    customdata = df[
        [
            "schedulingBlockId",
            "priority",
            "requested_hours",
            "scheduled_flag",
        ]
    ].values

    # Determine color mapping
    if color_by == "scheduled_flag":
        color_discrete_map: dict[bool, str] = {True: "#1f77b4", False: "#ff7f0e"}
        color_column = df["scheduled_flag"].map({True: "Scheduled", False: "Unscheduled"})
    else:
        # Generic categorical coloring
        color_column = (
            df[color_by].fillna("Sin dato").astype(str)
            if color_by in df.columns
            else pd.Series(["Sin dato"] * len(df))
        )

    # Normalize size
    if size_by in df.columns:
        size_values = df[size_by].fillna(0)
    else:
        size_values = pd.Series([0] * len(df), index=df.index)

    max_size = size_values.max()
    if max_size > 0:
        size_normalized = 5 + (size_values / max_size) * 30
    else:
        size_normalized = pd.Series([10] * len(df), index=df.index)

    # Create scatter plot
    fig = go.Figure()

    if color_by == "scheduled_flag":
        # Split by scheduled status for better legend
        # Ensure color_discrete_map is a dict with bool keys
        color_map_bool: dict[bool, str] = color_discrete_map  # type: ignore[assignment]

        for status, status_label in [(True, "Scheduled"), (False, "Unscheduled")]:
            mask = df["scheduled_flag"] == status
            if mask.sum() == 0:
                continue

            fig.add_trace(
                go.Scatter(
                    x=df.loc[mask, "raInDeg"],
                    y=df.loc[mask, "decInDeg"],
                    mode="markers",
                    name=status_label,
                    marker=dict(
                        size=size_normalized[mask],
                        color=color_map_bool.get(status, "#999999"),
                        opacity=0.7,
                        line=dict(width=0.5, color="white"),
                    ),
                    customdata=customdata[mask],
                    hovertemplate=hover_template,
                )
            )
    else:
        categories = color_column.unique()
        categories_list: list[str] = [str(cat) for cat in categories]
        palette: dict[str, str] = category_palette or _default_palette(categories_list)

        for category in sorted(categories):
            mask = color_column == category
            if mask.sum() == 0:
                continue

            fig.add_trace(
                go.Scatter(
                    x=df.loc[mask, "raInDeg"],
                    y=df.loc[mask, "decInDeg"],
                    mode="markers",
                    name=category,
                    marker=dict(
                        size=size_normalized[mask],
                        color=palette.get(category, "#999"),
                        opacity=0.7,
                        line=dict(width=0.5, color="white"),
                    ),
                    customdata=customdata[mask],
                    hovertemplate=hover_template,
                )
            )

    # Update layout
    fig.update_layout(
        title=dict(
            text=f"Sky Map: Celestial Coordinates ({len(df)} observations)",
            x=0.5,
            xanchor="center",
        ),
        xaxis=dict(
            title="Right Ascension (degrees)",
            range=[360, 0] if flip_ra else [0, 360],
            showgrid=True,
            gridcolor="rgba(100, 100, 100, 0.3)",
        ),
        yaxis=dict(
            title="Declination (degrees)",
            range=[-90, 90],
            showgrid=True,
            gridcolor="rgba(100, 100, 100, 0.3)",
        ),
        height=PLOT_HEIGHT,
        margin=PLOT_MARGIN,
        hovermode="closest",
        plot_bgcolor="rgba(14, 17, 23, 0.3)",
        paper_bgcolor="rgba(0, 0, 0, 0)",
        legend=dict(
            orientation="v",
            yanchor="top",
            y=1,
            xanchor="left",
            x=1.02,
        ),
    )

    return fig


def _default_palette(categories: Sequence[str]) -> dict[str, str]:
    """Generate default colors for categorical values."""
    base_colors = [
        "#1f77b4",
        "#ff7f0e",
        "#2ca02c",
        "#d62728",
        "#9467bd",
        "#8c564b",
        "#e377c2",
        "#7f7f7f",
        "#bcbd22",
        "#17becf",
    ]
    palette = {}
    for idx, category in enumerate(categories):
        palette[category] = base_colors[idx % len(base_colors)]
    return palette
