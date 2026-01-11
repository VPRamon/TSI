"""Sky Map data filtering and processing services."""

from __future__ import annotations

from collections.abc import Sequence
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from tsi_rust import LightweightBlock, Period


def filter_blocks(
    blocks: list[LightweightBlock],
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
        priority = block.priority
        is_scheduled = block.scheduled_period is not None
        priority_bin = block.priority_bin

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
            if not schedule_window.contains_mjd(block.scheduled_period.start_mjd):
                continue

        filtered.append(block)

    return filtered


__all__ = [
    "filter_blocks",
]
