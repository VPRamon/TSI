"""Sky map data helpers that pull scheduling blocks from the Rust backend."""

from __future__ import annotations

from typing import TYPE_CHECKING

from tsi.services.database import get_sky_map_data


if TYPE_CHECKING:
    from tsi_rust import SkyMapData


def load_sky_map_data(
    *,
    schedule_id: int | None = None,
    schedule_name: str | None = None,
) -> SkyMapData:
    """
    Fetch complete sky map data with all processing done in Rust.
    
    This function returns a SkyMapData object from the Rust backend containing:
    - All blocks with computed priority bins
    - Priority bin metadata (4 bins with ranges and colors)
    - Statistics (min/max for priority, RA, Dec, time)
    - Counts (total blocks, scheduled blocks)
    
    The Rust backend handles:
    - Database queries (optimized to fetch only needed fields)
    - Priority bin computation (4 bins proportional to min/max)
    - All statistical calculations
    
    The frontend just needs to:
    - Filter the blocks if needed (by status, time window, etc.)
    - Create the plots using the provided data and colors

    Args:
        schedule_id: Numeric identifier of the schedule in the database.
        schedule_name: Optional schedule name (used when ID is unavailable).

    Returns:
        SkyMapData object from Rust with all computed data.
    """
    return get_sky_map_data(schedule_id=schedule_id, schedule_name=schedule_name)


__all__ = ["load_sky_map_data"]
