"""Sky Map data filtering and processing services."""

from __future__ import annotations

from collections.abc import Sequence
from typing import Any, TYPE_CHECKING

from tsi.services.time_utils import ModifiedJulianDate

if TYPE_CHECKING:
    from tsi_rust import Period, SkyMapBlock

def filter_blocks(
    blocks: list[SkyMapBlock],
    priority_range: tuple[float, float],
    scheduled_filter: str,
    selected_bins: Sequence[str],
    schedule_window: Period | None,
) -> list[Any]:
    """
    Apply priority, bin, scheduled status and time filters to blocks.

    Args:
        blocks: List of SchedulingBlock PyO3 objects
        priority_range: Tuple of (min_priority, max_priority)
        scheduled_filter: One of "All", "Scheduled", "Unscheduled"
        selected_bins: Sequence of priority bins to include
        schedule_window: Optional Period restricting scheduled start times

    Returns:
        Filtered list of blocks
    """
    filtered = []

    for block in blocks:
        priority = float(block.priority)
        is_scheduled = block.scheduled_period is not None
        priority_bin = str(getattr(block, "priority_bin", ""))

        # Priority range filter
        if not (priority_range[0] <= priority <= priority_range[1]):
            continue

        # Priority bin filter
        if selected_bins and priority_bin not in selected_bins:
            continue

        # Scheduled status filter
        if scheduled_filter == "Scheduled" and not is_scheduled:
            continue
        elif scheduled_filter == "Unscheduled" and is_scheduled:
            continue

        # Time window filter
        if schedule_window and is_scheduled and block.scheduled_period:
            start_mjd = ModifiedJulianDate(block.scheduled_period.start_mjd)
            if not schedule_window.contains_mjd(float(start_mjd)):
                if scheduled_filter != "All":
                    continue

        filtered.append(block)

    return filtered


def build_palette(labels: Sequence[str]) -> dict:
    """
    Generate a simple color palette for categorical bins.

    Args:
        labels: Sequence of category labels

    Returns:
        Dictionary mapping labels to colors
    """
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
    for idx, label in enumerate(labels):
        palette[label] = base_colors[idx % len(base_colors)]
    return palette


__all__ = [
    "filter_blocks",
    "build_palette",
]
