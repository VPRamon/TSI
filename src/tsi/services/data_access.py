"""
ETL Data Access Layer

This module provides data access functions for the TSI application using
the ETL (analytics tables) database as the single source of truth.

All data is retrieved from pre-computed analytics tables which provide:
- Denormalized data for faster queries
- Pre-computed metrics and statistics
- Optimized indexes for common access patterns

Architecture:
-----------
┌─────────────────┐
│  Pages/UI       │
└────────┬────────┘
         │
         v
┌─────────────────┐
│  Data Access    │  ← This module: ETL-only data retrieval
│  (This Module)  │
└────────┬────────┘
         │
         v
┌─────────────────┐
│   ETL Tables    │  analytics.schedule_blocks_analytics, etc.
│  (Rust Backend) │
└─────────────────┘

Usage Example:
-------------
from tsi.services.data_access import get_sky_map_data

# Uses ETL analytics tables
data = get_sky_map_data(schedule_id=123)
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING, Any, cast

from tsi.exceptions import ServerError

if TYPE_CHECKING:
    from tsi_rust import (
        CompareData,
        DistributionData,
        InsightsData,
        ScheduleTimelineData,
        SkyMapData,
        TrendsData,
        VisibilityMapData,
    )

logger = logging.getLogger(__name__)


def _rust_call(method: str, *args: Any) -> Any:
    """
    Call a Rust backend function by name.

    Args:
        method: Name of the Rust function (e.g., "py_get_sky_map_data")
        *args: Arguments to pass to the Rust function

    Returns:
        Result from the Rust function

    Raises:
        ServerError: If the operation fails
    """
    try:
        import tsi_rust

        return getattr(tsi_rust, method)(*args)
    except ImportError as e:
        raise ServerError("Rust backend not available", details={"method": method}) from e
    except AttributeError as e:
        raise ServerError(
            f"Rust backend method '{method}' not found", details={"method": method}
        ) from e
    except Exception as e:
        raise ServerError(
            f"Rust backend call '{method}' failed: {str(e)}",
            details={"method": method, "args_count": len(args)},
        ) from e


# =============================================================================
# Sky Map Data
# =============================================================================


def get_sky_map_data(schedule_id: int) -> SkyMapData:
    """
    Get sky map data from ETL analytics tables.

    Retrieves pre-computed data from analytics.schedule_blocks_analytics with:
    - Denormalized target coordinates
    - Pre-computed priority bins
    - Pre-computed visibility metrics

    Args:
        schedule_id: Database ID of the schedule

    Returns:
        SkyMapData with computed bins and metadata

    Raises:
        ServerError: If analytics data is not available
    """
    logger.debug(f"Fetching sky map data (ETL) for schedule_id={schedule_id}")
    return cast("SkyMapData", _rust_call("get_sky_map_data", schedule_id))


# =============================================================================
# Distribution Data
# =============================================================================


def get_distribution_data(
    schedule_id: int,
) -> DistributionData:
    """
    Get distribution data from ETL analytics tables.

    Retrieves pre-computed block data with:
    - Pre-computed visibility statistics
    - Pre-computed priority metrics
    - Pre-computed elevation ranges

    Note:
        Impossible blocks (zero visibility) are automatically excluded during ETL.

    Args:
        schedule_id: Database ID of the schedule

    Returns:
        DistributionData with computed statistics

    Raises:
        ServerError: If analytics data is not available
    """
    logger.debug(f"Fetching distribution data (ETL) for schedule_id={schedule_id}")
    try:
        return cast(
            "DistributionData", _rust_call("get_distribution_data", schedule_id)
        )
    except ServerError as e:
        if "No analytics data available" in str(e):
            raise ServerError(
                f"No analytics data available for schedule {schedule_id}. "
                "Analytics tables may need to be populated. "
                "Run populate_schedule_analytics() or check ETL process.",
                details={"schedule_id": schedule_id},
            ) from e
        raise


# =============================================================================
# Timeline Data
# =============================================================================


def get_schedule_timeline_data(schedule_id: int) -> ScheduleTimelineData:
    """
    Get schedule timeline data.

    Retrieves scheduled blocks and their time periods from the database.

    Args:
        schedule_id: Database ID of the schedule

    Returns:
        ScheduleTimelineData with scheduled blocks
    """
    logger.debug(f"Fetching timeline data for schedule_id={schedule_id}")
    return cast("ScheduleTimelineData", _rust_call("py_get_schedule_timeline_data", schedule_id))


# =============================================================================
# Insights Data
# =============================================================================


def get_insights_data(
    schedule_id: int,
) -> InsightsData:
    """
    Get insights data with pre-computed analytics.

    Retrieves summary analytics from analytics.schedule_summary_analytics which
    are populated during schedule upload. Includes:
    - Block counts and scheduling rates
    - Priority statistics
    - Visibility statistics
    - Correlation metrics

    Note:
        Impossible blocks (zero visibility) are automatically excluded during ETL.
        To see validation issues, use get_validation_report_data().

    Args:
        schedule_id: Database ID of the schedule

    Returns:
        InsightsData with summary statistics and correlations
    """
    logger.debug(f"Fetching insights data for schedule_id={schedule_id}")
    return cast("InsightsData", _rust_call("py_get_insights_data", schedule_id))


# =============================================================================
# Trends Data
# =============================================================================


def get_trends_data(
    schedule_id: int,
    n_bins: int = 10,
    bandwidth: float = 0.3,
    n_smooth_points: int = 100,
) -> TrendsData:
    """
    Get trends data with computed empirical rates and smoothed curves.

    Retrieves pre-computed rate analytics from:
    - analytics.schedule_priority_rates
    - analytics.schedule_visibility_bins
    - analytics.schedule_heatmap_bins

    Note:
        Impossible blocks (zero visibility) are automatically excluded during ETL.

    Args:
        schedule_id: Database ID of the schedule
        n_bins: Number of bins for continuous variables
        bandwidth: Bandwidth for smoothing as fraction of range
        n_smooth_points: Number of points in smoothed curves

    Returns:
        TrendsData with empirical rates and smoothed curves
    """
    logger.debug(f"Fetching trends data for schedule_id={schedule_id}")
    return cast(
        "TrendsData",
        _rust_call(
            "py_get_trends_data",
            schedule_id,
            n_bins,
            bandwidth,
            n_smooth_points,
        ),
    )


# =============================================================================
# Compare Data
# =============================================================================


def get_compare_data(
    current_schedule_id: int,
    comparison_schedule_id: int,
    current_name: str,
    comparison_name: str,
) -> CompareData:
    """
    Get comparison data for two schedules.

    Loads block lists from both schedules and computes comparison statistics.

    Args:
        current_schedule_id: Database ID of the current schedule
        comparison_schedule_id: Database ID of the comparison schedule
        current_name: Display name for current schedule
        comparison_name: Display name for comparison schedule

    Returns:
        CompareData with comparison statistics
    """
    logger.debug(
        f"Fetching compare data for schedules {current_schedule_id} vs {comparison_schedule_id}"
    )
    return cast(
        "CompareData",
        _rust_call(
            "py_get_compare_data",
            current_schedule_id,
            comparison_schedule_id,
            current_name,
            comparison_name,
        ),
    )


# =============================================================================
# Visibility Map Data
# =============================================================================


def get_visibility_map_data(schedule_id: int) -> VisibilityMapData:
    """
    Get visibility map data with block summaries.

    Retrieves visibility period information for all blocks in a schedule.

    Args:
        schedule_id: Database ID of the schedule

    Returns:
        VisibilityMapData with block summaries
    """
    logger.debug(f"Fetching visibility map data for schedule_id={schedule_id}")
    return cast("VisibilityMapData", _rust_call("py_get_visibility_map_data", schedule_id))


__all__ = [
    "get_sky_map_data",
    "get_distribution_data",
    "get_schedule_timeline_data",
    "get_insights_data",
    "get_trends_data",
    "get_compare_data",
    "get_visibility_map_data",
]
