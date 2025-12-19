"""Visibility schedule data processing services."""

from __future__ import annotations

from collections.abc import Iterable, Sequence
from typing import Any


def get_all_block_ids(blocks: Sequence[Any]) -> list[str]:
    """
    Get sorted list of all block IDs from backend blocks.

    Args:
        blocks: Sequence of VisibilityBlockSummary-like objects

    Returns:
        Sorted list of block IDs (as strings, which are the original IDs from JSON)
    """
    return sorted({str(getattr(block, "scheduling_block_id")) for block in blocks})


def compute_effective_priority_range(
    sidebar_range: tuple[float, float],
    settings_range: tuple[float, float],
) -> tuple[float, float]:
    """
    Compute the effective priority range by combining sidebar and settings filters.

    Takes the more restrictive range (intersection of both ranges).

    Args:
        sidebar_range: Priority range from sidebar
        settings_range: Priority range from histogram settings

    Returns:
        Effective priority range (min, max)
    """
    effective_min = max(sidebar_range[0], settings_range[0])
    effective_max = min(sidebar_range[1], settings_range[1])
    return (effective_min, effective_max)


def filter_visibility_blocks(
    blocks: Iterable[Any],
    *,
    priority_range: tuple[float, float],
    block_ids: list[str] | None = None,
) -> list[Any]:
    """
    Filter visibility blocks by priority range and optional block IDs.

    Args:
        blocks: VisibilityBlockSummary objects
        priority_range: Inclusive priority range (min, max)
        block_ids: Optional list of block IDs (strings) to include

    Returns:
        Filtered list of blocks
    """
    min_priority, max_priority = priority_range
    allowed_ids = {str(bid) for bid in block_ids} if block_ids else None

    def _matches(block: Any) -> bool:
        priority = float(getattr(block, "priority"))
        block_id = str(getattr(block, "scheduling_block_id"))
        if priority < min_priority or priority > max_priority:
            return False
        if allowed_ids is not None and block_id not in allowed_ids:
            return False
        return True

    return [block for block in blocks if _matches(block)]
