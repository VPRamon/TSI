"""
Data Source Orchestration Layer

This module provides unified data access functions that route between legacy 
(normalized tables) and ETL (analytics tables) data sources based on configuration.

The data_source configuration flag (in app_config.settings) controls which path is used:
- "legacy": Uses normalized database tables with complex joins (current production)
- "etl": Uses pre-computed analytics tables (new optimized path)

This enables zero-downtime migration by keeping both paths available during transition.

Architecture:
-----------
┌─────────────────┐
│  Pages/UI       │
└────────┬────────┘
         │
         v
┌─────────────────┐
│  Orchestrators  │  ← This module: Route based on config
│  (This Module)  │
└────────┬────────┘
         │
    ┌────┴────┐
    v         v
┌───────┐  ┌─────┐
│Legacy │  │ ETL │
│ Path  │  │Path │
└───────┘  └─────┘

Usage Example:
-------------
from tsi.services.data_source import get_sky_map_data_unified

# Automatically uses correct path based on DATA_SOURCE config
data = get_sky_map_data_unified(schedule_id=123)
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING, Any, cast

from app_config import get_settings
from tsi.exceptions import DatabaseQueryError

if TYPE_CHECKING:
    from tsi_rust import (
        SkyMapData,
        DistributionData,
        ScheduleTimelineData,
        InsightsData,
        TrendsData,
        CompareData,
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
        DatabaseQueryError: If the operation fails
    """
    try:
        import tsi_rust
        return getattr(tsi_rust, method)(*args)
    except ImportError as e:
        raise DatabaseQueryError(
            "Rust backend not available",
            details={"method": method}
        ) from e
    except AttributeError as e:
        raise DatabaseQueryError(
            f"Rust backend method '{method}' not found",
            details={"method": method}
        ) from e
    except Exception as e:
        raise DatabaseQueryError(
            f"Rust backend call '{method}' failed: {str(e)}",
            details={"method": method, "args_count": len(args)}
        ) from e


# =============================================================================
# Sky Map Data
# =============================================================================

def get_sky_map_data_legacy(schedule_id: int) -> SkyMapData:
    """
    Get sky map data using legacy normalized tables.
    
    This path uses complex joins across multiple tables:
    - schedule_scheduling_blocks
    - scheduling_blocks
    - targets
    - constraints
    
    Args:
        schedule_id: Database ID of the schedule
        
    Returns:
        SkyMapData with computed bins and metadata
    """
    logger.debug(f"Fetching sky map data (legacy path) for schedule_id={schedule_id}")
    return cast("SkyMapData", _rust_call("py_get_sky_map_data_legacy", schedule_id))


def get_sky_map_data_etl(schedule_id: int) -> SkyMapData:
    """
    Get sky map data using ETL analytics table.
    
    This path uses the pre-computed analytics.schedule_blocks_analytics table
    which has denormalized data and pre-computed metrics.
    
    Args:
        schedule_id: Database ID of the schedule
        
    Returns:
        SkyMapData with computed bins and metadata
        
    Raises:
        DatabaseQueryError: If analytics data is not available
    """
    logger.debug(f"Fetching sky map data (ETL path) for schedule_id={schedule_id}")
    return cast("SkyMapData", _rust_call("py_get_sky_map_data_analytics", schedule_id))


def get_sky_map_data_unified(schedule_id: int) -> SkyMapData:
    """
    Get sky map data using the configured data source.
    
    This is the public API that pages should use. It automatically routes to
    the appropriate implementation based on the DATA_SOURCE configuration.
    
    Args:
        schedule_id: Database ID of the schedule
        
    Returns:
        SkyMapData with computed bins and metadata
    """
    settings = get_settings()
    data_source = settings.data_source
    
    logger.debug(f"get_sky_map_data_unified: Using data_source={data_source}")
    
    if data_source == "etl":
        try:
            return get_sky_map_data_etl(schedule_id)
        except DatabaseQueryError as e:
            logger.warning(
                f"ETL path failed for sky map (schedule_id={schedule_id}): {e}. "
                "This may indicate missing analytics data."
            )
            raise
    else:  # legacy
        return get_sky_map_data_legacy(schedule_id)


# =============================================================================
# Distribution Data
# =============================================================================

def get_distribution_data_legacy(
    schedule_id: int,
    filter_impossible: bool = False,
) -> DistributionData:
    """
    Get distribution data using legacy normalized tables.
    
    Args:
        schedule_id: Database ID of the schedule
        filter_impossible: If True, exclude blocks with zero visibility
        
    Returns:
        DistributionData with computed statistics
    """
    logger.debug(f"Fetching distribution data (legacy path) for schedule_id={schedule_id}")
    return cast(
        "DistributionData",
        _rust_call("py_get_distribution_data_legacy", schedule_id, filter_impossible)
    )


def get_distribution_data_etl(
    schedule_id: int,
    filter_impossible: bool = False,
) -> DistributionData:
    """
    Get distribution data using ETL analytics table.
    
    Args:
        schedule_id: Database ID of the schedule
        filter_impossible: If True, exclude blocks with zero visibility
        
    Returns:
        DistributionData with computed statistics
        
    Raises:
        DatabaseQueryError: If analytics data is not available
    """
    logger.debug(f"Fetching distribution data (ETL path) for schedule_id={schedule_id}")
    return cast(
        "DistributionData",
        _rust_call("py_get_distribution_data_analytics", schedule_id, filter_impossible)
    )


def get_distribution_data_unified(
    schedule_id: int,
    filter_impossible: bool = False,
) -> DistributionData:
    """
    Get distribution data using the configured data source.
    
    This is the public API that pages should use.
    
    Args:
        schedule_id: Database ID of the schedule
        filter_impossible: If True, exclude blocks with zero visibility
        
    Returns:
        DistributionData with computed statistics
    """
    settings = get_settings()
    data_source = settings.data_source
    
    logger.debug(f"get_distribution_data_unified: Using data_source={data_source}")
    
    if data_source == "etl":
        try:
            return get_distribution_data_etl(schedule_id, filter_impossible)
        except DatabaseQueryError as e:
            logger.warning(
                f"ETL path failed for distributions (schedule_id={schedule_id}): {e}. "
                "This may indicate missing analytics data."
            )
            raise
    else:  # legacy
        return get_distribution_data_legacy(schedule_id, filter_impossible)


# =============================================================================
# Timeline Data
# =============================================================================

def get_schedule_timeline_data_unified(schedule_id: int) -> ScheduleTimelineData:
    """
    Get schedule timeline data.
    
    Note: Timeline currently uses the same implementation for both legacy and ETL,
    as it primarily queries scheduled periods which are the same in both paths.
    
    Args:
        schedule_id: Database ID of the schedule
        
    Returns:
        ScheduleTimelineData with scheduled blocks
    """
    settings = get_settings()
    logger.debug(f"get_schedule_timeline_data_unified: Using data_source={settings.data_source}")
    
    # Currently both paths use the same implementation
    return cast("ScheduleTimelineData", _rust_call("py_get_schedule_timeline_data", schedule_id))


# =============================================================================
# Insights Data
# =============================================================================

def get_insights_data_unified(
    schedule_id: int,
    filter_impossible: bool = False,
) -> InsightsData:
    """
    Get insights data with pre-computed analytics.
    
    Note: Insights uses pre-computed summary analytics which are populated
    during schedule upload regardless of the data_source setting. Both paths
    currently use the same implementation.
    
    Args:
        schedule_id: Database ID of the schedule
        filter_impossible: If True, exclude impossible blocks from metrics
        
    Returns:
        InsightsData with summary statistics and correlations
    """
    settings = get_settings()
    logger.debug(f"get_insights_data_unified: Using data_source={settings.data_source}")
    
    # Currently both paths use the same implementation
    return cast("InsightsData", _rust_call("py_get_insights_data", schedule_id, filter_impossible))


# =============================================================================
# Trends Data
# =============================================================================

def get_trends_data_unified(
    schedule_id: int,
    filter_impossible: bool = False,
    n_bins: int = 10,
    bandwidth: float = 0.3,
    n_smooth_points: int = 100,
) -> TrendsData:
    """
    Get trends data with computed empirical rates and smoothed curves.
    
    Note: Trends uses pre-computed rate analytics which are populated during
    schedule upload regardless of the data_source setting. Both paths currently
    use the same implementation.
    
    Args:
        schedule_id: Database ID of the schedule
        filter_impossible: If True, exclude blocks with zero visibility
        n_bins: Number of bins for continuous variables
        bandwidth: Bandwidth for smoothing as fraction of range
        n_smooth_points: Number of points in smoothed curves
        
    Returns:
        TrendsData with empirical rates and smoothed curves
    """
    settings = get_settings()
    logger.debug(f"get_trends_data_unified: Using data_source={settings.data_source}")
    
    # Currently both paths use the same implementation
    return cast(
        "TrendsData",
        _rust_call(
            "py_get_trends_data",
            schedule_id,
            filter_impossible,
            n_bins,
            bandwidth,
            n_smooth_points,
        )
    )


# =============================================================================
# Compare Data
# =============================================================================

def get_compare_data_unified(
    current_schedule_id: int,
    comparison_schedule_id: int,
    current_name: str,
    comparison_name: str,
) -> CompareData:
    """
    Get comparison data for two schedules.
    
    Note: Compare currently uses the same implementation for both legacy and ETL,
    as it needs to load full block lists from both schedules regardless of source.
    
    Args:
        current_schedule_id: Database ID of the current schedule
        comparison_schedule_id: Database ID of the comparison schedule
        current_name: Display name for current schedule
        comparison_name: Display name for comparison schedule
        
    Returns:
        CompareData with comparison statistics
    """
    settings = get_settings()
    logger.debug(f"get_compare_data_unified: Using data_source={settings.data_source}")
    
    # Currently both paths use the same implementation
    return cast(
        "CompareData",
        _rust_call(
            "py_get_compare_data",
            current_schedule_id,
            comparison_schedule_id,
            current_name,
            comparison_name,
        )
    )


# =============================================================================
# Visibility Map Data
# =============================================================================

def get_visibility_map_data_unified(schedule_id: int) -> VisibilityMapData:
    """
    Get visibility map data with block summaries.
    
    Note: Visibility map currently uses the same implementation for both legacy
    and ETL, as it needs visibility period counts which are available in both.
    
    Args:
        schedule_id: Database ID of the schedule
        
    Returns:
        VisibilityMapData with block summaries
    """
    settings = get_settings()
    logger.debug(f"get_visibility_map_data_unified: Using data_source={settings.data_source}")
    
    # Currently both paths use the same implementation
    return cast("VisibilityMapData", _rust_call("py_get_visibility_map_data", schedule_id))


__all__ = [
    # Sky Map
    "get_sky_map_data_unified",
    "get_sky_map_data_legacy",
    "get_sky_map_data_etl",
    # Distributions
    "get_distribution_data_unified",
    "get_distribution_data_legacy",
    "get_distribution_data_etl",
    # Timeline
    "get_schedule_timeline_data_unified",
    # Insights
    "get_insights_data_unified",
    # Trends
    "get_trends_data_unified",
    # Compare
    "get_compare_data_unified",
    # Visibility Map
    "get_visibility_map_data_unified",
]
