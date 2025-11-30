"""Sky map plotting functionality."""

from collections.abc import Sequence
from typing import Any

import plotly.graph_objects as go
import streamlit as st

from tsi.config import PLOT_HEIGHT, PLOT_MARGIN, CACHE_TTL


@st.cache_data(show_spinner=False, ttl=CACHE_TTL)
def build_figure(
    _blocks: list[Any],
    color_by: str = "priority_bin",
    size_by: str = "requested_hours",
    flip_ra: bool = True,
    category_palette: dict | None = None,
    cache_key: str | None = None,
) -> go.Figure:
    """
    Build an interactive sky map scatter plot with RA and Dec.

    Args:
        _blocks: List of SchedulingBlock PyO3 objects (not hashed for caching)
        color_by: Column to use for color coding ("priority_bin" or "scheduled_flag")
        size_by: Column to use for marker size
        flip_ra: If True, reverse RA axis (astronomy convention)
        category_palette: Optional dict mapping category -> color
        cache_key: Optional key for cache invalidation based on filtered data

    Returns:
        Plotly Figure object
    """
    blocks = list(_blocks)

    if len(blocks) == 0:
        # Return empty figure
        fig = go.Figure()
        fig.update_layout(
            title="No data matching filters",
            xaxis_title="Right Ascension (deg)",
            yaxis_title="Declination (deg)",
        )
        return fig

    # Extract data from blocks
    block_ids = []
    priorities = []
    requested_hours = []
    ra_values = []
    dec_values = []
    scheduled_flags = []
    priority_bins = []

    for block in blocks:
        block_id = getattr(block.id, "value", None)
        scheduling_block_id = int(block_id) if block_id is not None else None
        block_ids.append(scheduling_block_id)

        priorities.append(float(block.priority))
        requested_hours.append(float(block.requested_duration_seconds) / 3600.0)
        ra_values.append(float(block.target_ra_deg))
        dec_values.append(float(block.target_dec_deg))
        
        scheduled_period = getattr(block, "scheduled_period", None)
        is_scheduled = scheduled_period is not None
        scheduled_flags.append(is_scheduled)
        
        priority_bin = str(getattr(block, "priority_bin", ""))
        priority_bins.append(priority_bin)

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

    customdata = list(zip(block_ids, priorities, requested_hours, scheduled_flags))

    # Normalize size
    max_size = max(requested_hours) if requested_hours else 0
    if max_size > 0:
        size_normalized = [5 + (h / max_size) * 30 for h in requested_hours]
    else:
        size_normalized = [10] * len(blocks)

    # Create scatter plot
    fig = go.Figure()

    if color_by == "scheduled_flag":
        # Split by scheduled status for better legend
        color_discrete_map: dict[bool, str] = {True: "#1f77b4", False: "#ff7f0e"}

        for status, status_label in [(True, "Scheduled"), (False, "Unscheduled")]:
            indices = [i for i, flag in enumerate(scheduled_flags) if flag == status]
            if not indices:
                continue

            x_data = [ra_values[i] for i in indices]
            y_data = [dec_values[i] for i in indices]
            sizes = [size_normalized[i] for i in indices]
            custom = [customdata[i] for i in indices]

            fig.add_trace(
                go.Scatter(
                    x=x_data,
                    y=y_data,
                    mode="markers",
                    name=status_label,
                    marker=dict(
                        size=sizes,
                        color=color_discrete_map.get(status, "#999999"),
                        opacity=0.7,
                        line=dict(width=0.5, color="white"),
                    ),
                    customdata=custom,
                    hovertemplate=hover_template,
                )
            )
    else:
        # Group by priority bin
        unique_bins = sorted(set(priority_bins))
        palette: dict[str, str] = category_palette or _default_palette(unique_bins)

        for bin_value in unique_bins:
            indices = [i for i, pb in enumerate(priority_bins) if pb == bin_value]
            if not indices:
                continue

            x_data = [ra_values[i] for i in indices]
            y_data = [dec_values[i] for i in indices]
            sizes = [size_normalized[i] for i in indices]
            custom = [customdata[i] for i in indices]

            fig.add_trace(
                go.Scatter(
                    x=x_data,
                    y=y_data,
                    mode="markers",
                    name=bin_value,
                    marker=dict(
                        size=sizes,
                        color=palette.get(bin_value, "#999"),
                        opacity=0.7,
                        line=dict(width=0.5, color="white"),
                    ),
                    customdata=custom,
                    hovertemplate=hover_template,
                )
            )

    # Update layout
    fig.update_layout(
        title=dict(
            text=f"Sky Map: Celestial Coordinates ({len(blocks)} observations)",
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
