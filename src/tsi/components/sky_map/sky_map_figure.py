"""Sky Map figure rendering helper component."""

from __future__ import annotations

from typing import Any

import streamlit as st

from tsi.plots.sky_map import build_figure


def _compute_cache_key(blocks: Any, controls: dict[str, Any]) -> str:
    """Compute a cache key representing the filtered blocks.

    Args:
        blocks: Filtered list of blocks
        controls: Filter controls dictionary

    Returns:
        A string cache key that changes when filters change
    """
    # Use block count and first/last block IDs to create a simple but effective cache key
    if not blocks:
        return "empty"

    block_ids = []
    for block in blocks:
        block_id = getattr(block.original_block_id, "value", None)
        if block_id is not None:
            block_ids.append(int(block_id))

    if not block_ids:
        return f"count_{len(blocks)}"

    # Create a key from count, min/max IDs, and filter values
    key_parts = [
        f"n={len(blocks)}",
        f"ids={min(block_ids)}-{max(block_ids)}",
        f"sched={controls.get('scheduled_filter', 'All')}",
    ]

    # Add priority range if present
    if "priority_range" in controls:
        pr = controls["priority_range"]
        key_parts.append(f"pri={pr[0]:.2f}-{pr[1]:.2f}")

    return "_".join(key_parts)


def render_sky_map_figure(
    blocks: Any,
    controls: dict[str, Any],
    priority_bins: Any,
    key: str = "sky_map_chart",
) -> object:
    """Build and render the sky map Plotly figure.

    This function constructs the optional category palette (when coloring by
    `priority_bin`), builds the figure with `build_figure`, displays it via
    Streamlit and returns the figure object.

    Args:
        blocks: Iterable of block objects to plot.
        controls: Dict of controls returned by the sidebar controls component.
        priority_bins: Priority bin definitions (used to map labels to colors).
        key: Streamlit chart key.

    Returns:
        The Plotly figure object produced by `build_figure`.
    """

    category_palette = None
    if controls.get("color_column") == "priority_bin":
        category_palette = {bin_info.label: bin_info.color for bin_info in priority_bins}

    # Create a cache key based on the filtered blocks to ensure cache invalidation
    # when filters change
    cache_key = _compute_cache_key(blocks, controls)

    fig = build_figure(
        _blocks=blocks,
        color_by=controls.get("color_column"),
        size_by="requested_hours",
        flip_ra=controls.get("flip_ra", False),
        category_palette=category_palette,
        cache_key=cache_key,
    )

    st.plotly_chart(fig, width="stretch", key=key)

    return fig
