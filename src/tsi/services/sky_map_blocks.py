"""Helper functions for working with scheduling blocks in sky map context."""

from __future__ import annotations

from typing import Any


def get_priority_range_from_blocks(blocks: list[Any]) -> tuple[float, float]:
    """
    Calculate the priority range from scheduling blocks.

    Args:
        blocks: List of SchedulingBlock PyO3 objects

    Returns:
        Tuple of (min_priority, max_priority)
    """
    if not blocks:
        return 0.0, 10.0

    priorities = [float(block.priority) for block in blocks]

    if not priorities:
        return 0.0, 10.0

    priority_min = min(priorities)
    priority_max = max(priorities)

    if priority_min == priority_max:
        priority_max = priority_min + 1.0

    return priority_min, priority_max


def prepare_priority_bins_from_blocks(blocks: list[Any]) -> tuple[list[Any], list[str]]:
    """
    Prepare priority bins from blocks.

    Args:
        blocks: List of SchedulingBlock PyO3 objects

    Returns:
        Tuple of (blocks, list of priority bins)
    """
    # Extract unique priority bins from blocks
    priority_bins = set()
    for block in blocks:
        bin_value = getattr(block, "priority_bin", None)
        if bin_value:
            priority_bins.add(str(bin_value))

    return blocks, sorted(list(priority_bins))


def get_scheduled_time_range(blocks: list[Any]) -> tuple[float | None, float | None]:
    """
    Get the min and max scheduled times from blocks.

    Args:
        blocks: List of SchedulingBlock PyO3 objects

    Returns:
        Tuple of (min_mjd, max_mjd) or (None, None) if no scheduled blocks
    """
    scheduled_times = []
    for block in blocks:
        scheduled_period = getattr(block, "scheduled_period", None)
        if scheduled_period:
            start_mjd = getattr(scheduled_period, "start_mjd", None)
            if start_mjd is not None:
                scheduled_times.append(float(start_mjd))

    if not scheduled_times:
        return None, None

    return min(scheduled_times), max(scheduled_times)


__all__ = [
    "get_priority_range_from_blocks",
    "prepare_priority_bins_from_blocks",
    "get_scheduled_time_range",
]
