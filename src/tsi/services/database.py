"""
Database Operations - Rust Backend Integration Layer

This module provides Python-callable wrappers for database operations implemented
in the Rust backend (rust_backend/src/db/).

## Architecture

### Rust Backend (tiberius + bb8)
All database operations use the high-performance Rust backend:
- Location: `rust_backend/src/db/`
- Driver: tiberius (pure Rust async TDS client)
- Connection Pool: bb8 (async connection pooling)
- Performance: 10-100x faster than Python/pyodbc
- Features: Type-safe operations, async/await, efficient resource management

### Database Configuration
**Database configuration is owned entirely by the Rust backend.**

The Rust backend reads configuration directly from environment variables:
- `DB_SERVER`: Database server address
- `DB_DATABASE`: Database name
- `DB_USERNAME`: Database username
- `DB_PASSWORD`: Database password
- `DB_PORT`: Database port (default: 1433)
- `DB_TRUST_CERT`: Trust server certificate (default: true)
- `DB_AUTH_METHOD`: Authentication method (sql_password | aad_password | aad_token)

For Azure AD authentication:
- `AZURE_TENANT_ID`: Azure tenant ID (default: common)
- `AZURE_CLIENT_ID`: Azure client ID
- `AZURE_ACCESS_TOKEN`: Direct access token (if using aad_token method)

See `rust_backend/src/db/config.rs` for complete configuration options.

## Usage Patterns

### Standard Operations
```python
from tsi.services.database import store_schedule_db

# Store schedule (Rust backend handles connection pooling automatically)
result = store_schedule_db(name, schedule_json, visibility_json)
```

## API Functions
- `store_schedule_db()`: Store schedule data
- `list_schedules_db()`: List all schedules
- `get_sky_map_data()`, `get_distribution_data()`, etc.: Page-specific data aggregation

See individual function docstrings for details.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING, Any

from numpy import int64
import pandas as pd

from app_config import get_settings
from tsi.error_handling import log_error, with_retry
from tsi.exceptions import (
    ServerError,
)

if TYPE_CHECKING:
    from tsi_rust import SkyMapData, VisibilityMapData, ValidationReport, ScheduleInfo
import tsi_rust as api

logger = logging.getLogger(__name__)


def _import_rust() -> Any:
    """
    Import the Rust backend module.

    Raises:
        ServerError: If tsi_rust module is not compiled/available
    """
    settings = get_settings()

    if not settings.enable_rust_backend:
        raise ServerError(
            "Rust backend is disabled in configuration", details={"enable_rust_backend": False}
        )

    try:
        import tsi_rust  # type: ignore[import-not-found]
    except ImportError as e:
        raise ServerError(
            "Rust backend is not available. Please compile the extension before using database features.",
            details={"install_command": "maturin develop --release"},
        ) from e
    return tsi_rust


def _rust_call(method: str, *args: Any) -> Any:
    """
    Call a Rust backend function by name with error handling.

    Args:
        method: Name of the Rust function (e.g., "py_list_schedules")
        *args: Arguments to pass to the Rust function

    Returns:
        Result from the Rust function

    Raises:
        ServerError: If Rust backend cannot be imported or operation fails
    """
    try:
        rust = _import_rust()
        return getattr(rust, method)(*args)
    except ServerError:
        # Re-raise server errors as-is
        raise
    except AttributeError as e:
        raise ServerError(
            f"Rust backend method '{method}' not found", details={"method": method}
        ) from e
    except Exception as e:
        # Log and re-raise other errors with context
        log_error(
            e,
            f"Rust backend call '{method}' failed",
            extra={"method": method, "args_count": len(args)},
        )
        raise


# Note: Connection pool initialization is handled automatically by the Rust backend
# on first database operation. No explicit initialization is needed from Python.


@with_retry(max_attempts=3, backoff_factor=1.5)
def store_schedule_db(
    schedule_name: str,
    schedule_json: str,
    visibility_json: str | None = None,
) -> int64:
    """
    Store a preprocessed schedule in the database.

    Backend: Rust (tiberius)

    Args:
        schedule_name: Human-readable schedule name
        schedule_json: JSON string containing schedule data
        visibility_json: Optional JSON string with visibility periods
        populate_analytics: If True, compute block and summary analytics (recommended)
        skip_time_bins: If True, skip expensive visibility time bin computation (default: True for performance)

    Returns:
        Dictionary with storage results including schedule_id

    Raises:
        ServerError: If storage operation fails

    Performance:
        - Fast mode (default): ~10-30 seconds for 1500 blocks
        - Full analytics (skip_time_bins=False): ~2-5 minutes for 1500 blocks
    """
    try:
        result = _rust_call(
            api.POST_SCHEDULE,
            schedule_name,
            schedule_json,
            visibility_json,
        )
        return result  # type: ignore[no-any-return]
    except Exception as e:
        raise ServerError(
            f"Failed to store schedule '{schedule_name}'",
            details={"schedule_name": schedule_name, "error": str(e)},
        ) from e


@with_retry(max_attempts=3, backoff_factor=1.5)
def list_schedules_db() -> list[ScheduleInfo]:
    """
    List available schedules using the Rust backend.

    Returns:
        List of schedule metadata dictionaries

    Raises:
        ServerError: If query fails
    """
    return _rust_call(api.LIST_SCHEDULES)

def get_schedule_blocks(schedule_id: int) -> list[Any]:
    """Fetch scheduling block models via PyO3 bindings."""
    return _rust_call("py_get_schedule_blocks", schedule_id)  # type: ignore[no-any-return]


def get_sky_map_data(
    *,
    schedule_id: int,
) -> SkyMapData:
    """
    Get complete sky map data with computed bins and metadata.

    Retrieves data from ETL analytics tables which contain pre-computed metrics.

    This is the main function for the sky map feature. It returns a SkyMapData
    object containing:
    - blocks: List of LightweightBlock objects with computed priority bins
    - priority_bins: List of PriorityBinInfo objects (4 bins with ranges and colors)
    - priority_min, priority_max: Priority range
    - ra_min, ra_max, dec_min, dec_max: Coordinate ranges
    - total_count, scheduled_count: Statistics
    - scheduled_time_min, scheduled_time_max: Time range for scheduled blocks

    All processing (querying, bin computation, statistics) is done in Rust
    for maximum performance. The frontend just needs to plot the data.
    """
    # Use Rust backend route for sky map data (consistent with other routes)
    return _rust_call(api.GET_SKY_MAP_DATA, schedule_id)


def get_visibility_map_data(
    *,
    schedule_id: int,
) -> VisibilityMapData:
    """
    Fetch visibility map metadata and block summaries from the Rust backend.

    Returns a VisibilityMapData object containing:
    - blocks: List of VisibilityBlockSummary with id, priority, num_visibility_periods, scheduled
    - priority_min/priority_max: Priority range for the schedule
    - total_count: Total blocks in the schedule
    - scheduled_count: Number of scheduled blocks
    """
    return _rust_call(api.GET_VISIBILITY_MAP_DATA, schedule_id)


def get_distribution_data(
    *,
    schedule_id: int,
) -> api.DistributionData:
    """
    Get complete distribution data with computed statistics.

    Retrieves data from ETL analytics tables which contain pre-computed metrics.

    This is the main function for the distributions feature. It returns a DistributionData
    object containing:
    - blocks: List of DistributionBlock objects with only required fields
    - priority_stats: DistributionStats with mean, median, std, min, max, sum
    - visibility_stats: DistributionStats for total_visibility_hours
    - requested_hours_stats: DistributionStats for requested_hours
    - total_count, scheduled_count, unscheduled_count: Counts
    - impossible_count: Number of blocks with zero visibility

    Note:
        Impossible blocks (zero visibility) are automatically excluded during ETL.

    All processing (querying, statistics computation) is done in Rust
    for maximum performance. The frontend just needs to plot the data.

    Args:
        schedule_id: Database ID of the schedule to load

    Returns:
        DistributionData object with all required data and pre-computed statistics
    """
    # Use Rust backend route for distribution data (ETL analytics path)
    return _rust_call(api.GET_DISTRIBUTION_DATA, schedule_id)


def get_schedule_timeline_data(
    *,
    schedule_id: int,
) -> Any:
    """
    Get complete schedule timeline data with computed statistics and metadata.

    This is the main function for the scheduled timeline feature. It returns a ScheduleTimelineData
    object containing:
    - blocks: List of ScheduleTimelineBlock objects with scheduled times and coordinates
    - priority_min, priority_max: Priority range
    - total_count, scheduled_count: Statistics
    - unique_months: List of unique month labels (YYYY-MM format)
    - dark_periods: List of (start_mjd, stop_mjd) tuples for dark periods

    All processing (querying, statistics computation, month extraction) is done in Rust
    for maximum performance. The frontend just needs to render the timeline.

    Args:
        schedule_id: Database ID of the schedule to load

    Returns:
        ScheduleTimelineData object with all required data and pre-computed metadata
    """
    # Use Rust backend route for schedule timeline data
    return _rust_call(api.GET_SCHEDULE_TIMELINE_DATA, schedule_id)


def get_insights_data(
    *,
    schedule_id: int,
) -> Any:
    """
    Get complete insights data with computed analytics and metadata.

    This is the main function for the insights feature. It returns an InsightsData
    object containing:
    - blocks: List of InsightsBlock objects with all required fields
    - metrics: AnalyticsMetrics with comprehensive statistics
    - correlations: List of CorrelationEntry objects with Spearman correlations
    - top_priority: List of TopObservation objects sorted by priority
    - top_visibility: List of TopObservation objects sorted by visibility hours
    - conflicts: List of ConflictRecord objects for overlapping scheduled observations
    - total_count, scheduled_count, impossible_count: Summary statistics

    Note:
        Impossible blocks (zero visibility) are automatically excluded during ETL.

    All processing (querying, analytics computation, correlations, conflict detection)
    is done in Rust for maximum performance. The frontend just needs to render the data.

    Args:
        schedule_id: Database ID of the schedule to load

    Returns:
        InsightsData object with all required data and pre-computed analytics
    """
    # Use Rust backend route for insights data
    return _rust_call(api.GET_INSIGHTS_DATA, schedule_id)


def get_trends_data(
    *,
    schedule_id: int,
    n_bins: int = 10,
    bandwidth: float = 0.3,
    n_smooth_points: int = 100,
) -> Any:
    """
    Get complete trends data with computed empirical rates, smoothed curves, and heatmap bins.

    This is the main function for the trends feature. It returns a TrendsData
    object containing:
    - blocks: List of TrendsBlock objects with scheduling data
    - metrics: TrendsMetrics with comprehensive statistics
    - by_priority: List of EmpiricalRatePoint objects for scheduling rates by priority
    - by_visibility: List of EmpiricalRatePoint objects binned by visibility hours
    - by_time: List of EmpiricalRatePoint objects binned by requested time
    - smoothed_visibility: List of SmoothedPoint objects for visibility trend
    - smoothed_time: List of SmoothedPoint objects for requested time trend
    - heatmap_bins: List of HeatmapBin objects for 2D visualization
    - priority_values: Unique priority values for filtering

    Note:
        Impossible blocks (zero visibility) are automatically excluded during ETL.

    All processing (querying, binning, smoothing, heatmap computation) is done in Rust
    for maximum performance. The frontend just needs to render the data.

    Args:
        schedule_id: Database ID of the schedule to load
        n_bins: Number of bins for continuous variables (default: 10)
        bandwidth: Bandwidth for smoothing as fraction of range (default: 0.3)
        n_smooth_points: Number of points in smoothed curves (default: 100)

    Returns:
        TrendsData object with all required data and pre-computed analytics
    """
    # Call Rust backend trends route directly
    return _rust_call(api.GET_TRENDS_DATA, schedule_id, n_bins, bandwidth, n_smooth_points)


def get_compare_data(
    *,
    current_schedule_id: int,
    comparison_schedule_id: int,
    current_name: str,
    comparison_name: str,
) -> Any:
    """
    Get complete comparison data for two schedules from the database.

    This is the main function for the schedule comparison feature. It returns a CompareData
    object containing:
    - current_blocks: List of CompareBlock objects from the current schedule
    - comparison_blocks: List of CompareBlock objects from the comparison schedule
    - current_stats: CompareStats with summary statistics for current schedule
    - comparison_stats: CompareStats with summary statistics for comparison schedule
    - common_ids: List of scheduling block IDs present in both schedules
    - only_in_current: List of IDs only in the current schedule
    - only_in_comparison: List of IDs only in the comparison schedule
    - scheduling_changes: List of SchedulingChange objects tracking status changes
    - current_name: Name of the current schedule
    - comparison_name: Name of the comparison schedule

    All processing (querying, comparison, statistics computation) is done in Rust
    for maximum performance. The frontend just needs to render the comparison.

    Args:
        current_schedule_id: Database ID of the current schedule
        comparison_schedule_id: Database ID of the schedule to compare with
        current_name: Display name for the current schedule
        comparison_name: Display name for the comparison schedule

    Returns:
        CompareData object with all required data and pre-computed comparisons
    """
    # Call Rust backend compare route directly
    return _rust_call(
        api.GET_COMPARE_DATA,
        current_schedule_id,
        comparison_schedule_id,
        current_name,
        comparison_name,
    )


def fetch_dark_periods_db(schedule_id: int) -> pd.DataFrame:
    """Fetch dark periods for a schedule (with global fallback)."""
    df_polars = _rust_call("py_fetch_dark_periods", schedule_id)
    return df_polars.to_pandas()  # type: ignore[no-any-return]


def fetch_possible_periods_db(schedule_id: int) -> pd.DataFrame:
    """Fetch possible/visibility periods for a schedule."""
    df_polars = _rust_call("py_fetch_possible_periods", schedule_id)
    return df_polars.to_pandas()  # type: ignore[no-any-return]


def _standardize_schedule_df(df: pd.DataFrame) -> pd.DataFrame:
    """Normalize column names and add required defaults for downstream processing."""
    if df is None or df.empty:
        return df

    rename_map = {
        "scheduling_block_id": "schedulingBlockId",
        "name": "targetName",
        "ra_deg": "raInDeg",
        "dec_deg": "decInDeg",
        "requested_duration_sec": "requestedDurationSec",
        "duration_min": "requested_duration_sec",
    }
    df = df.rename(columns={k: v for k, v in rename_map.items() if k in df.columns})

    # Ensure snake_case columns exist for internal use
    if "requestedDurationSec" in df.columns and "requested_duration_sec" not in df.columns:
        df["requested_duration_sec"] = df["requestedDurationSec"]

    # Convert duration minutes (legacy) to seconds
    if "duration_min" in df.columns and "requested_duration_sec" not in df.columns:
        df["requested_duration_sec"] = df["duration_min"] * 60.0

    # Mirror key columns in both naming styles
    if "scheduling_block_id" in df.columns and "schedulingBlockId" not in df.columns:
        df["schedulingBlockId"] = df["scheduling_block_id"]
    if "schedulingBlockId" in df.columns and "scheduling_block_id" not in df.columns:
        df["scheduling_block_id"] = df["schedulingBlockId"]
    if "requested_duration_sec" in df.columns and "requestedDurationSec" not in df.columns:
        df["requestedDurationSec"] = df["requested_duration_sec"]
    if "raInDeg" in df.columns and "ra_deg" not in df.columns:
        df["ra_deg"] = df["raInDeg"]
    if "decInDeg" in df.columns and "dec_deg" not in df.columns:
        df["dec_deg"] = df["decInDeg"]
    if "targetName" in df.columns and "name" not in df.columns:
        df["name"] = df["targetName"]

    # Derive simple metrics/defaults to satisfy downstream expectations
    if "requested_duration_sec" in df.columns:
        df["minObservationTimeInSec"] = df.get(
            "minObservationTimeInSec", df["requested_duration_sec"]
        )
        df["requested_hours"] = df["requested_duration_sec"] / 3600.0
    else:
        df["requested_duration_sec"] = None
        df["requestedDurationSec"] = None
        df["minObservationTimeInSec"] = None
        df["requested_hours"] = None

    defaults: dict[str, Any] = {
        "fixedStartTime": None,
        "fixedStopTime": None,
        "scheduled_period.start": None,
        "scheduled_period.stop": None,
        "visibility": None,
        "num_visibility_periods": 0,
        "total_visibility_hours": 0.0,
        "priority_bin": None,
        "elevation_range_deg": None,
        "minAzimuthAngleInDeg": None,
        "maxAzimuthAngleInDeg": None,
        "minElevationAngleInDeg": None,
        "maxElevationAngleInDeg": None,
        "scheduled_flag": False,
    }
    for col, default in defaults.items():
        if col not in df.columns:
            df[col] = default

    return df


def get_visibility_histogram(
    schedule_id: int,
    start: pd.Timestamp,
    end: pd.Timestamp,
    bin_duration_minutes: int,
    priority_range: tuple[int, int] | None = None,
    block_ids: list[int] | None = None,
) -> list[dict[str, Any]]:
    """
    Compute visibility histogram from the backend.

    Returns a list of time bins with counts of visible scheduling blocks.
    This function offloads heavy computation to Rust and returns only
    the minimal JSON-serializable payload needed for visualization.

    Performance Note:
        When no filters (priority_range, block_ids) are applied, uses pre-computed
        analytics bins which is ~10-100x faster. With filters, falls back to
        real-time computation which parses visibility JSON.

    Args:
        schedule_id: Schedule ID to analyze
        start: Start of time range
        end: End of time range
        bin_duration_minutes: Duration of each histogram bin in minutes
        priority_range: Optional (min, max) priority filter (inclusive)
        block_ids: Optional list of specific block IDs to include

    Returns:
        List of dicts with keys:
        - bin_start_unix: Start of bin (Unix timestamp)
        - bin_end_unix: End of bin (Unix timestamp)
        - count: Number of unique blocks visible in this bin

    Example:
        >>> from datetime import datetime, timezone
        >>> start = pd.Timestamp(datetime(2024, 1, 1, tzinfo=timezone.utc))
        >>> end = pd.Timestamp(datetime(2024, 1, 2, tzinfo=timezone.utc))
        >>> bins = get_visibility_histogram(
        ...     schedule_id=1,
        ...     start=start,
        ...     end=end,
        ...     bin_duration_minutes=60,
        ...     priority_range=(5, 10),
        ... )
        >>> print(f"Total bins: {len(bins)}")
        >>> print(f"Max visible: {max(b['count'] for b in bins)}")
    """
    # Convert pandas timestamps to Unix timestamps
    start_unix = int(start.timestamp())
    end_unix = int(end.timestamp())

    # Try fast analytics path when no filters are applied
    # (pre-computed bins are much faster but don't support filtering)
    if priority_range is None and block_ids is None:
        try:
            result = _rust_call(
                "py_get_visibility_histogram_analytics",
                schedule_id,
                start_unix,
                end_unix,
                bin_duration_minutes,
            )
            if result:
                logger.debug(f"Using pre-computed visibility histogram for schedule {schedule_id}")
                return result  # type: ignore[no-any-return]
        except Exception as e:
            logger.debug(f"Analytics histogram not available, falling back: {e}")

    # Extract priority min/max
    priority_min = priority_range[0] if priority_range else None
    priority_max = priority_range[1] if priority_range else None

    # Fall back to real-time computation (slower but supports filters)
    return _rust_call(  # type: ignore[no-any-return]
        api.GET_VISIBILITY_HISTOGRAM,
        schedule_id,
        start_unix,
        end_unix,
        bin_duration_minutes,
        priority_min,
        priority_max,
        block_ids,
    )


def get_schedule_time_range(schedule_id: int) -> tuple[pd.Timestamp, pd.Timestamp] | None:
    """
    Get the time range (min/max timestamps) for a schedule's visibility periods.

    This function queries the database to find the earliest and latest times
    across all visibility periods for the given schedule.

    Args:
        schedule_id: Schedule ID to analyze

    Returns:
        Tuple of (start_time, end_time) as pandas Timestamps, or None if no
        visibility periods exist or if schedule not found.

    Raises:
        RuntimeError: If database query fails

    Example:
        >>> time_range = get_schedule_time_range(schedule_id=1)
        >>> if time_range:
        ...     start, end = time_range
        ...     print(f"Schedule spans {(end - start).days} days")
        ... else:
        ...     print("No visibility periods found")
    """
    result = _rust_call(api.GET_SCHEDULE_TIME_RANGE, schedule_id)

    if result is None:
        return None

    start_unix, end_unix = result

    # Convert Unix timestamps to pandas timestamps (UTC)
    start_time = pd.Timestamp(start_unix, unit="s", tz="UTC")
    end_time = pd.Timestamp(end_unix, unit="s", tz="UTC")

    return start_time, end_time


def get_validation_report_data(schedule_id: int) -> ValidationReport:
    """
    Get validation report data for a schedule.

    This function retrieves validation results computed during the ETL Transform stage.
    Validation results are persisted to the database and include:
    - Blocks that are impossible to schedule (zero/insufficient visibility)
    - Validation errors (constraint violations, invalid values)
    - Warnings (potential data quality issues)

    Args:
        schedule_id: Schedule ID to analyze

    Returns:
        Dictionary containing:
        - metrics: Overall statistics (total_blocks, valid_blocks, etc.)
        - impossible_blocks: List of blocks that cannot be scheduled
        - validation_errors: List of data validation errors
        - validation_warnings: List of data warnings

    Example:
        >>> data = get_validation_report_data(schedule_id=1)
        >>> print(f"Found {len(data['impossible_blocks'])} impossible blocks")

    Note:
        Validation is performed automatically during schedule upload as part of the ETL process.
        Impossible blocks are automatically filtered from analytics queries.
    """
    # Get validation data from Rust backend (use exported constant)
    report = _rust_call(api.GET_VALIDATION_REPORT, schedule_id)
    # TODO: Assert fields exist on report object
    return report


__all__ = [
    "store_schedule_db",
    "list_schedules_db",
    "fetch_dark_periods_db",
    "fetch_possible_periods_db",
    "get_visibility_histogram",
    "get_schedule_time_range",
    "get_validation_report_data",
]
