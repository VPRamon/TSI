"""Database helpers that call directly into the Rust layer (tsi_rust)."""

from __future__ import annotations

from typing import Any, TYPE_CHECKING

import pandas as pd


if TYPE_CHECKING:
    from tsi_rust import SkyMapBlock, SkyMapData


def _import_rust():
    try:
        import tsi_rust  # type: ignore[import-not-found]
    except ImportError as e:
        raise RuntimeError(
            "Rust backend is not available. Please compile the extension before using database features."
        ) from e
    return tsi_rust


def _rust_call(method: str, *args: Any):
    rust = _import_rust()
    return getattr(rust, method)(*args)


def init_database() -> None:
    """Initialize the Rust-backed database pool."""
    _rust_call("py_init_database")


def db_health_check() -> bool:
    """Check database connectivity using the Python fallback."""
    try:
        from tsi.services.database_pyodbc import health_check

        return health_check()
    except Exception as e:
        raise RuntimeError(f"Database health check failed: {e}") from e


def store_schedule_db(
    schedule_name: str,
    schedule_json: str,
    visibility_json: str | None = None,
) -> dict[str, Any]:
    """Store a preprocessed schedule via the Rust bindings."""
    return _rust_call(
        "py_store_schedule",
        schedule_name,
        schedule_json,
        visibility_json,
    )


def fetch_schedule_db(
    schedule_id: int | None = None,
    schedule_name: str | None = None,
) -> pd.DataFrame:
    """Fetch a stored schedule as a pandas DataFrame."""
    if schedule_id is None and schedule_name is None:
        raise ValueError("Either schedule_id or schedule_name must be provided")

    try:
        df_polars = _rust_call("py_fetch_schedule", schedule_id, schedule_name)
        df = _standardize_schedule_df(df_polars.to_pandas())
        if df.empty:
            raise RuntimeError("Schedule not found")
        return df
    except Exception as rust_err:
        # Legacy/alternate schema fallback using the Python client
        fallback_df = _fetch_schedule_pyodbc(schedule_id=schedule_id, schedule_name=schedule_name)
        if fallback_df is not None and not fallback_df.empty:
            return fallback_df
        raise RuntimeError(
            f"Schedule not found (id={schedule_id}, name={schedule_name})"
        ) from rust_err


def list_schedules_db() -> list[dict[str, Any]]:
    """List available schedules using the Rust backend (same connection as fetch)."""
    try:
        return _rust_call("py_list_schedules")
    except Exception as e:
        raise RuntimeError(f"Failed to list schedules: {e}") from e


def get_schedule_from_backend(
    *,
    schedule_id: int | None = None,
    schedule_name: str | None = None,
):
    """Fetch a fully materialized Schedule model via PyO3 bindings."""
    return _rust_call("py_get_schedule", schedule_id, schedule_name)


def get_schedule_blocks(schedule_id: int) -> list[Any]:
    """Fetch scheduling block models via PyO3 bindings."""
    return _rust_call("py_get_schedule_blocks", schedule_id)


def get_sky_map_blocks(
    *,
    schedule_id: int | None = None,
    schedule_name: str | None = None,
) -> list[SkyMapBlock]:
    """
    Fetch lightweight sky map blocks optimized for visualization.
    
    This function returns minimal SkyMapBlock objects containing only:
    - id, priority, priority_bin
    - requested_duration_seconds
    - target_ra_deg, target_dec_deg
    - scheduled_period (if scheduled)
    
    This avoids the overhead of loading full Schedule objects with visibility
    periods and dark periods, making the sky map page much faster.
    """
    return _rust_call("py_get_sky_map_blocks", schedule_id, schedule_name)

def get_sky_map_data(
    *,
    schedule_id: int,
) -> SkyMapData:
    """
    Get complete sky map data with computed bins and metadata.
    
    This is the main function for the sky map feature. It returns a SkyMapData
    object containing:
    - blocks: List of SkyMapBlock objects with computed priority bins
    - priority_bins: List of PriorityBinInfo objects (4 bins with ranges and colors)
    - priority_min, priority_max: Priority range
    - ra_min, ra_max, dec_min, dec_max: Coordinate ranges
    - total_count, scheduled_count: Statistics
    - scheduled_time_min, scheduled_time_max: Time range for scheduled blocks
    
    All processing (querying, bin computation, statistics) is done in Rust
    for maximum performance. The frontend just needs to plot the data.
    """
    return _rust_call("py_get_sky_map_data", schedule_id)


def fetch_dark_periods_db(schedule_id: int) -> pd.DataFrame:
    """Fetch dark periods for a schedule (with global fallback)."""
    df_polars = _rust_call("py_fetch_dark_periods", schedule_id)
    return df_polars.to_pandas()


def fetch_possible_periods_db(schedule_id: int) -> pd.DataFrame:
    """Fetch possible/visibility periods for a schedule."""
    df_polars = _rust_call("py_fetch_possible_periods", schedule_id)
    return df_polars.to_pandas()


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
        df["minObservationTimeInSec"] = df.get("minObservationTimeInSec", df["requested_duration_sec"])
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


def _fetch_schedule_pyodbc(
    schedule_id: int | None = None,
    schedule_name: str | None = None,
) -> pd.DataFrame | None:
    """
    Legacy/backup fetch using the Python ODBC client.

    This covers databases that still store schedule_id directly on scheduling_blocks.
    """
    try:
        from tsi.services import database_pyodbc
    except Exception:
        return None

    attempts: list[pd.DataFrame | None] = []

    # Preferred path: through the junction table
    if schedule_id is not None:
        try:
            data = database_pyodbc.fetch_schedule_by_id(schedule_id)
            if data:
                attempts.append(pd.DataFrame(data))
        except Exception:
            pass

    if not attempts and schedule_name:
        try:
            data = database_pyodbc.fetch_schedule_by_name(schedule_name)
            if data:
                attempts.append(pd.DataFrame(data))
        except Exception:
            pass

    # Legacy schema: scheduling_blocks has schedule_id column
    if not attempts and schedule_id is not None:
        try:
            with database_pyodbc.get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute(
                    """
                    SELECT sb.scheduling_block_id, t.name, t.ra_deg, t.dec_deg,
                           sb.requested_duration_sec, sb.priority
                    FROM dbo.scheduling_blocks sb
                    JOIN dbo.targets t ON sb.target_id = t.target_id
                    WHERE sb.schedule_id = ?
                    """,
                    (schedule_id,),
                )
                rows = cursor.fetchall()
                if rows:
                    attempts.append(
                        pd.DataFrame(
                            [
                                {
                                    "scheduling_block_id": r[0],
                                    "name": r[1],
                                    "ra_deg": r[2],
                                    "dec_deg": r[3],
                                    "requested_duration_sec": r[4],
                                    "priority": r[5],
                                }
                                for r in rows
                            ]
                        )
                    )
        except Exception:
            pass

    for df in attempts:
        if df is not None and not df.empty:
            return _standardize_schedule_df(df)

    return None


__all__ = [
    "init_database",
    "db_health_check",
    "store_schedule_db",
    "fetch_schedule_db",
    "list_schedules_db",
    "fetch_dark_periods_db",
    "fetch_possible_periods_db",
]
