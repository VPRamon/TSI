"""Sky map plotting functionality."""

from collections.abc import Sequence
from typing import Any

import plotly.graph_objects as go
import streamlit as st

from tsi.config import CACHE_TTL, PLOT_HEIGHT, PLOT_MARGIN
from tsi.plots.plot_theme import (
    SCHEDULED_COLOR,
    UNSCHEDULED_COLOR,
    PlotTheme,
    apply_theme,
    get_scatter_marker,
)


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
        # Return empty figure with standard theme
        fig = go.Figure()
        apply_theme(fig, show_legend=False)
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
        block_ids.append(block.original_block_id)

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
        color_discrete_map: dict[bool, str] = {True: SCHEDULED_COLOR, False: UNSCHEDULED_COLOR}

        for status, status_label in [(True, "Scheduled"), (False, "Unscheduled")]:
            indices = [i for i, flag in enumerate(scheduled_flags) if flag == status]
            if not indices:
                continue

            x_data = [ra_values[i] for i in indices]
            y_data = [dec_values[i] for i in indices]
            sizes = [size_normalized[i] for i in indices]
            custom = [customdata[i] for i in indices]

            marker_config = get_scatter_marker(
                size=sizes,
                color=color_discrete_map.get(status, PlotTheme.GRAY),
            )

            fig.add_trace(
                go.Scatter(
                    x=x_data,
                    y=y_data,
                    mode="markers",
                    name=status_label,
                    marker=marker_config,
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

            marker_config = get_scatter_marker(
                size=sizes,
                color=palette.get(bin_value, PlotTheme.GRAY),
            )

            fig.add_trace(
                go.Scatter(
                    x=x_data,
                    y=y_data,
                    mode="markers",
                    name=bin_value,
                    marker=marker_config,
                    customdata=custom,
                    hovertemplate=hover_template,
                )
            )

    # Apply standard theme with vertical legend for sky map
    apply_theme(fig, height=PLOT_HEIGHT, margin=PLOT_MARGIN, legend_style="vertical")

    # Update layout with sky map specific settings
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
            gridcolor=PlotTheme.GRID_COLOR,
        ),
        yaxis=dict(
            title="Declination (degrees)",
            range=[-90, 90],
            showgrid=True,
            gridcolor=PlotTheme.GRID_COLOR,
        ),
        hovermode="closest",
    )

    return fig


def _default_palette(categories: Sequence[str]) -> dict[str, str]:
    """Generate default colors for categorical values."""
    return {
        category: PlotTheme.COLOR_SEQUENCE[idx % len(PlotTheme.COLOR_SEQUENCE)]
        for idx, category in enumerate(categories)
    }
