"""Helper functions for working with scheduling blocks in sky map context."""

from __future__ import annotations

from typing import Any

import pandas as pd


def get_priority_range(source: pd.DataFrame | list[Any]) -> tuple[float, float]:
    """
    Calculate the priority range from a DataFrame or list of scheduling blocks.

    Handles edge cases:
    - Missing priority column (DataFrame) → returns (0.0, 10.0)
    - Empty priority values → returns (0.0, 10.0)
    - Single priority value → returns (value, value + 1.0)

    Args:
        source: Either a pandas DataFrame with a 'priority' column,
                or a list of SchedulingBlock PyO3 objects

    Returns:
        Tuple of (min_priority, max_priority)

    Examples:
        >>> # From DataFrame
        >>> df = pd.DataFrame({'priority': [1.0, 5.0, 10.0]})
        >>> get_priority_range(df)
        (1.0, 10.0)

        >>> # From blocks list
        >>> blocks = [block1, block2, block3]  # PyO3 objects
        >>> get_priority_range(blocks)
        (1.0, 10.0)
    """
    priorities = []

    if isinstance(source, pd.DataFrame):
        # DataFrame path
        if "priority" not in source.columns:
            return 0.0, 10.0
        priorities = source["priority"].dropna().tolist()
    elif isinstance(source, list):
        # Blocks list path
        if not source:
            return 0.0, 10.0
        priorities = [float(block.priority) for block in source]
    else:
        # Unknown type, return default
        return 0.0, 10.0

    if not priorities:
        return 0.0, 10.0

    priority_min = float(min(priorities))
    priority_max = float(max(priorities))

    if priority_min == priority_max:
        priority_max = priority_min + 1.0

    return priority_min, priority_max


def get_priority_range_from_blocks(blocks: list[Any]) -> tuple[float, float]:
    """
    Calculate the priority range from scheduling blocks.

    **DEPRECATED**: Use `get_priority_range(blocks)` instead.
    This function is kept for backward compatibility.

    Args:
        blocks: List of SchedulingBlock PyO3 objects

    Returns:
        Tuple of (min_priority, max_priority)
    """
    return get_priority_range(blocks)


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
    "get_priority_range",
    "get_priority_range_from_blocks",  # Deprecated, use get_priority_range
    "prepare_priority_bins_from_blocks",
    "get_scheduled_time_range",
]
